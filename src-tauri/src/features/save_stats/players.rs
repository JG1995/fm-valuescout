//! Player records in `game_db`: where they are, the game's Unique ID, and the goalkeeper
//! decision, with the structural checks the record scan must pass.
//!
//! Ported from fmsave 0.10.0 (commit a71abe9b7b38e35affa05c49c3619ff05eb21795, MIT
//! License, Copyright (c) 2026 Randal S. Olson): `readers/player_scan.py` (the marker and
//! completeness scans, acceptance, and the closing person header), the pool locator of
//! `readers/names.py`, `_derived_positions` and the record-field counts of
//! `collect_player_stats` from `readers/players.py`, and the matching gates of
//! `evaluate_players` in `_checks.py`. See `THIRD_PARTY_NOTICES.md`.
//!
//! Every offset is checked against `game_db` before it is read: a record the scan accepts
//! reaches `decode_extent`, or the scan fails.

use std::collections::HashSet;

use super::container::{read_u16, read_u32};
use super::decode_date;
use super::layouts::{admits, GateBounds, NamePoolLayout, PlayerRecordLayout};
use super::SaveStatsError;

/// The value a u32 id field stores when it refers to nothing.
const MISSING_REFERENCE: u32 = 0xFFFF_FFFF;
/// A position rating from which the position is natural.
const NATURAL_POSITION_RATING: u8 = 18;
const SIGNATURE_BYTES: usize = 24;
const POOL_COUNT: usize = 3;
const POOL_ENTRY_HEADER_BYTES: usize = 8;
const CLOSING_HEADER_BYTES: usize = 12;

/// What the season statistics need of one player record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlayerRecord {
    /// The key the season statistics store, less one.
    pub pindex: u32,
    pub unique_id: Option<u32>,
    /// Whether GK is a natural position, which decides how a line's shared words read.
    pub goalkeeper: bool,
}

/// Every player record in `game_db`, in record offset order, once the scan's checks pass.
pub(super) fn read_players(
    game_db: &[u8],
    name_pools: &NamePoolLayout,
    layout: &PlayerRecordLayout,
    bounds: &GateBounds,
) -> Result<Vec<PlayerRecord>, SaveStatsError> {
    let pools_end = name_pools_end(game_db, name_pools)?;
    let candidates = locate_player_records(game_db, pools_end, layout)?;
    check_players(
        &record_counts(game_db, &candidates, layout, bounds)?,
        bounds,
        game_db.len(),
    )?;
    let mut players = Vec::with_capacity(candidates.len());
    for (position, candidate) in candidates.iter().enumerate() {
        let next = candidates.get(position + 1);
        let ratings_at = candidate.offset + layout.ratings_offset;
        players.push(PlayerRecord {
            pindex: candidate.pindex,
            unique_id: closing_unique_id(game_db, candidate, next, layout)?,
            goalkeeper: game_db[ratings_at] >= NATURAL_POSITION_RATING,
        });
    }
    Ok(players)
}

/// An accepted record: its start in `game_db`, its pindex, and its header uid.
#[derive(Debug, Clone, Copy)]
struct Candidate {
    offset: usize,
    pindex: u32,
    uid: u32,
}

/// Every start position of `needle` lying wholly inside `[start, end)`, ascending.
fn positions_of<'a>(
    haystack: &'a [u8],
    needle: &'a [u8],
    start: usize,
    end: usize,
) -> impl Iterator<Item = usize> + 'a {
    let window = haystack.get(start..end.min(haystack.len())).unwrap_or(&[]);
    window
        .windows(needle.len())
        .enumerate()
        .filter(move |(_, bytes)| *bytes == needle)
        .map(move |(position, _)| start + position)
}

/// The offset just past the three name pools, which must follow exactly one signature.
fn name_pools_end(game_db: &[u8], layout: &NamePoolLayout) -> Result<usize, SaveStatsError> {
    let mut found = None;
    let mut first_error = None;
    for signature_at in signature_candidates(game_db, layout) {
        match read_pools(game_db, signature_at + SIGNATURE_BYTES, layout) {
            Ok(_) if found.is_some() => return Err(SaveStatsError::Corrupt),
            Ok(end) => found = Some(end),
            Err(error) => {
                first_error.get_or_insert(error);
            }
        }
    }
    found.ok_or(first_error.unwrap_or(SaveStatsError::Corrupt))
}

