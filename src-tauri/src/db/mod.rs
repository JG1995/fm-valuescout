pub mod migrations;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{
    Arc, Condvar, LockResult, Mutex, MutexGuard, OnceLock, PoisonError, TryLockError, TryLockResult,
};
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};
use tauri::{AppHandle, Manager};

/// SQLite filename under `app_data_dir`.
///
/// The compact-metric generation uses a fresh file so the legacy `app.db` is
/// never opened, migrated, or converted.
pub const APP_DB_FILE: &str = "app-v2.db";

const READER_COUNT: usize = 4;
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
const WRITER_CACHE_KIB: i64 = 256 * 1024;
const READER_CACHE_KIB: i64 = 64 * 1024;
const JOURNAL_SIZE_LIMIT_BYTES: i64 = 64 * 1024 * 1024;
/// The bundled SQLite's default `SQLITE_MAX_MMAP_SIZE`; SQLite clamps larger requests to it.
const MMAP_SIZE_BYTES: i64 = 0x7fff_0000;

/// The application database: one writer and, for a database file, a pool of read-only
/// readers. Clones share both.
///
/// Every write goes through `.0`, because releasing that lock advances the data generation.
#[derive(Clone)]
pub struct Db(pub WriterLock, Option<Arc<ReaderPool>>);

impl Db {
    /// Wraps a test connection as the writer, without readers: reads fall back to the writer.
    #[cfg(test)]
    pub fn new(conn: Connection) -> Self {
        Self(WriterLock::new(conn, Arc::default()), None)
    }

    /// Hands out a read connection inside one deferred read transaction, so every statement
    /// of one request sees the same committed data. Blocks while every reader is in use.
    pub fn read(&self) -> Result<Reader<'_>, String> {
        let Some(pool) = self.1.as_deref() else {
            let conn = self
                .0
                .lock()
                .map_err(|_| "database lock poisoned".to_string())?;
            let generation = self.0.generation.load(Ordering::Acquire);
            return Ok(Reader {
                source: ReadSource::Writer(conn),
                generation,
            });
        };

        let conn = pool.acquire()?;
        // Read before the transaction takes its snapshot, so a result is never labeled with
        // a generation newer than the data it saw.
        let generation = self.0.generation.load(Ordering::Acquire);
        let reader = Reader {
            source: ReadSource::Pooled {
                conn: Some(conn),
                pool,
            },
            generation,
        };
        reader
            .execute_batch("BEGIN DEFERRED")
            .map_err(|error| error.to_string())?;
        Ok(reader)
    }
}

/// The writer connection behind a lock. Its guard advances the data generation when the
/// connection changed data while the guard was held.
#[derive(Clone)]
pub struct WriterLock {
    conn: Arc<Mutex<Connection>>,
    generation: Arc<AtomicU64>,
}

impl WriterLock {
    fn new(conn: Connection, generation: Arc<AtomicU64>) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            generation,
        }
    }

    pub fn lock(&self) -> LockResult<WriterGuard<'_>> {
        match self.conn.lock() {
            Ok(conn) => Ok(self.guard(conn)),
            Err(poisoned) => Err(PoisonError::new(self.guard(poisoned.into_inner()))),
        }
    }

    pub fn try_lock(&self) -> TryLockResult<WriterGuard<'_>> {
        match self.conn.try_lock() {
            Ok(conn) => Ok(self.guard(conn)),
            Err(TryLockError::Poisoned(poisoned)) => Err(TryLockError::Poisoned(PoisonError::new(
                self.guard(poisoned.into_inner()),
            ))),
            Err(TryLockError::WouldBlock) => Err(TryLockError::WouldBlock),
        }
    }

    fn guard<'a>(&'a self, conn: MutexGuard<'a, Connection>) -> WriterGuard<'a> {
        WriterGuard {
            changes_at_lock: conn.total_changes(),
            conn,
            generation: &self.generation,
        }
    }
}

pub struct WriterGuard<'a> {
    conn: MutexGuard<'a, Connection>,
    generation: &'a AtomicU64,
    changes_at_lock: u64,
}

impl Deref for WriterGuard<'_> {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        &self.conn
    }
}

impl DerefMut for WriterGuard<'_> {
    fn deref_mut(&mut self) -> &mut Connection {
        &mut self.conn
    }
}

impl Drop for WriterGuard<'_> {
    fn drop(&mut self) {
        // Transactions borrow the guard, so they have committed or rolled back by now. The
        // advance happens before the mutex unlocks, so the next writer sees it.
        if self.conn.total_changes() != self.changes_at_lock {
            self.generation.fetch_add(1, Ordering::Release);
        }
    }
}

