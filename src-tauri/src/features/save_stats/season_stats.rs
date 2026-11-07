//! The season-statistics section: one record per player, walked end to end, and each player's
//! league, cup, and continental totals.
//!
//! Ported from fmsave 0.10.0 (commit a71abe9b7b38e35affa05c49c3619ff05eb21795, MIT
//! License, Copyright (c) 2026 Randal S. Olson): `readers/season_stats.py` (the section
//! search, the walk, line decoding, and the counts of `build_season_stats`) and
//! `evaluate_season_stats` in `_checks.py`. See `THIRD_PARTY_NOTICES.md`.
//!
//! The section is one frame of the span region, which the directory does not list. Every
//! record is read by its own structure rather than searched for, so a byte out of place
//! anywhere fails the walk instead of shifting what later records read. The totals follow the
//! save-file season stats design: competitive club lines only, summed over every team.

use std::collections::{HashMap, HashSet};

use super::container::{
    read_chosen_region_frames, read_u16, read_u32, region_frame_heads, ContainerIndex, SaveSource,
};
use super::layouts::{admits, FieldWidth, GateBounds, LineField, SeasonStatsLayout, SlotKind};
use super::players::PlayerRecord;
use super::{SavePlayerSeasonStats, SaveStatsError, SeasonStat};

/// The unnamed run of frames after the `non_pl_hist_ls` section.
const SPAN_REGION: &str = "unlisted_after_non_pl_hist_ls";
const SECTION_MAGIC: &[u8] = b"\x03\x01tad.";
const RECORD_PRESENT: u8 = 1;
const LIST_END: u8 = 0;
const SLOT_ABSENT: u8 = 0;
const COUNT_BYTES: usize = 4;
/// The kinds of line a player's totals count.
const COUNTED_KINDS: [SlotKind; 3] = [SlotKind::League, SlotKind::Cup, SlotKind::Continental];

/// The decompressed span frames whose body starts with the section header at the layout's
/// schema. None means the save's season statistics are not in the registered schema.
pub(super) fn read_section_frames(
    source: &mut impl SaveSource,
    index: &ContainerIndex,
    layout: &SeasonStatsLayout,
) -> Result<Vec<Vec<u8>>, SaveStatsError> {
    let header = section_header(layout);
    let spans: Vec<_> = region_frame_heads(source, index, SPAN_REGION, header.len())?
        .into_iter()
        .filter(|(_, head)| *head == header)
        .map(|(span, _)| span)
        .collect();
    if spans.is_empty() {
        return Err(SaveStatsError::UnsupportedBuild);
    }
    read_chosen_region_frames(source, index, &spans)
}

fn section_header(layout: &SeasonStatsLayout) -> Vec<u8> {
    let mut header = SECTION_MAGIC.to_vec();
    header.extend_from_slice(&layout.section_schema.to_le_bytes());
    header
}

/// One record as walked: its key and the offset of each line, `None` for an absent slot.
struct RawRecord {
    key: u32,
    own_lines: Vec<Option<usize>>,
    other_team_lines: Vec<Vec<Option<usize>>>,
}

