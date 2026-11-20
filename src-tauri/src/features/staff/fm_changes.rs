//! Applying the staff assignment optimizer's result in FM.
//!
//! A recruit is hired through the in-game editor's "Move to my club" with the slot's FM job and
//! team, so FM gives a contract and wage for the job. A club staff member who changes job or team
//! is sacked and hired again the same way, because FM cannot change a contract's job. Every
//! surplus staff member is sacked. The bridge runs every sack before any hire and confirms each
//! change from FM's contracts; this module plans the changes and records what FM reports.

use rusqlite::{params, Connection, OptionalExtension};

use super::assignment_optimizer::{
    current_job_classification, CoachRequirement, StaffAssignmentAction, StaffAssignmentSlot,
};
use super::assignment_optimizer_query::{
    optimize_staff_assignments, StaffAssignmentOptimizationState,
};
use crate::db::Db;
use crate::features::managed_club::service::capture_fm_action_context;
use crate::features::memory_read::service::{
    StaffChangeItem, StaffChangeOutcome, StaffChangeResponse, MAX_STAFF_CHANGES,
};
use crate::features::player::boost_gate;

pub(super) const LOAD_DATA_FIRST: &str =
    "Load Data with FM open on this save before changing staff in FM";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FmStaffChangeKind {
    /// End the contract of club staff the optimizer leaves without a slot.
    Sack,
    /// Sack club staff and hire them again in the slot's job and team.
    Change,
    /// Bring in someone from another club, or unemployed.
    Hire,
}

impl FmStaffChangeKind {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Sack => "sack",
            Self::Change => "change",
            Self::Hire => "hire",
        }
    }
}

/// The slot a hire or change fills, with the FM job type and team level it asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmStaffTarget {
    pub(super) scope: String,
    pub(super) scope_display_name: String,
    pub(super) job_label: &'static str,
    pub(super) job_type: i32,
    pub(super) level: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmStaffChange {
    pub(super) uid: i64,
    pub(super) name: Option<String>,
    pub(super) kind: FmStaffChangeKind,
    /// The job they hold now, as FM names it, for club staff whose job is known.
    pub(super) current_job: Option<String>,
    /// The display name of their current job's team, when the job has one and it is known.
    pub(super) current_team_name: Option<String>,
    /// Their club now, for a hire; none when they are unemployed.
    pub(super) current_club: Option<String>,
    /// Their weekly wage now, from the snapshot.
    pub(super) weekly_wage_gbp: Option<i64>,
    pub(super) target: Option<FmStaffTarget>,
}

/// FM's job type for a slot, and the job's name as FM shows it.
pub(super) fn fm_job(
    job_id: &str,
    coach_requirement: Option<CoachRequirement>,
) -> Option<(i32, &'static str)> {
    Some(match job_id {
        "manager" => (16, "Manager"),
        "assistant_manager" => (20, "Assistant Manager"),
        "coaches" => match coach_requirement {
            Some(CoachRequirement::Fitness) => (26, "Fitness Coach"),
            Some(CoachRequirement::Goalkeeping) => (34, "Goalkeeping Coach"),
            _ => (2, "Coach"),
        },
        "performance_analyst" => (60, "Performance Analyst"),
        "physio" => (12, "Physio"),
        "sports_scientist" => (48, "Sports Scientist"),
        "head_of_youth_development" => (64, "Head of Youth Development"),
        "head_performance_analyst" => (62, "Head Performance Analyst"),
        // Manchester United's set piece specialist holds job type 22; FM's title for it is not
        // yet confirmed on screen.
        "set_piece_coach" => (22, "Set Piece Coach"),
        "director_of_football" => (10, "Director of Football"),
        "chief_scout" => (44, "Chief Scout"),
        "technical_director" => (88, "Technical Director"),
        "scout" => (14, "Scout"),
        "recruitment_analyst" => (58, "Recruitment Analyst"),
        "loan_manager" => (86, "Loan Manager"),
        "head_physio" => (50, "Head Physio"),
        "head_sports_science" => (40, "Head of Sports Science"),
        _ => return None,
    })
}

/// The FM team level a slot's scope stands for; club-wide jobs belong to the first team.
fn level_for_scope(scope: &str) -> &'static str {
    match scope {
        "reserves" => "reserve",
        "youth" => "youth",
        _ => "senior",
    }
}

/// FM's name for a job type, when FM ValueScout knows it.
pub(super) fn fm_job_label(job_type: i32) -> Option<&'static str> {
    if job_type == 22 {
        return Some("Set Piece Coach");
    }
    current_job_classification(i64::from(job_type)).map(|(label, _)| label)
}

