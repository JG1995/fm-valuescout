//! FM27 layout values and the rules for choosing them.
//!
//! Ported from fmsave 0.10.0 (commit a71abe9b7b38e35affa05c49c3619ff05eb21795, MIT
//! License, Copyright (c) 2026 Randal S. Olson): `_layouts/__init__.py` (`find_layout` for
//! FM27 builds) and `_layouts/fm27_27_0_0.py`, with the FM26 values from
//! `_layouts/fm26_26_3_2.py` that FM27 keeps. See `THIRD_PARTY_NOTICES.md`.

use super::{SaveStatsError, SeasonStat};

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

/// How to find the three name pools in `game_db`. Only their end is used: the completeness
/// scan for player records starts there.
///
/// `signature` sits immediately before the first pool's entry count. Its three capacity words
/// (the third to fifth) may scale together up to `max_signature_scale`, and the bytes at
/// `unchecked_signature_bytes` may hold any value.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct NamePoolLayout {
    pub signature: [u32; 6],
    pub max_name_bytes: u32,
    pub max_signature_scale: u32,
    pub unchecked_signature_bytes: &'static [usize],
}

/// The FM26 values, which FM27 keeps.
pub(super) const NAME_POOLS: NamePoolLayout = NamePoolLayout {
    signature: [46421, 0, 1024, 256, 2048, 0],
    max_name_bytes: 1024,
    max_signature_scale: 1024,
    unchecked_signature_bytes: &[5, 6, 7, 8],
};

/// How to recognise player records in `game_db` and where the fields this reader uses sit.
///
/// A marker candidate starts `marker_offset` bytes before each `marker` hit. A completeness
/// candidate starts `ratings_offset` bytes before `ratings_count` bytes in `rating_range`
/// followed by `attribute_count` bytes in the attribute range, searched only after the name
/// pools. Every other offset counts from the record start, and ranges are inclusive. The
/// `*_attribute_index` values index the attribute bytes at `attributes_offset`.
/// `decode_extent` is one past the last byte a full decode reads.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct PlayerRecordLayout {
    pub marker: [u8; 4],
    pub marker_offset: usize,
    /// The header fields sit before the record start, this many bytes back.
    pub pindex_back: usize,
    pub uid_back: usize,
    pub uid_copy_back: usize,
    /// Identities from here up may carry a distinct second header word, if the object kind
    /// is a player's.
    pub allocated_uid_minimum: u32,
    pub object_kind_back: usize,
    pub player_kinds: &'static [u8],
    pub home_reputation_back: usize,
    pub current_reputation_back: usize,
    pub world_reputation_back: usize,
    pub current_ability_offset: usize,
    pub current_ability_range: (u16, u16),
    pub potential_ability_offset: usize,
    pub potential_ability_range: (i16, i16),
    pub reputation_bucket_offset: usize,
    pub reputation_bucket_range: (u8, u8),
    pub ratings_offset: usize,
    pub ratings_count: usize,
    pub rating_range: (u8, u8),
    pub attributes_offset: usize,
    pub attribute_count: usize,
    pub attribute_range: (u8, u8),
    /// Edited records may exceed the ordinary maxima up to these.
    pub edited_ability_maximum: u16,
    pub edited_attribute_maximum: u8,
    pub finishing_attribute_index: usize,
    pub handling_attribute_index: usize,
    pub throwing_attribute_index: usize,
    pub club_join_date_offset: usize,
    pub match_sharpness_offset: usize,
    pub condition_offset: usize,
    pub height_offset: usize,
    pub decode_extent: usize,
}

/// The FM26 values, which FM27 keeps. The position ratings store GK first, and the attribute
/// indexes follow fmsave's `attribute_field_order`, which no foot byte precedes.
pub(super) const PLAYER_RECORDS: PlayerRecordLayout = PlayerRecordLayout {
    marker: [0x01, 0x00, 0x6C, 0x07],
    marker_offset: 102,
    pindex_back: 19,
    uid_back: 15,
    uid_copy_back: 11,
    allocated_uid_minimum: 2_000_000_000,
    object_kind_back: 7,
    player_kinds: &[2, 3],
    home_reputation_back: 6,
    current_reputation_back: 4,
    world_reputation_back: 2,
    current_ability_offset: 0,
    current_ability_range: (0, 200),
    potential_ability_offset: 2,
    potential_ability_range: (-10, 200),
    reputation_bucket_offset: 8,
    reputation_bucket_range: (0, 200),
    ratings_offset: 24,
    ratings_count: 15,
    rating_range: (1, 20),
    attributes_offset: 39,
    attribute_count: 54,
    attribute_range: (1, 100),
    edited_ability_maximum: 255,
    edited_attribute_maximum: 125,
    finishing_attribute_index: 2,
    handling_attribute_index: 11,
    throwing_attribute_index: 16,
    club_join_date_offset: 98,
    match_sharpness_offset: 106,
    condition_offset: 110,
    height_offset: 121,
    decode_extent: 122,
};

