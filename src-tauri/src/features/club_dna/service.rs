use std::collections::HashSet;

use rusqlite::{params, Connection, OptionalExtension, Transaction};

use crate::features::player_metrics::resolver::{HIDDEN_ATTRIBUTE_KEYS, PERSONALITY_KEYS};
use crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClubDnaDefinition {
    pub attribute_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClubDnaUpsertResult {
    pub definition: ClubDnaDefinition,
    pub created: bool,
}

pub fn get_club_dna(
    conn: &Connection,
    save_id: i64,
    context_token: &str,
) -> Result<Option<ClubDnaDefinition>, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|_| "Unable to access Club DNA".to_string())?;
    ensure_active_save_context(&tx, save_id, context_token)?;
    let definition = tx
        .query_row(
            "SELECT attribute_ids_json FROM club_dna_definitions WHERE save_id = ?1",
            [save_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|_| "Unable to access Club DNA".to_string())?
        .map(|attribute_ids_json| {
            serde_json::from_str::<Vec<String>>(&attribute_ids_json)
                .map(|attribute_ids| ClubDnaDefinition { attribute_ids })
                .map_err(|_| "Stored Club DNA definition is invalid".to_string())
        })
        .transpose()?;
    tx.commit()
        .map_err(|_| "Unable to access Club DNA".to_string())?;
    Ok(definition)
}

pub fn set_club_dna(
    conn: &Connection,
    save_id: i64,
    context_token: &str,
    attribute_ids: Vec<String>,
) -> Result<ClubDnaUpsertResult, String> {
    validate_attribute_ids(&attribute_ids)?;
    let attribute_ids_json =
        serde_json::to_string(&attribute_ids).map_err(|_| "Unable to save Club DNA".to_string())?;
    let tx = conn
        .unchecked_transaction()
        .map_err(|_| "Unable to save Club DNA".to_string())?;
    ensure_active_save_context(&tx, save_id, context_token)?;
    let created: bool = tx
        .query_row(
            "SELECT NOT EXISTS(SELECT 1 FROM club_dna_definitions WHERE save_id = ?1)",
            [save_id],
            |row| row.get(0),
        )
        .map_err(|_| "Unable to save Club DNA".to_string())?;
    tx.execute(
        "INSERT INTO club_dna_definitions (save_id, attribute_ids_json)
         VALUES (?1, ?2)
         ON CONFLICT(save_id) DO UPDATE SET attribute_ids_json = excluded.attribute_ids_json",
        params![save_id, attribute_ids_json],
    )
    .map_err(|_| "Unable to save Club DNA".to_string())?;
    tx.commit()
        .map_err(|_| "Unable to save Club DNA".to_string())?;

    Ok(ClubDnaUpsertResult {
        definition: ClubDnaDefinition { attribute_ids },
        created,
    })
}

pub fn remove_club_dna(
    conn: &Connection,
    save_id: i64,
    context_token: &str,
) -> Result<bool, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|_| "Unable to remove Club DNA".to_string())?;
    ensure_active_save_context(&tx, save_id, context_token)?;
    let removed = tx
        .execute(
            "DELETE FROM club_dna_definitions WHERE save_id = ?1",
            [save_id],
        )
        .map_err(|_| "Unable to remove Club DNA".to_string())?
        > 0;
    tx.commit()
        .map_err(|_| "Unable to remove Club DNA".to_string())?;
    Ok(removed)
}

fn ensure_active_save_context(
    tx: &Transaction<'_>,
    save_id: i64,
    context_token: &str,
) -> Result<(), String> {
    let matches: bool = tx
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM saves
                WHERE id = ?1 AND context_token = ?2 AND is_active = 1
            )",
            params![save_id, context_token],
            |row| row.get(0),
        )
        .map_err(|_| "Unable to access Club DNA".to_string())?;

    matches
        .then_some(())
        .ok_or_else(|| "Save changed or is no longer active".to_string())
}

fn validate_attribute_ids(attribute_ids: &[String]) -> Result<(), String> {
    if attribute_ids.is_empty() {
        return Err("Club DNA requires at least one attribute".to_string());
    }

    let mut seen = HashSet::new();
    for attribute_id in attribute_ids {
        if !is_supported_attribute_id(attribute_id) {
            return Err("Club DNA contains an unsupported attribute".to_string());
        }
        if !seen.insert(attribute_id) {
            return Err("Club DNA attributes must be unique".to_string());
        }
    }
    Ok(())
}

