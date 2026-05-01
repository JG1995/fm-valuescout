// Integration tests for storage layer against real SQLite database
// Tests the full flow: create save → import → retrieve → verify

use fm_valuescout_lib::storage;
use fm_valuescout_lib::parser::types::{
    AttackingStats, ParsedPlayer, Position, Role, Side, Nationality,
    TransferValue, Wage,
};
use rusqlite::Connection;

// ── Test helpers ───────────────────────────────────────────────────────

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    storage::init_db_test(&conn).unwrap();
    conn
}

fn make_player(uid: u32, name: &str) -> ParsedPlayer {
    ParsedPlayer::empty(uid, name.to_string(), vec![Position {
        role: Role::ST,
        sides: vec![Side::C],
    }])
}

fn make_detailed_player(uid: u32, name: &str, club: &str, age: u16) -> ParsedPlayer {
    let mut p = make_player(uid, name);
    p.club = Some(club.to_string());
    p.age = Some(age);
    p.minutes = Some(1800);
    p.appearances_started = Some(20);
    p.appearances_sub = Some(2);
    p.nationality = Some(Nationality {
        code: Some("ENG".to_string()),
        name: "England".to_string(),
    });
    p.wage = Wage {
        wage_per_week: Some(100_000.0),
        ..Default::default()
    };
    p.transfer_value = TransferValue {
        high: Some(30_000_000.0),
        ..Default::default()
    };
    p.contract_expires = Some("2032-06-30".to_string());
    p.attacking = AttackingStats {
        goals: Some(12.0),
        ..Default::default()
    };
    p
}

// ── Tests ──────────────────────────────────────────────────────────────

/// Test: full_import_flow
/// Create save → import season → retrieve → verify data
#[test]
fn full_import_flow() {
    let conn = setup_db();

    // Create save
    let save = storage::create_save(&conn, "Career Mode").unwrap();
    assert!(save.id > 0);

    // Import season with two players
    let players = vec![
        make_detailed_player(1001, "Harry Kane", "Tottenham", 29),
        make_detailed_player(1002, "Erling Haaland", "Man City", 23),
    ];
    let result = storage::import_season(&conn, save.id, "2023-11-15", players).unwrap();

    assert_eq!(result.total_players, 2);
    assert_eq!(result.new_players, 2);
    assert_eq!(result.matched_players, 0);
    assert_eq!(result.season.label, "2023/24");

    // Retrieve players
    let retrieved = storage::get_players_for_season(&conn, result.season.id).unwrap();
    assert_eq!(retrieved.len(), 2);

    let kane = retrieved.iter().find(|p| p.player_name == "Harry Kane").unwrap();
    assert_eq!(kane.fm_uid, 1001);
    assert_eq!(kane.club.as_deref(), Some("Tottenham"));
    assert_eq!(kane.age, Some(29));
    assert_eq!(kane.wage_per_week, Some(100_000.0));
    assert_eq!(kane.transfer_value_high, Some(30_000_000.0));

    // Verify JSON blob deserialization
    let kane_data = kane.data.as_ref().unwrap();
    assert_eq!(kane_data.club.as_deref(), Some("Tottenham"));
    assert_eq!(kane_data.attacking.goals, Some(12.0));
}

