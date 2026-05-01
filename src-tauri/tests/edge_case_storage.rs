// Edge case tests for storage layer
// Tests boundary conditions, error paths, and unusual scenarios

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

// ── Edge case tests ─────────────────────────────────────────────────────

/// Empty player list → error with "No players" message
#[test]
fn import_empty_player_list_rejected() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Test Save").unwrap();

    let result = storage::import_season(&conn, save.id, "2028-11-15", vec![]);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.to_lowercase().contains("no players"),
             "Expected 'no players' in error: {}", err_msg);
}

/// Save with zero seasons → get_latest returns None, list is empty
#[test]
fn save_with_zero_seasons() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Empty Career").unwrap();

    let seasons = storage::list_seasons(&conn, save.id).unwrap();
    assert!(seasons.is_empty());

    let latest = storage::get_latest_season(&conn, save.id).unwrap();
    assert!(latest.is_none());
}

/// Player career with exactly one season → career Vec has 1 entry
#[test]
fn player_career_with_one_season() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "One Season").unwrap();

    let result = storage::import_season(
        &conn, save.id, "2028-11-15",
        vec![make_player(42, "Lone Wolf")],
    ).unwrap();

    let players = storage::get_players_for_season(&conn, result.season.id).unwrap();
    assert_eq!(players.len(), 1);
    let player_id = players[0].player_id;

    let career = storage::get_player_career(&conn, save.id, player_id).unwrap();
    assert_eq!(career.len(), 1);
}

/// Rename save to same name → succeeds (no-op)
#[test]
fn rename_save_to_same_name_noop() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Same Name").unwrap();

    let result = storage::rename_save(&conn, save.id, "Same Name");
    assert!(result.is_ok());
}

/// Rename season to same name → succeeds (no-op)
#[test]
fn rename_season_to_same_name_noop() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Test").unwrap();
    let result = storage::import_season(
        &conn, save.id, "2028-11-15",
        vec![make_player(1, "Player")],
    ).unwrap();

    let result2 = storage::rename_season(&conn, result.season.id, "2028/29");
    assert!(result2.is_ok());

    // Verify still works with same name
    let result3 = storage::rename_season(&conn, result.season.id, "2028/29");
    assert!(result3.is_ok());
}

/// Delete non-existent save → NotFound error
#[test]
fn delete_nonexistent_save() {
    let conn = setup_db();
    let result = storage::delete_save(&conn, 99999);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.to_lowercase().contains("not found"),
             "Expected 'not found' in error: {}", err_msg);
}

/// Delete non-existent season → NotFound error
#[test]
fn delete_nonexistent_season() {
    let conn = setup_db();
    let result = storage::delete_season(&conn, 99999);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.to_lowercase().contains("not found"),
             "Expected 'not found' in error: {}", err_msg);
}

/// Rename non-existent save → NotFound error
#[test]
fn rename_nonexistent_save() {
    let conn = setup_db();
    let result = storage::rename_save(&conn, 99999, "New Name");
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.to_lowercase().contains("not found"),
             "Expected 'not found' in error: {}", err_msg);
}

/// Rename non-existent season → NotFound error
#[test]
fn rename_nonexistent_season() {
    let conn = setup_db();
    let result = storage::rename_season(&conn, 99999, "New Label");
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.to_lowercase().contains("not found"),
             "Expected 'not found' in error: {}", err_msg);
}

/// Special characters in save name → handled correctly
#[test]
fn special_characters_in_save_name() {
    let conn = setup_db();

    let special_names = vec![
        "Årnes & Ølstad",
        "O'Brien's Save",
        "Save #1 (2025/26)",
        "日本語セーブ",
        "Emojis 🏆 Season",
        "Path\\with\\backslashes",
        "New\nLine\tTab",
    ];

    for name in special_names {
        let save = storage::create_save(&conn, name).unwrap();
        let saves = storage::list_saves(&conn).unwrap();
        let found = saves.iter().find(|s| s.name == name);
        assert!(found.is_some(), "Save '{}' not found after creation", name);

        storage::delete_save(&conn, save.id).unwrap();
    }
}

/// JSON deserialization failure → graceful degradation with data: None
#[test]
fn json_deserialization_failure_graceful() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Test").unwrap();

    // First create a normal season with one player
    let result = storage::import_season(
        &conn, save.id, "2028-11-15",
        vec![make_player(1, "Good Player")],
    ).unwrap();
    let season_id = result.season.id;

    // Get the player_id from the good player
    let players = storage::get_players_for_season(&conn, season_id).unwrap();
    let player_id = players[0].player_id;

    // Now corrupt the JSON data field directly
    conn.execute(
        "UPDATE player_seasons SET data = ?1 WHERE player_id = ?2 AND season_id = ?3",
        rusqlite::params!["{invalid json\x00", player_id, season_id],
    ).unwrap();

    // Re-fetch — data should be None but queryable fields available
    let players = storage::get_players_for_season(&conn, season_id).unwrap();
    assert_eq!(players.len(), 1);

    let player = &players[0];
    assert!(player.data.is_none(), "Expected data: None for invalid JSON");
    // Queryable fields should still work
    assert_eq!(player.player_name, "Good Player");
    assert_eq!(player.fm_uid, 1);
}

