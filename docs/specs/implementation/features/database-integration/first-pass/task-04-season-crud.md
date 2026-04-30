# Task 04 - Season CRUD Operations

## Overview

Implement season management: `create_season` (internal helper for import), `list_seasons`, `rename_season`, `delete_season`. The delete includes orphan player cleanup — players with no remaining seasons in the save are removed.

## Files to Create/Modify

- Modify: `src-tauri/src/storage/mod.rs` — add season CRUD functions and tests

## Steps

- [ ] **Step 1: Write tests for season CRUD**

Add to the test module in `src-tauri/src/storage/mod.rs`:

```rust
    // ── Season CRUD tests ────────────────────────────────────────────────

    fn create_test_save(conn: &Connection) -> Save {
        create_save(conn, "Test Save").unwrap()
    }

    #[test]
    fn create_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        assert_eq!(season.save_id, save.id);
        assert_eq!(season.in_game_date, "2030-11-15");
        assert_eq!(season.label, "2030/31");
    }

    #[test]
    fn create_season_invalid_date_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let result = create_season(&conn, save.id, "not-a-date");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid date format"));
    }

    #[test]
    fn create_season_duplicate_date_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        create_season(&conn, save.id, "2030-11-15").unwrap();
        let result = create_season(&conn, save.id, "2030-11-15");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn create_season_save_not_found() {
        let conn = setup_test_db();
        let result = create_season(&conn, 9999, "2030-11-15");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn list_seasons_empty() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let seasons = list_seasons(&conn, save.id).unwrap();
        assert!(seasons.is_empty());
    }

    #[test]
    fn list_seasons_ordered_by_date() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        create_season(&conn, save.id, "2031-06-15").unwrap(); // "2030/31"
        create_season(&conn, save.id, "2030-11-15").unwrap(); // "2030/31"
        create_season(&conn, save.id, "2031-11-15").unwrap(); // "2031/32"
        let seasons = list_seasons(&conn, save.id).unwrap();
        assert_eq!(seasons.len(), 3);
        // Ordered by in_game_date ascending
        assert_eq!(seasons[0].in_game_date, "2030-11-15");
        assert_eq!(seasons[1].in_game_date, "2031-06-15");
        assert_eq!(seasons[2].in_game_date, "2031-11-15");
    }

    #[test]
    fn rename_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        rename_season(&conn, season.id, "Våren 2026").unwrap();
        let seasons = list_seasons(&conn, save.id).unwrap();
        assert_eq!(seasons[0].label, "Våren 2026");
    }

    #[test]
    fn rename_season_empty_rejected() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        let result = rename_season(&conn, season.id, "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn rename_season_not_found() {
        let conn = setup_test_db();
        let result = rename_season(&conn, 9999, "New Label");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn delete_season_basic() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let season = create_season(&conn, save.id, "2030-11-15").unwrap();
        delete_season(&conn, season.id).unwrap();
        let seasons = list_seasons(&conn, save.id).unwrap();
        assert!(seasons.is_empty());
    }

    #[test]
    fn delete_season_not_found() {
        let conn = setup_test_db();
        let result = delete_season(&conn, 9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn delete_season_cleans_up_orphan_players() {
        let conn = setup_test_db();
        let save = create_test_save(&conn);
        let s1 = create_season(&conn, save.id, "2030-11-15").unwrap();
        let s2 = create_season(&conn, save.id, "2031-11-15").unwrap();

        // Insert a player manually
        conn.execute(
            "INSERT INTO players (save_id, fm_uid, name) VALUES (?1, ?2, ?3)",
            rusqlite::params![save.id, 12345, "John Smith"],
        ).unwrap();
        let player_id = conn.last_insert_rowid();

        // Player has seasons in both s1 and s2
        conn.execute(
            "INSERT INTO player_seasons (player_id, season_id, position, data) VALUES (?1, ?2, 'ST', '{}')",
            rusqlite::params![player_id, s1.id],
        ).unwrap();
        conn.execute(
            "INSERT INTO player_seasons (player_id, season_id, position, data) VALUES (?1, ?2, 'ST', '{}')",
            rusqlite::params![player_id, s2.id],
        ).unwrap();

        // Delete s1 — player still has s2, should NOT be orphaned
        delete_season(&conn, s1.id).unwrap();
        let player_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM players WHERE id = ?1",
            rusqlite::params![player_id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(player_count, 1);

        // Delete s2 — player is now orphaned, should be removed
        delete_season(&conn, s2.id).unwrap();
        let player_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM players WHERE id = ?1",
            rusqlite::params![player_id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(player_count, 0);
    }
```

- [ ] **Step 2: Implement season CRUD functions**

Add to `src-tauri/src/storage/mod.rs` (after save CRUD, before `#[cfg(test)]`):

