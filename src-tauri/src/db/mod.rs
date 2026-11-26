pub mod migrations;
pub mod result_cache;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{
    Arc, Condvar, LockResult, Mutex, MutexGuard, OnceLock, PoisonError, TryLockError, TryLockResult,
};
use std::time::{Duration, Instant};

use rusqlite::{Connection, OpenFlags};
use tauri::{AppHandle, Manager};

/// SQLite filename under `app_data_dir`.
///
/// The compact-metric generation uses a fresh file so the legacy `app.db` is
/// never opened, migrated, or converted.
pub const APP_DB_FILE: &str = "app-v2.db";

const READER_COUNT: usize = 4;
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);
/// The writer's page cache while a write holds it, for the pages a transaction changes.
const WRITING_CACHE_KIB: i64 = 256 * 1024;
/// Every connection's page cache otherwise. Reads take their pages from the memory-mapped
/// file, so a cache only holds pages that are still in the WAL.
///
/// The bundled SQLite is built with `SQLITE_ENABLE_MEMORY_MANAGEMENT`, which makes all
/// connections share one cache budget: any of them may use the sum of their sizes. The sum,
/// not one connection's size, is therefore what bounds the memory.
const IDLE_CACHE_KIB: i64 = 8 * 1024;
const JOURNAL_SIZE_LIMIT_BYTES: i64 = 64 * 1024 * 1024;
/// How much of the database file a connection maps. `src-tauri/.cargo/config.toml` raises the
/// bundled SQLite's `SQLITE_MAX_MMAP_SIZE` from its 2 GiB default to this; a build without it
/// clamps the mapping, and reads past it go through the page cache.
const MMAP_SIZE_BYTES: i64 = 64 * 1024 * 1024 * 1024;
/// The migration that moved the season statistics from a JSON object into columns, which
/// left most of the pages of a large database file free.
const SEASON_STATISTICS_REWRITE_VERSION: i32 = 66;
/// The least free space in the database file that is worth rewriting the file to return.
const WORTH_RETURNING_BYTES: i64 = 256 * 1024 * 1024;

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

    /// The current data generation. A result cache looks up with it before any reader is held.
    pub fn generation(&self) -> u64 {
        self.0.generation.load(Ordering::Acquire)
    }

    /// Folds the WAL into the database file and empties it. `writer` is this database's
    /// locked writer.
    ///
    /// A checkpoint completes only while no reader holds a WAL read lock. On Windows, SQLite
    /// 3.53.2 can keep a pooled reader's lock after its read transaction ended when several
    /// readers run at once, and every later checkpoint is then blocked for as long as that
    /// connection stays open. Closing a connection releases its locks. So this first copies
    /// what the running readers allow, then waits up to `reader_wait` for the read requests
    /// that are running, closes every reader, and checkpoints with none open. Read requests
    /// wait meanwhile and reopen their readers afterwards.
    ///
    /// Fails when a read request outlasted `reader_wait` or another process blocked the
    /// checkpoint. The WAL then stays, and reads stay correct.
    pub fn checkpoint(&self, writer: &Connection, reader_wait: Duration) -> Result<(), String> {
        // Result ignored: the checkpoint below reports what is still blocked.
        let _ = writer.query_row("PRAGMA wal_checkpoint(PASSIVE)", [], |_| Ok(()));
        let _closed = match self.1.as_deref() {
            Some(pool) => Some(pool.close_readers(reader_wait)?),
            None => None,
        };
        checkpoint_wal(writer)
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
        // The boxed connection keeps its address while the reader moves, so the record stays
        // findable from the connection.
        reader_generations().insert(connection_key(&reader), generation);
        reader
            .execute_batch("BEGIN DEFERRED")
            .map_err(|error| error.to_string())?;
        Ok(reader)
    }

    /// Closes the readers that no request holds, once no read request has ended for
    /// `idle_for`. It does nothing more until another request has ended; the next requests
    /// reopen their readers.
    ///
    /// A reader maps the database file, and every mapped page it touched counts towards the
    /// process's working set until the reader closes, although the page belongs to the
    /// operating system's file cache.
    pub fn close_idle_readers(&self, idle_for: Duration) {
        if let Some(pool) = self.1.as_deref() {
            pool.close_idle(idle_for);
        }
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
        // Result ignored: a cache that stays small only makes a large write slower.
        let _ = set_cache_size(&conn, WRITING_CACHE_KIB);
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
            drop_mapping(&self.conn);
        }
        release_write_cache(&self.conn);
    }
}

