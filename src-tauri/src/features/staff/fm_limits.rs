//! FM's board staff limits as staffing needs. Each Load Data reads the limits through the
//! bridge and sets a need to its limit whenever that limit differs from the one read before, so
//! the needs follow FM while an edit made in between stays until FM's limit changes. Each
//! Planner team's limits set its team jobs; the first team's limits set the club-wide jobs.

use std::collections::BTreeMap;

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

use crate::db::Db;
use crate::features::managed_club::service::selected_club_identity;
use crate::features::memory_read::service::StaffLimitsResult;
use crate::features::snapshot::service::SaveContext;

use super::assignment_optimizer::target_job_for_fm_job;
use super::assignment_targets::{enabled_teams, is_team_job, target_max_slot_count, CLUB_SCOPE};

/// FM's board counts every coach, general, goalkeeping or fitness, in one row of this job type,
/// which no staff contract holds.
const BOARD_COACHES_JOB: i32 = 104;

/// What following FM's staff limits did during one Load Data. Only `Followed` changes needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub(crate) enum StaffLimitsOutcome {
    /// `changed` is how many needs took a new limit. `unread_teams` names the enabled Planner
    /// teams FM sent no limits for; their needs are unchanged.
    Followed {
        changed: usize,
        unread_teams: Vec<String>,
    },
    /// The limits were not read or not stored; `reason` says why. Needs are unchanged.
    NotRead { reason: String },
    /// The save has no managed club with an FM identity, so there is nothing to read.
    NoManagedClub,
    /// The active save changed during the read, so the limits were discarded.
    StaleContext,
}

/// Reads FM's staff limits for the managed club through `read`, then sets every need whose
/// limit changed. `read` takes the live scan's request ID and the club's Unique ID.
pub(crate) fn follow_fm_staff_limits(
    db: &Db,
    save_context: &SaveContext,
    source_request_id: &str,
    read: impl FnOnce(&str, u32) -> Result<StaffLimitsResult, String>,
) -> StaffLimitsOutcome {
    let club_uid = match managed_club_uid(db, save_context.id) {
        Ok(Some(uid)) => uid,
        Ok(None) => return StaffLimitsOutcome::NoManagedClub,
        Err(reason) => return StaffLimitsOutcome::NotRead { reason },
    };
    let limits = match read(source_request_id, club_uid) {
        Ok(limits) => limits,
        Err(reason) => return StaffLimitsOutcome::NotRead { reason },
    };
    let stored =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())
            .and_then(|conn| apply_staff_limits(&conn, save_context, &limits));
    stored.unwrap_or_else(|error| {
        log::warn!("FM staff limits were read but not stored: {error}");
        StaffLimitsOutcome::NotRead {
            reason: "they could not be stored".to_string(),
        }
    })
}

fn managed_club_uid(db: &Db, save_id: i64) -> Result<Option<u32>, String> {
    let conn = db.read()?;
    Ok(match selected_club_identity(&conn, save_id)? {
        Some((_, Some(uid))) => u32::try_from(uid).ok().filter(|uid| *uid > 0),
        _ => None,
    })
}

