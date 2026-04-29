// Integration tests for CSV parser using real sample data
// Tests the full pipeline with the actual sample CSV file (258 lines = 1 header + 257 data rows)

use fm_valuescout_lib::parse_csv;

#[test]
fn sample_csv_256_players_zero_skips() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    // Debug output
    println!("Total players: {}", result.players.len());
    println!("Skipped rows: {}", result.skipped_rows.len());
    println!("Warnings: {}", result.warnings.len());

    for skipped in &result.skipped_rows {
        println!("Skipped row {}: {}", skipped.row_number, skipped.reason);
    }

    for warning in &result.warnings {
        println!("Warning row {} field {}: {}", warning.row_number, warning.field, warning.message);
    }

    assert_eq!(result.players.len(), 256, "Should parse all 256 players");
    assert_eq!(result.skipped_rows.len(), 0, "Should have zero skipped rows");
    assert_eq!(result.warnings.len(), 1, "Should have 1 warning for 'Not for Sale' transfer value");
}
#[test]
fn sample_csv_trubin_uid_and_name() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    // Find Anatolii Trubin by UID
    let trubin = result.players.iter()
        .find(|p| p.uid == 71101334)
        .expect("Should find Trubin by UID 71101334");

    assert_eq!(trubin.name, "Anatolii Trubin");
}

#[test]
fn sample_csv_trubin_identity_fields() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let trubin = result.players.iter()
        .find(|p| p.uid == 71101334)
        .expect("Should find Trubin");

    // Nationality is a struct with name and optional code
    assert_eq!(trubin.nationality.as_ref().map(|n| n.name.as_str()), Some("Ukraine"));
    assert_eq!(trubin.second_nationality.as_ref().map(|n| n.name.as_str()), Some("Ireland"));
    assert_eq!(trubin.club, Some("Benfica".to_string()));
    assert_eq!(trubin.age, Some(24));
    assert_eq!(trubin.height, Some(199));
}

#[test]
fn sample_csv_trubin_footedness() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let trubin = result.players.iter()
        .find(|p| p.uid == 71101334)
        .expect("Should find Trubin");

    // Footedness is a struct with label and score
    // Left Foot: Reasonable (score 10), Right Foot: Very Strong (score 20)
    assert_eq!(trubin.left_foot.as_ref().map(|f| f.score), Some(2));
    assert_eq!(trubin.left_foot.as_ref().map(|f| f.label.as_str()), Some("Reasonable"));
    assert_eq!(trubin.right_foot.as_ref().map(|f| f.score), Some(5));
    assert_eq!(trubin.right_foot.as_ref().map(|f| f.label.as_str()), Some("Very Strong"));
}

#[test]
fn sample_csv_trubin_financial() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let trubin = result.players.iter()
        .find(|p| p.uid == 71101334)
        .expect("Should find Trubin");

    assert_eq!(trubin.ca, Some(153));
    assert_eq!(trubin.pa, Some(162));

    // Transfer value: €62M - €94M → check individual fields
    assert_eq!(trubin.transfer_value.low, Some(62000000.0));
    assert_eq!(trubin.transfer_value.high, Some(94000000.0));
    assert_eq!(trubin.transfer_value.currency_symbol, Some("€".to_string()));

    // Wage: €74K p/w
    assert_eq!(trubin.wage.wage_per_week, Some(74000.0));
    assert_eq!(trubin.wage.currency_symbol, Some("€".to_string()));

    // Contract expires: 30/6/2028
    assert_eq!(trubin.contract_expires, Some("2028-06-30".to_string()));
}

#[test]
fn sample_csv_trubin_playing_time() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let trubin = result.players.iter()
        .find(|p| p.uid == 71101334)
        .expect("Should find Trubin");

    // Appearances: 46 (9) → 46 started, 9 sub
    assert_eq!(trubin.appearances_started, Some(46));
    assert_eq!(trubin.appearances_sub, Some(9));

    // Minutes: 4470
    assert_eq!(trubin.minutes, Some(4470));
}

#[test]
fn sample_csv_trubin_goalkeeping_stats() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let trubin = result.players.iter()
        .find(|p| p.uid == 71101334)
        .expect("Should find Trubin");

    assert_eq!(trubin.goalkeeping.clean_sheets, Some(21.0));
    assert_eq!(trubin.goalkeeping.goals_conceded, Some(39.0));
    assert_eq!(trubin.goalkeeping.saves_per_90, Some(2.0));
    assert_eq!(trubin.goalkeeping.expected_save_pct, Some(87.0));
    assert_eq!(trubin.goalkeeping.expected_goals_prevented, Some(14.63));
    assert_eq!(trubin.goalkeeping.saves_held, Some(27.0));
    assert_eq!(trubin.goalkeeping.saves_parried, Some(62.0));
    assert_eq!(trubin.goalkeeping.saves_tipped, Some(21.0));
    assert_eq!(trubin.goalkeeping.penalties_faced, Some(6.0));
    assert_eq!(trubin.goalkeeping.penalties_saved, Some(4.0));
}

#[test]
fn sample_csv_woltemade_positions() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let woltemade = result.players.iter()
        .find(|p| p.uid == 91187791)
        .expect("Should find Woltemade");

    // Position: AM (C), ST (C) → two positions
    assert_eq!(woltemade.positions.len(), 2);

    // First position: AM (C)
    assert_eq!(woltemade.positions[0].role, fm_valuescout_lib::parser::types::Role::AM);
    assert_eq!(woltemade.positions[0].sides, vec![fm_valuescout_lib::parser::types::Side::C]);

    // Second position: ST (C)
    assert_eq!(woltemade.positions[1].role, fm_valuescout_lib::parser::types::Role::ST);
    assert_eq!(woltemade.positions[1].sides, vec![fm_valuescout_lib::parser::types::Side::C]);
}

