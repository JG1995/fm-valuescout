# Task 02 - Database Initialization and Schema

## Overview

Implement database lifecycle: create/open the SQLite file, initialize the schema (tables + indexes), and expose `init_db`. Implement the `derive_season_label` helper for season labeling.

## Files to Create/Modify

- Modify: `src-tauri/src/storage/mod.rs` — add schema DDL, init_db, derive_season_label

## Steps

- [ ] **Step 1: Write tests for `derive_season_label`**

Add these tests to the `#[cfg(test)] mod tests` block in `src-tauri/src/storage/mod.rs`:

```rust
    // ── derive_season_label tests ────────────────────────────────────────

    #[test]
    fn season_label_july_starts_new_season() {
        assert_eq!(derive_season_label("2030-07-01").unwrap(), "2030/31");
    }

    #[test]
    fn season_label_december_in_same_season() {
        assert_eq!(derive_season_label("2030-11-15").unwrap(), "2030/31");
    }

    #[test]
    fn season_label_january_in_previous_season() {
        assert_eq!(derive_season_label("2030-01-15").unwrap(), "2029/30");
    }

    #[test]
    fn season_label_june_end_of_season() {
        assert_eq!(derive_season_label("2030-06-30").unwrap(), "2029/30");
    }

    #[test]
    fn season_label_invalid_date_rejected() {
        let result = derive_season_label("not-a-date");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid date format"));
    }

    #[test]
    fn season_label_invalid_format_rejected() {
        let result = derive_season_label("30-06-2026");
        assert!(result.is_err());
    }

    #[test]
    fn season_label_feb_29_leap_year() {
        assert_eq!(derive_season_label("2028-02-29").unwrap(), "2027/28");
    }

    #[test]
    fn season_label_feb_29_non_leap_rejected() {
        let result = derive_season_label("2027-02-29");
        assert!(result.is_err());
    }

    #[test]
    fn season_label_century_boundary() {
        // 2099-12-01 → "2099/00" (year+1 mod 100 = 0)
        assert_eq!(derive_season_label("2099-12-01").unwrap(), "2099/00");
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test --lib storage`
Expected: Compilation fails — `derive_season_label` not yet defined.

- [ ] **Step 3: Implement `derive_season_label`**

Add above the `#[cfg(test)]` block in `src-tauri/src/storage/mod.rs`:

```rust
/// Derive a football-season label from an in-game date.
/// July-December: "year/year+1 (mod 100)" e.g. "2030/31"
/// January-June: "year-1/year (mod 100)" e.g. "2029/30"
pub fn derive_season_label(in_game_date: &str) -> Result<String, StorageError> {
    let date = chrono::NaiveDate::parse_from_str(in_game_date, "%Y-%m-%d")
        .map_err(|_| StorageError::Validation(
            "Invalid date format. Expected YYYY-MM-DD.".to_string()
        ))?;
    let (year, month) = (date.year(), date.month());
    if month >= 7 {
        Ok(format!("{}/{}", year, (year + 1) % 100))
    } else {
        Ok(format!("{}/{}", year - 1, year % 100))
    }
}
```

Note: `chrono` is already in Cargo.toml with `serde` feature.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --lib storage`
Expected: All derive_season_label tests pass.

- [ ] **Step 5: Write test for schema initialization**

Add to the test module:

```rust
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn schema_creates_all_tables() {
        let conn = setup_test_db();
        let tables: Vec<String> = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        ).unwrap()
            .query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(tables.contains(&"saves".to_string()));
        assert!(tables.contains(&"seasons".to_string()));
        assert!(tables.contains(&"players".to_string()));
        assert!(tables.contains(&"player_seasons".to_string()));
    }

    #[test]
    fn schema_creates_indexes() {
        let conn = setup_test_db();
        let indexes: Vec<String> = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%' ORDER BY name"
        ).unwrap()
            .query_map([], |row| row.get(0)).unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(indexes.contains(&"idx_seasons_save_id".to_string()));
        assert!(indexes.contains(&"idx_players_save_uid".to_string()));
        assert!(indexes.contains(&"idx_player_seasons_player".to_string()));
        assert!(indexes.contains(&"idx_player_seasons_season".to_string()));
    }

    #[test]
    fn schema_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        init_schema(&conn).unwrap(); // Second call should not fail
    }

    #[test]
    fn init_db_creates_file_and_schema() {
        let dir = std::env::temp_dir().join("fm_valuescout_test_init_db");
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test_init.db");
        let path_str = db_path.to_string_lossy().to_string();

        let conn = init_db(&path_str).unwrap();

        // Verify schema
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0),
        ).unwrap();
        assert_eq!(count, 4);

        // Clean up
        drop(conn);
        std::fs::remove_file(&db_path).ok();
        std::fs::remove_dir(&dir).ok();
    }