/// An inclusive (lowest, highest) bound on an observed value; `None` leaves that side open.
pub(super) type Bound = (Option<f64>, Option<f64>);

/// Whether `observed` lies inside `bound`. A missing observation never does.
pub(super) fn admits(bound: Bound, observed: Option<f64>) -> bool {
    let (lowest, highest) = bound;
    observed.is_some_and(|value| {
        lowest.map_or(true, |lowest| value >= lowest)
            && highest.map_or(true, |highest| value <= highest)
    })
}

/// The structural checks this reader enforces, from fmsave's gate bounds. They apply only when
/// `game_db` is at least `minimum_applies_from_bytes` long, as on every full save.
#[derive(Debug, PartialEq)]
pub(super) struct GateBounds {
    pub minimum_applies_from_bytes: usize,
    pub height_range_cm: (u8, u8),
    pub home_reputation_window: u16,
    pub condition_sharpness_maximum: u16,
    pub natural_goalkeeper_rating: u8,
    pub goalkeeper_block_low_maximum: u8,
    pub players_minimum: Bound,
    pub handling_above_finishing: Bound,
    pub outfield_goalkeeper_block_low: Bound,
    pub with_natural_position: Bound,
    pub height_in_range: Bound,
    pub height_median: Bound,
    pub condition_sharpness_in_range: Bound,
    pub join_date_valid: Bound,
    pub world_not_above_current: Bound,
    pub home_near_current: Bound,
    pub season_stats_records_keyed_to_players: Bound,
    pub season_stats_players_with_record: Bound,
    pub season_stats_overall_sums_competitions: Bound,
    pub season_stats_minutes_in_range: Bound,
}

/// FM26's player bounds, and FM27's season-statistics bounds, which require every record,
/// player, overall line, and line to pass.
pub(super) const FM27_GATE_BOUNDS: GateBounds = GateBounds {
    minimum_applies_from_bytes: 16 * 1024 * 1024,
    height_range_cm: (150, 210),
    home_reputation_window: 1_000,
    condition_sharpness_maximum: 10_000,
    natural_goalkeeper_rating: 18,
    goalkeeper_block_low_maximum: 30,
    players_minimum: (Some(5_000.0), None),
    handling_above_finishing: (Some(0.10), Some(0.30)),
    outfield_goalkeeper_block_low: (Some(0.80), None),
    with_natural_position: (Some(0.97), None),
    height_in_range: (Some(0.999), None),
    height_median: (Some(170.0), Some(190.0)),
    condition_sharpness_in_range: (Some(0.999), None),
    join_date_valid: (Some(0.05), None),
    world_not_above_current: (Some(0.95), None),
    home_near_current: (Some(0.95), None),
    season_stats_records_keyed_to_players: (Some(1.0), Some(1.0)),
    season_stats_players_with_record: (Some(1.0), Some(1.0)),
    season_stats_overall_sums_competitions: (Some(1.0), Some(1.0)),
    season_stats_minutes_in_range: (Some(1.0), Some(1.0)),
};

pub(super) const NAME_POOL_LAYOUTS: &[LayoutEntry<NamePoolLayout>] = &[LayoutEntry {
    schema: Some(4000),
    build: REGISTERED_FM27_BUILD,
    layout: &NAME_POOLS,
}];

pub(super) const PLAYER_RECORD_LAYOUTS: &[LayoutEntry<PlayerRecordLayout>] = &[LayoutEntry {
    schema: Some(4000),
    build: REGISTERED_FM27_BUILD,
    layout: &PLAYER_RECORDS,
}];

pub(super) const GATE_BOUNDS_LAYOUTS: &[LayoutEntry<GateBounds>] = &[LayoutEntry {
    schema: Some(4000),
    build: REGISTERED_FM27_BUILD,
    layout: &FM27_GATE_BOUNDS,
}];

/// How a line stores one statistic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FieldWidth {
    U8,
    U16,
    I16,
}

/// One statistic of a line: its offset from the line start, its width, and how many stored
/// units make one public unit (1 for a count).
#[derive(Debug, PartialEq, Eq)]
pub(super) struct LineField {
    pub stat: SeasonStat,
    pub offset: usize,
    pub width: FieldWidth,
    pub unit: u32,
}

