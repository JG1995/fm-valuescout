# Task 08 - Integration Tests

## Overview

Write integration tests that exercise the full storage layer against a real SQLite database (in-memory). These tests validate end-to-end flows: create save → import season → retrieve data → verify integrity.

## Files to Create/Modify

- Create: `src-tauri/tests/integration_storage.rs`

## Steps

- [ ] **Step 1: Create the integration test file**

Create `src-tauri/tests/integration_storage.rs` with:

```rust
// Integration tests for storage layer against real SQLite database
// Tests the full flow: create save → import → retrieve → verify

use fm_valuescout_lib::storage;
use fm_valuescout_lib::parser::types::{
    ParsedPlayer, Position, Role, Side, Nationality,
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
        wage_per_week: Some(100000.0),
        ..Default::default()
    };
    p.transfer_value = TransferValue {
        high: Some(30_000_000.0),
        ..Default::default()
    };
    p.contract_expires = Some("2032-06-30".to_string());
    p.attacking.goals = Some(12.0);
    p
}

// ── Full import flow ───────────────────────────────────────────────────

#[test]
fn full_import_flow() {
    let conn = setup_db();

    // Create save
    let save = storage::create_save(&conn, "My Career").unwrap();
    assert_eq!(save.name, "My Career");

    // Import season 1
    let players_s1 = vec![
        make_detailed_player(1, "John Smith", "Arsenal", 25),
        make_detailed_player(2, "Jane Doe", "Chelsea", 28),
        make_player(3, "Rookie"),
    ];
    let result1 = storage::import_season(&conn, save.id, &players_s1, "2030-11-15").unwrap();
    assert_eq!(result1.total_players, 3);
    assert_eq!(result1.new_players, 3);
    assert_eq!(result1.matched_players, 0);
    assert_eq!(result1.season.label, "2030/31");

    // Retrieve players for season
    let season_players = storage::get_players_for_season(&conn, result1.season.id).unwrap();
    assert_eq!(season_players.len(), 3);

    // Verify detailed player data
    let john = season_players.iter().find(|p| p.player_name == "John Smith").unwrap();
    assert_eq!(john.club, Some("Arsenal".to_string()));
    assert_eq!(john.age, Some(25));
    assert_eq!(john.minutes, Some(1800));
    assert_eq!(john.wage_per_week, Some(100000.0));
    assert!(john.data.is_some());
    assert_eq!(john.data.as_ref().unwrap().attacking.goals, Some(12.0));

    // Import season 2 with same player + new player
    let players_s2 = vec![
        make_detailed_player(1, "John Smith", "Arsenal", 26),
        make_player(4, "New Guy"),
    ];
    let result2 = storage::import_season(&conn, save.id, &players_s2, "2031-11-15").unwrap();
    assert_eq!(result2.total_players, 2);
    assert_eq!(result2.new_players, 1);
    assert_eq!(result2.matched_players, 1);
    assert_eq!(result2.season.label, "2031/32");

    // Verify total players in save
    let saves = storage::list_saves(&conn).unwrap();
    assert_eq!(saves[0].player_count, 4); // 4 unique players
    assert_eq!(saves[0].season_count, 2);
}

// ── Career timeline ────────────────────────────────────────────────────

#[test]
fn career_timeline_across_seasons() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Timeline Test").unwrap();

    // Import 3 seasons
    let p1 = make_player(100, "Veteran");
    for (i, date) in ["2030-11-15", "2031-11-15", "2032-11-15"].iter().enumerate() {
        let mut player = p1.clone();
        player.age = Some((25 + i as u16));
        storage::import_season(&conn, save.id, &vec![player], date).unwrap();
    }

    // Get player ID
    let player_id: i64 = conn.query_row(
        "SELECT id FROM players WHERE save_id = ?1 AND fm_uid = 100",
        rusqlite::params![save.id],
        |r| r.get(0),
    ).unwrap();

    // Retrieve career
    let career = storage::get_player_career(&conn, save.id, player_id).unwrap();
    assert_eq!(career.len(), 3);

    // Verify ordering by date
    assert_eq!(career[0].age, Some(25));
    assert_eq!(career[1].age, Some(26));
    assert_eq!(career[2].age, Some(27));

    // All entries should have valid data
    for entry in &career {
        assert!(entry.data.is_some());
    }
}

// ── UID reuse scenario ─────────────────────────────────────────────────

#[test]
fn uid_reuse_creates_separate_records() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "UID Reuse").unwrap();

    // Season 1: John Smith with UID 12345
    storage::import_season(
        &conn, save.id,
        &vec![make_player(12345, "John Smith")],
        "2030-11-15",
    ).unwrap();

    // Season 5: Carlos Garcia reuses UID 12345 (newgen)
    let result = storage::import_season(
        &conn, save.id,
        &vec![make_player(12345, "Carlos Garcia")],
        "2034-11-15",
    ).unwrap();
    assert_eq!(result.new_players, 1);
    assert_eq!(result.matched_players, 0);

    // Two distinct player records with same fm_uid
    let player_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM players WHERE save_id = ?1 AND fm_uid = 12345",
        rusqlite::params![save.id],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(player_count, 2);

    // Each has their own career
    let ids: Vec<i64> = conn.query_row_and_then(
        "SELECT id FROM players WHERE save_id = ?1 AND fm_uid = 12345 AND name = 'John Smith'",
        rusqlite::params![save.id],
        |r| r.get(0),
    ).into_iter().collect();
    let john_id = ids[0];
    let john_career = storage::get_player_career(&conn, save.id, john_id).unwrap();
    assert_eq!(john_career.len(), 1);
}

// ── Duplicate season rejection ─────────────────────────────────────────

#[test]
fn duplicate_season_rejected_with_count() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Dup Test").unwrap();

    let players = vec![make_player(1, "A"), make_player(2, "B")];
    storage::import_season(&conn, save.id, &players, "2030-11-15").unwrap();

    let result = storage::import_season(&conn, save.id, &players, "2030-11-15");
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("already exists"));
    assert!(err.contains("2 players"));
}

// ── Cascade delete ─────────────────────────────────────────────────────

#[test]
fn delete_save_cascades_all() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "To Delete").unwrap();

    storage::import_season(
        &conn, save.id,
        &vec![make_player(1, "A"), make_player(2, "B")],
        "2030-11-15",
    ).unwrap();

    storage::delete_save(&conn, save.id).unwrap();

    // Verify all data removed
    let saves = storage::list_saves(&conn).unwrap();
    assert!(saves.is_empty());

    let seasons: i64 = conn.query_row("SELECT COUNT(*) FROM seasons", [], |r| r.get(0)).unwrap();
    assert_eq!(seasons, 0);

    let players: i64 = conn.query_row("SELECT COUNT(*) FROM players", [], |r| r.get(0)).unwrap();
    assert_eq!(players, 0);

    let ps: i64 = conn.query_row("SELECT COUNT(*) FROM player_seasons", [], |r| r.get(0)).unwrap();
    assert_eq!(ps, 0);
}

// ── Season delete cleanup ──────────────────────────────────────────────

#[test]
fn delete_season_cleans_orphans_preserves_shared() {
    let conn = setup_db();
    let save = storage::create_save(&conn, "Cleanup").unwrap();

    // Import two seasons with overlapping players
    let shared = make_player(1, "Shared");
    let only_s1 = make_player(2, "Only S1");
    let only_s2 = make_player(3, "Only S2");

    let s1 = storage::import_season(
        &conn, save.id, &vec![shared.clone(), only_s1], "2030-11-15",
    ).unwrap();
    let s2 = storage::import_season(
        &conn, save.id, &vec![shared, only_s2], "2031-11-15",
    ).unwrap();

    // Delete season 1
    storage::delete_season(&conn, s1.season.id).unwrap();

    // Season 2 should be intact
    let s2_players = storage::get_players_for_season(&conn, s2.season.id).unwrap();
    assert_eq!(s2_players.len(), 2);

    // "Only S1" should be orphaned and deleted
    let only_s1_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM players WHERE save_id = ?1 AND name = 'Only S1')",
        rusqlite::params![save.id],
        |r| r.get(0),
    ).unwrap();
    assert!(!only_s1_exists);

    // "Shared" should still exist (has season 2)
    let shared_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM players WHERE save_id = ?1 AND name = 'Shared')",
        rusqlite::params![save.id],
        |r| r.get(0),
    ).unwrap();
    assert!(shared_exists);
}

// ── Save-game isolation ────────────────────────────────────────────────

#[test]
fn save_game_isolation() {
    let conn = setup_db();
    let save1 = storage::create_save(&conn, "Career 1").unwrap();
    let save2 = storage::create_save(&conn, "Career 2").unwrap();

    let player = make_player(1, "Cross-Save Player");

    storage::import_season(&conn, save1.id, &vec![player.clone()], "2030-11-15").unwrap();
    storage::import_season(&conn, save2.id, &vec![player], "2030-11-15").unwrap();

    // Each save has its own player record
    let s1_players = storage::get_players_for_season(
        &conn,
        storage::get_latest_season(&conn, save1.id).unwrap().unwrap().id,
    ).unwrap();
    let s2_players = storage::get_players_for_season(
        &conn,
        storage::get_latest_season(&conn, save2.id).unwrap().unwrap().id,
    ).unwrap();

    assert_eq!(s1_players.len(), 1);
    assert_eq!(s2_players.len(), 1);
    assert_ne!(s1_players[0].player_id, s2_players[0].player_id);

    // Deleting save1 doesn't affect save2
    storage::delete_save(&conn, save1.id).unwrap();
    let s2_still = storage::get_players_for_season(
        &conn,
        storage::get_latest_season(&conn, save2.id).unwrap().unwrap().id,
    ).unwrap();
    assert_eq!(s2_still.len(), 1);
}
```

