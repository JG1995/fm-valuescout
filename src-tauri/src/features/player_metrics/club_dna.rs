#![cfg_attr(not(test), allow(dead_code))]

use std::collections::HashSet;

use rusqlite::{
    params, params_from_iter, types::Value, Connection, OptionalExtension, Transaction,
};
use serde_json::{Map, Value as JsonValue};

use crate::features::club_dna::service::{self, ClubDnaDefinition};

pub const SCORE_MODEL_VERSION: i64 = 1;

const MATERIALIZATION_BATCH_SIZE: usize = 250;

struct PlayerForScore {
    uid: i64,
    attributes_json: String,
    hidden_attributes_json: String,
    personality_json: String,
}

/// Materializes exact-version Club DNA rows for the requested player UIDs.
///
/// Callers retain their synchronous database mutex while this runs. Each batch is
/// scored before its short write transaction, so an interrupted request leaves
/// only complete, reusable derived batches behind.
pub fn materialize_player_scores(
    conn: &Connection,
    snapshot_id: i64,
    player_uids: &[i64],
) -> Result<(), String> {
    let unique_uids = requested_uids(player_uids)?;
    let definition = definition_for_snapshot(conn, snapshot_id)?;
    let Some(definition) = definition else {
        return Ok(());
    };
    service::validate_attribute_ids(&definition.attribute_ids)
        .map_err(|_| "Stored Club DNA definition is invalid".to_string())?;
    for uid_batch in unique_uids.chunks(MATERIALIZATION_BATCH_SIZE) {
        let players = load_uncached_players(conn, snapshot_id, uid_batch, definition.version)?;
        if players.is_empty() {
            continue;
        }
        let scores = players
            .iter()
            .map(|player| {
                score_validated_club_dna(
                    &definition,
                    &player.attributes_json,
                    &player.hidden_attributes_json,
                    &player.personality_json,
                )
                .map(|score| (player.uid, score))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let tx = conn
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;
        persist_scores(&tx, snapshot_id, definition.version, &scores)?;
        tx.commit().map_err(|error| error.to_string())?;
    }

    Ok(())
}

/// Deletes a player's disposable Club DNA rows with the source-data transaction.
pub fn invalidate_player_cache(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    player_uid: i64,
) -> Result<(), rusqlite::Error> {
    tx.execute(
        "DELETE FROM club_dna_scores WHERE snapshot_id = ?1 AND uid = ?2",
        params![snapshot_id, player_uid],
    )?;
    Ok(())
}

/// Scores one player from a validated Club DNA definition without database access.
pub fn score_club_dna(
    definition: &ClubDnaDefinition,
    attributes_json: &str,
    hidden_attributes_json: &str,
    personality_json: &str,
) -> Result<Option<i64>, String> {
    service::validate_attribute_ids(&definition.attribute_ids)
        .map_err(|_| "Stored Club DNA definition is invalid".to_string())?;
    score_validated_club_dna(
        definition,
        attributes_json,
        hidden_attributes_json,
        personality_json,
    )
}

fn score_validated_club_dna(
    definition: &ClubDnaDefinition,
    attributes_json: &str,
    hidden_attributes_json: &str,
    personality_json: &str,
) -> Result<Option<i64>, String> {
    let attributes = parse_object(attributes_json, "attributes")?;
    let hidden_attributes = parse_object(hidden_attributes_json, "hidden attributes")?;
    let personality = parse_object(personality_json, "personality")?;
    let mut total = 0_i64;

    for attribute_id in &definition.attribute_ids {
        let value = if let Some(key) = attribute_id.strip_prefix("attr.") {
            attributes.get(key)
        } else if let Some(key) = attribute_id.strip_prefix("hidden.") {
            hidden_attributes.get(key)
        } else if let Some(key) = attribute_id.strip_prefix("personality.") {
            personality.get(key)
        } else {
            return Err("Stored Club DNA definition is invalid".to_string());
        };
        let Some(value) = value.and_then(JsonValue::as_i64) else {
            return Ok(None);
        };
        if !(1..=20).contains(&value) {
            return Ok(None);
        }
        total += value;
    }

    Ok(Some(
        ((total * 5) as f64 / definition.attribute_ids.len() as f64).round() as i64,
    ))
}

fn definition_for_snapshot(
    conn: &Connection,
    snapshot_id: i64,
) -> Result<Option<ClubDnaDefinition>, String> {
    let row: Option<(Option<String>, Option<i64>)> = conn
        .query_row(
            "SELECT d.attribute_ids_json, d.definition_version
             FROM snapshots s
             LEFT JOIN club_dna_definitions d ON d.save_id = s.save_id
             WHERE s.id = ?1",
            [snapshot_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some((attribute_ids_json, version)) = row else {
        return Err("Club DNA snapshot does not exist".to_string());
    };
    let Some(attribute_ids_json) = attribute_ids_json else {
        return Ok(None);
    };
    let attribute_ids = serde_json::from_str(&attribute_ids_json)
        .map_err(|_| "Stored Club DNA definition is invalid".to_string())?;
    let version = version.ok_or_else(|| "Stored Club DNA definition is invalid".to_string())?;
    if version <= 0 {
        return Err("Stored Club DNA definition is invalid".to_string());
    }
    Ok(Some(ClubDnaDefinition {
        attribute_ids,
        version,
    }))
}

fn requested_uids(player_uids: &[i64]) -> Result<Vec<i64>, String> {
    let mut seen = HashSet::new();
    let mut unique_uids = Vec::with_capacity(player_uids.len());
    for uid in player_uids {
        if *uid <= 0 {
            return Err("Club DNA player UID is invalid".to_string());
        }
        if seen.insert(*uid) {
            unique_uids.push(*uid);
        }
    }
    Ok(unique_uids)
}

fn load_uncached_players(
    conn: &Connection,
    snapshot_id: i64,
    player_uids: &[i64],
    definition_version: i64,
) -> Result<Vec<PlayerForScore>, String> {
    let placeholders = player_uids
        .iter()
        .enumerate()
        .map(|(index, _)| format!("?{}", index + 4))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT p.uid, p.attributes_json, p.hidden_attributes_json, p.personality_json
         FROM players p
         WHERE p.snapshot_id = ?1
           AND p.uid IN ({placeholders})
           AND NOT EXISTS (
               SELECT 1 FROM club_dna_scores cached
               WHERE cached.snapshot_id = p.snapshot_id
                 AND cached.uid = p.uid
                 AND cached.definition_version = ?2
                 AND cached.score_model_version = ?3
           )
         ORDER BY p.uid"
    );
    let mut values = vec![
        Value::Integer(snapshot_id),
        Value::Integer(definition_version),
        Value::Integer(SCORE_MODEL_VERSION),
    ];
    values.extend(player_uids.iter().copied().map(Value::Integer));
    let mut statement = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(params_from_iter(values.iter()), |row| {
            Ok(PlayerForScore {
                uid: row.get(0)?,
                attributes_json: row.get(1)?,
                hidden_attributes_json: row.get(2)?,
                personality_json: row.get(3)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn persist_scores(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    definition_version: i64,
    scores: &[(i64, Option<i64>)],
) -> Result<(), String> {
    let mut statement = tx
        .prepare(
            "INSERT INTO club_dna_scores (
                snapshot_id, uid, definition_version, score_model_version, score
             ) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(snapshot_id, uid, definition_version, score_model_version) DO UPDATE SET
                score = excluded.score",
        )
        .map_err(|error| error.to_string())?;
    for (uid, score) in scores {
        statement
            .execute(params![
                snapshot_id,
                uid,
                definition_version,
                SCORE_MODEL_VERSION,
                score,
            ])
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn parse_object(json: &str, source: &str) -> Result<Map<String, JsonValue>, String> {
    serde_json::from_str(json).map_err(|_| format!("Player {source} JSON is invalid"))
}

#[cfg(test)]
mod tests {
    use rusqlite::{params, Connection};

    use super::*;
    use crate::db::migrations;
    use crate::features::club_dna::service::set_club_dna;

    fn connection() -> Connection {
        let conn = Connection::open_in_memory().expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn insert_save(conn: &Connection) -> (i64, String) {
        let save_id: i64 = conn
            .query_row(
                "INSERT INTO saves (name, is_active) VALUES ('Club DNA', 1) RETURNING id",
                [],
                |row| row.get(0),
            )
            .expect("insert save");
        let token = conn
            .query_row(
                "SELECT context_token FROM saves WHERE id = ?1",
                [save_id],
                |row| row.get(0),
            )
            .expect("read token");
        (save_id, token)
    }

    fn insert_snapshot(conn: &Connection, save_id: i64) -> i64 {
        conn.query_row(
            "INSERT INTO snapshots (
                 save_id, is_current, schema_version, generated_at_utc,
                 game_version, supported_game_version, bridge_version,
                 protocol_version, game_date_source, scan_truncated, player_count
             ) VALUES (?1, 1, 8, '2026-08-18T00:00:00Z', '26.3.2',
                       '26.3', '0.4.0', 1, 'inGame', 0, 0)
             RETURNING id",
            [save_id],
            |row| row.get(0),
        )
        .expect("insert snapshot")
    }

    fn insert_player(
        conn: &Connection,
        snapshot_id: i64,
        uid: i64,
        attributes: &str,
        hidden: &str,
        personality: &str,
    ) {
        conn.execute(
            "INSERT INTO players (
                snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                nationalities_json, preferred_foot, positions_json, attributes_json,
                hidden_attributes_json, personality_json
             ) VALUES (?1, ?2, 100, 120, 'Player', 2000, 1, '[]', 'Right',
                '{}', ?3, ?4, ?5)",
            params![snapshot_id, uid, attributes, hidden, personality],
        )
        .expect("insert player");
    }

    fn definition(ids: &[&str]) -> ClubDnaDefinition {
        ClubDnaDefinition {
            attribute_ids: ids.iter().map(|id| (*id).to_string()).collect(),
            version: 1,
        }
    }

    fn cache_count(conn: &Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM club_dna_scores", [], |row| row.get(0))
            .expect("count cache rows")
    }

    #[test]
    fn score_requires_complete_integer_values_and_rounds_once_across_sources() {
        let complete_definition = definition(&[
            "attr.Acceleration",
            "attr.Handling",
            "hidden.Consistency",
            "personality.Ambition",
        ]);
        assert_eq!(
            score_club_dna(
                &complete_definition,
                r#"{"Acceleration":1,"Handling":20}"#,
                r#"{"Consistency":10}"#,
                r#"{"Ambition":11}"#,
            )
            .expect("score player"),
            Some(53)
        );

        for (ids, attributes, hidden, personality) in [
            (
                vec!["attr.Acceleration"],
                r#"{"Acceleration":0}"#,
                "{}",
                "{}",
            ),
            (vec!["attr.Handling"], r#"{"Handling":21}"#, "{}", "{}"),
            (
                vec!["hidden.Consistency"],
                "{}",
                r#"{"Consistency":0}"#,
                "{}",
            ),
            (
                vec!["hidden.Consistency"],
                "{}",
                r#"{"Consistency":21}"#,
                "{}",
            ),
            (
                vec!["personality.Ambition"],
                "{}",
                "{}",
                r#"{"Ambition":0}"#,
            ),
            (
                vec!["personality.Ambition"],
                "{}",
                "{}",
                r#"{"Ambition":21}"#,
            ),
            (vec!["attr.Acceleration"], "{}", "{}", "{}"),
            (
                vec!["hidden.Consistency"],
                "{}",
                r#"{"Consistency":null}"#,
                "{}",
            ),
            (
                vec!["personality.Ambition"],
                "{}",
                "{}",
                r#"{"Ambition":10.5}"#,
            ),
        ] {
            assert_eq!(
                score_club_dna(&definition(&ids), attributes, hidden, personality)
                    .expect("score invalid player"),
                None,
                "{ids:?} must make the complete score unavailable"
            );
        }
    }

    #[test]
    fn materialization_is_page_scoped_nullable_and_reuses_matching_versions() {
        let conn = connection();
        let (save_id, token) = insert_save(&conn);
        let snapshot_id = insert_snapshot(&conn, save_id);
        set_club_dna(
            &conn,
            save_id,
            &token,
            vec!["attr.Acceleration".to_string()],
        )
        .expect("set definition");
        insert_player(&conn, snapshot_id, 1, r#"{"Acceleration":10}"#, "{}", "{}");
        insert_player(&conn, snapshot_id, 2, "{}", "{}", "{}");
        insert_player(&conn, snapshot_id, 3, r#"{"Acceleration":20}"#, "{}", "{}");

        materialize_player_scores(&conn, snapshot_id, &[1, 2]).expect("materialize page");
        assert_eq!(cache_count(&conn), 2);
        let rows = conn
            .prepare("SELECT uid, score FROM club_dna_scores ORDER BY uid")
            .expect("prepare cache query")
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?))
            })
            .expect("query cache")
            .collect::<Result<Vec<_>, _>>()
            .expect("read cache");
        assert_eq!(rows, vec![(1, Some(50)), (2, None)]);

        conn.execute(
            "UPDATE players SET attributes_json = '{\"Acceleration\":20}'
             WHERE snapshot_id = ?1 AND uid = 1",
            [snapshot_id],
        )
        .expect("change player after cache");
        materialize_player_scores(&conn, snapshot_id, &[1, 2]).expect("reuse matching rows");
        assert_eq!(
            conn.query_row(
                "SELECT score FROM club_dna_scores WHERE uid = 1",
                [],
                |row| row.get::<_, i64>(0)
            )
            .expect("read reused score"),
            50
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM club_dna_scores WHERE uid = 3",
                [],
                |row| row.get::<_, i64>(0)
            )
            .expect("count untouched page player"),
            0
        );
    }

    #[test]
    fn materialization_replaces_stale_versions_and_leaves_invalid_requests_empty() {
        let conn = connection();
        let (save_id, token) = insert_save(&conn);
        let snapshot_id = insert_snapshot(&conn, save_id);
        insert_player(&conn, snapshot_id, 1, r#"{"Acceleration":10}"#, "{}", "{}");

        assert!(materialize_player_scores(&conn, snapshot_id, &[1]).is_ok());
        assert_eq!(cache_count(&conn), 0);
        assert!(materialize_player_scores(&conn, snapshot_id, &[-1]).is_err());
        assert_eq!(cache_count(&conn), 0);
        assert!(materialize_player_scores(&conn, snapshot_id + 1, &[1]).is_err());
        assert_eq!(cache_count(&conn), 0);

        set_club_dna(
            &conn,
            save_id,
            &token,
            vec!["attr.Acceleration".to_string()],
        )
        .expect("set definition");
        conn.execute(
            "INSERT INTO club_dna_scores (
                snapshot_id, uid, definition_version, score_model_version, score
             ) VALUES (?1, 1, 1, 99, 99), (?1, 1, 99, 1, 99)",
            [snapshot_id],
        )
        .expect("seed stale version rows");
        materialize_player_scores(&conn, snapshot_id, &[1]).expect("replace stale version rows");
        assert_eq!(cache_count(&conn), 3);
        assert_eq!(
            conn.query_row(
                "SELECT score FROM club_dna_scores
                 WHERE snapshot_id = ?1 AND uid = 1 AND definition_version = 1
                   AND score_model_version = ?2",
                params![snapshot_id, SCORE_MODEL_VERSION],
                |row| row.get::<_, i64>(0),
            )
            .expect("read current score"),
            50
        );
    }

    #[test]
    fn materialization_commits_bounded_batches_and_resumes_idempotently() {
        let conn = connection();
        let (save_id, token) = insert_save(&conn);
        let snapshot_id = insert_snapshot(&conn, save_id);
        set_club_dna(
            &conn,
            save_id,
            &token,
            vec!["attr.Acceleration".to_string()],
        )
        .expect("set definition");
        let uids = (1..=MATERIALIZATION_BATCH_SIZE as i64 + 2).collect::<Vec<_>>();
        for uid in &uids {
            insert_player(
                &conn,
                snapshot_id,
                *uid,
                r#"{"Acceleration":10}"#,
                "{}",
                "{}",
            );
        }
        conn.execute_batch(
            "CREATE TRIGGER fail_second_cache_batch
             BEFORE INSERT ON club_dna_scores
             WHEN NEW.uid = 252
             BEGIN
                 SELECT RAISE(ABORT, 'forced cache failure');
             END;",
        )
        .expect("create failure trigger");

        assert!(materialize_player_scores(&conn, snapshot_id, &uids).is_err());
        let cached_uids = conn
            .prepare("SELECT uid FROM club_dna_scores ORDER BY uid")
            .expect("prepare cache UID query")
            .query_map([], |row| row.get::<_, i64>(0))
            .expect("query cache UIDs")
            .collect::<Result<Vec<_>, _>>()
            .expect("collect cache UIDs");
        assert_eq!(
            cached_uids,
            (1..=MATERIALIZATION_BATCH_SIZE as i64).collect::<Vec<_>>()
        );
        conn.execute("DROP TRIGGER fail_second_cache_batch", [])
            .expect("drop failure trigger");
        materialize_player_scores(&conn, snapshot_id, &uids).expect("resume materialization");
        assert_eq!(cache_count(&conn), uids.len() as i64);
        materialize_player_scores(&conn, snapshot_id, &uids).expect("repeat materialization");
        assert_eq!(cache_count(&conn), uids.len() as i64);
    }
}