fn is_supported_attribute_id(attribute_id: &str) -> bool {
    attribute_id
        .strip_prefix("attr.")
        .is_some_and(|key| DUMP_ATTRIBUTE_KEYS.contains(&key))
        || attribute_id
            .strip_prefix("hidden.")
            .is_some_and(|key| HIDDEN_ATTRIBUTE_KEYS.contains(&key))
        || attribute_id
            .strip_prefix("personality.")
            .is_some_and(|key| PERSONALITY_KEYS.contains(&key))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::db::migrations;

    fn connection(path: &Path) -> Connection {
        let conn = Connection::open(path).expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn insert_save(conn: &Connection, name: &str, is_active: bool) -> (i64, String) {
        let save_id: i64 = conn
            .query_row(
                "INSERT INTO saves (name, is_active) VALUES (?1, ?2) RETURNING id",
                params![name, i32::from(is_active)],
                |row| row.get(0),
            )
            .expect("insert save");
        let context_token = conn
            .query_row(
                "SELECT context_token FROM saves WHERE id = ?1",
                [save_id],
                |row| row.get(0),
            )
            .expect("read save context token");
        (save_id, context_token)
    }

    fn activate_save(conn: &Connection, save_id: i64) {
        conn.execute("UPDATE saves SET is_active = 0", [])
            .expect("deactivate saves");
        conn.execute("UPDATE saves SET is_active = 1 WHERE id = ?1", [save_id])
            .expect("activate save");
    }

    fn insert_snapshot(conn: &Connection, save_id: i64, is_current: bool) -> i64 {
        conn.query_row(
            "INSERT INTO snapshots (
                 save_id, is_current, schema_version, generated_at_utc,
                 game_version, supported_game_version, bridge_version,
                 protocol_version, game_date_source, scan_truncated,
                 player_count
             ) VALUES (?1, ?2, 8, '2026-08-18T00:00:00Z', '26.3.2',
                       '26.3', '0.4.0', 1, 'inGame', 0, 0)
             RETURNING id",
            params![save_id, i32::from(is_current)],
            |row| row.get(0),
        )
        .expect("insert snapshot")
    }

    #[test]
    fn accepts_the_complete_supported_catalog_in_canonical_order() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = connection(&temp_dir.path().join("club-dna-catalog.db"));
        let (save_id, context_token) = insert_save(&conn, "Save", true);
        let attribute_ids = DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| format!("attr.{key}"))
            .chain(
                HIDDEN_ATTRIBUTE_KEYS
                    .iter()
                    .map(|key| format!("hidden.{key}")),
            )
            .chain(
                PERSONALITY_KEYS
                    .iter()
                    .map(|key| format!("personality.{key}")),
            )
            .collect::<Vec<_>>();

        let result = set_club_dna(&conn, save_id, &context_token, attribute_ids.clone())
            .expect("store complete catalog");

        assert!(result.created);
        assert_eq!(result.definition.attribute_ids, attribute_ids);
        assert_eq!(
            get_club_dna(&conn, save_id, &context_token).expect("read definition"),
            Some(ClubDnaDefinition { attribute_ids })
        );
    }

    #[test]
    fn rejects_empty_unknown_and_duplicate_attribute_ids() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = connection(&temp_dir.path().join("club-dna-validation.db"));
        let (save_id, context_token) = insert_save(&conn, "Save", true);

        for attribute_ids in [
            vec![],
            vec!["attr.NotARealAttribute".to_string()],
            vec![
                "attr.Acceleration".to_string(),
                "attr.Acceleration".to_string(),
            ],
        ] {
            assert!(set_club_dna(&conn, save_id, &context_token, attribute_ids).is_err());
        }
        assert_eq!(
            get_club_dna(&conn, save_id, &context_token).expect("read empty definition"),
            None
        );
    }

    #[test]
    fn definitions_are_save_scoped_replace_in_full_and_survive_reopen_and_snapshot_changes() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("club-dna-persistence.db");
        let conn = connection(&db_path);
        let (first_save, first_token) = insert_save(&conn, "First", true);
        let (second_save, second_token) = insert_save(&conn, "Second", false);
        let first_ids = vec![
            "attr.Acceleration".to_string(),
            "hidden.Consistency".to_string(),
            "personality.Ambition".to_string(),
        ];

        assert!(
            set_club_dna(&conn, first_save, &first_token, first_ids.clone())
                .expect("create first definition")
                .created
        );
        activate_save(&conn, second_save);
        assert!(
            set_club_dna(
                &conn,
                second_save,
                &second_token,
                vec!["attr.Handling".to_string()],
            )
            .expect("create second definition")
            .created
        );
        assert_eq!(
            get_club_dna(&conn, second_save, &second_token).expect("read second definition"),
            Some(ClubDnaDefinition {
                attribute_ids: vec!["attr.Handling".to_string()],
            })
        );

        activate_save(&conn, first_save);
        let replacement = vec![
            "personality.Professionalism".to_string(),
            "attr.Pace".to_string(),
        ];
        assert!(
            !set_club_dna(&conn, first_save, &first_token, replacement.clone())
                .expect("replace first definition")
                .created
        );
        let snapshot = insert_snapshot(&conn, first_save, true);
        conn.execute("DELETE FROM snapshots WHERE id = ?1", [snapshot])
            .expect("delete snapshot");
        assert_eq!(
            get_club_dna(&conn, first_save, &first_token).expect("read replaced definition"),
            Some(ClubDnaDefinition {
                attribute_ids: replacement.clone(),
            })
        );

        drop(conn);
        let reopened = connection(&db_path);
        assert_eq!(
            get_club_dna(&reopened, first_save, &first_token).expect("read reopened definition"),
            Some(ClubDnaDefinition {
                attribute_ids: replacement,
            })
        );
        activate_save(&reopened, second_save);
        assert_eq!(
            get_club_dna(&reopened, second_save, &second_token).expect("read isolated definition"),
            Some(ClubDnaDefinition {
                attribute_ids: vec!["attr.Handling".to_string()],
            })
        );
    }

    #[test]
    fn stale_context_get_set_and_remove_cannot_read_or_mutate_a_previous_active_save() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = connection(&temp_dir.path().join("club-dna-stale-context.db"));
        let (first_save, first_token) = insert_save(&conn, "First", true);
        let (second_save, second_token) = insert_save(&conn, "Second", false);
        set_club_dna(
            &conn,
            first_save,
            &first_token,
            vec!["attr.Acceleration".to_string()],
        )
        .expect("create first definition");
        activate_save(&conn, second_save);
        set_club_dna(
            &conn,
            second_save,
            &second_token,
            vec!["attr.Handling".to_string()],
        )
        .expect("create second definition");

        for result in [
            get_club_dna(&conn, first_save, &first_token).map(|_| ()),
            set_club_dna(
                &conn,
                first_save,
                &first_token,
                vec!["attr.Pace".to_string()],
            )
            .map(|_| ()),
            remove_club_dna(&conn, first_save, &first_token).map(|_| ()),
        ] {
            assert!(result
                .expect_err("reject stale context")
                .contains("changed"));
        }
        assert_eq!(
            get_club_dna(&conn, second_save, &second_token).expect("read active definition"),
            Some(ClubDnaDefinition {
                attribute_ids: vec!["attr.Handling".to_string()],
            })
        );
        activate_save(&conn, first_save);
        assert_eq!(
            get_club_dna(&conn, first_save, &first_token).expect("read retained definition"),
            Some(ClubDnaDefinition {
                attribute_ids: vec!["attr.Acceleration".to_string()],
            })
        );
    }

    #[test]
    fn remove_is_idempotent_and_save_deletion_cascades_the_definition() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = connection(&temp_dir.path().join("club-dna-delete.db"));
        let (save_id, context_token) = insert_save(&conn, "Save", true);
        set_club_dna(
            &conn,
            save_id,
            &context_token,
            vec!["attr.Acceleration".to_string()],
        )
        .expect("create definition");

        assert!(remove_club_dna(&conn, save_id, &context_token).expect("remove definition"));
        assert!(!remove_club_dna(&conn, save_id, &context_token).expect("remove absent definition"));
        set_club_dna(
            &conn,
            save_id,
            &context_token,
            vec!["hidden.Consistency".to_string()],
        )
        .expect("recreate definition");
        conn.execute("DELETE FROM saves WHERE id = ?1", [save_id])
            .expect("delete save");
        let remaining: i64 = conn
            .query_row("SELECT COUNT(*) FROM club_dna_definitions", [], |row| {
                row.get(0)
            })
            .expect("count definitions");
        assert_eq!(remaining, 0);
    }
}
