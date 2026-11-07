//! Reads FM27 save files: identification, build support, the in-game date, and each player's
//! current-season statistics.
//!
//! The metadata decoding here is ported from fmsave 0.10.0 (commit
//! a71abe9b7b38e35affa05c49c3619ff05eb21795, MIT License, Copyright (c) 2026 Randal S.
//! Olson): `_version.py` (`read_save_info`, `find_game_version`, `read_summary_clock`, and
//! the `game_info` checks), `decode_date` from `_scan.py`, the section schemas of
//! `_version.py`, and the season-statistics fields of `models/season_stats.py`. See
//! `THIRD_PARTY_NOTICES.md`.
//!
//! The module is pure: a path goes in, and save facts or a path-free error come out.

mod container;
mod layouts;
mod players;
#[cfg(test)]
mod reference;
mod season_stats;
#[cfg(test)]
mod test_support;

use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use container::{
    find_marker, read_length_prefixed_string, read_metadata_section, read_u16, read_u32,
    ContainerIndex, FileStamp, SaveSource,
};
use layouts::{
    find_fm27_layout, is_registered_fm27_build, GameInfoLayout, SaveSummaryLayout,
    GAME_INFO_LAYOUTS, GATE_BOUNDS_LAYOUTS, NAME_POOL_LAYOUTS, PLAYER_RECORD_LAYOUTS, SAVE_SUMMARY,
    SAVE_SUMMARY_LAYOUTS, SEASON_STATS_LAYOUTS,
};

const GAME_INFO_SECTION: &str = "game_info";
const SAVE_SUMMARY_SECTION: &str = "save_game_summary";
const GAME_DB_SECTION: &str = "game_db";
const SUPPORTED_GAME_MAJOR: u32 = 27;
const SECTION_MAGIC_PREFIX: &[u8] = b"\x03\x01";
const SECTION_HEAD_BYTES: usize = 8;
const SCHEMA_OFFSET: usize = 6;
const LENGTH_PREFIX_BYTES: usize = 4;
const MAX_MIGRATION_NAMES: u32 = 4096;
const MAX_MIGRATION_NAME_BYTES: usize = 4096;
/// A `game_info` clock of day 1, 1900: the game has not set it yet.
const UNSET_GAME_INFO_CLOCK: [u8; 4] = [1, 0, 0x6C, 0x07];
const DAY_OF_YEAR_MASK: u16 = 0x1FF;
const EARLIEST_GAME_YEAR: u16 = 1901;
const LATEST_GAME_YEAR: u16 = 2200;

/// What a supported FM27 save says about itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SaveInfo {
    pub build: String,
    /// Whether fmsave registers layouts for this exact build. An unregistered FM27 build is
    /// accepted only when its metadata fits the registered layouts.
    pub known_build: bool,
    /// The save's displayed in-game date as `YYYY-MM-DD`.
    pub game_date: String,
}

/// Why a save file could not be read. Messages never include the file's path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SaveStatsError {
    NotFound,
    Unreadable,
    NotFmSave,
    UnsupportedGame,
    UnsupportedBuild,
    Corrupt,
    OutOfBounds,
    ChangedDuringRead,
}

impl SaveStatsError {
    /// A stable machine-readable kind.
    pub(crate) fn kind(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::Unreadable => "unreadable",
            Self::NotFmSave => "not_fm_save",
            Self::UnsupportedGame => "unsupported_game",
            Self::UnsupportedBuild => "unsupported_build",
            Self::Corrupt => "corrupt",
            Self::OutOfBounds => "out_of_bounds",
            Self::ChangedDuringRead => "changed_during_read",
        }
    }
}

impl fmt::Display for SaveStatsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::NotFound => "The save file could not be found",
            Self::Unreadable => "The save file could not be opened",
            Self::NotFmSave => "The file is not a Football Manager save",
            Self::UnsupportedGame => "The file is not a supported FM27 save",
            Self::UnsupportedBuild => "This FM27 build is not supported yet",
            Self::Corrupt => "The save file is damaged and could not be read safely",
            Self::OutOfBounds => "The save file exceeds the reader's safety limits",
            Self::ChangedDuringRead => {
                "The save file changed while it was being read. Wait for FM to finish saving, then try again"
            }
        };
        f.write_str(message)
    }
}

impl std::error::Error for SaveStatsError {}

/// Identifies an FM27 save from bounded reads of its header, trailer directory, and two
/// metadata sections, without reading the rest of the file.
pub(crate) fn read_save_info(path: &Path) -> Result<SaveInfo, SaveStatsError> {
    container::read_unchanged(path, FileStamp::of, |file, file_size| {
        let index = container::read_index(file, file_size)?;
        save_info_from(file, &index)
    })
}

