use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};

use crate::features::{
    academy::service as academy_service, memory_read::dump_validation::canonical_game_date,
    player_metrics::potential_scores, staff::scoring as staff_scoring,
};

use super::query::get_snapshot_metadata;

pub const DEFAULT_SAVE_NAME: &str = "Default save";
pub const MAX_SAVE_NAME_LEN: usize = 100;
pub const MAX_SNAPSHOT_NAME_LEN: usize = MAX_SAVE_NAME_LEN;
pub(crate) const SNAPSHOT_ORDER_BY: &str =
    "game_date IS NULL, game_date DESC, loaded_at_utc DESC, id DESC";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveSummary {
    pub id: i64,
    pub context_token: String,
    pub name: String,
    pub is_active: bool,
    pub created_at_utc: String,
    pub updated_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotMetadata {
    pub id: i64,
    pub context_token: String,
    pub save_id: i64,
    pub custom_name: Option<String>,
    pub game_date: Option<String>,
    pub game_date_source: String,
    pub player_count: i64,
    pub loaded_at_utc: String,
    pub is_current: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SaveContext {
    pub(crate) id: i64,
    pub(crate) context_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotDeleteResult {
    pub deleted_snapshot_id: i64,
    pub save_id: i64,
    pub current_snapshot_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotGameDateUpdateResult {
    pub snapshot: SnapshotMetadata,
    pub previous_current_snapshot_id: Option<i64>,
    pub current_snapshot_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveDeleteResult {
    pub deleted_save_id: i64,
    pub deleted_was_active: bool,
    pub active_save: SaveSummary,
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

pub fn validate_snapshot_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Snapshot name must not be empty".to_string());
    }
    if trimmed.chars().count() > MAX_SNAPSHOT_NAME_LEN {
        return Err(format!(
            "Snapshot name must be at most {} characters",
            MAX_SNAPSHOT_NAME_LEN
        ));
    }

    Ok(trimmed.to_string())
}

fn create_default_save_in_transaction(tx: &Transaction<'_>) -> Result<i64, String> {
    tx.execute(
        "INSERT INTO saves (name, is_active) VALUES (?1, 1)",
        [DEFAULT_SAVE_NAME],
    )
    .map_err(|error| error.to_string())?;
    let save_id = tx.last_insert_rowid();
    academy_service::ensure_baseline_class(tx, save_id)?;
    Ok(save_id)
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
    capture_active_save_context(conn).map(|context| context.id)
}

pub(crate) fn capture_active_save_context(conn: &Connection) -> Result<SaveContext, String> {
    ensure_default_save(conn)?;
    conn.query_row(
        "SELECT id, context_token FROM saves WHERE is_active = 1 LIMIT 1",
        [],
        |row| {
            Ok(SaveContext {
                id: row.get(0)?,
                context_token: row.get(1)?,
            })
        },
    )
    .map_err(|error| error.to_string())
}

pub(crate) fn save_context_for_id(conn: &Connection, save_id: i64) -> Result<SaveContext, String> {
    conn.query_row(
        "SELECT id, context_token FROM saves WHERE id = ?1",
        [save_id],
        |row| {
            Ok(SaveContext {
                id: row.get(0)?,
                context_token: row.get(1)?,
            })
        },
    )
    .optional()
    .map_err(|error| error.to_string())?
    .ok_or_else(|| format!("Save {save_id} not found"))
}

pub(crate) fn ensure_save_context(
    tx: &Transaction<'_>,
    context: &SaveContext,
) -> Result<(), String> {
    let exists: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1)",
            params![context.id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !exists {
        return Err(format!("Save {} not found", context.id));
    }

    let matches: bool = tx
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM saves
                WHERE id = ?1 AND context_token = ?2
            )",
            params![context.id, context.context_token],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;

    matches
        .then_some(())
        .ok_or_else(|| "Save changed or no longer exists".to_string())
}

pub fn list_saves(conn: &Connection) -> Result<Vec<SaveSummary>, String> {
    ensure_default_save(conn)?;

    let mut stmt = conn
        .prepare(
            "SELECT id, context_token, name, is_active, created_at_utc, updated_at_utc
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

pub fn list_snapshot_metadata(
    conn: &Connection,
    requested_save_id: Option<i64>,
) -> Result<Vec<SnapshotMetadata>, String> {
    super::query::list_snapshot_metadata(conn, requested_save_id)
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

pub fn rename_snapshot(
    conn: &Connection,
    snapshot_id: i64,
    context_token: &str,
    custom_name: Option<&str>,
) -> Result<SnapshotMetadata, String> {
    let custom_name = custom_name.map(validate_snapshot_name).transpose()?;
    let rows = conn
        .execute(
            "UPDATE snapshots
             SET custom_name = ?1
             WHERE id = ?2 AND context_token = ?3",
            params![custom_name, snapshot_id, context_token],
        )
        .map_err(|error| error.to_string())?;
    if rows == 0 {
        return Err("Snapshot changed or no longer exists".to_string());
    }

    get_snapshot_metadata(conn, snapshot_id)
}

pub fn update_snapshot_game_date(
    conn: &mut Connection,
    snapshot_id: i64,
    context_token: &str,
    game_date: &str,
) -> Result<SnapshotGameDateUpdateResult, String> {
    if !canonical_game_date(game_date) {
        return Err("Game date must be a valid date in YYYY-MM-DD format".to_string());
    }

    let tx = conn.transaction().map_err(|error| error.to_string())?;
    let save_id: i64 = tx
        .query_row(
            "SELECT save_id FROM snapshots WHERE id = ?1 AND context_token = ?2",
            params![snapshot_id, context_token],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Snapshot changed or no longer exists".to_string())?;
    let previous_current_snapshot_id: Option<i64> = tx
        .query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            params![save_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    tx.execute(
        "UPDATE snapshots SET game_date = ?1 WHERE id = ?2 AND context_token = ?3",
        params![game_date, snapshot_id, context_token],
    )
    .map_err(|error| error.to_string())?;

    let current_snapshot_id = select_current_snapshot(&tx, save_id)?;
    let snapshot = get_snapshot_metadata(&tx, snapshot_id)?;

    tx.commit().map_err(|error| error.to_string())?;
    Ok(SnapshotGameDateUpdateResult {
        snapshot,
        previous_current_snapshot_id,
        current_snapshot_id,
    })
}

pub fn delete_snapshot(
    conn: &mut Connection,
    snapshot_id: i64,
    context_token: &str,
) -> Result<SnapshotDeleteResult, String> {
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    let snapshot: Option<(i64, bool)> = tx
        .query_row(
            "SELECT save_id, is_current
             FROM snapshots
             WHERE id = ?1 AND context_token = ?2",
            params![snapshot_id, context_token],
            |row| Ok((row.get(0)?, row.get::<_, i32>(1)? == 1)),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some((save_id, was_current)) = snapshot else {
        return Err("Snapshot changed or no longer exists".to_string());
    };

    tx.execute(
        "DELETE FROM snapshots WHERE id = ?1 AND context_token = ?2",
        params![snapshot_id, context_token],
    )
    .map_err(|error| error.to_string())?;

    let current_snapshot_id = if was_current {
        let next_snapshot_id = select_current_snapshot(&tx, save_id)?;
        if let Some(next_snapshot_id) = next_snapshot_id {
            let (game_date, game_date_source): (Option<String>, String) = tx
                .query_row(
                    "SELECT game_date, game_date_source FROM snapshots WHERE id = ?1",
                    [next_snapshot_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .map_err(|error| error.to_string())?;
            academy_service::ensure_class_for_game_date(
                &tx,
                save_id,
                game_date.as_deref(),
                &game_date_source,
            )?;
        }
        next_snapshot_id
    } else {
        tx.query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            [save_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?
    };

    tx.commit().map_err(|error| error.to_string())?;
    Ok(SnapshotDeleteResult {
        deleted_snapshot_id: snapshot_id,
        save_id,
        current_snapshot_id,
    })
}

pub fn delete_save(
    conn: &mut Connection,
    save_id: i64,
    context_token: &str,
) -> Result<SaveDeleteResult, String> {
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    let is_active: Option<bool> = tx
        .query_row(
            "SELECT is_active FROM saves WHERE id = ?1 AND context_token = ?2",
            params![save_id, context_token],
            |row| Ok(row.get::<_, i32>(0)? == 1),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some(is_active) = is_active else {
        return Err("Save changed or no longer exists".to_string());
    };

    tx.execute(
        "DELETE FROM saves WHERE id = ?1 AND context_token = ?2",
        params![save_id, context_token],
    )
    .map_err(|error| error.to_string())?;

    let active_save_id = if is_active {
        let fallback_save_id: Option<i64> = tx
            .query_row("SELECT id FROM saves ORDER BY id LIMIT 1", [], |row| {
                row.get(0)
            })
            .optional()
            .map_err(|error| error.to_string())?;
        match fallback_save_id {
            Some(fallback_save_id) => {
                set_active_save_in_transaction(&tx, fallback_save_id)?;
                fallback_save_id
            }
            None => create_default_save_in_transaction(&tx)?,
        }
    } else {
        tx.query_row(
            "SELECT id FROM saves WHERE is_active = 1 LIMIT 1",
            [],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?
    };

    tx.commit().map_err(|error| error.to_string())?;
    Ok(SaveDeleteResult {
        deleted_save_id: save_id,
        deleted_was_active: is_active,
        active_save: get_save_by_id(conn, active_save_id)?,
    })
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
    let select_current_sql = format!(
        "SELECT id
         FROM snapshots
         WHERE save_id = ?1
         ORDER BY {SNAPSHOT_ORDER_BY}
         LIMIT 1"
    );
    let current_snapshot_id = tx
        .query_row(&select_current_sql, params![save_id], |row| row.get(0))
        .optional()
        .map_err(|error| error.to_string())?;
    let previous_snapshot_id = tx
        .query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
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

    potential_scores::reconcile_current_selection(
        tx,
        save_id,
        previous_snapshot_id,
        current_snapshot_id,
    )?;
    staff_scoring::reconcile_current_selection(
        tx,
        save_id,
        previous_snapshot_id,
        current_snapshot_id,
    )?;

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
        "SELECT id, context_token, name, is_active, created_at_utc, updated_at_utc
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
        context_token: row.get(1)?,
        name: row.get(2)?,
        is_active: row.get::<_, i32>(3)? == 1,
        created_at_utc: row.get(4)?,
        updated_at_utc: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::{
        player_metrics::potential_scores::PROJECTION_MODEL_VERSION,
        scoring::catalog::DUMP_ATTRIBUTE_KEYS,
    };
    use rusqlite::params;
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

    fn insert_snapshot(
        conn: &Connection,
        save_id: i64,
        game_date: Option<&str>,
        loaded_at_utc: &str,
    ) -> i64 {
        conn.execute(
            "INSERT INTO snapshots (
                save_id, is_current, schema_version, generated_at_utc, game_version,
                supported_game_version, bridge_version, protocol_version, game_date,
                game_date_source, scan_truncated, max_accepted, player_count, loaded_at_utc
             ) VALUES (
                ?1, 0, 6, '2026-08-11T10:00:00.000Z', '26.3.2', '26.3', '0.1.0', 1, ?2,
                'memory', 0, NULL, 1, ?3
             )",
            params![save_id, game_date, loaded_at_utc],
        )
        .expect("insert snapshot");
        conn.last_insert_rowid()
    }

    fn insert_player(conn: &Connection, snapshot_id: i64, uid: i64) {
        let attributes = DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), serde_json::Value::from(10)))
            .collect::<serde_json::Map<_, _>>();
        conn.execute(
            "INSERT INTO players (
                snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year, age,
                nationalities_json, preferred_foot, positions_json, attributes_json,
                hidden_attributes_json, personality_json
             ) VALUES (?1, ?2, 100, 150, 'History player', 2000, 1, 20, '[]', 'Right',
                       '{\"ST\":20}', ?3, '{}', '{}')",
            params![
                snapshot_id,
                uid,
                serde_json::to_string(&attributes).expect("serialize player attributes")
            ],
        )
        .expect("insert player");
    }

    fn insert_staff_record(conn: &Connection, snapshot_id: i64, uid: i64) {
        conn.execute(
            "INSERT INTO staff (
                snapshot_id, uid, name, birth_year, birth_day_of_year, age,
                nationalities_json, nation_uid, gender, ca, pa, staff_attributes_json,
                job_id, weekly_wage_gbp, contract_expiry_year, contract_expiry_day_of_year,
                club, division
             ) VALUES (
                ?1, ?2, 'Compact staff', 1990, 1, 35, '[]', NULL, 'unknown', 100, 140, ?3,
                NULL, NULL, NULL, NULL, NULL, NULL
             )",
            params![
                snapshot_id,
                uid,
                crate::features::staff::scoring::test_support::all_ten_attributes_json()
            ],
        )
        .expect("insert staff record");
    }

    fn potential_state(conn: &Connection, snapshot_id: i64) -> (Option<String>, Option<i64>, i64) {
        let fields = conn
            .query_row(
                "SELECT potential_attributes_json, potential_projection_model_version
                 FROM players WHERE snapshot_id = ?1 ORDER BY uid LIMIT 1",
                [snapshot_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read projected player fields");
        let compact_count =
            crate::features::player_metrics::compact::test_support::count_rows(conn, snapshot_id);
        (fields.0, fields.1, compact_count)
    }

    fn assert_complete_potential_state(conn: &Connection, snapshot_id: i64) {
        let state = potential_state(conn, snapshot_id);
        assert!(state.0.is_some());
        assert_eq!(state.1, Some(PROJECTION_MODEL_VERSION));
        assert_eq!(state.2, 1);
    }

    fn assert_empty_potential_state(conn: &Connection, snapshot_id: i64) {
        assert_eq!(potential_state(conn, snapshot_id), (None, None, 0));
    }

    fn compact_row_count(conn: &Connection, snapshot_id: i64) -> i64 {
        crate::features::player_metrics::compact::test_support::count_rows(conn, snapshot_id)
    }

    fn staff_compact_row(
        conn: &Connection,
        snapshot_id: i64,
        uid: i64,
    ) -> Option<crate::features::staff::scoring::test_support::CompactStaffRowShape> {
        crate::features::staff::scoring::test_support::read_row(conn, snapshot_id, uid)
    }

    fn staff_compact_row_count(conn: &Connection, snapshot_id: i64) -> i64 {
        crate::features::staff::scoring::test_support::count_rows(conn, snapshot_id)
    }

    fn deny_potential_writes(conn: &Connection) {
        conn.execute_batch(
            "CREATE TRIGGER deny_projected_player_updates
             BEFORE UPDATE OF potential_attributes_json, potential_projection_model_version ON players
             BEGIN SELECT RAISE(ABORT, 'potential player writes are forbidden'); END;
             CREATE TRIGGER deny_compact_row_inserts
             BEFORE INSERT ON player_role_metrics
             BEGIN SELECT RAISE(ABORT, 'compact row writes are forbidden'); END;
             CREATE TRIGGER deny_compact_row_deletes
             BEFORE DELETE ON player_role_metrics
             BEGIN SELECT RAISE(ABORT, 'compact row writes are forbidden'); END;
             CREATE TRIGGER deny_staff_compact_row_inserts
             BEFORE INSERT ON staff_role_metrics
             BEGIN SELECT RAISE(ABORT, 'staff compact row writes are forbidden'); END;
             CREATE TRIGGER deny_staff_compact_row_deletes
             BEFORE DELETE ON staff_role_metrics
             BEGIN SELECT RAISE(ABORT, 'staff compact row writes are forbidden'); END;",
        )
        .expect("deny potential writes");
    }

    fn snapshot_token(conn: &Connection, snapshot_id: i64) -> String {
        conn.query_row(
            "SELECT context_token FROM snapshots WHERE id = ?1",
            [snapshot_id],
            |row| row.get(0),
        )
        .expect("read snapshot token")
    }

    fn save_token(conn: &Connection, save_id: i64) -> String {
        conn.query_row(
            "SELECT context_token FROM saves WHERE id = ?1",
            [save_id],
            |row| row.get(0),
        )
        .expect("read save token")
    }

    fn current_snapshot_id(conn: &Connection, save_id: i64) -> Option<i64> {
        conn.query_row(
            "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
            [save_id],
            |row| row.get(0),
        )
        .optional()
        .expect("read current snapshot")
    }

    fn select_current_snapshot_for_test(conn: &mut Connection, save_id: i64) -> Option<i64> {
        let tx = conn.transaction().expect("start selector transaction");
        let selected = select_current_snapshot(&tx, save_id).expect("select current snapshot");
        tx.commit().expect("commit selector transaction");
        selected
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

    #[test]
    fn snapshot_metadata_uses_the_shared_date_order_for_one_requested_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("snapshot-metadata.db"));
        let active_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let older = insert_snapshot(
            &conn,
            active_save.id,
            Some("2026-06-01"),
            "2026-08-11T10:00:00.000Z",
        );
        let equal_date = insert_snapshot(
            &conn,
            active_save.id,
            Some("2026-06-01"),
            "2026-08-11T11:00:00.000Z",
        );
        let latest = insert_snapshot(
            &conn,
            active_save.id,
            Some("2027-01-01"),
            "2026-08-11T09:00:00.000Z",
        );
        let undated = insert_snapshot(&conn, active_save.id, None, "2026-08-11T12:00:00.000Z");
        assert_eq!(
            select_current_snapshot_for_test(&mut conn, active_save.id),
            Some(latest)
        );

        let other_save = create_save(&conn, "Other save").expect("create other save");
        let other_snapshot = insert_snapshot(
            &conn,
            other_save.id,
            Some("2030-01-01"),
            "2026-08-11T13:00:00.000Z",
        );

        let snapshots = list_snapshot_metadata(&conn, Some(active_save.id))
            .expect("list requested save snapshots");

        assert_eq!(
            snapshots
                .iter()
                .map(|snapshot| snapshot.id)
                .collect::<Vec<_>>(),
            vec![latest, equal_date, older, undated]
        );
        assert!(snapshots[0].is_current);
        assert!(snapshots
            .iter()
            .all(|snapshot| snapshot.save_id == active_save.id));
        assert_ne!(
            snapshots[0].context_token,
            snapshot_token(&conn, other_snapshot)
        );
    }

    #[test]
    fn renaming_a_snapshot_is_token_bound_and_does_not_change_its_order_or_owner() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("rename-snapshot.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let older = insert_snapshot(
            &conn,
            save.id,
            Some("2026-01-01"),
            "2026-08-11T10:00:00.000Z",
        );
        let target = insert_snapshot(
            &conn,
            save.id,
            Some("2026-02-01"),
            "2026-08-11T11:00:00.000Z",
        );
        select_current_snapshot_for_test(&mut conn, save.id);
        let target_token = snapshot_token(&conn, target);

        let renamed = rename_snapshot(&conn, target, &target_token, Some("  First window  "))
            .expect("rename snapshot");

        assert_eq!(renamed.custom_name.as_deref(), Some("First window"));
        assert_eq!(renamed.save_id, save.id);
        assert_eq!(
            list_snapshot_metadata(&conn, Some(save.id))
                .expect("list renamed snapshots")
                .iter()
                .map(|snapshot| snapshot.id)
                .collect::<Vec<_>>(),
            vec![target, older]
        );
        assert!(
            rename_snapshot(&conn, target, "stale-token", Some("Wrong target"))
                .expect_err("reject stale token")
                .contains("changed")
        );
        assert!(rename_snapshot(&conn, target, &target_token, Some("   "))
            .expect_err("reject empty custom name")
            .contains("must not be empty"));
        let too_long_name = "x".repeat(MAX_SNAPSHOT_NAME_LEN + 1);
        assert!(
            rename_snapshot(&conn, target, &target_token, Some(&too_long_name))
                .expect_err("reject long custom name")
                .contains("at most")
        );
    }

    #[test]
    fn deleting_a_noncurrent_snapshot_cascades_its_snapshot_data_and_preserves_save_data() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("delete-historical-snapshot.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let historical = insert_snapshot(
            &conn,
            save.id,
            Some("2026-01-01"),
            "2026-08-11T10:00:00.000Z",
        );
        let current = insert_snapshot(
            &conn,
            save.id,
            Some("2026-02-01"),
            "2026-08-11T11:00:00.000Z",
        );
        insert_player(&conn, historical, 77);
        insert_player(&conn, current, 77);
        insert_staff_record(&conn, historical, 77);
        insert_staff_record(&conn, current, 77);
        conn.execute(
            "INSERT INTO player_moneyball_stats (snapshot_id, player_uid, statistics_json)
             VALUES (?1, 77, '{}')",
            [historical],
        )
        .expect("insert historical Moneyball row");
        conn.execute(
            "INSERT INTO managed_club_settings (save_id, club_name) VALUES (?1, 'History FC')",
            [save.id],
        )
        .expect("insert planner setting");
        conn.execute(
            "INSERT INTO player_youth_career_stats (save_id, player_uid, career_appearances)
             VALUES (?1, 77, 3)",
            [save.id],
        )
        .expect("insert youth enrichment");
        let academy_class_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM academy_classes WHERE save_id = ?1",
                [save.id],
                |row| row.get(0),
            )
            .expect("count academy classes before deletion");
        select_current_snapshot_for_test(&mut conn, save.id);
        assert_empty_potential_state(&conn, historical);
        assert_complete_potential_state(&conn, current);
        assert_eq!(compact_row_count(&conn, historical), 0);
        assert_eq!(compact_row_count(&conn, current), 1);
        assert_eq!(staff_compact_row_count(&conn, historical), 0);
        assert_eq!(staff_compact_row_count(&conn, current), 1);
        let current_potential_state = potential_state(&conn, current);
        deny_potential_writes(&conn);

        let historical_token = snapshot_token(&conn, historical);
        let result = delete_snapshot(&mut conn, historical, &historical_token)
            .expect("delete historical snapshot");

        assert_eq!(result.current_snapshot_id, Some(current));
        assert_eq!(current_snapshot_id(&conn, save.id), Some(current));
        assert_eq!(potential_state(&conn, current), current_potential_state);
        assert_eq!(compact_row_count(&conn, current), 1);
        assert_eq!(compact_row_count(&conn, historical), 0);
        assert_eq!(staff_compact_row_count(&conn, current), 1);
        assert_eq!(staff_compact_row_count(&conn, historical), 0);
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM players WHERE snapshot_id = ?1",
                [historical],
                |row| row.get::<_, i64>(0),
            )
            .expect("count cascaded players"),
            0
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM player_moneyball_stats WHERE snapshot_id = ?1",
                [historical],
                |row| row.get::<_, i64>(0),
            )
            .expect("count cascaded Moneyball rows"),
            0
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM managed_club_settings", [], |row| row
                .get::<_, i64>(
                0
            ))
            .expect("count retained planner settings"),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM player_youth_career_stats",
                [],
                |row| row.get::<_, i64>(0)
            )
            .expect("count retained youth enrichment"),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM academy_classes WHERE save_id = ?1",
                [save.id],
                |row| row.get::<_, i64>(0),
            )
            .expect("count retained academy classes"),
            academy_class_count
        );
    }

    #[test]
    fn deleting_the_current_snapshot_promotes_retained_club_dna_rows_and_materializes_potential_scores(
    ) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("promote-snapshot.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        conn.execute(
            "INSERT INTO club_dna_definitions (save_id, attribute_ids_json)
             VALUES (?1, '[\"attr.Acceleration\"]')",
            [save.id],
        )
        .expect("set Club DNA definition");
        let promoted = insert_snapshot(
            &conn,
            save.id,
            Some("2026-05-01"),
            "2026-08-11T10:00:00.000Z",
        );
        let current = insert_snapshot(
            &conn,
            save.id,
            Some("2027-05-01"),
            "2026-08-11T11:00:00.000Z",
        );
        insert_player(&conn, promoted, 77);
        insert_player(&conn, current, 77);
        insert_staff_record(&conn, promoted, 88);
        insert_staff_record(&conn, current, 88);
        conn.execute(
            "INSERT INTO club_dna_scores (
                snapshot_id, uid, definition_version, score_model_version, score
             ) VALUES (?1, 77, 1, 1, 55), (?2, 77, 1, 1, 70)",
            params![promoted, current],
        )
        .expect("seed exact Club DNA rows");
        select_current_snapshot_for_test(&mut conn, save.id);
        assert_empty_potential_state(&conn, promoted);
        assert_complete_potential_state(&conn, current);
        assert_eq!(compact_row_count(&conn, promoted), 0);
        assert_eq!(compact_row_count(&conn, current), 1);
        assert_eq!(staff_compact_row_count(&conn, promoted), 0);
        assert_eq!(staff_compact_row_count(&conn, current), 1);
        conn.execute_batch(
            "CREATE TRIGGER reject_club_dna_backfill
             BEFORE INSERT ON club_dna_scores
             BEGIN
                 SELECT RAISE(ABORT, 'Club DNA promotion must not backfill');
             END;
             CREATE TRIGGER reject_club_dna_rewrite
             BEFORE UPDATE ON club_dna_scores
             BEGIN
                 SELECT RAISE(ABORT, 'Club DNA promotion must not rewrite');
             END;",
        )
        .expect("reject promotion score writes");

        let current_token = snapshot_token(&conn, current);
        let promoted_result =
            delete_snapshot(&mut conn, current, &current_token).expect("delete current snapshot");
        assert_eq!(promoted_result.current_snapshot_id, Some(promoted));
        assert_eq!(current_snapshot_id(&conn, save.id), Some(promoted));
        assert_complete_potential_state(&conn, promoted);
        assert_eq!(compact_row_count(&conn, promoted), 1);
        assert_eq!(staff_compact_row_count(&conn, promoted), 1);
        let (staff_version, staff_scores) =
            staff_compact_row(&conn, promoted, 88).expect("promoted staff compact row");
        assert_eq!(
            staff_version,
            crate::features::staff::scoring::SCORE_MODEL_VERSION
        );
        assert_eq!(staff_scores.len(), 21);
        assert!(
            staff_scores.iter().all(|score| *score == Some(50)),
            "promotion recalculates every role from the retained attributes"
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM club_dna_scores WHERE snapshot_id = ?1",
                [current],
                |row| row.get::<_, i64>(0),
            )
            .expect("confirm current score cascade"),
            0
        );
        assert_eq!(
            conn.query_row(
                "SELECT definition_version, score_model_version, score
                 FROM club_dna_scores WHERE snapshot_id = ?1 AND uid = 77",
                [promoted],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, Option<i64>>(2)?,
                    ))
                },
            )
            .expect("read unchanged promoted score"),
            (1, 1, Some(55))
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM academy_classes WHERE save_id = ?1 AND class_year = 2026 AND is_automatic = 1",
                [save.id],
                |row| row.get::<_, i64>(0),
            )
            .expect("read promoted automatic class"),
            1
        );

        let promoted_token = snapshot_token(&conn, promoted);
        let final_result =
            delete_snapshot(&mut conn, promoted, &promoted_token).expect("delete final snapshot");
        assert_eq!(final_result.current_snapshot_id, None);
        assert_eq!(current_snapshot_id(&conn, save.id), None);

        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM player_role_metrics", [], |row| row
                .get::<_, i64>(0),)
                .expect("count final compact rows"),
            0
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM staff_role_metrics", [], |row| row
                .get::<_, i64>(0),)
                .expect("count final staff compact rows"),
            0
        );
    }

    #[test]
    fn promoted_snapshot_compact_materialization_failure_rolls_back_current_deletion() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("promote-compact-rollback.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let promoted = insert_snapshot(
            &conn,
            save.id,
            Some("2026-05-01"),
            "2026-08-11T10:00:00.000Z",
        );
        let current = insert_snapshot(
            &conn,
            save.id,
            Some("2027-05-01"),
            "2026-08-11T11:00:00.000Z",
        );
        insert_player(&conn, promoted, 77);
        insert_player(&conn, current, 77);
        select_current_snapshot_for_test(&mut conn, save.id);
        assert_eq!(compact_row_count(&conn, current), 1);
        conn.execute_batch(&format!(
            "CREATE TRIGGER reject_promoted_compact_rows
             BEFORE INSERT ON player_role_metrics
             WHEN NEW.snapshot_id = {promoted}
             BEGIN SELECT RAISE(ABORT, 'promoted compact writes fail'); END;",
            promoted = promoted
        ))
        .expect("reject promoted compact rows");

        let current_token = snapshot_token(&conn, current);
        assert!(delete_snapshot(&mut conn, current, &current_token)
            .expect_err("roll back promoted compact materialization")
            .contains("promoted compact writes fail"));

        assert_eq!(current_snapshot_id(&conn, save.id), Some(current));
        assert_eq!(compact_row_count(&conn, current), 1);
        assert_eq!(compact_row_count(&conn, promoted), 0);
        assert!(!snapshot_token(&conn, promoted).is_empty());
        assert!(!snapshot_token(&conn, current).is_empty());
    }

    #[test]
    fn promoted_snapshot_staff_compact_materialization_failure_rolls_back_current_deletion() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("promote-staff-compact-rollback.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let promoted = insert_snapshot(
            &conn,
            save.id,
            Some("2026-05-01"),
            "2026-08-11T10:00:00.000Z",
        );
        let current = insert_snapshot(
            &conn,
            save.id,
            Some("2027-05-01"),
            "2026-08-11T11:00:00.000Z",
        );
        insert_staff_record(&conn, promoted, 88);
        insert_staff_record(&conn, current, 99);
        select_current_snapshot_for_test(&mut conn, save.id);
        assert_eq!(staff_compact_row_count(&conn, current), 1);
        assert_eq!(staff_compact_row_count(&conn, promoted), 0);
        let prior_current_row = staff_compact_row(&conn, current, 99).expect("current compact row");
        conn.execute_batch(&format!(
            "CREATE TRIGGER reject_promoted_staff_compact_rows
             BEFORE INSERT ON staff_role_metrics
             WHEN NEW.snapshot_id = {promoted}
             BEGIN SELECT RAISE(ABORT, 'promoted staff compact writes fail'); END;",
            promoted = promoted
        ))
        .expect("reject promoted staff compact rows");

        let current_token = snapshot_token(&conn, current);
        assert!(delete_snapshot(&mut conn, current, &current_token)
            .expect_err("roll back promoted staff compact materialization")
            .contains("promoted staff compact writes fail"));

        assert_eq!(current_snapshot_id(&conn, save.id), Some(current));
        assert_eq!(
            staff_compact_row(&conn, current, 99),
            Some(prior_current_row),
            "the previously visible current compact row must survive byte-for-byte"
        );
        assert_eq!(staff_compact_row_count(&conn, promoted), 0);
        assert!(!snapshot_token(&conn, promoted).is_empty());
        assert!(!snapshot_token(&conn, current).is_empty());
    }

    #[test]
    fn wrong_staff_compact_model_version_is_rejected_without_writes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("staff-compact-wrong-version.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let current = insert_snapshot(
            &conn,
            save.id,
            Some("2027-05-01"),
            "2026-08-11T11:00:00.000Z",
        );
        insert_staff_record(&conn, current, 99);
        select_current_snapshot_for_test(&mut conn, save.id);
        assert_eq!(
            staff_compact_row(&conn, current, 99)
                .expect("compact row")
                .0,
            crate::features::staff::scoring::SCORE_MODEL_VERSION
        );
        conn.execute(
            "UPDATE staff_role_metrics SET score_model_version = 2
             WHERE snapshot_id = ?1",
            [current],
        )
        .expect("corrupt the model version");

        let error = crate::features::staff::scoring::assert_snapshot_complete(&conn, current)
            .expect_err("wrong score model version must be rejected");
        assert!(error.contains("incomplete"));
        // The corrupted row is still there untouched: the rejection is a
        // read-side guard, not a repair run.
        assert_eq!(
            conn.query_row(
                "SELECT score_model_version FROM staff_role_metrics
                 WHERE snapshot_id = ?1",
                [current],
                |row| row.get::<_, i64>(0),
            )
            .expect("read corrupted version"),
            crate::features::staff::scoring::SCORE_MODEL_VERSION + 1
        );
    }

    #[test]
    fn promoted_snapshot_materialization_failure_rolls_back_current_deletion() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("promote-potential-rollback.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let promoted = insert_snapshot(
            &conn,
            save.id,
            Some("2026-05-01"),
            "2026-08-11T10:00:00.000Z",
        );
        let current = insert_snapshot(
            &conn,
            save.id,
            Some("2027-05-01"),
            "2026-08-11T11:00:00.000Z",
        );
        insert_player(&conn, promoted, 77);
        insert_player(&conn, current, 77);
        select_current_snapshot_for_test(&mut conn, save.id);
        let current_potential_state = potential_state(&conn, current);
        conn.execute_batch(
            "CREATE TRIGGER reject_promoted_compact_rows
             BEFORE INSERT ON player_role_metrics
             BEGIN SELECT RAISE(ABORT, 'promoted compact writes fail'); END;",
        )
        .expect("reject promoted potential rows");

        let current_token = snapshot_token(&conn, current);
        assert!(delete_snapshot(&mut conn, current, &current_token)
            .expect_err("roll back promoted snapshot materialization")
            .contains("promoted compact writes fail"));

        assert_eq!(current_snapshot_id(&conn, save.id), Some(current));
        assert_eq!(potential_state(&conn, current), current_potential_state);
        assert!(!snapshot_token(&conn, promoted).is_empty());
        assert!(!snapshot_token(&conn, current).is_empty());
    }

    #[test]
    fn switching_between_materialized_saves_performs_no_potential_writes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("save-switch-potential.db"));
        let first_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let second_save = create_save(&conn, "Second save").expect("create second save");
        let first_snapshot = insert_snapshot(
            &conn,
            first_save.id,
            Some("2026-05-01"),
            "2026-08-11T10:00:00.000Z",
        );
        let second_snapshot = insert_snapshot(
            &conn,
            second_save.id,
            Some("2026-06-01"),
            "2026-08-11T11:00:00.000Z",
        );
        insert_player(&conn, first_snapshot, 77);
        insert_player(&conn, second_snapshot, 88);
        insert_staff_record(&conn, first_snapshot, 77);
        insert_staff_record(&conn, second_snapshot, 88);
        select_current_snapshot_for_test(&mut conn, first_save.id);
        select_current_snapshot_for_test(&mut conn, second_save.id);
        assert_complete_potential_state(&conn, first_snapshot);
        assert_complete_potential_state(&conn, second_snapshot);
        assert_eq!(compact_row_count(&conn, first_snapshot), 1);
        assert_eq!(compact_row_count(&conn, second_snapshot), 1);
        assert_eq!(staff_compact_row_count(&conn, first_snapshot), 1);
        assert_eq!(staff_compact_row_count(&conn, second_snapshot), 1);
        deny_potential_writes(&conn);

        let switched = set_active_save(&mut conn, second_save.id).expect("switch saves");

        assert!(switched.is_active);
        assert_eq!(
            current_snapshot_id(&conn, first_save.id),
            Some(first_snapshot)
        );
        assert_eq!(
            current_snapshot_id(&conn, second_save.id),
            Some(second_snapshot)
        );
        assert_eq!(compact_row_count(&conn, first_snapshot), 1);
        assert_eq!(compact_row_count(&conn, second_snapshot), 1);
        assert_eq!(staff_compact_row_count(&conn, first_snapshot), 1);
        assert_eq!(staff_compact_row_count(&conn, second_snapshot), 1);
    }

    #[test]
    fn deleting_saves_preserves_or_rebuilds_the_active_context_and_rejects_reused_ids() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("delete-save.db"));
        let default_save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("default save");
        let inactive = create_save(&conn, "Inactive save").expect("create inactive save");
        let inactive_snapshot = insert_snapshot(
            &conn,
            inactive.id,
            Some("2026-03-01"),
            "2026-08-11T10:00:00.000Z",
        );
        insert_player(&conn, inactive_snapshot, 77);
        conn.execute(
            "INSERT INTO managed_club_settings (save_id, club_name) VALUES (?1, 'Inactive FC')",
            [inactive.id],
        )
        .expect("insert inactive planner setting");
        conn.execute(
            "INSERT INTO club_dna_definitions (save_id, attribute_ids_json)
             VALUES (?1, '[\"attr.Acceleration\"]')",
            [inactive.id],
        )
        .expect("insert inactive Club DNA definition");
        conn.execute(
            "INSERT INTO player_youth_career_stats (save_id, player_uid, career_appearances)
             VALUES (?1, 77, 3)",
            [inactive.id],
        )
        .expect("insert inactive youth enrichment");
        let inactive_token = save_token(&conn, inactive.id);

        let inactive_result =
            delete_save(&mut conn, inactive.id, &inactive_token).expect("delete inactive save");
        assert_eq!(inactive_result.active_save.id, default_save.id);
        assert!(!inactive_result.deleted_was_active);
        assert_eq!(current_snapshot_id(&conn, default_save.id), None);
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM snapshots WHERE id = ?1",
                [inactive_snapshot],
                |row| row.get::<_, i64>(0)
            )
            .expect("check inactive snapshot cascade"),
            0
        );
        for table in [
            "managed_club_settings",
            "club_dna_definitions",
            "player_youth_career_stats",
            "academy_classes",
        ] {
            let remaining: i64 = conn
                .query_row(
                    &format!("SELECT COUNT(*) FROM {table} WHERE save_id = ?1"),
                    [inactive.id],
                    |row| row.get(0),
                )
                .expect("check full save cascade");
            assert_eq!(remaining, 0, "{table} must cascade with its save");
        }

        let replacement = create_save(&conn, "Replacement save").expect("create replacement");
        set_active_save(&mut conn, replacement.id).expect("activate replacement");
        let replacement_token = save_token(&conn, replacement.id);
        let fallback_result = delete_save(&mut conn, replacement.id, &replacement_token)
            .expect("delete active save with fallback");
        assert_eq!(fallback_result.active_save.id, default_save.id);
        assert!(fallback_result.deleted_was_active);
        assert!(fallback_result.active_save.is_active);

        let final_token = save_token(&conn, default_save.id);
        let final_result =
            delete_save(&mut conn, default_save.id, &final_token).expect("delete final save");
        assert_eq!(final_result.active_save.name, DEFAULT_SAVE_NAME);
        assert!(final_result.deleted_was_active);
        assert!(final_result.active_save.is_active);
        assert_eq!(save_count(&conn).expect("count recreated saves"), 1);
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM academy_classes WHERE save_id = ?1 AND class_year = 2025",
                [final_result.active_save.id],
                |row| row.get::<_, i64>(0),
            )
            .expect("read recreated baseline class"),
            1
        );
        assert!(
            delete_save(&mut conn, final_result.active_save.id, &final_token)
                .expect_err("reject reused save id with old token")
                .contains("changed")
        );
        assert!(delete_save(&mut conn, 999, "missing-token")
            .expect_err("reject unknown save")
            .contains("changed"));
    }

    #[test]
    fn deleting_the_final_save_rolls_back_after_replacement_baseline_failure() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("delete-final-save-rollback.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let snapshot = insert_snapshot(
            &conn,
            save.id,
            Some("2026-05-01"),
            "2026-08-11T10:00:00.000Z",
        );
        insert_player(&conn, snapshot, 77);
        conn.execute(
            "INSERT INTO managed_club_settings (save_id, club_name) VALUES (?1, 'Rollback FC')",
            [save.id],
        )
        .expect("insert planner setting");
        conn.execute(
            "INSERT INTO club_dna_definitions (save_id, attribute_ids_json)
             VALUES (?1, '[\"attr.Acceleration\"]')",
            [save.id],
        )
        .expect("insert Club DNA definition");
        conn.execute(
            "INSERT INTO player_youth_career_stats (save_id, player_uid, career_appearances)
             VALUES (?1, 77, 3)",
            [save.id],
        )
        .expect("insert youth enrichment");
        select_current_snapshot_for_test(&mut conn, save.id);
        conn.execute_batch(
            "CREATE TRIGGER reject_recreated_baseline
             BEFORE INSERT ON academy_classes
             WHEN NEW.class_year = 2025
             BEGIN
                 SELECT RAISE(ABORT, 'test replacement baseline failure');
             END;",
        )
        .expect("create replacement failure trigger");

        assert!(delete_save(&mut conn, save.id, &save.context_token)
            .expect_err("reject final save replacement")
            .contains("test replacement baseline failure"));

        assert_eq!(save_count(&conn).expect("count rolled-back saves"), 1);
        assert_eq!(
            active_save_count(&conn).expect("count rolled-back active saves"),
            1
        );
        assert_eq!(
            get_save_by_id(&conn, save.id).expect("read rolled-back save"),
            save
        );
        assert_eq!(current_snapshot_id(&conn, save.id), Some(snapshot));
        assert_eq!(compact_row_count(&conn, snapshot), 1);
        let retained_player_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM players", [], |row| row.get(0))
            .expect("count retained players");
        assert_eq!(retained_player_count, 1);
        for table in [
            "managed_club_settings",
            "club_dna_definitions",
            "player_youth_career_stats",
            "academy_classes",
        ] {
            let remaining: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .expect("count rolled-back save children");
            assert_eq!(remaining, 1, "{table} must roll back with its save");
        }
    }

    #[test]
    fn deleting_a_snapshot_rolls_back_when_the_database_rejects_the_delete() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("delete-snapshot-rollback.db"));
        let save = list_saves(&conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let snapshot = insert_snapshot(
            &conn,
            save.id,
            Some("2026-05-01"),
            "2026-08-11T10:00:00.000Z",
        );
        select_current_snapshot_for_test(&mut conn, save.id);
        conn.execute_batch(
            "CREATE TRIGGER reject_snapshot_delete
             BEFORE DELETE ON snapshots
             BEGIN
                 SELECT RAISE(ABORT, 'test delete failure');
             END;",
        )
        .expect("create rollback trigger");

        let snapshot_token = snapshot_token(&conn, snapshot);
        assert!(delete_snapshot(&mut conn, 999, "missing-token")
            .expect_err("reject unknown snapshot")
            .contains("changed"));
        assert!(delete_snapshot(&mut conn, snapshot, &snapshot_token)
            .expect_err("reject snapshot delete")
            .contains("test delete failure"));
        assert_eq!(current_snapshot_id(&conn, save.id), Some(snapshot));
    }

    fn game_date_edit_setup(conn: &mut Connection) -> (SaveSummary, i64, i64) {
        let save = list_saves(conn)
            .expect("seed default save")
            .into_iter()
            .find(|save| save.is_active)
            .expect("active save");
        let older = insert_snapshot(
            conn,
            save.id,
            Some("2026-01-01"),
            "2026-08-11T10:00:00.000Z",
        );
        let current = insert_snapshot(
            conn,
            save.id,
            Some("2026-06-01"),
            "2026-08-11T11:00:00.000Z",
        );
        insert_player(conn, older, 77);
        insert_staff_record(conn, older, 77);
        insert_player(conn, current, 78);
        insert_staff_record(conn, current, 78);
        select_current_snapshot_for_test(conn, save.id);
        (save, older, current)
    }

    #[test]
    fn rejects_noncanonical_game_dates_with_state_unchanged() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("reject-bad-game-date.db"));
        let (save, older, current) = game_date_edit_setup(&mut conn);
        let older_token = snapshot_token(&conn, older);
        rename_snapshot(&conn, older, &older_token, Some("Window label"))
            .expect("label older snapshot");
        let before = get_snapshot_metadata(&conn, older).expect("read older snapshot");
        let player_before: String = conn
            .query_row(
                "SELECT attributes_json FROM players WHERE snapshot_id = ?1 AND uid = 77",
                [older],
                |row| row.get(0),
            )
            .expect("read player attributes");

        for invalid in [
            "",
            " 2024-02-29 ",
            "2026-13-01",
            "2026-02-30",
            "2023-02-29",
            "2026-1-1",
            "2026/01/01",
            "not-a-date",
        ] {
            let error = update_snapshot_game_date(&mut conn, older, &older_token, invalid)
                .expect_err(&format!("reject {invalid:?}"));
            assert!(
                error.contains("YYYY-MM-DD"),
                "unexpected error for {invalid:?}: {error}"
            );

            let after = get_snapshot_metadata(&conn, older).expect("read older snapshot");
            assert_eq!(
                after, before,
                "rejected input {invalid:?} must leave the row intact"
            );
            assert_eq!(current_snapshot_id(&conn, save.id), Some(current));
            assert_eq!(compact_row_count(&conn, older), 0);
            assert_eq!(compact_row_count(&conn, current), 1);
            assert_eq!(staff_compact_row_count(&conn, current), 1);
            let player_after: String = conn
                .query_row(
                    "SELECT attributes_json FROM players WHERE snapshot_id = ?1 AND uid = 77",
                    [older],
                    |row| row.get(0),
                )
                .expect("read player attributes");
            assert_eq!(player_after, player_before);
        }
    }

    #[test]
    fn accepts_leap_day_and_updates_only_game_date() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("leap-day-game-date.db"));
        let (save, older, current) = game_date_edit_setup(&mut conn);
        let older_token = snapshot_token(&conn, older);
        rename_snapshot(&conn, older, &older_token, Some("Window label"))
            .expect("label older snapshot");
        let before = get_snapshot_metadata(&conn, older).expect("read older snapshot");
        let player_before: String = conn
            .query_row(
                "SELECT attributes_json FROM players WHERE snapshot_id = ?1 AND uid = 77",
                [older],
                |row| row.get(0),
            )
            .expect("read player attributes");

        let result = update_snapshot_game_date(&mut conn, older, &older_token, "2024-02-29")
            .expect("accept leap day");

        assert_eq!(result.snapshot.game_date.as_deref(), Some("2024-02-29"));
        assert_eq!(result.snapshot.id, older);
        assert_eq!(result.previous_current_snapshot_id, Some(current));
        assert_eq!(result.current_snapshot_id, Some(current));
        assert_eq!(result.snapshot.custom_name.as_deref(), Some("Window label"));
        assert_eq!(
            result.snapshot.game_date_source, before.game_date_source,
            "game_date_source must stay unchanged"
        );
        assert_eq!(
            result.snapshot.loaded_at_utc, before.loaded_at_utc,
            "loaded_at_utc must stay unchanged"
        );
        assert!(!result.snapshot.is_current);
        let stored = get_snapshot_metadata(&conn, older).expect("read updated snapshot");
        assert_eq!(stored, result.snapshot);
        assert_eq!(current_snapshot_id(&conn, save.id), Some(current));
        let player_after: String = conn
            .query_row(
                "SELECT attributes_json FROM players WHERE snapshot_id = ?1 AND uid = 77",
                [older],
                |row| row.get(0),
            )
            .expect("read player attributes");
        assert_eq!(player_after, player_before);
    }

    #[test]
    fn rejects_stale_snapshot_token_without_writing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("stale-token-game-date.db"));
        let (save, older, current) = game_date_edit_setup(&mut conn);
        let before = get_snapshot_metadata(&conn, older).expect("read older snapshot");

        let error = update_snapshot_game_date(&mut conn, older, "stale-token", "2026-03-01")
            .expect_err("reject stale token");

        assert!(error.contains("changed"), "unexpected error: {error}");
        assert_eq!(
            get_snapshot_metadata(&conn, older).expect("read older snapshot"),
            before
        );
        assert_eq!(current_snapshot_id(&conn, save.id), Some(current));
    }

    #[test]
    fn date_edit_promotes_and_demotes_per_shared_order() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("promote-demote-game-date.db"));
        let (save, older, current) = game_date_edit_setup(&mut conn);
        let older_token = snapshot_token(&conn, older);

        let promoted = update_snapshot_game_date(&mut conn, older, &older_token, "2027-01-01")
            .expect("promote older snapshot");
        assert_eq!(promoted.previous_current_snapshot_id, Some(current));
        assert_eq!(promoted.current_snapshot_id, Some(older));
        assert!(promoted.snapshot.is_current);
        assert_eq!(current_snapshot_id(&conn, save.id), Some(older));
        assert_eq!(
            list_snapshot_metadata(&conn, Some(save.id))
                .expect("list reordered snapshots")
                .iter()
                .map(|snapshot| snapshot.id)
                .collect::<Vec<_>>(),
            vec![older, current]
        );
        assert_complete_potential_state(&conn, older);
        assert_eq!(staff_compact_row_count(&conn, older), 1);

        let demoted = update_snapshot_game_date(&mut conn, older, &older_token, "2025-01-01")
            .expect("demote edited snapshot");
        assert_eq!(demoted.previous_current_snapshot_id, Some(older));
        assert_eq!(demoted.current_snapshot_id, Some(current));
        assert!(!demoted.snapshot.is_current);
        assert_eq!(current_snapshot_id(&conn, save.id), Some(current));
        assert_eq!(
            list_snapshot_metadata(&conn, Some(save.id))
                .expect("list demoted snapshots")
                .iter()
                .map(|snapshot| snapshot.id)
                .collect::<Vec<_>>(),
            vec![current, older]
        );
        assert_complete_potential_state(&conn, current);
    }

    #[test]
    fn equal_dates_break_ties_by_load_time_then_id() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("tie-break-game-date.db"));
        let (save, older, current) = game_date_edit_setup(&mut conn);
        let older_token = snapshot_token(&conn, older);

        let tied = update_snapshot_game_date(&mut conn, older, &older_token, "2026-06-01")
            .expect("tie on game date");
        assert_eq!(tied.previous_current_snapshot_id, Some(current));
        assert_eq!(
            tied.current_snapshot_id,
            Some(current),
            "newer loaded_at_utc must win an equal-date tie"
        );
        assert!(!tied.snapshot.is_current);
        assert_eq!(current_snapshot_id(&conn, save.id), Some(current));

        let third = insert_snapshot(
            &conn,
            save.id,
            Some("2026-01-01"),
            "2026-08-11T12:00:00.000Z",
        );
        let fourth = insert_snapshot(
            &conn,
            save.id,
            Some("2026-09-01"),
            "2026-08-11T12:00:00.000Z",
        );
        insert_player(&conn, third, 79);
        insert_staff_record(&conn, third, 79);
        insert_player(&conn, fourth, 80);
        insert_staff_record(&conn, fourth, 80);
        select_current_snapshot_for_test(&mut conn, save.id);
        assert_eq!(current_snapshot_id(&conn, save.id), Some(fourth));
        let third_token = snapshot_token(&conn, third);

        let id_tied = update_snapshot_game_date(&mut conn, third, &third_token, "2026-09-01")
            .expect("tie on game date and load time");
        assert_eq!(
            id_tied.current_snapshot_id,
            Some(fourth),
            "greater snapshot id must win a full tie"
        );
        assert!(!id_tied.snapshot.is_current);
        assert_eq!(current_snapshot_id(&conn, save.id), Some(fourth));
    }

    #[test]
    fn compact_materialization_failure_rolls_back_date_edit() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut player_conn = open_migrated(&temp_dir.path().join("player-rollback-game-date.db"));
        let (save, older, current) = game_date_edit_setup(&mut player_conn);
        let current_potential = potential_state(&player_conn, current);
        player_conn
            .execute_batch(&format!(
                "CREATE TRIGGER reject_promoted_player_compact
                 BEFORE INSERT ON player_role_metrics
                 WHEN NEW.snapshot_id = {older}
                 BEGIN SELECT RAISE(ABORT, 'promoted player compact writes fail'); END;",
            ))
            .expect("reject promoted player compact rows");

        let older_token = snapshot_token(&player_conn, older);
        let error = update_snapshot_game_date(&mut player_conn, older, &older_token, "2027-01-01")
            .expect_err("roll back player compact failure");
        assert!(
            error.contains("promoted player compact writes fail"),
            "unexpected error: {error}"
        );
        assert_eq!(
            get_snapshot_metadata(&player_conn, older)
                .expect("read rolled-back snapshot")
                .game_date
                .as_deref(),
            Some("2026-01-01")
        );
        assert_eq!(current_snapshot_id(&player_conn, save.id), Some(current));
        assert_eq!(compact_row_count(&player_conn, older), 0);
        assert_eq!(compact_row_count(&player_conn, current), 1);
        assert_eq!(potential_state(&player_conn, current), current_potential);
        assert_eq!(staff_compact_row_count(&player_conn, older), 0);
        assert_eq!(staff_compact_row_count(&player_conn, current), 1);

        let mut staff_conn = open_migrated(&temp_dir.path().join("staff-rollback-game-date.db"));
        let (save, older, current) = game_date_edit_setup(&mut staff_conn);
        let current_staff_row =
            staff_compact_row(&staff_conn, current, 78).expect("current staff compact row");
        staff_conn
            .execute_batch(&format!(
                "CREATE TRIGGER reject_promoted_staff_compact
                 BEFORE INSERT ON staff_role_metrics
                 WHEN NEW.snapshot_id = {older}
                 BEGIN SELECT RAISE(ABORT, 'promoted staff compact writes fail'); END;",
            ))
            .expect("reject promoted staff compact rows");

        let older_token = snapshot_token(&staff_conn, older);
        let error = update_snapshot_game_date(&mut staff_conn, older, &older_token, "2027-01-01")
            .expect_err("roll back staff compact failure");
        assert!(
            error.contains("promoted staff compact writes fail"),
            "unexpected error: {error}"
        );
        assert_eq!(
            get_snapshot_metadata(&staff_conn, older)
                .expect("read rolled-back snapshot")
                .game_date
                .as_deref(),
            Some("2026-01-01")
        );
        assert_eq!(current_snapshot_id(&staff_conn, save.id), Some(current));
        assert_eq!(staff_compact_row_count(&staff_conn, older), 0);
        assert_eq!(
            staff_compact_row(&staff_conn, current, 78),
            Some(current_staff_row),
            "the previously visible staff compact row must survive byte-for-byte"
        );
        assert_eq!(compact_row_count(&staff_conn, older), 0);
        assert_eq!(compact_row_count(&staff_conn, current), 1);
    }

    #[test]
    fn date_edit_creates_no_academy_class() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("no-academy-game-date.db"));
        let (save, older, current) = game_date_edit_setup(&mut conn);
        select_current_snapshot_for_test(&mut conn, save.id);
        assert_eq!(current_snapshot_id(&conn, save.id), Some(current));
        let classes_before: Vec<(i64, i32)> = conn
            .prepare(
                "SELECT class_year, is_automatic FROM academy_classes
                 WHERE save_id = ?1 ORDER BY class_year",
            )
            .expect("prepare academy classes")
            .query_map([save.id], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("read academy classes")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect academy classes");
        conn.execute_batch(
            "CREATE TRIGGER reject_academy_class_writes
             BEFORE INSERT ON academy_classes
             BEGIN SELECT RAISE(ABORT, 'date edit must not create Academy classes'); END;",
        )
        .expect("reject Academy class writes");

        let older_token = snapshot_token(&conn, older);
        let promoted = update_snapshot_game_date(&mut conn, older, &older_token, "2028-01-01")
            .expect("promoting edit must not write Academy classes");

        assert_eq!(promoted.current_snapshot_id, Some(older));
        let classes_after: Vec<(i64, i32)> = conn
            .prepare(
                "SELECT class_year, is_automatic FROM academy_classes
                 WHERE save_id = ?1 ORDER BY class_year",
            )
            .expect("prepare academy classes")
            .query_map([save.id], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("read academy classes")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect academy classes");
        assert_eq!(
            classes_after, classes_before,
            "no Academy class may be created or changed by a date edit"
        );
    }
}
