use std::sync::Mutex;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::parser::types::ParsedPlayer;

// ── Database state ─────────────────────────────────────────────────────

/// Tauri-managed state wrapping a single SQLite connection.
/// Single-user app; Mutex prevents concurrent access within the app.
pub struct DbState {
    pub conn: Mutex<Connection>,
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

/// Database initialization (Task 02).
pub fn init_db(_app_data_dir: &str) -> Result<Connection, StorageError> {
    unimplemented!("Task 02: init_db")
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
}