/// Offsets where the signature may start: its capacity words scaled together and its
/// unchecked bytes read as zero.
fn signature_candidates(game_db: &[u8], layout: &NamePoolLayout) -> Vec<usize> {
    let words = layout.signature;
    let signature: Vec<u8> = words.iter().flat_map(|word| word.to_le_bytes()).collect();
    let prefix_length = layout
        .unchecked_signature_bytes
        .iter()
        .copied()
        .fold(8, usize::min);
    let fits = |at: usize| {
        let Some(stored) = game_db.get(at..at + SIGNATURE_BYTES) else {
            return false;
        };
        let mut stored = stored.to_vec();
        for &unchecked in layout.unchecked_signature_bytes {
            stored[unchecked] = 0;
        }
        let stored: Vec<u32> = stored
            .chunks_exact(4)
            .map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]]))
            .collect();
        let (scale, remainder) = (stored[3] / words[3], stored[3] % words[3]);
        remainder == 0
            && (1..=layout.max_signature_scale).contains(&scale)
            && stored[..2] == words[..2]
            && (2..5).all(|word| Some(stored[word]) == words[word].checked_mul(scale))
            && stored[5] == words[5]
    };
    positions_of(game_db, &signature[..prefix_length], 0, game_db.len())
        .filter(|&at| fits(at))
        .collect()
}

/// Walks the three pools from the first entry count, checking each entry's id and length.
fn read_pools(
    game_db: &[u8],
    mut cursor: usize,
    layout: &NamePoolLayout,
) -> Result<usize, SaveStatsError> {
    for _ in 0..POOL_COUNT {
        let entry_count = read_u32(game_db, cursor)? as usize;
        cursor += 4;
        if entry_count.saturating_mul(POOL_ENTRY_HEADER_BYTES) > game_db.len() - cursor {
            return Err(SaveStatsError::Corrupt);
        }
        for entry_index in 0..entry_count {
            let entry_id = read_u32(game_db, cursor)?;
            let name_length = read_u32(game_db, cursor + 4)?;
            if entry_id as usize != entry_index || name_length > layout.max_name_bytes {
                return Err(SaveStatsError::Corrupt);
            }
            cursor += POOL_ENTRY_HEADER_BYTES + name_length as usize;
        }
        if cursor > game_db.len() {
            return Err(SaveStatsError::Corrupt);
        }
    }
    Ok(cursor)
}

/// Accepted records from the marker scan and the completeness scan after the name pools,
/// merged by offset. No records, or a pindex or uid in two records, fails the scan.
fn locate_player_records(
    game_db: &[u8],
    pools_end: usize,
    layout: &PlayerRecordLayout,
) -> Result<Vec<Candidate>, SaveStatsError> {
    let mut candidates = Vec::new();
    for hit in positions_of(game_db, &layout.marker, 0, game_db.len()) {
        let Some(offset) = hit.checked_sub(layout.marker_offset) else {
            continue;
        };
        if let Some(candidate) = accept_candidate(game_db, offset, layout, true)? {
            candidates.push(candidate);
        }
    }
    let marker_offsets: HashSet<usize> = candidates.iter().map(|found| found.offset).collect();
    for ratings_at in completeness_matches(game_db, pools_end, layout) {
        let Some(offset) = ratings_at.checked_sub(layout.ratings_offset) else {
            continue;
        };
        if marker_offsets.contains(&offset) {
            continue;
        }
        if let Some(candidate) = accept_candidate(game_db, offset, layout, false)? {
            candidates.push(candidate);
        }
    }
    candidates.sort_by_key(|candidate| candidate.offset);
    if candidates.is_empty() {
        return Err(SaveStatsError::Corrupt);
    }
    let mut pindexes = HashSet::new();
    let mut uids = HashSet::new();
    for candidate in &candidates {
        if !pindexes.insert(candidate.pindex) || !uids.insert(candidate.uid) {
            return Err(SaveStatsError::Corrupt);
        }
    }
    Ok(candidates)
}

/// The highest byte the completeness pattern's attribute bytes may hold.
fn scan_attribute_maximum(layout: &PlayerRecordLayout) -> u8 {
    layout
        .attribute_range
        .1
        .max(layout.edited_attribute_maximum)
}

