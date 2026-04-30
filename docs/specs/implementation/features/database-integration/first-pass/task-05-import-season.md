# Task 05 - Import Season (Core Persistence)

## Overview

Implement `import_season` — the core function that replaces the `save_import` stub. This function matches players by `fm_uid + name`, creates or reuses player records, and batch-inserts `player_seasons` rows with JSON blobs in a single transaction.

## Files to Create/Modify

- Modify: `src-tauri/src/storage/mod.rs` — add import_season function and tests

## Steps

- [ ] **Step 1: Write tests for import_season**

Add to the test module in `src-tauri/src/storage/mod.rs`:

```rust
    // ── Import season tests ──────────────────────────────────────────────

    use crate::parser::types::{Position, Role, Side};

    fn make_player(uid: u32, name: &str) -> ParsedPlayer {
        ParsedPlayer::empty(uid, name.to_string(), vec![Position {
            role: Role::ST,
            sides: vec![Side::C],
        }])
    }

    fn make_player_with_club(uid: u32, name: &str, club: &str) -> ParsedPlayer {
        let mut p = make_player(uid, name);
        p.club = Some(club.to_string());
        p
    }

    #[test]
    fn import_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let players = vec![
            make_player(1, "Player A"),
            make_player(2, "Player B"),
        ];
        let result = import_season(&conn, save.id, &players, "2030-11-15").unwrap();
        assert_eq!(result.total_players, 2);
        assert_eq!(result.new_players, 2);
        assert_eq!(result.matched_players, 0);
        assert_eq!(result.season.label, "2030/31");
    }

    #[test]
    fn import_season_empty_players_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let result = import_season(&conn, save.id, &[], "2030-11-15");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No players to import"));
    }

    #[test]
    fn import_season_save_not_found() {
        let conn = setup_test_db();
        let players = vec![make_player(1, "Player A")];
        let result = import_season(&conn, 9999, &players, "2030-11-15");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn import_season_invalid_date_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let players = vec![make_player(1, "Player A")];
        let result = import_season(&conn, save.id, &players, "bad-date");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid date format"));
    }

    #[test]
    fn import_season_duplicate_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let players = vec![make_player(1, "Player A")];
        import_season(&conn, save.id, &players, "2030-11-15").unwrap();
        let result = import_season(&conn, save.id, &players, "2030-11-15");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn import_season_matches_existing_player() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        // Import season 1
        let players_s1 = vec![make_player(100, "John Smith")];
        import_season(&conn, save.id, &players_s1, "2030-11-15").unwrap();

        // Import season 2 with same player
        let players_s2 = vec![make_player(100, "John Smith")];
        let result = import_season(&conn, save.id, &players_s2, "2031-11-15").unwrap();
        assert_eq!(result.total_players, 1);
        assert_eq!(result.new_players, 0);
        assert_eq!(result.matched_players, 1);

        // Verify only 1 player record exists
        let player_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM players WHERE save_id = ?1",
            rusqlite::params![save.id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(player_count, 1);
    }

    #[test]
    fn import_season_uid_reuse_different_name_creates_new() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        // Import season 1: John Smith with UID 12345
        let players_s1 = vec![make_player(12345, "John Smith")];
        import_season(&conn, save.id, &players_s1, "2030-11-15").unwrap();

        // Import season 5: Carlos Garcia reuses UID 12345
        let players_s5 = vec![make_player(12345, "Carlos Garcia")];
        let result = import_season(&conn, save.id, &players_s5, "2034-11-15").unwrap();
        assert_eq!(result.total_players, 1);
        assert_eq!(result.new_players, 1);
        assert_eq!(result.matched_players, 0);

        // Verify 2 distinct player records
        let player_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM players WHERE save_id = ?1 AND fm_uid = 12345",
            rusqlite::params![save.id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(player_count, 2);
    }

    #[test]
    fn import_season_json_blob_stores_full_data() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let mut player = make_player(1, "Stats Player");
        player.club = Some("Arsenal".to_string());
        player.age = Some(25);
        player.minutes = Some(1800);
        player.attacking.goals = Some(15.0);

        import_season(&conn, save.id, &vec![player], "2030-11-15").unwrap();

        let data_json: String = conn.query_row(
            "SELECT data FROM player_seasons WHERE season_id = \
             (SELECT id FROM seasons WHERE save_id = ?1 AND in_game_date = '2030-11-15')",
            rusqlite::params![save.id],
            |row| row.get(0),
        ).unwrap();

        let parsed: ParsedPlayer = serde_json::from_str(&data_json).unwrap();
        assert_eq!(parsed.uid, 1);
        assert_eq!(parsed.name, "Stats Player");
        assert_eq!(parsed.club, Some("Arsenal".to_string()));
        assert_eq!(parsed.age, Some(25));
        assert_eq!(parsed.minutes, Some(1800));
        assert_eq!(parsed.attacking.goals, Some(15.0));
    }

    #[test]
    fn import_season_extracts_queryable_columns() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let mut player = make_player(1, "Detailed");
        player.club = Some("Chelsea".to_string());
        player.age = Some(28);
        player.nationality = Some(crate::parser::types::Nationality {
            code: Some("ENG".to_string()),
            name: "England".to_string(),
        });
        player.minutes = Some(2700);
        player.appearances_started = Some(30);
        player.appearances_sub = Some(2);
        player.wage.wage_per_week = Some(150000.0);
        player.transfer_value.high = Some(50000000.0);
        player.contract_expires = Some("2032-06-30".to_string());

        import_season(&conn, save.id, &vec![player], "2030-11-15").unwrap();

        let (club, age, nat, pos, mins, started, sub, wage, tv, contract): (
            Option<String>, Option<i64>, Option<String>, String,
            Option<i64>, Option<i64>, Option<i64>,
            Option<f64>, Option<f64>, Option<String>,
        ) = conn.query_row(
            "SELECT club, age, nationality, position, minutes, \
                    appearances_started, appearances_sub, \
                    wage_per_week, transfer_value_high, contract_expires \
             FROM player_seasons \
             WHERE season_id = (SELECT id FROM seasons WHERE save_id = ?1 AND in_game_date = '2030-11-15')",
            rusqlite::params![save.id],
            |row| Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?,
                row.get(4)?, row.get(5)?, row.get(6)?,
                row.get(7)?, row.get(8)?, row.get(9)?,
            )),
        ).unwrap();

        assert_eq!(club, Some("Chelsea".to_string()));
        assert_eq!(age, Some(28));
        assert_eq!(nat, Some("England".to_string()));
        assert_eq!(pos, "ST (C)");
        assert_eq!(mins, Some(2700));
        assert_eq!(started, Some(30));
        assert_eq!(sub, Some(2));
        assert_eq!(wage, Some(150000.0));
        assert_eq!(tv, Some(50000000.0));
        assert_eq!(contract, Some("2032-06-30".to_string()));
    }

    #[test]
    fn import_season_rollback_on_failure() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        // Import a season first
        import_season(&conn, save.id, &vec![make_player(1, "First")], "2030-11-15").unwrap();

        // Try importing the same season again — should fail
        let result = import_season(&conn, save.id, &vec![make_player(2, "Second")], "2030-11-15");
        assert!(result.is_err());

        // Verify no new player was created
        let player_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM players WHERE save_id = ?1",
            rusqlite::params![save.id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(player_count, 1); // Only "First"
    }
```

