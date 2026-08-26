use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::Db;
use crate::features::memory_read::service::{
    request_staff_boost_from_local_app_data, DumpWaitConfig, StaffBoostResult,
};
use crate::features::player::boost_gate;

use super::assignment_optimizer::{
    CoachDiscipline, StaffAssignmentClassification, StaffAssignmentEvidence, StaffAssignmentSlot,
};
use super::assignment_optimizer_query::{
    self, StaffAssignmentOptimization, StaffAssignmentOptimizationState, StaffAssignmentResultSlot,
};
use super::assignment_targets::{
    self, StaffAssignmentTarget, StaffAssignmentTargetInput, StaffAssignmentTargetTeam,
    StaffAssignmentTargets,
};
use super::filter::{self, FilterAst, FilterRule};
use super::query::{
    self, SortDir, SortField, StaffDetail, StaffPage, StaffPageState, StaffRoleScore, StaffScope,
    StaffShortlistMetadata, StaffSummary,
};
use super::service::{
    self, PreparedStaffBoost, StaffBoostBatchContext, StaffBoostError, VerifiedStaffBoost,
};

#[derive(Deserialize)]
pub struct StaffFilterRuleInput {
    pub field: String,
    pub op: String,
    pub value: serde_json::Value,
}
impl TryFrom<StaffFilterRuleInput> for FilterRule {
    type Error = String;
    fn try_from(value: StaffFilterRuleInput) -> Result<Self, Self::Error> {
        Ok(Self {
            field: value.field,
            op: value.op,
            value: filter::filter_value_from_json(value.value)?,
        })
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffSummaryDto {
    pub uid: i64,
    pub name: Option<String>,
    pub age: Option<i64>,
    pub birth_year: Option<i64>,
    pub birth_day_of_year: Option<i64>,
    pub nationalities: Vec<String>,
    pub nation_uid: Option<i64>,
    pub gender: String,
    pub club: Option<String>,
    pub division: Option<String>,
    pub ca: i64,
    pub pa: i64,
    pub job_id: Option<i64>,
    pub weekly_wage_gbp: Option<i64>,
    pub contract_expiry_year: Option<i64>,
    pub contract_expiry_day_of_year: Option<i64>,
    pub dynamic_values: BTreeMap<String, Option<i64>>,
    pub shortlist: Option<StaffShortlistMetadataDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffShortlistMetadataDto {
    pub preferred_job: String,
    pub club_job: String,
    pub coaching_qualifications: String,
}

impl From<StaffShortlistMetadata> for StaffShortlistMetadataDto {
    fn from(value: StaffShortlistMetadata) -> Self {
        Self {
            preferred_job: value.preferred_job,
            club_job: value.club_job,
            coaching_qualifications: value.coaching_qualifications,
        }
    }
}
impl From<StaffSummary> for StaffSummaryDto {
    fn from(row: StaffSummary) -> Self {
        Self {
            uid: row.uid,
            name: row.name,
            age: row.age,
            birth_year: row.birth_year,
            birth_day_of_year: row.birth_day_of_year,
            nationalities: row.nationalities,
            nation_uid: row.nation_uid,
            gender: row.gender,
            club: row.club,
            division: row.division,
            ca: row.ca,
            pa: row.pa,
            job_id: row.job_id,
            weekly_wage_gbp: row.weekly_wage_gbp,
            contract_expiry_year: row.contract_expiry_year,
            contract_expiry_day_of_year: row.contract_expiry_day_of_year,
            dynamic_values: row.dynamic_values,
            shortlist: row.shortlist.map(StaffShortlistMetadataDto::from),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffPageDto {
    pub state: &'static str,
    pub staff: Vec<StaffSummaryDto>,
    pub total: i64,
    pub preferred_job_options: Vec<String>,
}
impl From<StaffPage> for StaffPageDto {
    fn from(page: StaffPage) -> Self {
        Self {
            state: match page.state {
                StaffPageState::Ready => "ready",
                StaffPageState::NoCurrentSnapshot => "no_current_snapshot",
                StaffPageState::NoManagedClub => "no_managed_club",
                StaffPageState::NoShortlist => "no_shortlist",
            },
            staff: page.staff.into_iter().map(StaffSummaryDto::from).collect(),
            total: page.total,
            preferred_job_options: page.preferred_job_options,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffRoleScoreDto {
    pub role_id: String,
    pub display_name: String,
    pub score: Option<i64>,
}

impl From<StaffRoleScore> for StaffRoleScoreDto {
    fn from(role: StaffRoleScore) -> Self {
        Self {
            role_id: role.role_id,
            display_name: role.display_name,
            score: role.score,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffDetailDto {
    pub uid: i64,
    pub name: Option<String>,
    pub age: Option<i64>,
    pub birth_year: Option<i64>,
    pub birth_day_of_year: Option<i64>,
    pub nationalities: Vec<String>,
    pub nation_uid: Option<i64>,
    pub gender: String,
    pub club: Option<String>,
    pub division: Option<String>,
    pub ca: i64,
    pub pa: i64,
    pub job_id: Option<i64>,
    pub weekly_wage_gbp: Option<i64>,
    pub contract_expiry_year: Option<i64>,
    pub contract_expiry_day_of_year: Option<i64>,
    pub attributes: BTreeMap<String, Option<i64>>,
    pub hidden_information_revealed: bool,
    pub role_scores: Vec<StaffRoleScoreDto>,
}

impl From<StaffDetail> for StaffDetailDto {
    fn from(staff: StaffDetail) -> Self {
        Self {
            uid: staff.uid,
            name: staff.name,
            age: staff.age,
            birth_year: staff.birth_year,
            birth_day_of_year: staff.birth_day_of_year,
            nationalities: staff.nationalities,
            nation_uid: staff.nation_uid,
            gender: staff.gender,
            club: staff.club,
            division: staff.division,
            ca: staff.ca,
            pa: staff.pa,
            job_id: staff.job_id,
            weekly_wage_gbp: staff.weekly_wage_gbp,
            contract_expiry_year: staff.contract_expiry_year,
            contract_expiry_day_of_year: staff.contract_expiry_day_of_year,
            attributes: staff.attributes,
            hidden_information_revealed: staff.hidden_information_revealed,
            role_scores: staff
                .role_scores
                .into_iter()
                .map(StaffRoleScoreDto::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffAssignmentTargetDto {
    pub scope: String,
    pub job_id: String,
    pub job_label: String,
    pub slot_count: i64,
}

impl From<StaffAssignmentTarget> for StaffAssignmentTargetDto {
    fn from(target: StaffAssignmentTarget) -> Self {
        Self {
            scope: target.scope,
            job_id: target.job_id,
            job_label: target.job_label,
            slot_count: target.slot_count,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffAssignmentTargetTeamDto {
    pub team: String,
    pub display_name: String,
}

impl From<StaffAssignmentTargetTeam> for StaffAssignmentTargetTeamDto {
    fn from(team: StaffAssignmentTargetTeam) -> Self {
        Self {
            team: team.team,
            display_name: team.display_name,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffAssignmentTargetsDto {
    pub teams: Vec<StaffAssignmentTargetTeamDto>,
    pub targets: Vec<StaffAssignmentTargetDto>,
}

impl From<StaffAssignmentTargets> for StaffAssignmentTargetsDto {
    fn from(targets: StaffAssignmentTargets) -> Self {
        Self {
            teams: targets.teams.into_iter().map(Into::into).collect(),
            targets: targets.targets.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffAssignmentTargetInputDto {
    pub scope: String,
    pub job_id: String,
    pub slot_count: i64,
}

impl From<StaffAssignmentTargetInputDto> for StaffAssignmentTargetInput {
    fn from(target: StaffAssignmentTargetInputDto) -> Self {
        Self {
            scope: target.scope,
            job_id: target.job_id,
            slot_count: target.slot_count,
        }
    }
}

#[tauri::command]
pub fn get_staff_assignment_targets(
    expected_save_context_token: String,
    db: State<'_, Db>,
) -> Result<StaffAssignmentTargetsDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    assignment_targets::get_targets(&conn, &expected_save_context_token).map(Into::into)
}

#[tauri::command]
pub fn save_staff_assignment_targets(
    expected_save_context_token: String,
    targets: Vec<StaffAssignmentTargetInputDto>,
    db: State<'_, Db>,
) -> Result<StaffAssignmentTargetsDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let inputs = targets.into_iter().map(Into::into).collect::<Vec<_>>();
    assignment_targets::save_targets(&conn, &expected_save_context_token, &inputs).map(Into::into)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffAssignmentEvidenceDto {
    pub job_id: String,
    pub joined_candidate_count: usize,
    pub eligible_score_count: usize,
    pub unavailable_score_count: usize,
}

impl From<StaffAssignmentEvidence> for StaffAssignmentEvidenceDto {
    fn from(evidence: StaffAssignmentEvidence) -> Self {
        Self {
            job_id: evidence.job_id,
            joined_candidate_count: evidence.joined_candidate_count,
            eligible_score_count: evidence.eligible_score_count,
            unavailable_score_count: evidence.unavailable_score_count,
        }
    }
}

#[derive(Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum StaffAssignmentSlotDto {
    Recommendation {
        scope: String,
        scope_display_name: String,
        job_id: String,
        job_label: String,
        slot_number: i64,
        uid: i64,
        name: String,
        preferred_job: String,
        classification: &'static str,
        score: u8,
        coach_discipline: Option<&'static str>,
    },
    Vacancy {
        scope: String,
        scope_display_name: String,
        job_id: String,
        job_label: String,
        slot_number: i64,
        evidence: StaffAssignmentEvidenceDto,
    },
}

impl From<StaffAssignmentResultSlot> for StaffAssignmentSlotDto {
    fn from(result_slot: StaffAssignmentResultSlot) -> Self {
        let StaffAssignmentResultSlot {
            scope_display_name,
            slot,
        } = result_slot;
        match slot {
            StaffAssignmentSlot::Recommendation(recommendation) => Self::Recommendation {
                scope: recommendation.scope,
                scope_display_name,
                job_id: recommendation.job_id,
                job_label: recommendation.job_label,
                slot_number: recommendation.slot_number,
                uid: recommendation.uid,
                name: recommendation.name,
                preferred_job: recommendation.preferred_job,
                classification: classification_name(recommendation.classification),
                score: recommendation.score,
                coach_discipline: recommendation.coach_discipline.map(coach_discipline_name),
            },
            StaffAssignmentSlot::Vacancy(vacancy) => Self::Vacancy {
                scope: vacancy.scope,
                scope_display_name,
                job_id: vacancy.job_id,
                job_label: vacancy.job_label,
                slot_number: vacancy.slot_number,
                evidence: vacancy.evidence.into(),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffAssignmentOptimizationDto {
    pub state: &'static str,
    pub save_id: i64,
    pub save_context_token: String,
    pub snapshot_id: Option<i64>,
    pub snapshot_context_token: Option<String>,
    pub joined_candidate_count: i64,
    pub configured_slot_count: i64,
    pub unsupported_preferred_job_count: i64,
    pub slots: Vec<StaffAssignmentSlotDto>,
    pub evidence: Vec<StaffAssignmentEvidenceDto>,
}

impl From<StaffAssignmentOptimization> for StaffAssignmentOptimizationDto {
    fn from(result: StaffAssignmentOptimization) -> Self {
        Self {
            state: match result.state {
                StaffAssignmentOptimizationState::StaleContext => "stale_context",
                StaffAssignmentOptimizationState::NoCurrentSnapshot => "no_current_snapshot",
                StaffAssignmentOptimizationState::NoManagedClub => "no_managed_club",
                StaffAssignmentOptimizationState::NoShortlist => "no_shortlist",
                StaffAssignmentOptimizationState::Ready => "ready",
            },
            save_id: result.save_id,
            save_context_token: result.save_context_token,
            snapshot_id: result.snapshot_id,
            snapshot_context_token: result.snapshot_context_token,
            joined_candidate_count: result.joined_candidate_count,
            configured_slot_count: result.configured_slot_count,
            unsupported_preferred_job_count: result.unsupported_preferred_job_count,
            slots: result.slots.into_iter().map(Into::into).collect(),
            evidence: result.evidence.into_iter().map(Into::into).collect(),
        }
    }
}

#[tauri::command]
pub fn optimize_staff_assignments(
    expected_save_context_token: String,
    expected_snapshot_context_token: String,
    db: State<'_, Db>,
) -> Result<StaffAssignmentOptimizationDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    assignment_optimizer_query::optimize_staff_assignments(
        &conn,
        &expected_save_context_token,
        &expected_snapshot_context_token,
    )
    .map(Into::into)
}

fn classification_name(classification: StaffAssignmentClassification) -> &'static str {
    match classification {
        StaffAssignmentClassification::CurrentStaff => "current_staff",
        StaffAssignmentClassification::Recruitment => "recruitment",
    }
}

fn coach_discipline_name(discipline: CoachDiscipline) -> &'static str {
    match discipline {
        CoachDiscipline::AttackingTechnical => "attacking_technical",
        CoachDiscipline::AttackingTactical => "attacking_tactical",
        CoachDiscipline::DefendingTechnical => "defending_technical",
        CoachDiscipline::DefendingTactical => "defending_tactical",
        CoachDiscipline::PossessionTechnical => "possession_technical",
        CoachDiscipline::PossessionTactical => "possession_tactical",
    }
}

fn parse_filters(
    filters: Option<Vec<StaffFilterRuleInput>>,
    combine: Option<&str>,
) -> Result<Option<FilterAst>, String> {
    filters
        .map(|rules| {
            rules
                .into_iter()
                .map(FilterRule::try_from)
                .collect::<Result<Vec<_>, _>>()
                .and_then(|rules| filter::parse_filter_ast(rules, combine))
        })
        .transpose()
}

#[allow(clippy::too_many_arguments)]
fn run(
    scope: StaffScope,
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    filters: Option<Vec<StaffFilterRuleInput>>,
    filter_combine: Option<String>,
    requested_fields: Option<Vec<String>>,
    db: State<'_, Db>,
) -> Result<StaffPageDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let sort = sort_by
        .as_deref()
        .map(SortField::parse)
        .transpose()?
        .unwrap_or(SortField::DEFAULT);
    let direction = sort_dir
        .as_deref()
        .map(SortDir::parse)
        .transpose()?
        .unwrap_or(SortDir::DEFAULT);
    let filters = parse_filters(filters, filter_combine.as_deref())?;
    query::list_staff(
        &conn,
        scope,
        offset.unwrap_or(0) as usize,
        limit.unwrap_or(query::DEFAULT_PAGE_LIMIT as u32) as usize,
        sort,
        direction,
        filters.as_ref(),
        &requested_fields.unwrap_or_default(),
    )
    .map(StaffPageDto::from)
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn search_staff(
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    filters: Option<Vec<StaffFilterRuleInput>>,
    filter_combine: Option<String>,
    requested_fields: Option<Vec<String>>,
    db: State<'_, Db>,
) -> Result<StaffPageDto, String> {
    run(
        StaffScope::Search,
        offset,
        limit,
        sort_by,
        sort_dir,
        filters,
        filter_combine,
        requested_fields,
        db,
    )
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn list_my_staff(
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    requested_fields: Option<Vec<String>>,
    db: State<'_, Db>,
) -> Result<StaffPageDto, String> {
    run(
        StaffScope::MyStaff,
        offset,
        limit,
        sort_by,
        sort_dir,
        None,
        None,
        requested_fields,
        db,
    )
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub fn list_staff_shortlist(
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    preferred_job: Option<String>,
    unemployed_only: Option<bool>,
    requested_fields: Option<Vec<String>>,
    db: State<'_, Db>,
) -> Result<StaffPageDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let sort = sort_by
        .as_deref()
        .map(SortField::parse)
        .transpose()?
        .unwrap_or(SortField::DEFAULT);
    let direction = sort_dir
        .as_deref()
        .map(SortDir::parse)
        .transpose()?
        .unwrap_or(SortDir::DEFAULT);
    query::list_staff_shortlist(
        &conn,
        offset.unwrap_or(0) as usize,
        limit.unwrap_or(query::DEFAULT_PAGE_LIMIT as u32) as usize,
        sort,
        direction,
        preferred_job.as_deref(),
        unemployed_only.unwrap_or(false),
        &requested_fields.unwrap_or_default(),
    )
    .map(StaffPageDto::from)
}

#[tauri::command]
pub fn get_staff(uid: i64, db: State<'_, Db>) -> Result<Option<StaffDetailDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    query::get_staff(&conn, uid).map(|staff| staff.map(StaffDetailDto::from))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(Debug)]
pub struct StaffBoostResultDto {
    pub snapshot_id: i64,
    pub operation: String,
    pub previous_current_ability: i64,
    pub current_ability: i64,
    pub potential_ability: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MyStaffBoostResultDto {
    pub updated: usize,
    pub skipped: usize,
    pub failed: usize,
    pub recovery_required: bool,
    pub recovery_message: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MyStaffBoostProgressDto {
    pub processed: usize,
    pub total: usize,
    pub updated: usize,
    pub skipped: usize,
    pub failed: usize,
}

impl From<VerifiedStaffBoost> for StaffBoostResultDto {
    fn from(result: VerifiedStaffBoost) -> Self {
        Self {
            snapshot_id: result.snapshot_id,
            operation: result.operation,
            previous_current_ability: result.previous_current_ability,
            current_ability: result.current_ability,
            potential_ability: result.potential_ability,
        }
    }
}

#[tauri::command]
pub fn boost_staff_current_ability(
    uid: i64,
    db: State<'_, Db>,
) -> Result<StaffBoostResultDto, StaffBoostError> {
    execute_staff_boost_with(uid, db.inner(), request_local_staff_boost)
}

#[tauri::command]
pub fn boost_my_staff_current_ability(
    db: State<'_, Db>,
    on_progress: tauri::ipc::Channel<MyStaffBoostProgressDto>,
) -> Result<MyStaffBoostResultDto, StaffBoostError> {
    execute_my_staff_boost_with_progress(db.inner(), request_local_staff_boost, move |progress| {
        match on_progress.send(progress) {
            Ok(()) => true,
            Err(error) => {
                log::debug!("My Staff boost progress delivery failed: {error}");
                false
            }
        }
    })
}

#[cfg(test)]
fn execute_my_staff_boost_with<F>(
    db: &Db,
    request_bridge_boost: F,
) -> Result<MyStaffBoostResultDto, StaffBoostError>
where
    F: FnMut(&PreparedStaffBoost) -> Result<StaffBoostResult, StaffBoostError>,
{
    execute_my_staff_boost_with_progress(db, request_bridge_boost, |_| true)
}

fn execute_my_staff_boost_with_progress<F, R>(
    db: &Db,
    mut request_bridge_boost: F,
    mut on_progress: R,
) -> Result<MyStaffBoostResultDto, StaffBoostError>
where
    F: FnMut(&PreparedStaffBoost) -> Result<StaffBoostResult, StaffBoostError>,
    R: FnMut(MyStaffBoostProgressDto) -> bool,
{
    let _boost_guard =
        boost_gate::acquire_boost_gate().map_err(|message| StaffBoostError::Bridge {
            kind: "inProgress".to_string(),
            message,
        })?;
    let (context, staff_uids) = capture_my_staff_boost_cohort(db)?;
    let mut result = MyStaffBoostResultDto {
        updated: 0,
        skipped: 0,
        failed: 0,
        recovery_required: false,
        recovery_message: None,
    };
    let total = staff_uids.len();
    report_my_staff_boost_progress(&mut on_progress, &result, total);

    for uid in staff_uids {
        let prepared = match prepare_my_staff_boost(db, uid, &context) {
            Ok(prepared) => prepared,
            Err(StaffBoostError::Eligibility { ref kind, .. })
                if kind == "currentAbilityAtLimit" =>
            {
                result.skipped += 1;
                report_my_staff_boost_progress(&mut on_progress, &result, total);
                continue;
            }
            Err(error) => return my_staff_recovery_result(db, &context, result, error),
        };

        match request_bridge_boost(&prepared) {
            Ok(bridge_result) => {
                let reconciled = {
                    let mut conn = db.0.lock().map_err(|_| StaffBoostError::SnapshotSync {
                        message:
                            "FM changed, but the local database is unavailable. Load Data again."
                                .to_string(),
                    })?;
                    service::reconcile_verified_boost(&mut conn, &prepared, bridge_result)
                };
                match reconciled {
                    Ok(_) => result.updated += 1,
                    Err(error) => return my_staff_recovery_result(db, &context, result, error),
                }
            }
            Err(StaffBoostError::LiveValue { .. }) => {
                result.failed += 1;
            }
            Err(error @ StaffBoostError::Bridge { .. }) => return Err(error),
            Err(error) => return my_staff_recovery_result(db, &context, result, error),
        }
        report_my_staff_boost_progress(&mut on_progress, &result, total);
    }

    Ok(result)
}

fn capture_my_staff_boost_cohort(
    db: &Db,
) -> Result<(StaffBoostBatchContext, Vec<i64>), StaffBoostError> {
    let conn = db.0.lock().map_err(|_| StaffBoostError::Eligibility {
        kind: "databaseUnavailable".to_string(),
        message: "could not read the current snapshot for this My Staff boost".to_string(),
    })?;
    let context = service::capture_boost_context(&conn)?;
    let staff_uids = query::list_my_staff_uids(&conn, context.save_id, context.snapshot_id)
        .map_err(|message| StaffBoostError::Eligibility {
            kind: "database".to_string(),
            message,
        })?
        .ok_or_else(|| StaffBoostError::Eligibility {
            kind: "managedClubRequired".to_string(),
            message: "Select your managed club in Settings before boosting My Staff.".to_string(),
        })?;
    Ok((context, staff_uids))
}

fn prepare_my_staff_boost(
    db: &Db,
    uid: i64,
    expected_context: &StaffBoostBatchContext,
) -> Result<PreparedStaffBoost, StaffBoostError> {
    let conn = db.0.lock().map_err(|_| StaffBoostError::Eligibility {
        kind: "databaseUnavailable".to_string(),
        message: "could not read the current snapshot for this My Staff boost".to_string(),
    })?;
    if !service::boost_context_matches(&conn, expected_context)? {
        return Err(StaffBoostError::SnapshotSync {
            message:
                "The active save or snapshot changed. Load Data again before continuing the My Staff boost."
                    .to_string(),
        });
    }
    service::prepare_current_ability_boost(&conn, uid)
}

fn report_my_staff_boost_progress<R>(
    on_progress: &mut R,
    result: &MyStaffBoostResultDto,
    total: usize,
) where
    R: FnMut(MyStaffBoostProgressDto) -> bool,
{
    let _ = on_progress(MyStaffBoostProgressDto {
        processed: result.updated + result.skipped + result.failed,
        total,
        updated: result.updated,
        skipped: result.skipped,
        failed: result.failed,
    });
}

fn my_staff_recovery_result(
    db: &Db,
    context: &StaffBoostBatchContext,
    mut result: MyStaffBoostResultDto,
    error: StaffBoostError,
) -> Result<MyStaffBoostResultDto, StaffBoostError> {
    let conn = db.0.lock().map_err(|_| StaffBoostError::SnapshotSync {
        message:
            "FM may have changed, but the recovery requirement could not be saved. Load Data again."
                .to_string(),
    })?;
    service::require_load_data_for_boost(&conn, &context.recovery_context())?;
    result.recovery_required = true;
    result.recovery_message = Some(error.to_string());
    Ok(result)
}

fn execute_staff_boost_with<F>(
    uid: i64,
    db: &Db,
    request_bridge_boost: F,
) -> Result<StaffBoostResultDto, StaffBoostError>
where
    F: FnOnce(&PreparedStaffBoost) -> Result<StaffBoostResult, StaffBoostError>,
{
    let _boost_guard =
        boost_gate::acquire_boost_gate().map_err(|message| StaffBoostError::Bridge {
            kind: "inProgress".to_string(),
            message,
        })?;
    let prepared = {
        let conn = db.0.lock().map_err(|_| StaffBoostError::Eligibility {
            kind: "databaseUnavailable".to_string(),
            message: "could not read the current snapshot for this staff boost".to_string(),
        })?;
        service::prepare_current_ability_boost(&conn, uid)?
    };
    let result = match request_bridge_boost(&prepared) {
        Ok(result) => result,
        Err(error @ StaffBoostError::SnapshotSync { .. }) => {
            mark_recovery_required(db, &prepared)?;
            return Err(error);
        }
        Err(error) => return Err(error),
    };
    let reconciled = {
        let mut conn = db.0.lock().map_err(|_| StaffBoostError::SnapshotSync {
            message: "FM changed, but the local database is unavailable. Load Data again."
                .to_string(),
        })?;
        service::reconcile_verified_boost(&mut conn, &prepared, result)
    };
    match reconciled {
        Ok(result) => Ok(result.into()),
        Err(error) => {
            mark_recovery_required(db, &prepared)?;
            Err(error)
        }
    }
}

fn request_local_staff_boost(
    prepared: &PreparedStaffBoost,
) -> Result<StaffBoostResult, StaffBoostError> {
    request_staff_boost_from_local_app_data(
        &prepared.source_request_id,
        prepared.staff_uid,
        prepared.expected_current_ability as i32,
        prepared.expected_potential_ability as i32,
        DumpWaitConfig::default(),
    )
    .map_err(service::map_bridge_error)
}

fn mark_recovery_required(db: &Db, prepared: &PreparedStaffBoost) -> Result<(), StaffBoostError> {
    let conn = db.0.lock().map_err(|_| StaffBoostError::SnapshotSync {
        message:
            "FM may have changed, but the recovery requirement could not be saved. Load Data again."
                .to_string(),
    })?;
    service::require_load_data_for_boost(&conn, &prepared.context())
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use rusqlite::Connection;

    use super::*;
    use crate::db::migrations;
    use crate::features::memory_read::service::OPERATION_BOOST_STAFF_CURRENT_ABILITY;
    use crate::features::player::boost_gate::BOOST_TEST_GATE;
    use crate::features::player::service::prepare_current_ability_boost_for_test;
    use crate::features::snapshot::ingest::ingest_dump_file;

    const GOLDEN_FIXTURE: &str = include_str!("../memory_read/fixtures/golden_dump_v8.json");

    fn seeded_db(ca: i64, pa: i64) -> (tempfile::TempDir, Db) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut conn =
            Connection::open(temp_dir.path().join("staff-command.db")).expect("open test database");
        conn.pragma_update(None, "foreign_keys", true)
            .expect("foreign keys");
        migrations::apply(&conn).expect("migrations");
        let mut dump: serde_json::Value = serde_json::from_str(GOLDEN_FIXTURE).expect("fixture");
        dump["staff"][0]["ca"] = serde_json::Value::from(ca);
        dump["staff"][0]["pa"] = serde_json::Value::from(pa);
        let path = temp_dir.path().join("staff.json");
        std::fs::write(&path, dump.to_string()).expect("write fixture");
        let snapshot = ingest_dump_file(&mut conn, &path).expect("ingest fixture");
        conn.execute(
            "UPDATE snapshots SET bridge_source_request_id = 'scan-staff-command'
             WHERE id = ?1",
            [snapshot.id],
        )
        .expect("bind scan");
        (temp_dir, Db(Mutex::new(conn)))
    }

    fn verified_result(prepared: &PreparedStaffBoost, current_ability: i32) -> StaffBoostResult {
        StaffBoostResult {
            operation: OPERATION_BOOST_STAFF_CURRENT_ABILITY.to_string(),
            outcome: "verified".to_string(),
            rollback: "not-needed".to_string(),
            previous_current_ability: Some(prepared.expected_current_ability as i32),
            current_ability: Some(current_ability),
            potential_ability: Some(prepared.expected_potential_ability as i32),
        }
    }

    fn configure_managed_club(db: &Db) {
        let conn = db.0.lock().expect("lock database");
        let save_id: i64 = conn
            .query_row("SELECT id FROM saves WHERE is_active = 1", [], |row| {
                row.get(0)
            })
            .expect("read active save");
        conn.execute(
            "INSERT INTO managed_club_settings (save_id, club_name) VALUES (?1, 'Golden FC')",
            [save_id],
        )
        .expect("configure managed club");
    }

    fn clone_staff(db: &Db, uid: i64, ca: i64, pa: i64, club: &str) {
        db.0.lock()
            .expect("lock database")
            .execute(
                "INSERT INTO staff (
                    snapshot_id, uid, name, birth_year, birth_day_of_year, age,
                    nationalities_json, nation_uid, gender, ca, pa, staff_attributes_json,
                    job_id, weekly_wage_gbp, contract_expiry_year,
                    contract_expiry_day_of_year, club, division
                 )
                 SELECT snapshot_id, ?1, name, birth_year, birth_day_of_year, age,
                        nationalities_json, nation_uid, gender, ?2, ?3, staff_attributes_json,
                        job_id, weekly_wage_gbp, contract_expiry_year,
                        contract_expiry_day_of_year, ?4, division
                 FROM staff WHERE uid = 88",
                rusqlite::params![uid, ca, pa, club],
            )
            .expect("clone staff row");
    }
    #[test]
    fn rejects_non_scalar_filter_values() {
        assert!(filter::filter_value_from_json(serde_json::json!({"x":1})).is_err());
        assert!(matches!(
            filter::filter_value_from_json(serde_json::json!(4)).unwrap(),
            filter::FilterValue::Integer(4)
        ));
    }

    #[test]
    fn shortlist_page_dto_serializes_metadata_options_and_state_in_camel_case() {
        let dto = StaffPageDto::from(StaffPage {
            state: StaffPageState::Ready,
            staff: vec![StaffSummary {
                uid: 88,
                name: Some("Staff".to_string()),
                age: None,
                birth_year: None,
                birth_day_of_year: None,
                nationalities: vec![],
                nation_uid: None,
                gender: "unknown".to_string(),
                club: None,
                division: None,
                ca: 100,
                pa: 120,
                job_id: None,
                weekly_wage_gbp: None,
                contract_expiry_year: None,
                contract_expiry_day_of_year: None,
                dynamic_values: BTreeMap::new(),
                shortlist: Some(StaffShortlistMetadata {
                    preferred_job: "Physio".to_string(),
                    club_job: "-".to_string(),
                    coaching_qualifications: "Continental Pro".to_string(),
                }),
            }],
            total: 1,
            preferred_job_options: vec!["Physio".to_string()],
        });
        let value = serde_json::to_value(dto).expect("serialize shortlist page");
        assert_eq!(value["state"], "ready");
        assert_eq!(value["total"], 1);
        assert_eq!(value["preferredJobOptions"], serde_json::json!(["Physio"]));
        assert_eq!(value["staff"][0]["shortlist"]["preferredJob"], "Physio");
        assert_eq!(value["staff"][0]["shortlist"]["clubJob"], "-");
        assert_eq!(
            value["staff"][0]["shortlist"]["coachingQualifications"],
            "Continental Pro"
        );
        assert_eq!(
            serde_json::to_value(StaffPageDto::from(StaffPage {
                state: StaffPageState::NoShortlist,
                staff: Vec::new(),
                total: 0,
                preferred_job_options: Vec::new(),
            }))
            .expect("serialize no shortlist state")["state"],
            "no_shortlist"
        );
    }

    #[test]
    fn staff_assignment_targets_dto_serializes_teams_and_targets_in_camel_case() {
        let value = serde_json::to_value(StaffAssignmentTargetsDto {
            teams: vec![StaffAssignmentTargetTeamDto {
                team: "reserves".to_string(),
                display_name: "B Team".to_string(),
            }],
            targets: vec![StaffAssignmentTargetDto {
                scope: "reserves".to_string(),
                job_id: "manager".to_string(),
                job_label: "Manager".to_string(),
                slot_count: 2,
            }],
        })
        .expect("serialize staff assignment targets");
        assert_eq!(value["teams"][0]["team"], "reserves");
        assert_eq!(value["teams"][0]["displayName"], "B Team");
        assert_eq!(value["targets"][0]["scope"], "reserves");
        assert_eq!(value["targets"][0]["jobId"], "manager");
        assert_eq!(value["targets"][0]["jobLabel"], "Manager");
        assert_eq!(value["targets"][0]["slotCount"], 2);
    }

    #[test]
    fn staff_assignment_optimizer_dto_serializes_context_slots_and_evidence_in_camel_case() {
        let value = serde_json::to_value(StaffAssignmentOptimizationDto {
            state: "ready",
            save_id: 3,
            save_context_token: "save-token".to_string(),
            snapshot_id: Some(7),
            snapshot_context_token: Some("snapshot-token".to_string()),
            joined_candidate_count: 2,
            configured_slot_count: 1,
            unsupported_preferred_job_count: 1,
            slots: vec![StaffAssignmentSlotDto::Vacancy {
                scope: "senior".to_string(),
                scope_display_name: "First Team".to_string(),
                job_id: "assistant_manager".to_string(),
                job_label: "Assistant Manager".to_string(),
                slot_number: 1,
                evidence: StaffAssignmentEvidenceDto {
                    job_id: "assistant_manager".to_string(),
                    joined_candidate_count: 2,
                    eligible_score_count: 0,
                    unavailable_score_count: 2,
                },
            }],
            evidence: vec![StaffAssignmentEvidenceDto {
                job_id: "assistant_manager".to_string(),
                joined_candidate_count: 2,
                eligible_score_count: 0,
                unavailable_score_count: 2,
            }],
        })
        .expect("serialize assignment optimizer");

        assert_eq!(value["state"], "ready");
        assert_eq!(value["saveContextToken"], "save-token");
        assert_eq!(value["snapshotContextToken"], "snapshot-token");
        assert_eq!(value["joinedCandidateCount"], 2);
        assert_eq!(value["configuredSlotCount"], 1);
        assert_eq!(value["unsupportedPreferredJobCount"], 1);
        assert_eq!(value["slots"][0]["kind"], "vacancy");
        assert_eq!(value["slots"][0]["scopeDisplayName"], "First Team");
        assert_eq!(value["slots"][0]["slotNumber"], 1);
        assert_eq!(value["slots"][0]["evidence"]["unavailableScoreCount"], 2);
    }

    #[test]
    fn staff_and_player_boosts_share_one_application_gate() {
        let _test_guard = BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let gate = boost_gate::acquire_boost_gate().expect("hold shared boost gate");
        let db = Db(Mutex::new(
            Connection::open_in_memory().expect("open test database"),
        ));
        let error = execute_staff_boost_with(88, &db, |_| {
            panic!("staff request must not reach the bridge while the shared gate is held")
        })
        .expect_err("concurrent boost must be rejected");
        drop(gate);

        assert!(matches!(
            error,
            StaffBoostError::Bridge { ref kind, .. } if kind == "inProgress"
        ));
    }

    #[test]
    fn uncertain_and_unreconciled_results_latch_recovery_for_staff_and_players() {
        let _test_guard = BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for failure in ["uncertain", "unreconciled"] {
            let (_temp, db) = seeded_db(115, 140);
            let error = execute_staff_boost_with(88, &db, |prepared| match failure {
                "uncertain" => Err(StaffBoostError::SnapshotSync {
                    message: "FM may have changed before verification".to_string(),
                }),
                _ => Ok(verified_result(prepared, 126)),
            })
            .expect_err("unsafe outcome must fail");
            assert!(matches!(error, StaffBoostError::SnapshotSync { .. }));

            let conn = db.0.lock().expect("lock database");
            let recovery: i64 = conn
                .query_row(
                    "SELECT boost_recovery_required FROM snapshots WHERE is_current = 1",
                    [],
                    |row| row.get(0),
                )
                .expect("read recovery");
            assert_eq!(recovery, 1, "{failure} must latch recovery");
            assert!(matches!(
                service::prepare_current_ability_boost(&conn, 88),
                Err(StaffBoostError::SnapshotSync { .. })
            ));
            assert!(prepare_current_ability_boost_for_test(&conn, 77).is_err());
        }
    }

    #[test]
    fn proven_no_write_failures_leave_shared_recovery_clear() {
        let _test_guard = BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for error in [
            StaffBoostError::Bridge {
                kind: "rejected".to_string(),
                message: "Load Data again before using staff boosts".to_string(),
            },
            StaffBoostError::LiveValue {
                message: "staff values changed in FM; Load Data again".to_string(),
            },
        ] {
            let (_temp, db) = seeded_db(115, 140);
            execute_staff_boost_with(88, &db, |_| Err(error))
                .expect_err("proven no-write failure must be returned");
            let conn = db.0.lock().expect("lock database");
            let recovery: i64 = conn
                .query_row(
                    "SELECT boost_recovery_required FROM snapshots WHERE is_current = 1",
                    [],
                    |row| row.get(0),
                )
                .expect("read recovery");
            assert_eq!(recovery, 0);
            assert!(service::prepare_current_ability_boost(&conn, 88).is_ok());
            assert!(prepare_current_ability_boost_for_test(&conn, 77).is_ok());
        }
    }

    #[test]
    fn my_staff_boost_uses_the_configured_family_and_skips_staff_at_their_cap() {
        let _test_guard = BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp, db) = seeded_db(115, 140);
        configure_managed_club(&db);
        clone_staff(&db, 89, 140, 140, "Golden FC");
        clone_staff(&db, 90, 100, 140, "Other FC");
        let calls = std::cell::RefCell::new(Vec::new());

        let result = execute_my_staff_boost_with(&db, |prepared| {
            calls.borrow_mut().push(prepared.staff_uid);
            Ok(verified_result(
                prepared,
                prepared.target_current_ability as i32,
            ))
        })
        .expect("boost My Staff");

        assert_eq!(calls.into_inner(), vec![88]);
        assert_eq!(result.updated, 1);
        assert_eq!(result.skipped, 1);
        assert_eq!(result.failed, 0);
        assert!(!result.recovery_required);
        let conn = db.0.lock().expect("lock database");
        let abilities: Vec<(i64, i64)> = conn
            .prepare("SELECT uid, ca FROM staff ORDER BY uid")
            .expect("prepare abilities")
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query abilities")
            .collect::<Result<_, _>>()
            .expect("read abilities");
        assert_eq!(abilities, vec![(88, 125), (89, 140), (90, 100)]);
    }

    #[test]
    fn my_staff_boost_stops_and_latches_recovery_after_an_uncertain_result() {
        let _test_guard = BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp, db) = seeded_db(115, 140);
        configure_managed_club(&db);
        clone_staff(&db, 89, 100, 140, "Golden FC");
        let calls = std::cell::RefCell::new(Vec::new());

        let result = execute_my_staff_boost_with(&db, |prepared| {
            calls.borrow_mut().push(prepared.staff_uid);
            Err(StaffBoostError::SnapshotSync {
                message: "FM may have changed before verification".to_string(),
            })
        })
        .expect("report partial My Staff result");

        assert_eq!(calls.into_inner(), vec![88]);
        assert!(result.recovery_required);
        assert_eq!(result.updated, 0);
        let recovery: i64 =
            db.0.lock()
                .expect("lock database")
                .query_row(
                    "SELECT boost_recovery_required FROM snapshots WHERE is_current = 1",
                    [],
                    |row| row.get(0),
                )
                .expect("read recovery state");
        assert_eq!(recovery, 1);
    }

    #[test]
    fn my_staff_boost_returns_a_global_bridge_failure_without_latching_recovery() {
        let _test_guard = BOOST_TEST_GATE
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let (_temp, db) = seeded_db(115, 140);
        configure_managed_club(&db);
        clone_staff(&db, 89, 100, 140, "Golden FC");
        let calls = std::cell::RefCell::new(Vec::new());

        let error = execute_my_staff_boost_with(&db, |prepared| {
            calls.borrow_mut().push(prepared.staff_uid);
            Err(StaffBoostError::Bridge {
                kind: "unavailable".to_string(),
                message: "Bridge is unavailable.".to_string(),
            })
        })
        .expect_err("return the global bridge failure");

        assert_eq!(calls.into_inner(), vec![88]);
        assert!(matches!(
            error,
            StaffBoostError::Bridge { ref kind, .. } if kind == "unavailable"
        ));
        let recovery: i64 =
            db.0.lock()
                .expect("lock database")
                .query_row(
                    "SELECT boost_recovery_required FROM snapshots WHERE is_current = 1",
                    [],
                    |row| row.get(0),
                )
                .expect("read recovery state");
        assert_eq!(recovery, 0);
    }
}