/// The sacks, job changes, and hires that put the optimizer's current result into FM, in that
/// order. Kept staff, and staff whose FM job FM ValueScout cannot name, need no change.
pub(super) fn plan_fm_staff_changes(
    conn: &Connection,
    expected_save_context_token: &str,
    expected_snapshot_context_token: &str,
) -> Result<Vec<FmStaffChange>, String> {
    let optimization = optimize_staff_assignments(
        conn,
        expected_save_context_token,
        expected_snapshot_context_token,
    )?;
    let snapshot_id = match optimization.state {
        StaffAssignmentOptimizationState::Ready => optimization
            .snapshot_id
            .ok_or_else(|| "Load Data first".to_string())?,
        StaffAssignmentOptimizationState::StaleContext => {
            return Err("The data changed; optimize your staff again".to_string())
        }
        StaffAssignmentOptimizationState::NoCurrentSnapshot => {
            return Err("Load Data first".to_string())
        }
        StaffAssignmentOptimizationState::NoManagedClub => {
            return Err("Choose your managed club in My Club first".to_string())
        }
        StaffAssignmentOptimizationState::NoShortlist => {
            return Err("Shortlist staff in FM first".to_string())
        }
    };
    let team_name = |scope: &str| optimization.team_names.get(scope).cloned();

    let mut sacks = Vec::new();
    for surplus in &optimization.surplus {
        sacks.push(FmStaffChange {
            uid: surplus.uid,
            name: surplus.name.clone(),
            kind: FmStaffChangeKind::Sack,
            current_job: surplus.current_job.map(str::to_string),
            current_team_name: surplus.current_team.and_then(team_name),
            current_club: None,
            weekly_wage_gbp: None,
            target: None,
        });
    }

    let mut changes = Vec::new();
    let mut hires = Vec::new();
    for result_slot in &optimization.slots {
        let StaffAssignmentSlot::Recommendation(recommendation) = &result_slot.slot else {
            continue;
        };
        let Some((job_type, job_label)) =
            fm_job(&recommendation.job_id, recommendation.coach_requirement)
        else {
            continue;
        };
        let target = FmStaffTarget {
            scope: recommendation.scope.clone(),
            scope_display_name: result_slot.scope_display_name.clone(),
            job_label,
            job_type,
            level: level_for_scope(&recommendation.scope),
        };
        let (kind, current_job, current_team_name, current_club) = match &recommendation.action {
            StaffAssignmentAction::Keep | StaffAssignmentAction::OnStaff => continue,
            StaffAssignmentAction::Recruit { club } => {
                (FmStaffChangeKind::Hire, None, None, club.clone())
            }
            StaffAssignmentAction::MoveTeam { current_team } => (
                FmStaffChangeKind::Change,
                Some(job_label.to_string()),
                team_name(current_team),
                None,
            ),
            StaffAssignmentAction::ChangeJob {
                current_job,
                current_team,
            } => (
                FmStaffChangeKind::Change,
                Some((*current_job).to_string()),
                current_team.and_then(team_name),
                None,
            ),
        };
        let change = FmStaffChange {
            uid: recommendation.uid,
            name: recommendation.name.clone(),
            kind,
            current_job,
            current_team_name,
            current_club,
            weekly_wage_gbp: None,
            target: Some(target),
        };
        if kind == FmStaffChangeKind::Hire {
            hires.push(change);
        } else {
            changes.push(change);
        }
    }

    let mut plan = sacks;
    plan.extend(changes);
    plan.extend(hires);
    for change in &mut plan {
        change.weekly_wage_gbp = conn
            .query_row(
                "SELECT weekly_wage_gbp FROM staff WHERE snapshot_id = ?1 AND uid = ?2",
                params![snapshot_id, change.uid],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .flatten();
    }
    Ok(plan)
}

/// What FM made of one change: the person's job, team level, and weekly wage afterwards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmStaffChangeDone {
    pub(super) uid: i64,
    pub(super) name: Option<String>,
    pub(super) kind: FmStaffChangeKind,
    pub(super) job_label: Option<&'static str>,
    pub(super) team_level: Option<String>,
    pub(super) weekly_wage_gbp: Option<i64>,
}

/// A change FM did not make, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct FmStaffChangeFailure {
    pub(super) uid: i64,
    pub(super) name: Option<String>,
    pub(super) kind: FmStaffChangeKind,
    pub(super) reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct FmStaffChangeReport {
    pub(super) done: Vec<FmStaffChangeDone>,
    pub(super) failed: Vec<FmStaffChangeFailure>,
    /// The error that stopped the request early, when one did.
    pub(super) error: Option<String>,
}

