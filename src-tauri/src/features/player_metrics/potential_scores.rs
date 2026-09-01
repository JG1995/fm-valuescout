use std::collections::HashMap;

#[cfg(test)]
use std::cell::Cell;

use rusqlite::{
    params, params_from_iter, types::Value, Connection, OptionalExtension, Transaction,
};

use crate::features::player_metrics::compact;
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

/// Rebuilds every potential-derived value for one snapshot: the current-only
/// compact rows and the projected player attributes.
pub(crate) fn rebuild_snapshot(tx: &Transaction<'_>, snapshot_id: i64) -> Result<(), String> {
    require_current_snapshot(tx, snapshot_id)?;
    clear_snapshot(tx, snapshot_id)?;
    let players = load_players(tx, snapshot_id)?;
    let compact_rows = persist_players(tx, snapshot_id, &players)?;
    compact::persist_rows(tx, snapshot_id, &compact_rows)?;
    compact::assert_snapshot_complete(tx, snapshot_id)?;
    assert_current_snapshot_complete(tx, snapshot_id)
}

/// Replaces one player's compact row and projected attributes after a source-player change.
pub(crate) fn replace_player(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    player_uid: i64,
) -> Result<(), String> {
    require_current_snapshot(tx, snapshot_id)?;
    let player = load_player(tx, snapshot_id, player_uid)?
        .ok_or_else(|| "Potential-score player does not exist".to_string())?;
    tx.execute(
        "UPDATE players
         SET potential_attributes_json = NULL, potential_projection_model_version = NULL
         WHERE snapshot_id = ?1 AND uid = ?2",
        params![snapshot_id, player_uid],
    )
    .map_err(|error| error.to_string())?;
    let compact_rows = persist_players(tx, snapshot_id, &[player])?;
    compact::persist_rows(tx, snapshot_id, &compact_rows)?;
    compact::assert_snapshot_complete(tx, snapshot_id)?;
    assert_current_snapshot_complete(tx, snapshot_id)
}