/// Defines `SeasonStat` with each statistic's fmsave `PlayerSeasonStats` field name.
macro_rules! season_stats {
    ($($stat:ident => $name:literal,)+) => {
        /// A season statistic fmsave's `PlayerSeasonStats` exposes as a raw total. Starts,
        /// substitute appearances, minutes, rated appearances, and ratings are fields of
        /// `SavePlayerSeasonStats` instead.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub(crate) enum SeasonStat {
            $($stat,)+
        }

        impl SeasonStat {
            pub(crate) const ALL: [SeasonStat; [$($name),+].len()] = [$(SeasonStat::$stat),+];
            pub(crate) const COUNT: usize = Self::ALL.len();

            /// The fmsave field name, which only the reference report and tests print.
            #[cfg(test)]
            pub(crate) fn name(self) -> &'static str {
                match self {
                    $(Self::$stat => $name,)+
                }
            }
        }
    };
}

season_stats! {
    PlayerOfTheMatch => "player_of_the_match",
    Goals => "goals",
    Assists => "assists",
    ExpectedGoals => "expected_goals",
    NonPenaltyExpectedGoals => "non_penalty_expected_goals",
    ExpectedAssists => "expected_assists",
    Shots => "shots",
    ShotsOnTarget => "shots_on_target",
    ShotsOutsideBox => "shots_outside_box",
    GoalsOutsideBox => "goals_outside_box",
    FreeKickShots => "free_kick_shots",
    PenaltiesTaken => "penalties_taken",
    PenaltiesScored => "penalties_scored",
    PassesAttempted => "passes_attempted",
    PassesCompleted => "passes_completed",
    ProgressivePasses => "progressive_passes",
    KeyPasses => "key_passes",
    OpenPlayKeyPasses => "open_play_key_passes",
    ClearCutChancesCreated => "clear_cut_chances_created",
    CrossesAttempted => "crosses_attempted",
    CrossesCompleted => "crosses_completed",
    OpenPlayCrossesAttempted => "open_play_crosses_attempted",
    OpenPlayCrossesCompleted => "open_play_crosses_completed",
    Dribbles => "dribbles",
    Offsides => "offsides",
    DistanceKm => "distance_km",
    HighIntensitySprints => "high_intensity_sprints",
    AerialChallengesAttempted => "aerial_challenges_attempted",
    HeadersWon => "headers_won",
    KeyHeaders => "key_headers",
    TacklesAttempted => "tackles_attempted",
    TacklesCompleted => "tackles_completed",
    KeyTackles => "key_tackles",
    Interceptions => "interceptions",
    PossessionWon => "possession_won",
    PossessionLost => "possession_lost",
    PressuresAttempted => "pressures_attempted",
    PressuresCompleted => "pressures_completed",
    Blocks => "blocks",
    ShotsBlocked => "shots_blocked",
    Clearances => "clearances",
    FoulsMade => "fouls_made",
    FoulsAgainst => "fouls_against",
    YellowCards => "yellow_cards",
    RedCards => "red_cards",
    MistakesLeadingToGoal => "mistakes_leading_to_goal",
    CleanSheets => "clean_sheets",
    GoalsAllowed => "goals_allowed",
    SavesHeld => "saves_held",
    SavesParried => "saves_parried",
    SavesTipped => "saves_tipped",
    ShotsOnTargetFaced => "shots_on_target_faced",
    ExpectedGoalsPrevented => "expected_goals_prevented",
}

/// One player's current-season totals over league, cup, and continental lines for every team
/// they played for.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SavePlayerSeasonStats {
    /// The game's Unique ID for the player.
    pub unique_id: u32,
    pub starts: u32,
    pub substitute_appearances: u32,
    pub minutes: u32,
    pub rated_appearances: u32,
    /// The sum of the player's match ratings, so that an average weights each rated
    /// appearance equally.
    pub rating_sum: f64,
    totals: [Option<f64>; SeasonStat::COUNT],
}

impl SavePlayerSeasonStats {
    /// The summed total in public units (kilometres, expected goals), or `None` when the save
    /// stores no value for this player: a goalkeeper's outfield words, an outfield player's
    /// goalkeeping words, or a statistic the save does not support.
    pub(crate) fn total(&self, stat: SeasonStat) -> Option<f64> {
        self.totals[stat as usize]
    }

