//! Game faces: the pre-generated portrait FM27 shows for a player it generated. The face is
//! worked out from the appearance stored with the snapshot and read from the portrait bundle
//! of the FM install, so a face mod that replaces the bundle is what the app shows too.

mod bundle;
mod dxt;
mod lz4;
mod matcher;
mod tables;

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime};

use rusqlite::{Connection, OptionalExtension};

use crate::features::player::appearance::PlayerAppearance;

use super::runtime::ImageLookupResult;
use bundle::{FaceImage, PortraitBundle};
use matcher::FaceSubject;

/// The portrait bundle's place inside the FM install.
const BUNDLE_PATH: [&str; 4] = [
    "fm_Data",
    "StreamingAssets",
    "AddrData",
    "portraits_assets_all.bundle",
];
/// Encoded faces kept for repeated requests; a face is about 150 KB.
const CACHE_BYTE_LIMIT: usize = 32 * 1024 * 1024;

pub struct GameFaces {
    bundle_path: Option<PathBuf>,
    /// Whether the bundle was there at startup and has not failed to load since.
    available: AtomicBool,
    state: Mutex<State>,
    /// Held while the bundle is read, so requests that arrive meanwhile wait for one read.
    loading: Mutex<()>,
}

struct State {
    loaded: Option<Arc<Loaded>>,
    /// The bundle file that could not be read; it is not tried again until it changes.
    failed: Option<FileIdentity>,
    cache: FaceCache,
    last_request: Option<Instant>,
}

struct Loaded {
    identity: FileIdentity,
    bundle: PortraitBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileIdentity {
    len: u64,
    modified: Option<SystemTime>,
}

impl FileIdentity {
    fn of(path: &Path) -> Option<Self> {
        let metadata = std::fs::metadata(path).ok().filter(|file| file.is_file())?;
        Some(Self {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        })
    }
}

/// Encoded faces by catalog index, least recently used first. `None` is a face the bundle
/// cannot show, kept so that it is not decoded again.
#[derive(Default)]
struct FaceCache {
    faces: HashMap<usize, Option<Vec<u8>>>,
    order: VecDeque<usize>,
    bytes: usize,
}

impl FaceCache {
    fn get(&mut self, index: usize) -> Option<Option<Vec<u8>>> {
        let face = self.faces.get(&index)?.clone();
        self.order.retain(|cached| *cached != index);
        self.order.push_back(index);
        Some(face)
    }

    fn put(&mut self, index: usize, face: Option<Vec<u8>>) {
        let len = face.as_ref().map_or(0, Vec::len);
        if len > CACHE_BYTE_LIMIT || self.faces.contains_key(&index) {
            return;
        }
        self.bytes += len;
        self.faces.insert(index, face);
        self.order.push_back(index);
        while self.bytes > CACHE_BYTE_LIMIT {
            let Some(oldest) = self.order.pop_front() else {
                break;
            };
            if let Some(evicted) = self.faces.remove(&oldest) {
                self.bytes -= evicted.map_or(0, |face| face.len());
            }
        }
    }
}

impl GameFaces {
    /// Game faces from the bundle at `bundle_path`; `None` turns them off.
    pub fn new(bundle_path: Option<PathBuf>) -> Self {
        let available = bundle_path
            .as_deref()
            .is_some_and(|path| FileIdentity::of(path).is_some());
        Self {
            bundle_path,
            available: AtomicBool::new(available),
            state: Mutex::new(State {
                loaded: None,
                failed: None,
                cache: FaceCache::default(),
                last_request: None,
            }),
            loading: Mutex::new(()),
        }
    }

    /// Game faces from the FM27 install that the bridge plugin installs into.
    pub fn for_fm_install() -> Self {
        let plugins = crate::features::memory_read::install::resolve_steam_plugins_path().ok();
        let install = plugins
            .as_deref()
            .and_then(|plugins| plugins.parent()?.parent());
        Self::new(install.map(|install| {
            BUNDLE_PATH
                .iter()
                .fold(install.to_path_buf(), |path, part| path.join(part))
        }))
    }

