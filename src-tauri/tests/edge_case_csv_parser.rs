// Edge case tests for CSV parser
// Tests error handling, boundary conditions, and unusual but valid input

use fm_valuescout_lib::parse_csv;
use std::fs;
use std::io::Write;

// ── Test Helper ─────────────────────────────────────────────────────────

fn create_test_csv(content: &str) -> String {
    let dir = std::env::temp_dir().join("fm_edge_test");
    fs::create_dir_all(&dir).unwrap();
    let filename = format!(
        "edge_test_{}_{}.csv",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = dir.join(filename);
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path.to_string_lossy().to_string()
}

// ── File Format Tests ────────────────────────────────────────────────────

#[test]
fn empty_csv_headers_only() {
    let csv = "Unique ID;Player;Position;Minutes\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Parsing should succeed");

    assert_eq!(result.players.len(), 0, "Should have zero players");
    assert_eq!(result.skipped_rows.len(), 0, "Should have zero skipped rows");
    assert_eq!(result.total_rows, 0, "Total rows should be 0");
}

#[test]
fn non_csv_file_rejected() {
    let csv = "Player;Age\nTest;25\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path);

    assert!(result.is_err(), "Should return error for non-CSV file");
    let err = result.unwrap_err();
    assert!(err.contains("File is not a valid Football Manager export"), "Error should mention invalid FM export");
}

#[test]
fn bom_prefixed_csv_parses_correctly() {
    // UTF-8 BOM (EF BB BF) followed by semicolon-delimited CSV
    let mut csv = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    csv.extend_from_slice(b"Unique ID;Player;Position;Minutes\n12345;Test Player;ST (C);1800\n");

    let path = create_test_csv(&String::from_utf8_lossy(&csv));
    let result = parse_csv(&path).expect("BOM-prefixed CSV should parse successfully");

    assert_eq!(result.players.len(), 1, "Should parse 1 player");
    assert_eq!(result.players[0].uid, 12345);
    assert_eq!(result.players[0].name, "Test Player");
    assert_eq!(result.players[0].minutes, Some(1800));
}

#[test]
fn comma_delimited_csv() {
    // Parser should auto-detect comma delimiter
    let csv = "Unique ID,Player,Position,Minutes\n12345,Test Player,ST (C),1800\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Comma-delimited CSV should parse");

    assert_eq!(result.players.len(), 1);
    assert_eq!(result.players[0].uid, 12345);
}

#[test]
fn tab_delimited_csv() {
    // Parser should auto-detect tab delimiter
    let csv = "Unique ID\tPlayer\tPosition\tMinutes\n12345\tTest Player\tST (C)\t1800\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Tab-delimited CSV should parse");

    assert_eq!(result.players.len(), 1);
    assert_eq!(result.players[0].uid, 12345);
}

#[test]
fn unrecognized_file_returns_error() {
    let result = parse_csv("/nonexistent/path/to/file.csv");

    assert!(result.is_err(), "Should return error for non-existent file");
    let err = result.unwrap_err();
    assert!(err.contains("Unable to read file"), "Error should mention file read failure");
}

// ── Column/Header Tests ──────────────────────────────────────────────────

#[test]
fn missing_optional_columns_no_ca_pa() {
    // CSV without CA and PA columns should still parse
    let csv = "Unique ID;Player;Position;Minutes;Goals\n12345;Test Player;ST (C);1800;10\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse without optional CA/PA columns");

    assert_eq!(result.players.len(), 1);
    assert_eq!(result.players[0].uid, 12345);
    assert_eq!(result.players[0].ca, None, "CA should be None when column missing");
    assert_eq!(result.players[0].pa, None, "PA should be None when column missing");
    assert_eq!(result.players[0].attacking.goals, Some(10.0));
}

#[test]
fn missing_required_column_rejected() {
    // Missing "Position" column (required)
    let csv = "Unique ID;Player;Minutes\n12345;Test Player;1800\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path);

    assert!(result.is_err(), "Should reject CSV missing required Position column");
    let err = result.unwrap_err();
    assert!(err.contains("Missing required columns"), "Error should mention missing required columns");
}

// ── Row Validation Tests (Hard Rejects) ───────────────────────────────────

