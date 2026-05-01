use rusqlite::Connection;

use crate::parser::types::ParsedPlayer;
use super::error::StorageError;
use super::types::{PlayerSeasonData, Season};

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
/// JSON blobs are deserialized with graceful degradation — rows with invalid JSON
/// are skipped (data field will be None for those rows).
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
