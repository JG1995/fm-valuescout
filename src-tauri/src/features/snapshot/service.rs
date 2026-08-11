use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};

use crate::features::academy::service as academy_service;

pub const DEFAULT_SAVE_NAME: &str = "Default save";
pub const MAX_SAVE_NAME_LEN: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSummary {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
    pub created_at_utc: String,
    pub updated_at_utc: String,
}

pub fn validate_save_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Save name must not be empty".to_string());
    }
    if trimmed.chars().count() > MAX_SAVE_NAME_LEN {
        return Err(format!(
            "Save name must be at most {} characters",
            MAX_SAVE_NAME_LEN
        ));
    }

    Ok(trimmed.to_string())
}

pub fn ensure_default_save(conn: &Connection) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO saves (name, is_active)
         SELECT ?1, 1
         WHERE NOT EXISTS (SELECT 1 FROM saves)",
        params![DEFAULT_SAVE_NAME],
    )
    .map_err(|error| error.to_string())?;

    let save_id: i64 = tx
        .query_row(
            "SELECT id FROM saves WHERE is_active = 1 LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    academy_service::ensure_baseline_class(&tx, save_id)?;
    tx.commit().map_err(|error| error.to_string())?;

    Ok(())
}

pub fn active_save_id(conn: &Connection) -> Result<i64, String> {
    ensure_default_save(conn)?;
    conn.query_row(
        "SELECT id FROM saves WHERE is_active = 1 LIMIT 1",
        [],
        |row| row.get(0),
    )
    .map_err(|error| error.to_string())
}

pub fn list_saves(conn: &Connection) -> Result<Vec<SaveSummary>, String> {
    ensure_default_save(conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, name, is_active, created_at_utc, updated_at_utc
             FROM saves
             ORDER BY id",
        )
        .map_err(|error| error.to_string())?;

    let saves = stmt
        .query_map([], map_save_row)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(saves)
}

pub fn create_save(conn: &Connection, name: &str) -> Result<SaveSummary, String> {
    let name = validate_save_name(name)?;
    ensure_default_save(conn)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    tx.execute(
        "INSERT INTO saves (name, is_active) VALUES (?1, 0)",
        params![name],
    )
    .map_err(|error| error.to_string())?;

    let id = tx.last_insert_rowid();
    academy_service::ensure_baseline_class(&tx, id)?;
    tx.commit().map_err(|error| error.to_string())?;
    get_save_by_id(conn, id)
}

pub fn rename_save(conn: &Connection, save_id: i64, name: &str) -> Result<SaveSummary, String> {
    let name = validate_save_name(name)?;

    let rows = conn
        .execute(
            "UPDATE saves
             SET name = ?1,
                 updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
             WHERE id = ?2",
            params![name, save_id],
        )
        .map_err(|error| error.to_string())?;

    if rows == 0 {
        return Err(format!("Save {save_id} not found"));
    }

    get_save_by_id(conn, save_id)
}

pub fn set_active_save(conn: &mut Connection, save_id: i64) -> Result<SaveSummary, String> {
    let tx = conn.transaction().map_err(|error| error.to_string())?;

    set_active_save_in_transaction(&tx, save_id)?;
    tx.commit().map_err(|error| error.to_string())?;

    get_save_by_id(conn, save_id)
}

