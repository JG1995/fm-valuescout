pub mod types;
pub mod countries;
pub mod positions;
pub mod headers;
pub mod fields;
pub mod metrics;

use std::collections::{HashMap, HashSet};
use std::fs;

use types::{ParseResult, ParsedPlayer, ParseWarning, SkippedRow};
use headers::{parse_headers, get_column_index, has_column};
use fields::*;
use metrics::compute_metrics;

/// Column definition for stat column lookup.
struct ColumnDef {
    csv_name: &'static str,
    allow_negative: bool,
}

/// Adding a new stat requires one line here + the field in the stat struct in types.rs.
macro_rules! define_stats {
    ($($field:literal => { csv: $csv:literal, neg: $neg:expr, access: $($access:ident).+ }),* $(,)?) => {
        const STAT_COLUMNS: &[(&str, ColumnDef)] = &[
            $(
                ($field, ColumnDef { csv_name: $csv, allow_negative: $neg }),
            )*
        ];

        fn assign_stat(player: &mut ParsedPlayer, field_name: &str, value: Option<f64>) {
            match field_name {
                $(
                    $field => player.$($access).+ = value,
                )*
                _ => {} // Unknown stat name, skip
            }
        }
    };
}

define_stats! {
    // Attacking
    "goals" => { csv: "Goals", neg: false, access: attacking.goals },
    "goals_from_outside_box" => { csv: "Goals From Outside The Box", neg: false, access: attacking.goals_from_outside_box },
    "xg" => { csv: "xG", neg: false, access: attacking.xg },
    "np_xg" => { csv: "NP-xG", neg: false, access: attacking.np_xg },
    "xg_overperformance" => { csv: "xG-OP", neg: true, access: attacking.xg_overperformance },
    "xg_per_shot" => { csv: "xG/shot", neg: false, access: attacking.xg_per_shot },
    "shots" => { csv: "Shots", neg: false, access: attacking.shots },
    "shots_from_outside_box_per_90" => { csv: "Shots From Outside The Box Per 90 minutes", neg: false, access: attacking.shots_from_outside_box_per_90 },
    "shots_on_target" => { csv: "ShT", neg: false, access: attacking.shots_on_target },
    "penalties_taken" => { csv: "Pens", neg: false, access: attacking.penalties_taken },
    "penalties_scored" => { csv: "Pens S", neg: false, access: attacking.penalties_scored },
    "free_kick_shots" => { csv: "Free Kick Shots", neg: false, access: attacking.free_kick_shots },
    // Chance creation
    "assists" => { csv: "Assists", neg: false, access: chance_creation.assists },
    "xa" => { csv: "xA", neg: false, access: chance_creation.xa },
    "chances_created_per_90" => { csv: "Ch C/90", neg: false, access: chance_creation.chances_created_per_90 },
    "clear_cut_chances" => { csv: "CCC", neg: false, access: chance_creation.clear_cut_chances },
    "key_passes" => { csv: "Key", neg: false, access: chance_creation.key_passes },
    "open_play_key_passes_per_90" => { csv: "OP-KP/90", neg: false, access: chance_creation.open_play_key_passes_per_90 },
    "crosses_attempted" => { csv: "Cr A", neg: false, access: chance_creation.crosses_attempted },
    "crosses_completed" => { csv: "Cr C", neg: false, access: chance_creation.crosses_completed },
    "open_play_crosses_attempted" => { csv: "OP-Crs A", neg: false, access: chance_creation.open_play_crosses_attempted },
    "open_play_crosses_completed" => { csv: "OP-Crs C", neg: false, access: chance_creation.open_play_crosses_completed },
    "passes_attempted" => { csv: "Pas A", neg: false, access: chance_creation.passes_attempted },
    "passes_completed" => { csv: "Pas C", neg: false, access: chance_creation.passes_completed },
    "progressive_passes" => { csv: "PsP", neg: false, access: chance_creation.progressive_passes },
    // Movement
    "dribbles" => { csv: "Drb", neg: false, access: movement.dribbles },
    "distance_km" => { csv: "Distance", neg: false, access: movement.distance_km },
    "sprints_per_90" => { csv: "Sprints/90", neg: false, access: movement.sprints_per_90 },
    "possession_lost_per_90" => { csv: "Poss Lost/90", neg: false, access: movement.possession_lost_per_90 },
    // Defending
    "tackles_attempted" => { csv: "Tck A", neg: false, access: defending.tackles_attempted },
    "tackles_completed" => { csv: "Tck C", neg: false, access: defending.tackles_completed },
    "key_tackles" => { csv: "K Tck", neg: false, access: defending.key_tackles },
    "interceptions" => { csv: "Itc", neg: false, access: defending.interceptions },
    "possession_won_per_90" => { csv: "Poss Won/90", neg: false, access: defending.possession_won_per_90 },
    "pressures_attempted" => { csv: "Pres A", neg: false, access: defending.pressures_attempted },
    "pressures_completed" => { csv: "Pres C", neg: false, access: defending.pressures_completed },
    "blocks" => { csv: "Blk", neg: false, access: defending.blocks },
    "shots_blocked" => { csv: "Shts Blckd", neg: false, access: defending.shots_blocked },
    "clearances" => { csv: "Clearances", neg: false, access: defending.clearances },
    // Aerial
    "aerial_challenges_attempted" => { csv: "Hdrs A", neg: false, access: aerial.aerial_challenges_attempted },
    "aerial_challenges_won" => { csv: "Hdrs", neg: false, access: aerial.aerial_challenges_won },
    "aerial_challenges_lost_per_90" => { csv: "Hdrs L/90", neg: false, access: aerial.aerial_challenges_lost_per_90 },
    "key_headers_per_90" => { csv: "K Hdrs/90", neg: false, access: aerial.key_headers_per_90 },
    // Goalkeeping
    "clean_sheets" => { csv: "Clean Sheets", neg: false, access: goalkeeping.clean_sheets },
    "goals_conceded" => { csv: "Goals Conceded", neg: false, access: goalkeeping.goals_conceded },
    "saves_per_90" => { csv: "Saves/90", neg: false, access: goalkeeping.saves_per_90 },
    "expected_save_pct" => { csv: "xSv %", neg: false, access: goalkeeping.expected_save_pct },
    "expected_goals_prevented" => { csv: "xGP", neg: true, access: goalkeeping.expected_goals_prevented },
    "saves_held" => { csv: "Svh", neg: false, access: goalkeeping.saves_held },
    "saves_parried" => { csv: "Svp", neg: false, access: goalkeeping.saves_parried },
    "saves_tipped" => { csv: "Svt", neg: false, access: goalkeeping.saves_tipped },
    "penalties_faced" => { csv: "Pens Faced", neg: false, access: goalkeeping.penalties_faced },
    "penalties_saved" => { csv: "Pens Saved", neg: false, access: goalkeeping.penalties_saved },
    // Discipline
    "fouls_made" => { csv: "Fouls Made", neg: false, access: discipline.fouls_made },
    "fouls_against" => { csv: "Fouls Against", neg: false, access: discipline.fouls_against },
    "yellow_cards" => { csv: "Yel", neg: false, access: discipline.yellow_cards },
    "red_cards" => { csv: "Red cards", neg: false, access: discipline.red_cards },
    "offsides" => { csv: "Off", neg: false, access: discipline.offsides },
    "mistakes_leading_to_goal" => { csv: "MLG", neg: false, access: discipline.mistakes_leading_to_goal },
    // Match outcome
    "average_rating" => { csv: "Rating", neg: false, access: match_outcome.average_rating },
    "player_of_the_match" => { csv: "PoM", neg: false, access: match_outcome.player_of_the_match },
    "games_won" => { csv: "Games Won", neg: false, access: match_outcome.games_won },
    "games_drawn" => { csv: "Games Drawn", neg: false, access: match_outcome.games_drawn },
    "games_lost" => { csv: "Games Lost", neg: false, access: match_outcome.games_lost },
    "team_goals" => { csv: "Team Goals", neg: false, access: match_outcome.team_goals },
}