    /// A player with no playing time and each total from `total`, for tests outside this
    /// module that need statistics no synthetic season line stores.
    #[cfg(test)]
    pub(crate) fn for_test(unique_id: u32, total: impl Fn(SeasonStat) -> Option<f64>) -> Self {
        Self {
            unique_id,
            starts: 0,
            substitute_appearances: 0,
            minutes: 0,
            rated_appearances: 0,
            rating_sum: 0.0,
            totals: SeasonStat::ALL.map(total),
        }
    }
}

/// Synthetic FM27 saves for tests outside this module.
#[cfg(test)]
pub(crate) mod synthetic {
    use std::path::{Path, PathBuf};

    use super::test_support::{
        season_stats_body, season_stats_save, stat_line, write_save, ExamplePlayer, ExampleRecord,
        FM27_BUILD,
    };

    /// The in-game date every synthetic save reports.
    pub(crate) const GAME_DATE: &str = "2031-03-02";

    const LEAGUE_SLOT: usize = 1;

    /// Writes a registered-build save in which each `(unique_id, fields)` outfield player
    /// played one league line. Field names and units follow the season-line fixture, for
    /// example `("minutes", 90.0)` or `("expected_goals", 0.45)`.
    ///
    /// Unique IDs must be distinct and at least 2: a closing Unique ID lies above its
    /// record's uid, which must be above zero.
    pub(crate) fn write_season_stats_save(
        directory: &Path,
        players: &[(u32, &[(&str, f64)])],
    ) -> PathBuf {
        let mut players = players.to_vec();
        players.sort_by_key(|(unique_id, _)| *unique_id);
        let mut example_players = Vec::new();
        let mut records = Vec::new();
        for (index, (unique_id, fields)) in players.iter().enumerate() {
            let pindex = 10 * (index as u32 + 1);
            example_players.push(ExamplePlayer::outfield(
                pindex,
                unique_id - 1,
                Some(*unique_id),
            ));
            records.push(ExampleRecord::with_lines(
                pindex + 1,
                &[(LEAGUE_SLOT, stat_line(fields))],
            ));
        }
        let save = season_stats_save(
            FM27_BUILD,
            4000,
            &example_players,
            &season_stats_body(&records, 15),
        );
        write_save(directory, &save)
    }
}

/// The save's facts and the statistics of every player with a Unique ID and competitive
/// minutes this season.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SaveSeasonStats {
    pub info: SaveInfo,
    pub players: Vec<SavePlayerSeasonStats>,
}

/// Reads the whole save once, then the players' current-season statistics.
///
/// Every structural check must pass first: a failure returns an error and no statistics.
/// Section schemas must match the registered layouts' on every build.
pub(crate) fn read_season_stats(path: &Path) -> Result<SaveSeasonStats, SaveStatsError> {
    let save = container::read_save_bytes(path)?;
    let mut source: &[u8] = &save;
    let index = container::read_index(&mut source, save.len() as u64)?;
    let info = save_info_from(&mut source, &index)?;
    let game_db = container::read_section(&mut source, &index, GAME_DB_SECTION)?;
    let schema = section_schema(&game_db, &index.section(GAME_DB_SECTION)?.extension)?;
    // Every layout is chosen, and the season-statistics frame found by its schema, before any
    // record is decoded.
    let player_layout = find_fm27_layout(PLAYER_RECORD_LAYOUTS, Some(schema), &info.build)?;
    let name_pool_layout = find_fm27_layout(NAME_POOL_LAYOUTS, Some(schema), &info.build)?;
    let bounds = find_fm27_layout(GATE_BOUNDS_LAYOUTS, Some(schema), &info.build)?;
    let season_layout = find_fm27_layout(SEASON_STATS_LAYOUTS, None, &info.build)?;
    let season_frames = season_stats::read_section_frames(&mut source, &index, season_layout)?;
    drop(save);

    let fit = |error| layout_fit(error, info.known_build);
    let players =
        players::read_players(&game_db, name_pool_layout, player_layout, bounds).map_err(fit)?;
    let players = season_stats::season_totals(
        &season_frames,
        &players,
        season_layout,
        bounds,
        game_db.len(),
    )
    .map_err(fit)?;
    // Cohorts are keyed by Unique ID, so a repeated one cannot be attributed safely.
    let mut unique_ids = HashSet::with_capacity(players.len());
    if !players
        .iter()
        .all(|player| unique_ids.insert(player.unique_id))
    {
        return Err(fit(SaveStatsError::Corrupt));
    }
    Ok(SaveSeasonStats { info, players })
}

/// Damage under a registered build's own layout is damage; under a borrowed layout it means
/// the layout does not fit.
fn layout_fit(error: SaveStatsError, known_build: bool) -> SaveStatsError {
    match error {
        SaveStatsError::Corrupt if !known_build => SaveStatsError::UnsupportedBuild,
        error => error,
    }
}

