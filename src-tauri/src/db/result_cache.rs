//! In-memory table results keyed by data generation, and the supersession of running table
//! computations. Nothing here is persisted; a restart starts empty.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

use rusqlite::{Connection, InterruptHandle};

/// The error a superseded table computation returns.
pub const SUPERSEDED: &str = "Superseded by a newer request";

/// A bounded cache of computed results, each keyed by a query fingerprint. Clones share it.
///
/// Every entry belongs to the newest data generation the cache has seen. A lookup or result
/// with a newer generation drops the older entries, and an entry serves any lookup whose
/// generation is not newer than its own.
pub struct ResultCache<V> {
    shared: Arc<Shared<V>>,
}

impl<V> Clone for ResultCache<V> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

struct Shared<V> {
    bound: usize,
    state: Mutex<CacheState<V>>,
    /// Signalled whenever a computation ends, so waiting requests look again.
    settled: Condvar,
}

struct CacheState<V> {
    generation: u64,
    entries: Vec<CacheEntry<V>>,
    /// Keys whose computation is running.
    computing: HashSet<String>,
    uses: u64,
    #[cfg(test)]
    waiting: usize,
}

struct CacheEntry<V> {
    key: String,
    value: Arc<V>,
    last_use: u64,
}

impl<V> ResultCache<V> {
    /// A cache that keeps at most `bound` entries, releasing the least recently used first.
    pub fn new(bound: usize) -> Self {
        Self {
            shared: Arc::new(Shared {
                bound,
                state: Mutex::new(CacheState {
                    generation: 0,
                    entries: Vec::new(),
                    computing: HashSet::new(),
                    uses: 0,
                    #[cfg(test)]
                    waiting: 0,
                }),
                settled: Condvar::new(),
            }),
        }
    }

    /// Returns the result for `key` at `generation` (`Db::generation`) or newer, computing it
    /// when absent.
    ///
    /// Concurrent requests for one missing key compute it once: the first runs `compute`, and
    /// the others wait for it. `compute` acquires its own reader, so a waiting request holds
    /// none. It returns the generation of the data it read (`Reader::generation`) with the
    /// value. A value from data older than the cache's generation is returned but not kept.
    pub fn get_or_compute<E>(
        &self,
        key: &str,
        generation: u64,
        compute: impl FnOnce() -> Result<(u64, V), E>,
    ) -> Result<Arc<V>, E> {
        let mut state = self.lock();
        loop {
            state.advance(generation);
            if let Some(value) = state.touch(key) {
                return Ok(value);
            }
            if state.computing.insert(key.to_string()) {
                break;
            }
            #[cfg(test)]
            {
                state.waiting += 1;
            }
            state = self
                .shared
                .settled
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
            #[cfg(test)]
            {
                state.waiting -= 1;
            }
        }
        drop(state);

        // Ends the computation even when `compute` fails or panics, so waiters never hang.
        let _computing = Computing { cache: self, key };
        let (data_generation, value) = compute()?;
        let value = Arc::new(value);
        let mut state = self.lock();
        state.advance(data_generation);
        if data_generation == state.generation {
            state.insert(key, Arc::clone(&value), self.shared.bound);
        }
        Ok(value)
    }

    fn lock(&self) -> MutexGuard<'_, CacheState<V>> {
        self.shared
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl<V> CacheState<V> {
    fn advance(&mut self, generation: u64) {
        if generation > self.generation {
            self.generation = generation;
            self.entries.clear();
        }
    }

    fn touch(&mut self, key: &str) -> Option<Arc<V>> {
        self.uses += 1;
        let entry = self.entries.iter_mut().find(|entry| entry.key == key)?;
        entry.last_use = self.uses;
        Some(Arc::clone(&entry.value))
    }

    fn insert(&mut self, key: &str, value: Arc<V>, bound: usize) {
        if self.entries.len() >= bound {
            let least_recent = (0..self.entries.len())
                .min_by_key(|&index| self.entries[index].last_use)
                .expect("a full cache has an entry");
            self.entries.swap_remove(least_recent);
        }
        self.uses += 1;
        self.entries.push(CacheEntry {
            key: key.to_string(),
            value,
            last_use: self.uses,
        });
    }
}

struct Computing<'a, V> {
    cache: &'a ResultCache<V>,
    key: &'a str,
}

impl<V> Drop for Computing<'_, V> {
    fn drop(&mut self) {
        self.cache.lock().computing.remove(self.key);
        self.cache.shared.settled.notify_all();
    }
}

/// The running computation of each table, so a newer request for a table can stop an older
/// one. Clones share it.
#[derive(Clone, Default)]
pub struct TableRegistry {
    running: Arc<Mutex<HashMap<String, Running>>>,
    next_run: Arc<AtomicU64>,
}

