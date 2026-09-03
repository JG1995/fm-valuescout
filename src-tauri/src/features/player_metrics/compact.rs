//! Trusted mapping from the closed player-role catalog to the checked-in
//! compact `player_role_metrics` columns and model versions, plus the
//! one-row-per-player persistence for the current-snapshot lifecycle.
//!
//! Migration 38 checks in one immutable inventory of 68 current and 68
//! potential named columns, extended by migration 40 with 11 generic OOP
//! roles to 79 current and 79 potential columns: the current column name is the verified role id,
//! and the matching potential column prefixes it with `potential_`. Writers
//! and readers derive SQL identifiers only through this module — closed-catalog
//! lookup followed by safe snake_case validation. WebView input never becomes
//! an SQL identifier.

use std::collections::HashMap;

use rusqlite::{params, params_from_iter, types::Value, Connection, Transaction};

use super::potential_scores;
use crate::features::scoring::catalog::{all_roles, DUMP_ATTRIBUTE_KEYS};
use crate::features::scoring::projection::project_attributes;
use crate::features::scoring::score::score_role;

/// Model version of the checked-in projection formula. Alias of
/// `potential_scores::PROJECTION_MODEL_VERSION` so the compact contract keeps
/// one version owner.
pub const PROJECTION_MODEL_VERSION: i64 = potential_scores::PROJECTION_MODEL_VERSION;

/// Model version of the checked-in current per-role score formula (`score_role`).
/// Bumped to 2 when migration 40 added the 11 generic OOP compact columns.
pub const SCORE_MODEL_VERSION: i64 = 2;

/// Returns the compact current column for a closed-catalog player role.
pub fn player_current_column(role_id: &str) -> Result<&'static str, String> {
    validated_player_role(role_id)
}

/// Returns the compact potential column for a closed-catalog player role.
pub fn player_potential_column(role_id: &str) -> Result<String, String> {
    Ok(format!("potential_{}", validated_player_role(role_id)?))
}

/// Fixed SQL alias of the one compact player metric row joined per current
/// player when a query reads role metrics.
pub const PLAYER_METRICS_ALIAS: &str = "player_metrics";

/// Builds the one-to-one compact player metrics join for the role kinds a
/// read consumes. The score model gates every current or potential role read,
/// because both select score columns from the same compact row; potential
/// reads additionally require the projection model. Model-version predicates
/// make only rows with the exact checked-in versions readable, so a
/// wrong-version row never contributes a value; a missing row stays NULL
/// through the LEFT JOIN.
pub fn player_metrics_join(
    player_alias: &str,
    require_score_model: bool,
    require_projection_model: bool,
) -> String {
    let mut predicates = vec![
        format!("{PLAYER_METRICS_ALIAS}.snapshot_id = {player_alias}.snapshot_id"),
        format!("{PLAYER_METRICS_ALIAS}.uid = {player_alias}.uid"),
    ];
    if require_score_model || require_projection_model {
        predicates.push(format!(
            "{PLAYER_METRICS_ALIAS}.score_model_version = {SCORE_MODEL_VERSION}"
        ));
    }
    if require_projection_model {
        predicates.push(format!(
            "{PLAYER_METRICS_ALIAS}.projection_model_version = {PROJECTION_MODEL_VERSION}"
        ));
    }
    format!(
        " LEFT JOIN player_role_metrics {PLAYER_METRICS_ALIAS} ON {}",
        predicates.join(" AND ")
    )
}

/// Scoped read validation: every current player must have one compact row
/// carrying each model version a read consumes — the checked-in score model
/// for every current or potential role request (both select score columns
/// from the same compact row) and the checked-in projection model for
/// potential role requests. Missing or wrong-version state fails before
/// values are read; a read never writes or repairs.
pub(crate) fn assert_read_models_complete(
    conn: &Connection,
    snapshot_id: i64,
    require_score_model: bool,
    require_projection_model: bool,
) -> Result<(), String> {
    if !require_score_model && !require_projection_model {
        return Ok(());
    }
    let mut model_predicates = Vec::new();
    if require_score_model || require_projection_model {
        model_predicates.push(format!(
            "{PLAYER_METRICS_ALIAS}.score_model_version = {SCORE_MODEL_VERSION}"
        ));
    }
    if require_projection_model {
        model_predicates.push(format!(
            "{PLAYER_METRICS_ALIAS}.projection_model_version = {PROJECTION_MODEL_VERSION}"
        ));
    }
    let sql = format!(
        "SELECT EXISTS(
             SELECT 1
             FROM players p
             LEFT JOIN player_role_metrics {PLAYER_METRICS_ALIAS}
               ON {PLAYER_METRICS_ALIAS}.snapshot_id = p.snapshot_id
              AND {PLAYER_METRICS_ALIAS}.uid = p.uid
              AND {}
             WHERE p.snapshot_id = ?1 AND {PLAYER_METRICS_ALIAS}.snapshot_id IS NULL
         )",
        model_predicates.join(" AND ")
    );
    let incomplete: bool = conn
        .query_row(&sql, [snapshot_id], |row| row.get(0))
        .map_err(|error| error.to_string())?;
    if incomplete {
        Err(if require_projection_model {
            "Current potential snapshot is incomplete".to_string()
        } else {
            "Current compact player snapshot is incomplete".to_string()
        })
    } else {
        Ok(())
    }
}

