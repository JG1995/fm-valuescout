# Task 10 — Edge Case Tests

## Overview

Test the parser against deliberately malformed CSVs: missing optional columns, bad rows, duplicate UIDs, empty data, non-CSV files, different delimiters, BOM-prefixed files, and various boundary conditions. These complement the integration test by targeting specific error paths.

## Files to Create/Modify

- Create: `src-tauri/tests/edge_case_csv_parser.rs`

## Steps

- [ ] **Step 1: Write edge case test file**

Create `src-tauri/tests/edge_case_csv_parser.rs`:

```rust
use fm_valuescout_lib::parser::parse_csv;
use std::io::Write;

fn create_temp_csv(content: &str) -> String {
    let dir = std::env::temp_dir().join("fm_valuescout_edge_tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("test_{}.csv", std::process::id()));
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    path.to_string_lossy().to_string()
}

fn create_temp_csv_bom(content: &str) -> String {
    let dir = std::env::temp_dir().join("fm_valuescout_edge_tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("test_bom_{}.csv", std::process::id()));
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(&[0xEF, 0xBB, 0xBF]).unwrap(); // UTF-8 BOM
    f.write_all(content.as_bytes()).unwrap();
    path.to_string_lossy().to_string()
}

#[test]
fn empty_csv_headers_only() {
    let csv = "Unique ID;Player;Position;Minutes\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 0);
    assert_eq!(result.skipped_rows.len(), 0);
    assert_eq!(result.total_rows, 0);
}

#[test]
fn missing_optional_columns_no_ca_pa() {
    let csv = "Unique ID;Player;Position;Minutes;Goals\n\
               100;Test Player;ST (C);900;10\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 1);
    let p = &result.players[0];
    assert!(p.ca.is_none());
    assert!(p.pa.is_none());
    assert_eq!(p.attacking.goals, Some(10.0));
}

#[test]
fn missing_required_column_rejected() {
    let csv = "Player;Age;Goals\nTest Player;25;10\n";
    let path = create_temp_csv(csv);
    let err = parse_csv(&path, "2026-01-01").unwrap_err();
    assert!(err.contains("Missing required columns"));
}

#[test]
fn non_csv_file_rejected() {
    let csv = "A;B\n1;2\n";
    let path = create_temp_csv(csv);
    let err = parse_csv(&path, "2026-01-01").unwrap_err();
    assert!(err.contains("not a valid Football Manager export"));
}

#[test]
fn missing_uid_row_skipped() {
    let csv = "Unique ID;Player;Position;Minutes\n\
               ;No UID Player;ST (C);900\n\
               200;Valid Player;D (C);1800\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 1);
    assert_eq!(result.players[0].uid, 200);
    assert_eq!(result.skipped_rows.len(), 1);
}

#[test]
fn missing_name_row_skipped() {
    let csv = "Unique ID;Player;Position;Minutes\n\
               100;;ST (C);900\n\
               200;Valid;D (C);1800\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 1);
    assert_eq!(result.skipped_rows.len(), 1);
}

#[test]
fn missing_position_row_skipped() {
    let csv = "Unique ID;Player;Position;Minutes\n\
               100;No Position;;900\n\
               200;Valid;D (C);1800\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 1);
    assert_eq!(result.skipped_rows.len(), 1);
}

#[test]
fn invalid_position_row_skipped() {
    let csv = "Unique ID;Player;Position;Minutes\n\
               100;Bad Position;XYZ;900\n\
               200;Valid;D (C);1800\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 1);
    assert!(result.skipped_rows[0].reason.contains("Invalid position"));
}

#[test]
fn duplicate_uid_second_skipped() {
    let csv = "Unique ID;Player;Position;Minutes\n\
               100;First;ST (C);900\n\
               100;Second;D (C);1800\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 1);
    assert_eq!(result.players[0].name, "First");
    assert_eq!(result.skipped_rows.len(), 1);
    assert!(result.skipped_rows[0].reason.contains("Duplicate UID"));
}

#[test]
fn zero_minutes_per90_all_none() {
    let csv = "Unique ID;Player;Position;Minutes;Goals\n\
               100;Benchwarmer;ST (C);0;0\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    let p = &result.players[0];
    assert_eq!(p.minutes, Some(0));
    assert!(p.attacking.goals_per_90.is_none());
}

#[test]
fn single_transfer_value_low_equals_high() {
    let csv = "Unique ID;Player;Position;Minutes;Transfer Value\n\
               100;Test;ST (C);900;€50M\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    let tv = &result.players[0].transfer_value;
    assert_eq!(tv.low, tv.high);
    assert_eq!(tv.low, Some(50_000_000.0));
}

#[test]
fn wage_per_month_normalized() {
    let csv = "Unique ID;Player;Position;Minutes;Wage\n\
               100;Test;ST (C);900;€100K p/m\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    let w = &result.players[0].wage;
    assert_eq!(w.denomination.as_deref(), Some("p/m"));
    let expected = 100_000.0 / 4.33;
    assert!((w.wage_per_week.unwrap() - expected).abs() < 1.0);
}

#[test]
fn bom_prefixed_csv_parses_correctly() {
    let csv = "Unique ID;Player;Position;Minutes\n\
               100;Test Player;ST (C);900\n";
    let path = create_temp_csv_bom(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 1);
    assert_eq!(result.players[0].name, "Test Player");
}

#[test]
fn comma_delimited_csv() {
    let csv = "Unique ID,Player,Position,Minutes\n\
               100,Test Player,ST (C),900\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 1);
}

#[test]
fn tab_delimited_csv() {
    let csv = "Unique ID\tPlayer\tPosition\tMinutes\n\
               100\tTest Player\tST (C)\t900\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 1);
}

#[test]
fn unrecognized_file_returns_error() {
    let path = "/nonexistent/path/to/file.csv";
    let result = parse_csv(path, "2026-01-01");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unable to read file"));
}

#[test]
fn mixed_valid_invalid_rows() {
    let csv = "Unique ID;Player;Position;Minutes;Goals\n\
               100;Valid Player;ST (C);1800;15\n\
               ;No UID;D (C);900;5\n\
               200;No Position;;900;3\n\
               300;Another Valid;M (C);2700;8\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    assert_eq!(result.players.len(), 2);
    assert_eq!(result.skipped_rows.len(), 2);
    assert_eq!(result.players[0].uid, 100);
    assert_eq!(result.players[1].uid, 300);
}

#[test]
fn negative_stat_rejected_for_normal_field() {
    // Goals cannot be negative
    let csv = "Unique ID;Player;Position;Minutes;Goals\n\
               100;Test;ST (C);900;-5\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    let p = &result.players[0];
    assert_eq!(p.attacking.goals, None); // -5 rejected
}

#[test]
fn negative_allowed_for_xgp() {
    // xGP (Expected Goals Prevented) CAN be negative
    let csv = "Unique ID;Player;Position;Minutes;xGP\n\
               100;GK;GK;900;-2.5\n";
    let path = create_temp_csv(csv);
    let result = parse_csv(&path, "2026-01-01").unwrap();
    let p = &result.players[0];
    assert_eq!(p.goalkeeping.expected_goals_prevented, Some(-2.5));
}
```

- [ ] **Step 2: Run edge case tests**

Run: `cd src-tauri && cargo test --test edge_case_csv_parser`
Expected: All 19 edge case tests PASS.

## Dependencies

- Task 08 (full parser must be wired and working)

## Success Criteria

- All 19 edge case tests pass.
- Empty CSV, missing columns, bad rows, duplicates, zero minutes, BOM, different delimiters, non-existent files, mixed valid/invalid, negative value handling all covered.
- Each test targets a specific error path or boundary condition from the design spec.

## Tests

### Tests cover every edge case from the spec's "Edge Cases and Boundary Conditions" section:
- Non-CSV file, empty CSV, different delimiters, duplicate UID, BOM prefix, special characters (via sample CSV), monetary values without K/M, single transfer value, position with no side, unrecognized footedness, unrecognized nationality, zero minutes, negative stats.
**Feasibility:** ✅ Can be tested
