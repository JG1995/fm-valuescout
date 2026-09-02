use std::collections::HashMap;

use rusqlite::{params, params_from_iter, types::Value, Connection, Transaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaffRoleDefinition {
    pub role_id: &'static str,
    pub display_name: &'static str,
    pub attributes: &'static [&'static str],
}

const STAFF_ROLES: &[StaffRoleDefinition] = &[
    StaffRoleDefinition {
        role_id: "assistant_manager",
        display_name: "Assistant Manager",
        attributes: &[
            "ManManagement",
            "JudgingPlayerPotential",
            "JudgingPlayerAbility",
        ],
    },
    StaffRoleDefinition {
        role_id: "manager",
        display_name: "Manager",
        attributes: &[
            "Motivating",
            "ManManagement",
            "JudgingPlayerAbility",
            "JudgingPlayerPotential",
            "TacticalKnowledge",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_attacking_technical",
        display_name: "Coach — Attacking Technical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Attacking",
            "Technical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_attacking_tactical",
        display_name: "Coach — Attacking Tactical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Attacking",
            "Tactical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_defending_technical",
        display_name: "Coach — Defending Technical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Defending",
            "Technical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_defending_tactical",
        display_name: "Coach — Defending Tactical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Defending",
            "Tactical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_possession_technical",
        display_name: "Coach — Possession Technical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Possession",
            "Technical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_possession_tactical",
        display_name: "Coach — Possession Tactical",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "Possession",
            "Tactical",
        ],
    },
    StaffRoleDefinition {
        role_id: "coach_fitness",
        display_name: "Coach — Fitness",
        attributes: &["Authority", "Determination", "Motivating", "Fitness"],
    },
    StaffRoleDefinition {
        role_id: "coach_goalkeeping",
        display_name: "Coach — Goalkeeping",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "GoalkeepingDistribution",
            "GoalkeepingHandling",
            "GoalkeepingReflexes",
        ],
    },
    StaffRoleDefinition {
        role_id: "set_piece_coach",
        display_name: "Set Piece Coach",
        attributes: &[
            "Authority",
            "Determination",
            "Motivating",
            "SetPieces",
            "TacticalKnowledge",
        ],
    },
    StaffRoleDefinition {
        role_id: "loan_manager",
        display_name: "Loan Manager",
        attributes: &[
            "ManManagement",
            "JudgingPlayerPotential",
            "JudgingPlayerAbility",
        ],
    },
    StaffRoleDefinition {
        role_id: "head_of_youth_development",
        display_name: "Head of Youth Development",
        attributes: &[
            "WorkingWithYoungsters",
            "JudgingPlayerPotential",
            "JudgingPlayerAbility",
        ],
    },
    StaffRoleDefinition {
        role_id: "scout",
        display_name: "Scout",
        attributes: &[
            "Adaptability",
            "JudgingPlayerPotential",
            "JudgingPlayerAbility",
        ],
    },
    StaffRoleDefinition {
        role_id: "director_of_football",
        display_name: "Director of Football",
        attributes: &[
            "JudgingPlayerPotential",
            "JudgingPlayerAbility",
            "Negotiating",
        ],
    },
    StaffRoleDefinition {
        role_id: "technical_director",
        display_name: "Technical Director",
        attributes: &["JudgingStaffAbility", "Negotiating"],
    },
    StaffRoleDefinition {
        role_id: "recruitment_analyst",
        display_name: "Recruitment Analyst",
        attributes: &["DataAnalysis", "JudgingPlayerAbility"],
    },
    StaffRoleDefinition {
        role_id: "head_performance_analyst",
        display_name: "Head Performance Analyst",
        attributes: &[
            "DataAnalysis",
            "Determination",
            "JudgingPlayerAbility",
            "TacticalKnowledge",
        ],
    },
    StaffRoleDefinition {
        role_id: "performance_analyst",
        display_name: "Performance Analyst",
        attributes: &["DataAnalysis", "TacticalKnowledge"],
    },
    StaffRoleDefinition {
        role_id: "physio",
        display_name: "Physio",
        attributes: &["Physiotherapy"],
    },
    StaffRoleDefinition {
        role_id: "sports_scientist",
        display_name: "Sports Scientist",
        attributes: &["SportsScience"],
    },
];

