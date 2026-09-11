use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt, OpenOptionsMaybeDirExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, File, OpenOptions};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use std::io::{self, BufReader, Read};
use std::path::Path;

const MAX_DEPTH: usize = 32;
const MAX_CONFIGS: usize = 10_000;
const MAX_ENTRIES: usize = 1_000_000;
const MAX_CONFIG_BYTES: u64 = 64 * 1024 * 1024;
const MAX_PARSER_BYTES: u64 = 256 * 1024 * 1024;
const MAX_CONFIG_RECORDS: usize = 2_000_000;
const MAX_PARSER_RECORDS: usize = 10_000_000;
const MAX_CONFIG_ATTRIBUTES: usize = 8_000_000;
const MAX_PARSER_ATTRIBUTES: usize = 40_000_000;
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
    pub config_byte_limit: usize,
    pub config_record_limit: usize,
    pub config_attribute_limit: usize,
    pub parser_byte_limit: usize,
    pub parser_record_limit: usize,
    pub parser_attribute_limit: usize,
}
#[derive(Clone, Copy, Debug)]
struct Limits {
    depth: usize,
    configs: usize,
    entries: usize,
    config_bytes: u64,
    parser_bytes: u64,
    config_records: usize,
    parser_records: usize,
    config_attributes: usize,
    parser_attributes: usize,
    mappings: usize,
    image_bytes: u64,
}
const PRODUCTION_LIMITS: Limits = Limits {
    depth: MAX_DEPTH,
    configs: MAX_CONFIGS,
    entries: MAX_ENTRIES,
    config_bytes: MAX_CONFIG_BYTES,
    parser_bytes: MAX_PARSER_BYTES,
    config_records: MAX_CONFIG_RECORDS,
    parser_records: MAX_PARSER_RECORDS,
    config_attributes: MAX_CONFIG_ATTRIBUTES,
    parser_attributes: MAX_PARSER_ATTRIBUTES,
    mappings: MAX_MAPPINGS,
    image_bytes: MAX_IMAGE_BYTES,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct Identity(Vec<String>);
impl Ord for Identity {
    fn cmp(&self, other: &Self) -> Ordering {
        fn components(parts: &[String]) -> impl Iterator<Item = char> + '_ {
            parts.iter().enumerate().flat_map(|(index, part)| {
                part.chars().chain((index + 1 < parts.len()).then_some('/'))
            })
        }
        components(&self.0).cmp(components(&other.0))
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

#[derive(Debug)]
pub(crate) struct ImageLocator {
    root: Dir,
    path: Vec<String>,
    limit: u64,
}
impl ImageLocator {
    pub(crate) fn read(self) -> Option<ImageResult> {
        read_image(&self.root, &self.path, self.limit).ok()
    }
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
        if !discover(&root_dir, &mut Vec::new(), 0, &mut state) {
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
        let mut parser_bytes = 0;
        let mut parser_records = 0;
        let mut parser_attributes = 0;
        for identity in configs {
            index.summary.configs += 1;
            match parse_config(
                &mut index,
                &identity,
                &mut parser_bytes,
                &mut parser_records,
                &mut parser_attributes,
            ) {
                ParseConfigResult::Complete { mappings, parent } => {
                    for candidate in mappings {
                        add_mapping(
                            &mut index,
                            &parent,
                            &candidate.parent,
                            &candidate.from,
                            &candidate.to,
                        );
                    }
                }
                ParseConfigResult::RootLimit => {
                    index.summary.truncated = true;
                    break;
                }
                ParseConfigResult::Discarded => {}
            }
        }
        index
    }
    pub fn summary(&self) -> &GraphicsSummary {
        &self.summary
    }
    pub(crate) fn resolve_locator(&self, kind: GraphicsKind, uid: u32) -> Option<ImageLocator> {
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
        Some(ImageLocator {
            root: self.root.as_ref()?.try_clone().ok()?,
            path: mapping.path.0.clone(),
            limit: self.limits.image_bytes,
        })
    }

    #[cfg(test)]
    pub fn resolve(&self, kind: GraphicsKind, uid: u32) -> Option<ImageResult> {
        self.resolve_locator(kind, uid)?.read()
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
    #[cfg(test)]
    pub(crate) fn calibration_requests(&self) -> Vec<(GraphicsKind, u32)> {
        self.people
            .keys()
            .map(|uid| (GraphicsKind::PersonPortrait, *uid))
            .chain(self.clubs.keys().flat_map(|uid| {
                [
                    (GraphicsKind::ClubLogo, *uid),
                    (GraphicsKind::ClubIcon, *uid),
                ]
            }))
            .collect()
    }
}
struct Discovery {
    configs: BinaryHeap<Identity>,
    entries: usize,
    limits: Limits,
    summary: GraphicsSummary,
}
fn discover(dir: &Dir, identity: &mut Vec<String>, depth: usize, state: &mut Discovery) -> bool {
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
        if !admit_entry(state) {
            return false;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        identity.push(name.clone());
        let ty = match entry.file_type() {
            Ok(t) => t,
            Err(_) => {
                identity.pop();
                state.summary.diagnostics.source_unreadable += 1;
                return true;
            }
        };
        let result = if ty.is_symlink() {
            true
        } else if ty.is_dir() {
            let child = match open_dir(dir, &name) {
                Ok(d) => d,
                Err(_) => {
                    identity.pop();
                    state.summary.diagnostics.source_unreadable += 1;
                    return true;
                }
            };
            discover(&child, identity, depth + 1, state)
        } else {
            if ty.is_file() && name == "config.xml" {
                retain_config(state, Identity(identity.clone()));
            }
            true
        };
        identity.pop();
        result
    })
}

fn admit_entry(state: &mut Discovery) -> bool {
    state.entries += 1;
    state.entries <= state.limits.entries
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
#[derive(Debug)]
struct ConfigCandidate {
    from: String,
    to: String,
    parent: Vec<String>,
}

enum ParseConfigResult {
    Complete {
        mappings: Vec<ConfigCandidate>,
        parent: Dir,
    },
    Discarded,
    RootLimit,
}

struct CountingReader {
    file: File,
    config_bytes: u64,
    root_bytes: u64,
    config_limit: u64,
    root_limit: u64,
    exceeded: bool,
}
impl Read for CountingReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.exceeded {
            return Ok(0);
        }
        let remaining_config = self.config_limit.saturating_sub(self.config_bytes);
        let remaining_root = self.root_limit.saturating_sub(self.root_bytes);
        let allowed = remaining_config.min(remaining_root);
        let request = (allowed + 1).min(buf.len() as u64) as usize;
        if request == 0 {
            self.exceeded = true;
            return Ok(0);
        }
        let read = self.file.read(&mut buf[..request])?;
        self.config_bytes += read as u64;
        self.root_bytes += read as u64;
        if self.config_bytes > self.config_limit || self.root_bytes > self.root_limit {
            self.exceeded = true;
        }
        Ok(read)
    }
}

fn parse_config(
    index: &mut GraphicsIndex,
    identity: &Identity,
    root_bytes: &mut u64,
    root_records: &mut usize,
    root_attributes: &mut usize,
) -> ParseConfigResult {
    let Some((parent, name)) = split_identity(identity) else {
        return ParseConfigResult::Discarded;
    };
    let Some(root) = index.root.as_ref() else {
        return ParseConfigResult::Discarded;
    };
    let dir = match open_parent(root, parent) {
        Ok(d) => d,
        Err(_) => {
            index.summary.diagnostics.config_unreadable += 1;
            return ParseConfigResult::Discarded;
        }
    };
    let file = match open_file(&dir, name) {
        Ok(file) => file,
        Err(_) => {
            index.summary.diagnostics.config_unreadable += 1;
            return ParseConfigResult::Discarded;
        }
    };
    let mut source = CountingReader {
        file,
        config_bytes: 0,
        root_bytes: *root_bytes,
        config_limit: index.limits.config_bytes,
        root_limit: index.limits.parser_bytes,
        exceeded: false,
    };
    let mut reader = Reader::from_reader(BufReader::new(&mut source));
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut mappings = Vec::new();
    let mut config_records = 0;
    let mut config_attributes = 0;
    let mut malformed = false;
    let mut local_limit = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) | Ok(Event::Start(e)) if e.name().as_ref() == b"record" => {
                config_records += 1;
                *root_records += 1;
                if config_records > index.limits.config_records {
                    local_limit = true;
                }
                if *root_records > index.limits.parser_records {
                    local_limit = true;
                }
                let mut from = None;
                let mut to = None;
                for attr in e.attributes() {
                    config_attributes += 1;
                    *root_attributes += 1;
                    if config_attributes > index.limits.config_attributes {
                        local_limit = true;
                    }
                    if *root_attributes > index.limits.parser_attributes {
                        local_limit = true;
                    }
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
                        from = Some(value.into_owned());
                    } else if attr.key.as_ref() == b"to" {
                        to = Some(value.into_owned());
                    }
                }
                if malformed {
                    break;
                }
                if let (Some(from), Some(to)) = (from, to) {
                    mappings.push(ConfigCandidate {
                        from,
                        to,
                        parent: parent.to_vec(),
                    });
                } else {
                    index.summary.diagnostics.invalid_mapping += 1;
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
    drop(reader);
    *root_bytes = source.root_bytes;
    if source.exceeded {
        if source.root_bytes > index.limits.parser_bytes {
            index.summary.diagnostics.parser_byte_limit += 1;
            return ParseConfigResult::RootLimit;
        }
        if source.config_bytes > index.limits.config_bytes {
            index.summary.diagnostics.config_byte_limit += 1;
            index.summary.diagnostics.config_too_large += 1;
            return ParseConfigResult::Discarded;
        }
        index.summary.diagnostics.parser_byte_limit += 1;
        return ParseConfigResult::RootLimit;
    }
    if *root_records > index.limits.parser_records
        || *root_attributes > index.limits.parser_attributes
    {
        index.summary.diagnostics.parser_record_limit +=
            usize::from(*root_records > index.limits.parser_records);
        index.summary.diagnostics.parser_attribute_limit +=
            usize::from(*root_attributes > index.limits.parser_attributes);
        return ParseConfigResult::RootLimit;
    }
    if config_records > index.limits.config_records
        || config_attributes > index.limits.config_attributes
        || local_limit
    {
        index.summary.diagnostics.config_record_limit +=
            usize::from(config_records > index.limits.config_records);
        index.summary.diagnostics.config_attribute_limit +=
            usize::from(config_attributes > index.limits.config_attributes);
        return ParseConfigResult::Discarded;
    }
    if malformed {
        index.summary.diagnostics.malformed_config += 1;
        return ParseConfigResult::Discarded;
    }
    ParseConfigResult::Complete {
        mappings,
        parent: dir,
    }
}
fn add_mapping(
    index: &mut GraphicsIndex,
    config_dir: &Dir,
    config_parent: &[String],
    source: &str,
    target: &str,
) {
    let Some((kind, uid)) = parse_target(target) else {
        index.summary.diagnostics.invalid_mapping += 1;
        return;
    };
    let Some(source_parts) = source_relative_identity(source) else {
        index.summary.diagnostics.source_unreadable += 1;
        return;
    };
    let Some(source) = source_identity(config_parent, source) else {
        index.summary.diagnostics.source_unreadable += 1;
        return;
    };
    let probe = source_parts.last().is_some_and(|x| !x.contains('.'));
    let path = source_candidates(&source, probe);
    let Some(path) = path.into_iter().find(|candidate| {
        let relative = candidate.0[config_parent.len()..].to_vec();
        source_exists_relative(config_dir, &relative)
    }) else {
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
fn source_relative_identity(source: &str) -> Option<Vec<String>> {
    let path = Path::new(source);
    if path.is_absolute() {
        return None;
    }
    path.components()
        .map(|component| match component {
            std::path::Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect()
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
fn source_exists_relative(parent: &Dir, identity: &[String]) -> bool {
    let Some((name, components)) = identity.split_last() else {
        return false;
    };
    let Ok(dir) = open_relative(parent, components) else {
        return false;
    };
    let Ok(file) = open_file(&dir, name) else {
        return false;
    };
    file.metadata().map(|m| m.is_file()).unwrap_or(false)
}
fn open_relative(root: &Dir, components: &[String]) -> std::io::Result<Dir> {
    let mut current = root.try_clone()?;
    for component in components {
        current = open_dir(&current, component)?;
    }
    Ok(current)
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
    fn entry_budget_equality_accepts_candidate() {
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
        assert!(!i.summary.truncated);
        assert_eq!(i.people_len(), 1);
    }

    #[test]
    fn entry_budget_overflow_discards_candidate() {
        let d = tempdir().unwrap();
        png(&d.path().join("p.png"));
        fs::write(d.path().join("extra.txt"), b"extra").unwrap();
        fs::write(
            d.path().join("config.xml"),
            r#"<record from="p.png" to="graphics/pictures/person/1/portrait"/>"#,
        )
        .unwrap();
        let index = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                entries: 2,
                ..PRODUCTION_LIMITS
            },
        );
        assert!(index.summary().truncated);
        assert_eq!(index.summary().diagnostics.entry_limit, 1);
        assert_eq!(index.people_len(), 0);
    }

    #[test]
    fn generated_entry_stream_over_legacy_limit_fails_on_first_excess() {
        let mut state = Discovery {
            configs: BinaryHeap::new(),
            entries: 0,
            limits: Limits {
                entries: MAX_ENTRIES,
                ..PRODUCTION_LIMITS
            },
            summary: GraphicsSummary::default(),
        };
        let mut visited = 0;
        let completed = consume_entries((0..=MAX_ENTRIES).map(Ok), &mut state, |_, state| {
            if !admit_entry(state) {
                return false;
            }
            visited += 1;
            true
        });
        assert!(!completed);
        assert_eq!(state.entries, MAX_ENTRIES + 1);
        assert_eq!(visited, MAX_ENTRIES);
    }
    #[test]
    fn missing_source_does_not_discard_valid_sibling() {
        let d = tempdir().unwrap();
        png(&d.path().join("ok.png"));
        fs::write(
            d.path().join("config.xml"),
            r#"<root><record from="missing.png" to="graphics/pictures/person/2/portrait"/><record from="ok.png" to="graphics/pictures/person/3/portrait"/></root>"#,
        )
        .unwrap();
        let index = GraphicsIndex::scan(d.path());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 2).is_none());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 3).is_some());
        assert_eq!(index.summary().diagnostics.source_unreadable, 1);
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
    fn malformed_after_valid_record_has_no_source_validation_side_effect() {
        let d = tempdir().unwrap();
        fs::write(
            d.path().join("config.xml"),
            r#"<root><record from="missing.png" to="graphics/pictures/person/2/portrait"/><record from="ok.png" to="graphics/pictures/person/3/portrait" broken="&invalid;"/></root>"#,
        )
        .unwrap();
        let index = GraphicsIndex::scan(d.path());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 2).is_none());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 3).is_none());
        assert_eq!(index.summary().diagnostics.source_unreadable, 0);
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
        assert!(!source_exists_relative(
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
    fn streaming_parser_accepts_config_larger_than_legacy_byte_limit() {
        let d = tempdir().unwrap();
        png(&d.path().join("p.png"));
        let padding = " ".repeat(9 * 1024 * 1024);
        fs::write(
            d.path().join("config.xml"),
            format!(r#"<root>{padding}<record from="p.png" to="graphics/pictures/person/39/portrait"/></root>"#),
        )
        .unwrap();
        let index = GraphicsIndex::scan(d.path());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 39).is_some());
    }

    #[test]
    fn per_config_record_limit_discards_only_that_config() {
        let d = tempdir().unwrap();
        png(&d.path().join("p.png"));
        fs::write(
            d.path().join("config.xml"),
            r#"<root><record from="p.png" to="graphics/pictures/person/1/portrait"/><record from="p.png" to="graphics/pictures/person/2/portrait"/></root>"#,
        )
        .unwrap();
        let index = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                config_records: 1,
                ..PRODUCTION_LIMITS
            },
        );
        assert!(index.resolve(GraphicsKind::PersonPortrait, 1).is_none());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 2).is_none());
        assert_eq!(index.summary().diagnostics.config_record_limit, 1);
    }

    #[test]
    fn root_parser_limit_keeps_only_complete_lexical_predecessors() {
        let d = tempdir().unwrap();
        let first = d.path().join("a");
        let second = d.path().join("b");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        png(&first.join("a.png"));
        png(&second.join("b.png"));
        fs::write(
            first.join("config.xml"),
            r#"<record from="a.png" to="graphics/pictures/person/1/portrait"/>"#,
        )
        .unwrap();
        fs::write(
            second.join("config.xml"),
            r#"<record from="b.png" to="graphics/pictures/person/2/portrait"/>"#,
        )
        .unwrap();
        let first_size = fs::metadata(first.join("config.xml")).unwrap().len();
        let index = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                parser_bytes: first_size + 1,
                ..PRODUCTION_LIMITS
            },
        );
        assert!(index.summary().truncated);
        assert!(index.resolve(GraphicsKind::PersonPortrait, 1).is_some());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 2).is_none());
        assert_eq!(index.summary().diagnostics.parser_byte_limit, 1);
    }

    #[test]
    fn per_config_byte_limit_discards_only_that_config() {
        let d = tempdir().unwrap();
        let first = d.path().join("a");
        let second = d.path().join("b");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        png(&second.join("b.png"));
        let first_config = format!(
            "<root>{}<record from=\"missing.png\" to=\"graphics/pictures/person/1/portrait\"/></root>",
            " ".repeat(512),
        );
        fs::write(first.join("config.xml"), &first_config).unwrap();
        let second_config = r#"<record from="b.png" to="graphics/pictures/person/2/portrait"/>"#;
        fs::write(second.join("config.xml"), second_config).unwrap();
        let second_size = fs::metadata(second.join("config.xml")).unwrap().len();
        let config_limit = 128;
        let first_read = config_limit + 1;
        let parser_limit = first_read + second_size + 10;
        assert!(first_config.len() as u64 > config_limit);
        assert!(second_size <= config_limit);
        assert!(first_read + second_size <= parser_limit);
        assert!(first_config.len() as u64 > parser_limit);
        let index = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                config_bytes: config_limit,
                parser_bytes: parser_limit,
                ..PRODUCTION_LIMITS
            },
        );
        assert!(!index.summary().truncated);
        assert!(index.resolve(GraphicsKind::PersonPortrait, 2).is_some());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 1).is_none());
        assert_eq!(index.summary().diagnostics.config_byte_limit, 1);
        assert_eq!(index.summary().diagnostics.config_too_large, 1);
        assert_eq!(index.summary().diagnostics.parser_byte_limit, 0);
    }

    #[test]
    fn simultaneous_actual_byte_breach_returns_root_truncation() {
        let d = tempdir().unwrap();
        let first = d.path().join("a");
        let second = d.path().join("b");
        fs::create_dir_all(&first).unwrap();
        fs::create_dir_all(&second).unwrap();
        png(&first.join("a.png"));
        png(&second.join("b.png"));
        let first_config = r#"<record from="a.png" to="graphics/pictures/person/1/portrait"/>"#;
        let second_config = format!(
            "<root>{}<record from=\"b.png\" to=\"graphics/pictures/person/2/portrait\"/></root>",
            " ".repeat(256),
        );
        fs::write(first.join("config.xml"), first_config).unwrap();
        fs::write(second.join("config.xml"), &second_config).unwrap();
        let first_size = fs::metadata(first.join("config.xml")).unwrap().len();
        let second_size = fs::metadata(second.join("config.xml")).unwrap().len();
        let config_limit = second_size - 1;
        let parser_limit = first_size + config_limit;
        assert!(first_size <= config_limit);
        assert!(second_size > config_limit);
        assert!(first_size + (config_limit + 1) > parser_limit);
        let index = GraphicsIndex::scan_with_limits(
            d.path(),
            Limits {
                config_bytes: config_limit,
                parser_bytes: parser_limit,
                ..PRODUCTION_LIMITS
            },
        );
        assert!(index.summary().truncated);
        assert!(index.resolve(GraphicsKind::PersonPortrait, 1).is_some());
        assert!(index.resolve(GraphicsKind::PersonPortrait, 2).is_none());
        assert_eq!(index.summary().diagnostics.parser_byte_limit, 1);
        assert_eq!(index.summary().diagnostics.config_byte_limit, 0);
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