```

- [ ] **Step 6: Implement `init_schema` and update `init_db`**

Replace the placeholder `init_db` function in `src-tauri/src/storage/mod.rs` with:

```rust
/// SQL statements for schema creation. Idempotent (IF NOT EXISTS).
const SCHEMA_DDL: &str = "
CREATE TABLE IF NOT EXISTS saves (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL UNIQUE,
    managed_club TEXT,
    created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS seasons (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id      INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    in_game_date TEXT    NOT NULL,
    label        TEXT    NOT NULL,
    imported_at  TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(save_id, in_game_date)
);

CREATE TABLE IF NOT EXISTS players (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    save_id INTEGER NOT NULL REFERENCES saves(id) ON DELETE CASCADE,
    fm_uid  INTEGER NOT NULL,
    name    TEXT    NOT NULL,
    UNIQUE(save_id, fm_uid, name)
);

CREATE TABLE IF NOT EXISTS player_seasons (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    player_id           INTEGER NOT NULL REFERENCES players(id) ON DELETE CASCADE,
    season_id           INTEGER NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    club                TEXT,
    age                 INTEGER,
    nationality         TEXT,
    position            TEXT    NOT NULL,
    minutes             INTEGER,
    appearances_started INTEGER,
    appearances_sub     INTEGER,
    wage_per_week       REAL,
    transfer_value_high REAL,
    contract_expires    TEXT,
    data                TEXT    NOT NULL,
    UNIQUE(player_id, season_id)
);

CREATE INDEX IF NOT EXISTS idx_seasons_save_id ON seasons(save_id);
CREATE INDEX IF NOT EXISTS idx_players_save_uid ON players(save_id, fm_uid);
CREATE INDEX IF NOT EXISTS idx_player_seasons_player ON player_seasons(player_id);
CREATE INDEX IF NOT EXISTS idx_player_seasons_season ON player_seasons(season_id);
";

/// Create all tables and indexes. Idempotent.
fn init_schema(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(SCHEMA_DDL)?;
    Ok(())
}

/// Open (or create) the SQLite database at the given path and initialize schema.
pub fn init_db(db_path: &str) -> Result<Connection, StorageError> {
    let conn = Connection::open(db_path)?;
    init_schema(&conn)?;
    Ok(conn)
}
```

- [ ] **Step 7: Run all tests**

Run: `cd src-tauri && cargo test --lib storage`
Expected: All tests pass.

- [ ] **Step 8: Verify existing tests still pass**

Run: `cd src-tauri && cargo test`
Expected: All tests pass (113 existing + new).

## Dependencies

- Task 01 (StorageError, DTOs must exist)

## Success Criteria

- `init_db` opens/creates SQLite file at given path.
- `init_schema` creates 4 tables and 4 indexes, idempotent.
- `derive_season_label` produces correct labels for all months, rejects invalid dates.
- Foreign keys enabled on every connection.
- All unit tests pass.

## Tests

### Test 1: Season label derivation

**What to test:** July→June heuristic for all months, edge dates (Feb 29, century boundary), invalid format.

**Feasibility:** ✅ Can be tested

### Test 2: Schema creation

**What to test:** 4 tables, 4 indexes, idempotent double-call.

**Feasibility:** ✅ Can be tested

### Test 3: init_db file creation

**What to test:** Creates file on disk, schema present.

**Feasibility:** ✅ Can be tested