/// July 1 → new season label (2025-07-01 → "2025/26")
#[test]
fn season_label_july_1_is_new_season() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Test").unwrap();

    let result = storage::import_season(
        &conn, save.id, "2025-07-01",
        vec![make_player(1, "Player")],
    ).unwrap();
    assert_eq!(result.season.label, "2025/26");
}

/// June 30 → old season label (2025-06-30 → "2024/25")
#[test]
fn season_label_june_30_is_old_season() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Test").unwrap();

    let result = storage::import_season(
        &conn, save.id, "2025-06-30",
        vec![make_player(1, "Player")],
    ).unwrap();
    assert_eq!(result.season.label, "2024/25");
}

/// Dec 31 boundary (2024-12-31 → "2024/25")
#[test]
fn season_label_dec_31() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Test").unwrap();

    let result = storage::import_season(
        &conn, save.id, "2024-12-31",
        vec![make_player(1, "Player")],
    ).unwrap();
    assert_eq!(result.season.label, "2024/25");
}

/// Jan 1 boundary (2025-01-01 → "2024/25")
#[test]
fn season_label_jan_1() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Test").unwrap();

    let result = storage::import_season(
        &conn, save.id, "2025-01-01",
        vec![make_player(1, "Player")],
    ).unwrap();
    assert_eq!(result.season.label, "2024/25");
}

/// Feb 29 in non-leap year → invalid date handling
#[test]
fn invalid_date_february_29_non_leap() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Test").unwrap();

    // 2023 is not a leap year - date validation should reject this
    let result = storage::import_season(&conn, save.id, "2023-02-29", vec![]);
    assert!(result.is_err(), "Feb 29 2023 should be rejected as invalid");
}

/// Feb 29 in leap year → valid
#[test]
fn valid_date_february_29_leap() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Test").unwrap();

    // 2024 is a leap year
    let result = storage::import_season(
        &conn, save.id, "2024-02-29",
        vec![make_player(1, "Player")],
    ).unwrap();
    assert_eq!(result.season.label, "2023/24");
}

/// 100 char save name → accepted
#[test]
fn save_name_100_chars_accepted() {
    let conn = setup_db();
    let name = "a".repeat(100);
    let save = storage::create_save(&conn, &name);
    assert!(save.is_ok(), "100 char name should be accepted");
}

/// 101 char save name → rejected
#[test]
fn save_name_101_chars_rejected() {
    let conn = setup_db();
    let name = "a".repeat(101);
    let save = storage::create_save(&conn, &name);
    assert!(save.is_err(), "101 char name should be rejected");
}

/// Whitespace-only save name → rejected
#[test]
fn save_name_whitespace_only_rejected() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "   ");
    assert!(save.is_err(), "Whitespace-only name should be rejected");
}

/// Save name with leading/trailing whitespace → trimmed
#[test]
fn save_name_trimmed() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "  Trimmed Name  ").unwrap();

    let saves = storage::list_saves(&conn).unwrap();
    let found = saves.iter().find(|s| s.name == "Trimmed Name");
    assert!(found.is_some(), "Name should be trimmed to 'Trimmed Name'");
}

/// Same date in different saves → allowed
#[test]
fn import_same_date_different_saves_ok() {
    let conn = setup_db();
    let save1 = storage::create_save(&conn, "Save 1").unwrap();
    let save2 = storage::create_save(&conn, "Save 2").unwrap();

    // Import same date in both saves
    let result1 = storage::import_season(
        &conn, save1.id, "2028-11-15",
        vec![make_player(1, "Player One")],
    ).unwrap();

    let result2 = storage::import_season(
        &conn, save2.id, "2028-11-15",
        vec![make_player(1, "Player One")],
    ).unwrap();

    // Both should succeed
    assert!(result1.season.id > 0);
    assert!(result2.season.id > 0);
    // Labels may differ based on save or be the same
}

/// Minimal player with all optionals None → imports successfully
#[test]
fn import_minimal_player() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Test").unwrap();

    let minimal = ParsedPlayer::empty(1, "Minimal Player".to_string(), vec![]);

    let result = storage::import_season(
        &conn, save.id, "2028-11-15",
        vec![minimal],
    ).unwrap();

    assert_eq!(result.total_players, 1);
    assert_eq!(result.new_players, 1);

    let players = storage::get_players_for_season(&conn, result.season.id).unwrap();
    assert_eq!(players.len(), 1);
    assert_eq!(players[0].player_name, "Minimal Player");
}

/// Large import: 500 players → all imported and retrievable
#[test]
fn import_500_players() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Large Import").unwrap();

    let players: Vec<ParsedPlayer> = (1..=500)
        .map(|i| make_player(i, &format!("Player {}", i)))
        .collect();

    let result = storage::import_season(
        &conn, save.id, "2028-11-15",
        players,
    ).unwrap();

    assert_eq!(result.total_players, 500);
    assert_eq!(result.new_players, 500);

    // Retrieve and verify all 500
    let retrieved = storage::get_players_for_season(&conn, result.season.id).unwrap();
    assert_eq!(retrieved.len(), 500);

    // Verify some specific players
    let p1 = retrieved.iter().find(|p| p.player_name == "Player 1");
    let p250 = retrieved.iter().find(|p| p.player_name == "Player 250");
    let p500 = retrieved.iter().find(|p| p.player_name == "Player 500");

    assert!(p1.is_some());
    assert!(p250.is_some());
    assert!(p500.is_some());
}