struct Running {
    run: u64,
    fingerprint: String,
    cancelled: Arc<AtomicBool>,
    interrupt: InterruptHandle,
}

/// Tells a running table computation whether a newer request superseded it.
pub struct Cancellation(Arc<AtomicBool>);

impl Cancellation {
    /// Fails with [`SUPERSEDED`] once the computation was superseded. An interrupt stops only
    /// a statement that is running, so a computation checks this between its statements.
    pub fn check(&self) -> Result<(), String> {
        if self.0.load(Ordering::Acquire) {
            Err(SUPERSEDED.to_string())
        } else {
            Ok(())
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))] // PR `cached-table-results` runs table computations here.
impl TableRegistry {
    /// Runs one computation on `conn`.
    ///
    /// With a table ID, it is that table's running computation until it returns. Starting a
    /// different fingerprint for the same table cancels it and interrupts its statement on
    /// `conn`; a failure it then returns becomes [`SUPERSEDED`]. It leaves the registry before
    /// returning, and `conn` stays borrowed until then, so an interrupt can never reach a later
    /// request on that connection. Without a table ID, nothing can supersede it.
    pub fn run<T>(
        &self,
        table: Option<&str>,
        fingerprint: &str,
        conn: &Connection,
        compute: impl FnOnce(&Cancellation) -> Result<T, String>,
    ) -> Result<T, String> {
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancellation = Cancellation(Arc::clone(&cancelled));
        let Some(table) = table else {
            return compute(&cancellation);
        };

        let registration = self.register(table, fingerprint, conn, cancelled);
        let result = compute(&cancellation);
        drop(registration);
        match result {
            Err(_) if cancellation.check().is_err() => Err(SUPERSEDED.to_string()),
            result => result,
        }
    }