#[test]
fn missing_uid_row_skipped() {
    let csv = "Unique ID;Player;Position;Minutes\n;Test Player;ST (C);1800\n99999;Another;D (C);900\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert_eq!(result.players.len(), 1, "Should parse only 1 valid player");
    assert_eq!(result.skipped_rows.len(), 1, "Should skip 1 row");
    assert!(result.skipped_rows[0].reason.contains("Missing or invalid UID"),
            "Skip reason should mention missing/invalid UID");
}

#[test]
fn missing_name_row_skipped() {
    let csv = "Unique ID;Player;Position;Minutes\n12345;;ST (C);1800\n99999;Another;D (C);900\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert_eq!(result.players.len(), 1, "Should parse only 1 valid player");
    assert_eq!(result.skipped_rows.len(), 1, "Should skip 1 row");
    assert!(result.skipped_rows[0].reason.contains("Missing player name"),
            "Skip reason should mention missing name");
}

#[test]
fn missing_position_row_skipped() {
    let csv = "Unique ID;Player;Position;Minutes\n12345;Test Player;;1800\n99999;Another;D (C);900\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert_eq!(result.players.len(), 1, "Should parse only 1 valid player");
    assert_eq!(result.skipped_rows.len(), 1, "Should skip 1 row");
    assert!(result.skipped_rows[0].reason.to_lowercase().contains("position"),
            "Skip reason should mention position");
}

#[test]
fn invalid_position_row_skipped() {
    let csv = "Unique ID;Player;Position;Minutes\n12345;Test Player;XYZ (C);1800\n99999;Another;D (C);900\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert_eq!(result.players.len(), 1, "Should parse only 1 valid player");
    assert_eq!(result.skipped_rows.len(), 1, "Should skip 1 row");
    assert!(result.skipped_rows[0].reason.contains("Invalid position"),
            "Skip reason should mention invalid position");
}

#[test]
fn duplicate_uid_second_skipped() {
    let csv = "Unique ID;Player;Position;Minutes\n100;Player A;ST (C);1800\n100;Player B;D (C);900\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert_eq!(result.players.len(), 1, "Should parse only first occurrence of UID");
    assert_eq!(result.players[0].name, "Player A", "First player should be kept");
    assert_eq!(result.skipped_rows.len(), 1, "Should skip second row");
    assert!(result.skipped_rows[0].reason.contains("Duplicate UID 100"),
            "Skip reason should mention duplicate UID");
}

#[test]
fn mixed_valid_invalid_rows() {
    // Mix of valid and invalid rows: missing UID, missing name, duplicate UID
    let csv = "Unique ID;Player;Position;Minutes\n\
        ;MissingUID;ST (C);1800\n\
        100;Player A;ST (C);1800\n\
        200;;D (C);900\n\
        100;Duplicate UID;D (C);900\n\
        300;Valid Player;M (C);1200\n";

    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert_eq!(result.players.len(), 2, "Should parse 2 valid players (UID 100 and 300)");
    assert_eq!(result.skipped_rows.len(), 3, "Should skip 3 invalid rows");

    // Verify skip reasons
    let reasons: Vec<&str> = result.skipped_rows.iter().map(|s| s.reason.as_str()).collect();
    assert!(reasons.iter().any(|r| r.contains("Missing or invalid UID")), "Should skip missing UID");
    assert!(reasons.iter().any(|r| r.contains("Missing player name")), "Should skip missing name");
    assert!(reasons.iter().any(|r| r.contains("Duplicate UID")), "Should skip duplicate UID");
}

// ── Data Validation Tests ────────────────────────────────────────────────