/// Walks every record of a section body, from its header to its footer. FM27 stores one
/// record per key, so a repeated key fails the walk.
fn walk(body: &[u8], layout: &SeasonStatsLayout) -> Result<Vec<RawRecord>, SaveStatsError> {
    let header = section_header(layout);
    if !body.starts_with(&header) {
        return Err(SaveStatsError::Corrupt);
    }
    let end = body.len();
    let read_slots = |mut position: usize, count: usize| {
        let mut lines = Vec::with_capacity(count);
        for _ in 0..count {
            match body.get(position) {
                Some(&SLOT_ABSENT) => {
                    lines.push(None);
                    position += 1;
                }
                Some(_)
                    if body[position..].starts_with(layout.line_marker)
                        && position + layout.line_bytes <= end =>
                {
                    lines.push(Some(position));
                    position += layout.line_bytes;
                }
                _ => return Err(SaveStatsError::Corrupt),
            }
        }
        Ok((lines, position))
    };
    let read_count = |position: usize, maximum: u32| match read_u32(body, position) {
        Ok(count) if count <= maximum => Ok(count as usize),
        _ => Err(SaveStatsError::Corrupt),
    };

    let mut records = Vec::new();
    let mut keys = HashSet::new();
    let mut position = header.len();
    loop {
        match body.get(position) {
            Some(&LIST_END) => break,
            Some(&RECORD_PRESENT) if position + 1 + COUNT_BYTES + layout.header_bytes <= end => {}
            _ => return Err(SaveStatsError::Corrupt),
        }
        let key = read_u32(body, position + 1)?;
        if !keys.insert(key) {
            return Err(SaveStatsError::Corrupt);
        }
        let (own_lines, after_own) = read_slots(
            position + 1 + COUNT_BYTES + layout.header_bytes,
            layout.own_slots,
        )?;
        position = after_own + layout.block_bytes;
        let other_teams = read_count(position, layout.maximum_other_teams)?;
        position += COUNT_BYTES;
        let mut other_team_lines = Vec::with_capacity(other_teams);
        for _ in 0..other_teams {
            let team_at = position + layout.other_team_lead_bytes;
            read_u32(body, team_at)?;
            let (lines, after_team) = read_slots(team_at + COUNT_BYTES, layout.other_team_slots)?;
            other_team_lines.push(lines);
            position = after_team;
        }
        let rating_items = read_count(position, layout.maximum_rating_items)?;
        position += COUNT_BYTES + rating_items * layout.rating_item_bytes;
        let form_entries = *body.get(position).ok_or(SaveStatsError::Corrupt)?;
        if form_entries > layout.maximum_form_entries {
            return Err(SaveStatsError::Corrupt);
        }
        position += 1 + usize::from(form_entries) * layout.form_entry_bytes;
        records.push(RawRecord {
            key,
            own_lines,
            other_team_lines,
        });
    }
    if position + 1 + layout.footer_bytes != end {
        return Err(SaveStatsError::Corrupt);
    }
    Ok(records)
}

/// The one candidate frame that walks end to end, and its records.
fn walk_section<'a>(
    frames: &'a [Vec<u8>],
    layout: &SeasonStatsLayout,
) -> Result<(&'a [u8], Vec<RawRecord>), SaveStatsError> {
    let mut walked = Vec::new();
    let mut first_error = None;
    for frame in frames {
        match walk(frame, layout) {
            Ok(records) => walked.push((frame.as_slice(), records)),
            Err(error) => {
                first_error.get_or_insert(error);
            }
        }
    }
    match walked.len() {
        1 => Ok(walked.remove(0)),
        0 => Err(first_error.unwrap_or(SaveStatsError::Corrupt)),
        _ => Err(SaveStatsError::Corrupt),
    }
}

fn read_field(
    body: &[u8],
    line: usize,
    offset: usize,
    width: FieldWidth,
) -> Result<i64, SaveStatsError> {
    let at = line + offset;
    Ok(match width {
        FieldWidth::U8 => i64::from(*body.get(at).ok_or(SaveStatsError::Corrupt)?),
        FieldWidth::U16 => i64::from(read_u16(body, at)?),
        FieldWidth::I16 => i64::from(read_u16(body, at)? as i16),
    })
}

/// The fields the checks compare: starts, substitute appearances, minutes, goals, assists.
struct SummaryReader<'a> {
    layout: &'a SeasonStatsLayout,
    goals: &'a LineField,
    assists: &'a LineField,
}

impl<'a> SummaryReader<'a> {
    fn new(layout: &'a SeasonStatsLayout) -> Self {
        let field = |stat| {
            layout
                .fields
                .iter()
                .find(|field| field.stat == stat)
                .expect("every season-statistics layout stores goals and assists")
        };
        Self {
            layout,
            goals: field(SeasonStat::Goals),
            assists: field(SeasonStat::Assists),
        }
    }

    fn read(&self, body: &[u8], line: usize) -> Result<[i64; 5], SaveStatsError> {
        let layout = self.layout;
        Ok([
            read_field(body, line, layout.starts_offset, FieldWidth::U8)?,
            read_field(
                body,
                line,
                layout.substitute_appearances_offset,
                FieldWidth::U8,
            )?,
            read_field(body, line, layout.minutes_offset, FieldWidth::U16)?,
            read_field(body, line, self.goals.offset, self.goals.width)?,
            read_field(body, line, self.assists.offset, self.assists.width)?,
        ])
    }
}