    fn register<'a>(
        &'a self,
        table: &'a str,
        fingerprint: &str,
        conn: &Connection,
        cancelled: Arc<AtomicBool>,
    ) -> Registration<'a> {
        let run = self.next_run.fetch_add(1, Ordering::Relaxed);
        let mut running = self.lock();
        if let Some(older) = running.get(table) {
            if older.fingerprint != fingerprint {
                // The older computation is still registered, so it still holds its connection.
                older.cancelled.store(true, Ordering::Release);
                older.interrupt.interrupt();
            }
        }
        running.insert(
            table.to_string(),
            Running {
                run,
                fingerprint: fingerprint.to_string(),
                cancelled,
                interrupt: conn.get_interrupt_handle(),
            },
        );
        Registration {
            registry: self,
            table,
            run,
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, Running>> {
        self.running.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Removes its computation from the registry when dropped, unless a newer one replaced it.
struct Registration<'a> {
    registry: &'a TableRegistry,
    table: &'a str,
    run: u64,
}

impl Drop for Registration<'_> {
    fn drop(&mut self) {
        let mut running = self.registry.lock();
        if running
            .get(self.table)
            .is_some_and(|entry| entry.run == self.run)
        {
            running.remove(self.table);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    use super::*;

    fn computed(generation: u64, value: &'static str) -> Result<(u64, &'static str), String> {
        Ok((generation, value))
    }

    #[test]
    fn a_newer_generation_discards_older_entries() {
        let cache = ResultCache::new(4);
        cache
            .get_or_compute("a", 1, || computed(1, "a at 1"))
            .unwrap();
        cache
            .get_or_compute("b", 1, || computed(1, "b at 1"))
            .unwrap();

        let a = cache
            .get_or_compute("a", 2, || computed(2, "a at 2"))
            .unwrap();
        assert_eq!(*a, "a at 2");
        // A request that read the older generation gets the newer result, never the older one.
        let b = cache
            .get_or_compute("b", 1, || computed(2, "b at 2"))
            .unwrap();
        assert_eq!(*b, "b at 2");

        // A computation that read data older than the cache's generation is not kept.
        let stale = cache
            .get_or_compute("c", 2, || computed(1, "c at 1"))
            .unwrap();
        assert_eq!(*stale, "c at 1");
        let c = cache
            .get_or_compute("c", 2, || computed(2, "c at 2"))
            .unwrap();
        assert_eq!(*c, "c at 2");
    }

    #[test]
    fn concurrent_misses_for_one_key_compute_once_and_waiters_do_not_compute() {
        let cache = ResultCache::new(4);
        let computations = Arc::new(AtomicUsize::new(0));
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel::<()>();

        let first = {
            let cache = cache.clone();
            let computations = Arc::clone(&computations);
            std::thread::spawn(move || {
                cache.get_or_compute("query", 1, || {
                    computations.fetch_add(1, Ordering::SeqCst);
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                    computed(1, "order")
                })
            })
        };
        started_rx.recv().unwrap();

        let second = {
            let cache = cache.clone();
            let computations = Arc::clone(&computations);
            std::thread::spawn(move || {
                cache.get_or_compute("query", 1, || {
                    // The reader is acquired here, so a waiter that never computes holds none.
                    computations.fetch_add(1, Ordering::SeqCst);
                    computed(1, "duplicate")
                })
            })
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        while cache.lock().waiting == 0 {
            assert!(Instant::now() < deadline, "the second request waits");
            std::thread::yield_now();
        }
        release_tx.send(()).unwrap();

        assert_eq!(*first.join().unwrap().unwrap(), "order");
        assert_eq!(*second.join().unwrap().unwrap(), "order");
        assert_eq!(computations.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_failed_computation_lets_the_next_request_compute() {
        let cache = ResultCache::<&str>::new(4);
        let error = cache
            .get_or_compute("query", 1, || Err::<(u64, &str), _>("interrupted"))
            .unwrap_err();
        assert_eq!(error, "interrupted");

        let value = cache
            .get_or_compute("query", 1, || computed(1, "order"))
            .unwrap();
        assert_eq!(*value, "order");
    }

    #[test]
    fn the_bound_releases_the_least_recently_used_entry() {
        let cache = ResultCache::new(2);
        cache.get_or_compute("a", 1, || computed(1, "a")).unwrap();
        cache.get_or_compute("b", 1, || computed(1, "b")).unwrap();
        cache
            .get_or_compute("a", 1, || computed(1, "a recomputed"))
            .unwrap();
        cache.get_or_compute("c", 1, || computed(1, "c")).unwrap();

        let a = cache
            .get_or_compute("a", 1, || computed(1, "a recomputed"))
            .unwrap();
        let b = cache
            .get_or_compute("b", 1, || computed(1, "b recomputed"))
            .unwrap();
        assert_eq!(*a, "a");
        assert_eq!(*b, "b recomputed");
    }

    /// Runs `statement`'s first step on `conn`, so the statement stays active, then calls
    /// `meanwhile` and returns the result of stepping the statement to its end.
    fn step_around(conn: &Connection, meanwhile: impl FnOnce()) -> Result<usize, rusqlite::Error> {
        let mut statement = conn.prepare(
            "WITH RECURSIVE counter(n) AS (SELECT 1 UNION ALL SELECT n + 1 FROM counter LIMIT 1000)
             SELECT n FROM counter",
        )?;
        let mut rows = statement.query([])?;
        rows.next()?;
        meanwhile();
        let mut count = 1;
        while rows.next()?.is_some() {
            count += 1;
        }
        Ok(count)
    }

    #[test]
    fn a_different_fingerprint_for_the_table_interrupts_and_cancels_the_running_one() {
        let registry = TableRegistry::default();
        let older = Connection::open_in_memory().unwrap();
        let newer = Connection::open_in_memory().unwrap();

        let result = registry.run(Some("search"), "sort by CA", &older, |cancellation| {
            let stepped = step_around(&older, || {
                registry
                    .run(Some("search"), "sort by PA", &newer, |_| Ok(()))
                    .unwrap();
            });
            assert!(cancellation.check().is_err());
            stepped.map_err(|error| error.to_string())
        });

        assert_eq!(result, Err(SUPERSEDED.to_string()));
    }

    #[test]
    fn the_same_fingerprint_or_no_table_does_not_interrupt() {
        let registry = TableRegistry::default();
        let running = Connection::open_in_memory().unwrap();
        let other = Connection::open_in_memory().unwrap();

        let result = registry.run(Some("search"), "sort by CA", &running, |cancellation| {
            let stepped = step_around(&running, || {
                registry
                    .run(Some("search"), "sort by CA", &other, |_| Ok(()))
                    .unwrap();
                registry
                    .run(None, "sort by PA", &other, |_| Ok(()))
                    .unwrap();
                registry
                    .run(Some("staff-search"), "sort by PA", &other, |_| Ok(()))
                    .unwrap();
            });
            cancellation.check()?;
            stepped.map_err(|error| error.to_string())
        });

        assert_eq!(result, Ok(1000));
    }

    #[test]
    fn a_finished_computation_cannot_interrupt_a_later_request_on_its_connection() {
        let registry = TableRegistry::default();
        let reader = Connection::open_in_memory().unwrap();
        let newer = Connection::open_in_memory().unwrap();
        registry
            .run(Some("search"), "sort by CA", &reader, |_| Ok(()))
            .unwrap();

        // The connection now serves an unrelated read while the table's next query starts.
        let unrelated = step_around(&reader, || {
            registry
                .run(Some("search"), "sort by PA", &newer, |_| Ok(()))
                .unwrap();
        });

        assert_eq!(unrelated.unwrap(), 1000);
    }
}
