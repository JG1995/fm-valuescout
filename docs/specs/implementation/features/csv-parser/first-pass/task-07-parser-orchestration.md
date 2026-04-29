# Task 07 — Parser Orchestration

## Overview

Wire all parser modules together into a single `parse_csv` function that: reads the file, detects delimiter, strips BOM, parses headers, iterates rows extracting fields, computes metrics, deduplicates by UID, and returns a `ParseResult`. This is the core pipeline.

## Files to Create/Modify

- Modify: `src-tauri/src/parser/mod.rs` (add `parse_csv` function and re-exports)

## Steps

- [ ] **Step 1: Write tests and implementation**

In `src-tauri/src/parser/mod.rs`, replace the existing content with:

```rust
pub mod types;
pub mod countries;
pub mod positions;
pub mod headers;
pub mod fields;
pub mod metrics;

use std::collections::{HashMap, HashSet};
use std::fs;

use types::{ParseResult, ParsedPlayer, ParseWarning, SkippedRow, ColumnStatus};
use headers::{parse_headers, get_column_index, has_column, HeaderMap};
use fields::*;
use metrics::compute_metrics;

/// Column name constants for all stat fields.
/// Maps internal field name → CSV header name (for lookup).
struct ColumnDef {
    csv_name: &'static str,
    allow_negative: bool,
}

// ── Stat column definitions ────────────────────────────────────────────
// Each stat has a CSV header name and whether negative values are allowed.

const STAT_COLUMNS: &[(&str, ColumnDef)] = &[
    // Attacking
    ("goals", ColumnDef { csv_name: "Goals", allow_negative: false }),
    ("goals_from_outside_box", ColumnDef { csv_name: "Goals From Outside The Box", allow_negative: false }),
    ("xg", ColumnDef { csv_name: "xG", allow_negative: false }),
    ("np_xg", ColumnDef { csv_name: "NP-xG", allow_negative: false }),
    ("xg_overperformance", ColumnDef { csv_name: "xG-OP", allow_negative: true }),
    ("xg_per_shot", ColumnDef { csv_name: "xG/shot", allow_negative: false }),
    ("shots", ColumnDef { csv_name: "Shots", allow_negative: false }),
    ("shots_from_outside_box_per_90", ColumnDef { csv_name: "Shots From Outside The Box Per 90 minutes", allow_negative: false }),
    ("shots_on_target", ColumnDef { csv_name: "ShT", allow_negative: false }),
    ("penalties_taken", ColumnDef { csv_name: "Pens", allow_negative: false }),
    ("penalties_scored", ColumnDef { csv_name: "Pens S", allow_negative: false }),
    ("free_kick_shots", ColumnDef { csv_name: "Free Kick Shots", allow_negative: false }),
    // Chance creation
    ("assists", ColumnDef { csv_name: "Assists", allow_negative: false }),
    ("xa", ColumnDef { csv_name: "xA", allow_negative: false }),
    ("chances_created_per_90", ColumnDef { csv_name: "Ch C/90", allow_negative: false }),
    ("clear_cut_chances", ColumnDef { csv_name: "CCC", allow_negative: false }),
    ("key_passes", ColumnDef { csv_name: "Key", allow_negative: false }),
    ("open_play_key_passes_per_90", ColumnDef { csv_name: "OP-KP/90", allow_negative: false }),
    ("crosses_attempted", ColumnDef { csv_name: "Cr A", allow_negative: false }),
    ("crosses_completed", ColumnDef { csv_name: "Cr C", allow_negative: false }),
    ("open_play_crosses_attempted", ColumnDef { csv_name: "OP-Crs A", allow_negative: false }),
    ("open_play_crosses_completed", ColumnDef { csv_name: "OP-Crs C", allow_negative: false }),
    ("passes_attempted", ColumnDef { csv_name: "Pas A", allow_negative: false }),
    ("passes_completed", ColumnDef { csv_name: "Pas C", allow_negative: false }),
    ("progressive_passes", ColumnDef { csv_name: "PsP", allow_negative: false }),
    // Movement
    ("dribbles", ColumnDef { csv_name: "Drb", allow_negative: false }),
    ("distance_km", ColumnDef { csv_name: "Distance", allow_negative: false }), // has "km" suffix
    ("sprints_per_90", ColumnDef { csv_name: "Sprints/90", allow_negative: false }),
    ("possession_lost_per_90", ColumnDef { csv_name: "Poss Lost/90", allow_negative: false }),
    // Defending
    ("tackles_attempted", ColumnDef { csv_name: "Tck A", allow_negative: false }),
    ("tackles_completed", ColumnDef { csv_name: "Tck C", allow_negative: false }),
    ("key_tackles", ColumnDef { csv_name: "K Tck", allow_negative: false }),
    ("interceptions", ColumnDef { csv_name: "Itc", allow_negative: false }),
    ("possession_won_per_90", ColumnDef { csv_name: "Poss Won/90", allow_negative: false }),
    ("pressures_attempted", ColumnDef { csv_name: "Pres A", allow_negative: false }),
    ("pressures_completed", ColumnDef { csv_name: "Pres C", allow_negative: false }),
    ("blocks", ColumnDef { csv_name: "Blk", allow_negative: false }),
    ("shots_blocked", ColumnDef { csv_name: "Shts Blckd", allow_negative: false }),
    ("clearances", ColumnDef { csv_name: "Clearances", allow_negative: false }),
    // Aerial
    ("aerial_challenges_attempted", ColumnDef { csv_name: "Hdrs A", allow_negative: false }),
    ("aerial_challenges_won", ColumnDef { csv_name: "Hdrs", allow_negative: false }),
    ("aerial_challenges_lost_per_90", ColumnDef { csv_name: "Hdrs L/90", allow_negative: false }),
    ("key_headers_per_90", ColumnDef { csv_name: "K Hdrs/90", allow_negative: false }),
    // Goalkeeping
    ("clean_sheets", ColumnDef { csv_name: "Clean Sheets", allow_negative: false }),
    ("goals_conceded", ColumnDef { csv_name: "Goals Conceded", allow_negative: false }),
    ("saves_per_90", ColumnDef { csv_name: "Saves/90", allow_negative: false }),
    ("expected_save_pct", ColumnDef { csv_name: "xSv %", allow_negative: false }),
    ("expected_goals_prevented", ColumnDef { csv_name: "xGP", allow_negative: true }),
    ("saves_held", ColumnDef { csv_name: "Svh", allow_negative: false }),
    ("saves_parried", ColumnDef { csv_name: "Svp", allow_negative: false }),
    ("saves_tipped", ColumnDef { csv_name: "Svt", allow_negative: false }),
    ("penalties_faced", ColumnDef { csv_name: "Pens Faced", allow_negative: false }),
    ("penalties_saved", ColumnDef { csv_name: "Pens Saved", allow_negative: false }),
    // Discipline
    ("fouls_made", ColumnDef { csv_name: "Fouls Made", allow_negative: false }),
    ("fouls_against", ColumnDef { csv_name: "Fouls Against", allow_negative: false }),
    ("yellow_cards", ColumnDef { csv_name: "Yel", allow_negative: false }),
    ("red_cards", ColumnDef { csv_name: "Red cards", allow_negative: false }),
    ("offsides", ColumnDef { csv_name: "Off", allow_negative: false }),
    ("mistakes_leading_to_goal", ColumnDef { csv_name: "MLG", allow_negative: false }),
    // Match outcome
    ("average_rating", ColumnDef { csv_name: "Rating", allow_negative: false }),
    ("player_of_the_match", ColumnDef { csv_name: "PoM", allow_negative: false }),
    ("games_won", ColumnDef { csv_name: "Games Won", allow_negative: false }),
    ("games_drawn", ColumnDef { csv_name: "Games Drawn", allow_negative: false }),
    ("games_lost", ColumnDef { csv_name: "Games Lost", allow_negative: false }),
    ("team_goals", ColumnDef { csv_name: "Team Goals", allow_negative: false }),
];

/// Main entry point: parse a CSV file and return structured results.
/// This is a pure function — no side effects, no database writes.
pub fn parse_csv(path: &str, in_game_date: &str) -> Result<ParseResult, String> {
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

        // ── Soft fields: nationality ────────────────────────────────────
        if let Some(idx) = col_nation {
            if let Some(raw) = record.get(idx) {
                player.nationality = parse_nationality(raw);
            }
        }

        if let Some(idx) = col_2nd_nat {
            if let Some(raw) = record.get(idx) {
                player.second_nationality = parse_second_nationality(raw);
            }
        }

        // ── Soft fields: club, age, height ─────────────────────────────
        if let Some(idx) = col_club {
            if let Some(raw) = record.get(idx) {
                player.club = parse_club(raw);
            }
        }
        if let Some(idx) = col_age {
            if let Some(raw) = record.get(idx) {
                player.age = parse_age(raw);
            }
        }
        if let Some(idx) = col_height {
            if let Some(raw) = record.get(idx) {
                player.height = parse_height(raw);
            }
        }

        // ── Soft fields: footedness ────────────────────────────────────
        if let Some(idx) = col_left_foot {
            if let Some(raw) = record.get(idx) {
                let (f, w) = parse_footedness(raw);
                player.left_foot = Some(f);
                if let Some(msg) = w {
                    warnings.push(ParseWarning {
                        row_number,
                        field: "Left Foot".to_string(),
                        message: msg,
                    });
                }
            }
        }
        if let Some(idx) = col_right_foot {
            if let Some(raw) = record.get(idx) {
                let (f, w) = parse_footedness(raw);
                player.right_foot = Some(f);
                if let Some(msg) = w {
                    warnings.push(ParseWarning {
                        row_number,
                        field: "Right Foot".to_string(),
                        message: msg,
                    });
                }
            }
        }

        // ── Optional columns: CA, PA ───────────────────────────────────
        if let Some(idx) = col_ca {
            if let Some(raw) = record.get(idx) {
                player.ca = parse_ability(raw);
            }
        }
        if let Some(idx) = col_pa {
            if let Some(raw) = record.get(idx) {
                player.pa = parse_ability(raw);
            }
        }

        // ── Financial fields ───────────────────────────────────────────
        if let Some(idx) = col_transfer_value {
            if let Some(raw) = record.get(idx) {
                let (tv, w) = parse_transfer_value(raw);
                player.transfer_value = tv;
                if let Some(msg) = w {
                    warnings.push(ParseWarning {
                        row_number,
                        field: "Transfer Value".to_string(),
                        message: msg,
                    });
                }
            }
        }
        if let Some(idx) = col_wage {
            if let Some(raw) = record.get(idx) {
                let (wage, w) = parse_wage(raw);
                player.wage = wage;
                if let Some(msg) = w {
                    warnings.push(ParseWarning {
                        row_number,
                        field: "Wage".to_string(),
                        message: msg,
                    });
                }
            }
        }

        // ── Date / playing time ─────────────────────────────────────────
        if let Some(idx) = col_expires {
            if let Some(raw) = record.get(idx) {
                player.contract_expires = parse_date(raw);
            }
        }
        if let Some(idx) = col_appearances {
            if let Some(raw) = record.get(idx) {
                let (started, sub) = parse_appearances(raw);
                player.appearances_started = started;
                player.appearances_sub = sub;
            }
        }
        if let Some(idx) = col_minutes {
            if let Some(raw) = record.get(idx) {
                player.minutes = parse_minutes(raw);
            }
        }

        // ── Stat fields ────────────────────────────────────────────────
        for &(field_name, ref col_def) in STAT_COLUMNS {
            if let Some(idx) = get_column_index(&header_map, col_def.csv_name) {
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

/// Assign a stat value to the correct field on a ParsedPlayer.
fn assign_stat(player: &mut ParsedPlayer, field_name: &str, value: Option<f64>) {
    match field_name {
        "goals" => player.attacking.goals = value,
        "goals_from_outside_box" => player.attacking.goals_from_outside_box = value,
        "xg" => player.attacking.xg = value,
        "np_xg" => player.attacking.np_xg = value,
        "xg_overperformance" => player.attacking.xg_overperformance = value,
        "xg_per_shot" => player.attacking.xg_per_shot = value,
        "shots" => player.attacking.shots = value,
        "shots_from_outside_box_per_90" => player.attacking.shots_from_outside_box_per_90 = value,
        "shots_on_target" => player.attacking.shots_on_target = value,
        "penalties_taken" => player.attacking.penalties_taken = value,
        "penalties_scored" => player.attacking.penalties_scored = value,
        "free_kick_shots" => player.attacking.free_kick_shots = value,
        "assists" => player.chance_creation.assists = value,
        "xa" => player.chance_creation.xa = value,
        "chances_created_per_90" => player.chance_creation.chances_created_per_90 = value,
        "clear_cut_chances" => player.chance_creation.clear_cut_chances = value,
        "key_passes" => player.chance_creation.key_passes = value,
        "open_play_key_passes_per_90" => player.chance_creation.open_play_key_passes_per_90 = value,
        "crosses_attempted" => player.chance_creation.crosses_attempted = value,
        "crosses_completed" => player.chance_creation.crosses_completed = value,
        "open_play_crosses_attempted" => player.chance_creation.open_play_crosses_attempted = value,
        "open_play_crosses_completed" => player.chance_creation.open_play_crosses_completed = value,
        "passes_attempted" => player.chance_creation.passes_attempted = value,
        "passes_completed" => player.chance_creation.passes_completed = value,
        "progressive_passes" => player.chance_creation.progressive_passes = value,
        "dribbles" => player.movement.dribbles = value,
        "distance_km" => player.movement.distance_km = value,
        "sprints_per_90" => player.movement.sprints_per_90 = value,
        "possession_lost_per_90" => player.movement.possession_lost_per_90 = value,
        "tackles_attempted" => player.defending.tackles_attempted = value,
        "tackles_completed" => player.defending.tackles_completed = value,
        "key_tackles" => player.defending.key_tackles = value,
        "interceptions" => player.defending.interceptions = value,
        "possession_won_per_90" => player.defending.possession_won_per_90 = value,
        "pressures_attempted" => player.defending.pressures_attempted = value,
        "pressures_completed" => player.defending.pressures_completed = value,
        "blocks" => player.defending.blocks = value,
        "shots_blocked" => player.defending.shots_blocked = value,
        "clearances" => player.defending.clearances = value,
        "aerial_challenges_attempted" => player.aerial.aerial_challenges_attempted = value,
        "aerial_challenges_won" => player.aerial.aerial_challenges_won = value,
        "aerial_challenges_lost_per_90" => player.aerial.aerial_challenges_lost_per_90 = value,
        "key_headers_per_90" => player.aerial.key_headers_per_90 = value,
        "clean_sheets" => player.goalkeeping.clean_sheets = value,
        "goals_conceded" => player.goalkeeping.goals_conceded = value,
        "saves_per_90" => player.goalkeeping.saves_per_90 = value,
        "expected_save_pct" => player.goalkeeping.expected_save_pct = value,
        "expected_goals_prevented" => player.goalkeeping.expected_goals_prevented = value,
        "saves_held" => player.goalkeeping.saves_held = value,
        "saves_parried" => player.goalkeeping.saves_parried = value,
        "saves_tipped" => player.goalkeeping.saves_tipped = value,
        "penalties_faced" => player.goalkeeping.penalties_faced = value,
        "penalties_saved" => player.goalkeeping.penalties_saved = value,
        "fouls_made" => player.discipline.fouls_made = value,
        "fouls_against" => player.discipline.fouls_against = value,
        "yellow_cards" => player.discipline.yellow_cards = value,
        "red_cards" => player.discipline.red_cards = value,
        "offsides" => player.discipline.offsides = value,
        "mistakes_leading_to_goal" => player.discipline.mistakes_leading_to_goal = value,
        "average_rating" => player.match_outcome.average_rating = value,
        "player_of_the_match" => player.match_outcome.player_of_the_match = value,
        "games_won" => player.match_outcome.games_won = value,
        "games_drawn" => player.match_outcome.games_drawn = value,
        "games_lost" => player.match_outcome.games_lost = value,
        "team_goals" => player.match_outcome.team_goals = value,
        _ => {} // Unknown stat name, skip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_csv(content: &str) -> String {
        let dir = std::env::temp_dir().join("fm_valuescout_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.csv");
        std::fs::write(&path, content).unwrap();
        path.to_string_lossy().to_string()
    }

    #[test]
    fn empty_csv_headers_only() {
        let csv = "Unique ID;Player;Position;Minutes\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path, "2026-01-01").unwrap();
        assert_eq!(result.players.len(), 0);
        assert_eq!(result.skipped_rows.len(), 0);
        assert_eq!(result.total_rows, 0);
    }

    #[test]
    fn single_valid_row() {
        let csv = "Unique ID;Player;Nation;Position;Minutes;Age;Goals\n\
                   12345;Test Player;ENG;ST (C);1800;25;10\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path, "2026-01-01").unwrap();
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
        let csv = "Unique ID;Player;Position;Minutes\n\
                   ;Test Player;ST (C);1800\n\
                   99999;Another;D (C);900\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path, "2026-01-01").unwrap();
        assert_eq!(result.players.len(), 1);
        assert_eq!(result.skipped_rows.len(), 1);
        assert_eq!(result.skipped_rows[0].reason, "Missing or invalid UID");
    }

    #[test]
    fn duplicate_uid_skipped() {
        let csv = "Unique ID;Player;Position;Minutes\n\
                   100;Player A;ST (C);1800\n\
                   100;Player B;D (C);900\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path, "2026-01-01").unwrap();
        assert_eq!(result.players.len(), 1);
        assert_eq!(result.skipped_rows.len(), 1);
        assert!(result.skipped_rows[0].reason.contains("Duplicate UID"));
    }

    #[test]
    fn missing_required_column_rejected() {
        let csv = "Player;Age;Goals\nTest;25;10\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path, "2026-01-01");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required columns"));
    }

    #[test]
    fn non_csv_rejected() {
        let csv = "A;B\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path, "2026-01-01");
        assert!(result.is_err());
    }

    #[test]
    fn all_rows_fail_validation() {
        let csv = "Unique ID;Player;Position;Minutes\n\
                   abc;Player A;ST (C);1800\n\
                   ;Player B;D (C);900\n";
        let path = create_test_csv(csv);
        let result = parse_csv(&path, "2026-01-01").unwrap();
        assert_eq!(result.players.len(), 0);
        assert_eq!(result.skipped_rows.len(), 2);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd src-tauri && cargo test --lib parser`
Expected: All tests pass (including tests from submodules).

## Dependencies

- Task 05 (field parsers)
- Task 06 (computed metrics)
- Task 04 (header parser)
- Task 03 (position parser)
- Task 02 (country codes)
- Task 01 (types)

## Success Criteria

- All orchestration tests pass: empty CSV, single valid row, missing UID skip, duplicate UID skip, missing required column rejection, non-CSV rejection, all-rows-fail.
- Full pipeline works: file read → header parse → row iteration → field extraction → metric computation → result assembly.

## Tests

### Test 1: Empty CSV returns 0 players
**Feasibility:** ✅ Can be tested

### Test 2: Single valid row parsed correctly
**Feasibility:** ✅ Can be tested

### Test 3: Row with missing UID is skipped
**Feasibility:** ✅ Can be tested

### Test 4: Duplicate UID is skipped
**Feasibility:** ✅ Can be tested

### Test 5: Missing required column rejects file
**Feasibility:** ✅ Can be tested

### Test 6: Non-CSV file rejected
**Feasibility:** ✅ Can be tested

### Test 7: All rows failing produces 0 players with skips
**Feasibility:** ✅ Can be tested