/// What the season-statistics checks count over a walked section.
#[derive(Debug, Default, Clone, PartialEq)]
struct SeasonCounts {
    records: usize,
    records_keyed_to_players: usize,
    players: usize,
    players_with_record: usize,
    records_with_overall: usize,
    overall_sums_competitions: usize,
    lines: usize,
    minutes_in_range: usize,
}

fn slot_of(layout: &SeasonStatsLayout, kind: SlotKind) -> usize {
    layout
        .slot_kinds
        .iter()
        .position(|&slot| slot == kind)
        .expect("every season-statistics layout names each slot kind")
}

fn season_counts(
    body: &[u8],
    records: &[RawRecord],
    players_by_pindex: &HashMap<u32, &PlayerRecord>,
    layout: &SeasonStatsLayout,
) -> Result<SeasonCounts, SaveStatsError> {
    let summary = SummaryReader::new(layout);
    let overall_slot = slot_of(layout, SlotKind::Overall);
    let mut counts = SeasonCounts {
        records: records.len(),
        players: players_by_pindex.len(),
        ..SeasonCounts::default()
    };
    for record in records {
        let lines = record
            .own_lines
            .iter()
            .chain(record.other_team_lines.iter().flatten())
            .flatten();
        for &line in lines {
            let [starts, substitutes, minutes, ..] = summary.read(body, line)?;
            counts.lines += 1;
            counts.minutes_in_range += usize::from(
                minutes
                    <= i64::from(layout.maximum_minutes_per_appearance) * (starts + substitutes),
            );
        }
        if let Some(overall) = record.own_lines[overall_slot] {
            counts.records_with_overall += 1;
            let mut added = [0; 5];
            for kind in COUNTED_KINDS {
                if let Some(part) = record.own_lines[slot_of(layout, kind)] {
                    for (total, value) in added.iter_mut().zip(summary.read(body, part)?) {
                        *total += value;
                    }
                }
            }
            counts.overall_sums_competitions += usize::from(added == summary.read(body, overall)?);
        }
        if record
            .key
            .checked_sub(1)
            .is_some_and(|pindex| players_by_pindex.contains_key(&pindex))
        {
            counts.records_keyed_to_players += 1;
            counts.players_with_record += 1;
        }
    }
    Ok(counts)
}