/// Read-only connections handed out one request at a time.
struct ReaderPool {
    path: PathBuf,
    state: Mutex<PoolState>,
    /// Signalled when a reader returns and when a checkpoint lets requests in again.
    changed: Condvar,
}

struct PoolState {
    /// `None` is a free slot whose connection was discarded; the next acquire reopens it.
    /// Boxed, so a connection's address identifies it for `data_generation`.
    idle: Vec<Option<Box<Connection>>>,
    /// A checkpoint is closing the readers: no request gets one until it ends.
    closing: bool,
    /// When a read request last ended, until `close_idle` has acted on it.
    last_release: Option<Instant>,
}

/// Keeps a pool's readers closed and its requests waiting until dropped.
struct ClosedReaders<'a>(&'a ReaderPool);

impl Drop for ClosedReaders<'_> {
    fn drop(&mut self) {
        self.0.lock().closing = false;
        self.0.changed.notify_all();
    }
}

impl ReaderPool {
    fn open(path: &Path) -> Result<Self, String> {
        let idle = (0..READER_COUNT)
            .map(|_| open_reader(path).map(|conn| Some(Box::new(conn))))
            .collect::<Result<_, _>>()?;
        Ok(Self {
            path: path.to_path_buf(),
            state: Mutex::new(PoolState {
                idle,
                closing: false,
                last_release: None,
            }),
            changed: Condvar::new(),
        })
    }

    fn lock(&self) -> MutexGuard<'_, PoolState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn acquire(&self) -> Result<Box<Connection>, String> {
        let mut state = self.lock();
        let slot = loop {
            if !state.closing {
                if let Some(slot) = state.idle.pop() {
                    break slot;
                }
            }
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        };
        drop(state);

        match slot {
            Some(conn) => Ok(conn),
            None => open_reader(&self.path)
                .map(Box::new)
                .inspect_err(|_| self.release(None)),
        }
    }

    fn release(&self, conn: Option<Box<Connection>>) {
        let mut state = self.lock();
        state.idle.push(conn);
        state.last_release = Some(Instant::now());
        drop(state);
        // A request and a checkpoint may both wait, and only one of them can go on.
        self.changed.notify_all();
    }

    /// Closes the readers no request holds, when the last read request ended at least
    /// `idle_for` ago and nothing has closed them since. Each slot stays free and reopens on
    /// demand.
    fn close_idle(&self, idle_for: Duration) {
        let mut state = self.lock();
        if !state
            .last_release
            .is_some_and(|released| released.elapsed() >= idle_for)
        {
            return;
        }
        state.last_release = None;
        let closed = state
            .idle
            .iter_mut()
            .filter_map(Option::take)
            .collect::<Vec<_>>();
        // Closed after the pool lock is released: unmapping a large file takes a while, and
        // no request should wait for it.
        drop(state);
        drop(closed);
    }

    /// Closes every reader once the requests holding one have returned it, waiting up to
    /// `wait` for them. No request gets a reader until the result is dropped.
    fn close_readers(&self, wait: Duration) -> Result<ClosedReaders<'_>, String> {
        let deadline = Instant::now() + wait;
        let mut state = self.lock();
        state.closing = true;
        let closed = ClosedReaders(self);
        while state.idle.len() < READER_COUNT {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                drop(state);
                return Err("read requests were still running".to_string());
            }
            state = self
                .changed
                .wait_timeout(state, remaining)
                .unwrap_or_else(PoisonError::into_inner)
                .0;
        }
        state.idle.fill_with(|| None);
        drop(state);
        Ok(closed)
    }
}

