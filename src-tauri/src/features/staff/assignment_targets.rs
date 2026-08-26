use std::collections::HashSet;

use rusqlite::{params, Connection, Transaction};

use crate::features::planner::teams::ensure_team_settings;
use crate::features::snapshot::service::{capture_active_save_context, ensure_save_context};

pub(crate) const CLUB_SCOPE: &str = "club";
const TEAM_SCOPES: [&str; 3] = ["senior", "reserves", "youth"];
const TEAM_JOBS: [(&str, &str); 10] = [
    ("manager", "Manager"),
    ("assistant_manager", "Assistant Manager"),
    ("coaches", "Coaches"),
    ("set_piece_coach", "Set Piece Coach"),
    ("head_performance_analyst", "Head Performance Analyst"),
    ("performance_analyst", "Performance Analyst"),
    ("head_physio", "Head Physio"),
    ("physio", "Physio"),
    ("head_sports_science", "Head of Sports Science"),
    ("sports_scientist", "Sports Scientist"),
];
const CLUB_JOBS: [(&str, &str); 6] = [
    ("head_of_youth_development", "Head of Youth Development"),
    ("director_of_football", "Director of Football"),
    ("technical_director", "Technical Director"),
    ("loan_manager", "Loan Manager"),
    ("chief_scout", "Chief Scout"),
    ("scout", "Scout"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaffAssignmentTarget {
    pub(crate) scope: String,
    pub(crate) job_id: String,
    pub(crate) job_label: String,
    pub(crate) slot_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaffAssignmentTargetInput {
    pub(crate) scope: String,
    pub(crate) job_id: String,
    pub(crate) slot_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaffAssignmentTargetTeam {
    pub(crate) team: String,
    pub(crate) display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaffAssignmentTargets {
    pub(crate) teams: Vec<StaffAssignmentTargetTeam>,
    pub(crate) targets: Vec<StaffAssignmentTarget>,
}

pub(crate) fn get_targets(
    conn: &Connection,
    expected_save_context_token: &str,
) -> Result<StaffAssignmentTargets, String> {
    let context = capture_active_save_context(conn)?;
    if context.context_token != expected_save_context_token {
        return Err("Save changed or no longer exists".to_string());
    }
    read_targets(conn, context.id)
}

pub(crate) fn save_targets(
    conn: &Connection,
    expected_save_context_token: &str,
    inputs: &[StaffAssignmentTargetInput],
) -> Result<StaffAssignmentTargets, String> {
    let context = capture_active_save_context(conn)?;
    if context.context_token != expected_save_context_token {
        return Err("Save changed or no longer exists".to_string());
    }
    let teams = enabled_teams(conn, context.id)?;
    let allowed = allowed_pairs_for_teams(&teams);
    validate_complete_inputs(inputs, &allowed)?;

    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    ensure_save_context(&tx, &context)?;
    tx.execute(
        "DELETE FROM staff_assignment_targets WHERE save_id = ?1",
        [context.id],
    )
    .map_err(|error| error.to_string())?;
    for input in inputs.iter().filter(|input| input.slot_count > 0) {
        tx.execute(
            "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
             VALUES (?1, ?2, ?3, ?4)",
            params![context.id, input.scope, input.job_id, input.slot_count],
        )
        .map_err(|error| error.to_string())?;
    }
    tx.commit().map_err(|error| error.to_string())?;
    Ok(StaffAssignmentTargets {
        teams,
        targets: expand_targets(conn, context.id, &allowed)?,
    })
}

fn read_targets(conn: &Connection, save_id: i64) -> Result<StaffAssignmentTargets, String> {
    let teams = enabled_teams(conn, save_id)?;
    let allowed = allowed_pairs_for_teams(&teams);
    Ok(StaffAssignmentTargets {
        teams,
        targets: expand_targets(conn, save_id, &allowed)?,
    })
}

pub(crate) fn read_nonzero_targets_without_initializing_teams(
    conn: &Connection,
    save_id: i64,
) -> Result<Vec<StaffAssignmentTarget>, String> {
    let teams = read_enabled_teams(conn, save_id)?;
    let allowed = allowed_pairs_for_teams(&teams);
    Ok(expand_targets(conn, save_id, &allowed)?
        .into_iter()
        .filter(|target| target.slot_count > 0)
        .collect())
}

pub(crate) fn nonzero_targets_for_scope(
    conn: &Connection,
    save_id: i64,
    scope: &str,
) -> Result<Vec<StaffAssignmentTarget>, String> {
    let allowed = allowed_pairs(conn, save_id)?;
    let targets = expand_targets(conn, save_id, &allowed)?;
    Ok(targets
        .into_iter()
        .filter(|target| target.scope == scope && target.slot_count > 0)
        .collect())
}

pub(crate) fn nonzero_targets_for_scope_tx(
    tx: &Transaction<'_>,
    save_id: i64,
    scope: &str,
) -> Result<Vec<StaffAssignmentTarget>, String> {
    let rows = tx
        .prepare(
            "SELECT job_id, slot_count FROM staff_assignment_targets
             WHERE save_id = ?1 AND scope = ?2 ORDER BY job_id",
        )
        .map_err(|error| error.to_string())?
        .query_map(params![save_id, scope], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    rows.into_iter()
        .map(|(job_id, slot_count)| {
            let job_label = job_label(scope, &job_id).ok_or_else(|| {
                "Stored staff assignment targets are invalid for the current Planner teams"
                    .to_string()
            })?;
            if !(1..=50).contains(&slot_count) {
                return Err(
                    "Stored staff assignment targets are invalid for the current Planner teams"
                        .to_string(),
                );
            }
            Ok(StaffAssignmentTarget {
                scope: scope.to_string(),
                job_id,
                job_label: job_label.to_string(),
                slot_count,
            })
        })
        .collect()
}

pub(crate) fn delete_scope_targets(
    tx: &Transaction<'_>,
    save_id: i64,
    scope: &str,
) -> Result<(), String> {
    tx.execute(
        "DELETE FROM staff_assignment_targets WHERE save_id = ?1 AND scope = ?2",
        params![save_id, scope],
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}

fn allowed_pairs(conn: &Connection, save_id: i64) -> Result<Vec<StaffAssignmentTarget>, String> {
    Ok(allowed_pairs_for_teams(&enabled_teams(conn, save_id)?))
}

fn enabled_teams(
    conn: &Connection,
    save_id: i64,
) -> Result<Vec<StaffAssignmentTargetTeam>, String> {
    ensure_team_settings(conn, save_id)?;
    let teams = read_enabled_teams(conn, save_id)?;
    if teams.is_empty() {
        return Err("Planner team settings are unavailable for this save".to_string());
    }
    Ok(teams)
}

fn read_enabled_teams(
    conn: &Connection,
    save_id: i64,
) -> Result<Vec<StaffAssignmentTargetTeam>, String> {
    let teams = conn
        .prepare(
            "SELECT team, display_name FROM planner_teams WHERE save_id = ?1
             ORDER BY CASE team WHEN 'senior' THEN 0 WHEN 'reserves' THEN 1 ELSE 2 END",
        )
        .map_err(|error| error.to_string())?
        .query_map([save_id], |row| {
            Ok(StaffAssignmentTargetTeam {
                team: row.get(0)?,
                display_name: row.get(1)?,
            })
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    if teams
        .iter()
        .any(|team| !TEAM_SCOPES.contains(&team.team.as_str()))
    {
        return Err("Planner team settings are unavailable for this save".to_string());
    }
    Ok(teams)
}

fn allowed_pairs_for_teams(teams: &[StaffAssignmentTargetTeam]) -> Vec<StaffAssignmentTarget> {
    let mut allowed = Vec::new();
    for team in teams {
        for (job_id, job_label) in TEAM_JOBS {
            if team.team == "senior" && job_id == "manager" {
                continue;
            }
            allowed.push(StaffAssignmentTarget {
                scope: team.team.clone(),
                job_id: job_id.to_string(),
                job_label: job_label.to_string(),
                slot_count: 0,
            });
        }
    }
    for (job_id, job_label) in CLUB_JOBS {
        allowed.push(StaffAssignmentTarget {
            scope: CLUB_SCOPE.to_string(),
            job_id: job_id.to_string(),
            job_label: job_label.to_string(),
            slot_count: 0,
        });
    }
    allowed
}

fn expand_targets(
    conn: &Connection,
    save_id: i64,
    allowed: &[StaffAssignmentTarget],
) -> Result<Vec<StaffAssignmentTarget>, String> {
    let allowed_keys = allowed
        .iter()
        .map(|target| (target.scope.as_str(), target.job_id.as_str()))
        .collect::<HashSet<_>>();
    let mut stored = conn
        .prepare(
            "SELECT scope, job_id, slot_count
             FROM staff_assignment_targets WHERE save_id = ?1",
        )
        .map_err(|error| error.to_string())?
        .query_map([save_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    for (scope, job_id, slot_count) in &stored {
        if !allowed_keys.contains(&(scope.as_str(), job_id.as_str()))
            || !(1..=50).contains(slot_count)
        {
            return Err(
                "Stored staff assignment targets are invalid for the current Planner teams"
                    .to_string(),
            );
        }
    }
    let mut result = allowed.to_vec();
    for target in &mut result {
        if let Some(index) = stored
            .iter()
            .position(|(scope, job_id, _)| scope == &target.scope && job_id == &target.job_id)
        {
            target.slot_count = stored.swap_remove(index).2;
        }
    }
    Ok(result)
}

fn job_label(scope: &str, job_id: &str) -> Option<&'static str> {
    if scope == CLUB_SCOPE {
        return CLUB_JOBS
            .iter()
            .find_map(|(candidate_id, label)| (*candidate_id == job_id).then_some(*label));
    }
    if !TEAM_SCOPES.contains(&scope) || (scope == "senior" && job_id == "manager") {
        return None;
    }
    TEAM_JOBS
        .iter()
        .find_map(|(candidate_id, label)| (*candidate_id == job_id).then_some(*label))
}

fn validate_complete_inputs(
    inputs: &[StaffAssignmentTargetInput],
    allowed: &[StaffAssignmentTarget],
) -> Result<(), String> {
    let allowed_keys = allowed
        .iter()
        .map(|target| (target.scope.as_str(), target.job_id.as_str()))
        .collect::<HashSet<_>>();
    if inputs.len() != allowed.len() {
        return Err(
            "Staff assignment targets must include every allowed scope and job".to_string(),
        );
    }
    let mut seen = HashSet::new();
    for input in inputs {
        let key = (input.scope.as_str(), input.job_id.as_str());
        if !allowed_keys.contains(&key) {
            return Err("Staff assignment target scope or job is not allowed".to_string());
        }
        if !seen.insert(key) {
            return Err("Staff assignment targets must not contain duplicates".to_string());
        }
        if !(0..=50).contains(&input.slot_count) {
            return Err("Staff assignment target slot count must be between 0 and 50".to_string());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations;
    use crate::features::snapshot::service;

    fn connection() -> (tempfile::TempDir, Connection, i64, String) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("targets.db")).expect("open db");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("foreign keys");
        migrations::apply(&conn).expect("migrate");
        let context = service::capture_active_save_context(&conn).expect("active save");
        (temp_dir, conn, context.id, context.context_token)
    }

    #[test]
    fn replaces_the_exact_complete_set_and_compacts_zeroes() {
        let (_temp_dir, conn, save_id, token) = connection();
        let before = get_targets(&conn, &token).expect("expanded targets");
        assert_eq!(
            before.teams,
            vec![
                StaffAssignmentTargetTeam {
                    team: "senior".to_string(),
                    display_name: "Senior".to_string(),
                },
                StaffAssignmentTargetTeam {
                    team: "reserves".to_string(),
                    display_name: "Reserves".to_string(),
                },
                StaffAssignmentTargetTeam {
                    team: "youth".to_string(),
                    display_name: "Youth".to_string(),
                },
            ]
        );
        assert_eq!(before.targets.len(), 35);
        let inputs = before
            .targets
            .iter()
            .map(|target| StaffAssignmentTargetInput {
                scope: target.scope.clone(),
                job_id: target.job_id.clone(),
                slot_count: i64::from(
                    target.scope == "senior" && target.job_id == "assistant_manager",
                ),
            })
            .collect::<Vec<_>>();
        let saved = save_targets(&conn, &token, &inputs).expect("save complete targets");
        assert_eq!(
            saved
                .targets
                .iter()
                .filter(|target| target.slot_count > 0)
                .count(),
            1
        );
        assert_eq!(
            conn.query_row(
                "SELECT COUNT(*) FROM staff_assignment_targets WHERE save_id = ?1",
                [save_id],
                |row| row.get::<_, i64>(0)
            )
            .expect("stored targets"),
            1
        );
    }

    #[test]
    fn rolls_back_replacement_when_an_insert_fails() {
        let (_temp_dir, conn, _save_id, token) = connection();
        let complete = get_targets(&conn, &token).expect("targets");
        let initial = complete
            .targets
            .iter()
            .map(|target| StaffAssignmentTargetInput {
                scope: target.scope.clone(),
                job_id: target.job_id.clone(),
                slot_count: i64::from(
                    target.scope == "senior" && target.job_id == "assistant_manager",
                ),
            })
            .collect::<Vec<_>>();
        save_targets(&conn, &token, &initial).expect("seed targets");
        conn.execute_batch(
            "CREATE TRIGGER reject_target_insert
             BEFORE INSERT ON staff_assignment_targets
             BEGIN SELECT RAISE(ABORT, 'insertion denied'); END",
        )
        .expect("reject trigger");
        let replacement = complete
            .targets
            .iter()
            .map(|target| StaffAssignmentTargetInput {
                scope: target.scope.clone(),
                job_id: target.job_id.clone(),
                slot_count: i64::from(target.job_id == "coaches"),
            })
            .collect::<Vec<_>>();
        assert!(save_targets(&conn, &token, &replacement).is_err());
        assert_eq!(
            get_targets(&conn, &token)
                .expect("rolled back targets")
                .targets
                .into_iter()
                .filter(|target| target.slot_count > 0)
                .map(|target| target.job_id)
                .collect::<Vec<_>>(),
            ["assistant_manager".to_string()]
        );
    }

    #[test]
    fn rejects_stale_tokens_and_persisted_disallowed_pairs() {
        let (_temp_dir, conn, save_id, token) = connection();
        assert!(get_targets(&conn, "stale-token").is_err());
        conn.execute(
            "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
             VALUES (?1, 'senior', 'manager', 1)",
            [save_id],
        )
        .expect("insert invalid pair");
        assert!(get_targets(&conn, &token).is_err());
    }

    #[test]
    fn rejects_invalid_complete_replacement_before_mutation() {
        let (_temp_dir, conn, _save_id, token) = connection();
        let complete = get_targets(&conn, &token).expect("targets");
        let inputs = complete
            .targets
            .iter()
            .map(|target| StaffAssignmentTargetInput {
                scope: target.scope.clone(),
                job_id: target.job_id.clone(),
                slot_count: 1,
            })
            .collect::<Vec<_>>();
        save_targets(&conn, &token, &inputs).expect("seed targets");
        let error = save_targets(&conn, &token, &inputs[..inputs.len() - 1])
            .expect_err("reject missing target");
        assert!(error.contains("every allowed"));
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM staff_assignment_targets", [], |row| {
                row.get::<_, i64>(0)
            })
            .expect("retained rows"),
            35
        );
    }
}
