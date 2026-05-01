use std::sync::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use chrono::Datelike;

use crate::parser::types::ParsedPlayer;

// ── Database state ─────────────────────────────────────────────────────

/// Tauri-managed state wrapping a single SQLite connection.
/// Single-user app; Mutex prevents concurrent access within the app.
pub struct DbState {
    pub conn: Mutex<Connection>,
}

// ── Database initialization ─────────────────────────────────────────────


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

// ── Helpers ──────────────────────────────────────────────────────────

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
        Ok(format!("{}/{:02}", year, (year + 1) % 100))
    } else {
        Ok(format!("{}/{:02}", year - 1, year % 100))
    }
}

// ── Error type ─────────────────────────────────────────────────────────

/// Internal error type for storage operations.
/// Command wrappers convert these to String for the Tauri boundary.
#[derive(Debug)]
pub enum StorageError {
    NotFound(String),
    Duplicate(String),
    Validation(String),
    Database(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::NotFound(msg) => write!(f, "{}", msg),
            StorageError::Duplicate(msg) => write!(f, "{}", msg),
            StorageError::Validation(msg) => write!(f, "{}", msg),
            StorageError::Database(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<StorageError> for String {
    fn from(err: StorageError) -> String {
        err.to_string()
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> StorageError {
        StorageError::Database(err.to_string())
    }
}

// ── DTOs ───────────────────────────────────────────────────────────────

/// A save-game record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Save {
    pub id: i64,
    pub name: String,
    pub managed_club: Option<String>,
    pub created_at: String,
    pub season_count: i64,
    pub player_count: i64,
}

/// A season snapshot within a save.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Season {
    pub id: i64,
    pub save_id: i64,
    pub in_game_date: String,
    pub label: String,
    pub imported_at: String,
}

/// Summary of a season import operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub season: Season,
    pub total_players: usize,
    pub new_players: usize,
    pub matched_players: usize,
}

/// A player's seasonal data record — one row from `player_seasons`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSeasonData {
    pub id: i64,
    pub player_id: i64,
    pub season_id: i64,
    pub fm_uid: i64,
    pub player_name: String,
    pub club: Option<String>,
    pub age: Option<i64>,
    pub nationality: Option<String>,
    pub position: String,
    pub minutes: Option<i64>,
    pub appearances_started: Option<i64>,
    pub appearances_sub: Option<i64>,
    pub wage_per_week: Option<f64>,
    pub transfer_value_high: Option<f64>,
    pub contract_expires: Option<String>,
    /// Full player data deserialized from the JSON blob.
    /// None if deserialization fails (graceful degradation).
    pub data: Option<ParsedPlayer>,
}

// ── Placeholder for future tasks ───────────────────────────────────────

/// Persist imported players to the database.
/// Currently an honest stub — returns an error until implemented.
/// Idempotent: skips players with same UID + in_game_date.
/// Will be implemented in a future task (DB write layer).
pub fn save_import(_players: Vec<ParsedPlayer>, _in_game_date: &str) -> Result<(), String> {
    Err("Storage is not yet implemented. Your data has not been saved.".to_string())
}

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
        "DELETE FROM players WHERE save_id = :save_id AND id NOT IN \
         (SELECT DISTINCT player_id FROM player_seasons \
          JOIN seasons ON player_seasons.season_id = seasons.id \
          WHERE seasons.save_id = :save_id)",
        rusqlite::named_params!{":save_id": save_id},
    )?;

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_error_to_string_not_found() {
        let err = StorageError::NotFound("Save not found.".to_string());
        assert_eq!(err.to_string(), "Save not found.");
    }

    #[test]
    fn storage_error_to_string_duplicate() {
        let err = StorageError::Duplicate("Already exists.".to_string());
        assert_eq!(err.to_string(), "Already exists.");
    }

    #[test]
    fn storage_error_to_string_validation() {
        let err = StorageError::Validation("Name cannot be empty.".to_string());
        assert_eq!(err.to_string(), "Name cannot be empty.");
    }

    #[test]
    fn storage_error_into_string() {
        let err = StorageError::Database("disk full".to_string());
        let s: String = err.into();
        assert_eq!(s, "disk full");
    }

    #[test]
    fn rusqlite_error_converts_to_database() {
        let rusqlite_err = rusqlite::Error::InvalidColumnIndex(999);
        let storage_err: StorageError = rusqlite_err.into();
        match storage_err {
            StorageError::Database(msg) => assert!(msg.contains("999")),
            _ => panic!("Expected Database variant"),
        }
    }


    #[test]
    fn import_result_serializable() {
        let result = ImportResult {
            season: Season {
                id: 1,
                save_id: 1,
                in_game_date: "2030-11-15".to_string(),
                label: "2030/31".to_string(),
                imported_at: "2026-04-30 12:00:00".to_string(),
            },
            total_players: 25,
            new_players: 20,
            matched_players: 5,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: ImportResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.total_players, 25);
        assert_eq!(back.season.label, "2030/31");
    }

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
        assert!(result.unwrap_err().to_string().contains("Invalid date format"));
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

    // ── schema initialization tests ───────────────────────────────────

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

        // Verify schema - check for our 4 expected tables (sqlite_sequence is auto-created for AUTOINCREMENT)
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


        // Clean up
        drop(conn);
        std::fs::remove_file(&db_path).ok();
        std::fs::remove_dir(&dir).ok();
    }

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
}