- [ ] **Step 2: Implement `import_season`**

Add to `src-tauri/src/storage/mod.rs` (after season CRUD, before `#[cfg(test)]`):

```rust
// ── Import ─────────────────────────────────────────────────────────────

/// Serialize player positions to a display string like "ST (C), AM (L)".
fn format_positions(positions: &[crate::parser::types::Position]) -> String {
    positions.iter().map(|p| {
        let sides: String = p.sides.iter().map(|s| match s {
            crate::parser::types::Side::L => "L",
            crate::parser::types::Side::C => "C",
            crate::parser::types::Side::R => "R",
        }).collect();
        format!("{:?} ({})", p.role, sides)
    }).collect::<Vec<_>>().join(", ")
}

/// Import a season's worth of parsed players into the database.
/// All inserts happen within a single transaction. All-or-nothing.
pub fn import_season(
    conn: &Connection,
    save_id: i64,
    players: &[ParsedPlayer],
    in_game_date: &str,
) -> Result<ImportResult, StorageError> {
    // Validate inputs
    if players.is_empty() {
        return Err(StorageError::Validation("No players to import.".to_string()));
    }

    // Verify save exists
    let save_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1)",
        rusqlite::params![save_id],
        |row| row.get(0),
    )?;
    if !save_exists {
        return Err(StorageError::NotFound("Save not found.".to_string()));
    }

    // Validate date (will also be checked by create_season)
    let _ = derive_season_label(in_game_date)?;

    // Begin transaction
    let tx = conn.unchecked_transaction()?;

    // Create season record (validates date and duplicate)
    let season = match create_season_tx(&tx, save_id, in_game_date) {
        Ok(s) => s,
        Err(e) => {
            let _ = tx.rollback();
            return Err(e);
        }
    };

    let mut new_players = 0usize;
    let mut matched_players = 0usize;

    for player in players {
        let player_name = player.name.trim();
        let position_str = format_positions(&player.positions);
        let data_json = serde_json::to_string(player)
            .map_err(|e| StorageError::Database(format!("JSON serialization failed: {}", e)))?;

        // Lookup existing player by (save_id, fm_uid, name) — case-insensitive name match
        let existing_id: Option<i64> = tx.query_row(
            "SELECT id FROM players WHERE save_id = ?1 AND fm_uid = ?2 AND LOWER(name) = LOWER(?3)",
            rusqlite::params![save_id, player.uid, player_name],
            |row| row.get(0),
        ).ok();

        let player_id = match existing_id {
            Some(id) => {
                matched_players += 1;
                id
            }
            None => {
                tx.execute(
                    "INSERT INTO players (save_id, fm_uid, name) VALUES (?1, ?2, ?3)",
                    rusqlite::params![save_id, player.uid, player_name],
                )?;
                new_players += 1;
                tx.last_insert_rowid()
            }
        };

        // Extract queryable fields from ParsedPlayer
        let nationality = player.nationality.as_ref().map(|n| n.name.clone());
        let wage = player.wage.wage_per_week;
        let transfer_value_high = player.transfer_value.high;

        tx.execute(
            "INSERT INTO player_seasons \
             (player_id, season_id, club, age, nationality, position, \
              minutes, appearances_started, appearances_sub, \
              wage_per_week, transfer_value_high, contract_expires, data) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                player_id,
                season.id,
                player.club,
                player.age.map(|a| a as i64),
                nationality,
                position_str,
                player.minutes.map(|m| m as i64),
                player.appearances_started.map(|a| a as i64),
                player.appearances_sub.map(|a| a as i64),
                wage,
                transfer_value_high,
                player.contract_expires,
                data_json,
            ],
        )?;
    }

    tx.commit()?;

    Ok(ImportResult {
        season,
        total_players: players.len(),
        new_players,
        matched_players,
    })
}

/// Internal create_season that takes a transaction instead of &Connection.
/// Used by import_season to keep everything in one transaction.
fn create_season_tx(
    tx: &rusqlite::Transaction,
    save_id: i64,
    in_game_date: &str,
) -> Result<Season, StorageError> {
    let label = derive_season_label(in_game_date)?;

    // Check for duplicate
    let exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM seasons WHERE save_id = ?1 AND in_game_date = ?2)",
        rusqlite::params![save_id, in_game_date],
        |row| row.get(0),
    )?;
    if exists {
        let player_count: i64 = tx.query_row(
            "SELECT COUNT(*) FROM player_seasons WHERE season_id = \
             (SELECT id FROM seasons WHERE save_id = ?1 AND in_game_date = ?2)",
            rusqlite::params![save_id, in_game_date],
            |row| row.get(0),
        ).unwrap_or(0);
        return Err(StorageError::Duplicate(format!(
            "Season for {} already exists ({} players). Delete it first to re-import.",
            in_game_date, player_count
        )));
    }

    tx.execute(
        "INSERT INTO seasons (save_id, in_game_date, label) VALUES (?1, ?2, ?3)",
        rusqlite::params![save_id, in_game_date, label],
    )?;
    let id = tx.last_insert_rowid();
    let imported_at: String = tx.query_row(
        "SELECT imported_at FROM seasons WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    )?;

    Ok(Season {
        id,
        save_id,
        in_game_date: in_game_date.to_string(),
        label,
        imported_at,
    })
}
```