/// Main entry point: parse a CSV file and return structured results.
/// This is a pure function — no side effects, no database writes.
pub fn parse_csv(path: &str) -> Result<ParseResult, String> {
    // Read file
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Unable to read file. It may be open in another program.\n{}", e))?;

    // Get first line for header parsing
    let first_line = content.lines().next()
        .ok_or("File is empty".to_string())?;

    // Parse headers
    let header_map = parse_headers(first_line)?;
    if !header_map.missing_required.is_empty() {
        return Err(format!(
            "Missing required columns: {}",
            header_map.missing_required.join(", ")
        ));
    }

    // Set up CSV reader with detected delimiter
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(header_map.delimiter)
        .flexible(true)
        .from_reader(content.as_bytes());

    // Skip header row (already parsed)
    let mut records = reader.records();

    let mut players = Vec::new();
    let mut skipped_rows = Vec::new();
    let mut warnings: Vec<ParseWarning> = Vec::new();
    let mut seen_uids: HashSet<u32> = HashSet::new();
    let mut row_number: usize = 1; // 1-indexed, header is row 0

    // Pre-compute column indices
    let col_uid = get_column_index(&header_map, "Unique ID");
    let col_player = get_column_index(&header_map, "Player");
    let col_nation = get_column_index(&header_map, "Nation");
    let col_2nd_nat = get_column_index(&header_map, "2nd Nat");
    let col_club = get_column_index(&header_map, "Club");
    let col_position = get_column_index(&header_map, "Position");
    let col_age = get_column_index(&header_map, "Age");
    let col_height = get_column_index(&header_map, "Height");
    let col_left_foot = get_column_index(&header_map, "Left Foot");
    let col_right_foot = get_column_index(&header_map, "Right Foot");
    let col_ca = get_column_index(&header_map, "CA");
    let col_pa = get_column_index(&header_map, "PA");
    let col_transfer_value = get_column_index(&header_map, "Transfer Value");
    let col_wage = get_column_index(&header_map, "Wage");
    let col_expires = get_column_index(&header_map, "Expires");
    let col_appearances = get_column_index(&header_map, "Appearances");
    let col_minutes = get_column_index(&header_map, "Minutes");


    // Pre-compute stat column indices for performance
    let mut stat_col_indices = std::collections::HashMap::new();
    for &(field_name, ref col_def) in STAT_COLUMNS {
        if let Some(idx) = get_column_index(&header_map, col_def.csv_name) {
            stat_col_indices.insert(field_name, idx);
        }
    }

    while let Some(result) = records.next() {
        row_number += 1;
        let record = match result {
            Ok(r) => r,
            Err(_) => {
                skipped_rows.push(SkippedRow {
                    row_number,
                    reason: "Malformed CSV row".to_string(),
                });
                continue;
            }
        };

        // ── Hard reject: UID ───────────────────────────────────────────
        let uid = match (col_uid, parse_uid_safe(&record, col_uid)) {
            (_, Some(Ok(u))) => u,
            _ => {
                skipped_rows.push(SkippedRow {
                    row_number,
                    reason: "Missing or invalid UID".to_string(),
                });
                continue;
            }
        };

        // ── Hard reject: Duplicate UID ─────────────────────────────────
        if seen_uids.contains(&uid) {
            skipped_rows.push(SkippedRow {
                row_number,
                reason: format!("Duplicate UID {}", uid),
            });
            continue;
        }
        seen_uids.insert(uid);

        // ── Hard reject: Name ──────────────────────────────────────────
        let name = match col_player.and_then(|i| record.get(i).map(|s| s.trim().to_string())) {
            Some(n) if !n.is_empty() => n,
            _ => {
                skipped_rows.push(SkippedRow {
                    row_number,
                    reason: "Missing player name".to_string(),
                });
                continue;
            }
        };

        // ── Hard reject: Position ──────────────────────────────────────
        let positions = match col_position.and_then(|i| record.get(i)) {
            Some(pos_str) => match parse_position_field(pos_str) {
                Ok(p) => p,
                Err(reason) => {
                    skipped_rows.push(SkippedRow {
                        row_number,
                        reason: format!("Invalid position: {}", reason),
                    });
                    continue;
                }
            },
            None => {
                skipped_rows.push(SkippedRow {
                    row_number,
                    reason: "Missing position".to_string(),
                });
                continue;
            }
        };

        // Build base player
        let mut player = ParsedPlayer::empty(uid, name, positions);

        // ── Soft fields ────────────────────────────────────────────────
        player.nationality = extract_field(&record, col_nation, parse_nationality);
        player.second_nationality = extract_field(&record, col_2nd_nat, parse_second_nationality);
        player.club = extract_field(&record, col_club, parse_club);
        player.age = extract_field(&record, col_age, parse_age);
        player.height = extract_field(&record, col_height, parse_height);

        // ── Soft fields with warnings ──────────────────────────────────
        player.left_foot = extract_field_with_warning(
            &record, col_left_foot, row_number, "Left Foot", parse_footedness, &mut warnings,
        );
        player.right_foot = extract_field_with_warning(
            &record, col_right_foot, row_number, "Right Foot", parse_footedness, &mut warnings,
        );

        // ── Optional columns ───────────────────────────────────────────
        player.ca = extract_field(&record, col_ca, parse_ability);
        player.pa = extract_field(&record, col_pa, parse_ability);

        // ── Financial fields ────────────────────────────────────────────
        player.transfer_value = extract_field_with_warning(
            &record, col_transfer_value, row_number, "Transfer Value",
            parse_transfer_value, &mut warnings,
        )
        .unwrap_or_default();
        player.wage = extract_field_with_warning(
            &record, col_wage, row_number, "Wage", parse_wage, &mut warnings,
        )
        .unwrap_or_default();

        // ── Date / playing time ─────────────────────────────────────────
        player.contract_expires = extract_field(&record, col_expires, parse_date);
        let (started, sub) = extract_field(&record, col_appearances, parse_appearances_option)
            .unwrap_or((None, None));
        player.appearances_started = started;
        player.appearances_sub = sub;
        player.minutes = extract_field(&record, col_minutes, parse_minutes);


        // ── Stat fields ────────────────────────────────────────────────
        for &(field_name, ref col_def) in STAT_COLUMNS {
            if let Some(&idx) = stat_col_indices.get(field_name) {
                if let Some(raw) = record.get(idx) {
                    let is_distance = field_name == "distance_km";
                    let value = if is_distance {
                        parse_distance(raw)
                    } else {
                        parse_stat(raw, col_def.allow_negative)
                    };

                    // Assign to the correct struct field
                    assign_stat(&mut player, field_name, value);
                }
            }
        }

        // ── Compute metrics ────────────────────────────────────────────
        compute_metrics(&mut player);

        players.push(player);
    }

    // Build columns missing list
    let columns_missing: Vec<String> = STAT_COLUMNS
        .iter()
        .filter(|(_, col_def)| !has_column(&header_map, col_def.csv_name))
        .map(|(_, col_def)| col_def.csv_name.to_string())
        .collect();

    Ok(ParseResult {
        total_rows: row_number - 1,
        players,
        skipped_rows,
        warnings,
        columns_found: header_map.columns_found,
        columns_missing,
    })
}

