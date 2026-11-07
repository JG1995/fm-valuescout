//! FM27 layout values and the rules for choosing them.
//!
//! Ported from fmsave 0.10.0 (commit a71abe9b7b38e35affa05c49c3619ff05eb21795, MIT
//! License, Copyright (c) 2026 Randal S. Olson): `_layouts/__init__.py` (`find_layout` for
//! FM27 builds) and `_layouts/fm27_27_0_0.py`. See `THIRD_PARTY_NOTICES.md`.

use super::SaveStatsError;

/// The one FM27 build fmsave 0.10.0 registers layouts for.
pub(super) const REGISTERED_FM27_BUILD: &str = "27.0.0+2486695";
const REGISTERED_FM27_BUILDS: &[&str] = &[REGISTERED_FM27_BUILD];

/// Offsets inside `game_info`.
///
/// Offsets named `*_after_db_version` count from the end of the length-prefixed database
/// version string. The clock and late-build offsets assume an empty filename list; stored
/// filenames shift both by their length-prefixed byte sizes. The late build number follows
/// another variable field, so it is searched for in a window.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct GameInfoLayout {
    pub db_version_length_offset: usize,
    pub max_db_version_bytes: usize,
    pub build_number_offsets_after_db_version: &'static [usize],
    pub game_date_offset_after_db_version: usize,
    pub late_build_number_window_after_db_version: (usize, usize),
    pub filename_count_offset_after_db_version: usize,
    pub max_filenames: u32,
    pub max_filename_bytes: usize,
}

/// Version detection and the structured clock in `save_game_summary`.
///
/// The clock follows the setup and build strings, the counted division names, the manager
/// header, the manager and club strings, and the club uid.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct SaveSummaryLayout {
    pub max_version_bytes: usize,
    pub structured_strings_offset: usize,
    pub max_setup_string_bytes: usize,
    pub max_summary_name_bytes: usize,
    pub max_divisions: u32,
    pub manager_header_bytes: usize,
    pub club_uid_bytes: usize,
}

/// Schema 47 holds two more one-byte flags than FM26's schema 46, so the filename count sits
/// at +167 instead of +165, the clock at +174 instead of +172, and the late build window at
/// +382..+446 instead of +176..+240. The database version and saved-by build word are where
/// FM26 keeps them.
pub(super) const FM27_GAME_INFO: GameInfoLayout = GameInfoLayout {
    db_version_length_offset: 8,
    max_db_version_bytes: 64,
    build_number_offsets_after_db_version: &[38],
    game_date_offset_after_db_version: 174,
    late_build_number_window_after_db_version: (382, 446),
    filename_count_offset_after_db_version: 167,
    max_filenames: 256,
    max_filename_bytes: 4096,
};

/// The FM26 summary values, unchanged on FM27 schema 31. fmsave detects the version with the
/// same values before it knows the edition.
pub(super) const SAVE_SUMMARY: SaveSummaryLayout = SaveSummaryLayout {
    max_version_bytes: 32,
    structured_strings_offset: 8,
    max_setup_string_bytes: 65_536,
    max_summary_name_bytes: 1_024,
    max_divisions: 4_096,
    manager_header_bytes: 8,
    club_uid_bytes: 4,
};

/// One layout registered for a section schema on a build.
pub(super) struct LayoutEntry<T: 'static> {
    pub schema: Option<u16>,
    pub build: &'static str,
    pub layout: &'static T,
}

pub(super) const GAME_INFO_LAYOUTS: &[LayoutEntry<GameInfoLayout>] = &[LayoutEntry {
    schema: Some(47),
    build: REGISTERED_FM27_BUILD,
    layout: &FM27_GAME_INFO,
}];

pub(super) const SAVE_SUMMARY_LAYOUTS: &[LayoutEntry<SaveSummaryLayout>] = &[LayoutEntry {
    schema: Some(31),
    build: REGISTERED_FM27_BUILD,
    layout: &SAVE_SUMMARY,
}];

pub(super) fn is_registered_fm27_build(build: &str) -> bool {
    REGISTERED_FM27_BUILDS.contains(&build)
}

/// The layout an FM27 save of `build` reads a section of `schema` with.
///
/// A registered build uses only its own layout at the schema it registered. Another FM27
/// build borrows a registered build's layout only when exactly one registered layout has
/// this schema. A region registered with no schema has none to compare and is borrowed.
/// FM26 layouts are never candidates, and every other case fails closed.
pub(super) fn find_fm27_layout<T: PartialEq>(
    entries: &'static [LayoutEntry<T>],
    schema: Option<u16>,
    build: &str,
) -> Result<&'static T, SaveStatsError> {
    if is_registered_fm27_build(build) {
        return entries
            .iter()
            .find(|entry| entry.build == build && entry.schema == schema)
            .map(|entry| entry.layout)
            .ok_or(SaveStatsError::UnsupportedBuild);
    }
    let mut matching: Vec<&'static T> = Vec::new();
    for entry in entries {
        if is_registered_fm27_build(entry.build)
            && entry.schema == schema
            && !matching.contains(&entry.layout)
        {
            matching.push(entry.layout);
        }
    }
    match matching.as_slice() {
        [layout] => Ok(layout),
        _ => Err(SaveStatsError::UnsupportedBuild),
    }
}