fn in_range(value: u8, (lowest, highest): (u8, u8)) -> bool {
    (lowest..=highest).contains(&value)
}

/// Whether `ratings_count` rating bytes then `attribute_count` attribute bytes start at `at`.
fn completeness_pattern_at(game_db: &[u8], at: usize, layout: &PlayerRecordLayout) -> bool {
    let attributes_at = at + layout.ratings_count;
    let attribute_range = (layout.attribute_range.0, scan_attribute_maximum(layout));
    match game_db.get(at..attributes_at + layout.attribute_count) {
        Some(bytes) => {
            let (ratings, attributes) = bytes.split_at(layout.ratings_count);
            ratings
                .iter()
                .all(|&rating| in_range(rating, layout.rating_range))
                && attributes
                    .iter()
                    .all(|&attribute| in_range(attribute, attribute_range))
        }
        None => false,
    }
}

/// The start of each completeness-pattern match at or after `start`, found left to right
/// without overlap, as a regular-expression search finds them.
///
/// Every pattern byte lies in the hull of the rating and attribute ranges, so a match lies
/// inside one run of hull bytes at least the pattern long. Inside such a run, a match is any
/// position where `ratings_count` rating bytes start and the pattern still ends in the run.
fn completeness_matches(game_db: &[u8], start: usize, layout: &PlayerRecordLayout) -> Vec<usize> {
    let pattern_length = layout.ratings_count + layout.attribute_count;
    let hull = (
        layout.rating_range.0.min(layout.attribute_range.0),
        layout.rating_range.1.max(scan_attribute_maximum(layout)),
    );
    let mut matches = Vec::new();
    let mut position = start;
    while position < game_db.len() {
        let Some(run_length) = game_db[position..]
            .iter()
            .position(|&byte| in_range(byte, hull))
        else {
            break;
        };
        let run_start = position + run_length;
        let run_end = game_db[run_start..]
            .iter()
            .position(|&byte| !in_range(byte, hull))
            .map_or(game_db.len(), |length| run_start + length);
        if run_end - run_start >= pattern_length {
            let mut streak = 0;
            let mut cursor = run_start;
            while cursor < run_end {
                streak = if in_range(game_db[cursor], layout.rating_range) {
                    streak + 1
                } else {
                    0
                };
                cursor += 1;
                if streak == layout.ratings_count {
                    let match_start = cursor - layout.ratings_count;
                    if match_start + pattern_length > run_end {
                        break;
                    }
                    matches.push(match_start);
                    cursor = match_start + pattern_length;
                    streak = 0;
                }
            }
        }
        position = run_end;
    }
    matches
}

/// The record at `offset` if every acceptance check holds. An accepted record that does not
/// reach `decode_extent` is a truncated record, which fails the scan.
fn accept_candidate(
    game_db: &[u8],
    offset: usize,
    layout: &PlayerRecordLayout,
    check_pattern: bool,
) -> Result<Option<Candidate>, SaveStatsError> {
    let header_back = layout.pindex_back.max(layout.object_kind_back);
    let checks_end = offset + layout.attributes_offset + layout.attribute_count;
    if offset < header_back || checks_end > game_db.len() {
        return Ok(None);
    }
    let back_u32 = |back: usize| read_u32(game_db, offset - back);
    let uid = back_u32(layout.uid_back)?;
    let uid_copy = back_u32(layout.uid_copy_back)?;
    if [0, MISSING_REFERENCE].contains(&uid) || [0, MISSING_REFERENCE].contains(&uid_copy) {
        return Ok(None);
    }
    let player_kind = layout
        .player_kinds
        .contains(&game_db[offset - layout.object_kind_back]);
    if uid != uid_copy && (uid < layout.allocated_uid_minimum || !player_kind) {
        return Ok(None);
    }
    let current_ability = read_u16(game_db, offset + layout.current_ability_offset)?;
    let potential_ability = read_u16(game_db, offset + layout.potential_ability_offset)? as i16;
    let attributes_at = offset + layout.attributes_offset;
    let highest_attribute = game_db[attributes_at..attributes_at + layout.attribute_count]
        .iter()
        .copied()
        .max()
        .unwrap_or(0);
    let (lowest_ability, mut highest_ability) = layout.current_ability_range;
    let (lowest_potential, mut highest_potential) = layout.potential_ability_range;
    if current_ability > highest_ability
        || potential_ability > highest_potential
        || highest_attribute > layout.attribute_range.1
    {
        if uid != uid_copy || !player_kind {
            return Ok(None);
        }
        highest_ability = layout.edited_ability_maximum;
        highest_potential = layout.edited_ability_maximum as i16;
    }
    let bucket = game_db[offset + layout.reputation_bucket_offset];
    if !(lowest_ability..=highest_ability).contains(&current_ability)
        || !(lowest_potential..=highest_potential).contains(&potential_ability)
        || !in_range(bucket, layout.reputation_bucket_range)
        || (check_pattern
            && !completeness_pattern_at(game_db, offset + layout.ratings_offset, layout))
    {
        return Ok(None);
    }
    if offset + layout.decode_extent > game_db.len() {
        return Err(SaveStatsError::Corrupt);
    }
    Ok(Some(Candidate {
        offset,
        pindex: back_u32(layout.pindex_back)?,
        uid,
    }))
}

