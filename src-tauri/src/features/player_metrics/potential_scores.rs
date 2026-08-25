use std::collections::HashMap;

#[cfg(test)]
use std::cell::Cell;

use rusqlite::{params, params_from_iter, types::Value, OptionalExtension, Transaction};

use crate::features::scoring::{
    catalog::{all_roles, DUMP_ATTRIBUTE_KEYS},
    projection::project_attributes,
    score::score_role,
};

pub const PROJECTION_MODEL_VERSION: i64 = 2;

struct PlayerForProjection {
    uid: i64,
    ca: i64,
    pa: i64,
    age: Option<i64>,
    positions_json: String,
    attributes_json: String,
}

#[cfg(test)]
thread_local! {
    static PROJECT_ATTRIBUTES_CALLS: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
fn reset_project_attributes_calls() {
    PROJECT_ATTRIBUTES_CALLS.with(|calls| calls.set(0));
}

#[cfg(test)]
fn project_attributes_call_count() -> usize {
    PROJECT_ATTRIBUTES_CALLS.with(Cell::get)
}

/// Rebuilds every potential-derived value for one snapshot.
pub(crate) fn rebuild_snapshot(tx: &Transaction<'_>, snapshot_id: i64) -> Result<(), String> {
    require_current_snapshot(tx, snapshot_id)?;
    clear_snapshot(tx, snapshot_id)?;
    let players = load_players(tx, snapshot_id)?;
    persist_players(tx, snapshot_id, &players)?;
    assert_current_snapshot_complete(tx, snapshot_id)
}

/// Replaces one player's potential-derived values after a source-player change.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn replace_player(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    player_uid: i64,
) -> Result<(), String> {
    require_current_snapshot(tx, snapshot_id)?;
    let player = load_player(tx, snapshot_id, player_uid)?
        .ok_or_else(|| "Potential-score player does not exist".to_string())?;
    tx.execute(
        "DELETE FROM player_potential_role_scores WHERE snapshot_id = ?1 AND uid = ?2",
        params![snapshot_id, player_uid],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "UPDATE players
         SET potential_attributes_json = NULL, potential_projection_model_version = NULL
         WHERE snapshot_id = ?1 AND uid = ?2",
        params![snapshot_id, player_uid],
    )
    .map_err(|error| error.to_string())?;
    persist_players(tx, snapshot_id, &[player])?;
    assert_current_snapshot_complete(tx, snapshot_id)
}