/// Safe UID parsing that returns Option<Result<u32, String>>.
fn parse_uid_safe(record: &csv::StringRecord, index: Option<usize>) -> Option<Result<u32, String>> {
    let idx = index?;
    let raw = record.get(idx)?.trim().to_string();
    if raw.is_empty() {
        return Some(Err("Missing UID".to_string()));
    }
    match raw.parse::<u32>() {
        Ok(uid) => Some(Ok(uid)),
        Err(_) => Some(Err(format!("Invalid UID: '{}'", raw))),
    }
}


#[cfg(test)]
mod tests {
    use super::*;



    fn create_test_csv(content: &str) -> String {
        let dir = std::env::temp_dir().join("fm_valuescout_test");
        std::fs::create_dir_all(&dir).unwrap();
        let filename = format!("test_{}_{:}.csv", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());
        let path = dir.join(filename);
        std::fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    #[test]
    fn empty_csv_headers_only() {
        let csv = "Unique ID;Player;Position;Minutes\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path).unwrap();

        assert_eq!(result.players.len(), 0);
        assert_eq!(result.skipped_rows.len(), 0);
        assert_eq!(result.total_rows, 0);
    }

    #[test]
    fn single_valid_row() {
        let csv = "Unique ID;Player;Nation;Position;Minutes;Age;Goals\n12345;Test Player;ENG;ST (C);1800;25;10\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path).unwrap();
        assert_eq!(result.players.len(), 1);
        let p = &result.players[0];
        assert_eq!(p.uid, 12345);
        assert_eq!(p.name, "Test Player");
        assert_eq!(p.minutes, Some(1800));
        assert_eq!(p.attacking.goals, Some(10.0));
        assert!(p.attacking.goals_per_90.is_some());
    }

    #[test]
    fn row_missing_uid_skipped() {
        let csv = "Unique ID;Player;Position;Minutes\n;Test Player;ST (C);1800\n99999;Another;D (C);900\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path).unwrap();
        assert_eq!(result.players.len(), 1);
        assert_eq!(result.skipped_rows.len(), 1);
        assert_eq!(result.skipped_rows[0].reason, "Missing or invalid UID");
    }

    #[test]
    fn duplicate_uid_skipped() {
        let csv = "Unique ID;Player;Position;Minutes\n100;Player A;ST (C);1800\n100;Player B;D (C);900\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path).unwrap();
        assert_eq!(result.players.len(), 1);
        assert_eq!(result.skipped_rows.len(), 1);
        assert!(result.skipped_rows[0].reason.contains("Duplicate UID"));
    }

    #[test]
    fn missing_required_column_rejected() {
        let csv = "Player;Age;Goals\nTest;25;10\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required columns"));
    }

    #[test]
    fn non_csv_rejected() {
        let csv = "A;B\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path);
        assert!(result.is_err());
    }

    #[test]
    fn all_rows_fail_validation() {
        let csv = "Unique ID;Player;Position;Minutes\nabc;Player A;ST (C);1800\n;Player B;D (C);900\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path).unwrap();
        assert_eq!(result.players.len(), 0);
        assert_eq!(result.skipped_rows.len(), 2);
    }
}