pub fn all_staff_roles() -> &'static [StaffRoleDefinition] {
    STAFF_ROLES
}

/// Model version of the checked-in staff per-role score formula
/// (`score_staff_role`).
pub const SCORE_MODEL_VERSION: i64 = 1;

/// Fixed SQL alias of the one compact staff metric row joined per current
/// staff member when a query reads role metrics.
pub const STAFF_METRICS_ALIAS: &str = "staff_metrics";

/// Builds the one-to-one compact staff metrics join for reads. The model
/// version predicate makes only rows with the exact checked-in version
/// readable; a missing row stays NULL through the LEFT JOIN.
pub fn staff_metrics_join(staff_alias: &str) -> String {
    format!(
        " LEFT JOIN staff_role_metrics {STAFF_METRICS_ALIAS} ON {STAFF_METRICS_ALIAS}.snapshot_id = {staff_alias}.snapshot_id AND {STAFF_METRICS_ALIAS}.uid = {staff_alias}.uid AND {STAFF_METRICS_ALIAS}.score_model_version = {SCORE_MODEL_VERSION}"
    )
}

/// Scoped read validation: every current staff member must have one compact
/// row carrying the checked-in score model. Missing or wrong-version state
/// fails before values are read; a read never writes or repairs.
pub(crate) fn assert_read_models_complete(
    conn: &Connection,
    snapshot_id: i64,
) -> Result<(), String> {
    let incomplete: bool = conn
        .query_row(
            "SELECT EXISTS(
                 SELECT 1
                 FROM staff s
                 LEFT JOIN staff_role_metrics m
                   ON m.snapshot_id = s.snapshot_id
                  AND m.uid = s.uid
                  AND m.score_model_version = ?2
                 WHERE s.snapshot_id = ?1 AND m.snapshot_id IS NULL
             )",
            params![snapshot_id, SCORE_MODEL_VERSION],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if incomplete {
        Err("Current compact staff snapshot is incomplete".to_string())
    } else {
        Ok(())
    }
}

/// Returns the compact `staff_role_metrics` column for a closed-catalog staff
/// role. The column name is the verified role id; identifiers never come from
/// WebView input.
pub fn staff_role_column(role_id: &str) -> Result<&'static str, String> {
    let role = all_staff_roles()
        .iter()
        .find(|role| role.role_id == role_id)
        .ok_or_else(|| format!("unknown staff role: {role_id}"))?;
    require_safe_snake_case(role.role_id)
}

#[cfg_attr(not(test), allow(dead_code))]
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

pub fn score_staff_role(
    attributes: &HashMap<String, Option<u8>>,
    role: &StaffRoleDefinition,
) -> Option<u8> {
    if role.attributes.is_empty() {
        return None;
    }
    let sum = role.attributes.iter().try_fold(0u32, |sum, key| {
        let value = attributes.get(*key).copied().flatten()?;
        (1..=20).contains(&value).then_some(sum + u32::from(value))
    })?;
    let mean = f64::from(sum) / role.attributes.len() as f64;
    Some((mean * 5.0).round() as u8)
}

/// One catalog-ordered compact row to persist for a staff member: the 21
/// role scores in `all_staff_roles()` order. Values are SQL null when a
/// required source attribute is missing.
pub(crate) struct CompactStaffRow {
    pub(crate) uid: i64,
    pub(crate) scores: Vec<Option<i64>>,
}