/// The kind of match a line slot counts, in fmsave's `SeasonStatsKind` terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum SlotKind {
    NonCompetitive,
    League,
    Cup,
    Continental,
    International,
    Overall,
    CalendarYearOverall,
    CalendarYearInternational,
}

/// The season-statistics section: one unlisted frame of the span region, one record per
/// player.
///
/// The frame's body is the section header carrying `section_schema`, then records back to back,
/// then a zero byte ending the list and `footer_bytes` that end the frame. A record is a `01`
/// byte, a u32 key (the player's pindex plus one), `header_bytes` of header, `own_slots` slots,
/// `block_bytes` of block, a u32 count of other teams, each `other_team_lead_bytes` then a u32
/// team id and `other_team_slots` slots, a u32 count of `rating_item_bytes` items, and a form
/// list: a count byte then `form_entry_bytes` per entry. A slot is a zero byte when absent, or
/// a `line_bytes` line starting with `line_marker`.
///
/// `outfield_fields` and `goalkeeper_fields` share their words: a line reads one set or the
/// other depending on whether its player is a natural goalkeeper. A goalkeeper's penalty
/// words are also the tackle words `fields` reads for every player. `rating_sum_offset` holds
/// `rating_scale` times the sum of the player's match ratings. `slot_kinds` names each own
/// slot; another team's slots are the first `other_team_slots` of them.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct SeasonStatsLayout {
    pub section_schema: u16,
    pub header_bytes: usize,
    pub own_slots: usize,
    pub other_team_slots: usize,
    pub line_bytes: usize,
    pub line_marker: &'static [u8],
    pub block_bytes: usize,
    pub other_team_lead_bytes: usize,
    pub maximum_other_teams: u32,
    pub rating_item_bytes: usize,
    pub maximum_rating_items: u32,
    pub form_entry_bytes: usize,
    pub maximum_form_entries: u8,
    pub footer_bytes: usize,
    pub rating_sum_offset: usize,
    pub rating_scale: u32,
    pub minutes_offset: usize,
    pub starts_offset: usize,
    pub substitute_appearances_offset: usize,
    pub rated_appearances_offset: usize,
    pub fields: &'static [LineField],
    pub outfield_fields: &'static [LineField],
    pub goalkeeper_fields: &'static [LineField],
    pub unsupported: &'static [SeasonStat],
    pub maximum_minutes_per_appearance: u32,
    pub slot_kinds: &'static [SlotKind],
}

const fn count(stat: SeasonStat, offset: usize, width: FieldWidth) -> LineField {
    scaled(stat, offset, width, 1)
}

const fn scaled(stat: SeasonStat, offset: usize, width: FieldWidth, unit: u32) -> LineField {
    LineField {
        stat,
        offset,
        width,
        unit,
    }
}

/// The FM26 line fields, at the same offsets on FM27's longer line. Minutes, starts,
/// substitute appearances, and rated appearances sit at the layout's own offsets.
///
/// fmsave reads no team results. Among players who appeared in every league match of their
/// team on an FM27 `27.0.0+2493426` save, games won and games lost equalled the league
/// table's on 98%, and team goals equalled its goals for on 97% of those who played every
/// minute. Team goals are the goals the team scored while the player was on the pitch. A
/// line stores no draws.
const SEASON_LINE_FIELDS: &[LineField] = {
    use FieldWidth::{I16, U16, U8};
    use SeasonStat::*;
    &[
        count(Goals, 9, U8),
        count(Assists, 10, U8),
        count(GoalsAllowed, 11, U8),
        count(PenaltiesTaken, 12, U8),
        count(PenaltiesScored, 13, U8),
        count(PlayerOfTheMatch, 14, U8),
        count(RedCards, 15, U8),
        count(YellowCards, 16, U8),
        count(CleanSheets, 17, U8),
        count(TeamGoals, 19, U16),
        count(GamesWon, 23, U16),
        count(GamesLost, 25, U16),
        count(PassesAttempted, 27, U16),
        count(PassesCompleted, 29, U16),
        count(TacklesAttempted, 31, U16),
        count(TacklesCompleted, 33, U16),
        count(Dribbles, 39, U16),
        count(FoulsMade, 41, U16),
        count(FoulsAgainst, 43, U16),
        count(Shots, 45, U16),
        count(ShotsOnTarget, 47, U16),
        count(MistakesLeadingToGoal, 59, U16),
        scaled(DistanceKm, 65, U16, 10),
        count(Offsides, 67, U16),
        count(KeyPasses, 73, U16),
        count(KeyTackles, 75, U16),
        count(KeyHeaders, 77, U16),
        count(Interceptions, 79, U16),
        count(ClearCutChancesCreated, 81, U16),
        count(PossessionLost, 87, U16),
        count(PossessionWon, 89, U16),
        scaled(ExpectedGoals, 91, U16, 100),
        scaled(ExpectedAssists, 93, U16, 100),
        count(FreeKickShots, 99, U16),
        count(GoalsOutsideBox, 101, U16),
        count(OpenPlayCrossesAttempted, 109, U16),
        count(ShotsOutsideBox, 111, U16),
        scaled(ExpectedGoalsPrevented, 113, I16, 100),
        count(HighIntensitySprints, 115, U16),
        count(ProgressivePasses, 119, U16),
        count(PressuresAttempted, 121, U16),
        count(PressuresCompleted, 123, U16),
        count(OpenPlayKeyPasses, 125, U16),
        scaled(NonPenaltyExpectedGoals, 127, U16, 100),
        count(CrossesAttempted, 131, U16),
        count(CrossesCompleted, 133, U16),
        count(ShotsBlocked, 135, U16),
    ]
};