struct GameVersion {
    major: u32,
    build: String,
    build_number: u32,
}

fn save_info_from(
    source: &mut impl SaveSource,
    index: &ContainerIndex,
) -> Result<SaveInfo, SaveStatsError> {
    let (summary, summary_schema) = read_signed_section(source, index, SAVE_SUMMARY_SECTION)?;
    let version = find_game_version(&summary, &SAVE_SUMMARY).ok_or(SaveStatsError::NotFmSave)?;
    if version.major != SUPPORTED_GAME_MAJOR {
        return Err(SaveStatsError::UnsupportedGame);
    }
    let summary_layout =
        find_fm27_layout(SAVE_SUMMARY_LAYOUTS, Some(summary_schema), &version.build)?;
    // fmsave requires the structured summary clock on FM27; it is the save's displayed date,
    // while `game_info` can keep the previous simulation day's clock.
    let game_date = read_summary_clock(&summary, summary_layout, &version)
        .ok_or(SaveStatsError::UnsupportedBuild)?;

    let known_build = is_registered_fm27_build(&version.build);
    let (game_info, game_info_schema) = read_signed_section(source, index, GAME_INFO_SECTION)?;
    let game_info_layout =
        find_fm27_layout(GAME_INFO_LAYOUTS, Some(game_info_schema), &version.build)?;
    let fit = if known_build {
        decode_game_info(&game_info, game_info_layout, &version).map(|_| ())
    } else {
        check_unregistered_game_info(&game_info, game_info_layout, &version)
    };
    match fit {
        // Damage under a registered build's own layout is damage; under a borrowed layout it
        // means the layout does not fit.
        Err(SaveStatsError::Corrupt) if !known_build => Err(SaveStatsError::UnsupportedBuild),
        Err(error) => Err(error),
        Ok(()) => Ok(SaveInfo {
            build: version.build,
            known_build,
            game_date: game_date.to_string(),
        }),
    }
}

/// The schema of a section that opens with its signature: two bytes, then its directory
/// extension reversed.
fn section_schema(head: &[u8], extension: &str) -> Result<u16, SaveStatsError> {
    let reversed_extension = extension.bytes().rev();
    if head.len() < SECTION_HEAD_BYTES
        || !head.starts_with(SECTION_MAGIC_PREFIX)
        || !head[SECTION_MAGIC_PREFIX.len()..SCHEMA_OFFSET]
            .iter()
            .copied()
            .eq(reversed_extension)
    {
        return Err(SaveStatsError::Corrupt);
    }
    read_u16(head, SCHEMA_OFFSET)
}

fn read_signed_section(
    source: &mut impl SaveSource,
    index: &ContainerIndex,
    name: &str,
) -> Result<(Vec<u8>, u16), SaveStatsError> {
    let section = read_metadata_section(source, index, name)?;
    let schema = section_schema(&section, &index.section(name)?.extension)?;
    Ok((section, schema))
}

/// The first length-prefixed string that is exactly a game version, `M.m.p+build`.
fn find_game_version(summary: &[u8], layout: &SaveSummaryLayout) -> Option<GameVersion> {
    (LENGTH_PREFIX_BYTES..summary.len())
        .filter(|&start| is_version_candidate_start(summary, start))
        .find_map(|start| {
            let length = read_u32(summary, start - LENGTH_PREFIX_BYTES).ok()? as usize;
            if length == 0 || length > layout.max_version_bytes {
                return None;
            }
            parse_game_version(summary.get(start..start.checked_add(length)?)?)
        })
}

/// One to three digits and a dot, not preceded by a digit or dot.
fn is_version_candidate_start(summary: &[u8], start: usize) -> bool {
    if start > 0 && (summary[start - 1].is_ascii_digit() || summary[start - 1] == b'.') {
        return false;
    }
    let digits = summary[start..]
        .iter()
        .take(4)
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    (1..=3).contains(&digits) && summary.get(start + digits) == Some(&b'.')
}

