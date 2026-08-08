use rusqlite::{params, Connection, OptionalExtension, Row};

use super::ingest::SnapshotSummary;
use super::service::ensure_default_save;

pub const DEFAULT_SANITY_LIMIT: usize = 20;
pub const MAX_SANITY_LIMIT: usize = 20;

/// Stable catalog role used as the sanity-list score proof column.
pub const PROOF_ROLE_ID: &str = "deep_lying_playmaker_ip";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSanityRow {
    pub name: String,
    pub ca: i64,
    pub club: Option<String>,
    pub proof_role_score: Option<i32>,
}

pub fn get_current_snapshot(conn: &Connection) -> Result<Option<SnapshotSummary>, String> {
    ensure_default_save(conn)?;

    conn.query_row(
        "SELECT
            s.id,
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

pub fn list_sanity_players(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<PlayerSanityRow>, String> {
    ensure_default_save(conn)?;

    let snapshot_id: Option<i64> = conn
        .query_row(
            "SELECT s.id
             FROM snapshots s
             INNER JOIN saves sv ON sv.id = s.save_id AND sv.is_active = 1
             WHERE s.is_current = 1
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| error.to_string())?;

    let Some(snapshot_id) = snapshot_id else {
        return Ok(Vec::new());
    };

    let limit = i64::try_from(limit).map_err(|_| "sanity list limit out of range".to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT p.name, p.ca, p.current_club, prs.score
             FROM players p
             LEFT JOIN player_role_scores prs
               ON prs.snapshot_id = p.snapshot_id
              AND prs.uid = p.uid
              AND prs.role_id = ?3
             WHERE p.snapshot_id = ?1
             ORDER BY p.name COLLATE NOCASE
             LIMIT ?2",
        )
        .map_err(|error| error.to_string())?;

    let players = stmt
        .query_map(
            params![snapshot_id, limit, PROOF_ROLE_ID],
            map_player_sanity_row,
        )
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(players)
}

fn map_snapshot_row(row: &Row<'_>) -> rusqlite::Result<SnapshotSummary> {
    Ok(SnapshotSummary {
        id: row.get(0)?,
        save_id: row.get(1)?,
        schema_version: row.get(2)?,
        generated_at_utc: row.get(3)?,
        game_version: row.get(4)?,
        supported_game_version: row.get(5)?,
        bridge_version: row.get(6)?,
        protocol_version: row.get(7)?,
        game_date: row.get(8)?,
        game_date_source: row.get(9)?,
        scan_truncated: row.get::<_, i32>(10)? == 1,
        max_accepted: row.get(11)?,
        player_count: row.get(12)?,
        loaded_at_utc: row.get(13)?,
    })
}

fn map_player_sanity_row(row: &Row<'_>) -> rusqlite::Result<PlayerSanityRow> {
    Ok(PlayerSanityRow {
        name: row.get(0)?,
        ca: row.get(1)?,
        club: row.get(2)?,
        proof_role_score: row.get(3)?,
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
        let players = list_sanity_players(&conn, DEFAULT_SANITY_LIMIT).expect("list players");

        assert!(snapshot.is_none());
        assert!(players.is_empty());
    }

    #[test]
    fn returns_snapshot_and_sanity_rows_for_active_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("with-snapshot.db"));
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v6.json");

        ingest_dump_file(&mut conn, &dump_path).expect("ingest golden dump");

        let snapshot = get_current_snapshot(&conn)
            .expect("query snapshot")
            .expect("current snapshot");
        let players = list_sanity_players(&conn, 5).expect("list players");

        assert!(snapshot.player_count > 0);
        assert!(!players.is_empty());
        assert!(players.len() <= 5);
        assert!(players.windows(2).all(|pair| pair[0].name <= pair[1].name));
    }

    #[test]
    fn sanity_list_includes_proof_role_score_after_ingest() {
        use crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS;
        use serde_json::{json, Value};

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("proof-role-score.db"));
        let mut root: Value =
            serde_json::from_str(include_str!("../memory_read/fixtures/golden_dump_v6.json"))
                .expect("parse golden fixture");
        let attributes: serde_json::Map<String, Value> = DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| (key.to_string(), json!(10)))
            .collect();
        root["players"][0]["attributes"] = Value::Object(attributes);

        let dump_path = temp_dir.path().join("uniform.json");
        std::fs::write(&dump_path, root.to_string()).expect("write dump");
        ingest_dump_file(&mut conn, &dump_path).expect("ingest uniform dump");

        let players = list_sanity_players(&conn, DEFAULT_SANITY_LIMIT).expect("list players");

        assert_eq!(players.len(), 1);
        assert_eq!(players[0].proof_role_score, Some(50));
    }

    #[test]
    fn sanity_list_follows_active_save_after_switch() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn = open_migrated(&temp_dir.path().join("switch-save.db"));
        let dump_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/features/memory_read/fixtures/golden_dump_v6.json");

        ingest_dump_file(&mut conn, &dump_path).expect("ingest into default save");
        let default_snapshot = get_current_snapshot(&conn)
            .expect("query snapshot")
            .expect("default snapshot");

        let second_save = create_save(&conn, "Second save").expect("create save");
        set_active_save(&mut conn, second_save.id).expect("switch save");

        let switched_snapshot = get_current_snapshot(&conn).expect("query after switch");
        let players = list_sanity_players(&conn, DEFAULT_SANITY_LIMIT).expect("list players");

        assert!(switched_snapshot.is_none());
        assert!(players.is_empty());

        set_active_save(&mut conn, default_snapshot.save_id).expect("switch back");
        let restored = get_current_snapshot(&conn)
            .expect("query restored")
            .expect("restored snapshot");
        assert_eq!(restored.id, default_snapshot.id);
    }
}
