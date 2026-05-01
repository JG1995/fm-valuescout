use rusqlite::Connection;

use crate::parser::types::ParsedPlayer;
use super::error::StorageError;
use super::types::ImportResult;
use super::seasons::create_season_tx;

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
    let season = match create_season_tx(&tx, save_id, in_game_date) {
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