/// The Unique ID from the one sound header that closes this player's object: it carries the
/// pindex plus one, then a doubled uid above this record's uid and not above the next
/// record's, since objects are stored in ascending uid order. `None` unless exactly one
/// header between the record's decoded fields and the next record qualifies.
fn closing_unique_id(
    game_db: &[u8],
    record: &Candidate,
    next: Option<&Candidate>,
    layout: &PlayerRecordLayout,
) -> Result<Option<u32>, SaveStatsError> {
    let Some(closing_id) = record.pindex.checked_add(1) else {
        return Ok(None);
    };
    if closing_id == MISSING_REFERENCE {
        return Ok(None);
    }
    let window_end = next.map_or(game_db.len(), |next| next.offset);
    let uid_ceiling = next.map(|next| next.uid);
    let last_header_start = game_db.len().saturating_sub(CLOSING_HEADER_BYTES);
    let needle = closing_id.to_le_bytes();
    let search_start = record.offset + layout.decode_extent;
    let mut found = None;
    for hit in positions_of(game_db, &needle, search_start, window_end) {
        if hit > last_header_start {
            break;
        }
        let uid = read_u32(game_db, hit + 4)?;
        let uid_copy = read_u32(game_db, hit + 8)?;
        if uid == uid_copy
            && uid != MISSING_REFERENCE
            && uid > record.uid
            && uid_ceiling.map_or(true, |ceiling| uid <= ceiling)
        {
            if found.is_some() {
                return Ok(None);
            }
            found = Some(uid);
        }
    }
    Ok(found)
}

/// What the record checks count, over every accepted record.
#[derive(Debug, Default, Clone, PartialEq)]
struct RecordCounts {
    records: usize,
    handling_above_finishing: usize,
    outfield_players: usize,
    outfield_goalkeeper_block_low: usize,
    with_natural_position: usize,
    height_in_range: usize,
    height_median: Option<u8>,
    condition_sharpness_in_range: usize,
    with_valid_join_date: usize,
    world_not_above_current: usize,
    home_near_current: usize,
}

fn record_counts(
    game_db: &[u8],
    candidates: &[Candidate],
    layout: &PlayerRecordLayout,
    bounds: &GateBounds,
) -> Result<RecordCounts, SaveStatsError> {
    let mut counts = RecordCounts {
        records: candidates.len(),
        ..RecordCounts::default()
    };
    let mut heights = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let offset = candidate.offset;
        let ratings_at = offset + layout.ratings_offset;
        let ratings = &game_db[ratings_at..ratings_at + layout.ratings_count];
        let attributes_at = offset + layout.attributes_offset;
        let attributes = &game_db[attributes_at..attributes_at + layout.attribute_count];
        let handling = attributes[layout.handling_attribute_index];
        counts.handling_above_finishing +=
            usize::from(handling > attributes[layout.finishing_attribute_index]);
        if ratings[0] < bounds.natural_goalkeeper_rating {
            counts.outfield_players += 1;
            counts.outfield_goalkeeper_block_low += usize::from(
                handling <= bounds.goalkeeper_block_low_maximum
                    && attributes[layout.throwing_attribute_index]
                        <= bounds.goalkeeper_block_low_maximum,
            );
        }
        counts.with_natural_position += usize::from(
            ratings
                .iter()
                .any(|&rating| rating >= NATURAL_POSITION_RATING),
        );
        let height = game_db[offset + layout.height_offset];
        heights.push(height);
        counts.height_in_range += usize::from(in_range(height, bounds.height_range_cm));
        let condition = read_u16(game_db, offset + layout.condition_offset)?;
        let sharpness = read_u16(game_db, offset + layout.match_sharpness_offset)?;
        counts.condition_sharpness_in_range += usize::from(
            condition <= bounds.condition_sharpness_maximum
                && sharpness <= bounds.condition_sharpness_maximum,
        );
        counts.with_valid_join_date +=
            usize::from(decode_date(game_db, offset + layout.club_join_date_offset)?.is_some());
        let home = read_u16(game_db, offset - layout.home_reputation_back)?;
        let current = read_u16(game_db, offset - layout.current_reputation_back)?;
        let world = read_u16(game_db, offset - layout.world_reputation_back)?;
        counts.world_not_above_current += usize::from(world <= current);
        counts.home_near_current +=
            usize::from(home.abs_diff(current) <= bounds.home_reputation_window);
    }
    heights.sort_unstable();
    counts.height_median = heights.get(heights.len().saturating_sub(1) / 2).copied();
    Ok(counts)
}

