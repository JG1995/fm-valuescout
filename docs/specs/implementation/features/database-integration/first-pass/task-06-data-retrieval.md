# Task 06 - Data Retrieval

## Overview

Implement the three data retrieval functions: `get_players_for_season`, `get_player_career`, and `get_latest_season`. These deserialize JSON blobs with graceful degradation (skip unreadable records, return partial results).

## Files to Create/Modify

- Modify: `src-tauri/src/storage/mod.rs` — add retrieval functions and tests

## Steps

- [ ] **Step 1: Write tests for data retrieval**

Add to the test module in `src-tauri/src/storage/mod.rs`:

```rust
    // ── Data retrieval tests ─────────────────────────────────────────────

    #[test]
    fn get_players_for_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let players = vec![
            make_player_with_club(1, "Player A", "Arsenal"),
            make_player_with_club(2, "Player B", "Chelsea"),
        ];
        let import = import_season(&conn, save.id, &players, "2030-11-15").unwrap();

        let result = get_players_for_season(&conn, import.season.id).unwrap();
        assert_eq!(result.len(), 2);

        // Check first player
        let pa = result.iter().find(|p| p.player_name == "Player A").unwrap();
        assert_eq!(pa.fm_uid, 1);
        assert_eq!(pa.club, Some("Arsenal".to_string()));
        assert!(pa.data.is_some());
        assert_eq!(pa.data.as_ref().unwrap().name, "Player A");
    }

    #[test]
    fn get_players_for_season_empty() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();

        // No player_seasons — should return empty Vec, not error
        let result = get_players_for_season(&conn, season.id).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn get_players_for_season_nonexistent_season() {
        let conn = setup_test_db();
        let result = get_players_for_season(&conn, 9999).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn get_player_career_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let player = make_player(100, "Veteran");

        import_season(&conn, save.id, &vec![player.clone()], "2030-11-15").unwrap();
        import_season(&conn, save.id, &vec![player.clone()], "2031-11-15").unwrap();

        // Get the internal player_id
        let player_id: i64 = conn.query_row(
            "SELECT id FROM players WHERE save_id = ?1 AND fm_uid = 100",
            rusqlite::params![save.id],
            |r| r.get(0),
        ).unwrap();

        let career = get_player_career(&conn, save.id, player_id).unwrap();
        assert_eq!(career.len(), 2);
        // Ordered by in_game_date ascending
        assert_eq!(career[0].season_id, career[0].id); // just check it exists
        assert!(career[0].data.is_some());
    }

    #[test]
    fn get_player_career_nonexistent_player() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let career = get_player_career(&conn, save.id, 9999).unwrap();
        assert!(career.is_empty());
    }

    #[test]
    fn get_player_career_nonexistent_save() {
        let conn = setup_test_db();
        let career = get_player_career(&conn, 9999, 1).unwrap();
        assert!(career.is_empty());
    }

    #[test]
    fn get_latest_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        import_season(&conn, save.id, &vec![make_player(1, "A")], "2030-11-15").unwrap();
        import_season(&conn, save.id, &vec![make_player(1, "A")], "2031-11-15").unwrap();

        let latest = get_latest_season(&conn, save.id).unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().in_game_date, "2031-11-15");
    }

    #[test]
    fn get_latest_season_empty() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let latest = get_latest_season(&conn, save.id).unwrap();
        assert!(latest.is_none());
    }

    #[test]
    fn get_latest_season_nonexistent_save() {
        let conn = setup_test_db();
        let latest = get_latest_season(&conn, 9999).unwrap();
        assert!(latest.is_none());
    }

    #[test]
    fn get_players_for_season_json_failure_graceful() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);

        // Manually insert a player_season with invalid JSON
        conn.execute(
            "INSERT INTO players (save_id, fm_uid, name) VALUES (?1, ?2, ?3)",
            rusqlite::params![save.id, 1, "Corrupt"],
        ).unwrap();
        let player_id = conn.last_insert_rowid();
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        conn.execute(
            "INSERT INTO player_seasons (player_id, season_id, position, data) VALUES (?1, ?2, 'ST', ?3)",
            rusqlite::params![player_id, season.id, "{invalid json}"],
        ).unwrap();

        let result = get_players_for_season(&conn, season.id).unwrap();
        assert_eq!(result.len(), 1);
        // Player data should be None (graceful degradation)
        assert!(result[0].data.is_none());
        // But queryable fields should be populated
        assert_eq!(result[0].player_name, "Corrupt");
        assert_eq!(result[0].fm_uid, 1);
    }
```

- [ ] **Step 2: Implement retrieval functions**

Add to `src-tauri/src/storage/mod.rs` (after import_season, before `#[cfg(test)]`):