- [ ] **Step 2: Add `init_db_test` helper to storage module**

The integration tests use `storage::init_db_test(&conn)` which initializes schema on an existing connection. Add to `src-tauri/src/storage/mod.rs`:

```rust
/// Initialize schema on an existing connection. Used for testing.
pub fn init_db_test(conn: &Connection) -> Result<(), StorageError> {
    init_schema(conn)
}
```

- [ ] **Step 3: Run integration tests**

Run: `cd src-tauri && cargo test --test integration_storage`
Expected: All integration tests pass.

- [ ] **Step 4: Verify all tests still pass**

Run: `cd src-tauri && cargo test`
Expected: All tests pass.

## Dependencies

- Tasks 01-07 (all storage functions and command wiring must be complete)

## Success Criteria

- Full import flow works end-to-end against in-memory SQLite.
- Career timeline returns multiple seasons in correct order.
- UID reuse creates separate player records.
- Duplicate season rejection includes player count.
- Cascade delete removes all related data.
- Season delete cleans orphans but preserves shared players.
- Save-game isolation verified — separate saves don't interfere.

## Tests

### Test 1: Full import flow

**What to test:** Create save → import → retrieve → verify data integrity.

**Feasibility:** ✅ Can be tested

### Test 2: Career timeline

**What to test:** 3 seasons, ordered by date, age progression.

**Feasibility:** ✅ Can be tested

### Test 3: UID reuse

**What to test:** Same UID, different names → separate records and careers.

**Feasibility:** ✅ Can be tested

### Test 4: Cascade delete

**What to test:** Delete save removes all data.

**Feasibility:** ✅ Can be tested

### Test 5: Season delete with orphan cleanup

**What to test:** Orphaned players deleted, shared players preserved.

**Feasibility:** ✅ Can be tested

### Test 6: Save-game isolation

**What to test:** Same player imported into different saves → independent records.

**Feasibility:** ✅ Can be tested