- [ ] **Step 3: Run tests**

Run: `cd src-tauri && cargo test --lib storage`
Expected: All tests pass.

- [ ] **Step 4: Verify existing tests still pass**

Run: `cd src-tauri && cargo test`
Expected: All tests pass.

## Dependencies

- Task 01 (StorageError, DTOs, ParsedPlayer)
- Task 02 (init_schema, derive_season_label, setup_test_db)
- Task 03 (create_save, create_test_save)

## Success Criteria

- `import_season` creates season + player_seasons in single transaction.
- Player matching by `fm_uid + name` (case-insensitive).
- UID reuse with different name creates new player record.
- Empty players list rejected.
- Duplicate season rejected with player count.
- JSON blob contains full ParsedPlayer data.
- Queryable columns extracted correctly.
- Transaction rollback on failure.

## Tests

### Test 1: Basic import

**What to test:** Two new players imported, counts correct.

**Feasibility:** ✅ Can be tested

### Test 2: Player matching

**What to test:** Same fm_uid+name across seasons reuses player record.

**Feasibility:** ✅ Can be tested

### Test 3: UID reuse

**What to test:** Same fm_uid, different name creates new player.

**Feasibility:** ✅ Can be tested

### Test 4: JSON blob integrity

**What to test:** Full ParsedPlayer survives roundtrip.

**Feasibility:** ✅ Can be tested

### Test 5: Queryable columns

**What to test:** Club, age, nationality, position, minutes, appearances, wage, transfer value, contract extracted.

**Feasibility:** ✅ Can be tested

### Test 6: Transaction rollback

**What to test:** Failed import doesn't leave partial data.

**Feasibility:** ✅ Can be tested