/// Parses `([0-9]{1,3})\.([0-9]{1,3})\.([0-9]{1,4})\+([0-9]{1,10})` as the whole of `text`.
/// The game writes no leading zero on the major and stores the build number as a u32.
fn parse_game_version(text: &[u8]) -> Option<GameVersion> {
    let text = std::str::from_utf8(text).ok()?;
    let (release, build_number) = text.split_once('+')?;
    let mut parts = release.split('.');
    let (major, minor, patch) = (parts.next()?, parts.next()?, parts.next()?);
    let fits = |part: &str, max_digits: usize| {
        (1..=max_digits).contains(&part.len()) && part.bytes().all(|byte| byte.is_ascii_digit())
    };
    if parts.next().is_some()
        || !fits(major, 3)
        || !fits(minor, 3)
        || !fits(patch, 4)
        || !fits(build_number, 10)
        || major.starts_with('0')
    {
        return None;
    }
    Some(GameVersion {
        major: major.parse().ok()?,
        build: text.to_owned(),
        build_number: build_number.parse().ok()?,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GameDate {
    year: u16,
    month: u8,
    day: u8,
}

impl fmt::Display for GameDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// A 4-byte game date: the day of the year in the low nine bits of a u16, then the year.
/// The null date and malformed values give `None`.
fn decode_date(buffer: &[u8], offset: usize) -> Result<Option<GameDate>, SaveStatsError> {
    let day_of_year = read_u16(buffer, offset)? & DAY_OF_YEAR_MASK;
    let year = read_u16(buffer, offset + 2)?;
    if !(EARLIEST_GAME_YEAR..=LATEST_GAME_YEAR).contains(&year) {
        return Ok(None);
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days_in_year = if leap { 366 } else { 365 };
    if !(1..=days_in_year).contains(&day_of_year) {
        return Ok(None);
    }
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut day = day_of_year;
    let mut month = 1;
    for days in month_days {
        if day <= days {
            break;
        }
        day -= days;
        month += 1;
    }
    Ok(Some(GameDate {
        year,
        month,
        day: day as u8,
    }))
}

/// The structured summary clock, never a date-shaped byte scan. Any read that does not fit
/// gives `None`.
fn read_summary_clock(
    summary: &[u8],
    layout: &SaveSummaryLayout,
    version: &GameVersion,
) -> Option<GameDate> {
    let (_, offset) = read_length_prefixed_string(
        summary,
        layout.structured_strings_offset,
        layout.max_setup_string_bytes,
    )
    .ok()?;
    let (stored_version, offset) =
        read_length_prefixed_string(summary, offset, layout.max_version_bytes).ok()?;
    if stored_version != version.build {
        return None;
    }
    let division_count = read_u32(summary, offset).ok()?;
    if division_count > layout.max_divisions {
        return None;
    }
    let mut offset = offset + LENGTH_PREFIX_BYTES;
    for _ in 0..division_count {
        (_, offset) =
            read_length_prefixed_string(summary, offset, layout.max_summary_name_bytes).ok()?;
    }
    let (_, offset) = read_length_prefixed_string(
        summary,
        offset + layout.manager_header_bytes,
        layout.max_summary_name_bytes,
    )
    .ok()?;
    let (_, offset) =
        read_length_prefixed_string(summary, offset, layout.max_summary_name_bytes).ok()?;
    decode_date(summary, offset + layout.club_uid_bytes).ok()?
}

/// Where the `game_info` fields sit once the variable-length fields are walked.
struct GameInfoFields {
    /// The offset of the clock and the late-build window base, past the database version and
    /// the filename list.
    base: usize,
    game_date: Option<GameDate>,
}

/// Checks `game_info` against `layout` for `version`.
///
/// Reads outside the section are `Corrupt`; a build word or filename count that is not what
/// the layout expects is `UnsupportedBuild`.
fn decode_game_info(
    game_info: &[u8],
    layout: &GameInfoLayout,
    version: &GameVersion,
) -> Result<GameInfoFields, SaveStatsError> {
    let (_, version_end) = read_length_prefixed_string(
        game_info,
        layout.db_version_length_offset,
        layout.max_db_version_bytes,
    )?;
    let base = version_end + game_info_filename_bytes(game_info, layout, version_end)?;
    let mut build_words_match = true;
    for offset in layout.build_number_offsets_after_db_version {
        build_words_match &= read_u32(game_info, version_end + offset)? == version.build_number;
    }
    let (window_start, window_end) = layout.late_build_number_window_after_db_version;
    let late_build_number_at = find_marker(
        game_info,
        &version.build_number.to_le_bytes(),
        base + window_start,
        (base + window_end).min(game_info.len()),
    )?;
    if !build_words_match || late_build_number_at.is_none() {
        return Err(SaveStatsError::UnsupportedBuild);
    }
    let game_date = decode_date(game_info, base + layout.game_date_offset_after_db_version)?;
    Ok(GameInfoFields { base, game_date })
}

/// The bytes of the bounded filename list before the clock. A count no save stores means the
/// layout differs, not that the save is damaged.
fn game_info_filename_bytes(
    game_info: &[u8],
    layout: &GameInfoLayout,
    version_end: usize,
) -> Result<usize, SaveStatsError> {
    let count_at = version_end + layout.filename_count_offset_after_db_version;
    let count = read_u32(game_info, count_at)?;
    if count > layout.max_filenames {
        return Err(SaveStatsError::UnsupportedBuild);
    }
    let strings_start = count_at + LENGTH_PREFIX_BYTES;
    let mut cursor = strings_start;
    for _ in 0..count {
        (_, cursor) = read_length_prefixed_string(game_info, cursor, layout.max_filename_bytes)?;
    }
    Ok(cursor - strings_start)
}

/// The stricter fit an unregistered build's `game_info` must pass to borrow a layout: a set or
/// explicitly unset clock, and exactly one late build word followed by a bounded migration
/// list that ends the section.
fn check_unregistered_game_info(
    game_info: &[u8],
    layout: &GameInfoLayout,
    version: &GameVersion,
) -> Result<(), SaveStatsError> {
    let fields = decode_game_info(game_info, layout, version)?;
    let clock_at = fields.base + layout.game_date_offset_after_db_version;
    let clock_is_unset = game_info.get(clock_at..clock_at + 4) == Some(&UNSET_GAME_INFO_CLOCK[..]);
    let (start, end) = layout.late_build_number_window_after_db_version;
    if (fields.game_date.is_none() && !clock_is_unset)
        || !game_info_migration_tail_fits(
            game_info,
            version.build_number,
            fields.base + start,
            (fields.base + end).min(game_info.len()),
        )?
    {
        return Err(SaveStatsError::UnsupportedBuild);
    }
    Ok(())
}

/// Whether exactly one build marker in `[start, end)` is followed by a bounded string list
/// that ends at the section end.
fn game_info_migration_tail_fits(
    game_info: &[u8],
    build_number: u32,
    mut start: usize,
    end: usize,
) -> Result<bool, SaveStatsError> {
    let marker = build_number.to_le_bytes();
    let mut matches = 0;
    while let Some(build_at) = find_marker(game_info, &marker, start, end)? {
        start = build_at + 1;
        let Ok(count) = read_u32(game_info, build_at + LENGTH_PREFIX_BYTES) else {
            continue;
        };
        if count > MAX_MIGRATION_NAMES {
            continue;
        }
        let mut cursor = Some(build_at + 2 * LENGTH_PREFIX_BYTES);
        for _ in 0..count {
            cursor = cursor.and_then(|at| {
                read_length_prefixed_string(game_info, at, MAX_MIGRATION_NAME_BYTES)
                    .ok()
                    .map(|(_, next)| next)
            });
        }
        if cursor == Some(game_info.len()) {
            matches += 1;
        }
    }
    Ok(matches == 1)
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Read;

    use super::test_support::{
        build_container, compress, container_with_trailer, fm27_container, fm27_game_info_body,
        fm27_sections, frame_declaring_content_size, packed_date, season_stats_body,
        season_stats_save, stat_line, structured_summary_body, with_body, write_save,
        ExamplePlayer, ExampleRecord, FM27_BUILD, UNREGISTERED_FM27_BUILD,
    };
    use super::*;

    const MIB: u64 = 1024 * 1024;

    fn read_content(content: &[u8]) -> Result<SaveInfo, SaveStatsError> {
        let directory = tempfile::tempdir().expect("temp dir");
        read_save_info(&write_save(directory.path(), content))
    }

    fn with_game_info(build: &str, game_info: Vec<u8>) -> Vec<u8> {
        build_container(
            &with_body(fm27_sections(build), "game_info", game_info),
            &[],
        )
    }

    fn with_summary_version(version: &str, schema: u16) -> Vec<u8> {
        let summary = structured_summary_body(version, schema, packed_date(61, 2031, 0));
        build_container(
            &with_body(fm27_sections(FM27_BUILD), "save_game_summary", summary),
            &[],
        )
    }

    #[test]
    fn registered_fm27_save_reports_its_build_and_summary_date() {
        let info = read_content(&fm27_container(FM27_BUILD)).expect("save info");

        assert_eq!(
            info,
            SaveInfo {
                build: FM27_BUILD.to_owned(),
                known_build: true,
                game_date: "2031-03-02".to_owned(),
            }
        );
    }

    #[test]
    fn unregistered_fm27_build_whose_game_info_fits_is_read_as_unknown() {
        let game_info =
            fm27_game_info_body(UNREGISTERED_FM27_BUILD, 47, 167, &["Example Tool.dll"]);

        let info =
            read_content(&with_game_info(UNREGISTERED_FM27_BUILD, game_info)).expect("save info");

        assert_eq!(info.build, UNREGISTERED_FM27_BUILD);
        assert!(!info.known_build);
        assert_eq!(info.game_date, "2031-03-02");
    }

    #[test]
    fn game_info_that_fits_no_registered_layout_is_an_unsupported_build() {
        let unregistered = UNREGISTERED_FM27_BUILD;
        let cases = [
            // A filename list at the FM26 offset moves the clock and late build word.
            (
                FM27_BUILD,
                fm27_game_info_body(FM27_BUILD, 47, 165, &["Tool.dll"]),
            ),
            (
                unregistered,
                fm27_game_info_body(unregistered, 47, 165, &["Tool.dll"]),
            ),
            // No FM27 layout is registered for schema 46.
            (FM27_BUILD, fm27_game_info_body(FM27_BUILD, 46, 167, &[])),
            (
                unregistered,
                fm27_game_info_body(unregistered, 46, 167, &[]),
            ),
            // The build words name another build.
            (FM27_BUILD, fm27_game_info_body(unregistered, 47, 167, &[])),
        ];

        for (build, game_info) in cases {
            assert_eq!(
                read_content(&with_game_info(build, game_info)),
                Err(SaveStatsError::UnsupportedBuild),
                "{build}"
            );
        }
    }

    #[test]
    fn unregistered_build_with_damaged_game_info_is_an_unsupported_build() {
        // The migration list no longer ends the section.
        let mut truncated = fm27_game_info_body(UNREGISTERED_FM27_BUILD, 47, 167, &[]);
        truncated.truncate(truncated.len() - 1);
        // A read past its bound, which is corrupt under a registered build's own layout.
        let mut overlong_database_version =
            fm27_game_info_body(UNREGISTERED_FM27_BUILD, 47, 167, &[]);
        overlong_database_version[8..12].copy_from_slice(&65_u32.to_le_bytes());

        for game_info in [truncated, overlong_database_version] {
            assert_eq!(
                read_content(&with_game_info(UNREGISTERED_FM27_BUILD, game_info)),
                Err(SaveStatsError::UnsupportedBuild)
            );
        }
    }

    #[test]
    fn other_editions_are_unsupported_games() {
        for (version, schema) in [("26.3.2+2329565", 29), ("28.0.0+3100000", 31)] {
            assert_eq!(
                read_content(&with_summary_version(version, schema)),
                Err(SaveStatsError::UnsupportedGame),
                "{version}"
            );
        }
    }

    #[test]
    fn files_without_the_save_magic_or_a_game_version_are_not_fm_saves() {
        let mut wrong_magic = fm27_container(FM27_BUILD);
        wrong_magic[0] = b'P';

        assert_eq!(read_content(&wrong_magic), Err(SaveStatsError::NotFmSave));
        assert_eq!(read_content(b""), Err(SaveStatsError::NotFmSave));
        assert_eq!(
            read_content(&with_summary_version("not a version", 31)),
            Err(SaveStatsError::NotFmSave)
        );
    }

    #[test]
    fn damaged_structure_is_corrupt() {
        let save = fm27_container(FM27_BUILD);
        let truncated_file = save[..save.len() - 3].to_vec();
        // The save name claims 14 bytes and the payload ends after one.
        let truncated_directory = container_with_trailer(&[], &compress(&[14, 0, 0, 0, b'E']));
        let truncated_game_info_frame = {
            let sections = fm27_sections(FM27_BUILD);
            let real = compress(&sections[0].body).len() as u64;
            build_container(&sections, &[("game_info.dat", (Some(real - 1), None))])
        };
        let entry_past_the_trailer = build_container(
            &fm27_sections(FM27_BUILD),
            &[("humans.dat", (Some(1 << 40), None))],
        );
        let overlong_database_version = {
            let mut game_info = fm27_game_info_body(FM27_BUILD, 47, 167, &[]);
            game_info[8..12].copy_from_slice(&65_u32.to_le_bytes());
            with_game_info(FM27_BUILD, game_info)
        };

        for (case, content) in [
            ("truncated file", truncated_file),
            ("truncated directory", truncated_directory),
            ("truncated game_info frame", truncated_game_info_frame),
            ("entry past the trailer", entry_past_the_trailer),
            ("overlong database version", overlong_database_version),
        ] {
            assert_eq!(
                read_content(&content),
                Err(SaveStatsError::Corrupt),
                "{case}"
            );
        }
    }

    #[test]
    fn declared_sizes_beyond_the_bounds_are_rejected_before_decompression() {
        let sections = fm27_sections(FM27_BUILD);
        // A part may always reach 64 MiB; a tiny one claiming more exceeds its ratio.
        let part_over_its_ratio =
            build_container(&sections, &[("game_db.dat", (None, Some(64 * MIB + 1)))]);
        // Each part fits its own bound, but together they exceed 32 times the file size.
        let save_over_its_ratio = build_container(
            &sections,
            &[
                ("game_db.dat", (None, Some(64 * MIB))),
                ("humans.dat", (None, Some(64 * MIB))),
            ],
        );
        let metadata_over_its_cap =
            build_container(&sections, &[("game_info.dat", (None, Some(MIB + 1)))]);
        let trailer_declaring_too_much =
            container_with_trailer(&[], &frame_declaring_content_size(64 * MIB + 1));

        for (case, content) in [
            ("part over its ratio", part_over_its_ratio),
            ("save over its ratio", save_over_its_ratio),
            ("metadata over its cap", metadata_over_its_cap),
            ("trailer declaring too much", trailer_declaring_too_much),
        ] {
            assert_eq!(
                read_content(&content),
                Err(SaveStatsError::OutOfBounds),
                "{case}"
            );
        }
    }

    #[test]
    fn oversized_trailer_is_rejected_before_it_is_read() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = write_save(directory.path(), &container_with_trailer(&[], &[]));
        // A sparse tail makes the trailer's compressed bytes exceed 64 MiB.
        let file = File::options()
            .write(true)
            .open(&path)
            .expect("open fixture");
        file.set_len(26 + 9 + 64 * MIB + 1).expect("extend fixture");

        assert_eq!(read_save_info(&path), Err(SaveStatsError::OutOfBounds));
    }

    #[test]
    fn trailer_without_a_declared_size_stops_at_its_cap() {
        let mut trailer = Vec::new();
        zstd::stream::copy_encode(std::io::repeat(0).take(64 * MIB + 1), &mut trailer, 3)
            .expect("compress trailer");
        assert!(
            matches!(zstd::zstd_safe::get_frame_content_size(&trailer), Ok(None)),
            "the stream encoder declares no content size"
        );

        assert_eq!(
            read_content(&container_with_trailer(&[], &trailer)),
            Err(SaveStatsError::OutOfBounds)
        );
    }

    #[test]
    fn missing_paths_and_non_regular_files_are_rejected_without_reading() {
        let directory = tempfile::tempdir().expect("temp dir");

        assert_eq!(
            read_save_info(directory.path()),
            Err(SaveStatsError::Unreadable)
        );
        assert_eq!(
            read_save_info(&directory.path().join("missing.fm")),
            Err(SaveStatsError::NotFound)
        );
    }

    #[test]
    fn error_messages_never_contain_the_supplied_path() {
        let directory = tempfile::tempdir().expect("temp dir");
        let damaged = write_save(directory.path(), &fm27_container(FM27_BUILD)[..100]);
        let missing = directory.path().join("missing.fm");

        for path in [damaged.as_path(), directory.path(), missing.as_path()] {
            let message = read_save_info(path).expect_err("read fails").to_string();
            assert!(!message.contains(&*path.to_string_lossy()), "{message}");
            let file_name = path.file_name().expect("file name").to_string_lossy();
            assert!(!message.contains(&*file_name), "{message}");
        }
    }

    #[test]
    fn two_players_sharing_a_unique_id_return_no_statistics() {
        // A closing Unique ID must lie above its record's uid and not above the next
        // record's, so a repeat needs a lower uid after a record that has none.
        let players = [
            ExamplePlayer::outfield(10, 1_000, Some(1_500)),
            ExamplePlayer::outfield(20, 2_000, None),
            ExamplePlayer::outfield(30, 1_200, Some(1_500)),
        ];
        let played = || stat_line(&[("minutes", 90.0), ("starts", 1.0)]);
        let records = [
            ExampleRecord::with_lines(11, &[(1, played())]),
            ExampleRecord::with_lines(31, &[(1, played())]),
        ];
        let directory = tempfile::tempdir().expect("temp dir");

        for (build, error) in [
            (FM27_BUILD, SaveStatsError::Corrupt),
            (UNREGISTERED_FM27_BUILD, SaveStatsError::UnsupportedBuild),
        ] {
            let save = season_stats_save(build, 4000, &players, &season_stats_body(&records, 15));
            assert_eq!(
                read_season_stats(&write_save(directory.path(), &save)),
                Err(error),
                "{build}"
            );
        }
    }

    #[test]
    fn reading_leaves_the_file_byte_identical() {
        let directory = tempfile::tempdir().expect("temp dir");
        let content = fm27_container(FM27_BUILD);
        let path = write_save(directory.path(), &content);

        read_save_info(&path).expect("save info");

        assert_eq!(fs::read(&path).expect("read fixture"), content);
    }
}
