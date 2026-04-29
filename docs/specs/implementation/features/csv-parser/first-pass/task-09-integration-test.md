# Task 09 — Integration Test: Sample CSV

## Overview

Feed the real sample CSV (`docs/notes/test-files/Test_CSV_2026_04_29.csv`, 258 lines = 1 header + 257 data rows) through the full `parse_csv` pipeline. Assert correct player count, zero skipped rows, key field values match expected data from known rows, and all computed metrics are accurate.

## Files to Create/Modify

- Create: `src-tauri/tests/integration_csv_parser.rs`

## Steps

- [ ] **Step 1: Write integration test file**

Create `src-tauri/tests/integration_csv_parser.rs`:

```rust
use fm_valuescout_lib::parser::parse_csv;

/// Path to the sample CSV file in the repository.
/// This test uses the actual sample data from the repo.
fn sample_csv_path() -> String {
    // From src-tauri/, go up one level to find the test file
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    format!("{}/../../docs/notes/test-files/Test_CSV_2026_04_29.csv", manifest_dir)
}

#[test]
fn sample_csv_257_players_zero_skips() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    assert_eq!(result.players.len(), 257, "Expected 257 parsed players");
    assert_eq!(result.skipped_rows.len(), 0, "Expected zero skipped rows, got: {:?}", result.skipped_rows);
}

#[test]
fn sample_csv_trubin_uid_and_name() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let trubin = result.players.iter().find(|p| p.uid == 71101334).unwrap();
    assert_eq!(trubin.name, "Anatolii Trubin");
    assert_eq!(trubin.positions.len(), 1);
    assert_eq!(trubin.positions[0].role, fm_valuescout_lib::parser::types::Role::GK);
    assert!(trubin.positions[0].sides.is_empty());
}

#[test]
fn sample_csv_trubin_identity_fields() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let trubin = result.players.iter().find(|p| p.uid == 71101334).unwrap();
    assert_eq!(trubin.age, Some(24));
    assert_eq!(trubin.height, Some(199));
    assert_eq!(trubin.club.as_deref(), Some("Benfica"));
    assert_eq!(trubin.nationality.as_ref().unwrap().code.as_deref(), Some("UKR"));
}

#[test]
fn sample_csv_trubin_footedness() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let trubin = result.players.iter().find(|p| p.uid == 71101334).unwrap();
    assert_eq!(trubin.left_foot.as_ref().unwrap().label, "Reasonable");
    assert_eq!(trubin.left_foot.as_ref().unwrap().score, 2);
    assert_eq!(trubin.right_foot.as_ref().unwrap().label, "Very Strong");
    assert_eq!(trubin.right_foot.as_ref().unwrap().score, 5);
}

#[test]
fn sample_csv_trubin_financial() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let trubin = result.players.iter().find(|p| p.uid == 71101334).unwrap();
    assert_eq!(trubin.transfer_value.low, Some(62_000_000.0));
    assert_eq!(trubin.transfer_value.high, Some(94_000_000.0));
    assert_eq!(trubin.wage.raw_value, Some(74_000.0));
    assert_eq!(trubin.wage.wage_per_week, Some(74_000.0)); // p/w
    assert_eq!(trubin.contract_expires.as_deref(), Some("2028-06-30"));
}

#[test]
fn sample_csv_trubin_playing_time() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let trubin = result.players.iter().find(|p| p.uid == 71101334).unwrap();
    assert_eq!(trubin.appearances_started, Some(46));
    assert_eq!(trubin.appearances_sub, Some(9));
    assert_eq!(trubin.minutes, Some(4470));
}

#[test]
fn sample_csv_trubin_goalkeeping_stats() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let trubin = result.players.iter().find(|p| p.uid == 71101334).unwrap();
    assert_eq!(trubin.goalkeeping.clean_sheets, Some(21.0));
    assert_eq!(trubin.goalkeeping.goals_conceded, Some(39.0));
    assert_eq!(trubin.goalkeeping.expected_goals_prevented, Some(14.63));
}

#[test]
fn sample_csv_woltemade_positions() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let nick = result.players.iter().find(|p| p.uid == 91187791).unwrap();
    assert_eq!(nick.name, "Nick Woltemade");
    assert_eq!(nick.positions.len(), 2);
    // AM (C) and ST (C)
    assert_eq!(nick.positions[0].role, fm_valuescout_lib::parser::types::Role::AM);
    assert_eq!(nick.positions[1].role, fm_valuescout_lib::parser::types::Role::ST);
}

#[test]
fn sample_csv_woltemade_stats() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let nick = result.players.iter().find(|p| p.uid == 91187791).unwrap();
    assert_eq!(nick.attacking.goals, Some(23.0));
    assert_eq!(nick.attacking.xg, Some(20.48));
    assert_eq!(nick.chance_creation.assists, Some(7.0));
    assert_eq!(nick.minutes, Some(3674));
}

#[test]
fn sample_csv_woltemade_per_90() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let nick = result.players.iter().find(|p| p.uid == 91187791).unwrap();
    let expected_goals_per_90 = 23.0 / 3674.0 * 90.0;
    let actual = nick.attacking.goals_per_90.unwrap();
    assert!((actual - expected_goals_per_90).abs() < 0.01,
        "goals_per_90: expected {}, got {}", expected_goals_per_90, actual);
}

#[test]
fn sample_csv_mamardashvili_negative_xgp() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let giorgi = result.players.iter().find(|p| p.uid == 59138294).unwrap();
    assert_eq!(giorgi.goalkeeping.expected_goals_prevented, Some(-2.94));
}

#[test]
fn sample_csv_donnarumma_ability() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let gigi = result.players.iter().find(|p| p.uid == 43252073).unwrap();
    assert_eq!(gigi.ca, Some(170));
    assert_eq!(gigi.pa, Some(170));
}

#[test]
fn sample_csv_donnarumma_appearances_no_sub() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let gigi = result.players.iter().find(|p| p.uid == 43252073).unwrap();
    // "51" — no parens
    assert_eq!(gigi.appearances_started, Some(51));
    assert_eq!(gigi.appearances_sub, Some(0));
}

#[test]
fn sample_csv_all_uids_unique() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let uids: std::collections::HashSet<u32> = result.players.iter().map(|p| p.uid).collect();
    assert_eq!(uids.len(), 257, "All UIDs should be unique");
}

#[test]
fn sample_csv_no_warnings_for_valid_data() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    // Sample CSV is clean — warnings should be minimal (0 or near-0)
    // Some "per 90" columns may already have values in the CSV that we also compute,
    // but the main concern is no warnings on identity/financial/stat fields
    let non_trivial_warnings: Vec<_> = result.warnings.iter()
        .filter(|w| !w.field.contains("per 90")) // allow per-90 columns
        .collect();
    assert!(non_trivial_warnings.is_empty(),
        "Unexpected warnings: {:?}", non_trivial_warnings);
}

#[test]
fn sample_csv_distance_parsed() {
    let result = parse_csv(&sample_csv_path(), "2026-04-29").unwrap();
    let trubin = result.players.iter().find(|p| p.uid == 71101334).unwrap();
    assert_eq!(trubin.movement.distance_km, Some(312.7));
}
```

- [ ] **Step 2: Make lib.rs export parser for integration tests**

In `src-tauri/src/lib.rs`, ensure `parser` is `pub mod`:

```rust
pub mod commands;
pub mod parser;
pub mod storage;
```

(Note: this may already be the case from Task 08.)

- [ ] **Step 3: Run integration tests**

Run: `cd src-tauri && cargo test --test integration_csv_parser`
Expected: All 16 integration tests PASS.

## Dependencies

- Task 08 (Tauri commands and wiring must be complete)

## Success Criteria

- All 16 integration tests pass using the real sample CSV.
- 257 players parsed with zero skipped rows.
- Specific player data (Trubin, Woltemade, Mamardashvili, Donnarumma) verified against known values.
- Per-90 values match manual calculations.
- Negative xGP preserved.
- All UIDs unique within result.
- Distance fields correctly parsed (km suffix stripped).

## Tests

### Tests verify: player count, specific player identity/financial/stats, per-90 computation, negative values, appearances with/without subs, UID uniqueness, distance parsing, absence of warnings.
**Feasibility:** ✅ Can be tested — uses actual sample CSV from repo