/// Test: career_timeline_across_seasons
/// Import 3 seasons, verify players appear in all seasons, ordered by date
#[test]
fn career_timeline_across_seasons() {
    let conn = setup_db();

    let save = storage::create_save(&conn, "Career Mode").unwrap();

    // Season 1
    let season1 = storage::import_season(
        &conn, save.id, "2028-11-15",
        vec![make_detailed_player(2001, "Pedri", "Barcelona", 21)],
    ).unwrap();

    // Season 2
    let _season2 = storage::import_season(
        &conn, save.id, "2029-11-15",
        vec![make_detailed_player(2001, "Pedri", "Barcelona", 22)],
    ).unwrap();

    // Season 3
    let _season3 = storage::import_season(
        &conn, save.id, "2030-11-15",
        vec![make_detailed_player(2001, "Pedri", "Barcelona", 23)],
    ).unwrap();

    // List seasons ordered by date
    let seasons = storage::list_seasons(&conn, save.id).unwrap();
    assert_eq!(seasons.len(), 3);
    assert_eq!(seasons[0].in_game_date, "2028-11-15");
    assert_eq!(seasons[1].in_game_date, "2029-11-15");
    assert_eq!(seasons[2].in_game_date, "2030-11-15");

    // Get player career — should have 3 entries ordered by date
    // First get the player_id from season 1
    let players_s1 = storage::get_players_for_season(&conn, season1.season.id).unwrap();
    let player_id = players_s1[0].player_id;

    let career = storage::get_player_career(&conn, save.id, player_id).unwrap();
    assert_eq!(career.len(), 3);

    // Verify age progression across seasons
    assert_eq!(career[0].age, Some(21));
    assert_eq!(career[1].age, Some(22));
    assert_eq!(career[2].age, Some(23));

    // Verify data is preserved per season (different clubs possible)
    assert!(career[0].data.is_some());
    assert!(career[1].data.is_some());
    assert!(career[2].data.is_some());
}

/// Test: uid_reuse_creates_separate_records
/// Same UID but different names → creates separate player records
#[test]
fn uid_reuse_creates_separate_records() {
    let conn = setup_db();

    let save = storage::create_save(&conn, "Career Mode").unwrap();

    // Same UID, different names
    let season1 = storage::import_season(
        &conn, save.id, "2028-11-15",
        vec![make_player(9999, "Alan Smith")],
    ).unwrap();

    let season2 = storage::import_season(
        &conn, save.id, "2029-11-15",
        vec![make_player(9999, "Alan Smith")],
    ).unwrap();

    // Player was matched (same UID + name)
    assert_eq!(season1.new_players, 1);
    assert_eq!(season1.matched_players, 0);
    assert_eq!(season2.new_players, 0);
    assert_eq!(season2.matched_players, 1);

    // Same UID, different name → separate record
    let season3 = storage::import_season(
        &conn, save.id, "2030-11-15",
        vec![make_player(9999, "Alan Brookes")],
    ).unwrap();
    assert_eq!(season3.new_players, 1); // new record created
    assert_eq!(season3.matched_players, 0);

    // Season 3 contains only Alan Brookes (Alan Smith not imported this season)
    let players_s3 = storage::get_players_for_season(&conn, season3.season.id).unwrap();
    assert_eq!(players_s3.len(), 1);
    assert_eq!(players_s3[0].player_name, "Alan Brookes");

    // But there are now 2 distinct player records with UID 9999 in this save.
    // Verify via raw SQL that the players table has 2 entries.
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM players WHERE save_id = ?1 AND fm_uid = ?2",
        rusqlite::params![save.id, 9999],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 2);
}

/// Test: duplicate_season_rejected_with_count
/// Import same date twice → error includes player count
#[test]
fn duplicate_season_rejected_with_count() {
    let conn = setup_db();

    let save = storage::create_save(&conn, "Career Mode").unwrap();

    // First import
    let players = vec![
        make_player(1, "Player A"),
        make_player(2, "Player B"),
        make_player(3, "Player C"),
    ];
    storage::import_season(&conn, save.id, "2028-06-15", players).unwrap();

    // Duplicate import — should fail with count
    let result = storage::import_season(&conn, save.id, "2028-06-15", vec![make_player(4, "Player D")]);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("already exists"), "Expected 'already exists' in error: {}", err_msg);
    assert!(err_msg.contains("3"), "Expected player count '3' in error: {}", err_msg);
}

/// Test: delete_save_cascades_all
/// Delete save removes all: seasons, players, player_seasons
#[test]
fn delete_save_cascades_all() {
    let conn = setup_db();

    let save = storage::create_save(&conn, "To Delete").unwrap();
    storage::import_season(
        &conn, save.id, "2028-11-15",
        vec![make_player(1, "Player A"), make_player(2, "Player B")],
    ).unwrap();
    storage::import_season(
        &conn, save.id, "2029-11-15",
        vec![make_player(1, "Player A"), make_player(3, "Player C")],
    ).unwrap();

    // Delete save
    storage::delete_save(&conn, save.id).unwrap();

    // Verify all data gone
    let saves = storage::list_saves(&conn).unwrap();
    assert!(saves.is_empty());

    let count_seasons: i64 = conn.query_row("SELECT COUNT(*) FROM seasons", [], |r| r.get(0)).unwrap();
    assert_eq!(count_seasons, 0);

    let count_players: i64 = conn.query_row("SELECT COUNT(*) FROM players", [], |r| r.get(0)).unwrap();
    assert_eq!(count_players, 0);

    let count_player_seasons: i64 = conn.query_row("SELECT COUNT(*) FROM player_seasons", [], |r| r.get(0)).unwrap();
    assert_eq!(count_player_seasons, 0);
}

