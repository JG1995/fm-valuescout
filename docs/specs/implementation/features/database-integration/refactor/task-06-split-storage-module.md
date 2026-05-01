# Task 06 - Split Storage Module

## Overview

Split the 1700-line `storage/mod.rs` monolith into a module directory with focused files. Production code separates into 7 files of ≤200 lines each. Tests remain in `mod.rs`. No behavioral changes — pure structural reorganization.

## Files to Create

- `src-tauri/src/storage/error.rs` — StorageError type
- `src-tauri/src/storage/types.rs` — DbState, DTOs (Save, Season, ImportResult, PlayerSeasonData)
- `src-tauri/src/storage/schema.rs` — SCHEMA_DDL, init_schema, init_db, init_db_test
- `src-tauri/src/storage/saves.rs` — Save CRUD operations
- `src-tauri/src/storage/seasons.rs` — Season CRUD + create_season_tx
- `src-tauri/src/storage/import.rs` — format_positions, import_season
- `src-tauri/src/storage/retrieval.rs` — row_to_player_season, retrieval queries

## Files to Modify

- `src-tauri/src/storage/mod.rs` — Replace with re-exports + test module

## Context

After Tasks 01-05, `storage/mod.rs` contains ~750 lines of production code and ~945 lines of tests. The production code has clear boundaries:

| Section | Functions | Lines (approx) |
|---|---|---|
| Error type | StorageError, Display, From impls | ~35 |
| DTOs | Save, Season, ImportResult, PlayerSeasonData, DbState | ~55 |
| Schema | SCHEMA_DDL, init_schema, init_db, init_db_test | ~70 |
| Save CRUD | validate_save_name, create_save, list_saves, rename_save, delete_save | ~130 |
| Season CRUD + helpers | derive_season_label, create_season_tx, create_season, list_seasons, rename_season, delete_season | ~185 |
| Import | format_positions, import_season | ~100 |
| Retrieval | row_to_player_season, get_players_for_season, get_player_career, get_latest_season | ~110 |

Each section has minimal dependencies on others — they share `StorageError`, `Connection`, and DTOs but are otherwise independent.

## Steps

- [ ] **Step 1: Create `src-tauri/src/storage/error.rs`**

```rust
use std::fmt;

/// Internal error type for storage operations.
/// Command wrappers convert these to String for the Tauri boundary.
#[derive(Debug)]
pub enum StorageError {
    NotFound(String),
    Duplicate(String),
    Validation(String),
    Database(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
```

- [ ] **Step 2: Create `src-tauri/src/storage/types.rs`**

Copy the type definitions from `mod.rs`. These are the DTOs shared across all storage submodules:

```rust
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::parser::types::ParsedPlayer;

// ── Database state ─────────────────────────────────────────────────────

/// Tauri-managed state wrapping a single SQLite connection.
/// Single-user app; Mutex prevents concurrent access within the app.
pub struct DbState {
    pub conn: std::sync::Mutex<Connection>,
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
```

- [ ] **Step 3: Create `src-tauri/src/storage/schema.rs`**

```rust
use rusqlite::Connection;

use super::error::StorageError;

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
pub(super) fn init_schema(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.execute_batch(SCHEMA_DDL)?;
    Ok(())
}

/// Initialize schema on an existing connection. Used for testing.
pub fn init_db_test(conn: &Connection) -> Result<(), StorageError> {
    init_schema(conn)
}

/// Open (or create) the SQLite database at the given path and initialize schema.
pub fn init_db(db_path: &str) -> Result<Connection, StorageError> {
    let conn = Connection::open(db_path)?;
    init_schema(&conn)?;
    Ok(conn)
}
```

Note: `init_schema` is `pub(super)` — visible within the storage module but not outside it. Tests call `init_db_test` instead.

- [ ] **Step 4: Create `src-tauri/src/storage/saves.rs`**

Copy the save CRUD functions from `mod.rs`. Add the necessary imports:

```rust
use rusqlite::Connection;

use super::error::StorageError;
use super::types::Save;

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

    let created_at: String = conn.query_row(
        "SELECT created_at FROM saves WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    )?;

    Ok(Save {
        id,
        name,
        managed_club: None,
        created_at,
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

    let saves: Vec<Save> = stmt.query_map([], |row| {
        Ok(Save {
            id: row.get(0)?,
            name: row.get(1)?,
            managed_club: row.get(2)?,
            created_at: row.get(3)?,
            season_count: row.get(4)?,
            player_count: row.get(5)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

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

- [ ] **Step 5: Create `src-tauri/src/storage/seasons.rs`**

Copy season-related functions. `create_season_tx` is `pub(crate)` for use by `import.rs`:

```rust
use rusqlite::Connection;
use chrono::Datelike;

use super::error::StorageError;
use super::types::Season;

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

// ── Season import helper (transaction-scoped) ──────────────────────────

/// Create a season within an existing transaction.
/// Does NOT commit — caller must commit or rollback.
/// Checks for duplicate season; if found, returns Duplicate error with player count.
pub(crate) fn create_season_tx(
    tx: &rusqlite::Transaction,
    save_id: i64,
    in_game_date: &str,
) -> Result<Season, StorageError> {
    let label = derive_season_label(in_game_date)?;

    // Check for duplicate season in this save
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

// ── Season CRUD ────────────────────────────────────────────────────────

/// Create a season record with auto-derived label.
/// Opens its own transaction and delegates to `create_season_tx`.
pub fn create_season(
    conn: &Connection,
    save_id: i64,
    in_game_date: &str,
) -> Result<Season, StorageError> {
    let tx = conn.unchecked_transaction()?;

    // Verify save exists
    let save_exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1)",
        rusqlite::params![save_id],
        |row| row.get(0),
    )?;
    if !save_exists {
        return Err(StorageError::NotFound("Save not found.".to_string()));
    }

    let season = create_season_tx(&tx, save_id, in_game_date)?;
    tx.commit()?;
    Ok(season)
}

