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
}