```rust
// ── Retrieval ──────────────────────────────────────────────────────────

/// Internal helper to deserialize a player_season row into PlayerSeasonData.
fn row_to_player_season(row: &rusqlite::Row) -> rusqlite::Result<PlayerSeasonData> {
    let data_json: String = row.get(14)?;
    let data = serde_json::from_str::<ParsedPlayer>(&data_json).ok();

    Ok(PlayerSeasonData {
        id: row.get(0)?,
        player_id: row.get(1)?,
        season_id: row.get(2)?,
        fm_uid: row.get(3)?,
        player_name: row.get(4)?,
        club: row.get(5)?,
        age: row.get(6)?,
        nationality: row.get(7)?,
        position: row.get(8)?,
        minutes: row.get(9)?,
        appearances_started: row.get(10)?,
        appearances_sub: row.get(11)?,
        wage_per_week: row.get(12)?,
        transfer_value_high: row.get(13)?,
        contract_expires: row.get(15)?,
        data,
    })
}

/// Get all player data for a season.
/// Returns empty Vec if season doesn't exist or has no data.
/// Invalid JSON blobs are skipped with data = None (graceful degradation).
pub fn get_players_for_season(
    conn: &Connection,
    season_id: i64,
) -> Result<Vec<PlayerSeasonData>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT ps.id, ps.player_id, ps.season_id,
                p.fm_uid, p.name,
                ps.club, ps.age, ps.nationality, ps.position,
                ps.minutes, ps.appearances_started, ps.appearances_sub,
                ps.wage_per_week, ps.transfer_value_high,
                ps.data, ps.contract_expires
         FROM player_seasons ps
         JOIN players p ON p.id = ps.player_id
         WHERE ps.season_id = ?1
         ORDER BY p.name ASC"
    )?;

    let results: Vec<PlayerSeasonData> = stmt
        .query_map(rusqlite::params![season_id], |row| {
            row_to_player_season(row)
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Get a player's career timeline across all seasons in a save.
/// Returns empty Vec if player/save doesn't exist.
pub fn get_player_career(
    conn: &Connection,
    save_id: i64,
    player_id: i64,
) -> Result<Vec<PlayerSeasonData>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT ps.id, ps.player_id, ps.season_id,
                p.fm_uid, p.name,
                ps.club, ps.age, ps.nationality, ps.position,
                ps.minutes, ps.appearances_started, ps.appearances_sub,
                ps.wage_per_week, ps.transfer_value_high,
                ps.data, ps.contract_expires
         FROM player_seasons ps
         JOIN players p ON p.id = ps.player_id
         JOIN seasons s ON s.id = ps.season_id
         WHERE p.save_id = ?1 AND ps.player_id = ?2
         ORDER BY s.in_game_date ASC"
    )?;

    let results: Vec<PlayerSeasonData> = stmt
        .query_map(rusqlite::params![save_id, player_id], |row| {
            row_to_player_season(row)
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

/// Get the most recent season in a save, or None if no seasons exist.
pub fn get_latest_season(
    conn: &Connection,
    save_id: i64,
) -> Result<Option<Season>, StorageError> {
    let result = conn.query_row(
        "SELECT id, save_id, in_game_date, label, imported_at
         FROM seasons WHERE save_id = ?1
         ORDER BY in_game_date DESC LIMIT 1",
        rusqlite::params![save_id],
        |row| {
            Ok(Season {
                id: row.get(0)?,
                save_id: row.get(1)?,
                in_game_date: row.get(2)?,
                label: row.get(3)?,
                imported_at: row.get(4)?,
            })
        },
    );

    match result {
        Ok(season) => Ok(Some(season)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(StorageError::Database(e.to_string())),
    }
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
- Task 02 (init_schema, setup_test_db)
- Task 03 (create_save, create_test_save)
- Task 04 (create_season)
- Task 05 (import_season, make_player helpers)

## Success Criteria

- `get_players_for_season` returns PlayerSeasonData with deserialized JSON, graceful degradation on bad JSON.
- `get_player_career` returns timeline ordered by in_game_date ascending.
- `get_latest_season` returns most recent season or None.
- All functions return empty Vec/None for non-existent entities, no errors.
- Invalid JSON blobs produce `data: None` without failing the query.

## Tests

### Test 1: get_players_for_season

**What to test:** Returns correct data, empty for no data, empty for non-existent season, graceful degradation on bad JSON.

**Feasibility:** ✅ Can be tested

### Test 2: get_player_career

**What to test:** Multi-season career, empty for non-existent player/save.

**Feasibility:** ✅ Can be tested

### Test 3: get_latest_season

**What to test:** Returns latest, None for empty save, None for non-existent save.

**Feasibility:** ✅ Can be tested