#[test]
fn zero_minutes_per90_all_none() {
    // When minutes = 0, all per-90 stats should be None (avoid division by zero)
    let csv = "Unique ID;Player;Position;Minutes;Goals;Shots;xA;Drb;Tck A;Itc;Hdrs;Rating\n\
        12345;Test Player;ST (C);0;10;20;5;15;30;25;10;7.5\n";

    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert_eq!(result.players.len(), 1);
    let p = &result.players[0];
    assert_eq!(p.minutes, Some(0), "Minutes should be 0");

    // All per-90 stats should be None when minutes = 0
    assert!(p.attacking.goals_per_90.is_none(), "goals_per_90 should be None");
    assert!(p.attacking.shots_per_90.is_none(), "shots_per_90 should be None");
    assert!(p.chance_creation.xa_per_90.is_none(), "xa_per_90 should be None");
    assert!(p.movement.dribbles_per_90.is_none(), "dribbles_per_90 should be None");
    assert!(p.defending.tackles_per_90.is_none(), "tackles_per_90 should be None");
    assert!(p.defending.interceptions_per_90.is_none(), "interceptions_per_90 should be None");
    assert!(p.aerial.aerial_duels_per_90.is_none(), "aerial_duels_per_90 should be None");

    // Raw stats should still be parsed
    assert_eq!(p.attacking.goals, Some(10.0), "Raw goals should be preserved");
    assert_eq!(p.attacking.shots, Some(20.0), "Raw shots should be preserved");
    assert_eq!(p.match_outcome.average_rating, Some(7.5), "Rating should be preserved");
}

#[test]
fn single_transfer_value_low_equals_high() {
    // Single transfer value (not a range) should have low == high
    let csv = "Unique ID;Player;Position;Minutes;Transfer Value\n\
        12345;Test Player;ST (C);1800;€50M\n";

    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert_eq!(result.players.len(), 1);
    let tv = &result.players[0].transfer_value;
    assert_eq!(tv.currency_symbol, Some("€".to_string()));
    assert_eq!(tv.low, Some(50000000.0), "Low should be 50M");
    assert_eq!(tv.high, Some(50000000.0), "High should equal low for single value");
    assert_eq!(tv.raw, Some("€50M".to_string()));
}

#[test]
fn wage_per_month_normalized() {
    // Test wage normalization: p/m and p/a should convert to per-week
    let csv = "Unique ID;Player;Position;Minutes;Wage\n\
        12345;Test Player;ST (C);1800;€50K p/m\n\
        12346;Test Player 2;D (C);1800;€600K p/a\n\
        12347;Test Player 3;M (C);1800;€10K p/w\n";

    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert_eq!(result.players.len(), 3);

    // p/m: divide by 4.33
    let p1 = &result.players[0];
    assert_eq!(p1.wage.denomination, Some("p/m".to_string()));
    assert!((p1.wage.wage_per_week.unwrap() - 50000.0 / 4.33).abs() < 0.01,
            "p/m should be divided by 4.33");

    // p/a: divide by 52
    let p2 = &result.players[1];
    assert_eq!(p2.wage.denomination, Some("p/a".to_string()));
    assert!((p2.wage.wage_per_week.unwrap() - 600000.0 / 52.0).abs() < 0.01,
            "p/a should be divided by 52");

    // p/w: no change
    let p3 = &result.players[2];
    assert_eq!(p3.wage.denomination, Some("p/w".to_string()));
    assert_eq!(p3.wage.wage_per_week, Some(10000.0), "p/w should be unchanged");
}

#[test]
fn negative_stat_converted_to_none_for_normal_field() {
    // Normal stats (goals, shots, etc.) convert negative values to None (not hard reject)
    let csv = "Unique ID;Player;Position;Minutes;Goals;Shots;Drb;Tck A\n12345;Test Player;ST (C);1800;-5;-10;-3;-15\n";
    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse successfully");

    assert_eq!(result.players.len(), 1);
    let p = &result.players[0];
    // Negative values are silently converted to None for normal fields
    assert!(p.attacking.goals.is_none(), "Negative goals should be None");
    assert!(p.attacking.shots.is_none(), "Negative shots should be None");
    assert!(p.movement.dribbles.is_none(), "Negative dribbles should be None");
    assert!(p.defending.tackles_attempted.is_none(), "Negative tackles should be None");
}

#[test]
fn negative_allowed_for_xgp() {
    // xG-OP and xGP fields allow negative values
    let csv = "Unique ID;Player;Position;Minutes;xG-OP;xGP\n\
        12345;Test Player;ST (C);1800;-5.5;-3.2\n";

    let path = create_test_csv(csv);
    let result = parse_csv(&path).expect("Should parse negative xG-OP and xGP");

    assert_eq!(result.players.len(), 1);
    let p = &result.players[0];
    assert_eq!(p.attacking.xg_overperformance, Some(-5.5), "xG-OP should allow negative");
    assert_eq!(p.goalkeeping.expected_goals_prevented, Some(-3.2), "xGP should allow negative");
}
