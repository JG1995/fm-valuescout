use rusqlite::{Connection, OptionalExtension, Row};

use super::ingest::SnapshotSummary;
use super::service::{ensure_default_save, SnapshotMetadata, SNAPSHOT_ORDER_BY};

pub fn get_current_snapshot(conn: &Connection) -> Result<Option<SnapshotSummary>, String> {
    ensure_default_save(conn)?;

    conn.query_row(
        "SELECT
            s.id,
            s.context_token,
            s.save_id,
            s.schema_version,
            s.generated_at_utc,
            s.game_version,
            s.supported_game_version,
            s.bridge_version,
            s.protocol_version,
            s.game_date,
            s.game_date_source,
            s.scan_truncated,
            s.max_accepted,
            s.player_count,
            s.loaded_at_utc
         FROM snapshots s
         INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
         WHERE s.is_current = 1
         LIMIT 1",
        [],
        map_snapshot_row,
    )
    .optional()
    .map_err(|error| error.to_string())
}

pub fn list_snapshot_metadata(
    conn: &Connection,
    requested_save_id: Option<i64>,
) -> Result<Vec<SnapshotMetadata>, String> {
    let save_id = match requested_save_id {
        Some(save_id) => {
            let exists: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1)",
                    [save_id],
                    |row| row.get(0),
                )
                .map_err(|error| error.to_string())?;
            if !exists {
                return Err(format!("Save {save_id} not found"));
            }
            save_id
        }
        None => {
            ensure_default_save(conn)?;
            conn.query_row(
                "SELECT id FROM saves WHERE is_active = 1 LIMIT 1",
                [],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?
        }
    };

    let list_snapshots_sql = format!(
        "SELECT
                id,
                context_token,
                save_id,
                custom_name,
                game_date,
                game_date_source,
                player_count,
                loaded_at_utc,
                is_current
             FROM snapshots
             WHERE save_id = ?1
             ORDER BY {SNAPSHOT_ORDER_BY}"
    );
    let mut statement = conn
        .prepare(&list_snapshots_sql)
        .map_err(|error| error.to_string())?;
    let snapshots = statement
        .query_map([save_id], map_snapshot_metadata_row)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    Ok(snapshots)
}

pub(crate) fn get_snapshot_metadata(
    conn: &Connection,
    snapshot_id: i64,
) -> Result<SnapshotMetadata, String> {
    conn.query_row(
        "SELECT
            id,
            context_token,
            save_id,
            custom_name,
            game_date,
            game_date_source,
            player_count,
            loaded_at_utc,
            is_current
         FROM snapshots
         WHERE id = ?1",
        [snapshot_id],
        map_snapshot_metadata_row,
    )
    .map_err(|error| error.to_string())
}

fn map_snapshot_row(row: &Row<'_>) -> rusqlite::Result<SnapshotSummary> {
    Ok(SnapshotSummary {
        id: row.get(0)?,
        context_token: row.get(1)?,
        save_id: row.get(2)?,
        schema_version: row.get(3)?,
        generated_at_utc: row.get(4)?,
        game_version: row.get(5)?,
        supported_game_version: row.get(6)?,
        bridge_version: row.get(7)?,
        protocol_version: row.get(8)?,
        game_date: row.get(9)?,
        game_date_source: row.get(10)?,
        scan_truncated: row.get::<_, i32>(11)? == 1,
        max_accepted: row.get(12)?,
        player_count: row.get(13)?,
        loaded_at_utc: row.get(14)?,
    })
}

fn map_snapshot_metadata_row(row: &Row<'_>) -> rusqlite::Result<SnapshotMetadata> {
    Ok(SnapshotMetadata {
        id: row.get(0)?,
        context_token: row.get(1)?,
        save_id: row.get(2)?,
        custom_name: row.get(3)?,
        game_date: row.get(4)?,
        game_date_source: row.get(5)?,
        player_count: row.get(6)?,
        loaded_at_utc: row.get(7)?,
        is_current: row.get::<_, i32>(8)? == 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::snapshot::ingest::ingest_dump_file;
    use crate::features::snapshot::service::{create_save, set_active_save};
    use std::path::Path;

    fn open_migrated(db_path: &Path) -> rusqlite::Connection {
        let conn = rusqlite::Connection::open(db_path).expect("open test db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    #[test]
    fn returns_none_when_active_save_has_no_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = open_migrated(&temp_dir.path().join("no-snapshot.db"));

        let snapshot = get_current_snapshot(&conn).expect("query snapshot");

        assert!(snapshot.is_none());
    }

    #[test]
    fn returns_current_snapshot_for_active_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("with-snapshot.db"));
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v8.json");

        ingest_dump_file(&mut conn, &dump_path).expect("ingest golden dump");

        let snapshot = get_current_snapshot(&conn)
            .expect("query snapshot")
            .expect("current snapshot");

        assert!(snapshot.player_count > 0);
        assert!(!snapshot.context_token.is_empty());
    }

    #[test]
    fn current_snapshot_follows_active_save_after_switch() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("switch-save.db"));
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v8.json");

        ingest_dump_file(&mut conn, &dump_path).expect("ingest into default save");
        let default_snapshot = get_current_snapshot(&conn)
            .expect("query snapshot")
            .expect("default snapshot");

        let second_save = create_save(&conn, "Second save").expect("create save");
        set_active_save(&mut conn, second_save.id).expect("switch save");

        let switched_snapshot = get_current_snapshot(&conn).expect("query after switch");

        assert!(switched_snapshot.is_none());

        set_active_save(&mut conn, default_snapshot.save_id).expect("switch back");
        let restored = get_current_snapshot(&conn)
            .expect("query restored")
            .expect("restored snapshot");
        assert_eq!(restored.id, default_snapshot.id);
    }
}
