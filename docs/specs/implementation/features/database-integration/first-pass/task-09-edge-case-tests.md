# Task 09 - Edge Case Tests

## Overview

Test boundary conditions, error paths, and unusual scenarios that the integration tests don't cover: empty player lists, zero-season saves, single-season careers, rename no-ops, non-existent entity operations, special characters, and JSON deserialization failure simulation.

## Files to Create/Modify

- Create: `src-tauri/tests/edge_case_storage.rs`

## Steps

- [ ] **Step 1: Create the edge case test file**

Create `src-tauri/tests/edge_case_storage.rs` with:

```rust
// Edge case tests for storage layer
// Tests boundary conditions, error paths, and unusual scenarios

use fm_valuescout_lib::storage;
use fm_valuescout_lib::parser::types::{
    ParsedPlayer, Position, Role, Side, Nationality, Footedness,
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

// ── Empty player list ──────────────────────────────────────────────────

#[test]
fn import_empty_player_list_rejected() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Empty Import").unwrap();
    let result = storage::import_season(&conn, save.id, &[], "2030-11-15");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No players to import"));
}

// ── Zero-season save ───────────────────────────────────────────────────

#[test]
fn save_with_zero_seasons() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "No Seasons").unwrap();

    let seasons = storage::list_seasons(&conn, save.id).unwrap();
    assert!(seasons.is_empty());

    let latest = storage::get_latest_season(&conn, save.id).unwrap();
    assert!(latest.is_none());

    let saves = storage::list_saves(&conn).unwrap();
    assert_eq!(saves[0].season_count, 0);
}

// ── Single-season career ───────────────────────────────────────────────

#[test]
fn player_career_with_one_season() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Single Season").unwrap();
    let result = storage::import_season(
        &conn, save.id, &vec![make_player(1, "One-Season Wonder")], "2030-11-15",
    ).unwrap();

    let player_id: i64 = conn.query_row(
        "SELECT id FROM players WHERE save_id = ?1 AND fm_uid = 1",
        rusqlite::params![save.id],
        |r| r.get(0),
    ).unwrap();

    let career = storage::get_player_career(&conn, save.id, player_id).unwrap();
    assert_eq!(career.len(), 1);
    assert_eq!(career[0].player_name, "One-Season Wonder");
}

// ── Rename to same name ────────────────────────────────────────────────

#[test]
fn rename_save_to_same_name_noop() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Same Name").unwrap();
    let result = storage::rename_save(&conn, save.id, "Same Name");
    assert!(result.is_ok());
}

#[test]
fn rename_season_to_same_name_noop() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Season Rename").unwrap();
    let season = storage::create_season(&conn, save.id, "2030-11-15").unwrap();
    let result = storage::rename_season(&conn, season.id, &season.label);
    assert!(result.is_ok());
}

// ── Delete non-existent entities ───────────────────────────────────────

#[test]
fn delete_nonexistent_save() {
    let conn = setup_db();
    let result = storage::delete_save(&conn, 9999);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn delete_nonexistent_season() {
    let conn = setup_db();
    let result = storage::delete_season(&conn, 9999);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn rename_nonexistent_save() {
    let conn = setup_db();
    let result = storage::rename_save(&conn, 9999, "Ghost");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn rename_nonexistent_season() {
    let conn = setup_db();
    let result = storage::rename_season(&conn, 9999, "Ghost");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

// ── Special characters in names ────────────────────────────────────────

#[test]
fn special_characters_in_save_name() {
    let conn = setup_db();
    let names = vec![
        "Årnes & Ølstad",
        "O'Brien's Save",
        "Save #1 (2025/26)",
        "日本語セーブ",
        "Emojis: ⚽🏆",
        "Save with \"quotes\"",
        "Save's apostrophe",
    ];
    for name in names {
        let save = storage::create_save(&conn, name).unwrap();
        assert_eq!(save.name, name);
    }
}

#[test]
fn special_characters_in_season_label() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Labels").unwrap();
    let season = storage::create_season(&conn, save.id, "2030-11-15").unwrap();
    let new_label = "Vårsesongen 2030/31 ⚽";
    storage::rename_season(&conn, season.id, new_label).unwrap();
    let seasons = storage::list_seasons(&conn, save.id).unwrap();
    assert_eq!(seasons[0].label, new_label);
}

// ── JSON deserialization failure ────────────────────────────────────────

#[test]
fn json_deserialization_failure_graceful() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Bad JSON").unwrap();

    // Create a player and season
    let season = storage::create_season(&conn, save.id, "2030-11-15").unwrap();
    conn.execute(
        "INSERT INTO players (save_id, fm_uid, name) VALUES (?1, ?2, ?3)",
        rusqlite::params![save.id, 1, "Good Player"],
    ).unwrap();
    let good_player_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO player_seasons (player_id, season_id, position, data) VALUES (?1, ?2, 'ST', ?3)",
        rusqlite::params![good_player_id, season.id, "{\"uid\":1,\"name\":\"Good Player\",\"positions\":[{\"role\":\"ST\",\"sides\":[\"C\"]}],\"attacking\":{\"goals\":10}}"],
    ).unwrap();

    // Add a player with invalid JSON
    conn.execute(
        "INSERT INTO players (save_id, fm_uid, name) VALUES (?1, ?2, ?3)",
        rusqlite::params![save.id, 2, "Bad Player"],
    ).unwrap();
    let bad_player_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO player_seasons (player_id, season_id, position, data) VALUES (?1, ?2, 'ST', ?3)",
        rusqlite::params![bad_player_id, season.id, "{this is not valid json}"],
    ).unwrap();

    // Query should return both, with graceful degradation
    let results = storage::get_players_for_season(&conn, season.id).unwrap();
    assert_eq!(results.len(), 2);

    let good = results.iter().find(|p| p.player_name == "Good Player").unwrap();
    assert!(good.data.is_some());

    let bad = results.iter().find(|p| p.player_name == "Bad Player").unwrap();
    assert!(bad.data.is_none()); // Graceful degradation
    assert_eq!(bad.fm_uid, 2); // Queryable fields still available
}

// ── Date boundary conditions ───────────────────────────────────────────

#[test]
fn season_label_july_1_is_new_season() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Dates").unwrap();
    let season = storage::create_season(&conn, save.id, "2030-07-01").unwrap();
    assert_eq!(season.label, "2030/31");
}

#[test]
fn season_label_june_30_is_old_season() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Dates").unwrap();
    let season = storage::create_season(&conn, save.id, "2030-06-30").unwrap();
    assert_eq!(season.label, "2029/30");
}

#[test]
fn season_label_dec_31() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Dates").unwrap();
    let season = storage::create_season(&conn, save.id, "2030-12-31").unwrap();
    assert_eq!(season.label, "2030/31");
}

#[test]
fn season_label_jan_1() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Dates").unwrap();
    let season = storage::create_season(&conn, save.id, "2031-01-01").unwrap();
    assert_eq!(season.label, "2030/31");
}

#[test]
fn invalid_date_february_29_non_leap() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Dates").unwrap();
    let result = storage::create_season(&conn, save.id, "2027-02-29");
    assert!(result.is_err());
}

#[test]
fn valid_date_february_29_leap() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Dates").unwrap();
    let result = storage::create_season(&conn, save.id, "2028-02-29");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().label, "2027/28");
}

// ── Save name validation edge cases ────────────────────────────────────

#[test]
fn save_name_100_chars_accepted() {
    let conn = setup_db();
    let name = "x".repeat(100);
    let save = storage::create_save(&conn, &name).unwrap();
    assert_eq!(save.name.len(), 100);
}

#[test]
fn save_name_101_chars_rejected() {
    let conn = setup_db();
    let name = "x".repeat(101);
    let result = storage::create_save(&conn, &name);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("100 characters"));
}

#[test]
fn save_name_whitespace_only_rejected() {
    let conn = setup_db();
    let result = storage::create_save(&conn, "   \t  ");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn save_name_trimmed() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "  Padded  ").unwrap();
    assert_eq!(save.name, "Padded");
}

// ── Multiple saves independence ────────────────────────────────────────

#[test]
fn import_same_date_different_saves_ok() {
    let conn = setup_db();
    let save1 = storage::create_save(&conn, "Save 1").unwrap();
    let save2 = storage::create_save(&conn, "Save 2").unwrap();

    let players = vec![make_player(1, "Shared")];

    let r1 = storage::import_season(&conn, save1.id, &players, "2030-11-15");
    assert!(r1.is_ok());

    let r2 = storage::import_season(&conn, save2.id, &players, "2030-11-15");
    assert!(r2.is_ok());
}

// ── Player with all optional fields None ───────────────────────────────

#[test]
fn import_minimal_player() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Minimal").unwrap();
    let mut player = ParsedPlayer::empty(1, "Minimal", vec![Position {
        role: Role::GK,
        sides: vec![Side::C],
    }]);
    // All optional fields are None by default
    let result = storage::import_season(&conn, save.id, &vec![player], "2030-11-15").unwrap();
    assert_eq!(result.total_players, 1);

    let data = storage::get_players_for_season(&conn, result.season.id).unwrap();
    assert_eq!(data.len(), 1);
    assert_eq!(data[0].club, None);
    assert_eq!(data[0].age, None);
    assert_eq!(data[0].minutes, None);
    assert!(data[0].data.is_some());
}

// ── Large import performance sanity ────────────────────────────────────

#[test]
fn import_500_players() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Large").unwrap();
    let players: Vec<ParsedPlayer> = (1..=500).map(|i| {
        make_player(i, &format!("Player {}", i))
    }).collect();

    let result = storage::import_season(&conn, save.id, &players, "2030-11-15").unwrap();
    assert_eq!(result.total_players, 500);
    assert_eq!(result.new_players, 500);

    let data = storage::get_players_for_season(&conn, result.season.id).unwrap();
    assert_eq!(data.len(), 500);
}
```