/// Read-only connections handed out one request at a time.
struct ReaderPool {
    path: PathBuf,
    /// `None` is a free slot whose connection was discarded; the next acquire reopens it.
    idle: Mutex<Vec<Option<Connection>>>,
    returned: Condvar,
}

impl ReaderPool {
    fn open(path: &Path) -> Result<Self, String> {
        let idle = (0..READER_COUNT)
            .map(|_| open_reader(path).map(Some))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            path: path.to_path_buf(),
            idle: Mutex::new(idle),
            returned: Condvar::new(),
        })
    }

    fn acquire(&self) -> Result<Connection, String> {
        let mut idle = self.idle.lock().unwrap_or_else(PoisonError::into_inner);
        let slot = loop {
            if let Some(slot) = idle.pop() {
                break slot;
            }
            idle = self
                .returned
                .wait(idle)
                .unwrap_or_else(PoisonError::into_inner);
        };
        drop(idle);

        match slot {
            Some(conn) => Ok(conn),
            None => open_reader(&self.path).inspect_err(|_| self.release(None)),
        }
    }

    fn release(&self, conn: Option<Connection>) {
        self.idle
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(conn);
        self.returned.notify_one();
    }
}

enum ReadSource<'a> {
    Pooled {
        /// Present until drop returns it to the pool.
        conn: Option<Connection>,
        pool: &'a ReaderPool,
    },
    Writer(WriterGuard<'a>),
}

/// One read request's connection. It dereferences to the connection, whose
/// `get_interrupt_handle` can stop the request's statements.
pub struct Reader<'a> {
    source: ReadSource<'a>,
    generation: u64,
}

impl Reader<'_> {
    /// The data generation read before this request's read transaction began.
    #[cfg_attr(not(test), allow(dead_code))] // PR `cached-table-results` keys its caches by it.
    pub fn generation(&self) -> u64 {
        self.generation
    }
}

impl Deref for Reader<'_> {
    type Target = Connection;

    fn deref(&self) -> &Connection {
        match &self.source {
            ReadSource::Pooled { conn, .. } => {
                conn.as_ref().expect("reader connection is held until drop")
            }
            ReadSource::Writer(conn) => conn,
        }
    }
}

impl Drop for Reader<'_> {
    fn drop(&mut self) {
        if let ReadSource::Pooled { conn, pool } = &mut self.source {
            // Statements borrow the reader, so none is running. A connection that cannot
            // leave its transaction would pin an old snapshot, so it is discarded instead.
            let conn = conn.take().filter(|conn| {
                conn.is_autocommit()
                    || (conn.execute_batch("ROLLBACK").is_ok() && conn.is_autocommit())
            });
            pool.release(conn);
        }
    }
}

pub fn resolve_db_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;

    Ok(app_data_dir.join(APP_DB_FILE))
}

pub fn open(db_path: &Path) -> Result<Db, String> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let conn = Connection::open(db_path).map_err(|error| error.to_string())?;
    configure_writer(&conn)?;
    migrations::apply(&conn).map_err(|error| error.to_string())?;

    let path = std::fs::canonicalize(db_path).map_err(|error| error.to_string())?;
    let readers = ReaderPool::open(&path)?;
    let generation = registered_generation(path);
    Ok(Db(
        WriterLock::new(conn, generation),
        Some(Arc::new(readers)),
    ))
}

fn configure_writer(conn: &Connection) -> Result<(), String> {
    let journal_mode: String = conn
        .pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if !journal_mode.eq_ignore_ascii_case("wal") {
        return Err(format!("database stayed in {journal_mode} journal mode"));
    }
    conn.pragma_update(None, "synchronous", "NORMAL")
        .and_then(|()| conn.pragma_update(None, "journal_size_limit", JOURNAL_SIZE_LIMIT_BYTES))
        .and_then(|()| configure_connection(conn, WRITER_CACHE_KIB))
        .and_then(|()| conn.pragma_update(None, "foreign_keys", true))
        .map_err(|error| error.to_string())
}

fn open_reader(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_URI
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| error.to_string())?;
    configure_connection(&conn, READER_CACHE_KIB)
        .and_then(|()| conn.pragma_update(None, "query_only", true))
        .map_err(|error| error.to_string())?;
    Ok(conn)
}

fn configure_connection(conn: &Connection, cache_kib: i64) -> rusqlite::Result<()> {
    conn.busy_timeout(BUSY_TIMEOUT)?;
    // A negative cache size is in KiB.
    conn.pragma_update(None, "cache_size", -cache_kib)?;
    conn.pragma_update(None, "mmap_size", MMAP_SIZE_BYTES)?;
    conn.pragma_update(None, "temp_store", "MEMORY")
}