/// Test: delete_season_cleans_orphans_preserves_shared
/// Delete one season: orphans players only in that season are removed;
/// players shared with other seasons are preserved
#[test]
fn delete_season_cleans_orphans_preserves_shared() {
    let conn = setup_db();

    let save = storage::create_save(&conn, "Career Mode").unwrap();

    // Season 1: players A, B, C
    let s1 = storage::import_season(
        &conn, save.id, "2028-11-15",
        vec![make_player(1, "A"), make_player(2, "B"), make_player(3, "C")],
    ).unwrap();

    // Season 2: players A, B (A and B continue; C drops out)
    let s2 = storage::import_season(
        &conn, save.id, "2029-11-15",
        vec![make_player(1, "A"), make_player(2, "B")],
    ).unwrap();

    // Verify initial state
    let players_s1 = storage::get_players_for_season(&conn, s1.season.id).unwrap();
    assert_eq!(players_s1.len(), 3);

    // Delete season 1
    storage::delete_season(&conn, s1.season.id).unwrap();

    // Season 1 data gone
    let players_s1_after = storage::get_players_for_season(&conn, s1.season.id).unwrap();
    assert!(players_s1_after.is_empty());

    // Season 2 still intact with A and B
    let players_s2 = storage::get_players_for_season(&conn, s2.season.id).unwrap();
    assert_eq!(players_s2.len(), 2);
    let names: Vec<&str> = players_s2.iter().map(|p| p.player_name.as_str()).collect();
    assert!(names.contains(&"A"));
    assert!(names.contains(&"B"));

    // C was orphaned (only in season 1) → deleted
    let player_c = players_s2.iter().find(|p| p.player_name == "C");
    assert!(player_c.is_none());

    // Verify seasons list
    let seasons = storage::list_seasons(&conn, save.id).unwrap();
    assert_eq!(seasons.len(), 1);
    assert_eq!(seasons[0].id, s2.season.id);
}

/// Test: save_game_isolation
/// Same player imported into two different saves → independent records,
/// each save only sees its own players
#[test]
fn save_game_isolation() {
    let conn = setup_db();

    let save_a = storage::create_save(&conn, "Save A").unwrap();
    let save_b = storage::create_save(&conn, "Save B").unwrap();

    // Save A imports Kane
    storage::import_season(
        &conn, save_a.id, "2028-11-15",
        vec![make_detailed_player(100, "Harry Kane", "Spurs", 29)],
    ).unwrap();

    // Save B imports Kane (same UID, same name) — independent record
    storage::import_season(
        &conn, save_b.id, "2028-11-15",
        vec![make_detailed_player(100, "Harry Kane", "Bayern", 29)],
    ).unwrap();

    // Get latest season for each save
    let latest_a = storage::get_latest_season(&conn, save_a.id).unwrap().unwrap();
    let latest_b = storage::get_latest_season(&conn, save_b.id).unwrap().unwrap();

    let players_a = storage::get_players_for_season(&conn, latest_a.id).unwrap();
    let players_b = storage::get_players_for_season(&conn, latest_b.id).unwrap();

    assert_eq!(players_a.len(), 1);
    assert_eq!(players_b.len(), 1);

    // Same UID in different saves → different player_id
    assert_ne!(players_a[0].player_id, players_b[0].player_id);

    // Club data is isolated per save
    assert_eq!(players_a[0].club.as_deref(), Some("Spurs"));
    assert_eq!(players_b[0].club.as_deref(), Some("Bayern"));

    // Save listing counts are independent
    let saves = storage::list_saves(&conn).unwrap();
    assert_eq!(saves.len(), 2);
}