enum ReadSource<'a> {
    Pooled {
        /// Present until drop returns it to the pool.
        conn: Option<Box<Connection>>,
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
            if let Some(conn) = conn.as_deref() {
                reader_generations().remove(&connection_key(conn));
            }
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
    // No reader is open yet, so nothing in this process can block these. The first folds in a
    // WAL that an earlier session left behind, so the migrations and every later read use the
    // database file; the second folds in what the migrations wrote.
    checkpoint_at_open(&conn);
    let version_before: i32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|error| error.to_string())?;
    migrations::apply(&conn).map_err(|error| error.to_string())?;
    checkpoint_at_open(&conn);
    if (1..SEASON_STATISTICS_REWRITE_VERSION).contains(&version_before) {
        return_free_space(&conn, WORTH_RETURNING_BYTES);
    }

    release_write_cache(&conn);

    let path = std::fs::canonicalize(db_path).map_err(|error| error.to_string())?;
    let readers = ReaderPool::open(&path)?;
    let generation = registered_generation(path);
    Ok(Db(
        WriterLock::new(conn, generation),
        Some(Arc::new(readers)),
    ))
}

/// A checkpoint that never fails the open: another process may hold the database.
fn checkpoint_at_open(conn: &Connection) {
    let started = Instant::now();
    match checkpoint_wal(conn) {
        Ok(()) if started.elapsed() >= Duration::from_secs(1) => log::info!(
            "database checkpoint at startup took {} s",
            started.elapsed().as_secs()
        ),
        Ok(()) => {}
        Err(error) => log::warn!("database checkpoint failed at startup: {error}"),
    }
}

/// Rewrites the database file without its free pages when they hold at least `worth_bytes`,
/// which shrinks the file. SQLite otherwise keeps a file at its largest size and only reuses
/// the pages. No other connection may be open: a file that one of them maps cannot shrink.
///
/// The rewrite takes minutes for gigabytes, and needs about twice the remaining data in free
/// disk space: once in the temporary directory and once beside the database. A failure
/// leaves the database as it was, free pages included.
fn return_free_space(conn: &Connection, worth_bytes: i64) {
    let pragma = |name: &str| conn.pragma_query_value(None, name, |row| row.get::<_, i64>(0));
    let free_bytes = pragma("freelist_count")
        .and_then(|pages| Ok(pages * pragma("page_size")?))
        .unwrap_or(0);
    if free_bytes < worth_bytes {
        return;
    }
    log::info!(
        "returning {} MiB of free database pages to the file system",
        free_bytes / (1024 * 1024)
    );
    let started = Instant::now();
    // This connection's own mapping would keep the file from shrinking too, and
    // `temp_store = MEMORY` would hold the rewritten copy in memory.
    let rewritten = conn.execute_batch("PRAGMA mmap_size = 0; PRAGMA temp_store = FILE; VACUUM");
    let restored = conn.pragma_update(None, "temp_store", "MEMORY");
    // The rewrite is in the WAL: the file shrinks when the checkpoint folds it in.
    checkpoint_at_open(conn);
    drop_mapping(conn);
    match rewritten.and(restored) {
        Ok(()) => log::info!(
            "database free pages returned in {} s",
            started.elapsed().as_secs()
        ),
        Err(error) => log::warn!("database free pages were not returned: {error}"),
    }
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
        .and_then(|()| configure_connection(conn, WRITING_CACHE_KIB))
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
    configure_connection(&conn, IDLE_CACHE_KIB)
        .and_then(|()| conn.pragma_update(None, "query_only", true))
        .map_err(|error| error.to_string())?;
    Ok(conn)
}

fn configure_connection(conn: &Connection, cache_kib: i64) -> rusqlite::Result<()> {
    conn.busy_timeout(BUSY_TIMEOUT)?;
    set_cache_size(conn, cache_kib)?;
    conn.pragma_update(None, "mmap_size", MMAP_SIZE_BYTES)?;
    conn.pragma_update(None, "temp_store", "MEMORY")
}