/// Makes the chosen changes, those of the current plan whose UIDs `staff_uids` lists, and records
/// each person's contract as FM reports it in the current snapshot.
pub(super) fn apply_fm_staff_changes<F>(
    db: &Db,
    expected_save_context_token: &str,
    expected_snapshot_context_token: &str,
    staff_uids: &[i64],
    request: F,
) -> Result<FmStaffChangeReport, String>
where
    F: FnOnce(&str, u32, &[StaffChangeItem]) -> Result<StaffChangeResponse, String>,
{
    let _guard = boost_gate::acquire_boost_gate()?;
    let (context, changes) = {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let context = capture_fm_action_context(&conn, LOAD_DATA_FIRST)?;
        let changes = plan_fm_staff_changes(
            &conn,
            expected_save_context_token,
            expected_snapshot_context_token,
        )?
        .into_iter()
        .filter(|planned| staff_uids.contains(&planned.uid))
        .collect::<Vec<_>>();
        (context, changes)
    };
    if changes.is_empty() {
        return Err("None of the chosen staff needs a change".to_string());
    }
    if changes.len() > MAX_STAFF_CHANGES {
        return Err(format!(
            "Change at most {MAX_STAFF_CHANGES} staff at a time"
        ));
    }
    let items = changes
        .iter()
        .map(|planned| {
            Ok(StaffChangeItem {
                staff_uid: u32::try_from(planned.uid).map_err(|_| {
                    format!(
                        "{} has an invalid staff identity",
                        planned.name.as_deref().unwrap_or("A staff member")
                    )
                })?,
                kind: planned.kind.as_str(),
                job_type: planned.target.as_ref().map(|target| target.job_type),
                target_level: planned.target.as_ref().map(|target| target.level),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let response = request(&context.source_request_id, context.club_uid, &items)?;
    if response.changes.len() != changes.len() {
        return Err(
            "The bridge reported a different number of staff changes than were sent".to_string(),
        );
    }

    let mut report = FmStaffChangeReport {
        error: response.error,
        ..FmStaffChangeReport::default()
    };
    let mut conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let tx = conn.transaction().map_err(|error| error.to_string())?;
    for (planned, outcome) in changes.iter().zip(response.changes) {
        if outcome.outcome != "not-found" {
            record_contract(&tx, context.snapshot_id, planned.uid, &outcome)?;
        }
        match outcome.outcome.as_str() {
            "sacked" | "hired" | "changed" => report.done.push(FmStaffChangeDone {
                uid: planned.uid,
                name: planned.name.clone(),
                kind: planned.kind,
                job_label: outcome.job_id.and_then(fm_job_label),
                team_level: outcome.team_level.clone(),
                weekly_wage_gbp: outcome.weekly_wage_gbp,
            }),
            other => report.failed.push(FmStaffChangeFailure {
                uid: planned.uid,
                name: planned.name.clone(),
                kind: planned.kind,
                reason: failure_reason(other, planned, &outcome),
            }),
        }
    }
    tx.commit().map_err(|error| error.to_string())?;
    Ok(report)
}

/// Writes the contract FM reports for a person into the snapshot; no contract clears it.
fn record_contract(
    conn: &Connection,
    snapshot_id: i64,
    uid: i64,
    outcome: &StaffChangeOutcome,
) -> Result<(), String> {
    conn.execute(
        "UPDATE staff SET
             club = ?3, club_uid = ?4, division = ?5, job_id = ?6, weekly_wage_gbp = ?7,
             contract_expiry_year = ?8, contract_expiry_day_of_year = ?9,
             team_type = ?10, team_level = ?11
         WHERE snapshot_id = ?1 AND uid = ?2",
        params![
            snapshot_id,
            uid,
            outcome.club,
            outcome.club_uid,
            outcome.division,
            outcome.job_id,
            outcome.weekly_wage_gbp,
            outcome.contract_expiry_year,
            outcome.contract_expiry_day_of_year,
            outcome.team_type,
            outcome.team_level,
        ],
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn failure_reason(outcome: &str, planned: &FmStaffChange, reported: &StaffChangeOutcome) -> String {
    match outcome {
        "left-unemployed" => {
            "FM ended the contract but did not hire them again; re-hire them with FM's editor"
                .to_string()
        }
        "not-found" => "FM could not find them; Load Data and try again".to_string(),
        "not-at-club" => "They no longer work for the club in FM".to_string(),
        "no-target-team" => format!(
            "The club has no {} team in FM",
            planned
                .target
                .as_ref()
                .map_or("matching", |target| target.level)
        ),
        "not-confirmed" => match (planned.kind, reported.job_id.and_then(fm_job_label)) {
            (FmStaffChangeKind::Sack, _) => "FM did not end the contract".to_string(),
            (_, Some(job)) if reported.club_uid.is_some() => {
                format!("FM did not confirm the new job; they hold {job} now")
            }
            _ => "FM did not confirm the hire; a single-holder job may still be taken".to_string(),
        },
        _ => "FM did not report an outcome".to_string(),
    }
}
