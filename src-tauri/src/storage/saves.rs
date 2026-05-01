use rusqlite::Connection;

use super::error::StorageError;
use super::types::Save;

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