fn set_cache_size(conn: &Connection, cache_kib: i64) -> rusqlite::Result<()> {
    // A negative cache size is in KiB.
    conn.pragma_update(None, "cache_size", -cache_kib)
}

/// Shrinks the writer's page cache to its idle size and frees the pages a write left in it,
/// which SQLite would otherwise keep until the connection closes.
fn release_write_cache(conn: &Connection) {
    // Results ignored: a cache that stays large costs memory, never correctness.
    let _ = set_cache_size(conn, IDLE_CACHE_KIB);
    let _ = conn.execute_batch("PRAGMA shrink_memory");
}

/// Drops the writer's mapping of the database file after it wrote or checkpointed. SQLite
/// maps the file again on the writer's next read, at the size the file then has.
///
/// A connection keeps its mapping until it sees another connection's change, so the writer's
/// own writes and checkpoints would leave it mapping the file's old size. The pages added
/// since would then come through slower file reads, which a later large write pays for when
/// it reads the rows it replaces.
fn drop_mapping(conn: &Connection) {
    // Results ignored: a stale mapping costs speed, never correctness.
    let _ = conn.pragma_update(None, "mmap_size", 0);
    let _ = conn.pragma_update(None, "mmap_size", MMAP_SIZE_BYTES);
}

/// Copies the WAL into the database file and truncates it, unless a reader blocks that.
/// `Db::checkpoint` closes the app's readers first.
pub fn checkpoint_wal(conn: &Connection) -> Result<(), String> {
    let blocked: i64 = conn
        .query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if blocked != 0 {
        return Err("a reader blocked the WAL checkpoint".to_string());
    }
    drop_mapping(conn);
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

/// Generations that pooled readers read before their read transactions began, by
/// `connection_key`. A reader keeps its entry until it returns its connection to the pool.
fn reader_generations() -> MutexGuard<'static, HashMap<usize, u64>> {
    static READER_GENERATIONS: OnceLock<Mutex<HashMap<usize, u64>>> = OnceLock::new();
    READER_GENERATIONS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn connection_key(conn: &Connection) -> usize {
    std::ptr::from_ref(conn) as usize
}

/// The data generation that `conn`'s data belongs to. A read-only connection gets the
/// generation its `Db::read` request read before its read transaction began, or `None`
/// outside such a request. Any other connection gets its file's current generation, or
/// `None` without a file or for a file that `open` never registered.
pub fn data_generation(conn: &Connection) -> Option<u64> {
    if conn.is_readonly(rusqlite::MAIN_DB).ok()? {
        return reader_generations().get(&connection_key(conn)).copied();
    }
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
        include_str!("../features/memory_read/fixtures/golden_dump_v10.json"),
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

    fn open_idle_readers(db: &Db) -> usize {
        let pool = db.1.as_deref().expect("reader pool");
        let state = pool.lock();
        state.idle.iter().flatten().count()
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
    fn every_connection_maps_the_whole_database_file() {
        let (_temp_dir, db) = opened();
        let mapped = |conn: &Connection| -> i64 {
            conn.pragma_query_value(None, "mmap_size", |row| row.get(0))
                .expect("read mmap size")
        };

        // SQLite reports the size it applied. A smaller one means that the bundled SQLite was
        // built without the limit in `src-tauri/.cargo/config.toml`.
        assert_eq!(mapped(&db.0.lock().expect("lock writer")), MMAP_SIZE_BYTES);
        assert_eq!(mapped(&db.read().expect("acquire reader")), MMAP_SIZE_BYTES);
    }

    #[test]
    fn the_writer_has_its_large_page_cache_only_while_a_write_holds_it() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db = open(&temp_dir.path().join(APP_DB_FILE)).expect("open db");
        let cache_kib = |conn: &Connection| -> i64 {
            -conn
                .pragma_query_value(None, "cache_size", |row| row.get::<_, i64>(0))
                .expect("read cache size")
        };
        let idle_writer = || cache_kib(&db.0.conn.lock().expect("writer connection"));
        assert_eq!(idle_writer(), IDLE_CACHE_KIB);

        let writer = db.0.lock().expect("lock writer");
        assert_eq!(cache_kib(&writer), WRITING_CACHE_KIB);
        drop(writer);

        assert_eq!(idle_writer(), IDLE_CACHE_KIB);
        assert_eq!(
            cache_kib(&db.read().expect("acquire reader")),
            IDLE_CACHE_KIB
        );
    }

    #[test]
    fn idle_readers_close_once_after_the_last_read_request() {
        let (_temp_dir, db) = opened();
        let held = db.read().expect("acquire reader");
        assert_eq!(probe_sum(&db.read().expect("another reader")), 1);

        // The request that just ended is too recent for a long wait.
        db.close_idle_readers(Duration::from_secs(3600));
        assert_eq!(open_idle_readers(&db), READER_COUNT - 1);

        db.close_idle_readers(Duration::ZERO);
        assert_eq!(open_idle_readers(&db), 0);
        // The held reader is untouched, and a closed one reopens for the next request.
        assert_eq!(probe_sum(&held), 1);
        assert_eq!(probe_sum(&db.read().expect("reopened reader")), 1);
    }

    #[test]
    fn closing_idle_readers_waits_for_a_read_request_to_end() {
        let (_temp_dir, db) = opened();
        drop(db.read().expect("acquire reader"));
        db.close_idle_readers(Duration::ZERO);
        assert_eq!(open_idle_readers(&db), 0);

        // No request has ended since, so a reader that opens for one stays open.
        let reader = db.read().expect("acquire reader");
        db.close_idle_readers(Duration::ZERO);
        drop(reader);
        assert_eq!(open_idle_readers(&db), 1);
    }

    #[test]
    fn the_writer_maps_the_file_again_after_a_write_and_a_checkpoint() {
        let (_temp_dir, db) = opened();
        let writer = db.0.lock().expect("lock writer");
        writer
            .execute("UPDATE probe SET value = 2", [])
            .expect("write");
        checkpoint_wal(&writer).expect("checkpoint");
        drop(writer);

        let writer = db.0.lock().expect("lock writer");
        let mapped: i64 = writer
            .pragma_query_value(None, "mmap_size", |row| row.get(0))
            .expect("read mmap size");
        assert_eq!(mapped, MMAP_SIZE_BYTES);
        assert_eq!(probe_sum(&writer), 2);
    }

    #[test]
    fn checkpoint_empties_the_wal_file() {
        let (temp_dir, db) = opened();
        let wal = temp_dir.path().join(format!("{APP_DB_FILE}-wal"));
        assert!(std::fs::metadata(&wal).expect("wal file").len() > 0);

        checkpoint_wal(&db.0.lock().expect("lock writer")).expect("checkpoint");

        assert_eq!(std::fs::metadata(&wal).expect("wal file").len(), 0);
    }

    #[test]
    fn free_space_worth_returning_shrinks_the_database_file() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let path = temp_dir.path().join(APP_DB_FILE);
        // The writer alone, as `open` has it before it opens the readers.
        let conn = Connection::open(&path).expect("open db");
        configure_writer(&conn).expect("configure writer");
        conn.execute_batch(
            "CREATE TABLE probe (value INTEGER NOT NULL);
             INSERT INTO probe (value) VALUES (7);
             CREATE TABLE filler (body BLOB NOT NULL);
             WITH RECURSIVE counter(n) AS (
                 SELECT 1 UNION ALL SELECT n + 1 FROM counter WHERE n < 2000
             )
             INSERT INTO filler SELECT zeroblob(4000) FROM counter;
             DROP TABLE filler;",
        )
        .expect("fill and free pages");
        checkpoint_wal(&conn).expect("checkpoint");
        let pragma = |name: &str| -> i64 {
            conn.pragma_query_value(None, name, |row| row.get(0))
                .expect("read pragma")
        };
        let file_bytes = || std::fs::metadata(&path).expect("database file").len() as i64;
        let filled_bytes = file_bytes();
        let free_bytes = pragma("freelist_count") * pragma("page_size");
        assert!(free_bytes > filled_bytes / 2);

        return_free_space(&conn, free_bytes + 1);
        assert_eq!(
            file_bytes(),
            filled_bytes,
            "too little to be worth a rewrite"
        );

        return_free_space(&conn, free_bytes);
        assert!(file_bytes() <= filled_bytes - free_bytes);
        assert_eq!(pragma("freelist_count"), 0);
        assert_eq!(probe_sum(&conn), 7);
        // The connection is configured as before.
        assert_eq!(pragma("mmap_size"), MMAP_SIZE_BYTES);
        assert_eq!(pragma("temp_store"), 2);
        let journal_mode: String = conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .expect("read journal mode");
        assert_eq!(journal_mode, "wal");
    }

    /// Returns a reader to the pool while it still holds a WAL read lock, as SQLite on
    /// Windows can leave one behind.
    fn pin_an_idle_reader(db: &Db) {
        let pool = db.1.as_deref().expect("reader pool");
        let conn = pool.acquire().expect("acquire reader");
        conn.execute_batch("BEGIN").expect("begin");
        assert_eq!(probe_sum(&conn), 1);
        pool.release(Some(conn));
    }

    #[test]
    fn a_checkpoint_completes_although_an_idle_reader_kept_its_read_lock() {
        let (temp_dir, db) = opened();
        let wal = temp_dir.path().join(format!("{APP_DB_FILE}-wal"));
        pin_an_idle_reader(&db);
        let writer = db.0.lock().expect("lock writer");
        writer
            .execute("UPDATE probe SET value = 7", [])
            .expect("commit change");
        writer
            .busy_timeout(Duration::from_millis(50))
            .expect("shorten the wait for the blocked checkpoint");
        checkpoint_wal(&writer).expect_err("the pinned reader blocks a plain checkpoint");

        db.checkpoint(&writer, Duration::from_secs(5))
            .expect("checkpoint");

        assert_eq!(std::fs::metadata(&wal).expect("wal file").len(), 0);
        drop(writer);
        assert_eq!(probe_sum(&db.read().expect("reader reopens")), 7);
    }

    #[test]
    fn a_checkpoint_waits_for_a_running_read_request() {
        let (temp_dir, db) = opened();
        let wal = temp_dir.path().join(format!("{APP_DB_FILE}-wal"));
        let reader = db.read().expect("acquire reader");
        assert_eq!(probe_sum(&reader), 1);
        std::thread::scope(|scope| {
            let checkpoint = scope.spawn(|| {
                let writer = db.0.lock().expect("lock writer");
                writer
                    .execute("UPDATE probe SET value = 7", [])
                    .expect("commit change");
                db.checkpoint(&writer, Duration::from_secs(5))
            });
            std::thread::sleep(Duration::from_millis(100));
            assert!(!checkpoint.is_finished(), "the checkpoint did not wait");
            drop(reader);
            checkpoint.join().expect("join").expect("checkpoint");
        });

        assert_eq!(std::fs::metadata(&wal).expect("wal file").len(), 0);
    }

    #[test]
    fn a_checkpoint_gives_up_on_a_read_request_that_outlasts_its_wait() {
        let (_temp_dir, db) = opened();
        let reader = db.read().expect("acquire reader");
        assert_eq!(probe_sum(&reader), 1);

        let error = db
            .checkpoint(
                &db.0.lock().expect("lock writer"),
                Duration::from_millis(50),
            )
            .expect_err("the request still runs");

        assert_eq!(error, "read requests were still running");
        // The pool serves requests again, and the held reader still works.
        assert_eq!(probe_sum(&db.read().expect("another reader")), 1);
        assert_eq!(probe_sum(&reader), 1);
    }
}