/// Copies the WAL into the database file and truncates it. The app runs it at exit.
pub fn checkpoint_wal(conn: &Connection) -> Result<(), String> {
    let blocked: i64 = conn
        .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if blocked != 0 {
        return Err("a reader blocked the WAL checkpoint".to_string());
    }
    Ok(())
}

/// Data generations by canonical database path. Reopening a file reuses its counter, so one
/// file never hands out the same generation twice within a process.
fn generations() -> &'static Mutex<HashMap<PathBuf, Arc<AtomicU64>>> {
    static GENERATIONS: OnceLock<Mutex<HashMap<PathBuf, Arc<AtomicU64>>>> = OnceLock::new();
    GENERATIONS.get_or_init(Mutex::default)
}

fn registered_generation(path: PathBuf) -> Arc<AtomicU64> {
    generations()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .entry(path)
        .or_default()
        .clone()
}

/// The current data generation of the file behind `conn`, or `None` for a connection
/// without a file or one whose file `open` never registered.
#[cfg_attr(not(test), allow(dead_code))] // PR `cached-table-results` reads it at check time.
pub fn data_generation(conn: &Connection) -> Option<u64> {
    let path = conn.path().filter(|path| !path.is_empty())?;
    let path = std::fs::canonicalize(path).ok()?;
    generations()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&path)
        .map(|generation| generation.load(Ordering::Acquire))
}

/// Runs `read` on another thread while the writer holds an open write transaction, and fails
/// unless it finishes, which a read that waited for the writer cannot do.
#[cfg(test)]
pub(crate) fn read_while_writer_is_held<T: Send + 'static>(
    db: &Db,
    read: impl FnOnce(&Db) -> T + Send + 'static,
) -> T {
    let writer = db.0.lock().expect("lock writer");
    writer
        .execute_batch("BEGIN IMMEDIATE")
        .expect("open write transaction");
    let reader_db = db.clone();
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || sender.send(read(&reader_db)));
    let value = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("the read finishes while the writer holds a write transaction");
    writer
        .execute_batch("ROLLBACK")
        .expect("end write transaction");
    value
}

