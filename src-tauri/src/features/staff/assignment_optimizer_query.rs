use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension};

use crate::features::managed_club::service::selected_club;

use super::assignment_optimizer::{
    allocate_staff_assignments, preferred_job_classification, StaffAssignmentAllocation,
    StaffAssignmentCandidate, StaffAssignmentClassification, StaffAssignmentEvidence,
    StaffAssignmentScoreSet, StaffAssignmentSlot,
};
use super::assignment_targets::read_nonzero_targets_without_initializing_teams;

const MAX_STAFF_ASSIGNMENT_SLOTS: usize = 1_108;
const SCORE_ROLE_IDS: [&str; 21] = [
    "manager",
    "assistant_manager",
    "coach_attacking_technical",
    "coach_attacking_tactical",
    "coach_defending_technical",
    "coach_defending_tactical",
    "coach_possession_technical",
    "coach_possession_tactical",
    "coach_fitness",
    "coach_goalkeeping",
    "set_piece_coach",
    "head_performance_analyst",
    "performance_analyst",
    "head_of_youth_development",
    "director_of_football",
    "technical_director",
    "loan_manager",
    "recruitment_analyst",
    "scout",
    "physio",
    "sports_scientist",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StaffAssignmentOptimizationState {
    StaleContext,
    NoCurrentSnapshot,
    NoManagedClub,
    NoShortlist,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaffAssignmentResultSlot {
    pub(super) scope_display_name: String,
    pub(super) slot: StaffAssignmentSlot,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaffAssignmentOptimization {
    pub(super) state: StaffAssignmentOptimizationState,
    pub(super) save_id: i64,
    pub(super) save_context_token: String,
    pub(super) snapshot_id: Option<i64>,
    pub(super) snapshot_context_token: Option<String>,
    pub(super) joined_candidate_count: i64,
    pub(super) configured_slot_count: i64,
    pub(super) unsupported_preferred_job_count: i64,
    pub(super) slots: Vec<StaffAssignmentResultSlot>,
    pub(super) evidence: Vec<StaffAssignmentEvidence>,
}

pub(super) fn optimize_staff_assignments(
    conn: &Connection,
    expected_save_context_token: &str,
    expected_snapshot_context_token: &str,
) -> Result<StaffAssignmentOptimization, String> {
    let Some((save_id, save_context_token)) = conn
        .query_row(
            "SELECT id, context_token FROM saves WHERE is_active = 1 LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?
    else {
        return Err("No active save is available".to_string());
    };
    if save_context_token != expected_save_context_token {
        return Ok(result(
            StaffAssignmentOptimizationState::StaleContext,
            save_id,
            save_context_token,
            None,
            None,
        ));
    }

    let snapshot: Option<(i64, String)> = conn
        .query_row(
            "SELECT id, context_token FROM snapshots
             WHERE save_id = ?1 AND is_current = 1 LIMIT 1",
            [save_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| error.to_string())?;
    let Some((snapshot_id, snapshot_context_token)) = snapshot else {
        return Ok(result(
            StaffAssignmentOptimizationState::NoCurrentSnapshot,
            save_id,
            save_context_token,
            None,
            None,
        ));
    };
    if snapshot_context_token != expected_snapshot_context_token {
        return Ok(result(
            StaffAssignmentOptimizationState::StaleContext,
            save_id,
            save_context_token,
            Some(snapshot_id),
            Some(snapshot_context_token),
        ));
    }

    let Some(managed_club) = selected_club(conn, save_id)? else {
        return Ok(result(
            StaffAssignmentOptimizationState::NoManagedClub,
            save_id,
            save_context_token,
            Some(snapshot_id),
            Some(snapshot_context_token),
        ));
    };
    let has_shortlist: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM staff_shortlist_entries WHERE save_id = ?1)",
            [save_id],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())?;
    if !has_shortlist {
        return Ok(result(
            StaffAssignmentOptimizationState::NoShortlist,
            save_id,
            save_context_token,
            Some(snapshot_id),
            Some(snapshot_context_token),
        ));
    }

    let targets = read_nonzero_targets_without_initializing_teams(conn, save_id)?;
    let scope_display_names = load_scope_display_names(conn, save_id)?;
    let configured_slot_count = targets.iter().map(|target| target.slot_count).sum::<i64>();
    let configured_slot_count_usize = usize::try_from(configured_slot_count)
        .map_err(|_| "Staff assignment targets exceed the supported slot limit".to_string())?;
    if configured_slot_count_usize > MAX_STAFF_ASSIGNMENT_SLOTS {
        return Err("Staff assignment targets exceed the supported slot limit".to_string());
    }
    let candidates = load_candidates(conn, save_id, snapshot_id, &managed_club)?;
    let unsupported_preferred_job_count = i64::try_from(
        candidates
            .iter()
            .filter(|candidate| preferred_job_classification(&candidate.preferred_job).is_none())
            .count(),
    )
    .map_err(|_| "Staff shortlist candidate count is too large".to_string())?;
    let joined_candidate_count = i64::try_from(candidates.len())
        .map_err(|_| "Staff shortlist candidate count is too large".to_string())?;
    let StaffAssignmentAllocation { slots, evidence } =
        allocate_staff_assignments(&targets, &candidates);
    if slots.len() > MAX_STAFF_ASSIGNMENT_SLOTS {
        return Err("Staff assignment result exceeds the supported slot limit".to_string());
    }

    let slots = slots
        .into_iter()
        .map(|slot| {
            let scope = match &slot {
                StaffAssignmentSlot::Recommendation(recommendation) => &recommendation.scope,
                StaffAssignmentSlot::Vacancy(vacancy) => &vacancy.scope,
            };
            let scope_display_name = if scope == "club" {
                "Club".to_string()
            } else {
                scope_display_names.get(scope).cloned().ok_or_else(|| {
                    "Staff assignment targets reference an unavailable Planner team".to_string()
                })?
            };
            Ok(StaffAssignmentResultSlot {
                scope_display_name,
                slot,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(StaffAssignmentOptimization {
        state: StaffAssignmentOptimizationState::Ready,
        save_id,
        save_context_token,
        snapshot_id: Some(snapshot_id),
        snapshot_context_token: Some(snapshot_context_token),
        joined_candidate_count,
        configured_slot_count,
        unsupported_preferred_job_count,
        slots,
        evidence,
    })
}

fn result(
    state: StaffAssignmentOptimizationState,
    save_id: i64,
    save_context_token: String,
    snapshot_id: Option<i64>,
    snapshot_context_token: Option<String>,
) -> StaffAssignmentOptimization {
    StaffAssignmentOptimization {
        state,
        save_id,
        save_context_token,
        snapshot_id,
        snapshot_context_token,
        joined_candidate_count: 0,
        configured_slot_count: 0,
        unsupported_preferred_job_count: 0,
        slots: Vec::new(),
        evidence: Vec::new(),
    }
}

fn load_scope_display_names(
    conn: &Connection,
    save_id: i64,
) -> Result<HashMap<String, String>, String> {
    conn.prepare("SELECT team, display_name FROM planner_teams WHERE save_id = ?1")
        .map_err(|error| error.to_string())?
        .query_map([save_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|error| error.to_string())?
        .collect::<Result<HashMap<_, _>, _>>()
        .map_err(|error| error.to_string())
}

pub(super) fn load_candidates(
    conn: &Connection,
    save_id: i64,
    snapshot_id: i64,
    managed_club: &str,
) -> Result<Vec<StaffAssignmentCandidate>, String> {
    let mut statement = conn
        .prepare(
            "SELECT staff.uid, staff.name, staff.club, shortlist.preferred_job,
                    scores.role_id, scores.score
             FROM staff
             INNER JOIN staff_shortlist_entries shortlist
                 ON shortlist.save_id = ?1 AND shortlist.staff_uid = staff.uid
             LEFT JOIN staff_role_scores scores
                 ON scores.snapshot_id = staff.snapshot_id
                AND scores.uid = staff.uid
                AND scores.role_id IN (?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12,
                                       ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22,
                                       ?23)
             WHERE staff.snapshot_id = ?2
             ORDER BY staff.uid ASC, scores.role_id ASC",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(
            params![
                save_id,
                snapshot_id,
                SCORE_ROLE_IDS[0],
                SCORE_ROLE_IDS[1],
                SCORE_ROLE_IDS[2],
                SCORE_ROLE_IDS[3],
                SCORE_ROLE_IDS[4],
                SCORE_ROLE_IDS[5],
                SCORE_ROLE_IDS[6],
                SCORE_ROLE_IDS[7],
                SCORE_ROLE_IDS[8],
                SCORE_ROLE_IDS[9],
                SCORE_ROLE_IDS[10],
                SCORE_ROLE_IDS[11],
                SCORE_ROLE_IDS[12],
                SCORE_ROLE_IDS[13],
                SCORE_ROLE_IDS[14],
                SCORE_ROLE_IDS[15],
                SCORE_ROLE_IDS[16],
                SCORE_ROLE_IDS[17],
                SCORE_ROLE_IDS[18],
                SCORE_ROLE_IDS[19],
                SCORE_ROLE_IDS[20],
            ],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<i64>>(5)?,
                ))
            },
        )
        .map_err(|error| error.to_string())?;
    let mut candidates = Vec::new();
    for row in rows {
        let (uid, name, club, preferred_job, role_id, score) =
            row.map_err(|error| error.to_string())?;
        if candidates
            .last()
            .map_or(true, |candidate: &StaffAssignmentCandidate| {
                candidate.uid != uid
            })
        {
            candidates.push(StaffAssignmentCandidate {
                uid,
                name,
                preferred_job,
                classification: if club.as_deref() == Some(managed_club) {
                    StaffAssignmentClassification::CurrentStaff
                } else {
                    StaffAssignmentClassification::Recruitment
                },
                scores: StaffAssignmentScoreSet::default(),
            });
        }
        if let Some(role_id) = role_id {
            set_score(
                &mut candidates
                    .last_mut()
                    .expect("candidate was inserted for every joined row")
                    .scores,
                &role_id,
                score.and_then(|value| u8::try_from(value).ok()),
            );
        }
    }
    Ok(candidates)
}

fn set_score(scores: &mut StaffAssignmentScoreSet, role_id: &str, score: Option<u8>) {
    match role_id {
        "manager" => scores.manager = score,
        "assistant_manager" => scores.assistant_manager = score,
        "coach_attacking_technical" => scores.coach_attacking_technical = score,
        "coach_attacking_tactical" => scores.coach_attacking_tactical = score,
        "coach_defending_technical" => scores.coach_defending_technical = score,
        "coach_defending_tactical" => scores.coach_defending_tactical = score,
        "coach_possession_technical" => scores.coach_possession_technical = score,
        "coach_possession_tactical" => scores.coach_possession_tactical = score,
        "coach_fitness" => scores.coach_fitness = score,
        "coach_goalkeeping" => scores.coach_goalkeeping = score,
        "set_piece_coach" => scores.set_piece_coach = score,
        "head_performance_analyst" => scores.head_performance_analyst = score,
        "performance_analyst" => scores.performance_analyst = score,
        "head_of_youth_development" => scores.head_of_youth_development = score,
        "director_of_football" => scores.director_of_football = score,
        "technical_director" => scores.technical_director = score,
        "loan_manager" => scores.loan_manager = score,
        "recruitment_analyst" => scores.recruitment_analyst = score,
        "scout" => scores.scout = score,
        "physio" => scores.physio = score,
        "sports_scientist" => scores.sports_scientist = score,
        _ => {}
    }
}
