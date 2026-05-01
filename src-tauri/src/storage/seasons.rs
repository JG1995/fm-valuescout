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