const SEASON_OUTFIELD_FIELDS: &[LineField] = &[
    count(SeasonStat::AerialChallengesAttempted, 35, FieldWidth::U16),
    count(SeasonStat::HeadersWon, 37, FieldWidth::U16),
    count(SeasonStat::OpenPlayCrossesCompleted, 71, FieldWidth::U16),
    count(SeasonStat::Blocks, 83, FieldWidth::U16),
    count(SeasonStat::Clearances, 95, FieldWidth::U16),
];

/// fmsave reads the saves and shots faced. On 2026-11-10 the other three matched the game on
/// build `27.0.0+2493426`: a goalkeeper's tackle words hold penalties faced and saved, which
/// the game also shows as that goalkeeper's tackles, and the open-play-crosses-completed word
/// holds the summed xG of the on-target shots faced.
const SEASON_GOALKEEPER_FIELDS: &[LineField] = &[
    count(SeasonStat::PenaltiesFaced, 31, FieldWidth::U16),
    count(SeasonStat::PenaltiesSaved, 33, FieldWidth::U16),
    count(SeasonStat::SavesHeld, 35, FieldWidth::U16),
    count(SeasonStat::SavesParried, 37, FieldWidth::U16),
    count(SeasonStat::SavesTipped, 69, FieldWidth::U16),
    scaled(SeasonStat::ExpectedGoalsFaced, 71, FieldWidth::U16, 100),
    count(SeasonStat::ShotsOnTargetFaced, 83, FieldWidth::U16),
];

/// FM26's layout with FM27's line marker and line length. Goals from outside the box stay
/// unsupported, as in fmsave: the word counts them only in matches the game plays in full
/// detail. fmsave also withholds both FM27 pressure counts as unconfirmed. This reader reads
/// them: on 2026-11-10 they matched the game's player statistics on build `27.0.0+2493426`.
pub(super) const FM27_SEASON_STATS: SeasonStatsLayout = SeasonStatsLayout {
    section_schema: 15,
    header_bytes: 16,
    own_slots: 8,
    other_team_slots: 6,
    line_bytes: 155,
    line_marker: b"\x01\x07",
    block_bytes: 20,
    other_team_lead_bytes: 1,
    maximum_other_teams: 16,
    rating_item_bytes: 8,
    maximum_rating_items: 256,
    form_entry_bytes: 3,
    maximum_form_entries: 16,
    footer_bytes: 16,
    rating_sum_offset: 2,
    rating_scale: 10,
    minutes_offset: 4,
    starts_offset: 6,
    substitute_appearances_offset: 7,
    rated_appearances_offset: 8,
    fields: SEASON_LINE_FIELDS,
    outfield_fields: SEASON_OUTFIELD_FIELDS,
    goalkeeper_fields: SEASON_GOALKEEPER_FIELDS,
    unsupported: &[SeasonStat::GoalsOutsideBox],
    // A match runs at most 120 minutes plus stoppage time.
    maximum_minutes_per_appearance: 130,
    slot_kinds: &[
        SlotKind::NonCompetitive,
        SlotKind::League,
        SlotKind::Cup,
        SlotKind::Continental,
        SlotKind::International,
        SlotKind::Overall,
        SlotKind::CalendarYearOverall,
        SlotKind::CalendarYearInternational,
    ],
};

/// Registered for the span region with no schema, as fmsave registers it. The section's own
/// schema is matched when its frame is found.
pub(super) const SEASON_STATS_LAYOUTS: &[LayoutEntry<SeasonStatsLayout>] = &[LayoutEntry {
    schema: None,
    build: REGISTERED_FM27_BUILD,
    layout: &FM27_SEASON_STATS,
}];
