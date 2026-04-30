# Task 03 - Save CRUD Operations

## Overview

Implement the four save management functions: `create_save`, `list_saves`, `rename_save`, `delete_save`. These operate directly on the `saves` table with full validation.

## Files to Create/Modify

- Modify: `src-tauri/src/storage/mod.rs` — add save CRUD functions and tests

## Steps

- [ ] **Step 1: Write tests for save CRUD**

Add these tests to the test module in `src-tauri/src/storage/mod.rs`:

```rust
    // ── Save CRUD tests ──────────────────────────────────────────────────

    #[test]
    fn create_save_basic() {
        let conn = setup_test_db();
        let save = create_save(&conn, "My Save").unwrap();
        assert_eq!(save.name, "My Save");
        assert!(save.id > 0);
        assert!(save.managed_club.is_none());
        assert_eq!(save.season_count, 0);
        assert_eq!(save.player_count, 0);
    }

    #[test]
    fn create_save_empty_name_rejected() {
        let conn = setup_test_db();
        let result = create_save(&conn, "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn create_save_whitespace_name_rejected() {
        let conn = setup_test_db();
        let result = create_save(&conn, "   ");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn create_save_name_too_long_rejected() {
        let conn = setup_test_db();
        let long_name = "x".repeat(101);
        let result = create_save(&conn, &long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("100 characters"));
    }

    #[test]
    fn create_save_name_100_chars_ok() {
        let conn = setup_test_db();
        let name = "x".repeat(100);
        let save = create_save(&conn, &name).unwrap();
        assert_eq!(save.name.len(), 100);
    }

    #[test]
    fn create_save_duplicate_name_rejected() {
        let conn = setup_test_db();
        create_save(&conn, "My Save").unwrap();
        let result = create_save(&conn, "My Save");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn create_save_case_insensitive_duplicate() {
        let conn = setup_test_db();
        create_save(&conn, "My Save").unwrap();
        let result = create_save(&conn, "my save");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn create_save_special_chars_allowed() {
        let conn = setup_test_db();
        let save = create_save(&conn, "Årnes & Ølstad — \"2025/26\"").unwrap();
        assert_eq!(save.name, "Årnes & Ølstad — \"2025/26\"");
    }

    #[test]
    fn list_saves_empty() {
        let conn = setup_test_db();
        let saves = list_saves(&conn).unwrap();
        assert!(saves.is_empty());
    }

    #[test]
    fn list_saves_returns_created_save() {
        let conn = setup_test_db();
        create_save(&conn, "Save A").unwrap();
        create_save(&conn, "Save B").unwrap();
        let saves = list_saves(&conn).unwrap();
        assert_eq!(saves.len(), 2);
        let names: Vec<&str> = saves.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"Save A"));
        assert!(names.contains(&"Save B"));
    }

    #[test]
    fn rename_save_basic() {
        let conn = setup_test_db();
        let save = create_save(&conn, "Old Name").unwrap();
        rename_save(&conn, save.id, "New Name").unwrap();
        let saves = list_saves(&conn).unwrap();
        assert_eq!(saves[0].name, "New Name");
    }

    #[test]
    fn rename_save_to_same_name_noop() {
        let conn = setup_test_db();
        let save = create_save(&conn, "Same").unwrap();
        let result = rename_save(&conn, save.id, "Same");
        assert!(result.is_ok());
    }

    #[test]
    fn rename_save_empty_name_rejected() {
        let conn = setup_test_db();
        let save = create_save(&conn, "Valid").unwrap();
        let result = rename_save(&conn, save.id, "");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn rename_save_name_too_long_rejected() {
        let conn = setup_test_db();
        let save = create_save(&conn, "Valid").unwrap();
        let result = rename_save(&conn, save.id, &"x".repeat(101));
        assert!(result.is_err());
    }

    #[test]
    fn rename_save_duplicate_name_rejected() {
        let conn = setup_test_db();
        create_save(&conn, "Save A").unwrap();
        let save_b = create_save(&conn, "Save B").unwrap();
        let result = rename_save(&conn, save_b.id, "Save A");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn rename_save_not_found() {
        let conn = setup_test_db();
        let result = rename_save(&conn, 9999, "New");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn delete_save_basic() {
        let conn = setup_test_db();
        let save = create_save(&conn, "To Delete").unwrap();
        delete_save(&conn, save.id).unwrap();
        let saves = list_saves(&conn).unwrap();
        assert!(saves.is_empty());
    }

    #[test]
    fn delete_save_not_found() {
        let conn = setup_test_db();
        let result = delete_save(&conn, 9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn delete_save_cascades_seasons() {
        let conn = setup_test_db();
        let save = create_save(&conn, "With Seasons").unwrap();
        // Insert a season manually to test cascade
        conn.execute(
            "INSERT INTO seasons (save_id, in_game_date, label) VALUES (?1, ?2, ?3)",
            rusqlite::params![save.id, "2030-11-15", "2030/31"],
        ).unwrap();
        delete_save(&conn, save.id).unwrap();
        // Verify seasons are gone
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM seasons", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0);
    }
```

- [ ] **Step 2: Implement save CRUD functions**