/// Opens a database file whose current snapshot holds the golden dump.
#[cfg(test)]
pub(crate) fn open_with_golden_snapshot() -> (tempfile::TempDir, Db) {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db = open(&temp_dir.path().join(APP_DB_FILE)).expect("open db");
    let dump = temp_dir.path().join("golden.json");
    std::fs::write(
        &dump,
        include_str!("../features/memory_read/fixtures/golden_dump_v9.json"),
    )
    .expect("write dump");
    crate::features::snapshot::ingest::ingest_dump_file(
        &mut db.0.lock().expect("lock writer"),
        &dump,
    )
    .expect("ingest golden dump");
    (temp_dir, db)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opened() -> (tempfile::TempDir, Db) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = open(&temp_dir.path().join(APP_DB_FILE)).expect("open db");
        db.0.lock()
            .expect("lock writer")
            .execute_batch(
                "CREATE TABLE probe (value INTEGER NOT NULL);
                 INSERT INTO probe (value) VALUES (1);",
            )
            .expect("create probe table");
        (temp_dir, db)
    }

    fn probe_sum(conn: &Connection) -> i64 {
        conn.query_row("SELECT SUM(value) FROM probe", [], |row| row.get(0))
            .expect("read probe")
    }

    #[test]
    fn open_enables_foreign_key_enforcement() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = open(&temp_dir.path().join("foreign-keys-test.db")).expect("open db");
        let conn = db.0.lock().expect("lock db");

        let foreign_keys: i32 = conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .expect("read foreign_keys");
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn app_database_filename_targets_only_the_fresh_v2_generation() {
        assert_eq!(APP_DB_FILE, "app-v2.db");
        assert_ne!(APP_DB_FILE, "app.db");
    }

    #[test]
    fn opening_the_v2_database_applies_the_compact_metrics_schema() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = open(&temp_dir.path().join(APP_DB_FILE)).expect("open fresh v2 database");
        let conn = db.0.lock().expect("lock db");

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("read schema version");
        assert_eq!(version, migrations::latest_version());

        let compact_tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master
                 WHERE type = 'table'
                   AND name IN ('player_role_metrics', 'staff_role_metrics')",
                [],
                |row| row.get(0),
            )
            .expect("count compact metric tables");
        assert_eq!(compact_tables, 2);
    }

    #[test]
    fn database_stays_in_wal_mode_after_reopening() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join(APP_DB_FILE);
        drop(open(&path).expect("open db"));

        let reopened = Connection::open(&path).expect("reopen without configuration");
        let journal_mode: String = reopened
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .expect("read journal mode");
        assert_eq!(journal_mode, "wal");
    }

    #[test]
    fn reader_sees_committed_data_while_the_writer_holds_a_write_transaction() {
        let (_temp_dir, db) = opened();
        let writer = db.0.lock().expect("lock writer");
        writer
            .execute_batch("BEGIN IMMEDIATE; UPDATE probe SET value = 2;")
            .expect("open write transaction");

        let reader_db = db.clone();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let reader = reader_db.read().expect("acquire reader");
            sender.send(probe_sum(&reader)).expect("send result");
        });
        let seen = receiver
            .recv_timeout(Duration::from_secs(10))
            .expect("reader finishes while the write transaction is open");

        assert_eq!(seen, 1);
        writer.execute_batch("COMMIT").expect("commit");
    }

    #[test]
    fn statements_of_one_read_request_see_one_snapshot_across_a_writer_commit() {
        let (_temp_dir, db) = opened();
        let reader = db.read().expect("acquire reader");
        assert_eq!(probe_sum(&reader), 1);

        db.0.lock()
            .expect("lock writer")
            .execute("UPDATE probe SET value = 5", [])
            .expect("commit change");

        assert_eq!(probe_sum(&reader), 1);
        drop(reader);
        assert_eq!(probe_sum(&db.read().expect("acquire reader")), 5);
    }

    #[test]
    fn writer_guard_advances_the_generation_only_after_a_change() {
        let (_temp_dir, db) = opened();
        let start = data_generation(&db.read().expect("acquire reader")).expect("registered");

        {
            let writer = db.0.lock().expect("lock writer");
            writer
                .query_row("SELECT COUNT(*) FROM probe", [], |row| row.get::<_, i64>(0))
                .expect("read without changing");
        }
        assert_eq!(db.read().expect("acquire reader").generation(), start);

        {
            let mut writer = db.0.lock().expect("lock writer");
            let tx = writer.transaction().expect("begin");
            tx.execute("UPDATE probe SET value = 3", [])
                .expect("change data");
            tx.commit().expect("commit");
            assert_eq!(
                db.0.generation.load(Ordering::Acquire),
                start,
                "the generation waits for the guard to drop"
            );
        }
        let reader = db.read().expect("acquire reader");
        assert_eq!(reader.generation(), start + 1);
        assert_eq!(data_generation(&reader), Some(start + 1));
    }

    #[test]
    fn reopening_a_file_continues_its_generation() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join(APP_DB_FILE);
        let first = open(&path).expect("open db");
        first
            .0
            .lock()
            .expect("lock writer")
            .execute_batch("CREATE TABLE probe (value INTEGER); INSERT INTO probe VALUES (1);")
            .expect("change data");
        let advanced = first.read().expect("acquire reader").generation();
        drop(first);

        let reopened = open(&path).expect("reopen db");
        assert_eq!(
            reopened.read().expect("acquire reader").generation(),
            advanced
        );
    }

    #[test]
    fn reader_rejects_writes() {
        let (_temp_dir, db) = opened();
        let reader = db.read().expect("acquire reader");

        assert!(reader.execute("UPDATE probe SET value = 9", []).is_err());
        assert!(reader.is_readonly("main").expect("read-only state"));
        drop(reader);
        assert_eq!(probe_sum(&db.read().expect("acquire reader")), 1);
    }

    #[test]
    fn a_failed_read_returns_its_connection_outside_a_transaction() {
        let (_temp_dir, db) = opened();
        let failed = (0..READER_COUNT)
            .map(|_| db.read().expect("acquire reader"))
            .collect::<Vec<_>>();
        for reader in &failed {
            assert_eq!(probe_sum(reader), 1);
            assert!(reader.execute_batch("SELECT missing FROM probe").is_err());
        }
        drop(failed);

        db.0.lock()
            .expect("lock writer")
            .execute("UPDATE probe SET value = 7", [])
            .expect("commit change");

        let readers = (0..READER_COUNT)
            .map(|_| db.read().expect("acquire reader"))
            .collect::<Vec<_>>();
        for reader in &readers {
            assert_eq!(probe_sum(reader), 7);
        }
    }

    #[test]
    fn checkpoint_empties_the_wal_file() {
        let (temp_dir, db) = opened();
        let wal = temp_dir.path().join(format!("{APP_DB_FILE}-wal"));
        assert!(std::fs::metadata(&wal).expect("wal file").len() > 0);

        checkpoint_wal(&db.0.lock().expect("lock writer")).expect("checkpoint");

        assert_eq!(std::fs::metadata(&wal).expect("wal file").len(), 0);
    }
}