fn rate(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

/// fmsave's player gates that read only the record itself. They apply to a full-size
/// `game_db`; on a smaller one, as in a test fragment, none applies.
fn check_players(
    counts: &RecordCounts,
    bounds: &GateBounds,
    game_db_bytes: usize,
) -> Result<(), SaveStatsError> {
    if game_db_bytes < bounds.minimum_applies_from_bytes {
        return Ok(());
    }
    let records = counts.records;
    let gates = [
        (bounds.players_minimum, Some(records as f64)),
        (
            bounds.handling_above_finishing,
            rate(counts.handling_above_finishing, records),
        ),
        (
            bounds.outfield_goalkeeper_block_low,
            rate(
                counts.outfield_goalkeeper_block_low,
                counts.outfield_players,
            ),
        ),
        (
            bounds.with_natural_position,
            rate(counts.with_natural_position, records),
        ),
        (
            bounds.height_in_range,
            rate(counts.height_in_range, records),
        ),
        (bounds.height_median, counts.height_median.map(f64::from)),
        (
            bounds.condition_sharpness_in_range,
            rate(counts.condition_sharpness_in_range, records),
        ),
        (
            bounds.join_date_valid,
            rate(counts.with_valid_join_date, records),
        ),
        (
            bounds.world_not_above_current,
            rate(counts.world_not_above_current, records),
        ),
        (
            bounds.home_near_current,
            rate(counts.home_near_current, records),
        ),
    ];
    if gates
        .iter()
        .all(|&(bound, observed)| admits(bound, observed))
    {
        Ok(())
    } else {
        Err(SaveStatsError::Corrupt)
    }
}

#[cfg(test)]
mod tests {
    use super::super::layouts::{FM27_GATE_BOUNDS, NAME_POOLS, PLAYER_RECORDS};
    use super::super::test_support::{game_db_payload, ExamplePlayer, PLAYER_RECORD_BODY_BYTES};
    use super::*;

    const FULL_SIZE: usize = 16 * 1024 * 1024;

    /// A gate's name and a change to passing counts that makes it fail.
    type BrokenGate = (&'static str, fn(&mut RecordCounts));

    fn read(game_db: &[u8]) -> Result<Vec<PlayerRecord>, SaveStatsError> {
        read_players(game_db, &NAME_POOLS, &PLAYER_RECORDS, &FM27_GATE_BOUNDS)
    }

    fn passing_counts() -> RecordCounts {
        RecordCounts {
            records: 10_000,
            handling_above_finishing: 2_000,
            outfield_players: 9_000,
            outfield_goalkeeper_block_low: 9_000,
            with_natural_position: 10_000,
            height_in_range: 10_000,
            height_median: Some(180),
            condition_sharpness_in_range: 10_000,
            with_valid_join_date: 9_000,
            world_not_above_current: 10_000,
            home_near_current: 10_000,
        }
    }

    #[test]
    fn records_report_their_unique_id_and_goalkeeper_rating() {
        let players = [
            ExamplePlayer::outfield(10, 1_000, Some(1_500)),
            ExamplePlayer::goalkeeper(20, 2_000, Some(2_500)),
            ExamplePlayer::outfield(30, 3_000, None),
        ];

        let records = read(&game_db_payload(&players)).expect("players");

        assert_eq!(
            records,
            [
                PlayerRecord {
                    pindex: 10,
                    unique_id: Some(1_500),
                    goalkeeper: false
                },
                PlayerRecord {
                    pindex: 20,
                    unique_id: Some(2_500),
                    goalkeeper: true
                },
                PlayerRecord {
                    pindex: 30,
                    unique_id: None,
                    goalkeeper: false
                },
            ]
        );
    }

    #[test]
    fn a_goalkeeper_rating_below_eighteen_is_not_a_natural_goalkeeper() {
        let player = ExamplePlayer {
            gk_rating: 17,
            ..ExamplePlayer::goalkeeper(10, 1_000, Some(1_500))
        };

        let records = read(&game_db_payload(&[player])).expect("players");

        assert!(!records[0].goalkeeper);
    }

    #[test]
    fn a_closing_header_above_the_next_records_uid_is_not_this_players() {
        // The header's uid lies above the next record's uid, so another object closes there.
        let players = [
            ExamplePlayer::outfield(10, 1_000, Some(2_500)),
            ExamplePlayer::outfield(20, 2_000, Some(2_600)),
        ];

        let records = read(&game_db_payload(&players)).expect("players");

        assert_eq!(records[0].unique_id, None);
        assert_eq!(records[1].unique_id, Some(2_600));
    }

    #[test]
    fn scan_failures_are_errors() {
        let repeated_pindex = [
            ExamplePlayer::outfield(10, 1_000, Some(1_500)),
            ExamplePlayer::outfield(10, 2_000, Some(2_500)),
        ];
        let repeated_uid = [
            ExamplePlayer::outfield(10, 1_000, Some(1_500)),
            ExamplePlayer::outfield(20, 1_000, Some(2_500)),
        ];
        let mut truncated_record = game_db_payload(&[ExamplePlayer::outfield(10, 1_000, None)]);
        // The record's checked bytes survive, but it ends before its decoded fields.
        let record_start = truncated_record.len() - PLAYER_RECORD_BODY_BYTES;
        truncated_record.truncate(record_start + 110);
        let mut without_name_pools = game_db_payload(&[ExamplePlayer::outfield(10, 1_000, None)]);
        without_name_pools[0] ^= 0xFF;

        for (case, game_db) in [
            ("no player records", game_db_payload(&[])),
            ("repeated pindex", game_db_payload(&repeated_pindex)),
            ("repeated uid", game_db_payload(&repeated_uid)),
            ("truncated record", truncated_record),
            ("no name pools", without_name_pools),
        ] {
            assert_eq!(read(&game_db), Err(SaveStatsError::Corrupt), "{case}");
        }
    }

    #[test]
    fn each_record_gate_fails_on_a_full_size_game_db() {
        assert_eq!(
            check_players(&passing_counts(), &FM27_GATE_BOUNDS, FULL_SIZE),
            Ok(())
        );
        let failing: [BrokenGate; 10] = [
            ("players minimum", |counts| counts.records = 4_999),
            ("handling above finishing", |counts| {
                counts.handling_above_finishing = 3_500
            }),
            ("outfield goalkeeper block", |counts| {
                counts.outfield_goalkeeper_block_low = 7_000
            }),
            ("natural position", |counts| {
                counts.with_natural_position = 9_000
            }),
            ("height in range", |counts| counts.height_in_range = 9_900),
            ("height median", |counts| counts.height_median = Some(195)),
            ("condition and sharpness", |counts| {
                counts.condition_sharpness_in_range = 9_900
            }),
            ("join date", |counts| counts.with_valid_join_date = 100),
            ("world reputation", |counts| {
                counts.world_not_above_current = 9_000
            }),
            ("home reputation", |counts| counts.home_near_current = 9_000),
        ];

        for (case, break_gate) in failing {
            let mut counts = passing_counts();
            break_gate(&mut counts);
            assert_eq!(
                check_players(&counts, &FM27_GATE_BOUNDS, FULL_SIZE),
                Err(SaveStatsError::Corrupt),
                "{case}"
            );
            assert_eq!(
                check_players(&counts, &FM27_GATE_BOUNDS, FULL_SIZE - 1),
                Ok(()),
                "{case} below the full-save size"
            );
        }
    }
}