Add these functions to `src-tauri/src/storage/mod.rs` (above the `#[cfg(test)]` block, after `init_db`):

```rust
// ── Save CRUD ──────────────────────────────────────────────────────────

/// Validate a save name: non-empty after trimming, max 100 chars.
fn validate_save_name(name: &str) -> Result<String, StorageError> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err(StorageError::Validation(
            "Save name cannot be empty.".to_string(),
        ));
    }
    if trimmed.len() > 100 {
        return Err(StorageError::Validation(
            "Save name must be 100 characters or fewer.".to_string(),
        ));
    }
    Ok(trimmed)
}

/// Create a new save-game. Names must be unique (case-insensitive).
pub fn create_save(conn: &Connection, name: &str) -> Result<Save, StorageError> {
    let name = validate_save_name(name)?;

    // Check for case-insensitive duplicate
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM saves WHERE LOWER(name) = LOWER(?1))",
        rusqlite::params![name],
        |row| row.get(0),
    )?;
    if exists {
        return Err(StorageError::Duplicate(format!(
            "A save with the name '{}' already exists.",
            name
        )));
    }

    conn.execute(
        "INSERT INTO saves (name) VALUES (?1)",
        rusqlite::params![name],
    )?;
    let id = conn.last_insert_rowid();

    Ok(Save {
        id,
        name,
        managed_club: None,
        created_at: String::new(), // Will be populated by list_saves
        season_count: 0,
        player_count: 0,
    })
}

/// List all saves with season and player counts.
pub fn list_saves(conn: &Connection) -> Result<Vec<Save>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.managed_club, s.created_at,
                COUNT(DISTINCT se.id) AS season_count,
                COUNT(DISTINCT p.id) AS player_count
         FROM saves s
         LEFT JOIN seasons se ON se.save_id = s.id
         LEFT JOIN players p ON p.save_id = s.id
         GROUP BY s.id
         ORDER BY s.created_at DESC"
    )?;

    let saves = stmt.query_map([], |row| {
        Ok(Save {
            id: row.get(0)?,
            name: row.get(1)?,
            managed_club: row.get(2)?,
            created_at: row.get(3)?,
            season_count: row.get(4)?,
            player_count: row.get(5)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(saves)
}

/// Rename a save. Validates the new name.
pub fn rename_save(conn: &Connection, save_id: i64, new_name: &str) -> Result<(), StorageError> {
    let new_name = validate_save_name(new_name)?;

    // Check for case-insensitive duplicate (excluding self)
    let exists: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM saves WHERE LOWER(name) = LOWER(?1) AND id != ?2)",
        rusqlite::params![new_name, save_id],
        |row| row.get(0),
    )?;
    if exists {
        return Err(StorageError::Duplicate(format!(
            "A save with the name '{}' already exists.",
            new_name
        )));
    }

    let rows = conn.execute(
        "UPDATE saves SET name = ?1 WHERE id = ?2",
        rusqlite::params![new_name, save_id],
    )?;
    if rows == 0 {
        return Err(StorageError::NotFound("Save not found.".to_string()));
    }
    Ok(())
}

/// Delete a save and all associated data (cascade: seasons, player_seasons, players).
pub fn delete_save(conn: &Connection, save_id: i64) -> Result<(), StorageError> {
    // Delete player_seasons first (no cascade from saves → player_seasons directly)
    conn.execute(
        "DELETE FROM player_seasons WHERE player_id IN (SELECT id FROM players WHERE save_id = ?1)",
        rusqlite::params![save_id],
    )?;
    // Delete players
    conn.execute(
        "DELETE FROM players WHERE save_id = ?1",
        rusqlite::params![save_id],
    )?;
    // Delete seasons
    conn.execute(
        "DELETE FROM seasons WHERE save_id = ?1",
        rusqlite::params![save_id],
    )?;
    // Delete save
    let rows = conn.execute(
        "DELETE FROM saves WHERE id = ?1",
        rusqlite::params![save_id],
    )?;
    if rows == 0 {
        return Err(StorageError::NotFound("Save not found.".to_string()));
    }
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

- Task 01 (StorageError, Save DTO)
- Task 02 (init_db, init_schema, setup_test_db helper)

## Success Criteria

- `create_save` validates name (empty, whitespace, length, duplicate case-insensitive), creates record.
- `list_saves` returns saves with season_count and player_count.
- `rename_save` validates new name, checks duplicate excluding self, rejects not-found.
- `delete_save` cascades to all related data (player_seasons, players, seasons).
- All tests pass.

## Tests

### Test 1: Create save validation

**What to test:** Empty, whitespace, too long, duplicate (case-insensitive), special characters.

**Feasibility:** ✅ Can be tested

### Test 2: List saves

**What to test:** Empty list, populated list with counts.

**Feasibility:** ✅ Can be tested

### Test 3: Rename save

**What to test:** Basic rename, same name no-op, empty rejection, duplicate rejection, not found.

**Feasibility:** ✅ Can be tested

### Test 4: Delete save

**What to test:** Basic delete, cascade to seasons, not found.

**Feasibility:** ✅ Can be tested