pub(crate) fn select_current_snapshot(
    tx: &Transaction<'_>,
    save_id: i64,
) -> Result<Option<i64>, String> {
    let current_snapshot_id = tx
        .query_row(
            "SELECT id
             FROM snapshots
             WHERE save_id = ?1
             ORDER BY game_date IS NULL, game_date DESC, loaded_at_utc DESC, id DESC
             LIMIT 1",
            params![save_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    tx.execute(
        "UPDATE snapshots SET is_current = 0 WHERE save_id = ?1 AND is_current = 1",
        params![save_id],
    )
    .map_err(|error| error.to_string())?;

    if let Some(snapshot_id) = current_snapshot_id {
        tx.execute(
            "UPDATE snapshots SET is_current = 1 WHERE id = ?1",
            params![snapshot_id],
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(current_snapshot_id)
}

fn set_active_save_in_transaction(tx: &Transaction<'_>, save_id: i64) -> Result<(), String> {
    let exists: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM saves WHERE id = ?1",
            params![save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    if exists == 0 {
        return Err(format!("Save {save_id} not found"));
    }

    tx.execute(
        "UPDATE saves
         SET is_active = 0,
             updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE is_active = 1",
        [],
    )
    .map_err(|error| error.to_string())?;

    tx.execute(
        "UPDATE saves
         SET is_active = 1,
             updated_at_utc = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
         WHERE id = ?1",
        params![save_id],
    )
    .map_err(|error| error.to_string())?;

    Ok(())
}

fn get_save_by_id(conn: &Connection, save_id: i64) -> Result<SaveSummary, String> {
    conn.query_row(
        "SELECT id, name, is_active, created_at_utc, updated_at_utc
         FROM saves
         WHERE id = ?1",
        params![save_id],
        map_save_row,
    )
    .map_err(|error| error.to_string())
}

fn map_save_row(row: &Row<'_>) -> rusqlite::Result<SaveSummary> {
    Ok(SaveSummary {
        id: row.get(0)?,
        name: row.get(1)?,
        is_active: row.get::<_, i32>(2)? == 1,
        created_at_utc: row.get(3)?,
        updated_at_utc: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use std::path::Path;

    fn active_save_count(conn: &Connection) -> Result<i64, String> {
        conn.query_row(
            "SELECT COUNT(*) FROM saves WHERE is_active = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())
    }

    fn save_count(conn: &Connection) -> Result<i64, String> {
        conn.query_row("SELECT COUNT(*) FROM saves", [], |row| row.get(0))
            .map_err(|error| error.to_string())
    }

    fn open_migrated(db_path: &Path) -> Connection {
        let conn = Connection::open(db_path).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    #[test]
    fn empty_database_gets_one_active_default_save_on_list() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("default-save.db"));

        let saves = list_saves(&conn).expect("list saves");

        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].name, DEFAULT_SAVE_NAME);
        assert!(saves[0].is_active);
        assert_eq!(active_save_count(&conn).expect("count active"), 1);
    }

    #[test]
    fn create_on_empty_database_seeds_default_and_adds_inactive_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("create-first.db"));

        let created = create_save(&conn, "My career").expect("create save on empty db");

        assert_eq!(created.name, "My career");
        assert!(!created.is_active);

        let saves = list_saves(&conn).expect("list saves after create");
        assert_eq!(saves.len(), 2);
        assert_eq!(active_save_count(&conn).expect("count active"), 1);

        let default_save = saves
            .iter()
            .find(|save| save.name == DEFAULT_SAVE_NAME)
            .expect("default save");
        assert!(default_save.is_active);
    }

    #[test]
    fn create_rename_and_set_active_preserve_exactly_one_active_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("save-crud.db"));

        let default_save = list_saves(&conn)
            .expect("list saves")
            .into_iter()
            .find(|save| save.is_active)
            .expect("default active save");

        let second_save = create_save(&conn, "  Second save  ").expect("create save");
        assert_eq!(second_save.name, "Second save");
        assert!(!second_save.is_active);

        let renamed = rename_save(&conn, second_save.id, "Renamed save").expect("rename save");
        assert_eq!(renamed.name, "Renamed save");

        let activated = set_active_save(&mut conn, second_save.id).expect("set active save");
        assert!(activated.is_active);

        let saves = list_saves(&conn).expect("list saves after switch");
        assert_eq!(saves.len(), 2);
        assert_eq!(active_save_count(&conn).expect("count active"), 1);

        let active_ids: Vec<i64> = saves
            .iter()
            .filter(|save| save.is_active)
            .map(|save| save.id)
            .collect();
        assert_eq!(active_ids, vec![second_save.id]);

        let previous_default = saves
            .iter()
            .find(|save| save.id == default_save.id)
            .expect("original default save");
        assert!(!previous_default.is_active);
    }

    #[test]
    fn rejects_empty_save_name_without_seeding_database() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("reject-empty.db"));

        let error = create_save(&conn, "   ").expect_err("reject empty name");

        assert!(error.contains("must not be empty"));
        assert_eq!(save_count(&conn).expect("count saves"), 0);
    }

    #[test]
    fn set_active_save_fails_for_unknown_id() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("unknown-save.db"));
        list_saves(&conn).expect("seed default save");

        let error = set_active_save(&mut conn, 999).expect_err("reject unknown save");

        assert!(error.contains("not found"));
        assert_eq!(active_save_count(&conn).expect("count active"), 1);
    }
}