/// Sets each need whose FM limit differs from the one stored, stores the new limits, and
/// leaves every other need alone.
pub(crate) fn apply_staff_limits(
    conn: &Connection,
    save_context: &SaveContext,
    limits: &StaffLimitsResult,
) -> Result<StaffLimitsOutcome, String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    let active: bool = tx
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1 AND context_token = ?2 AND is_active = 1)",
            params![save_context.id, save_context.context_token],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !active {
        return Ok(StaffLimitsOutcome::StaleContext);
    }

    let save_id = save_context.id;
    let teams = enabled_teams(&tx, save_id)?;
    let enabled = teams
        .iter()
        .map(|team| team.team.as_str())
        .collect::<Vec<_>>();
    let (needs, read_scopes) = needs_from_limits(limits, &enabled);
    let mut changed = 0;
    for ((scope, job_id), limit) in &needs {
        let previous: Option<i64> = tx
            .query_row(
                "SELECT slot_limit FROM staff_assignment_fm_limits
                 WHERE save_id = ?1 AND scope = ?2 AND job_id = ?3",
                params![save_id, scope, job_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        if previous == Some(*limit) {
            continue;
        }

        let need: i64 = tx
            .query_row(
                "SELECT slot_count FROM staff_assignment_targets
                 WHERE save_id = ?1 AND scope = ?2 AND job_id = ?3",
                params![save_id, scope, job_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .unwrap_or(0);
        if need != *limit {
            changed += 1;
            if *limit == 0 {
                tx.execute(
                    "DELETE FROM staff_assignment_targets
                     WHERE save_id = ?1 AND scope = ?2 AND job_id = ?3",
                    params![save_id, scope, job_id],
                )
            } else {
                tx.execute(
                    "INSERT INTO staff_assignment_targets (save_id, scope, job_id, slot_count)
                     VALUES (?1, ?2, ?3, ?4)
                     ON CONFLICT (save_id, scope, job_id) DO UPDATE SET slot_count = excluded.slot_count",
                    params![save_id, scope, job_id, limit],
                )
            }
            .map_err(|error| error.to_string())?;
        }
        tx.execute(
            "INSERT INTO staff_assignment_fm_limits (save_id, scope, job_id, slot_limit)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT (save_id, scope, job_id) DO UPDATE SET slot_limit = excluded.slot_limit",
            params![save_id, scope, job_id, limit],
        )
        .map_err(|error| error.to_string())?;
    }

    let unread_teams = teams
        .iter()
        .filter(|team| !read_scopes.contains(&team.team.as_str()))
        .map(|team| team.display_name.clone())
        .collect();
    tx.commit().map_err(|error| error.to_string())?;
    Ok(StaffLimitsOutcome::Followed {
        changed,
        unread_teams,
    })
}

/// The needs FM's limits set, keyed by scope and target job, and the enabled Planner teams FM
/// sent limits for. Rows of FM jobs that fill one target add up. The first team's Manager is the
/// user, so it sets no need.
fn needs_from_limits(
    limits: &StaffLimitsResult,
    enabled: &[&str],
) -> (BTreeMap<(String, String), i64>, Vec<&'static str>) {
    let mut needs = BTreeMap::<(String, String), i64>::new();
    let mut read_scopes = Vec::new();
    for team in limits.teams.iter().filter(|team| !team.groups.is_empty()) {
        let Some(scope) = planner_scope(&team.level) else {
            continue;
        };
        let team_enabled = enabled.contains(&scope);
        if team_enabled {
            read_scopes.push(scope);
        }
        for job in team.groups.iter().flat_map(|group| &group.jobs) {
            let target = if job.job_type == BOARD_COACHES_JOB {
                Some("coaches")
            } else {
                target_job_for_fm_job(i64::from(job.job_type))
            };
            let Some(target) = target else {
                continue;
            };
            let need_scope = if is_team_job(target) {
                if !team_enabled {
                    continue;
                }
                scope
            } else if scope == "senior" {
                CLUB_SCOPE
            } else {
                continue;
            };
            if target_max_slot_count(need_scope, target).is_some() {
                *needs
                    .entry((need_scope.to_string(), target.to_string()))
                    .or_default() += i64::from(job.max.max(0));
            }
        }
    }
    for ((scope, job_id), need) in &mut needs {
        if let Some(max) = target_max_slot_count(scope, job_id) {
            *need = (*need).min(max);
        }
    }
    (needs, read_scopes)
}

fn planner_scope(level: &str) -> Option<&'static str> {
    match level {
        "senior" => Some("senior"),
        "reserve" => Some("reserves"),
        "youth" => Some("youth"),
        _ => None,
    }
}
