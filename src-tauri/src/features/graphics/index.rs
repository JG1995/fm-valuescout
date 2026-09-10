use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsMaybeDirExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, File, OpenOptions};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use std::io::Read;
use std::path::Path;

const MAX_DEPTH: usize = 32;
const MAX_CONFIGS: usize = 10_000;
const MAX_ENTRIES: usize = 1_000_000;
const MAX_CONFIG_BYTES: u64 = 8 * 1024 * 1024;
const MAX_MAPPINGS: usize = 500_000;
const MAX_IMAGE_BYTES: u64 = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GraphicsKind {
    PersonPortrait,
    ClubLogo,
    ClubIcon,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GraphicsSummary {
    pub configs: usize,
    pub mappings: usize,
    pub diagnostics: ScanDiagnostics,
    pub truncated: bool,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScanDiagnostics {
    pub config_limit: usize,
    pub entry_limit: usize,
    pub depth_limit: usize,
    pub mapping_limit: usize,
    pub config_too_large: usize,
    pub config_unreadable: usize,
    pub malformed_config: usize,
    pub invalid_mapping: usize,
    pub source_unreadable: usize,
}
#[derive(Clone, Copy, Debug)]
struct Limits {
    depth: usize,
    configs: usize,
    entries: usize,
    config_bytes: u64,
    mappings: usize,
    image_bytes: u64,
}
const PRODUCTION_LIMITS: Limits = Limits {
    depth: MAX_DEPTH,
    configs: MAX_CONFIGS,
    entries: MAX_ENTRIES,
    config_bytes: MAX_CONFIG_BYTES,
    mappings: MAX_MAPPINGS,
    image_bytes: MAX_IMAGE_BYTES,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct Identity(Vec<String>);
impl Ord for Identity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.join("/").cmp(&other.0.join("/"))
    }
}
impl PartialOrd for Identity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
#[derive(Debug)]
struct Mapping {
    path: Identity,
}
#[derive(Debug)]
pub struct GraphicsIndex {
    root: Option<Dir>,
    people: BTreeMap<u32, Mapping>,
    clubs: BTreeMap<u32, (Option<Mapping>, Option<Mapping>)>,
    summary: GraphicsSummary,
    limits: Limits,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageResult {
    pub bytes: Vec<u8>,
    pub mime: &'static str,
}

impl GraphicsIndex {
    pub fn empty() -> Self {
        Self {
            root: None,
            people: BTreeMap::new(),
            clubs: BTreeMap::new(),
            summary: GraphicsSummary::default(),
            limits: PRODUCTION_LIMITS,
        }
    }

    pub fn scan(root: &Path) -> Self {
        Self::scan_with_limits(root, PRODUCTION_LIMITS)
    }
    fn scan_with_limits(root: &Path, limits: Limits) -> Self {
        let root_dir = match Dir::open_ambient_dir(root, ambient_authority()) {
            Ok(dir) => dir,
            Err(_) => {
                return Self {
                    root: None,
                    people: BTreeMap::new(),
                    clubs: BTreeMap::new(),
                    summary: GraphicsSummary {
                        diagnostics: ScanDiagnostics {
                            source_unreadable: 1,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    limits,
                }
            }
        };
        let mut state = Discovery {
            configs: BinaryHeap::new(),
            entries: 0,
            limits,
            summary: GraphicsSummary::default(),
        };
        if !discover(&root_dir, &[], 0, &mut state) {
            state.summary.truncated = true;
            state.summary.diagnostics.entry_limit += 1;
        }
        if state.summary.truncated {
            return Self {
                root: Some(root_dir),
                people: BTreeMap::new(),
                clubs: BTreeMap::new(),
                summary: state.summary,
                limits,
            };
        }
        let mut configs = state.configs.into_vec();
        configs.sort();
        let mut index = Self {
            root: Some(root_dir),
            people: BTreeMap::new(),
            clubs: BTreeMap::new(),
            summary: state.summary,
            limits,
        };
        for identity in configs {
            index.summary.configs += 1;
            parse_config(&mut index, &identity);
        }
        index
    }
    pub fn summary(&self) -> &GraphicsSummary {
        &self.summary
    }
    pub fn resolve(&self, kind: GraphicsKind, uid: u32) -> Option<ImageResult> {
        if uid == 0 {
            return None;
        }
        let mapping = match kind {
            GraphicsKind::PersonPortrait => self.people.get(&uid),
            GraphicsKind::ClubLogo => self
                .clubs
                .get(&uid)
                .and_then(|x| x.0.as_ref().or(x.1.as_ref())),
            GraphicsKind::ClubIcon => self.clubs.get(&uid).and_then(|x| x.1.as_ref()),
        }?;
        read_image(
            self.root.as_ref()?,
            &mapping.path.0,
            self.limits.image_bytes,
        )
        .ok()
    }
    #[cfg(test)]
    pub fn people_len(&self) -> usize {
        self.people.len()
    }
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn clubs_len(&self) -> usize {
        self.clubs.len()
    }
}
struct Discovery {
    configs: BinaryHeap<Identity>,
    entries: usize,
    limits: Limits,
    summary: GraphicsSummary,
}
fn discover(dir: &Dir, parent: &[String], depth: usize, state: &mut Discovery) -> bool {
    if depth > state.limits.depth {
        state.summary.diagnostics.depth_limit += 1;
        return true;
    }
    let entries = match dir.entries() {
        Ok(e) => e,
        Err(_) => {
            state.summary.diagnostics.source_unreadable += 1;
            return true;
        }
    };
    consume_entries(entries, state, |entry, state| {
        state.entries += 1;
        if state.entries >= state.limits.entries {
            return false;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let mut identity = parent.to_vec();
        identity.push(name.clone());
        let ty = match entry.file_type() {
            Ok(t) => t,
            Err(_) => {
                state.summary.diagnostics.source_unreadable += 1;
                return true;
            }
        };
        if ty.is_symlink() {
            return true;
        }
        if ty.is_dir() {
            let child = match open_dir(dir, &name) {
                Ok(d) => d,
                Err(_) => {
                    state.summary.diagnostics.source_unreadable += 1;
                    return true;
                }
            };
            discover(&child, &identity, depth + 1, state)
        } else {
            if ty.is_file() && name == "config.xml" {
                retain_config(state, Identity(identity));
            }
            true
        }
    })
}

fn consume_entries<I, T, F>(entries: I, state: &mut Discovery, mut visit: F) -> bool
where
    I: Iterator<Item = std::io::Result<T>>,
    F: FnMut(T, &mut Discovery) -> bool,
{
    for item in entries {
        match item {
            Ok(entry) => {
                if !visit(entry, state) {
                    return false;
                }
            }
            Err(_) => {
                state.summary.diagnostics.source_unreadable += 1;
            }
        }
    }
    true
}
fn retain_config(state: &mut Discovery, identity: Identity) {
    if state.configs.len() < state.limits.configs {
        state.configs.push(identity);
        return;
    }
    state.summary.diagnostics.config_limit += 1;
    if let Some(largest) = state.configs.peek() {
        if identity < *largest {
            state.configs.pop();
            state.configs.push(identity);
        }
    }
}
fn open_dir(parent: &Dir, name: &str) -> std::io::Result<Dir> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .follow(FollowSymlinks::No)
        .maybe_dir(true);
    let file = parent.open_with(name, &options)?;
    Ok(Dir::from_std_file(file.into_std()))
}
fn open_file(parent: &Dir, name: &str) -> std::io::Result<File> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .follow(FollowSymlinks::No)
        .maybe_dir(false);
    parent.open_with(name, &options)
}
fn open_parent(root: &Dir, components: &[String]) -> std::io::Result<Dir> {
    let mut current = root.try_clone()?;
    for component in components {
        current = open_dir(&current, component)?;
    }
    Ok(current)
}
fn split_identity(identity: &Identity) -> Option<(&[String], &str)> {
    let (last, parent) = identity.0.split_last()?;
    Some((parent, last))
}
enum ReadBoundedError {
    TooLarge,
    Unreadable,
}
fn read_bounded(file: File, limit: u64) -> Result<Vec<u8>, ReadBoundedError> {
    let meta = file.metadata().map_err(|_| ReadBoundedError::Unreadable)?;
    if !meta.is_file() {
        return Err(ReadBoundedError::Unreadable);
    }
    if meta.len() > limit {
        return Err(ReadBoundedError::TooLarge);
    }
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| ReadBoundedError::Unreadable)?;
    if bytes.len() as u64 > limit {
        Err(ReadBoundedError::TooLarge)
    } else {
        Ok(bytes)
    }
}
fn parse_config(index: &mut GraphicsIndex, identity: &Identity) {
    let Some((parent, name)) = split_identity(identity) else {
        return;
    };
    let dir = match open_parent(index.root.as_ref().ok_or(()).unwrap(), parent) {
        Ok(d) => d,
        Err(_) => {
            index.summary.diagnostics.config_unreadable += 1;
            return;
        }
    };
    let bytes = match open_file(&dir, name)
        .map_err(|_| ReadBoundedError::Unreadable)
        .and_then(|f| read_bounded(f, index.limits.config_bytes))
    {
        Ok(bytes) => bytes,
        Err(ReadBoundedError::TooLarge) => {
            index.summary.diagnostics.config_too_large += 1;
            return;
        }
        Err(ReadBoundedError::Unreadable) => {
            index.summary.diagnostics.config_unreadable += 1;
            return;
        }
    };
    let mut reader = Reader::from_reader(bytes.as_slice());
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut mappings = Vec::new();
    let mut malformed = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) if e.name().as_ref() == b"record" => {
                let mut from = None;
                let mut to = None;
                for attr in e.attributes() {
                    let attr = match attr {
                        Ok(attr) => attr,
                        Err(_) => {
                            malformed = true;
                            break;
                        }
                    };
                    let value = match attr.decoded_and_normalized_value(
                        quick_xml::XmlVersion::default(),
                        reader.decoder(),
                    ) {
                        Ok(value) => value,
                        Err(_) => {
                            malformed = true;
                            break;
                        }
                    };
                    if attr.key.as_ref() == b"from" {
                        from = Some(value.into_owned())
                    } else if attr.key.as_ref() == b"to" {
                        to = Some(value.into_owned())
                    }
                }
                if malformed {
                    break;
                }
                match (from, to) {
                    (Some(from), Some(to)) => mappings.push((from, to)),
                    _ => index.summary.diagnostics.invalid_mapping += 1,
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => {
                malformed = true;
                break;
            }
            _ => {}
        }
        buf.clear();
    }
    if malformed {
        index.summary.diagnostics.malformed_config += 1;
        return;
    }
    for (from, to) in mappings {
        add_mapping(index, parent, &from, &to);
    }
}
fn add_mapping(index: &mut GraphicsIndex, config_parent: &[String], source: &str, target: &str) {
    let Some((kind, uid)) = parse_target(target) else {
        index.summary.diagnostics.invalid_mapping += 1;
        return;
    };
    let Some(source) = source_identity(config_parent, source) else {
        index.summary.diagnostics.invalid_mapping += 1;
        return;
    };
    let path = source_candidates(&source, source.0.last().is_some_and(|x| !x.contains('.')));
    let Some(path) = path
        .into_iter()
        .find(|candidate| source_exists(index.root.as_ref().ok_or(()).unwrap(), &candidate.0))
    else {
        index.summary.diagnostics.source_unreadable += 1;
        return;
    };
    let duplicate = match kind {
        GraphicsKind::PersonPortrait => index.people.contains_key(&uid),
        GraphicsKind::ClubLogo => index.clubs.get(&uid).is_some_and(|x| x.0.is_some()),
        GraphicsKind::ClubIcon => index.clubs.get(&uid).is_some_and(|x| x.1.is_some()),
    };
    if duplicate {
        return;
    }
    if index.summary.mappings >= index.limits.mappings {
        index.summary.diagnostics.mapping_limit += 1;
        index.summary.truncated = true;
        return;
    }
    let mapping = Mapping { path };
    match kind {
        GraphicsKind::PersonPortrait => {
            index.people.insert(uid, mapping);
        }
        GraphicsKind::ClubLogo => {
            index.clubs.entry(uid).or_default().0 = Some(mapping);
        }
        GraphicsKind::ClubIcon => {
            index.clubs.entry(uid).or_default().1 = Some(mapping);
        }
    }
    index.summary.mappings += 1;
}
fn source_identity(parent: &[String], source: &str) -> Option<Identity> {
    let path = Path::new(source);
    if path.is_absolute() {
        return None;
    }
    let mut parts = parent.to_vec();
    for component in path.components() {
        match component {
            std::path::Component::Normal(value) => parts.push(value.to_string_lossy().into_owned()),
            _ => return None,
        }
    }
    Some(Identity(parts))
}
fn source_candidates(identity: &Identity, probe: bool) -> Vec<Identity> {
    if !probe {
        return vec![identity.clone()];
    }
    ["png", "jpeg", "jpg", "webp"]
        .iter()
        .map(|ext| {
            let mut p = identity.0.clone();
            let last = p.last_mut().unwrap();
            last.push('.');
            last.push_str(ext);
            Identity(p)
        })
        .collect()
}
fn source_exists(root: &Dir, identity: &[String]) -> bool {
    let Some((name, parent)) = identity.split_last() else {
        return false;
    };
    let Ok(dir) = open_parent(root, parent) else {
        return false;
    };
    let Ok(file) = open_file(&dir, name) else {
        return false;
    };
    file.metadata().map(|m| m.is_file()).unwrap_or(false)
}
fn parse_target(target: &str) -> Option<(GraphicsKind, u32)> {
    let mut p = target.split('/');
    if p.next()? != "graphics" || p.next()? != "pictures" {
        return None;
    }
    let category = p.next()?;
    let uid = p.next()?.parse().ok().filter(|x: &u32| *x > 0)?;
    let variant = p.next()?;
    if p.next().is_some() {
        return None;
    }
    Some((
        match (category, variant) {
            ("person", "portrait") => GraphicsKind::PersonPortrait,
            ("club" | "team", "logo") => GraphicsKind::ClubLogo,
            ("club" | "team", "icon") => GraphicsKind::ClubIcon,
            _ => return None,
        },
        uid,
    ))
}
fn read_image(root: &Dir, identity: &[String], limit: u64) -> Result<ImageResult, ()> {
    let (name, parent) = identity.split_last().ok_or(())?;
    let dir = open_parent(root, parent).map_err(|_| ())?;
    let file = open_file(&dir, name).map_err(|_| ())?;
    let bytes = read_bounded(file, limit).map_err(|_| ())?;
    let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        "image/jpeg"
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        return Err(());
    };
    Ok(ImageResult { bytes, mime })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    fn png(path: &Path) {
        fs::write(path, b"\x89PNG\r\n\x1a\n").unwrap();
    }
    #[test]
    fn entry_errors_are_reported_and_later_entries_are_processed() {
        let mut state = Discovery {
            configs: BinaryHeap::new(),
            entries: 0,
            limits: PRODUCTION_LIMITS,
            summary: GraphicsSummary::default(),
        };
        let mut visited = Vec::new();
        assert!(consume_entries(
            vec![
                Err(std::io::Error::other("unreadable entry")),
                Ok("valid"),
                Ok("later"),
            ]
            .into_iter(),
            &mut state,
            |entry, _| {
                visited.push(entry);
                true
            },
        ));
        assert_eq!(visited, ["valid", "later"]);
        assert_eq!(state.summary.diagnostics.source_unreadable, 1);
    }

    #[test]
    fn nested_and_deterministic_resolution() {
        let d = tempdir().unwrap();
        let a = d.path().join("a");
        fs::create_dir(&a).unwrap();
        png(&a.join("p.png"));
        fs::write(
            a.join("config.xml"),
            r#"<record from="p" to="graphics/pictures/person/2/portrait"/>"#,
        )
        .unwrap();
        let i = GraphicsIndex::scan(d.path());
        assert!(i.resolve(GraphicsKind::PersonPortrait, 2).is_some());
    }
    #[test]
    fn entry_budget_equality_discards_candidate() {
        let d = tempdir().unwrap();
        png(&d.path().join("p.png"));
        fs::write(
            d.path().join("config.xml"),
            r#"<record from="p.png" to="graphics/pictures/person/1/portrait"/>"#,
        )
        .unwrap();
        let i = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                entries: 2,
                ..PRODUCTION_LIMITS
            },
        );
        assert!(i.summary.truncated);
        assert_eq!(i.people_len(), 0);
    }
    #[test]
    fn malformed_record_attribute_discards_config_mappings() {
        let d = tempdir().unwrap();
        png(&d.path().join("p.png"));
        fs::write(
            d.path().join("config.xml"),
            r#"<record from="p.png" to="graphics/pictures/person/1/portrait" broken="&invalid;"/>"#,
        )
        .unwrap();
        let index = GraphicsIndex::scan(d.path());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 1).is_none());
        assert_eq!(index.summary().diagnostics.malformed_config, 1);
    }
    #[test]
    fn logo_beats_icon_and_probe_order_is_fixed() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("x.jpeg"), b"\xff\xd8\xff").unwrap();
        png(&d.path().join("x.png"));
        fs::write(d.path().join("config.xml"), r#"<record from="x.jpeg" to="graphics/pictures/club/4/icon"/><record from="x.png" to="graphics/pictures/club/4/logo"/>"#).unwrap();
        let i = GraphicsIndex::scan(d.path());
        assert_eq!(
            i.resolve(GraphicsKind::ClubLogo, 4).unwrap().mime,
            "image/png"
        );
        assert_eq!(
            i.resolve(GraphicsKind::ClubIcon, 4).unwrap().mime,
            "image/jpeg"
        );
    }
    #[cfg(unix)]
    #[test]
    fn no_follow_rejects_symlinked_directory_and_file() {
        use std::os::unix::fs::symlink;
        let d = tempdir().unwrap();
        let outside = tempdir().unwrap();
        png(&outside.path().join("portrait.png"));
        fs::write(
            d.path().join("config.xml"),
            r#"<record from="linked/portrait.png" to="graphics/pictures/person/9/portrait"/>"#,
        )
        .unwrap();
        symlink(outside.path(), d.path().join("linked")).unwrap();
        symlink(
            outside.path().join("portrait.png"),
            d.path().join("direct.png"),
        )
        .unwrap();
        assert!(GraphicsIndex::scan(d.path())
            .resolve(GraphicsKind::PersonPortrait, 9)
            .is_none());
        assert!(!source_exists(
            &Dir::open_ambient_dir(d.path(), ambient_authority()).unwrap(),
            &["direct.png".to_owned()]
        ));
    }
    #[cfg(windows)]
    #[test]
    fn windows_junction_and_reparse_points_are_not_traversed() {
        use std::process::Command;
        let d = tempdir().unwrap();
        let outside = tempdir().unwrap();
        png(&outside.path().join("portrait.png"));
        fs::write(
            outside.path().join("config.xml"),
            r#"<record from="portrait.png" to="graphics/pictures/person/11/portrait"/>"#,
        )
        .unwrap();
        let junction = d.path().join("linked");
        assert!(Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(outside.path())
            .status()
            .unwrap()
            .success());
        fs::write(
            d.path().join("config.xml"),
            r#"<record from="linked/portrait.png" to="graphics/pictures/person/10/portrait"/>"#,
        )
        .unwrap();
        let index = GraphicsIndex::scan(d.path());
        // The valid target config is behind the junction and must not be discovered.
        assert_eq!(index.summary().configs, 1);
        assert!(index.resolve(GraphicsKind::PersonPortrait, 11).is_none());
        // A source discovered in the real root must not resolve through that junction later.
        assert!(index.resolve(GraphicsKind::PersonPortrait, 10).is_none());
    }
    #[test]
    fn each_extensionless_fallback_is_independently_available() {
        for (extension, expected_mime, bytes) in [
            ("png", "image/png", b"\x89PNG\r\n\x1a\n".as_slice()),
            ("jpeg", "image/jpeg", b"\xff\xd8\xff".as_slice()),
            ("jpg", "image/jpeg", b"\xff\xd8\xff".as_slice()),
            ("webp", "image/webp", b"RIFF0000WEBP".as_slice()),
        ] {
            let d = tempdir().unwrap();
            fs::write(d.path().join(format!("x.{extension}")), bytes).unwrap();
            fs::write(
                d.path().join("config.xml"),
                r#"<record from="x" to="graphics/pictures/person/7/portrait"/>"#,
            )
            .unwrap();
            assert_eq!(
                GraphicsIndex::scan(d.path())
                    .resolve(GraphicsKind::PersonPortrait, 7)
                    .unwrap()
                    .mime,
                expected_mime
            );
        }
    }
    #[cfg(unix)]
    #[test]
    fn intermediate_directory_replacement_cannot_escape_root() {
        use std::os::unix::fs::symlink;
        let d = tempdir().unwrap();
        let nested = d.path().join("nested");
        let outside = tempdir().unwrap();
        fs::create_dir(&nested).unwrap();
        png(&nested.join("portrait.png"));
        fs::write(
            nested.join("config.xml"),
            r#"<record from="portrait.png" to="graphics/pictures/person/12/portrait"/>"#,
        )
        .unwrap();
        let index = GraphicsIndex::scan(d.path());
        fs::rename(&nested, d.path().join("replaced")).unwrap();
        png(&outside.path().join("portrait.png"));
        symlink(outside.path(), &nested).unwrap();
        assert!(index.resolve(GraphicsKind::PersonPortrait, 12).is_none());
    }
    #[test]
    fn global_lexical_order_controls_duplicate_precedence() {
        let d = tempdir().unwrap();
        let dotted = d.path().join("a.b");
        let plain = d.path().join("a");
        fs::create_dir(&dotted).unwrap();
        fs::create_dir(&plain).unwrap();
        fs::write(dotted.join("dotted.png"), b"\x89PNG\r\n\x1a\ndotted").unwrap();
        fs::write(plain.join("plain.png"), b"\x89PNG\r\n\x1a\nplain").unwrap();
        fs::write(
            dotted.join("config.xml"),
            r#"<record from="dotted.png" to="graphics/pictures/person/13/portrait"/>"#,
        )
        .unwrap();
        fs::write(
            plain.join("config.xml"),
            r#"<record from="plain.png" to="graphics/pictures/person/13/portrait"/>"#,
        )
        .unwrap();
        assert_eq!(
            GraphicsIndex::scan(d.path())
                .resolve(GraphicsKind::PersonPortrait, 13)
                .unwrap()
                .bytes,
            b"\x89PNG\r\n\x1a\ndotted"
        );
    }
    #[test]
    fn configured_bounds_are_reported_and_enforced() {
        let d = tempdir().unwrap();
        let deep = d.path().join("one").join("two");
        fs::create_dir_all(&deep).unwrap();
        png(&deep.join("deep.png"));
        fs::write(
            deep.join("config.xml"),
            r#"<record from="deep.png" to="graphics/pictures/person/30/portrait"/>"#,
        )
        .unwrap();
        let depth_limited = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                depth: 1,
                ..PRODUCTION_LIMITS
            },
        );
        assert_eq!(depth_limited.people_len(), 0);
        assert_eq!(depth_limited.summary().diagnostics.depth_limit, 1);

        let config = d.path().join("config.xml");
        fs::write(&config, vec![b'x'; 16]).unwrap();
        let byte_limited = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                config_bytes: 8,
                ..PRODUCTION_LIMITS
            },
        );
        assert_eq!(byte_limited.summary().diagnostics.config_too_large, 2);

        png(&d.path().join("one.png"));
        fs::write(
            &config,
            r#"<record from="one.png" to="graphics/pictures/person/31/portrait"/><record from="one.png" to="graphics/pictures/person/32/portrait"/>"#,
        )
        .unwrap();
        let mapping_limited = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                mappings: 1,
                ..PRODUCTION_LIMITS
            },
        );
        assert_eq!(mapping_limited.summary().mappings, 1);
        assert_eq!(mapping_limited.people_len(), 1);
        assert!(mapping_limited.summary().diagnostics.mapping_limit >= 1);
    }

    #[test]
    fn config_limit_retains_lexically_smallest_configs() {
        let d = tempdir().unwrap();
        for (name, image, uid) in [("a", "a.png", 40), ("b", "b.png", 41), ("c", "c.png", 42)] {
            let dir = d.path().join(name);
            fs::create_dir(&dir).unwrap();
            png(&dir.join(image));
            fs::write(
                dir.join("config.xml"),
                format!(r#"<record from="{image}" to="graphics/pictures/person/{uid}/portrait"/>"#),
            )
            .unwrap();
        }
        let index = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                configs: 2,
                ..PRODUCTION_LIMITS
            },
        );
        assert_eq!(index.summary().configs, 2);
        assert_eq!(index.summary().diagnostics.config_limit, 1);
        assert!(index.resolve(GraphicsKind::PersonPortrait, 40).is_some());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 41).is_some());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 42).is_none());
    }

    #[test]
    fn resolved_image_observes_removed_file_and_enforces_result_limit() {
        let d = tempdir().unwrap();
        let image = d.path().join("x.png");
        png(&image);
        fs::write(
            d.path().join("config.xml"),
            r#"<record from="x.png" to="graphics/pictures/person/50/portrait"/>"#,
        )
        .unwrap();
        let index = GraphicsIndex::scan(d.path());
        fs::remove_file(&image).unwrap();
        assert!(index.resolve(GraphicsKind::PersonPortrait, 50).is_none());

        png(&image);
        let limited = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                image_bytes: 4,
                ..PRODUCTION_LIMITS
            },
        );
        assert!(limited.resolve(GraphicsKind::PersonPortrait, 50).is_none());
    }

    #[test]
    fn invalid_source_and_signature_are_missing() {
        let d = tempdir().unwrap();
        fs::write(d.path().join("bad.png"), b"bad").unwrap();
        fs::write(d.path().join("config.xml"),r#"<record from="../bad.png" to="graphics/pictures/person/1/portrait"/><record from="bad.png" to="graphics/pictures/person/2/portrait"/>"#).unwrap();
        let i = GraphicsIndex::scan(d.path());
        assert!(i.resolve(GraphicsKind::PersonPortrait, 1).is_none());
        assert!(i.resolve(GraphicsKind::PersonPortrait, 2).is_none());
    }
}