```rust
// ── Season CRUD ────────────────────────────────────────────────────────

/// Create a season record with auto-derived label.
/// Internal helper used by import_season.
pub fn create_season(
    conn: &Connection,
    save_id: i64,
    in_game_date: &str,
) -> Result<Season, StorageError> {
    // Verify save exists
    let save_exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1)",
        rusqlite::params![save_id],
        |row| row.get(0),
    )?;
    if !save_exists {
        return Err(StorageError::NotFound("Save not found.".to_string()));
    }

    let label = derive_season_label(in_game_date)?;

    // Check for duplicate season in this save
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM seasons WHERE save_id = ?1 AND in_game_date = ?2)",
        rusqlite::params![save_id, in_game_date],
        |row| row.get(0),
    )?;
    if exists {
        // Get existing season info for the error message
        let player_count: i64 = conn.query_row(
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

    conn.execute(
        "INSERT INTO seasons (save_id, in_game_date, label) VALUES (?1, ?2, ?3)",
        rusqlite::params![save_id, in_game_date, label],
    )?;
    let id = conn.last_insert_rowid();
    let imported_at: String = conn.query_row(
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

/// List all seasons for a save, ordered by in_game_date ascending.
pub fn list_seasons(conn: &Connection, save_id: i64) -> Result<Vec<Season>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, save_id, in_game_date, label, imported_at
         FROM seasons WHERE save_id = ?1
         ORDER BY in_game_date ASC"
    )?;
    let seasons = stmt.query_map(rusqlite::params![save_id], |row| {
        Ok(Season {
            id: row.get(0)?,
            save_id: row.get(1)?,
            in_game_date: row.get(2)?,
            label: row.get(3)?,
            imported_at: row.get(4)?,
        })
    })?.filter_map(|r| r.ok()).collect();
    Ok(seasons)
}

/// Rename a season (updates display label only).
pub fn rename_season(conn: &Connection, season_id: i64, new_label: &str) -> Result<(), StorageError> {
    let label = new_label.trim().to_string();
    if label.is_empty() {
        return Err(StorageError::Validation(
            "Season name cannot be empty.".to_string(),
        ));
    }
    let rows = conn.execute(
        "UPDATE seasons SET label = ?1 WHERE id = ?2",
        rusqlite::params![label, season_id],
    )?;
    if rows == 0 {
        return Err(StorageError::NotFound("Season not found.".to_string()));
    }
    Ok(())
}

/// Delete a season, all associated player_seasons, and orphaned players.
pub fn delete_season(conn: &Connection, season_id: i64) -> Result<(), StorageError> {
    // Get save_id for orphan cleanup
    let save_id: Option<i64> = conn.query_row(
        "SELECT save_id FROM seasons WHERE id = ?1",
        rusqlite::params![season_id],
        |row| row.get(0),
    ).ok();

    let save_id = match save_id {
        Some(sid) => sid,
        None => return Err(StorageError::NotFound("Season not found.".to_string())),
    };

    // Delete player_seasons for this season
    conn.execute(
        "DELETE FROM player_seasons WHERE season_id = ?1",
        rusqlite::params![season_id],
    )?;

    // Delete the season
    conn.execute(
        "DELETE FROM seasons WHERE id = ?1",
        rusqlite::params![season_id],
    )?;

    // Clean up orphaned players (players with no remaining seasons in this save)
    conn.execute(
        "DELETE FROM players WHERE save_id = ?1 AND id NOT IN \
         (SELECT DISTINCT player_id FROM player_seasons \
          JOIN seasons ON player_seasons.season_id = seasons.id \
          WHERE seasons.save_id = ?1)",
        rusqlite::params![save_id, save_id],
    )?;

    Ok(())
}
```

- [ ] **Step 3: Run tests**

Run: `cd src-tauri && cargo test --lib storage`
Expected: All tests pass.

- [ ] **Step 4: Verify existing tests still pass**

Run: `cd src-tauri && cargo test`
Expected: All tests pass.

## Dependencies

- Task 01 (StorageError, Season DTO)
- Task 02 (init_schema, derive_season_label, setup_test_db)
- Task 03 (create_save, create_test_save helper)

## Success Criteria

- `create_season` auto-derives label, validates date, rejects duplicates, verifies save exists.
- `list_seasons` returns seasons ordered by in_game_date ascending.
- `rename_season` validates non-empty, rejects not-found.
- `delete_season` cascades to player_seasons and cleans up orphaned players.
- All tests pass.

## Tests

### Test 1: Create season

**What to test:** Basic creation, label derivation, invalid date, duplicate date, save not found.

**Feasibility:** ✅ Can be tested

### Test 2: List seasons

**What to test:** Empty, ordered by date.

**Feasibility:** ✅ Can be tested

### Test 3: Rename season

**What to test:** Basic rename, empty rejection, not found.

**Feasibility:** ✅ Can be tested

### Test 4: Delete season with orphan cleanup

**What to test:** Basic delete, cascade to player_seasons, orphan player cleanup when last season deleted.

**Feasibility:** ✅ Can be tested
