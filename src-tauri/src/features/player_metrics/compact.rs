//! Trusted mapping from the closed player-role catalog to the checked-in
//! compact `player_role_metrics` columns and model versions, plus the
//! one-row-per-player persistence for the current-snapshot lifecycle.
//!
//! Migration 38 checks in one immutable inventory of 68 current and 68
//! potential named columns: the current column name is the verified role id,
//! and the matching potential column prefixes it with `potential_`. Writers
//! and readers derive SQL identifiers only through this module — closed-catalog
//! lookup followed by safe snake_case validation. WebView input never becomes
//! an SQL identifier.

use rusqlite::{params, params_from_iter, types::Value, Connection, Transaction};

use super::potential_scores;
use crate::features::scoring::catalog::all_roles;

/// Model version of the checked-in projection formula. Alias of
/// `potential_scores::PROJECTION_MODEL_VERSION` so the compact contract keeps
/// one version owner.
pub const PROJECTION_MODEL_VERSION: i64 = potential_scores::PROJECTION_MODEL_VERSION;

/// Model version of the checked-in current per-role score formula (`score_role`).
pub const SCORE_MODEL_VERSION: i64 = 1;

/// Returns the compact current column for a closed-catalog player role.
pub fn player_current_column(role_id: &str) -> Result<&'static str, String> {
    validated_player_role(role_id)
}

/// Returns the compact potential column for a closed-catalog player role.
pub fn player_potential_column(role_id: &str) -> Result<String, String> {
    Ok(format!("potential_{}", validated_player_role(role_id)?))
}

/// One catalog-ordered compact row to persist for a player: the 68 current
/// scores in `all_roles()` order followed by the 68 potential scores in the
/// same order. Values are SQL null when a required source attribute is missing.
pub(crate) struct CompactPlayerRow {
    pub(crate) uid: i64,
    pub(crate) current_scores: Vec<Option<i64>>,
    pub(crate) potential_scores: Vec<Option<i64>>,
}

/// Inserts or replaces one compact row per prepared player for one snapshot,
/// with the exact checked-in score and projection model versions.
pub(crate) fn persist_rows(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    rows: &[CompactPlayerRow],
) -> Result<(), String> {
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

    for row in rows {
        if row.current_scores.len() != roles.len() || row.potential_scores.len() != roles.len() {
            return Err("Compact player row has the wrong role count".to_string());
        }
        let mut values = Vec::with_capacity(columns.len() + 4);
        values.push(Value::Integer(snapshot_id));
        values.push(Value::Integer(row.uid));
        values.push(Value::Integer(SCORE_MODEL_VERSION));
        values.push(Value::Integer(PROJECTION_MODEL_VERSION));
        values.extend(
            row.current_scores
                .iter()
                .map(|score| score.map_or(Value::Null, Value::Integer)),
        );
        values.extend(
            row.potential_scores
                .iter()
                .map(|score| score.map_or(Value::Null, Value::Integer)),
        );
        statement
            .execute(params_from_iter(values.iter()))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
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
        assert_eq!(roles.len(), 68);
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
            68,
            "current columns must be unique per role"
        );
        assert_eq!(
            potential.iter().collect::<HashSet<_>>().len(),
            68,
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
        .collect::<Vec<_>>();
        assert_eq!(schema.len(), 140);
        assert_eq!(schema, expected);

        // The version columns reject zero; writers persist exactly these
        // checked-in model versions into them.
        let model_versions = [SCORE_MODEL_VERSION, PROJECTION_MODEL_VERSION];
        assert!(model_versions.iter().all(|version| *version > 0));
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
