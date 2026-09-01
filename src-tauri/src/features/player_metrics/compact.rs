//! Trusted mapping from the closed player-role catalog to the checked-in
//! compact `player_role_metrics` columns and model versions.
//!
//! Migration 38 checks in one immutable inventory of 68 current and 68
//! potential named columns: the current column name is the verified role id,
//! and the matching potential column prefixes it with `potential_`. Writers
//! and readers derive SQL identifiers only through this module — closed-catalog
//! lookup followed by safe snake_case validation. WebView input never becomes
//! an SQL identifier.
#![cfg_attr(not(test), allow(dead_code))]

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