/// List all seasons for a save, ordered by in_game_date ascending.
pub fn list_seasons(conn: &Connection, save_id: i64) -> Result<Vec<Season>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT id, save_id, in_game_date, label, imported_at
         FROM seasons WHERE save_id = ?1
         ORDER BY in_game_date ASC"
    )?;
    let seasons: Vec<Season> = stmt.query_map(rusqlite::params![save_id], |row| {
        Ok(Season {
            id: row.get(0)?,
            save_id: row.get(1)?,
            in_game_date: row.get(2)?,
            label: row.get(3)?,
            imported_at: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;
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
/// All operations are atomic within a single transaction.
pub fn delete_season(conn: &Connection, season_id: i64) -> Result<(), StorageError> {
    let tx = conn.unchecked_transaction()?;

    let save_id: Option<i64> = tx.query_row(
        "SELECT save_id FROM seasons WHERE id = ?1",
        rusqlite::params![season_id],
        |row| row.get(0),
    ).ok();

    let save_id = match save_id {
        Some(sid) => sid,
        None => return Err(StorageError::NotFound("Season not found.".to_string())),
    };

    // Delete player_seasons for this season
    tx.execute(
        "DELETE FROM player_seasons WHERE season_id = ?1",
        rusqlite::params![season_id],
    )?;

    // Delete the season
    tx.execute(
        "DELETE FROM seasons WHERE id = ?1",
        rusqlite::params![season_id],
    )?;

    // Clean up orphaned players (players with no remaining seasons in this save)
    tx.execute(
        "DELETE FROM players WHERE save_id = :save_id AND id NOT IN \
         (SELECT DISTINCT player_id FROM player_seasons \
          JOIN seasons ON player_seasons.season_id = seasons.id \
          WHERE seasons.save_id = :save_id)",
        rusqlite::named_params!{":save_id": save_id},
    )?;

    tx.commit()?;
    Ok(())
}
```

- [ ] **Step 6: Create `src-tauri/src/storage/import.rs`**

```rust
use rusqlite::Connection;

use crate::parser::types::ParsedPlayer;
use super::error::StorageError;
use super::types::{ImportResult, Season};
use super::seasons::create_season_tx;

// ── Season import (core persistence) ──────────────────────────────────

/// Format positions for storage as a readable string.
/// e.g. "AM (L, C), ST (C)"
fn format_positions(positions: &[crate::parser::types::Position]) -> String {
    positions.iter().map(|p| {
        let sides = p.sides.iter().map(|s| match s {
            crate::parser::types::Side::L => "L",
            crate::parser::types::Side::C => "C",
            crate::parser::types::Side::R => "R",
        }).collect::<Vec<_>>().join(", ");
        format!("{:?} ({})", p.role, sides)
    }).collect::<Vec<_>>().join(", ")
}

/// Import a season: creates season record, matches/inserts players, stores JSON blobs.
///
/// Player matching: `(save_id, fm_uid, LOWER(name))` — case-insensitive name.
/// If a player with same UID+name exists, reuse the record (different season = same player).
/// If same UID but different name, create a new player record.
///
/// Returns `ImportResult` with season and player counts.
/// All work happens in a single transaction — rollback on any failure.
pub fn import_season(
    conn: &Connection,
    save_id: i64,
    in_game_date: &str,
    players: Vec<ParsedPlayer>,
) -> Result<ImportResult, StorageError> {
    // Validate non-empty
    if players.is_empty() {
        return Err(StorageError::Validation(
            "Cannot import a season with no players.".to_string(),
        ));
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

    // Begin transaction
    let tx = conn.unchecked_transaction()?;

    // Create season (checks for duplicates internally)
    let season: Season = match create_season_tx(&tx, save_id, in_game_date) {
        Ok(s) => s,
        Err(e) => return Err(e), // No commit, no rollback needed on failure
    };

    let mut new_players = 0usize;
    let mut matched_players = 0usize;

    for player in players {
        // Look up existing player by (save_id, fm_uid, LOWER(name))
        let existing_player_id: Option<i64> = tx.query_row(
            "SELECT id FROM players WHERE save_id = ?1 AND fm_uid = ?2 AND LOWER(name) = LOWER(?3)",
            rusqlite::params![save_id, player.uid as i64, player.name],
            |row| row.get(0),
        ).ok();

        let player_id = if let Some(pid) = existing_player_id {
            matched_players += 1;
            pid
        } else {
            // Insert new player record
            tx.execute(
                "INSERT INTO players (save_id, fm_uid, name) VALUES (?1, ?2, ?3)",
                rusqlite::params![save_id, player.uid as i64, player.name],
            )?;
            new_players += 1;
            tx.last_insert_rowid()
        };

        // Extract queryable columns
        let position_str = format_positions(&player.positions);
        let club = player.club.clone();
        let age = player.age.map(|a| a as i64);
        let nationality = player.nationality.as_ref().map(|n| n.name.clone());
        let minutes = player.minutes.map(|m| m as i64);
        let appearances_started = player.appearances_started.map(|a| a as i64);
        let appearances_sub = player.appearances_sub.map(|a| a as i64);
        let wage_per_week = player.wage.wage_per_week;
        let transfer_value_high = player.transfer_value.high;
        let contract_expires = player.contract_expires.clone();

        // Serialize full ParsedPlayer as JSON blob
        let data_json = serde_json::to_string(&player)
            .map_err(|_| StorageError::Validation("Failed to serialize player data.".to_string()))?;

        // Insert player_season record
        tx.execute(
            "INSERT INTO player_seasons \
             (player_id, season_id, club, age, nationality, position, \
              minutes, appearances_started, appearances_sub, \
              wage_per_week, transfer_value_high, contract_expires, data) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            rusqlite::params![
                player_id, season.id, club, age, nationality, position_str,
                minutes, appearances_started, appearances_sub,
                wage_per_week, transfer_value_high, contract_expires, data_json,
            ],
        )?;
    }

    // Commit transaction
    tx.commit()?;

    Ok(ImportResult {
        season,
        total_players: new_players + matched_players,
        new_players,
        matched_players,
    })
}
```

- [ ] **Step 7: Create `src-tauri/src/storage/retrieval.rs`**

```rust
use rusqlite::Connection;

use crate::parser::types::ParsedPlayer;
use super::error::StorageError;
use super::types::{PlayerSeasonData, Season};

// ── Data retrieval ────────────────────────────────────────────────────

/// Deserialize a database row into PlayerSeasonData.
/// Uses named column references — robust against SELECT column reordering.
/// Handles JSON blob deserialization with graceful degradation (None on failure).
fn row_to_player_season(row: &rusqlite::Row) -> rusqlite::Result<PlayerSeasonData> {
    let data_json: String = row.get("data")?;
    let data = serde_json::from_str::<ParsedPlayer>(&data_json).ok();

    Ok(PlayerSeasonData {
        id: row.get("id")?,
        player_id: row.get("player_id")?,
        season_id: row.get("season_id")?,
        fm_uid: row.get("fm_uid")?,
        player_name: row.get("player_name")?,
        club: row.get("club")?,
        age: row.get("age")?,
        nationality: row.get("nationality")?,
        position: row.get("position")?,
        minutes: row.get("minutes")?,
        appearances_started: row.get("appearances_started")?,
        appearances_sub: row.get("appearances_sub")?,
        wage_per_week: row.get("wage_per_week")?,
        transfer_value_high: row.get("transfer_value_high")?,
        contract_expires: row.get("contract_expires")?,
        data,
    })
}

/// Get all players for a season, ordered by name ascending.
/// Returns an empty Vec for non-existent seasons.
pub fn get_players_for_season(
    conn: &Connection,
    season_id: i64,
) -> Result<Vec<PlayerSeasonData>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT ps.id, ps.player_id, ps.season_id, p.fm_uid, p.name AS player_name,
                ps.club, ps.age, ps.nationality, ps.position, ps.minutes,
                ps.appearances_started, ps.appearances_sub, ps.wage_per_week,
                ps.transfer_value_high, ps.data, ps.contract_expires
         FROM player_seasons ps
         JOIN players p ON ps.player_id = p.id
         WHERE ps.season_id = ?1
         ORDER BY p.name ASC",
    )?;

    let players: Vec<PlayerSeasonData> = stmt
        .query_map(rusqlite::params![season_id], |row| row_to_player_season(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(players)
}

/// Get a player's entire career across all seasons in a save, ordered by
/// in_game_date ascending (earliest season first).
/// Returns an empty Vec for non-existent player or save.
pub fn get_player_career(
    conn: &Connection,
    save_id: i64,
    player_id: i64,
) -> Result<Vec<PlayerSeasonData>, StorageError> {
    let mut stmt = conn.prepare(
        "SELECT ps.id, ps.player_id, ps.season_id, p.fm_uid, p.name AS player_name,
                ps.club, ps.age, ps.nationality, ps.position, ps.minutes,
                ps.appearances_started, ps.appearances_sub, ps.wage_per_week,
                ps.transfer_value_high, ps.data, ps.contract_expires
         FROM player_seasons ps
         JOIN players p ON ps.player_id = p.id
         JOIN seasons s ON ps.season_id = s.id
         WHERE p.save_id = ?1 AND ps.player_id = ?2
         ORDER BY s.in_game_date ASC",
    )?;

    let career: Vec<PlayerSeasonData> = stmt
        .query_map(rusqlite::params![save_id, player_id], |row| row_to_player_season(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(career)
}

/// Get the most recent season for a save (by in_game_date descending).
/// Returns None if no seasons exist for the save.
pub fn get_latest_season(conn: &Connection, save_id: i64) -> Result<Option<Season>, StorageError> {
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

- [ ] **Step 8: Replace `src-tauri/src/storage/mod.rs` with re-exports + tests**

The new `mod.rs` contains only: module declarations, re-exports, and the entire test module from the original file.

```rust
mod error;
mod schema;
mod types;
mod saves;
mod seasons;
mod import;
mod retrieval;

// ── Public API re-exports ──────────────────────────────────────────────

pub use error::StorageError;
pub use types::{DbState, Save, Season, ImportResult, PlayerSeasonData};
pub use schema::{init_db, init_db_test};
pub use saves::{create_save, list_saves, rename_save, delete_save};
pub use seasons::{create_season, list_seasons, rename_season, delete_season, derive_season_label};
pub use import::import_season;
pub use retrieval::{get_players_for_season, get_player_career, get_latest_season};

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    // ... all existing test code copied verbatim from the original mod.rs ...
    //
    // IMPORTANT: Update the setup_test_db() helper to use init_db_test instead of init_schema:
    //
    // BEFORE:
    //   fn setup_test_db() -> Connection {
    //       let conn = Connection::open_in_memory().unwrap();
    //       init_schema(&conn).unwrap();
    //       conn
    //   }
    //
    // AFTER:
    //   fn setup_test_db() -> Connection {
    //       let conn = Connection::open_in_memory().unwrap();
    //       init_db_test(&conn).unwrap();
    //       conn
    //   }
    //
    // All other test code remains unchanged. `use super::*` gives access to all re-exported items.
}
```

**The test module is large (~945 lines).** Copy it verbatim from the original file with one change: replace `init_schema(&conn)` with `init_db_test(&conn)` in the `setup_test_db()` helper (since `init_schema` is now private to `schema.rs`).

- [ ] **Step 9: Run all tests**

```bash
cd src-tauri && cargo test
```

Expected: All 219 tests pass. The re-exports in `mod.rs` ensure all external callers (`commands/storage.rs`, `tests/integration_storage.rs`, `tests/edge_case_storage.rs`) see the same public API.

- [ ] **Step 10: Verify no production file exceeds 200 lines**

```bash
cd src-tauri && for f in src/storage/error.rs src/storage/types.rs src/storage/schema.rs src/storage/saves.rs src/storage/seasons.rs src/storage/import.rs src/storage/retrieval.rs; do echo "$f: $(wc -l < $f) lines"; done
```

Expected (approximate):
- `error.rs`: ~35 lines
- `types.rs`: ~55 lines
- `schema.rs`: ~70 lines
- `saves.rs`: ~110 lines
- `seasons.rs`: ~140 lines
- `import.rs`: ~115 lines
- `retrieval.rs`: ~85 lines

- [ ] **Step 11: Verify mod.rs is re-exports only (excluding tests)**

```bash
cd src-tauri && head -20 src/storage/mod.rs
```

Expected: Module declarations and re-exports only — no function implementations.

## Dependencies

- Tasks 01-05 should be completed first. This task captures their fixes in the split.

## Success Criteria

- All 219 tests pass without modification (except `setup_test_db` using `init_db_test`)
- No production file exceeds 200 lines
- `mod.rs` contains only module declarations, re-exports, and tests
- `grep -rn "save_import" src-tauri/src/storage/` returns no matches (dead code not carried forward)
- External callers (`commands/storage.rs`, integration tests) require no changes

## Tests

### Test 1: All existing tests pass

**What to test:** The split preserves all behavior.

**Feasibility:** ✅ `cargo test` runs all 219 tests.

### Test 2: External API unchanged

**What to test:** `commands/storage.rs` and `tests/integration_storage.rs` compile and work without changes.

**Feasibility:** ✅ `cargo build` + `cargo test` verifies. The re-exports provide the same public surface.
