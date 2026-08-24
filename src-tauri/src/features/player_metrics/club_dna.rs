use rusqlite::{params, OptionalExtension, Transaction};
use serde_json::{Map, Value as JsonValue};

use crate::features::club_dna::service::{self, ClubDnaDefinition};

pub const SCORE_MODEL_VERSION: i64 = 1;

const SCORE_WRITE_BATCH_SIZE: usize = 250;

struct PlayerForScore {
    uid: i64,
    attributes_json: String,
    hidden_attributes_json: String,
    personality_json: String,
}

/// Scores one player from a validated Club DNA definition without database access.
#[cfg_attr(not(test), allow(dead_code))]
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

pub(crate) fn definition_for_save(
    tx: &Transaction<'_>,
    save_id: i64,
) -> Result<Option<ClubDnaDefinition>, String> {
    let definition = tx
        .query_row(
            "SELECT attribute_ids_json, definition_version
             FROM club_dna_definitions WHERE save_id = ?1",
            [save_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .map(|(attribute_ids_json, version)| {
            let attribute_ids = serde_json::from_str::<Vec<String>>(&attribute_ids_json)
                .map_err(|_| "Stored Club DNA definition is invalid".to_string())?;
            if version <= 0 {
                return Err("Stored Club DNA definition is invalid".to_string());
            }
            service::validate_attribute_ids(&attribute_ids)
                .map_err(|_| "Stored Club DNA definition is invalid".to_string())?;
            Ok(ClubDnaDefinition {
                attribute_ids,
                version,
            })
        })
        .transpose()?;
    Ok(definition)
}

pub(crate) fn persist_save_scores(
    tx: &Transaction<'_>,
    save_id: i64,
    definition: &ClubDnaDefinition,
) -> Result<(), String> {
    let mut previous_snapshot_id = None;
    while let Some(snapshot_id) = next_snapshot_id(tx, save_id, previous_snapshot_id)? {
        persist_snapshot_scores(tx, snapshot_id, definition)?;
        previous_snapshot_id = Some(snapshot_id);
    }
    Ok(())
}

pub(crate) fn persist_snapshot_scores(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    definition: &ClubDnaDefinition,
) -> Result<(), String> {
    persist_snapshot_scores_in_batches(tx, snapshot_id, definition, |_| {})
}

fn next_snapshot_id(
    tx: &Transaction<'_>,
    save_id: i64,
    previous_snapshot_id: Option<i64>,
) -> Result<Option<i64>, String> {
    tx.query_row(
        "SELECT id FROM snapshots
         WHERE save_id = ?1 AND (?2 IS NULL OR id > ?2)
         ORDER BY id LIMIT 1",
        params![save_id, previous_snapshot_id],
        |row| row.get(0),
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn persist_snapshot_scores_in_batches(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    definition: &ClubDnaDefinition,
    mut observe_batch: impl FnMut(usize),
) -> Result<(), String> {
    let mut previous_player_uid = None;
    loop {
        let players = load_player_score_batch(tx, snapshot_id, previous_player_uid)?;
        let Some(last_player_uid) = players.last().map(|player| player.uid) else {
            return Ok(());
        };
        observe_batch(players.len());
        persist_player_scores(tx, snapshot_id, definition, &players)?;
        previous_player_uid = Some(last_player_uid);
    }
}

fn load_player_score_batch(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    previous_player_uid: Option<i64>,
) -> Result<Vec<PlayerForScore>, String> {
    let mut statement = tx
        .prepare(
            "SELECT uid, attributes_json, hidden_attributes_json, personality_json
             FROM players
             WHERE snapshot_id = ?1 AND (?2 IS NULL OR uid > ?2)
             ORDER BY uid
             LIMIT ?3",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(
            params![
                snapshot_id,
                previous_player_uid,
                SCORE_WRITE_BATCH_SIZE as i64
            ],
            |row| {
                Ok(PlayerForScore {
                    uid: row.get(0)?,
                    attributes_json: row.get(1)?,
                    hidden_attributes_json: row.get(2)?,
                    personality_json: row.get(3)?,
                })
            },
        )
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

pub(crate) fn persist_player_score(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    player_uid: i64,
    definition: &ClubDnaDefinition,
) -> Result<(), String> {
    let player = tx
        .query_row(
            "SELECT attributes_json, hidden_attributes_json, personality_json
             FROM players WHERE snapshot_id = ?1 AND uid = ?2",
            params![snapshot_id, player_uid],
            |row| {
                Ok(PlayerForScore {
                    uid: player_uid,
                    attributes_json: row.get(0)?,
                    hidden_attributes_json: row.get(1)?,
                    personality_json: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Club DNA player does not exist".to_string())?;
    persist_player_scores(tx, snapshot_id, definition, std::slice::from_ref(&player))
}

fn persist_player_scores(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    definition: &ClubDnaDefinition,
    players: &[PlayerForScore],
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
    for player in players {
        let score = score_validated_club_dna(
            definition,
            &player.attributes_json,
            &player.hidden_attributes_json,
            &player.personality_json,
        )?;
        statement
            .execute(params![
                snapshot_id,
                player.uid,
                definition.version,
                SCORE_MODEL_VERSION,
                score,
            ])
            .map_err(|error| error.to_string())?;
    }
    Ok(())
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

fn parse_object(json: &str, source: &str) -> Result<Map<String, JsonValue>, String> {
    serde_json::from_str(json).map_err(|_| format!("Player {source} JSON is invalid"))
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;

    use super::*;
    use crate::db::migrations;

    fn definition(ids: &[&str]) -> ClubDnaDefinition {
        ClubDnaDefinition {
            attribute_ids: ids.iter().map(|id| (*id).to_string()).collect(),
            version: 1,
        }
    }

    fn connection() -> Connection {
        let conn = Connection::open_in_memory().expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn insert_snapshot(conn: &Connection) -> i64 {
        let save_id: i64 = conn
            .query_row(
                "INSERT INTO saves (name, is_active) VALUES ('Club DNA', 1) RETURNING id",
                [],
                |row| row.get(0),
            )
            .expect("insert save");
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

    fn insert_player(conn: &Connection, snapshot_id: i64, uid: i64) {
        conn.execute(
            "INSERT INTO players (
                snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year,
                nationalities_json, preferred_foot, positions_json, attributes_json,
                hidden_attributes_json, personality_json
             ) VALUES (?1, ?2, 100, 120, 'Player', 2000, 1, '[]', 'Right',
                '{}', '{\"Acceleration\":14,\"Pace\":15,\"Determination\":12}',
                '{\"Consistency\":12}', '{\"Ambition\":14}')",
            params![snapshot_id, uid],
        )
        .expect("insert player");
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
    fn persists_realistic_cohorts_in_fixed_size_score_buffers() {
        let mut conn = connection();
        let snapshot_id = insert_snapshot(&conn);
        let player_count = SCORE_WRITE_BATCH_SIZE * 3 + 7;
        for uid in 1..=player_count as i64 {
            insert_player(&conn, snapshot_id, uid);
        }

        let tx = conn.transaction().expect("start score transaction");
        let mut max_buffered_players = 0;
        persist_snapshot_scores_in_batches(
            &tx,
            snapshot_id,
            &definition(&[
                "attr.Acceleration",
                "hidden.Consistency",
                "personality.Ambition",
            ]),
            |batch_size| max_buffered_players = max_buffered_players.max(batch_size),
        )
        .expect("persist cohort scores");
        tx.commit().expect("commit cohort scores");

        assert_eq!(max_buffered_players, SCORE_WRITE_BATCH_SIZE);
        assert!(max_buffered_players < player_count);
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM club_dna_scores", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("count persisted cohort scores"),
            player_count as i64
        );
    }
}