fn share(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

/// fmsave's four season-statistics gates. A share with nothing to divide by judges nothing,
/// and none applies below a full-size `game_db`.
fn check_season_stats(
    counts: &SeasonCounts,
    bounds: &GateBounds,
    game_db_bytes: usize,
) -> Result<(), SaveStatsError> {
    if game_db_bytes < bounds.minimum_applies_from_bytes {
        return Ok(());
    }
    let gates = [
        (
            bounds.season_stats_records_keyed_to_players,
            share(counts.records_keyed_to_players, counts.records),
        ),
        (
            bounds.season_stats_players_with_record,
            share(counts.players_with_record, counts.players),
        ),
        (
            bounds.season_stats_overall_sums_competitions,
            share(
                counts.overall_sums_competitions,
                counts.records_with_overall,
            ),
        ),
        (
            bounds.season_stats_minutes_in_range,
            share(counts.minutes_in_range, counts.lines),
        ),
    ];
    if gates
        .iter()
        .all(|&(bound, observed)| observed.is_none() || admits(bound, observed))
    {
        Ok(())
    } else {
        Err(SaveStatsError::Corrupt)
    }
}

/// Each player's league, cup, and continental totals over their own team and every other team
/// they played for, once the section walks and every check passes. Players with no Unique ID
/// or no counted minutes are left out.
pub(super) fn season_totals(
    frames: &[Vec<u8>],
    players: &[PlayerRecord],
    layout: &SeasonStatsLayout,
    bounds: &GateBounds,
    game_db_bytes: usize,
) -> Result<Vec<SavePlayerSeasonStats>, SaveStatsError> {
    let (body, records) = walk_section(frames, layout)?;
    let players_by_pindex: HashMap<u32, &PlayerRecord> = players
        .iter()
        .map(|player| (player.pindex, player))
        .collect();
    check_season_stats(
        &season_counts(body, &records, &players_by_pindex, layout)?,
        bounds,
        game_db_bytes,
    )?;

    let readable = |extra: &'static [LineField]| -> Vec<&'static LineField> {
        layout
            .fields
            .iter()
            .chain(extra)
            .filter(|field| !layout.unsupported.contains(&field.stat))
            .collect()
    };
    let outfield_fields = readable(layout.outfield_fields);
    let goalkeeper_fields = readable(layout.goalkeeper_fields);
    let counted_slots = COUNTED_KINDS.map(|kind| slot_of(layout, kind));

    let mut totals = Vec::new();
    for record in &records {
        let Some(player) = record
            .key
            .checked_sub(1)
            .and_then(|pindex| players_by_pindex.get(&pindex))
        else {
            continue;
        };
        let Some(unique_id) = player.unique_id else {
            continue;
        };
        let fields = if player.goalkeeper {
            &goalkeeper_fields
        } else {
            &outfield_fields
        };
        let lines = std::iter::once(&record.own_lines)
            .chain(&record.other_team_lines)
            .flat_map(|team| {
                counted_slots
                    .iter()
                    .filter_map(|&slot| team.get(slot).copied().flatten())
            });
        if let Some(stats) = player_totals(body, lines, unique_id, fields, layout)? {
            totals.push(stats);
        }
    }
    Ok(totals)
}

/// The sum of `lines`, or `None` when they hold no minutes. A statistic is `None` only when
/// no line stores it.
fn player_totals(
    body: &[u8],
    lines: impl Iterator<Item = usize>,
    unique_id: u32,
    fields: &[&LineField],
    layout: &SeasonStatsLayout,
) -> Result<Option<SavePlayerSeasonStats>, SaveStatsError> {
    let mut sums = [0_i64; 5];
    let mut stored: [Option<i64>; SeasonStat::COUNT] = [None; SeasonStat::COUNT];
    for line in lines {
        let header = [
            (layout.starts_offset, FieldWidth::U8),
            (layout.substitute_appearances_offset, FieldWidth::U8),
            (layout.minutes_offset, FieldWidth::U16),
            (layout.rated_appearances_offset, FieldWidth::U8),
            (layout.rating_sum_offset, FieldWidth::U16),
        ];
        for (sum, (offset, width)) in sums.iter_mut().zip(header) {
            *sum += read_field(body, line, offset, width)?;
        }
        for field in fields {
            let value = read_field(body, line, field.offset, field.width)?;
            let total = &mut stored[field.stat as usize];
            *total = Some(total.unwrap_or(0) + value);
        }
    }
    let [starts, substitute_appearances, minutes, rated_appearances, rating_sum] = sums;
    if minutes == 0 {
        return Ok(None);
    }
    let mut totals = [None; SeasonStat::COUNT];
    for field in fields {
        totals[field.stat as usize] =
            stored[field.stat as usize].map(|total| total as f64 / f64::from(field.unit));
    }
    // Every sum is of at most a few hundred unsigned 8- and 16-bit words.
    let count = |sum: i64| sum as u32;
    Ok(Some(SavePlayerSeasonStats {
        unique_id,
        starts: count(starts),
        substitute_appearances: count(substitute_appearances),
        minutes: count(minutes),
        rated_appearances: count(rated_appearances),
        rating_sum: rating_sum as f64 / f64::from(layout.rating_scale),
        totals,
    }))
}

#[cfg(test)]
mod tests {
    use super::super::layouts::FM27_GATE_BOUNDS;
    use super::super::test_support::{
        season_stats_body, season_stats_save, stat_line, write_save, ExamplePlayer, ExampleRecord,
        FM27_BUILD, UNREGISTERED_FM27_BUILD,
    };
    use super::super::{read_season_stats, SaveSeasonStats};
    use super::*;

    const FULL_SIZE: usize = 16 * 1024 * 1024;

    /// A gate's name and a change to passing counts that makes it fail.
    type BrokenGate = (&'static str, fn(&mut SeasonCounts));
    const NON_COMPETITIVE: usize = 0;
    const LEAGUE: usize = 1;
    const CUP: usize = 2;
    const CONTINENTAL: usize = 3;
    const INTERNATIONAL: usize = 4;
    const OVERALL: usize = 5;
    const CALENDAR_YEAR_OVERALL: usize = 6;
    const CALENDAR_YEAR_INTERNATIONAL: usize = 7;

    fn read(
        build: &str,
        players: &[ExamplePlayer],
        records: &[ExampleRecord],
    ) -> Result<SaveSeasonStats, SaveStatsError> {
        read_with_schemas(build, 4000, players, &season_stats_body(records, 15))
    }

    fn read_with_schemas(
        build: &str,
        game_db_schema: u16,
        players: &[ExamplePlayer],
        season_stats: &[u8],
    ) -> Result<SaveSeasonStats, SaveStatsError> {
        let directory = tempfile::tempdir().expect("temp dir");
        let save = season_stats_save(build, game_db_schema, players, season_stats);
        read_season_stats(&write_save(directory.path(), &save))
    }

    fn only_player(stats: SaveSeasonStats) -> SavePlayerSeasonStats {
        let [player] = <[_; 1]>::try_from(stats.players).expect("one player");
        player
    }

    /// An outfield player's league, cup, and continental lines, plus a league line for a
    /// second team.
    fn two_team_record(key: u32) -> ExampleRecord {
        let league = stat_line(&[
            ("minutes", 90.0),
            ("starts", 1.0),
            ("rated_appearances", 1.0),
            ("rating_sum", 72.0),
            ("goals", 1.0),
            ("expected_goals", 0.45),
            ("distance_km", 10.5),
        ]);
        let cup = stat_line(&[
            ("minutes", 30.0),
            ("substitute_appearances", 1.0),
            ("rated_appearances", 1.0),
            ("rating_sum", 65.0),
            ("expected_goals", 0.1),
        ]);
        let continental = stat_line(&[
            ("minutes", 0.0),
            ("substitute_appearances", 1.0),
            ("shots", 1.0),
        ]);
        let other_team_league = stat_line(&[
            ("minutes", 80.0),
            ("starts", 1.0),
            ("rated_appearances", 1.0),
            ("rating_sum", 68.0),
            ("assists", 1.0),
            ("expected_goals", 0.2),
            ("distance_km", 9.2),
        ]);
        let overall = stat_line(&[
            ("minutes", 120.0),
            ("starts", 1.0),
            ("substitute_appearances", 2.0),
            ("goals", 1.0),
        ]);
        let mut record = ExampleRecord::with_lines(
            key,
            &[
                (LEAGUE, league),
                (CUP, cup),
                (CONTINENTAL, continental),
                (OVERALL, overall),
            ],
        );
        let mut other_team = vec![None; 6];
        other_team[LEAGUE] = Some(other_team_league);
        record.other_teams.push((902, other_team));
        record
    }

    fn assert_two_team_totals(player: &SavePlayerSeasonStats) {
        assert_eq!(player.unique_id, 1_500);
        assert_eq!(
            (
                player.starts,
                player.substitute_appearances,
                player.minutes,
                player.rated_appearances
            ),
            (2, 2, 200, 3)
        );
        assert!((player.rating_sum - 20.5).abs() < 1e-9);
        assert_eq!(player.total(SeasonStat::Goals), Some(1.0));
        assert_eq!(player.total(SeasonStat::Assists), Some(1.0));
        assert_eq!(player.total(SeasonStat::Shots), Some(1.0));
        let expected_goals = player.total(SeasonStat::ExpectedGoals).expect("xG");
        assert!((expected_goals - 0.75).abs() < 1e-9);
        let distance = player.total(SeasonStat::DistanceKm).expect("distance");
        assert!((distance - 19.7).abs() < 1e-9);
    }

    #[test]
    fn competitive_lines_for_every_team_are_summed() {
        let player = ExamplePlayer::outfield(10, 1_000, Some(1_500));

        let stats = read(FM27_BUILD, &[player], &[two_team_record(11)]).expect("stats");

        assert_eq!(stats.info.build, FM27_BUILD);
        assert_two_team_totals(&only_player(stats));
    }

    #[test]
    fn international_non_competitive_and_calendar_year_lines_are_left_out() {
        let player = ExamplePlayer::outfield(10, 1_000, Some(1_500));
        let excluded = || {
            Some(stat_line(&[
                ("minutes", 900.0),
                ("starts", 10.0),
                ("goals", 9.0),
                ("expected_goals", 5.0),
            ]))
        };
        let mut record = two_team_record(11);
        for slot in [
            NON_COMPETITIVE,
            INTERNATIONAL,
            CALENDAR_YEAR_OVERALL,
            CALENDAR_YEAR_INTERNATIONAL,
        ] {
            record.own_slots[slot] = excluded();
        }
        for slot in [NON_COMPETITIVE, INTERNATIONAL, OVERALL] {
            record.other_teams[0].1[slot] = excluded();
        }

        let stats = read(FM27_BUILD, &[player], &[record]).expect("stats");

        assert_two_team_totals(&only_player(stats));
    }

    #[test]
    fn a_goalkeepers_shared_words_read_as_saves_and_an_outfield_players_as_outfield_fields() {
        let players = [
            ExamplePlayer::goalkeeper(10, 1_000, Some(1_500)),
            ExamplePlayer::outfield(20, 2_000, Some(2_500)),
        ];
        let shared_words = || {
            stat_line(&[
                ("minutes", 90.0),
                ("starts", 1.0),
                ("word_35", 4.0),
                ("word_37", 3.0),
                ("word_69", 2.0),
                ("word_71", 6.0),
                ("word_83", 10.0),
                ("word_95", 7.0),
            ])
        };
        let records = [
            ExampleRecord::with_lines(11, &[(LEAGUE, shared_words())]),
            ExampleRecord::with_lines(21, &[(LEAGUE, shared_words())]),
        ];

        let stats = read(FM27_BUILD, &players, &records).expect("stats");

        let totals = |player: &SavePlayerSeasonStats, stats: &[SeasonStat]| {
            stats
                .iter()
                .map(|&stat| player.total(stat))
                .collect::<Vec<_>>()
        };
        let goalkeeping = [
            SeasonStat::SavesHeld,
            SeasonStat::SavesParried,
            SeasonStat::SavesTipped,
            SeasonStat::ShotsOnTargetFaced,
        ];
        let outfield = [
            SeasonStat::AerialChallengesAttempted,
            SeasonStat::HeadersWon,
            SeasonStat::OpenPlayCrossesCompleted,
            SeasonStat::Blocks,
            SeasonStat::Clearances,
        ];
        let (goalkeeper, outfielder) = (&stats.players[0], &stats.players[1]);
        assert_eq!(
            totals(goalkeeper, &goalkeeping),
            [Some(4.0), Some(3.0), Some(2.0), Some(10.0)]
        );
        assert_eq!(totals(goalkeeper, &outfield), [None; 5]);
        assert_eq!(
            totals(outfielder, &outfield),
            [Some(4.0), Some(3.0), Some(6.0), Some(10.0), Some(7.0)]
        );
        assert_eq!(totals(outfielder, &goalkeeping), [None; 4]);
    }

    #[test]
    fn players_without_counted_minutes_or_a_unique_id_are_left_out() {
        let players = [
            ExamplePlayer::outfield(10, 1_000, Some(1_500)),
            ExamplePlayer::outfield(20, 2_000, None),
            ExamplePlayer::outfield(30, 3_000, Some(3_500)),
        ];
        let played = || stat_line(&[("minutes", 90.0), ("starts", 1.0)]);
        let records = [
            // Minutes only in a friendly.
            ExampleRecord::with_lines(11, &[(NON_COMPETITIVE, played())]),
            ExampleRecord::with_lines(21, &[(LEAGUE, played())]),
            ExampleRecord::with_lines(31, &[(LEAGUE, played())]),
        ];

        let stats = read(FM27_BUILD, &players, &records).expect("stats");

        assert_eq!(only_player(stats).unique_id, 3_500);
    }

    #[test]
    fn unsupported_statistics_are_none_and_stored_zeros_are_zero() {
        let player = ExamplePlayer::outfield(10, 1_000, Some(1_500));
        let record = ExampleRecord::with_lines(
            11,
            &[(
                LEAGUE,
                stat_line(&[
                    ("minutes", 90.0),
                    ("starts", 1.0),
                    ("goals_outside_box", 3.0),
                    ("pressures_attempted", 7.0),
                    ("pressures_completed", 5.0),
                ]),
            )],
        );

        let player = only_player(read(FM27_BUILD, &[player], &[record]).expect("stats"));

        for stat in [
            SeasonStat::GoalsOutsideBox,
            SeasonStat::PressuresAttempted,
            SeasonStat::PressuresCompleted,
        ] {
            assert_eq!(player.total(stat), None, "{}", stat.name());
        }
        assert_eq!(player.total(SeasonStat::Offsides), Some(0.0));
        assert_eq!(player.total(SeasonStat::ExpectedGoalsPrevented), Some(0.0));
    }

    #[test]
    fn a_section_that_does_not_walk_returns_no_statistics() {
        let player = ExamplePlayer::outfield(10, 1_000, Some(1_500));
        let played = || stat_line(&[("minutes", 90.0), ("starts", 1.0)]);
        let record = || ExampleRecord::with_lines(11, &[(LEAGUE, played())]);
        let body = |records: &[ExampleRecord]| season_stats_body(records, 15);

        let repeated_key = body(&[record(), record()]);
        let mut slot_neither_absent_nor_a_line = body(&[record()]);
        // The first own slot, after the record's flag, key, and header.
        slot_neither_absent_nor_a_line[8 + 1 + 4 + 16] = 9;
        let mut too_many_other_teams = record();
        too_many_other_teams.other_teams = vec![(902, vec![None; 6]); 17];
        let too_many_other_teams = body(&[too_many_other_teams]);
        let mut footer_too_long = body(&[record()]);
        footer_too_long.push(0);
        // A line claimed where too few bytes remain: the walk overruns the frame.
        let mut line_past_the_end = body(&[record()]);
        let last_line_at = line_past_the_end.len() - 17 - 1 - 48 - 4 - 8 - 4 - 20 - 6;
        line_past_the_end.truncate(last_line_at);
        line_past_the_end.extend_from_slice(b"\x01\x07");

        for (case, season_stats) in [
            ("repeated key", repeated_key),
            ("unknown slot byte", slot_neither_absent_nor_a_line),
            ("too many other teams", too_many_other_teams),
            ("footer too long", footer_too_long),
            ("line past the end", line_past_the_end),
        ] {
            assert_eq!(
                read_with_schemas(FM27_BUILD, 4000, &[player], &season_stats),
                Err(SaveStatsError::Corrupt),
                "{case}"
            );
            assert_eq!(
                read_with_schemas(UNREGISTERED_FM27_BUILD, 4000, &[player], &season_stats),
                Err(SaveStatsError::UnsupportedBuild),
                "{case} under a borrowed layout"
            );
        }
    }

    #[test]
    fn a_game_db_without_player_records_returns_no_statistics() {
        let record = ExampleRecord::with_lines(11, &[(LEAGUE, stat_line(&[("minutes", 90.0)]))]);

        assert_eq!(
            read(FM27_BUILD, &[], &[record]),
            Err(SaveStatsError::Corrupt)
        );
    }

    #[test]
    fn section_schemas_that_differ_from_the_registered_ones_are_unsupported_builds() {
        let player = ExamplePlayer::outfield(10, 1_000, Some(1_500));
        let record = ExampleRecord::with_lines(11, &[(LEAGUE, stat_line(&[("minutes", 90.0)]))]);

        for build in [FM27_BUILD, UNREGISTERED_FM27_BUILD] {
            assert_eq!(
                read_with_schemas(
                    build,
                    4001,
                    &[player],
                    &season_stats_body(std::slice::from_ref(&record), 15)
                ),
                Err(SaveStatsError::UnsupportedBuild),
                "game_db on {build}"
            );
            assert_eq!(
                read_with_schemas(
                    build,
                    4000,
                    &[player],
                    &season_stats_body(std::slice::from_ref(&record), 16)
                ),
                Err(SaveStatsError::UnsupportedBuild),
                "season statistics on {build}"
            );
        }
        assert!(read_with_schemas(
            UNREGISTERED_FM27_BUILD,
            4000,
            &[player],
            &season_stats_body(&[record], 15)
        )
        .is_ok());
    }

    fn passing_counts() -> SeasonCounts {
        SeasonCounts {
            records: 100,
            records_keyed_to_players: 100,
            players: 100,
            players_with_record: 100,
            records_with_overall: 80,
            overall_sums_competitions: 80,
            lines: 300,
            minutes_in_range: 300,
        }
    }

    #[test]
    fn each_season_gate_fails_on_a_full_size_game_db() {
        assert_eq!(
            check_season_stats(&passing_counts(), &FM27_GATE_BOUNDS, FULL_SIZE),
            Ok(())
        );
        let empty = SeasonCounts::default();
        assert_eq!(
            check_season_stats(&empty, &FM27_GATE_BOUNDS, FULL_SIZE),
            Ok(()),
            "an empty population judges nothing"
        );
        let failing: [BrokenGate; 4] = [
            ("records keyed to players", |counts| {
                counts.records_keyed_to_players = 99
            }),
            ("players with a record", |counts| {
                counts.players_with_record = 99
            }),
            ("overall sums competitions", |counts| {
                counts.overall_sums_competitions = 79
            }),
            ("minutes in range", |counts| counts.minutes_in_range = 299),
        ];

        for (case, break_gate) in failing {
            let mut counts = passing_counts();
            break_gate(&mut counts);
            assert_eq!(
                check_season_stats(&counts, &FM27_GATE_BOUNDS, FULL_SIZE),
                Err(SaveStatsError::Corrupt),
                "{case}"
            );
            assert_eq!(
                check_season_stats(&counts, &FM27_GATE_BOUNDS, FULL_SIZE - 1),
                Ok(()),
                "{case} below the full-save size"
            );
        }
    }

    #[test]
    fn season_counts_judge_overall_lines_minutes_and_keys() {
        let players = [
            PlayerRecord {
                pindex: 10,
                unique_id: Some(1_500),
                goalkeeper: false,
            },
            PlayerRecord {
                pindex: 20,
                unique_id: None,
                goalkeeper: false,
            },
        ];
        let by_pindex: HashMap<u32, &PlayerRecord> = players
            .iter()
            .map(|player| (player.pindex, player))
            .collect();
        // The overall line adds the league and cup lines up; the second record's does not,
        // and its league line holds more minutes than one appearance can.
        let summed = ExampleRecord::with_lines(
            11,
            &[
                (
                    LEAGUE,
                    stat_line(&[("minutes", 90.0), ("starts", 1.0), ("goals", 1.0)]),
                ),
                (
                    CUP,
                    stat_line(&[("minutes", 20.0), ("substitute_appearances", 1.0)]),
                ),
                (
                    OVERALL,
                    stat_line(&[
                        ("minutes", 110.0),
                        ("starts", 1.0),
                        ("substitute_appearances", 1.0),
                        ("goals", 1.0),
                    ]),
                ),
            ],
        );
        let not_summed = ExampleRecord::with_lines(
            41,
            &[
                (LEAGUE, stat_line(&[("minutes", 200.0), ("starts", 1.0)])),
                (OVERALL, stat_line(&[("minutes", 90.0), ("starts", 1.0)])),
            ],
        );
        let body = season_stats_body(&[summed, not_summed], 15);
        let layout = &super::super::layouts::FM27_SEASON_STATS;
        let records = walk(&body, layout).expect("walk");

        let counts = season_counts(&body, &records, &by_pindex, layout).expect("counts");

        assert_eq!(
            counts,
            SeasonCounts {
                records: 2,
                records_keyed_to_players: 1,
                players: 2,
                players_with_record: 1,
                records_with_overall: 2,
                overall_sums_competitions: 1,
                lines: 5,
                minutes_in_range: 4,
            }
        );
    }
}