- [ ] **Step 2: Run edge case tests**

Run: `cd src-tauri && cargo test --test edge_case_storage`
Expected: All edge case tests pass.

- [ ] **Step 3: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass.

## Dependencies

- Tasks 01-07 (all storage functions must be complete)

## Success Criteria

- Empty player list rejected.
- Zero-season save returns empty results without error.
- Rename to same name succeeds as no-op.
- Delete/rename non-existent entities returns NotFound.
- Special characters in save names handled correctly.
- JSON deserialization failure returns partial results with `data: None`.
- Date boundary conditions correct (July 1, June 30, leap year).
- Save name validation (100 chars, 101 chars, whitespace-only, trimming).
- Same in_game_date in different saves is allowed.
- Minimal player (all optionals None) imported successfully.
- 500-player import works.

## Tests

### Test 1: Empty import

**What to test:** Zero players rejected.

**Feasibility:** ✅ Can be tested

### Test 2: Special characters

**What to test:** Unicode, quotes, apostrophes, emojis in save names.

**Feasibility:** ✅ Can be tested

### Test 3: JSON deserialization failure

**What to test:** Invalid JSON produces data: None, queryable fields still available.

**Feasibility:** ✅ Can be tested

### Test 4: Date boundaries

**What to test:** July 1, June 30, Dec 31, Jan 1, Feb 29 leap/non-leap.

**Feasibility:** ✅ Can be tested

### Test 5: Large import

**What to test:** 500 players imported and retrievable.

**Feasibility:** ✅ Can be tested
