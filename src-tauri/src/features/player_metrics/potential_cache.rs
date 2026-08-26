use std::collections::{HashMap, HashSet};

use rusqlite::{params, params_from_iter, types::Value, Connection, Transaction};

use crate::features::scoring::{
    catalog::{all_roles, RoleDefinition},
    projection::project_attributes,
    score::score_role,
};

pub use super::potential_scores::PROJECTION_MODEL_VERSION;

const MATERIALIZATION_BATCH_SIZE: usize = 250;

struct PlayerForProjection {
    uid: i64,
    ca: i64,
    pa: i64,
    age: Option<i64>,
    positions_json: String,
    attributes_json: String,
}

/// Materializes missing or stale potential role scores for a bounded requested cohort.
///
/// Display-only table fields use this after their page UIDs are known. The cache retains
/// the same version and nullable-result semantics as full-cohort materialization.
pub fn materialize_player_roles(
    conn: &Connection,
    snapshot_id: i64,
    player_uids: &[i64],
    role_ids: &[String],
) -> Result<(), String> {
    let roles = requested_roles(role_ids)?;
    if roles.is_empty() || player_uids.is_empty() {
        return Ok(());
    }

    let mut seen_uids = HashSet::new();
    let unique_uids = player_uids
        .iter()
        .copied()
        .filter(|uid| seen_uids.insert(*uid))
        .collect::<Vec<_>>();

    for uid_batch in unique_uids.chunks(MATERIALIZATION_BATCH_SIZE) {
        let players = load_players_by_uids(conn, snapshot_id, uid_batch)?;
        if players.is_empty() {
            continue;
        }
        let cached_role_ids = load_cached_role_ids(conn, snapshot_id, &players, &roles)?;
        let scores = score_missing_roles(players, &roles, &cached_role_ids)?;
        if scores.is_empty() {
            continue;
        }
        let tx = conn
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;
        persist_scores(&tx, snapshot_id, &scores)?;
        tx.commit().map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn requested_roles(role_ids: &[String]) -> Result<Vec<&'static RoleDefinition>, String> {
    let mut roles = Vec::new();
    for role_id in role_ids {
        if roles
            .iter()
            .any(|role: &&RoleDefinition| role.role_id == role_id)
        {
            continue;
        }
        let role = all_roles()
            .iter()
            .find(|role| role.role_id == role_id)
            .ok_or_else(|| format!("unknown role id: {role_id}"))?;
        roles.push(role);
    }
    Ok(roles)
}

fn load_players_by_uids(
    conn: &Connection,
    snapshot_id: i64,
    player_uids: &[i64],
) -> Result<Vec<PlayerForProjection>, String> {
    let placeholders = player_uids
        .iter()
        .enumerate()
        .map(|(index, _)| format!("?{}", index + 2))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT uid, ca, pa, age, positions_json, attributes_json
         FROM players
         WHERE snapshot_id = ?1 AND uid IN ({placeholders})
         ORDER BY uid"
    );
    let mut values = vec![Value::Integer(snapshot_id)];
    values.extend(player_uids.iter().copied().map(Value::Integer));
    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(values.iter()), |row| {
            Ok(PlayerForProjection {
                uid: row.get(0)?,
                ca: row.get(1)?,
                pa: row.get(2)?,
                age: row.get(3)?,
                positions_json: row.get(4)?,
                attributes_json: row.get(5)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

fn load_cached_role_ids(
    conn: &Connection,
    snapshot_id: i64,
    players: &[PlayerForProjection],
    roles: &[&RoleDefinition],
) -> Result<HashMap<i64, HashSet<String>>, String> {
    let role_placeholders = roles
        .iter()
        .enumerate()
        .map(|(index, _)| format!("?{}", index + 3))
        .collect::<Vec<_>>()
        .join(", ");
    let uid_start_index = roles.len() + 3;
    let uid_placeholders = players
        .iter()
        .enumerate()
        .map(|(index, _)| format!("?{}", uid_start_index + index))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT uid, role_id
         FROM player_potential_role_scores
         WHERE snapshot_id = ?1
           AND projection_model_version = ?2
           AND role_id IN ({role_placeholders})
           AND uid IN ({uid_placeholders})"
    );
    let mut values = vec![
        Value::Integer(snapshot_id),
        Value::Integer(PROJECTION_MODEL_VERSION),
    ];
    values.extend(
        roles
            .iter()
            .map(|role| Value::Text(role.role_id.to_string())),
    );
    values.extend(players.iter().map(|player| Value::Integer(player.uid)));

    let mut stmt = conn.prepare(&sql).map_err(|error| error.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(values.iter()), |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|error| error.to_string())?;
    let mut cached = HashMap::<i64, HashSet<String>>::new();
    for row in rows {
        let (uid, role_id) = row.map_err(|error| error.to_string())?;
        cached.entry(uid).or_default().insert(role_id);
    }
    Ok(cached)
}

fn score_missing_roles(
    players: Vec<PlayerForProjection>,
    roles: &[&RoleDefinition],
    cached_role_ids: &HashMap<i64, HashSet<String>>,
) -> Result<Vec<(i64, String, Option<i64>)>, String> {
    let mut scores = Vec::new();
    for player in players {
        let cached_for_player = cached_role_ids.get(&player.uid);
        let missing_roles = roles
            .iter()
            .filter(|role| !cached_for_player.is_some_and(|cached| cached.contains(role.role_id)))
            .copied()
            .collect::<Vec<_>>();
        if missing_roles.is_empty() {
            continue;
        }
        let attributes = serde_json::from_str(&player.attributes_json)
            .map_err(|error| format!("invalid player {} attributes JSON: {error}", player.uid))?;
        let positions = serde_json::from_str::<HashMap<String, Option<i64>>>(
            &player.positions_json,
        )
        .map_err(|error| format!("invalid player {} positions JSON: {error}", player.uid))?;
        let projected = project_attributes(
            &attributes,
            player.ca,
            player.pa,
            player.age,
            positions
                .iter()
                .map(|(position, familiarity)| (position.as_str(), *familiarity)),
        );
        for role in missing_roles {
            scores.push((
                player.uid,
                role.role_id.to_string(),
                score_role(&projected, role).map(i64::from),
            ));
        }
    }
    Ok(scores)
}

fn persist_scores(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    scores: &[(i64, String, Option<i64>)],
) -> Result<(), String> {
    let mut stmt = tx
        .prepare(
            "INSERT INTO player_potential_role_scores (
                snapshot_id, uid, role_id, score, projection_model_version
             ) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(snapshot_id, uid, role_id) DO UPDATE SET
                score = excluded.score,
                projection_model_version = excluded.projection_model_version",
        )
        .map_err(|error| error.to_string())?;
    for (uid, role_id, score) in scores {
        stmt.execute(params![
            snapshot_id,
            uid,
            role_id,
            score,
            PROJECTION_MODEL_VERSION,
        ])
        .map_err(|error| error.to_string())?;
    }
    Ok(())
}
