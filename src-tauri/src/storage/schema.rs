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