    /// Whether a request can expect a face. The bundle is read on the first request.
    pub fn available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }

    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().expect("game faces state poisoned")
    }

    /// The face of player `uid` in snapshot `snapshot_id`, as a PNG.
    pub fn resolve(&self, conn: &Connection, snapshot_id: i64, uid: u32) -> ImageLookupResult {
        self.face(conn, snapshot_id, uid)
            .map(|bytes| ImageLookupResult::Available {
                bytes,
                mime: "image/png",
            })
            .unwrap_or(ImageLookupResult::Missing)
    }

    fn face(&self, conn: &Connection, snapshot_id: i64, uid: u32) -> Option<Vec<u8>> {
        let path = self.bundle_path.as_deref()?;
        let key = matcher::face_key(&read_subject(conn, snapshot_id, uid)?)?;
        let loaded = self.loaded(path)?;
        let index = matcher::best_face(key, loaded.bundle.faces())?;
        {
            let mut state = self.state();
            state.last_request = Some(Instant::now());
            if let Some(face) = state.cache.get(index) {
                return face;
            }
        }
        // A face the catalog lists without a texture, or one that cannot be decoded, stays
        // missing; a replaced bundle has another identity and is read again.
        let face = match loaded.bundle.read_face(path, index) {
            Ok(image) => image.as_ref().and_then(encode_png),
            Err(error) => {
                log::warn!("game face could not be read: {}", error.0);
                None
            }
        };
        let mut state = self.state();
        if state
            .loaded
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &loaded))
        {
            state.cache.put(index, face.clone());
        }
        face
    }

    /// The bundle's catalog, read on first use and again when the file changes.
    fn loaded(&self, path: &Path) -> Option<Arc<Loaded>> {
        let identity = FileIdentity::of(path)?;
        let current = |state: &State| {
            let loaded = state
                .loaded
                .as_ref()
                .filter(|loaded| loaded.identity == identity);
            (loaded.cloned(), state.failed == Some(identity))
        };
        match current(&self.state()) {
            (Some(loaded), _) => return Some(loaded),
            (None, true) => return None,
            (None, false) => {}
        }
        let _loading = self.loading.lock().expect("game faces load gate poisoned");
        match current(&self.state()) {
            (Some(loaded), _) => return Some(loaded),
            (None, true) => return None,
            (None, false) => {}
        }
        let result = PortraitBundle::open(path);
        let mut state = self.state();
        state.cache = FaceCache::default();
        match result {
            Ok(bundle) => {
                let loaded = Arc::new(Loaded { identity, bundle });
                state.loaded = Some(Arc::clone(&loaded));
                state.failed = None;
                self.available.store(true, Ordering::Relaxed);
                Some(loaded)
            }
            Err(error) => {
                log::warn!("game faces are unavailable: {}", error.0);
                state.loaded = None;
                state.failed = Some(identity);
                self.available.store(false, Ordering::Relaxed);
                None
            }
        }
    }

    /// Drops the encoded faces once none has been requested for `idle_for`.
    pub fn release_idle(&self, idle_for: Duration) {
        let mut state = self.state();
        if state
            .last_request
            .is_some_and(|last| last.elapsed() >= idle_for)
        {
            state.cache = FaceCache::default();
            state.last_request = None;
        }
    }
}

