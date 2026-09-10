use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;

use super::index::{GraphicsIndex, GraphicsKind, GraphicsSummary, ImageResult};

const CACHE_LIMIT: usize = 256;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphicsStatus {
    pub generation: u64,
    pub selected: bool,
    pub rebuilding: bool,
    pub candidate: CandidateState,
    pub summary: GraphicsSummaryDto,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateState {
    pub available: bool,
    pub source: &'static str,
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphicsSummaryDto {
    pub configs: usize,
    pub mappings: usize,
    pub truncated: bool,
    pub diagnostics: GraphicsDiagnosticsDto,
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphicsDiagnosticsDto {
    #[serde(rename = "configLimit")]
    pub config_limit: usize,
    #[serde(rename = "entryLimit")]
    pub entry_limit: usize,
    #[serde(rename = "depthLimit")]
    pub depth_limit: usize,
    #[serde(rename = "mappingLimit")]
    pub mapping_limit: usize,
    #[serde(rename = "configTooLarge")]
    pub config_too_large: usize,
    #[serde(rename = "configUnreadable")]
    pub config_unreadable: usize,
    #[serde(rename = "malformedConfig")]
    pub malformed_config: usize,
    #[serde(rename = "invalidMapping")]
    pub invalid_mapping: usize,
    #[serde(rename = "sourceUnreadable")]
    pub source_unreadable: usize,
}
impl From<&GraphicsSummary> for GraphicsSummaryDto {
    fn from(s: &GraphicsSummary) -> Self {
        Self {
            configs: s.configs,
            mappings: s.mappings,
            truncated: s.truncated,
            diagnostics: GraphicsDiagnosticsDto {
                config_limit: s.diagnostics.config_limit,
                entry_limit: s.diagnostics.entry_limit,
                depth_limit: s.diagnostics.depth_limit,
                mapping_limit: s.diagnostics.mapping_limit,
                config_too_large: s.diagnostics.config_too_large,
                config_unreadable: s.diagnostics.config_unreadable,
                malformed_config: s.diagnostics.malformed_config,
                invalid_mapping: s.diagnostics.invalid_mapping,
                source_unreadable: s.diagnostics.source_unreadable,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum ResolveResult {
    Available { bytes: Vec<u8>, mime: &'static str },
    Missing,
}

struct Lru {
    available: HashMap<u32, ImageResult>,
    missing: HashMap<u32, ()>,
    order_available: VecDeque<u32>,
    order_missing: VecDeque<u32>,
}
impl Lru {
    fn new() -> Self {
        Self {
            available: HashMap::new(),
            missing: HashMap::new(),
            order_available: VecDeque::new(),
            order_missing: VecDeque::new(),
        }
    }
    fn clear(&mut self) {
        *self = Self::new();
    }
    fn get(&mut self, uid: u32) -> Option<Option<ImageResult>> {
        if let Some(v) = self.available.get(&uid).cloned() {
            self.order_available.retain(|x| *x != uid);
            self.order_available.push_back(uid);
            return Some(Some(v));
        }
        if self.missing.contains_key(&uid) {
            self.order_missing.retain(|x| *x != uid);
            self.order_missing.push_back(uid);
            return Some(None);
        }
        None
    }
    fn put(&mut self, uid: u32, value: Option<ImageResult>) {
        self.available.remove(&uid);
        self.missing.remove(&uid);
        self.order_available.retain(|x| *x != uid);
        self.order_missing.retain(|x| *x != uid);
        match value {
            Some(v) => {
                self.available.insert(uid, v);
                self.order_available.push_back(uid);
                while self.order_available.len() > CACHE_LIMIT {
                    if let Some(x) = self.order_available.pop_front() {
                        self.available.remove(&x);
                    }
                }
            }
            None => {
                self.missing.insert(uid, ());
                self.order_missing.push_back(uid);
                while self.order_missing.len() > CACHE_LIMIT {
                    if let Some(x) = self.order_missing.pop_front() {
                        self.missing.remove(&x);
                    }
                }
            }
        }
    }
}

struct Target {
    generation: u64,
    root: Option<PathBuf>,
}
struct WorkerState {
    stopping: bool,
    pending: Option<Target>,
}
struct WorkerControl {
    state: Mutex<WorkerState>,
    wake: Condvar,
}
struct State {
    next_reservation: u64,
    committed: Target,
    index: Option<GraphicsIndex>,
    installed_generation: Option<u64>,
    in_flight: Option<Target>,
    summary: GraphicsSummaryDto,
    candidate: CandidateState,
    caches: [Lru; 3],
}
type Scanner = Arc<dyn Fn(Option<&std::path::Path>) -> GraphicsIndex + Send + Sync>;

pub struct GraphicsRuntime {
    state: Arc<Mutex<State>>,
    transition_gate: Arc<Mutex<()>>,
    scanner: Scanner,
    worker: Arc<WorkerControl>,
}

impl Drop for GraphicsRuntime {
    fn drop(&mut self) {
        let mut w = self
            .worker
            .state
            .lock()
            .expect("graphics worker state poisoned");
        w.stopping = true;
        w.pending = None;
        self.worker.wake.notify_one();
    }
}

impl GraphicsRuntime {
    pub fn new(conn: &Connection, app: &AppHandle) -> Result<Self, String> {
        let root: Option<String> = conn
            .query_row("SELECT root FROM graphics_settings WHERE id=1", [], |r| {
                r.get(0)
            })
            .map_err(|e| e.to_string())?;
        Ok(Self::from_root(
            root.map(PathBuf::from),
            detect_candidate(&candidate_paths(app)),
        ))
    }
    fn from_root(root: Option<PathBuf>, candidate: CandidateState) -> Self {
        Self::from_root_with_scanner(
            root,
            candidate,
            Arc::new(|root| {
                root.map(GraphicsIndex::scan)
                    .unwrap_or_else(GraphicsIndex::empty)
            }),
        )
    }
    fn from_root_with_scanner(
        root: Option<PathBuf>,
        candidate: CandidateState,
        scanner: Scanner,
    ) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                next_reservation: 0,
                committed: Target {
                    generation: 0,
                    root,
                },
                index: None,
                installed_generation: None,
                in_flight: None,
                summary: GraphicsSummaryDto::default(),
                candidate,
                caches: [Lru::new(), Lru::new(), Lru::new()],
            })),
            transition_gate: Arc::new(Mutex::new(())),
            scanner,
            worker: Arc::new(WorkerControl {
                state: Mutex::new(WorkerState {
                    stopping: false,
                    pending: None,
                }),
                wake: Condvar::new(),
            }),
        }
    }
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().expect("graphics runtime state poisoned")
    }
    pub fn start_worker(&self) {
        let state = Arc::clone(&self.state);
        let gate = Arc::clone(&self.transition_gate);
        let worker = Arc::clone(&self.worker);
        let scanner = Arc::clone(&self.scanner);
        let initial = {
            let s = self.state();
            s.committed.root.as_ref().map(|_| Target {
                generation: s.committed.generation,
                root: s.committed.root.clone(),
            })
        };
        {
            let mut w = worker.state.lock().expect("graphics worker state poisoned");
            if w.pending.is_some() {
                return;
            }
            w.pending = initial;
            worker.wake.notify_one();
        }
        std::thread::spawn(move || loop {
            let target = {
                let mut w = worker.state.lock().expect("graphics worker state poisoned");
                while !w.stopping && w.pending.is_none() {
                    w = worker.wake.wait(w).expect("graphics worker wait poisoned");
                }
                if w.stopping {
                    return;
                }
                w.pending.take().expect("pending target")
            };
            let index = scanner(target.root.as_deref());
            let w = worker.state.lock().expect("graphics worker state poisoned");
            if w.stopping {
                return;
            }
            drop(w);
            let _gate = gate.lock().expect("graphics transition gate poisoned");
            let w = worker.state.lock().expect("graphics worker state poisoned");
            if w.stopping {
                return;
            }
            drop(w);
            let mut s = state.lock().expect("graphics runtime state poisoned");
            if s.committed.generation == target.generation && s.committed.root == target.root {
                GraphicsRuntime::commit_locked(&mut s, target, Some(index));
            }
        });
    }
    pub fn enqueue(&self, target: (u64, Option<PathBuf>)) {
        let mut w = self
            .worker
            .state
            .lock()
            .expect("graphics worker state poisoned");
        if !w.stopping {
            w.pending = Some(Target {
                generation: target.0,
                root: target.1,
            });
            self.worker.wake.notify_one();
        }
    }
    fn wake_worker(&self) {
        self.worker.wake.notify_one();
    }
    pub fn cancel_pending(&self) {
        let mut w = self
            .worker
            .state
            .lock()
            .expect("graphics worker state poisoned");
        w.pending = None;
        self.wake_worker();
    }
    pub fn status(&self) -> GraphicsStatus {
        let s = self.state();
        GraphicsStatus {
            generation: s.committed.generation,
            selected: s.committed.root.is_some(),
            rebuilding: s.in_flight.is_some(),
            candidate: s.candidate.clone(),
            summary: s.summary.clone(),
        }
    }
    fn reserve_locked(s: &mut State, root: Option<PathBuf>) -> Target {
        s.next_reservation = s.next_reservation.saturating_add(1);
        Target {
            generation: s.next_reservation,
            root,
        }
    }
    fn commit_locked(s: &mut State, target: Target, index: Option<GraphicsIndex>) {
        s.committed = Target {
            generation: target.generation,
            root: target.root,
        };
        s.in_flight = None;
        s.index = index;
        s.installed_generation = s.index.as_ref().map(|_| target.generation);
        s.summary = s
            .index
            .as_ref()
            .map(|i| GraphicsSummaryDto::from(i.summary()))
            .unwrap_or_default();
        for c in &mut s.caches {
            c.clear();
        }
    }
    /// Reserve and persist under the transition gate. Filesystem work starts only after this returns.
    pub fn persist_transition(
        &self,
        db: &Mutex<Connection>,
        root: Option<PathBuf>,
    ) -> Result<Option<(u64, Option<PathBuf>)>, String> {
        let persisted_root = root.clone();
        self.persist_transition_with(root, || {
            let conn = db
                .lock()
                .map_err(|_| "database lock poisoned".to_string())?;
            let changed = match persisted_root.as_deref() {
                Some(path) => conn.execute(
                    "UPDATE graphics_settings SET root=?1 WHERE id=1",
                    [path.to_string_lossy().as_ref()],
                ),
                None => conn.execute("UPDATE graphics_settings SET root=NULL WHERE id=1", []),
            }
            .map_err(|e| e.to_string())?;
            Ok(changed == 1)
        })
    }

    /// Controlled persistence seam used by orchestration tests. The callback runs while the
    /// transition gate is held, but never while the runtime state lock is held.
    pub fn persist_transition_with<F>(
        &self,
        root: Option<PathBuf>,
        persist: F,
    ) -> Result<Option<(u64, Option<PathBuf>)>, String>
    where
        F: FnOnce() -> Result<bool, String>,
    {
        let _gate = self
            .transition_gate
            .lock()
            .expect("graphics transition gate poisoned");
        let target = {
            let mut s = self.state();
            Self::reserve_locked(&mut s, root)
        };
        if !persist()? {
            return Ok(None);
        }
        let generation = target.generation;
        let root = target.root.clone();
        let mut s = self.state();
        Self::commit_locked(
            &mut s,
            Target {
                generation,
                root: root.clone(),
            },
            None,
        );
        s.in_flight = Some(Target {
            generation,
            root: root.clone(),
        });
        Ok(Some((generation, root)))
    }
    pub fn begin_rescan(
        &self,
        db: &Mutex<Connection>,
    ) -> Result<Option<(u64, Option<PathBuf>)>, String> {
        self.begin_rescan_with(|root| {
            let conn = db
                .lock()
                .map_err(|_| "database lock poisoned".to_string())?;
            let changed = match root {
                Some(path) => conn.execute(
                    "UPDATE graphics_settings SET root=?1 WHERE id=1",
                    [path.to_string_lossy().as_ref()],
                ),
                None => conn.execute("UPDATE graphics_settings SET root=NULL WHERE id=1", []),
            }
            .map_err(|e| e.to_string())?;
            Ok(changed == 1)
        })
    }

    pub fn begin_rescan_with<F>(&self, persist: F) -> Result<Option<(u64, Option<PathBuf>)>, String>
    where
        F: FnOnce(Option<&std::path::Path>) -> Result<bool, String>,
    {
        let _gate = self
            .transition_gate
            .lock()
            .expect("graphics transition gate poisoned");
        let target = {
            let mut s = self.state();
            let root = s.committed.root.clone();
            Self::reserve_locked(&mut s, root)
        };
        if !persist(target.root.as_deref())? {
            return Ok(None);
        }
        let generation = target.generation;
        let root = target.root.clone();
        let mut s = self.state();
        Self::commit_locked(
            &mut s,
            Target {
                generation,
                root: root.clone(),
            },
            None,
        );
        s.in_flight = Some(Target {
            generation,
            root: root.clone(),
        });
        Ok(Some((generation, root)))
    }
    #[cfg(test)]
    pub fn complete_scan(&self, target: u64, root: Option<PathBuf>, index: GraphicsIndex) -> bool {
        let _gate = self
            .transition_gate
            .lock()
            .expect("graphics transition gate poisoned");
        let mut s = self.state();
        if s.committed.generation != target || s.committed.root != root {
            return false;
        }
        Self::commit_locked(
            &mut s,
            Target {
                generation: target,
                root,
            },
            Some(index),
        );
        true
    }
    #[cfg(test)]
    pub fn scan_reserved(&self, target: u64, root: Option<PathBuf>) -> bool {
        self.scan_reserved_with(target, root, |root| {
            root.map(GraphicsIndex::scan)
                .unwrap_or_else(GraphicsIndex::empty)
        })
    }

    /// Controlled scanner seam. Production callers use the filesystem-backed scanner above;
    /// tests supply a deterministic scanner to interleave completion and transitions.
    #[cfg(test)]
    pub fn scan_reserved_with<F>(&self, target: u64, root: Option<PathBuf>, scan: F) -> bool
    where
        F: FnOnce(Option<&std::path::Path>) -> GraphicsIndex,
    {
        let index = scan(root.as_deref());
        self.complete_scan(target, root, index)
    }
    pub fn resolve(&self, kind: GraphicsKind, uid: u32) -> ResolveResult {
        self.resolve_with_reader(kind, uid, |locator| locator.read())
    }

    fn resolve_with_reader<F>(&self, kind: GraphicsKind, uid: u32, read: F) -> ResolveResult
    where
        F: FnOnce(super::index::ImageLocator) -> Option<ImageResult>,
    {
        if uid == 0 {
            return ResolveResult::Missing;
        }
        let (generation, cached, locator) = {
            let mut s = self.state();
            if s.installed_generation != Some(s.committed.generation) {
                return ResolveResult::Missing;
            }
            let cached = s.caches[kind as usize].get(uid);
            let locator = if cached.is_none() {
                s.index
                    .as_ref()
                    .and_then(|index| index.resolve_locator(kind, uid))
            } else {
                None
            };
            (s.committed.generation, cached, locator)
        };
        if let Some(value) = cached {
            return value
                .map(|x| ResolveResult::Available {
                    bytes: x.bytes,
                    mime: x.mime,
                })
                .unwrap_or(ResolveResult::Missing);
        }
        let value = locator.and_then(read);
        let mut s = self.state();
        if s.committed.generation != generation || s.installed_generation != Some(generation) {
            return ResolveResult::Missing;
        }
        s.caches[kind as usize].put(uid, value.clone());
        value
            .map(|x| ResolveResult::Available {
                bytes: x.bytes,
                mime: x.mime,
            })
            .unwrap_or(ResolveResult::Missing)
    }
}

pub fn candidate_paths(app: &AppHandle) -> CandidatePaths {
    CandidatePaths {
        documents: app
            .path()
            .document_dir()
            .ok()
            .map(|p| p.join("Sports Interactive/Football Manager 2026/graphics")),
        onedrive: app.path().home_dir().ok().map(|p| {
            p.join("OneDrive/Documents/Sports Interactive/Football Manager 2026/graphics")
        }),
    }
}
#[derive(Clone, Debug)]
pub struct CandidatePaths {
    pub documents: Option<PathBuf>,
    pub onedrive: Option<PathBuf>,
}
pub fn detect_candidate(paths: &CandidatePaths) -> CandidateState {
    for (source, path) in [
        ("documents", paths.documents.as_ref()),
        ("onedrive", paths.onedrive.as_ref()),
    ] {
        if path.is_some_and(|p| p.is_dir() && std::fs::read_dir(p).is_ok()) {
            return CandidateState {
                available: true,
                source,
            };
        }
    }
    CandidateState {
        available: false,
        source: "absent",
    }
}
pub fn candidate_start_from(paths: &CandidatePaths) -> Option<PathBuf> {
    if let Some(path) = [&paths.documents, &paths.onedrive]
        .into_iter()
        .flatten()
        .find(|p| p.is_dir() && std::fs::read_dir(p).is_ok())
    {
        return Some(path.clone());
    }
    for path in [&paths.documents, &paths.onedrive].into_iter().flatten() {
        let mut current = path.as_path();
        while let Some(parent) = current.parent() {
            current = parent;
            if current.is_dir() && std::fs::read_dir(current).is_ok() {
                return Some(current.to_path_buf());
            }
        }
    }
    None
}
pub fn candidate_start(app: &AppHandle) -> Option<PathBuf> {
    candidate_start_from(&candidate_paths(app))
}
pub fn picker(app: &AppHandle) -> Result<Option<PathBuf>, String> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut dialog = app.dialog().file();
    if let Some(start) = candidate_start(app) {
        dialog = dialog.set_directory(start)
    }
    dialog.pick_folder(move |p| {
        let _ = tx.send(p.and_then(|x| x.into_path().ok()));
    });
    rx.recv().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::graphics::commands::GraphicsKindDto;
    use std::sync::mpsc;
    use std::thread;

    fn root_with_image(name: &str, uid: u32, bytes: &[u8]) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join(name), bytes).unwrap();
        std::fs::write(
            root.path().join("config.xml"),
            format!(r#"<record from="{name}" to="graphics/pictures/person/{uid}/portrait"/>"#),
        )
        .unwrap();
        root
    }

    fn test_db() -> Mutex<Connection> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE graphics_settings (id INTEGER PRIMARY KEY CHECK (id=1), root TEXT); INSERT INTO graphics_settings VALUES (1, NULL);").unwrap();
        Mutex::new(conn)
    }
    #[test]
    fn resolve_and_kind_dtos_are_pathless_and_closed() {
        let available = serde_json::to_string(&ResolveResult::Available {
            bytes: vec![1, 2],
            mime: "image/png",
        })
        .unwrap();
        assert_eq!(
            available,
            r#"{"status":"available","bytes":[1,2],"mime":"image/png"}"#
        );
        assert_eq!(
            serde_json::to_string(&ResolveResult::Missing).unwrap(),
            r#"{"status":"missing"}"#
        );
        assert!(!available.contains("path"));
        assert_eq!(
            serde_json::to_string(&GraphicsKindDto::PersonPortrait).unwrap(),
            r#""personPortrait""#
        );
        assert!(serde_json::from_str::<GraphicsKindDto>(r#""Unknown""#).is_err());
    }

    #[test]
    fn every_kind_has_bounded_available_and_missing_lru_classes() {
        for _ in 0..3 {
            let mut c = Lru::new();
            for uid in 1..=257 {
                c.put(
                    uid,
                    Some(ImageResult {
                        bytes: vec![1],
                        mime: "image/png",
                    }),
                )
            }
            assert_eq!(c.available.len(), CACHE_LIMIT);
            assert!(!c.available.contains_key(&1));
            let _ = c.get(2);
            c.put(
                258,
                Some(ImageResult {
                    bytes: vec![1],
                    mime: "image/png",
                }),
            );
            assert!(c.available.contains_key(&2));
            assert!(!c.available.contains_key(&3));
            for uid in 1..=257 {
                c.put(uid, None)
            }
            assert_eq!(c.missing.len(), CACHE_LIMIT);
            assert!(!c.missing.contains_key(&1));
            let _ = c.get(2);
            c.put(258, None);
            assert!(c.missing.contains_key(&2));
            assert!(!c.missing.contains_key(&3));
        }
    }
    #[test]
    fn serialized_status_contains_no_filesystem_identity() {
        let status = GraphicsStatus {
            generation: 4,
            selected: true,
            rebuilding: false,
            candidate: CandidateState {
                available: false,
                source: "absent",
            },
            summary: GraphicsSummaryDto::default(),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(!json.contains('/') && !json.contains('\\'));
        assert!(!json.contains("path"));
    }
    #[test]
    fn candidate_order_and_safe_absence() {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().join("docs");
        let o = t.path().join("drive");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::create_dir_all(&o).unwrap();
        let p = CandidatePaths {
            documents: Some(d.clone()),
            onedrive: Some(o.clone()),
        };
        assert_eq!(detect_candidate(&p).source, "documents");
        std::fs::remove_dir(&d).unwrap();
        assert_eq!(detect_candidate(&p).source, "onedrive");
        std::fs::remove_dir(&o).unwrap();
        assert!(!detect_candidate(&p).available);
    }
    #[test]
    fn picker_start_falls_back_to_closest_hierarchy() {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().join("Documents");
        let o = t.path().join("OneDrive/Documents/Sports Interactive");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::create_dir_all(&o).unwrap();
        let p = CandidatePaths {
            documents: Some(d.join("Sports Interactive/Football Manager 2026/graphics")),
            onedrive: Some(o.join("Football Manager 2026/graphics")),
        };
        assert_eq!(candidate_start_from(&p), Some(d));
    }

    fn test_runtime_with_root(root: Option<PathBuf>) -> GraphicsRuntime {
        GraphicsRuntime::from_root(
            root,
            CandidateState {
                available: false,
                source: "absent",
            },
        )
    }

    fn test_runtime() -> GraphicsRuntime {
        test_runtime_with_root(None)
    }

    #[test]
    fn real_transition_observes_available_missing_and_failed_persistence_preservation() {
        let runtime = test_runtime();
        let db = test_db();
        let a = root_with_image("a.png", 101, b"\x89PNG\r\n\x1a\nA");
        let b = tempfile::tempdir().unwrap();
        let (generation, root) = runtime
            .persist_transition(&db, Some(a.path().to_path_buf()))
            .unwrap()
            .unwrap();
        assert!(runtime.scan_reserved(generation, root));
        assert_eq!(runtime.status().generation, generation);
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 101),
            ResolveResult::Available {
                bytes: b"\x89PNG\r\n\x1a\nA".to_vec(),
                mime: "image/png"
            }
        );
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 999),
            ResolveResult::Missing
        );
        let before = runtime.status();
        db.lock()
            .unwrap()
            .execute_batch(
                "CREATE TRIGGER fail_graphics_update
                 BEFORE UPDATE OF root ON graphics_settings
                 WHEN NEW.root IS NOT OLD.root
                 BEGIN SELECT RAISE(ABORT, 'forced graphics persistence failure'); END;",
            )
            .unwrap();
        let failure = runtime.persist_transition(&db, Some(b.path().to_path_buf()));
        assert_eq!(failure.unwrap_err(), "forced graphics persistence failure");
        {
            let state = runtime.state();
            let cache = &state.caches[GraphicsKind::PersonPortrait as usize];
            let available = cache.available.get(&101).expect("available cache entry");
            assert_eq!(available.bytes, b"\x89PNG\r\n\x1a\nA");
            assert_eq!(available.mime, "image/png");
            assert!(cache.missing.contains_key(&999));
        }
        assert_eq!(runtime.status().generation, before.generation);
        assert_eq!(runtime.status().selected, before.selected);
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 101),
            ResolveResult::Available {
                bytes: b"\x89PNG\r\n\x1a\nA".to_vec(),
                mime: "image/png"
            }
        );
        let persisted: Option<String> = db
            .lock()
            .unwrap()
            .query_row("SELECT root FROM graphics_settings WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(persisted, Some(a.path().to_string_lossy().into_owned()));
    }

    #[test]
    fn successful_root_replacement_clears_available_and_missing_classes() {
        let runtime = test_runtime();
        let a = root_with_image("a.png", 101, b"\x89PNG\r\n\x1a\nA");
        let b = root_with_image("b.png", 202, b"\x89PNG\r\n\x1a\nB");
        let (ag, ar) = runtime
            .persist_transition_with(Some(a.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        assert!(runtime.scan_reserved(ag, ar));
        assert!(matches!(
            runtime.resolve(GraphicsKind::PersonPortrait, 101),
            ResolveResult::Available { .. }
        ));
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 999),
            ResolveResult::Missing
        );
        let (bg, br) = runtime
            .persist_transition_with(Some(b.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        assert!(runtime.scan_reserved(bg, br));
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 101),
            ResolveResult::Missing
        );
        assert!(matches!(
            runtime.resolve(GraphicsKind::PersonPortrait, 202),
            ResolveResult::Available { .. }
        ));
    }

    #[test]
    fn inverted_scan_completion_is_stale_and_converges_on_b() {
        let runtime = test_runtime();
        let a = root_with_image("a.png", 101, b"\x89PNG\r\n\x1a\nA");
        let b = root_with_image("b.png", 202, b"\x89PNG\r\n\x1a\nB");
        let (ag, ar) = runtime
            .persist_transition_with(Some(a.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        let (bg, br) = runtime
            .persist_transition_with(Some(b.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        assert!(runtime.scan_reserved(bg, br));
        assert!(!runtime.scan_reserved(ag, ar));
        assert_eq!(runtime.status().generation, bg);
        assert!(matches!(
            runtime.resolve(GraphicsKind::PersonPortrait, 202),
            ResolveResult::Available { .. }
        ));
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 101),
            ResolveResult::Missing
        );
    }

    #[test]
    fn blocked_image_read_releases_transition_gate_and_rejects_stale_result() {
        let runtime = std::sync::Arc::new(test_runtime());
        let a = root_with_image("a.png", 101, b"\x89PNG\r\n\x1a\nA");
        let b = root_with_image("b.png", 101, b"\x89PNG\r\n\x1a\nB");
        let db = test_db();
        let (a_generation, a_root) = runtime
            .persist_transition(&db, Some(a.path().to_path_buf()))
            .unwrap()
            .unwrap();
        assert!(runtime.complete_scan(a_generation, a_root, GraphicsIndex::scan(a.path())));
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let reading = runtime.clone();
        let handle = thread::spawn(move || {
            reading.resolve_with_reader(GraphicsKind::PersonPortrait, 101, |locator| {
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                locator.read()
            })
        });
        started_rx.recv().unwrap();
        let (b_generation, b_root) = runtime
            .persist_transition(&db, Some(b.path().to_path_buf()))
            .unwrap()
            .unwrap();
        assert!(runtime.complete_scan(b_generation, b_root, GraphicsIndex::scan(b.path())));
        release_tx.send(()).unwrap();
        assert_eq!(handle.join().unwrap(), ResolveResult::Missing);
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 101),
            ResolveResult::Available {
                bytes: b"\x89PNG\r\n\x1a\nB".to_vec(),
                mime: "image/png"
            }
        );
    }

    #[test]
    fn blocked_scan_releases_db_and_transition_gate() {
        let runtime = std::sync::Arc::new(test_runtime());
        let root = tempfile::tempdir().unwrap();
        let (generation, scan_root) = runtime
            .persist_transition_with(Some(root.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let scanner = runtime.clone();
        let handle = thread::spawn(move || {
            scanner.scan_reserved_with(generation, scan_root, |_| {
                started_tx.send(()).unwrap();
                release_rx.recv().unwrap();
                GraphicsIndex::empty()
            })
        });
        started_rx.recv().unwrap();
        let db = test_db();
        let (new_generation, _) = runtime.persist_transition(&db, None).unwrap().unwrap();
        assert!(new_generation > generation);
        release_tx.send(()).unwrap();
        assert!(!handle.join().unwrap());
        assert_eq!(runtime.status().generation, new_generation);
    }

    #[test]
    fn choose_clear_rescan_and_lazy_scan_supersede_by_committed_generation() {
        let runtime = test_runtime();
        let db = test_db();
        let a = root_with_image("a.png", 101, b"\x89PNG\r\n\x1a\nA");
        let (a_generation, a_root) = runtime
            .persist_transition(&db, Some(a.path().to_path_buf()))
            .unwrap()
            .unwrap();
        assert!(runtime.scan_reserved(a_generation, a_root));
        assert!(matches!(
            runtime.resolve(GraphicsKind::PersonPortrait, 101),
            ResolveResult::Available { .. }
        ));

        let (clear_generation, clear_root) =
            runtime.persist_transition(&db, None).unwrap().unwrap();
        assert!(runtime.scan_reserved(clear_generation, clear_root));
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 101),
            ResolveResult::Missing
        );

        let (rescan_generation, rescan_root) = runtime.begin_rescan(&db).unwrap().unwrap();
        assert!(runtime.scan_reserved(rescan_generation, rescan_root));
        let persisted: Option<String> = db
            .lock()
            .unwrap()
            .query_row("SELECT root FROM graphics_settings WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(persisted, None);
        let lazy_runtime = test_runtime_with_root(Some(a.path().to_path_buf()));
        assert_eq!(
            lazy_runtime.resolve(GraphicsKind::PersonPortrait, 101),
            ResolveResult::Missing
        );
    }

    #[test]
    fn resolve_never_starts_a_scan() {
        let (called_tx, called_rx) = mpsc::channel();
        let runtime = GraphicsRuntime::from_root_with_scanner(
            Some(PathBuf::from("root")),
            CandidateState {
                available: false,
                source: "absent",
            },
            Arc::new(move |_| {
                called_tx.send(()).unwrap();
                GraphicsIndex::empty()
            }),
        );
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 101),
            ResolveResult::Missing
        );
        assert!(called_rx.try_recv().is_err());
    }

    #[test]
    fn successful_transition_invalidates_both_cache_classes_and_stale_completion() {
        let runtime = test_runtime();
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let (a_generation, a_root) = runtime
            .persist_transition_with(Some(a.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        assert!(runtime.complete_scan(a_generation, a_root.clone(), GraphicsIndex::empty()));
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 1),
            ResolveResult::Missing
        );
        assert_eq!(
            runtime.resolve(GraphicsKind::ClubLogo, 2),
            ResolveResult::Missing
        );

        let (b_generation, b_root) = runtime
            .persist_transition_with(Some(b.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        assert!(!runtime.complete_scan(a_generation, a_root, GraphicsIndex::empty()));
        assert!(runtime.complete_scan(b_generation, b_root, GraphicsIndex::empty()));
        assert_eq!(runtime.status().generation, b_generation);
        assert_eq!(
            runtime.resolve(GraphicsKind::PersonPortrait, 1),
            ResolveResult::Missing
        );
        assert_eq!(
            runtime.resolve(GraphicsKind::ClubLogo, 2),
            ResolveResult::Missing
        );
    }

    #[test]
    fn failed_reservation_does_not_discard_earlier_scan_eligibility() {
        let runtime = test_runtime();
        let a = tempfile::tempdir().unwrap();
        let (generation, root) = runtime
            .persist_transition_with(Some(a.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        assert!(runtime
            .persist_transition_with(Some(PathBuf::from("failed")), || Ok(false))
            .unwrap()
            .is_none());
        assert!(runtime.complete_scan(generation, root, GraphicsIndex::empty()));
        assert_eq!(runtime.status().generation, generation);
    }

    #[test]
    fn rescan_commits_new_generation_and_discards_older_completion() {
        let runtime = test_runtime();
        let root = tempfile::tempdir().unwrap();
        let (first, first_root) = runtime
            .persist_transition_with(Some(root.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        let (second, second_root) = runtime.begin_rescan_with(|_| Ok(true)).unwrap().unwrap();
        assert!(!runtime.complete_scan(first, first_root, GraphicsIndex::empty()));
        assert!(runtime.complete_scan(second, second_root, GraphicsIndex::empty()));
        assert_eq!(runtime.status().generation, second);
    }

    #[test]
    fn worker_replaces_pending_target_and_scans_serially() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let (started, started_rx) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        let (scanned, scanned_rx) = mpsc::channel();
        let first_path = first.path().to_path_buf();
        let scanner = Arc::new(move |root: Option<&std::path::Path>| {
            scanned.send(root.unwrap().to_path_buf()).unwrap();
            if root == Some(first_path.as_path()) {
                started.send(()).unwrap();
                release_rx.lock().unwrap().recv().unwrap();
            }
            GraphicsIndex::empty()
        });
        let runtime = GraphicsRuntime::from_root_with_scanner(
            None,
            CandidateState {
                available: false,
                source: "absent",
            },
            scanner,
        );
        runtime.start_worker();
        let a = runtime
            .persist_transition_with(Some(first.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        runtime.enqueue(a);
        started_rx.recv().unwrap();
        let b = runtime
            .persist_transition_with(Some(second.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        let b_generation = b.0;
        runtime.enqueue(b);
        release.send(()).unwrap();
        assert_eq!(scanned_rx.recv().unwrap(), first.path());
        assert_eq!(scanned_rx.recv().unwrap(), second.path());
        assert_eq!(runtime.status().generation, b_generation);
    }

    #[test]
    fn dropping_runtime_revokes_active_installation_without_waiting() {
        let root = tempfile::tempdir().unwrap();
        let (started, started_rx) = mpsc::channel();
        let (release, release_rx) = mpsc::channel();
        let release_rx = Mutex::new(release_rx);
        let (finished, finished_rx) = mpsc::channel();
        let scanner = Arc::new(move |_root: Option<&std::path::Path>| {
            started.send(()).unwrap();
            release_rx.lock().unwrap().recv().unwrap();
            finished.send(()).unwrap();
            GraphicsIndex::empty()
        });
        let runtime = Arc::new(GraphicsRuntime::from_root_with_scanner(
            None,
            CandidateState {
                available: false,
                source: "absent",
            },
            scanner,
        ));
        runtime.start_worker();
        let target = runtime
            .persist_transition_with(Some(root.path().to_path_buf()), || Ok(true))
            .unwrap()
            .unwrap();
        runtime.enqueue(target);
        started_rx.recv().unwrap();
        let observer = Arc::clone(&runtime.state);
        drop(runtime);
        release.send(()).unwrap();
        finished_rx.recv().unwrap();
        assert!(observer.lock().unwrap().index.is_none());
    }
}