/// One catalog-ordered compact row to persist for a player: the 79 current
/// scores in `all_roles()` order followed by the 79 potential scores in the
/// same order. Values are SQL null when a required source attribute is missing.
pub(crate) struct CompactPlayerRow {
    pub(crate) uid: i64,
    pub(crate) current_scores: Vec<Option<i64>>,
    pub(crate) potential_scores: Vec<Option<i64>>,
}

/// Pure preparation of player projection and compact scores without any
/// database read/write or `rusqlite` ownership. Callers build the returned
/// values outside the `Db(Mutex<Connection>)` lock; `persist_rows` writes
/// them in the final transaction. Memory is proportional to one player's
/// 79+79 scores.
#[allow(clippy::type_complexity)]
pub(crate) fn prepare_player_derived(
    uid: i64,
    attributes_json: &str,
    positions_json: &str,
    ca: i64,
    pa: i64,
    age: Option<i64>,
) -> Result<(String, Vec<Option<i64>>, Vec<Option<i64>>), String> {
    let source_attributes = serde_json::from_str::<HashMap<String, Option<u8>>>(attributes_json)
        .map_err(|error| format!("invalid player {uid} attributes JSON: {error}"))?;
    for key in DUMP_ATTRIBUTE_KEYS {
        if let Some(value) = source_attributes.get(*key).copied().flatten() {
            if !(1..=20).contains(&value) {
                return Err(format!(
                    "player {uid} attribute `{key}` must be between 1 and 20"
                ));
            }
        }
    }
    let attributes_for_scoring: HashMap<String, Option<u8>> = DUMP_ATTRIBUTE_KEYS
        .iter()
        .map(|key| {
            (
                (*key).to_string(),
                source_attributes.get(*key).copied().flatten(),
            )
        })
        .collect();
    let positions_map = serde_json::from_str::<HashMap<String, Option<i64>>>(positions_json)
        .map_err(|error| format!("invalid player {uid} positions JSON: {error}"))?;
    let projected = project_attributes(
        &attributes_for_scoring,
        ca,
        pa,
        age,
        positions_map.iter().map(|(k, v)| (k.as_str(), *v)),
    );
    let projected_json = serde_json::to_string(&projected).map_err(|e| e.to_string())?;
    let mut current_scores = Vec::with_capacity(all_roles().len());
    let mut potential_scores = Vec::with_capacity(all_roles().len());
    for role in all_roles() {
        current_scores.push(score_role(&attributes_for_scoring, role).map(i64::from));
        potential_scores.push(score_role(&projected, role).map(i64::from));
    }
    Ok((projected_json, current_scores, potential_scores))
}