/// Clears compact rows and potential-derived state for one snapshot.
pub(crate) fn clear_snapshot(tx: &Transaction<'_>, snapshot_id: i64) -> Result<(), String> {
    compact::clear_snapshot(tx, snapshot_id)?;
    tx.execute(
        "UPDATE players
         SET potential_attributes_json = NULL, potential_projection_model_version = NULL
         WHERE snapshot_id = ?1",
        [snapshot_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Clears compact rows and potential-derived state from every
/// non-current snapshot of one save, so only the effective current snapshot
/// keeps derived player state.
pub(crate) fn clear_non_current_snapshots(
    tx: &Transaction<'_>,
    save_id: i64,
) -> Result<(), String> {
    compact::clear_non_current_snapshots(tx, save_id)?;
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

/// Clears every non-current snapshot and materializes a newly selected current snapshot.
pub(crate) fn reconcile_current_selection(
    tx: &Transaction<'_>,
    save_id: i64,
    previous_snapshot_id: Option<i64>,
    current_snapshot_id: Option<i64>,
) -> Result<(), String> {
    clear_non_current_snapshots(tx, save_id)?;
    if current_snapshot_id != previous_snapshot_id {
        if let Some(snapshot_id) = current_snapshot_id {
            rebuild_snapshot(tx, snapshot_id)?;
        }
    }
    Ok(())
}

/// Verifies that an already-resolved current snapshot has complete persisted potential state.
pub(crate) fn assert_current_snapshot_complete(
    conn: &Connection,
    snapshot_id: i64,
) -> Result<(), String> {
    let attribute_placeholders = (1..=DUMP_ATTRIBUTE_KEYS.len())
        .map(|index| format!("(?{index})"))
        .collect::<Vec<_>>()
        .join(", ");
    let snapshot_parameter = DUMP_ATTRIBUTE_KEYS.len() + 1;
    let version_parameter = snapshot_parameter + 1;
    let sql = format!(
        "WITH expected_attributes(attribute_key) AS (VALUES {attribute_placeholders})
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
                   )
             )",
    );
    let mut values = DUMP_ATTRIBUTE_KEYS
        .iter()
        .map(|key| Value::Text((*key).to_string()))
        .collect::<Vec<_>>();
    values.push(Value::Integer(snapshot_id));
    values.push(Value::Integer(PROJECTION_MODEL_VERSION));
    let incomplete: bool = conn
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
) -> Result<Vec<compact::CompactPlayerRow>, String> {
    let mut attributes_statement = tx
        .prepare(
            "UPDATE players
             SET potential_attributes_json = ?3, potential_projection_model_version = ?4
             WHERE snapshot_id = ?1 AND uid = ?2",
        )
        .map_err(|error| error.to_string())?;
    let mut compact_rows = Vec::with_capacity(players.len());

    for player in players {
        let source_attributes = serde_json::from_str::<HashMap<String, Option<u8>>>(
            &player.attributes_json,
        )
        .map_err(|error| format!("invalid player {} attributes JSON: {error}", player.uid))?;
        for key in DUMP_ATTRIBUTE_KEYS {
            if let Some(value) = source_attributes.get(*key).copied().flatten() {
                if !(1..=20).contains(&value) {
                    return Err(format!(
                        "player {} attribute `{key}` must be between 1 and 20",
                        player.uid
                    ));
                }
            }
        }
        let attributes = DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| {
                (
                    (*key).to_string(),
                    source_attributes.get(*key).copied().flatten(),
                )
            })
            .collect::<HashMap<_, _>>();
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
        let mut current_scores = Vec::with_capacity(all_roles().len());
        let mut potential_scores = Vec::with_capacity(all_roles().len());
        for role in all_roles() {
            potential_scores.push(score_role(&projected, role).map(i64::from));
            current_scores.push(score_role(&attributes, role).map(i64::from));
        }
        compact_rows.push(compact::CompactPlayerRow {
            uid: player.uid,
            current_scores,
            potential_scores,
        });
    }
    Ok(compact_rows)
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

    fn compact_row(
        conn: &Connection,
        snapshot_id: i64,
        uid: i64,
    ) -> Option<crate::features::player_metrics::compact::test_support::CompactRowShape> {
        crate::features::player_metrics::compact::test_support::read_row(conn, snapshot_id, uid)
    }

    fn compact_row_count(conn: &Connection, snapshot_id: i64) -> i64 {
        crate::features::player_metrics::compact::test_support::count_rows(conn, snapshot_id)
    }

    type DerivedState = (
        Option<String>,
        Option<i64>,
        Option<crate::features::player_metrics::compact::test_support::CompactRowShape>,
    );

    fn derived_state(conn: &Connection, snapshot_id: i64) -> DerivedState {
        let fields = conn
            .query_row(
                "SELECT potential_attributes_json, potential_projection_model_version
                 FROM players WHERE snapshot_id = ?1 AND uid = 42",
                [snapshot_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("read projected fields");
        let compact =
            crate::features::player_metrics::compact::test_support::read_row(conn, snapshot_id, 42);
        (fields.0, fields.1, compact)
    }

    fn deny_derived_writes(conn: &Connection) {
        conn.execute_batch(
            "CREATE TRIGGER deny_projected_player_updates
             BEFORE UPDATE OF potential_attributes_json, potential_projection_model_version ON players
             BEGIN SELECT RAISE(ABORT, 'derived player writes are forbidden'); END;
             CREATE TRIGGER deny_compact_inserts
             BEFORE INSERT ON player_role_metrics
             BEGIN SELECT RAISE(ABORT, 'derived score writes are forbidden'); END;
             CREATE TRIGGER deny_compact_updates
             BEFORE UPDATE ON player_role_metrics
             BEGIN SELECT RAISE(ABORT, 'derived score writes are forbidden'); END;
             CREATE TRIGGER deny_compact_deletes
             BEFORE DELETE ON player_role_metrics
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
    fn rebuild_writes_one_compact_row_per_player_with_exact_versions_and_values() {
        let (conn, snapshot_id) = snapshot_with_persisted_scores();

        assert_eq!(compact_row_count(&conn, snapshot_id), 1);
        let state = derived_state(&conn, snapshot_id);
        let projected = serde_json::from_str::<HashMap<String, Option<u8>>>(
            state.0.as_deref().expect("persisted projected attributes"),
        )
        .expect("parse persisted projected attributes");
        let raw_attributes = DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), Some(10_u8)))
            .collect::<HashMap<_, _>>();

        let (score_version, projection_version, current, potential) =
            compact_row(&conn, snapshot_id, 42).expect("compact row for the materialized player");
        assert_eq!(
            score_version,
            crate::features::player_metrics::compact::SCORE_MODEL_VERSION
        );
        assert_eq!(projection_version, PROJECTION_MODEL_VERSION);
        for (index, role) in all_roles().iter().enumerate() {
            assert_eq!(
                current[index],
                score_role(&raw_attributes, role).map(i64::from),
                "current compact score for {}",
                role.role_id
            );
            assert_eq!(
                potential[index],
                score_role(&projected, role).map(i64::from),
                "potential compact score for {}",
                role.role_id
            );
        }
    }

    #[test]
    fn rebuild_persists_nullable_compact_scores_as_sql_null() {
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
        rebuild_snapshot(&tx, snapshot_id).expect("persist potential and compact scores");
        tx.commit().expect("commit potential scores");

        let (score_version, projection_version, current, potential) =
            compact_row(&conn, snapshot_id, 42).expect("compact row");
        assert_eq!(
            (score_version, projection_version),
            (
                crate::features::player_metrics::compact::SCORE_MODEL_VERSION,
                PROJECTION_MODEL_VERSION
            )
        );
        let affected_role = all_roles()
            .iter()
            .position(|role| role.role_id == "ball_playing_goalkeeper_ip")
            .expect("role requiring Composure");
        assert_eq!(potential[affected_role], None);
        assert_eq!(current[affected_role], None);
        let goalkeeper_role = all_roles()
            .iter()
            .position(|role| role.role_id == "goalkeeper_ip")
            .expect("fully satisfied role");
        assert_eq!(current[goalkeeper_role], Some(50));
    }

    #[test]
    fn replace_player_replaces_only_that_players_compact_row() {
        let (conn, snapshot_id) = snapshot_with_players(&[42, 43]);
        let tx = conn
            .unchecked_transaction()
            .expect("start writer transaction");
        rebuild_snapshot(&tx, snapshot_id).expect("persist compact rows");
        tx.commit().expect("commit initial materialization");

        let other_before = compact_row(&conn, snapshot_id, 43).expect("row for unchanged player");
        let upgraded_attributes = DUMP_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), Some(20_u8)))
            .collect::<HashMap<_, _>>();
        conn.execute(
            "UPDATE players SET attributes_json = ?3 WHERE snapshot_id = ?1 AND uid = ?2",
            params![
                snapshot_id,
                42,
                serde_json::to_string(&upgraded_attributes).expect("serialize upgraded attributes")
            ],
        )
        .expect("store upgraded source attributes");

        let tx = conn
            .unchecked_transaction()
            .expect("start replacement transaction");
        replace_player(&tx, snapshot_id, 42).expect("replace one player compact row");
        tx.commit().expect("commit player replacement");

        assert_eq!(compact_row_count(&conn, snapshot_id), 2);
        let (score_version, projection_version, current, _potential) =
            compact_row(&conn, snapshot_id, 42).expect("replaced compact row");
        assert_eq!(
            (score_version, projection_version),
            (
                crate::features::player_metrics::compact::SCORE_MODEL_VERSION,
                PROJECTION_MODEL_VERSION
            )
        );
        assert!(
            current.iter().all(|score| *score == Some(100)),
            "upgraded player scores every role"
        );
        assert_eq!(
            compact_row(&conn, snapshot_id, 43).as_ref(),
            Some(&other_before)
        );
    }

    #[test]
    fn rebuild_rejects_out_of_domain_catalog_attributes_before_projection() {
        let (conn, snapshot_id) = snapshot_with_players(&[42]);
        let initial_tx = conn
            .unchecked_transaction()
            .expect("start initial writer transaction");
        rebuild_snapshot(&initial_tx, snapshot_id).expect("persist initial potential scores");
        initial_tx
            .commit()
            .expect("commit initial potential scores");
        let before = derived_state(&conn, snapshot_id);
        conn.execute(
            "UPDATE players SET attributes_json = '{\"Acceleration\":0}'
             WHERE snapshot_id = ?1 AND uid = 42",
            [snapshot_id],
        )
        .expect("store invalid source attribute");

        reset_project_attributes_calls();
        let tx = conn
            .unchecked_transaction()
            .expect("start rejecting writer transaction");
        let error = rebuild_snapshot(&tx, snapshot_id)
            .expect_err("reject zero-valued catalog source attribute before projection");
        assert!(error.contains("player 42 attribute `Acceleration` must be between 1 and 20"));
        assert_eq!(project_attributes_call_count(), 0);
        drop(tx);

        assert_eq!(derived_state(&conn, snapshot_id), before);
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

    fn assert_compact_rejected_without_writes(conn: &Connection, snapshot_id: i64) {
        let before = derived_state(conn, snapshot_id);
        deny_derived_writes(conn);
        let tx = conn
            .unchecked_transaction()
            .expect("start assertion transaction");
        assert_eq!(
            crate::features::player_metrics::compact::assert_snapshot_complete(&tx, snapshot_id),
            Err("Current compact player snapshot is incomplete".to_string())
        );
        tx.commit().expect("commit read-only assertion");
        assert_eq!(derived_state(conn, snapshot_id), before);
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
        let compact = state.2.as_ref().expect("compact row");
        assert_eq!(compact.2.len(), all_roles().len());
        assert_eq!(compact.3.len(), all_roles().len());
        assert_eq!(
            compact.0,
            crate::features::player_metrics::compact::SCORE_MODEL_VERSION
        );
        assert_eq!(compact.1, PROJECTION_MODEL_VERSION);
        for (index, role) in all_roles().iter().enumerate() {
            assert_eq!(
                compact.3[index],
                score_role(&projected, role).map(i64::from),
                "potential compact score for {}",
                role.role_id
            );
        }

        let tx = conn
            .unchecked_transaction()
            .expect("start replacement transaction");
        replace_player(&tx, snapshot_id, 42).expect("replace one player");
        tx.commit().expect("commit player replacement");
        let after = derived_state(&conn, snapshot_id)
            .2
            .expect("compact after replace");
        assert_eq!(after.3.len(), all_roles().len());
    }

    #[test]
    fn rebuild_normalizes_omitted_attributes_to_null() {
        let (conn, snapshot_id) = snapshot_with_players(&[42]);
        conn.execute(
            "UPDATE players SET attributes_json = '{\"Acceleration\":10,\"Unknown\":20}'
             WHERE snapshot_id = ?1 AND uid = 42",
            [snapshot_id],
        )
        .expect("store sparse source attributes");

        let tx = conn
            .unchecked_transaction()
            .expect("start writer transaction");
        rebuild_snapshot(&tx, snapshot_id).expect("persist potential scores");
        tx.commit().expect("commit potential scores");

        let projected = serde_json::from_str::<HashMap<String, Option<u8>>>(
            derived_state(&conn, snapshot_id)
                .0
                .as_deref()
                .expect("persisted projected attributes"),
        )
        .expect("parse persisted projected attributes");
        assert_eq!(projected.len(), DUMP_ATTRIBUTE_KEYS.len());
        assert_eq!(projected.get("Acceleration"), Some(&Some(11)));
        assert_eq!(projected.get("Pace"), Some(&None));
        assert!(!projected.contains_key("Unknown"));
        let compact = derived_state(&conn, snapshot_id).2.expect("compact row");
        assert_eq!(compact.2.len(), all_roles().len());
        assert_eq!(compact.3.len(), all_roles().len());
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
        let compact = state.2.expect("compact row");
        assert_eq!(compact.3.len(), all_roles().len());
        for (index, role) in all_roles().iter().enumerate() {
            assert_eq!(
                compact.3[index],
                score_role(&projected, role).map(i64::from),
                "potential score for {}",
                role.role_id
            );
        }

        let affected_role = all_roles()
            .iter()
            .find(|role| role.role_id == "ball_playing_goalkeeper_ip")
            .expect("role requiring Composure");
        assert!(affected_role.secondary.contains(&"Composure"));
        let col = crate::features::player_metrics::compact::player_potential_column(
            affected_role.role_id,
        )
        .expect("potential column");
        let sql =
            format!("SELECT {col} FROM player_role_metrics WHERE snapshot_id = ?1 AND uid = ?2");
        let affected_score: Option<i64> = conn
            .query_row(&sql, params![snapshot_id, 42], |row| row.get(0))
            .expect("read nullable compact score");
        assert_eq!(affected_score, None);
    }

    #[test]
    fn assertion_rejects_missing_or_wrong_version_compact_rows_without_writes() {
        let (conn, snapshot_id) = snapshot_with_persisted_scores();
        conn.execute(
            "DELETE FROM player_role_metrics WHERE snapshot_id = ?1 AND uid = 42",
            [snapshot_id],
        )
        .expect("delete compact row");
        assert_compact_rejected_without_writes(&conn, snapshot_id);

        let (conn, snapshot_id) = snapshot_with_persisted_scores();
        conn.execute(
            "UPDATE player_role_metrics SET projection_model_version = ?2 WHERE snapshot_id = ?1 AND uid = 42",
            params![snapshot_id, PROJECTION_MODEL_VERSION - 1],
        )
        .expect("make compact row stale");
        assert_compact_rejected_without_writes(&conn, snapshot_id);

        let (conn, snapshot_id) = snapshot_with_persisted_scores();
        conn.execute(
            "UPDATE player_role_metrics SET score_model_version = ?2 WHERE snapshot_id = ?1 AND uid = 42",
            params![
                snapshot_id,
                crate::features::player_metrics::compact::SCORE_MODEL_VERSION + 99
            ],
        )
        .expect("make compact score version stale");
        assert_compact_rejected_without_writes(&conn, snapshot_id);
    }
}