/// Scores every closed-catalog staff role in `all_staff_roles()` order.
/// A role whose source attributes are missing, null, or out of range scores
/// SQL null.
pub(crate) fn score_all_staff_roles(attributes: &HashMap<String, Option<u8>>) -> Vec<Option<i64>> {
    all_staff_roles()
        .iter()
        .map(|role| score_staff_role(attributes, role).map(i64::from))
        .collect()
}

/// Borrowed persistence implementation: inserts or replaces one compact row per staff member
/// without cloning the 21-value score vectors. Accepts any iterator over borrowed
/// slices directly from `PreparedStaff`.
pub(crate) fn persist_rows_borrowed<'a, I>(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    rows: I,
) -> Result<(), String>
where
    I: IntoIterator<Item = (i64, &'a [Option<i64>])>,
{
    let roles = all_staff_roles();
    let columns = roles
        .iter()
        .map(|role| staff_role_column(role.role_id).map(str::to_string))
        .collect::<Result<Vec<_>, _>>()?;
    let placeholders = (0..columns.len() + 3)
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "INSERT OR REPLACE INTO staff_role_metrics (
            snapshot_id, uid, score_model_version, {}
         ) VALUES ({placeholders})",
        columns.join(", ")
    );
    let mut statement = tx.prepare(&sql).map_err(|error| error.to_string())?;

    for (uid, scores) in rows {
        if scores.len() != roles.len() {
            return Err("Compact staff row has the wrong role count".to_string());
        }
        let mut values = Vec::with_capacity(columns.len() + 3);
        values.push(Value::Integer(snapshot_id));
        values.push(Value::Integer(uid));
        values.push(Value::Integer(SCORE_MODEL_VERSION));
        values.extend(
            scores
                .iter()
                .map(|score| score.map_or(Value::Null, Value::Integer)),
        );
        statement
            .execute(params_from_iter(values.iter()))
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// Inserts or replaces one compact row per staff member for one snapshot,
/// with the exact checked-in score model version. The SQL column list comes
/// only from the closed catalog via `staff_role_column`, never from WebView
/// input.
pub(crate) fn persist_rows(
    tx: &Transaction<'_>,
    snapshot_id: i64,
    rows: &[CompactStaffRow],
) -> Result<(), String> {
    persist_rows_borrowed(
        tx,
        snapshot_id,
        rows.iter().map(|row| (row.uid, row.scores.as_slice())),
    )
}

/// Deletes every compact staff row for one snapshot.
pub(crate) fn clear_snapshot(tx: &Transaction<'_>, snapshot_id: i64) -> Result<(), String> {
    tx.execute(
        "DELETE FROM staff_role_metrics WHERE snapshot_id = ?1",
        [snapshot_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Deletes compact staff rows from every non-current snapshot of one save,
/// so only the effective current snapshot keeps compact staff metrics.
pub(crate) fn clear_non_current_snapshots(
    tx: &Transaction<'_>,
    save_id: i64,
) -> Result<(), String> {
    tx.execute(
        "DELETE FROM staff_role_metrics
         WHERE snapshot_id IN (
             SELECT id FROM snapshots WHERE save_id = ?1 AND is_current = 0
         )",
        [save_id],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

/// Verifies that every staff member of an effective current snapshot has
/// exactly one compact row with the checked-in score model version. The
/// `(snapshot_id, uid)` primary key makes a second row per member impossible.
pub(crate) fn assert_snapshot_complete(conn: &Connection, snapshot_id: i64) -> Result<(), String> {
    let incomplete: bool = conn
        .query_row(
            "SELECT
                 NOT EXISTS(SELECT 1 FROM snapshots WHERE id = ?1 AND is_current = 1)
                 OR EXISTS(
                     SELECT 1
                     FROM staff s
                     LEFT JOIN staff_role_metrics m
                       ON m.snapshot_id = s.snapshot_id AND m.uid = s.uid
                     WHERE s.snapshot_id = ?1
                       AND (
                           m.snapshot_id IS NULL
                           OR m.score_model_version <> ?2
                       )
                 )",
            params![snapshot_id, SCORE_MODEL_VERSION],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if incomplete {
        Err("Current compact staff snapshot is incomplete".to_string())
    } else {
        Ok(())
    }
}

/// Rebuilds every current-only compact staff row for one snapshot from the
/// retained raw `staff_attributes_json` facts.
pub(crate) fn rebuild_snapshot(tx: &Transaction<'_>, snapshot_id: i64) -> Result<(), String> {
    require_current_snapshot(tx, snapshot_id)?;
    clear_snapshot(tx, snapshot_id)?;
    let mut rows = Vec::with_capacity(1);
    for member in load_staff(tx, snapshot_id)? {
        let attributes =
            serde_json::from_str::<HashMap<String, Option<u8>>>(&member.staff_attributes_json)
                .map_err(|error| {
                    format!("invalid staff {} attributes JSON: {error}", member.uid)
                })?;
        rows.push(CompactStaffRow {
            uid: member.uid,
            scores: score_all_staff_roles(&attributes),
        });
    }
    persist_rows(tx, snapshot_id, &rows)?;
    assert_snapshot_complete(tx, snapshot_id)
}

/// Clears compact staff rows from every non-current snapshot and rebuilds a
/// newly selected current snapshot from raw staff facts, mirroring the player
/// compact lifecycle from Commit 3.
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

struct StaffForMetrics {
    uid: i64,
    staff_attributes_json: String,
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
        Err("Staff role metrics require a current snapshot".to_string())
    }
}

fn load_staff(tx: &Transaction<'_>, snapshot_id: i64) -> Result<Vec<StaffForMetrics>, String> {
    let mut statement = tx
        .prepare(
            "SELECT uid, staff_attributes_json
             FROM staff WHERE snapshot_id = ?1 ORDER BY uid",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map([snapshot_id], |row| {
            Ok(StaffForMetrics {
                uid: row.get(0)?,
                staff_attributes_json: row.get(1)?,
            })
        })
        .map_err(|error| error.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use rusqlite::{params, Connection, OptionalExtension};

    /// A compact staff row read back for assertions: `(score_model_version,
    /// scores)` where scores are the 21 role values in catalog order.
    pub(crate) type CompactStaffRowShape = (i64, Vec<Option<i64>>);

    pub(crate) fn count_rows(conn: &Connection, snapshot_id: i64) -> i64 {
        conn.query_row(
            "SELECT COUNT(*) FROM staff_role_metrics WHERE snapshot_id = ?1",
            [snapshot_id],
            |row| row.get::<_, i64>(0),
        )
        .expect("count staff compact rows")
    }

    /// Reads one staff member's compact row, or `None` when no compact row
    /// exists for that `(snapshot_id, uid)`.
    pub(crate) fn read_row(
        conn: &Connection,
        snapshot_id: i64,
        uid: i64,
    ) -> Option<CompactStaffRowShape> {
        let roles = all_staff_roles();
        let columns = roles
            .iter()
            .map(|role| {
                staff_role_column(role.role_id)
                    .expect("staff column")
                    .to_string()
            })
            .collect::<Vec<_>>();
        let sql = format!(
            "SELECT score_model_version, {}
             FROM staff_role_metrics WHERE snapshot_id = ?1 AND uid = ?2",
            columns.join(", ")
        );
        conn.query_row(&sql, params![snapshot_id, uid], |row| {
            let scores = (0..roles.len())
                .map(|index| row.get::<_, Option<i64>>(index + 1))
                .collect::<Result<Vec<_>, _>>()?;
            Ok((row.get(0)?, scores))
        })
        .optional()
        .expect("read staff compact row")
    }

    /// JSON attributes for a staff member with every staff attribute key set
    /// to 10, so every closed-catalog role scores 50.
    pub(crate) fn all_ten_attributes_json() -> String {
        let attributes = super::super::metrics::STAFF_ATTRIBUTE_KEYS
            .iter()
            .map(|key| ((*key).to_string(), serde_json::Value::from(10)))
            .collect::<serde_json::Map<_, _>>();
        serde_json::to_string(&attributes).expect("serialize staff attributes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(pairs: &[(&str, Option<u8>)]) -> HashMap<String, Option<u8>> {
        pairs
            .iter()
            .map(|(key, value)| ((*key).to_string(), *value))
            .collect()
    }

    fn role(role_id: &str) -> &'static StaffRoleDefinition {
        all_staff_roles()
            .iter()
            .find(|role| role.role_id == role_id)
            .expect("known staff role")
    }

    #[test]
    fn catalog_has_twenty_one_unique_stable_ids_and_exact_special_formulas() {
        let roles = all_staff_roles();
        let unique_ids = roles
            .iter()
            .map(|role| role.role_id)
            .collect::<std::collections::HashSet<_>>();

        assert_eq!(roles.len(), 21);
        assert_eq!(unique_ids.len(), 21);
        let expected = [
            (
                "assistant_manager",
                &[
                    "ManManagement",
                    "JudgingPlayerPotential",
                    "JudgingPlayerAbility",
                ][..]
                    .as_ref(),
            ),
            (
                "manager",
                &[
                    "Motivating",
                    "ManManagement",
                    "JudgingPlayerAbility",
                    "JudgingPlayerPotential",
                    "TacticalKnowledge",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_attacking_technical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Attacking",
                    "Technical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_attacking_tactical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Attacking",
                    "Tactical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_defending_technical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Defending",
                    "Technical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_defending_tactical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Defending",
                    "Tactical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_possession_technical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Possession",
                    "Technical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_possession_tactical",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "Possession",
                    "Tactical",
                ][..]
                    .as_ref(),
            ),
            (
                "coach_fitness",
                &["Authority", "Determination", "Motivating", "Fitness"][..].as_ref(),
            ),
            (
                "coach_goalkeeping",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "GoalkeepingDistribution",
                    "GoalkeepingHandling",
                    "GoalkeepingReflexes",
                ][..]
                    .as_ref(),
            ),
            (
                "set_piece_coach",
                &[
                    "Authority",
                    "Determination",
                    "Motivating",
                    "SetPieces",
                    "TacticalKnowledge",
                ][..]
                    .as_ref(),
            ),
            (
                "loan_manager",
                &[
                    "ManManagement",
                    "JudgingPlayerPotential",
                    "JudgingPlayerAbility",
                ][..]
                    .as_ref(),
            ),
            (
                "head_of_youth_development",
                &[
                    "WorkingWithYoungsters",
                    "JudgingPlayerPotential",
                    "JudgingPlayerAbility",
                ][..]
                    .as_ref(),
            ),
            (
                "scout",
                &[
                    "Adaptability",
                    "JudgingPlayerPotential",
                    "JudgingPlayerAbility",
                ][..]
                    .as_ref(),
            ),
            (
                "director_of_football",
                &[
                    "JudgingPlayerPotential",
                    "JudgingPlayerAbility",
                    "Negotiating",
                ][..]
                    .as_ref(),
            ),
            (
                "technical_director",
                &["JudgingStaffAbility", "Negotiating"][..].as_ref(),
            ),
            (
                "recruitment_analyst",
                &["DataAnalysis", "JudgingPlayerAbility"][..].as_ref(),
            ),
            (
                "head_performance_analyst",
                &[
                    "DataAnalysis",
                    "Determination",
                    "JudgingPlayerAbility",
                    "TacticalKnowledge",
                ][..]
                    .as_ref(),
            ),
            (
                "performance_analyst",
                &["DataAnalysis", "TacticalKnowledge"][..].as_ref(),
            ),
            ("physio", &["Physiotherapy"][..].as_ref()),
            ("sports_scientist", &["SportsScience"][..].as_ref()),
        ];
        for (role_id, attributes) in expected {
            assert_eq!(
                role(role_id).attributes,
                *attributes,
                "formula for {role_id}"
            );
        }
    }

    #[test]
    fn scores_single_and_multi_attribute_roles_on_a_hundred_point_scale() {
        assert_eq!(
            score_staff_role(&attrs(&[("Physiotherapy", Some(17))]), role("physio")),
            Some(85)
        );
        assert_eq!(
            score_staff_role(
                &attrs(&[
                    ("ManManagement", Some(10)),
                    ("JudgingPlayerPotential", Some(11)),
                    ("JudgingPlayerAbility", Some(12)),
                ]),
                role("assistant_manager"),
            ),
            Some(55)
        );
        assert_eq!(
            score_staff_role(
                &attrs(&[
                    ("Motivating", Some(10)),
                    ("ManManagement", Some(11)),
                    ("JudgingPlayerAbility", Some(12)),
                    ("JudgingPlayerPotential", Some(13)),
                    ("TacticalKnowledge", Some(14)),
                ]),
                role("manager"),
            ),
            Some(60)
        );
    }

    #[test]
    fn rounds_the_mean_after_scaling() {
        assert_eq!(
            score_staff_role(
                &attrs(&[("DataAnalysis", Some(10)), ("TacticalKnowledge", Some(11))]),
                role("performance_analyst"),
            ),
            Some(53)
        );
    }

    #[test]
    fn rejects_missing_null_and_out_of_range_inputs() {
        assert_eq!(score_staff_role(&HashMap::new(), role("physio")), None);
        assert_eq!(
            score_staff_role(&attrs(&[("Physiotherapy", None)]), role("physio")),
            None
        );
        assert_eq!(
            score_staff_role(&attrs(&[("Physiotherapy", Some(0))]), role("physio")),
            None
        );
        assert_eq!(
            score_staff_role(&attrs(&[("Physiotherapy", Some(21))]), role("physio")),
            None
        );
    }

    #[test]
    fn runtime_staff_catalog_maps_once_to_the_checked_in_compact_staff_schema() {
        let roles = all_staff_roles();
        assert_eq!(roles.len(), 21);
        let columns = roles
            .iter()
            .map(|role| staff_role_column(role.role_id))
            .collect::<Result<Vec<_>, _>>()
            .expect("map staff columns");
        assert_eq!(
            columns
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            21,
            "staff columns must be unique per role"
        );

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = rusqlite::Connection::open(temp_dir.path().join("compact-staff-contract.db"))
            .expect("open contract test db");
        crate::db::migrations::apply(&conn).expect("apply migrations");
        let mut statement = conn
            .prepare("SELECT name FROM pragma_table_info(?1) ORDER BY cid")
            .expect("prepare table info query");
        let schema = statement
            .query_map(["staff_role_metrics"], |row| row.get(0))
            .expect("query table info")
            .collect::<Result<Vec<String>, _>>()
            .expect("read table columns");

        let expected = ["snapshot_id", "uid", "score_model_version"]
            .into_iter()
            .map(str::to_string)
            .chain(columns.into_iter().map(str::to_string))
            .collect::<Vec<_>>();
        assert_eq!(schema.len(), 24);
        assert_eq!(schema, expected);

        // The version column rejects zero; writers persist exactly this
        // checked-in score model version into it.
        assert!([SCORE_MODEL_VERSION].iter().all(|version| *version > 0));
    }

    #[test]
    fn rejects_unknown_or_unsafe_staff_role_identifiers() {
        for id in [
            "physiotherapist",
            "Physio",
            "Manager",
            "set-piece-coach",
            "1st_coach",
            "",
            "with space",
            "coach!",
            "camelCase",
            "_leading_underscore",
        ] {
            assert!(staff_role_column(id).is_err(), "{id}");
        }
    }
}