#[test]
fn sample_csv_woltemade_stats() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let woltemade = result.players.iter()
        .find(|p| p.uid == 91187791)
        .expect("Should find Woltemade");

    assert_eq!(woltemade.attacking.goals, Some(23.0));
    assert_eq!(woltemade.attacking.xg, Some(20.48));
    assert_eq!(woltemade.attacking.np_xg, Some(19.67));
    assert_eq!(woltemade.attacking.xg_overperformance, Some(2.52));
    assert_eq!(woltemade.attacking.shots, Some(162.0));
    assert_eq!(woltemade.attacking.penalties_scored, Some(3.0));

    assert_eq!(woltemade.chance_creation.assists, Some(7.0));
    assert_eq!(woltemade.chance_creation.xa, Some(8.06));
    assert_eq!(woltemade.chance_creation.clear_cut_chances, Some(6.0));
    assert_eq!(woltemade.chance_creation.key_passes, Some(67.0));

    assert_eq!(woltemade.movement.distance_km, Some(415.8));
}

#[test]
fn sample_csv_woltemade_per_90() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let woltemade = result.players.iter()
        .find(|p| p.uid == 91187791)
        .expect("Should find Woltemade");

    // Woltemade: 3674 minutes = 40.82 * 90
    // Goals: 23 → 23 / 40.82 = 0.5633...
    // xG: 20.48 → 20.48 / 40.82 = 0.5017...
    // xG-OP: 2.52 → 2.52 / 40.82 = 0.0617...
    // Shots: 162 → 162 / 40.82 = 3.968...
    // Pens S: 4 → 4 / 40.82 = 0.0979...
    // Assists: 4 → 4 / 40.82 = 0.0979...
    // xA: 11 → 11 / 40.82 = 0.2694...
    // CCC: 4 → 4 / 40.82 = 0.0979...
    // Key: 320 → 320 / 40.82 = 7.839...

    // Per-90 stats are computed and stored in the same structs
    assert!(woltemade.attacking.goals_per_90.unwrap() > 0.56 && woltemade.attacking.goals_per_90.unwrap() < 0.57,
        "Goals per 90 should be ~0.563, got {}", woltemade.attacking.goals_per_90.unwrap());
    assert!(woltemade.attacking.xg_per_90.unwrap() > 0.50 && woltemade.attacking.xg_per_90.unwrap() < 0.51,
        "xG per 90 should be ~0.502, got {}", woltemade.attacking.xg_per_90.unwrap());
    assert!(woltemade.attacking.np_xg_per_90.unwrap() > 0.48 && woltemade.attacking.np_xg_per_90.unwrap() < 0.49,
        "NP-xG per 90 should be ~0.482, got {}", woltemade.attacking.np_xg_per_90.unwrap());

    assert!(woltemade.chance_creation.assists_per_90.unwrap() > 0.17 && woltemade.chance_creation.assists_per_90.unwrap() < 0.18,
        "Assists per 90 should be ~0.171, got {}", woltemade.chance_creation.assists_per_90.unwrap());
}

#[test]
fn sample_csv_mamardashvili_negative_xgp() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let mamardashvili = result.players.iter()
        .find(|p| p.uid == 59138294)
        .expect("Should find Mamardashvili");

    // xGP should be negative: -2.94
    assert_eq!(mamardashvili.goalkeeping.expected_goals_prevented, Some(-2.94));
}

#[test]
fn sample_csv_donnarumma_ability() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let donnarumma = result.players.iter()
        .find(|p| p.uid == 43252073)
        .expect("Should find Donnarumma");

    assert_eq!(donnarumma.ca, Some(170));
    assert_eq!(donnarumma.pa, Some(170));
}

#[test]
fn sample_csv_donnarumma_appearances_no_sub() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let donnarumma = result.players.iter()
        .find(|p| p.uid == 43252073)
        .expect("Should find Donnarumma");

    // Appearances: 51 (no sub notation) → 51 started, 0 sub
    assert_eq!(donnarumma.appearances_started, Some(51));
    assert_eq!(donnarumma.appearances_sub, Some(0));
}

#[test]
fn sample_csv_all_uids_unique() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    let mut uids = std::collections::HashSet::new();
    for player in &result.players {
        assert!(!uids.contains(&player.uid), "Duplicate UID found: {}", player.uid);
        uids.insert(player.uid);
    }

    assert_eq!(uids.len(), 256, "Should have 256 unique UIDs");
}

#[test]
fn sample_csv_one_warning_for_not_for_sale() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    assert_eq!(result.warnings.len(), 1, "Should have 1 warning for 'Not for Sale' transfer value");
}

#[test]
fn sample_csv_distance_parsed() {
    let result = parse_csv("../docs/notes/test-files/Test_CSV_2026_04_29.csv")
        .expect("Parsing should succeed");

    // Trubin has 312.7km, Woltemade has 415.8km
    let trubin = result.players.iter()
        .find(|p| p.uid == 71101334)
        .expect("Should find Trubin");

    let woltemade = result.players.iter()
        .find(|p| p.uid == 91187791)
        .expect("Should find Woltemade");

    // Distance should be parsed as f64 with "km" suffix stripped
    assert_eq!(trubin.movement.distance_km, Some(312.7));
    assert_eq!(woltemade.movement.distance_km, Some(415.8));
}