/// Clears potential-derived state for one snapshot.
pub(crate) fn clear_snapshot(tx: &Transaction<'_>, snapshot_id: i64) -> Result<(), String> {
    tx.execute(
        "DELETE FROM player_potential_role_scores WHERE snapshot_id = ?1",
        [snapshot_id],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "UPDATE players
         SET potential_attributes_json = NULL, potential_projection_model_version = NULL
         WHERE snapshot_id = ?1",
        [snapshot_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Clears potential-derived state from every retained snapshot in one save.
pub(crate) fn clear_non_current_snapshots(
    tx: &Transaction<'_>,
    save_id: i64,
) -> Result<(), String> {
    tx.execute(
        "DELETE FROM player_potential_role_scores
         WHERE snapshot_id IN (
             SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 0
         )",
        [save_id],
    )
    .map_err(|error| error.to_string())?;
    tx.execute(
        "UPDATE players
         SET potential_attributes_json = NULL, potential_projection_model_version = NULL
         WHERE snapshot_id IN (
             SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 0
         )",
        [save_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Deletes the disposable cache and materializes each existing effective current snapshot.
pub(crate) fn backfill_current_snapshots(tx: &Transaction<'_>) -> Result<(), String> {
    let save_ids = tx
        .prepare("SELECT id FROM saves ORDER BY id")
        .map_err(|error| error.to_string())?
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    for save_id in save_ids {
        clear_non_current_snapshots(tx, save_id)?;
        let current_snapshot_id = tx
            .query_row(
                "SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 1",
                [save_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        if let Some(snapshot_id) = current_snapshot_id {
            rebuild_snapshot(tx, snapshot_id)?;
        }
    }
    Ok(())
}

/// Verifies that an already-resolved current snapshot has complete persisted potential state.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn assert_current_snapshot_complete(
    tx: &Transaction<'_>,
    snapshot_id: i64,
) -> Result<(), String> {
    let roles = all_roles();
    let role_placeholders = (1..=roles.len())
        .map(|index| format!("(?{index})"))
        .collect::<Vec<_>>()
        .join(", ");
    let attribute_placeholders = (roles.len() + 1..=roles.len() + DUMP_ATTRIBUTE_KEYS.len())
        .map(|index| format!("(?{index})"))
        .collect::<Vec<_>>()
        .join(", ");
    let snapshot_parameter = roles.len() + DUMP_ATTRIBUTE_KEYS.len() + 1;
    let version_parameter = snapshot_parameter + 1;
    let sql = format!(
        "WITH expected_roles(role_id) AS (VALUES {role_placeholders}),
              expected_attributes(attribute_key) AS (VALUES {attribute_placeholders})
         SELECT
             NOT EXISTS(
                 SELECT 1 FROM snapshots WHERE id = ?{snapshot_parameter} AND is_current = 1
             )
             OR EXISTS(
                 SELECT 1
                 FROM players p
                 WHERE p.snapshot_id = ?{snapshot_parameter}
                   AND (
                       p.potential_attributes_json IS NULL
                       OR CASE
                           WHEN json_valid(p.potential_attributes_json) = 1
                           THEN json_type(p.potential_attributes_json) <> 'object'
                           ELSE 1
                       END
                       OR EXISTS(
                           SELECT 1
                           FROM expected_attributes attribute
                           WHERE NOT EXISTS(
                               SELECT 1
                               FROM json_each(CASE
                                   WHEN json_valid(p.potential_attributes_json) = 1 THEN
                                       CASE json_type(p.potential_attributes_json)
                                           WHEN 'object' THEN p.potential_attributes_json
                                           ELSE '{{}}'
                                       END
                                   ELSE '{{}}'
                               END) projected
                               WHERE projected.key = attribute.attribute_key
                           )
                       )
                       OR EXISTS(
                           SELECT 1
                           FROM json_each(CASE
                               WHEN json_valid(p.potential_attributes_json) = 1 THEN
                                   CASE json_type(p.potential_attributes_json)
                                       WHEN 'object' THEN p.potential_attributes_json
                                       ELSE '{{}}'
                                   END
                               ELSE '{{}}'
                           END) projected
                           WHERE NOT EXISTS(
                               SELECT 1
                               FROM expected_attributes attribute
                               WHERE attribute.attribute_key = projected.key
                           )
                           OR (
                               projected.type <> 'null'
                               AND (
                                   projected.type <> 'integer'
                                   OR projected.value NOT BETWEEN 1 AND 20
                               )
                           )
                       )
                       OR p.potential_projection_model_version IS NOT ?{version_parameter}
                       OR EXISTS(
                           SELECT 1
                           FROM expected_roles role
                           WHERE NOT EXISTS(
                               SELECT 1
                               FROM player_potential_role_scores score
                               WHERE score.snapshot_id = p.snapshot_id
                                 AND score.uid = p.uid
                                 AND score.role_id = role.role_id
                                 AND score.projection_model_version = ?{version_parameter}
                           )
                       )
                   )
             )",
    );
    let mut values = roles
        .iter()
        .map(|role| Value::Text(role.role_id.to_string()))
        .collect::<Vec<_>>();
    values.extend(
        DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| Value::Text((*key).to_string())),
    );
    values.push(Value::Integer(snapshot_id));
    values.push(Value::Integer(PROJECTION_MODEL_VERSION));
    let incomplete: bool = tx
        .query_row(&sql, params_from_iter(values.iter()), |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if incomplete {
        Err("Current potential snapshot is incomplete".to_string())
    } else {
        Ok(())
    }
}

fn require_current_snapshot(tx: &Transaction<'_>, snapshot_id: i64) -> Result<(), String> {
    let is_current: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM snapshots WHERE id = ?1 AND is_current = 1)",
            [snapshot_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if is_current {
        Ok(())
    } else {
        Err("Potential scores require a current snapshot".to_string())
    }
}

fn load_players(
    tx: &Transaction<'_>,
    snapshot_id: i64,
) -> Result<Vec<PlayerForProjection>, String> {
    let mut statement = tx
        .prepare(
            "SELECT uid, ca, pa, age, positions_json, attributes_json
             FROM players WHERE snapshot_id = ?1 ORDER BY uid",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([snapshot_id], player_from_row)
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[cfg_attr(not(test), allow(dead_code))]
fn load_player(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    player_uid: i64,
) -> Result<Option<PlayerForProjection>, String> {
    tx.query_row(
        "SELECT uid, ca, pa, age, positions_json, attributes_json
         FROM players WHERE snapshot_id = ?1 AND uid = ?2",
        params![snapshot_id, player_uid],
        player_from_row,
    )
    .optional()
    .map_err(|error| error.to_string())
}

fn player_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlayerForProjection> {
    Ok(PlayerForProjection {
        uid: row.get(0)?,
        ca: row.get(1)?,
        pa: row.get(2)?,
        age: row.get(3)?,
        positions_json: row.get(4)?,
        attributes_json: row.get(5)?,
    })
}

fn persist_players(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    players: &[PlayerForProjection],
) -> Result<(), String> {
    let mut attributes_statement = tx
        .prepare(
            "UPDATE players
             SET potential_attributes_json = ?3, potential_projection_model_version = ?4
             WHERE snapshot_id = ?1 AND uid = ?2",
        )
        .map_err(|error| error.to_string())?;
    let mut scores_statement = tx
        .prepare(
            "INSERT INTO player_potential_role_scores (
                snapshot_id, uid, role_id, score, projection_model_version
             ) VALUES (?1, ?2, ?3, ?4, ?5)",
        )
        .map_err(|error| error.to_string())?;

    for player in players {
        let attributes = serde_json::from_str::<HashMap<String, Option<u8>>>(
            &player.attributes_json,
        )
        .map_err(|error| format!("invalid player {} attributes JSON: {error}", player.uid))?;
        let positions = serde_json::from_str::<HashMap<String, Option<i64>>>(
            &player.positions_json,
        )
        .map_err(|error| format!("invalid player {} positions JSON: {error}", player.uid))?;
        #[cfg(test)]
        PROJECT_ATTRIBUTES_CALLS.with(|calls| calls.set(calls.get() + 1));
        let projected = project_attributes(
            &attributes,
            player.ca,
            player.pa,
            player.age,
            positions
                .iter()
                .map(|(position, familiarity)| (position.as_str(), *familiarity)),
        );
        let projected_json = serde_json::to_string(&projected).map_err(|error| {
            format!(
                "serialize player {} projected attributes: {error}",
                player.uid
            )
        })?;
        attributes_statement
            .execute(params![
                snapshot_id,
                player.uid,
                projected_json,
                PROJECTION_MODEL_VERSION,
            ])
            .map_err(|error| error.to_string())?;
        for role in all_roles() {
            scores_statement
                .execute(params![
                    snapshot_id,
                    player.uid,
                    role.role_id,
                    score_role(&projected, role).map(i64::from),
                    PROJECTION_MODEL_VERSION,
                ])
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use serde_json::{Map, Value as JsonValue};

    use super::*;
    use crate::{db::migrations, features::scoring::catalog::DUMP_ATTRIBUTE_KEYS};

    fn connection() -> Connection {
        let conn = Connection::open_in_memory().expect("open database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("enable foreign keys");
        migrations::apply(&conn).expect("apply migrations");
        conn
    }

    fn complete_attributes() -> String {
        let attributes = DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), Some(10_u8)))
            .collect::<HashMap<_, _>>();
        serde_json::to_string(&attributes).expect("serialize attributes")
    }

    fn snapshot_with_players(player_uids: &[i64]) -> (Connection, i64) {
        let conn = connection();
        conn.execute("INSERT INTO saves (name) VALUES ('Potential scores')", [])
            .expect("insert save");
        let save_id = conn.last_insert_rowid();
        let snapshot_id: i64 = conn
            .query_row(
                "INSERT INTO snapshots (
                    save_id, is_current, schema_version, generated_at_utc, game_version,
                    supported_game_version, bridge_version, protocol_version, game_date_source,
                    scan_truncated, player_count
                 ) VALUES (?1, ?2, 8, '2026-08-18T00:00:00Z', '26.3.2', '26.3',
                           '0.4.0', 1, 'memory', 0, 1)
                 RETURNING id",
                params![save_id, true],
                |row| row.get(0),
            )
            .expect("insert snapshot");
        for uid in player_uids {
            conn.execute(
                "INSERT INTO players (
                    snapshot_id, uid, ca, pa, name, birth_year, birth_day_of_year, age,
                    nationalities_json, preferred_foot, positions_json, attributes_json,
                    hidden_attributes_json, personality_json
                 ) VALUES (?1, ?2, 100, 140, 'Potential player', 2006, 1, 20, '[]', 'Right',
                           '{\"ST\":20}', ?3, '{}', '{}')",
                params![snapshot_id, uid, complete_attributes()],
            )
            .expect("insert player");
        }
        (conn, snapshot_id)
    }

    fn snapshot_with_persisted_scores() -> (Connection, i64) {
        let (conn, snapshot_id) = snapshot_with_players(&[42]);
        let tx = conn
            .unchecked_transaction()
            .expect("start writer transaction");
        rebuild_snapshot(&tx, snapshot_id).expect("persist potential scores");
        tx.commit().expect("commit potential scores");
        (conn, snapshot_id)
    }

    type DerivedState = (Option<String>, Option<i64>, Vec<(String, Option<i64>, i64)>);

    fn derived_state(conn: &Connection, snapshot_id: i64) -> DerivedState {
        let fields = conn
            .query_row(
                "SELECT potential_attributes_json, potential_projection_model_version
                 FROM players WHERE snapshot_id = ?1 AND uid = 42",
                [snapshot_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read projected fields");
        let rows = conn
            .prepare(
                "SELECT role_id, score, projection_model_version
                 FROM player_potential_role_scores
                 WHERE snapshot_id = ?1 AND uid = 42 ORDER BY role_id",
            )
            .expect("prepare derived rows")
            .query_map([snapshot_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .expect("query derived rows")
            .collect::<Result<_, _>>()
            .expect("read derived rows");
        (fields.0, fields.1, rows)
    }

    fn deny_derived_writes(conn: &Connection) {
        conn.execute_batch(
            "CREATE TRIGGER deny_projected_player_updates
             BEFORE UPDATE OF potential_attributes_json, potential_projection_model_version ON players
             BEGIN SELECT RAISE(ABORT, 'derived player writes are forbidden'); END;
             CREATE TRIGGER deny_potential_score_inserts
             BEFORE INSERT ON player_potential_role_scores
             BEGIN SELECT RAISE(ABORT, 'derived score writes are forbidden'); END;
             CREATE TRIGGER deny_potential_score_updates
             BEFORE UPDATE ON player_potential_role_scores
             BEGIN SELECT RAISE(ABORT, 'derived score writes are forbidden'); END;
             CREATE TRIGGER deny_potential_score_deletes
             BEFORE DELETE ON player_potential_role_scores
             BEGIN SELECT RAISE(ABORT, 'derived score writes are forbidden'); END;",
        )
        .expect("deny derived writes");
    }

    fn assert_rejected_without_writes(conn: &Connection, snapshot_id: i64) {
        let before = derived_state(conn, snapshot_id);
        deny_derived_writes(conn);
        let tx = conn
            .unchecked_transaction()
            .expect("start assertion transaction");
        assert_eq!(
            assert_current_snapshot_complete(&tx, snapshot_id),
            Err("Current potential snapshot is incomplete".to_string())
        );
        tx.commit().expect("commit read-only assertion");
        assert_eq!(derived_state(conn, snapshot_id), before);
    }

    fn mutate_projected_json(
        projected_json: &str,
        mutate: impl FnOnce(&mut Map<String, JsonValue>),
    ) -> String {
        let mut projected = serde_json::from_str(projected_json).expect("parse projected map");
        mutate(&mut projected);
        serde_json::to_string(&projected).expect("serialize corrupted projected map")
    }

    fn assert_projected_corruption_rejected(
        model_version: Option<i64>,
        corrupt_json: impl FnOnce(&str) -> Option<String>,
    ) {
        let (conn, snapshot_id) = snapshot_with_persisted_scores();
        let complete_projected_json = derived_state(&conn, snapshot_id)
            .0
            .expect("complete projected map");
        conn.execute(
            "UPDATE players
             SET potential_attributes_json = ?3, potential_projection_model_version = ?4
             WHERE snapshot_id = ?1 AND uid = ?2",
            params![
                snapshot_id,
                42,
                corrupt_json(&complete_projected_json),
                model_version
            ],
        )
        .expect("corrupt projected map");
        assert_rejected_without_writes(&conn, snapshot_id);
    }

    #[test]
    fn rebuild_projects_each_player_once() {
        for (player_uids, expected_calls) in [(vec![42], 1), (vec![42, 43], 2)] {
            let (conn, snapshot_id) = snapshot_with_players(&player_uids);
            reset_project_attributes_calls();

            let tx = conn
                .unchecked_transaction()
                .expect("start writer transaction");
            rebuild_snapshot(&tx, snapshot_id).expect("persist potential scores");
            tx.commit().expect("commit potential scores");

            assert_eq!(project_attributes_call_count(), expected_calls);
        }
    }

    #[test]
    fn rebuild_scores_every_role_from_the_one_persisted_projection_map() {
        let (conn, snapshot_id) = snapshot_with_persisted_scores();
        let state = derived_state(&conn, snapshot_id);
        let projected = serde_json::from_str::<HashMap<String, Option<u8>>>(
            state.0.as_deref().expect("persisted projected attributes"),
        )
        .expect("parse persisted projected attributes");

        assert_eq!(state.1, Some(PROJECTION_MODEL_VERSION));
        assert_eq!(state.2.len(), all_roles().len());
        for (role_id, score, version) in state.2 {
            let role = all_roles()
                .iter()
                .find(|role| role.role_id == role_id)
                .expect("catalog role");
            assert_eq!(score, score_role(&projected, role).map(i64::from));
            assert_eq!(version, PROJECTION_MODEL_VERSION);
        }

        let tx = conn
            .unchecked_transaction()
            .expect("start replacement transaction");
        replace_player(&tx, snapshot_id, 42).expect("replace one player");
        tx.commit().expect("commit player replacement");
        assert_eq!(derived_state(&conn, snapshot_id).2.len(), all_roles().len());
    }

    #[test]
    fn rebuild_persists_nullable_scores_as_sql_null() {
        let (conn, snapshot_id) = snapshot_with_players(&[42]);
        let mut attributes =
            serde_json::from_str::<HashMap<String, Option<u8>>>(&complete_attributes())
                .expect("parse complete attributes");
        attributes.insert("Composure".to_string(), None);
        conn.execute(
            "UPDATE players SET attributes_json = ?3 WHERE snapshot_id = ?1 AND uid = ?2",
            params![
                snapshot_id,
                42,
                serde_json::to_string(&attributes).expect("serialize nullable attributes")
            ],
        )
        .expect("store nullable source attribute");

        let tx = conn
            .unchecked_transaction()
            .expect("start writer transaction");
        rebuild_snapshot(&tx, snapshot_id).expect("persist potential scores");
        assert_current_snapshot_complete(&tx, snapshot_id)
            .expect("complete snapshot accepts nullable score rows");
        tx.commit().expect("commit potential scores");

        let state = derived_state(&conn, snapshot_id);
        let projected = serde_json::from_str::<HashMap<String, Option<u8>>>(
            state.0.as_deref().expect("persisted projected attributes"),
        )
        .expect("parse persisted projected attributes");
        assert_eq!(projected.get("Composure"), Some(&None));
        assert_eq!(state.2.len(), all_roles().len());
        for (role_id, score, _) in state.2 {
            let role = all_roles()
                .iter()
                .find(|role| role.role_id == role_id)
                .expect("catalog role");
            assert_eq!(score, score_role(&projected, role).map(i64::from));
        }

        let affected_role = all_roles()
            .iter()
            .find(|role| role.role_id == "ball_playing_goalkeeper_ip")
            .expect("role requiring Composure");
        assert!(affected_role.secondary.contains(&"Composure"));
        let affected_score: Option<i64> = conn
            .query_row(
                "SELECT score FROM player_potential_role_scores
                 WHERE snapshot_id = ?1 AND uid = ?2 AND role_id = ?3",
                params![snapshot_id, 42, affected_role.role_id],
                |row| row.get(0),
            )
            .expect("read nullable score");
        assert_eq!(affected_score, None);
    }

    #[test]
    fn complete_current_snapshot_passes_the_read_only_assertion() {
        let (conn, snapshot_id) = snapshot_with_persisted_scores();
        let tx = conn
            .unchecked_transaction()
            .expect("start assertion transaction");
        assert_current_snapshot_complete(&tx, snapshot_id).expect("complete current snapshot");
        tx.commit().expect("commit read-only assertion");
    }

    #[test]
    fn assertion_rejects_missing_or_wrong_version_roles_without_writes() {
        let (conn, snapshot_id) = snapshot_with_persisted_scores();
        let missing_role_id = all_roles()[0].role_id;
        conn.execute(
            "DELETE FROM player_potential_role_scores
             WHERE snapshot_id = ?1 AND uid = 42 AND role_id = ?2",
            params![snapshot_id, missing_role_id],
        )
        .expect("delete expected role");
        conn.execute(
            "INSERT INTO player_potential_role_scores (
                snapshot_id, uid, role_id, score, projection_model_version
             ) VALUES (?1, 42, 'obsolete_role', 99, ?2)",
            params![snapshot_id, PROJECTION_MODEL_VERSION],
        )
        .expect("add extra role that preserves the total row count");
        assert_rejected_without_writes(&conn, snapshot_id);

        let (conn, snapshot_id) = snapshot_with_persisted_scores();
        conn.execute(
            "UPDATE player_potential_role_scores
             SET projection_model_version = ?3
             WHERE snapshot_id = ?1 AND uid = 42 AND role_id = ?2",
            params![
                snapshot_id,
                all_roles()[0].role_id,
                PROJECTION_MODEL_VERSION - 1
            ],
        )
        .expect("make role stale");
        assert_rejected_without_writes(&conn, snapshot_id);
    }

    #[test]
    fn assertion_rejects_independently_corrupt_projected_maps_without_writes() {
        assert_projected_corruption_rejected(Some(PROJECTION_MODEL_VERSION), |_| None);
        assert_projected_corruption_rejected(Some(PROJECTION_MODEL_VERSION), |_| {
            Some("not-json".to_string())
        });
        assert_projected_corruption_rejected(Some(PROJECTION_MODEL_VERSION), |json| {
            Some(mutate_projected_json(json, |map| {
                map.remove(DUMP_ATTRIBUTE_KEYS[0]);
            }))
        });
        assert_projected_corruption_rejected(Some(PROJECTION_MODEL_VERSION), |json| {
            Some(mutate_projected_json(json, |map| {
                map.insert(
                    DUMP_ATTRIBUTE_KEYS[0].to_string(),
                    JsonValue::String("fast".to_string()),
                );
            }))
        });
        assert_projected_corruption_rejected(Some(PROJECTION_MODEL_VERSION), |json| {
            Some(mutate_projected_json(json, |map| {
                map.insert(DUMP_ATTRIBUTE_KEYS[0].to_string(), JsonValue::from(21));
            }))
        });
        assert_projected_corruption_rejected(Some(PROJECTION_MODEL_VERSION), |json| {
            Some(mutate_projected_json(json, |map| {
                map.insert("unexpected".to_string(), JsonValue::from(10));
            }))
        });
        assert_projected_corruption_rejected(Some(PROJECTION_MODEL_VERSION - 1), |json| {
            Some(json.to_string())
        });
    }

    #[test]
    fn assertion_rejects_non_current_snapshots_without_writes() {
        let (conn, snapshot_id) = snapshot_with_persisted_scores();
        conn.execute(
            "UPDATE snapshots SET is_current = 0 WHERE id = ?1",
            [snapshot_id],
        )
        .expect("corrupt current marker");
        assert_rejected_without_writes(&conn, snapshot_id);
    }
}