/// Reads what the face of one player depends on. `None` for a player FM did not generate.
fn read_subject(conn: &Connection, snapshot_id: i64, uid: u32) -> Option<FaceSubject> {
    let row: (Option<i64>, String, Option<i64>) = conn
        .query_row(
            "SELECT appearance, gender, age FROM players WHERE snapshot_id = ?1 AND uid = ?2",
            (snapshot_id, uid),
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .ok()??;
    let (appearance, gender, age) = row;
    let female = match gender.as_str() {
        "male" => false,
        "female" => true,
        _ => return None,
    };
    Some(FaceSubject {
        uid,
        female,
        age: age.and_then(|age| u32::try_from(age).ok()),
        appearance: PlayerAppearance::unpack(appearance?)?,
    })
}

fn encode_png(image: &FaceImage) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, image.width, image.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Fast);
    let mut writer = encoder.write_header().ok()?;
    writer.write_image_data(&image.rgba).ok()?;
    writer.finish().ok()?;
    Some(bytes)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::bundle::tests::{sample_bundle, write};
    use super::*;
    use crate::db::Db;

    /// The golden snapshot, with its one player (uid 77, aged 26) given this appearance.
    pub(crate) fn snapshot(
        appearance: Option<[u8; 7]>,
        gender: &str,
    ) -> (tempfile::TempDir, Db, i64) {
        let (dir, db) = crate::db::open_with_golden_snapshot();
        let snapshot_id = {
            let conn = db.0.lock().unwrap();
            let packed =
                appearance.map(|values| PlayerAppearance::from_dump(values).unwrap().pack());
            conn.execute(
                "UPDATE players SET appearance = ?1, gender = ?2 WHERE uid = 77",
                (packed, gender),
            )
            .unwrap();
            conn.query_row("SELECT id FROM snapshots WHERE is_current = 1", [], |row| {
                row.get(0)
            })
            .unwrap()
        };
        (dir, db, snapshot_id)
    }

    /// Game faces over the sample bundle: a blue female face, a red male face, and a male face
    /// of ethnicity 9 that has no texture.
    pub(crate) fn sample_faces() -> (tempfile::TempDir, PathBuf, GameFaces) {
        let (dir, path) = write(&sample_bundle());
        let faces = GameFaces::new(Some(path.clone()));
        (dir, path, faces)
    }

    /// The first pixel of the face served for uid 77, or `None` when there is no face.
    pub(crate) fn first_pixel(result: ImageLookupResult) -> Option<[u8; 4]> {
        let ImageLookupResult::Available { bytes, mime } = result else {
            return None;
        };
        assert_eq!(mime, "image/png");
        let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
            .read_info()
            .unwrap();
        let mut pixels = [0; 4 * 4 * 4];
        reader.next_frame(&mut pixels).unwrap();
        Some(pixels[..4].try_into().unwrap())
    }

    const RED: [u8; 4] = [255, 0, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 255];
    const WHITE_MALE: [u8; 7] = [0, 3, 2, 11, 0, 2, 0];

    #[test]
    fn a_generated_player_gets_the_bundles_nearest_face_of_his_gender() {
        let (_bundle_dir, _path, faces) = sample_faces();
        assert!(faces.available());

        let (_dir, db, snapshot_id) = snapshot(Some(WHITE_MALE), "male");
        let male = faces.resolve(&db.read().unwrap(), snapshot_id, 77);
        let (_dir, db, snapshot_id) = snapshot(Some(WHITE_MALE), "female");
        let female = faces.resolve(&db.read().unwrap(), snapshot_id, 77);

        assert_eq!(first_pixel(male), Some(RED));
        assert_eq!(first_pixel(female), Some(BLUE));
    }

    #[test]
    fn players_without_a_usable_appearance_have_no_face() {
        let (_bundle_dir, _path, faces) = sample_faces();
        let missing = |appearance, gender, snapshot_offset, uid| {
            let (_dir, db, snapshot_id) = snapshot(appearance, gender);
            let conn = db.read().unwrap();
            let result = faces.resolve(&conn, snapshot_id + snapshot_offset, uid);
            result == ImageLookupResult::Missing
        };

        // A player FM did not generate, one of unknown gender, an unknown player, another
        // snapshot, and a generated player whose nearest face has no texture.
        assert!(missing(None, "male", 0, 77));
        assert!(missing(Some(WHITE_MALE), "unknown", 0, 77));
        assert!(missing(Some(WHITE_MALE), "male", 0, 78));
        assert!(missing(Some(WHITE_MALE), "male", 1, 77));
        assert!(missing(Some([9, 3, 2, 11, 0, 2, 0]), "male", 0, 77));
        assert!(!missing(Some(WHITE_MALE), "male", 0, 77));
    }

    #[test]
    fn a_bundle_that_cannot_be_read_turns_game_faces_off_until_it_changes() {
        let (_bundle_dir, path, faces) = sample_faces();
        let (_dir, db, snapshot_id) = snapshot(Some(WHITE_MALE), "male");
        let face = || first_pixel(faces.resolve(&db.read().unwrap(), snapshot_id, 77));
        assert_eq!(face(), Some(RED));

        std::fs::write(&path, b"a mod installer left this file half written").unwrap();
        assert_eq!(face(), None);
        assert!(!faces.available());

        std::fs::write(&path, sample_bundle()).unwrap();
        assert_eq!(face(), Some(RED));
        assert!(faces.available());
    }

    #[test]
    fn no_bundle_means_no_faces() {
        let (_dir, db, snapshot_id) = snapshot(Some(WHITE_MALE), "male");
        let absent = tempfile::tempdir()
            .unwrap()
            .path()
            .join("portraits_assets_all.bundle");

        for faces in [GameFaces::new(None), GameFaces::new(Some(absent))] {
            assert!(!faces.available());
            assert_eq!(
                faces.resolve(&db.read().unwrap(), snapshot_id, 77),
                ImageLookupResult::Missing
            );
        }
    }

    /// Private check of the reader against a real portrait bundle; see `./scripts/dev face-reference`.
    #[test]
    #[ignore = "reads the private bundle named by FM_VALUESCOUT_PORTRAIT_BUNDLE"]
    fn face_reference_harness() {
        let path =
            PathBuf::from(std::env::var_os("FM_VALUESCOUT_PORTRAIT_BUNDLE").expect("bundle"));
        let started = Instant::now();
        let bundle = PortraitBundle::open(&path).unwrap_or_else(|error| panic!("{}", error.0));
        let load_ms = started.elapsed().as_millis();
        let faces = bundle.faces();
        let with_texture = (0..faces.len())
            .filter(|index| faces[*index].has_texture())
            .count();

        // Every 97th face spreads the sample over the whole file.
        let mut decode_ms = Vec::new();
        let mut png_bytes = Vec::new();
        let mut sizes = std::collections::BTreeSet::new();
        for index in (0..faces.len()).step_by(97) {
            let started = Instant::now();
            let image = bundle
                .read_face(&path, index)
                .unwrap_or_else(|error| panic!("{}", error.0))
                .expect("texture");
            sizes.insert((image.width, image.height));
            png_bytes.push(encode_png(&image).expect("png").len());
            decode_ms.push(started.elapsed().as_secs_f64() * 1000.0);
        }
        decode_ms.sort_by(f64::total_cmp);
        png_bytes.sort_unstable();

        let fixture = zstd::decode_all(&include_bytes!("fixtures/catalog.bin.zst")[..]).unwrap();
        let catalog: Vec<u8> = faces.iter().flat_map(|face| face.properties).collect();
        println!(
            "{}",
            serde_json::json!({
                "test": "face_reference_harness",
                "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
                "faces": faces.len(),
                "facesWithTexture": with_texture,
                "loadMs": load_ms,
                "decodedFaces": decode_ms.len(),
                "decodeMsMedian": decode_ms[decode_ms.len() / 2],
                "pngBytesMedian": png_bytes[png_bytes.len() / 2],
                "sizes": sizes.iter().map(|(w, h)| format!("{w}x{h}")).collect::<Vec<_>>(),
                "catalogMatchesFixture": catalog == fixture,
            })
        );
    }
}