/// Borrowed persistence implementation: inserts or replaces one compact row per player
/// without cloning the 158-value score vectors. Accepts any iterator over borrowed
/// slices directly from `PreparedPlayer`.
pub(crate) fn persist_rows_borrowed<'a, I>(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    rows: I,
) -> Result<(), String>
where
    I: IntoIterator<Item = (i64, &'a [Option<i64>], &'a [Option<i64>])>,
{
    let roles = all_roles();
    let mut columns = roles
        .iter()
        .map(|role| player_current_column(role.role_id).map(str::to_string))
        .collect::<Result<Vec<_>, _>>()?;
    columns.extend(
        roles
            .iter()
            .map(|role| player_potential_column(role.role_id))
            .collect::<Result<Vec<_>, _>>()?,
    );
    let placeholders = (0..columns.len() + 4)
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT OR REPLACE INTO player_role_metrics (
            snapshot_id, uid, score_model_version, projection_model_version, {}
         ) VALUES ({placeholders})",
        columns.join(", ")
    );
    let mut statement = tx.prepare(&sql).map_err(|error| error.to_string())?;

    for (uid, current_scores, potential_scores) in rows {
        if current_scores.len() != roles.len() || potential_scores.len() != roles.len() {
            return Err("Compact player row has the wrong role count".to_string());
        }
        let mut values = Vec::with_capacity(columns.len() + 4);
        values.push(Value::Integer(snapshot_id));
        values.push(Value::Integer(uid));
        values.push(Value::Integer(SCORE_MODEL_VERSION));
        values.push(Value::Integer(PROJECTION_MODEL_VERSION));
        values.extend(
            current_scores
                .iter()
                .map(|score| score.map_or(Value::Null, Value::Integer)),
        );
        values.extend(
            potential_scores
                .iter()
                .map(|score| score.map_or(Value::Null, Value::Integer)),
        );
        statement
            .execute(params_from_iter(values.iter()))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// Inserts or replaces one compact row per prepared player for one snapshot,
/// with the exact checked-in score and projection model versions.
pub(crate) fn persist_rows(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    rows: &[CompactPlayerRow],
) -> Result<(), String> {
    persist_rows_borrowed(
        tx,
        snapshot_id,
        rows.iter().map(|row| {
            (
                row.uid,
                row.current_scores.as_slice(),
                row.potential_scores.as_slice(),
            )
        }),
    )
}

/// Deletes every compact row for one snapshot.
pub(crate) fn clear_snapshot(tx: &Transaction<'_>, snapshot_id: i64) -> Result<(), String> {
    tx.execute(
        "DELETE FROM player_role_metrics WHERE snapshot_id = ?1",
        [snapshot_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Deletes compact rows from every non-current snapshot of one save.
pub(crate) fn clear_non_current_snapshots(
    tx: &Transaction<'_>,
    save_id: i64,
) -> Result<(), String> {
    tx.execute(
        "DELETE FROM player_role_metrics
         WHERE snapshot_id IN (
             SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 0
         )",
        [save_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Verifies that every player of an effective current snapshot has exactly one
/// compact row with the checked-in score and projection model versions. The
/// `(snapshot_id, uid)` primary key makes a second row per player impossible.
pub(crate) fn assert_snapshot_complete(conn: &Connection, snapshot_id: i64) -> Result<(), String> {
    let incomplete: bool = conn
        .query_row(
            "SELECT
                 NOT EXISTS(SELECT 1 FROM snapshots WHERE id = ?1 AND is_current = 1)
                 OR EXISTS(
                     SELECT 1
                     FROM players p
                     LEFT JOIN player_role_metrics m
                       ON m.snapshot_id = p.snapshot_id AND m.uid = p.uid
                     WHERE p.snapshot_id = ?1
                       AND (
                           m.snapshot_id IS NULL
                           OR m.score_model_version <> ?2
                           OR m.projection_model_version <> ?3
                       )
                 )",
            params![snapshot_id, SCORE_MODEL_VERSION, PROJECTION_MODEL_VERSION],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if incomplete {
        Err("Current compact player snapshot is incomplete".to_string())
    } else {
        Ok(())
    }
}

fn validated_player_role(role_id: &str) -> Result<&'static str, String> {
    let role = all_roles()
        .iter()
        .find(|role| role.role_id == role_id)
        .ok_or_else(|| format!("unknown player role: {role_id}"))?;
    require_safe_snake_case(role.role_id)
}

fn require_safe_snake_case(identifier: &'static str) -> Result<&'static str, String> {
    let safe = !identifier.is_empty()
        && identifier.as_bytes()[0].is_ascii_lowercase()
        && identifier
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    if safe {
        Ok(identifier)
    } else {
        Err(format!("unsafe role identifier: {identifier}"))
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use rusqlite::OptionalExtension;

    pub(crate) type CompactRowShape = (i64, i64, Vec<Option<i64>>, Vec<Option<i64>>);

    pub(crate) fn count_rows(conn: &Connection, snapshot_id: i64) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM player_role_metrics WHERE snapshot_id = ?1",
            [snapshot_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("count compact rows")
    }

    /// Reads one player's compact row: `(score_model_version,
    /// projection_model_version, current scores, potential scores)` in catalog
    /// order, or `None` when no compact row exists.
    pub(crate) fn read_row(
        conn: &Connection,
        snapshot_id: i64,
        uid: i64,
    ) -> Option<CompactRowShape> {
        let roles = all_roles();
        let columns = roles
            .iter()
            .map(|role| {
                player_current_column(role.role_id)
                    .expect("current column")
                    .to_string()
            })
            .chain(
                roles
                    .iter()
                    .map(|role| player_potential_column(role.role_id).expect("potential column")),
            )
            .collect::<Vec<_>>();
        let sql = format!(
            "SELECT score_model_version, projection_model_version, {}
             FROM player_role_metrics WHERE snapshot_id = ?1 AND uid = ?2",
            columns.join(", ")
        );
        conn.query_row(&sql, params![snapshot_id, uid], |row| {
            let current = (0..roles.len())
                .map(|index| row.get::<_, Option<i64>>(index + 2))
                .collect::<Result<Vec<_>, _>>()?;
            let potential = (0..roles.len())
                .map(|index| row.get::<_, Option<i64>>(index + 2 + roles.len()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((row.get(0)?, row.get(1)?, current, potential))
        })
        .optional()
        .expect("read compact row")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    use rusqlite::Connection;

    fn migrated_table_columns(table: &str) -> Vec<String> {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("compact-contract.db"))
            .expect("open contract test db");
        crate::db::migrations::apply(&conn).expect("apply migrations");
        let mut statement = conn
            .prepare("SELECT name FROM pragma_table_info(?1) ORDER BY cid")
            .expect("prepare table info query");
        statement
            .query_map([table], |row| row.get(0))
            .expect("query table info")
            .collect::<Result<Vec<_>, _>>()
            .expect("read table columns")
    }

    #[test]
    fn runtime_player_catalog_maps_once_to_the_checked_in_compact_schema() {
        let roles = all_roles();
        assert_eq!(roles.len(), 79);
        let current = roles
            .iter()
            .map(|role| player_current_column(role.role_id))
            .collect::<Result<Vec<_>, _>>()
            .expect("map current columns");
        let potential = roles
            .iter()
            .map(|role| player_potential_column(role.role_id))
            .collect::<Result<Vec<_>, _>>()
            .expect("map potential columns");
        assert_eq!(
            current.iter().collect::<HashSet<_>>().len(),
            79,
            "current columns must be unique per role"
        );
        assert_eq!(
            potential.iter().collect::<HashSet<_>>().len(),
            79,
            "potential columns must be unique per role"
        );

        let schema = migrated_table_columns("player_role_metrics");
        let expected = [
            "snapshot_id",
            "uid",
            "score_model_version",
            "projection_model_version",
        ]
        .into_iter()
        .map(str::to_string)
        .chain(current.into_iter().map(str::to_string))
        .chain(potential)
        .collect::<HashSet<_>>();
        assert_eq!(schema.len(), 162);
        // Additive migrations append columns, so physical order follows
        // migration history; the contract pins the exact column set.
        assert_eq!(schema.into_iter().collect::<HashSet<_>>(), expected);

        // The version columns reject zero; writers persist exactly these
        // checked-in model versions into them.
        let model_versions = [SCORE_MODEL_VERSION, PROJECTION_MODEL_VERSION];
        assert!(model_versions.iter().all(|version| *version > 0));
    }

    #[test]
    fn player_metrics_join_uses_a_missing_preserving_one_to_one_relation_with_kind_versions() {
        let current_only = player_metrics_join("players", true, false);
        assert!(current_only.contains(" LEFT JOIN player_role_metrics player_metrics ON "));
        assert!(current_only.contains("player_metrics.snapshot_id = players.snapshot_id"));
        assert!(current_only.contains("player_metrics.uid = players.uid"));
        assert!(current_only.contains("player_metrics.score_model_version = 2"));
        assert!(!current_only.contains("projection_model_version"));

        let potential_only = player_metrics_join("players", false, true);
        assert!(potential_only.contains("player_metrics.snapshot_id = players.snapshot_id"));
        assert!(potential_only.contains("player_metrics.score_model_version = 2"));
        assert!(potential_only.contains("player_metrics.projection_model_version = 2"));

        let both = player_metrics_join("players", true, true);
        assert!(both.contains("player_metrics.score_model_version = 2"));
        assert!(both.contains("player_metrics.projection_model_version = 2"));
        assert!(both.contains(" LEFT JOIN player_role_metrics "));
    }

    #[test]
    fn rejects_unknown_or_unsafe_player_role_identifiers() {
        for id in [
            "unknown_role",
            "Physio",
            "Goalkeeper",
            "centre-back-ip",
            "1st_role",
            "",
            "with space",
            "role!",
            "goalkeeper-ip",
            "camelCase",
            "_leading_underscore",
        ] {
            assert!(player_current_column(id).is_err(), "current {id}");
            assert!(player_potential_column(id).is_err(), "potential {id}");
        }
    }

    #[test]
    fn safe_snake_case_validation_accepts_only_catalog_shaped_identifiers() {
        for id in ["goalkeeper_ip", "role_2", "a", "x_y_z_9"] {
            assert_eq!(
                require_safe_snake_case(id).map(str::to_string),
                Ok(id.to_string())
            );
        }
        for id in [
            "",
            "1st",
            "A",
            "a-b",
            "a b",
            "camelCase",
            "_leading",
            "dot.id",
            "a!b",
        ] {
            assert!(require_safe_snake_case(id).is_err(), "{id}");
        }
    }
}
