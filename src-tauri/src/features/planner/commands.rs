use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::blocking;
use crate::db::Db;
use crate::features::memory_read::service::{self as bridge, DumpWaitConfig};
use crate::features::snapshot::service;

use super::depth::{
    self as depth_service, PlannerAssignment, PlannerAssignmentLocation, PlannerDepth,
    PlannerDepthTeam, PlannerSlotCandidate, PlannerString, PlannerTeam,
};
use super::fit::PositionRule;
use super::fm_moves::{self, FmSquadMove, FmSquadMoveReport};
use super::fm_training::{self, FmTrainingChange, FmTrainingReport};
use super::loans::{self, LoanCandidate};
use super::optimizer;
use super::role_reference::{
    self as role_reference_service, RoleReference, RoleReferenceBasis, RoleReferencePhase,
};
use super::squad::{
    self as squad_service, SquadPlayer, SquadPlayersPage, SquadSortDir, SquadSortField,
    DEFAULT_SQUAD_PAGE_LIMIT, MAX_SQUAD_PAGE_LIMIT,
};
use super::tactic::{
    self as tactic_service, PlannerTactic, RecruitmentFilter, TacticLane, TacticOptions,
};
use super::teams::{
    self as teams_service, PlannerStaffingTargetRemovalImpact, PlannerTeamInput,
    PlannerTeamRemovalImpact,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadPlayerDto {
    pub uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub birth_year: i64,
    pub birth_day_of_year: i64,
    pub nationalities: Vec<String>,
    pub club: Option<String>,
    pub current_club_uid: Option<i64>,
    pub division: Option<String>,
    pub ca: i64,
    pub pa: i64,
    pub market_value_gbp: Option<i64>,
    pub suggested_training: Option<String>,
    pub dynamic_values: std::collections::BTreeMap<String, Option<DynamicValueDto>>,
    pub loaned_out: bool,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum DynamicValueDto {
    Integer(i64),
    Real(f64),
    Text(String),
}

impl From<crate::features::player_metrics::resolver::DynamicValue> for DynamicValueDto {
    fn from(value: crate::features::player_metrics::resolver::DynamicValue) -> Self {
        match value {
            crate::features::player_metrics::resolver::DynamicValue::Integer(number) => {
                Self::Integer(number)
            }
            crate::features::player_metrics::resolver::DynamicValue::Real(number) => {
                Self::Real(number)
            }
            crate::features::player_metrics::resolver::DynamicValue::Text(text) => Self::Text(text),
        }
    }
}

impl From<SquadPlayer> for SquadPlayerDto {
    fn from(player: SquadPlayer) -> Self {
        Self {
            uid: player.uid,
            name: player.name,
            age: player.age,
            birth_year: player.birth_year,
            birth_day_of_year: player.birth_day_of_year,
            nationalities: player.nationalities,
            club: player.club,
            current_club_uid: player.current_club_uid,
            division: player.division,
            ca: player.ca,
            pa: player.pa,
            market_value_gbp: player.market_value_gbp,
            suggested_training: player.suggested_training,
            loaned_out: player.loaned_out,
            dynamic_values: player
                .dynamic_values
                .into_iter()
                .map(|(key, value)| (key, value.map(DynamicValueDto::from)))
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadPlayersPageDto {
    pub players: Vec<SquadPlayerDto>,
    pub total: i64,
}

impl From<SquadPlayersPage> for SquadPlayersPageDto {
    fn from(page: SquadPlayersPage) -> Self {
        Self {
            players: page.players.into_iter().map(SquadPlayerDto::from).collect(),
            total: page.total,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TacticLaneDto {
    pub lane_id: String,
    pub ip_weight: f64,
    pub importance_rank: Option<u8>,
    pub preferred_foot: String,
    pub foot_preference: String,
    pub ip_position: String,
    pub ip_role_id: String,
    pub oop_position: String,
    pub oop_role_id: String,
    pub custom_metric_keys: Vec<String>,
    pub recruitment_filters: Vec<RecruitmentFilterDto>,
}

/// A recruitment filter rule in Search's filter IPC shape.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecruitmentFilterDto {
    pub field: String,
    pub op: String,
    pub value: serde_json::Value,
}

impl From<RecruitmentFilterDto> for RecruitmentFilter {
    fn from(filter: RecruitmentFilterDto) -> Self {
        Self {
            field: filter.field,
            op: filter.op,
            value: filter.value,
        }
    }
}

impl From<RecruitmentFilter> for RecruitmentFilterDto {
    fn from(filter: RecruitmentFilter) -> Self {
        Self {
            field: filter.field,
            op: filter.op,
            value: filter.value,
        }
    }
}

impl From<TacticLaneDto> for TacticLane {
    fn from(lane: TacticLaneDto) -> Self {
        Self {
            lane_id: lane.lane_id,
            ip_weight: lane.ip_weight,
            importance_rank: lane.importance_rank,
            preferred_foot: lane.preferred_foot,
            foot_preference: lane.foot_preference,
            ip_position: lane.ip_position,
            ip_role_id: lane.ip_role_id,
            oop_position: lane.oop_position,
            oop_role_id: lane.oop_role_id,
            custom_metric_keys: lane.custom_metric_keys,
            recruitment_filters: lane
                .recruitment_filters
                .into_iter()
                .map(RecruitmentFilter::from)
                .collect(),
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerTacticInputDto {
    pub lanes: Vec<TacticLaneDto>,
}

impl From<PlannerTacticInputDto> for PlannerTactic {
    fn from(input: PlannerTacticInputDto) -> Self {
        Self {
            lanes: input.lanes.into_iter().map(TacticLane::from).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TacticLaneResponseDto {
    pub lane_id: String,
    pub ip_weight: f64,
    pub importance_rank: Option<u8>,
    pub preferred_foot: String,
    pub foot_preference: String,
    pub ip_position: String,
    pub ip_role_id: String,
    pub oop_position: String,
    pub oop_role_id: String,
    pub custom_metric_keys: Vec<String>,
    pub recruitment_filters: Vec<RecruitmentFilterDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerTacticDto {
    pub lanes: Vec<TacticLaneResponseDto>,
}

impl From<PlannerTactic> for PlannerTacticDto {
    fn from(tactic: PlannerTactic) -> Self {
        Self {
            lanes: tactic
                .lanes
                .into_iter()
                .map(|lane| TacticLaneResponseDto {
                    lane_id: lane.lane_id,
                    ip_weight: lane.ip_weight,
                    importance_rank: lane.importance_rank,
                    preferred_foot: lane.preferred_foot,
                    foot_preference: lane.foot_preference,
                    ip_position: lane.ip_position,
                    ip_role_id: lane.ip_role_id,
                    oop_position: lane.oop_position,
                    oop_role_id: lane.oop_role_id,
                    custom_metric_keys: lane.custom_metric_keys,
                    recruitment_filters: lane
                        .recruitment_filters
                        .into_iter()
                        .map(RecruitmentFilterDto::from)
                        .collect(),
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerRoleReferencePlayerDto {
    pub player_uid: i64,
    pub name: String,
    pub current_score: Option<u8>,
    pub potential_score: Option<u8>,
}

impl From<role_reference_service::RoleReferencePlayer> for PlannerRoleReferencePlayerDto {
    fn from(player: role_reference_service::RoleReferencePlayer) -> Self {
        Self {
            player_uid: player.player_uid,
            name: player.name,
            current_score: player.current_score,
            potential_score: player.potential_score,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerRoleReferenceLaneDto {
    pub lane_id: String,
    pub players: Vec<PlannerRoleReferencePlayerDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerRoleReferenceDto {
    pub lanes: Vec<PlannerRoleReferenceLaneDto>,
    pub no_eligible: Vec<PlannerRoleReferencePlayerDto>,
}

impl From<RoleReference> for PlannerRoleReferenceDto {
    fn from(reference: RoleReference) -> Self {
        Self {
            lanes: reference
                .lanes
                .into_iter()
                .map(|lane| PlannerRoleReferenceLaneDto {
                    lane_id: lane.lane_id,
                    players: lane
                        .players
                        .into_iter()
                        .map(PlannerRoleReferencePlayerDto::from)
                        .collect(),
                })
                .collect(),
            no_eligible: reference
                .no_eligible
                .into_iter()
                .map(PlannerRoleReferencePlayerDto::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TacticRoleOptionDto {
    pub role_id: String,
    pub display_name: String,
    pub phase: String,
    pub position_tags: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TacticOptionsDto {
    pub placements: Vec<String>,
    pub roles: Vec<TacticRoleOptionDto>,
}

impl From<TacticOptions> for TacticOptionsDto {
    fn from(options: TacticOptions) -> Self {
        Self {
            placements: options.placements,
            roles: options
                .roles
                .into_iter()
                .map(|role| TacticRoleOptionDto {
                    role_id: role.role_id,
                    display_name: role.display_name,
                    phase: role.phase,
                    position_tags: role.position_tags,
                })
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerAssignmentDto {
    pub id: i64,
    pub lane_id: String,
    pub player_uid: i64,
    pub last_known_name: String,
    pub current_name: Option<String>,
    pub state: String,
    pub age: Option<i64>,
    pub pinned: bool,
    pub over_age: bool,
    pub out_of_position: bool,
    pub combined_score: Option<u8>,
    pub expected_combined_score: Option<u8>,
    pub potential_combined_score: Option<u8>,
}

impl From<PlannerAssignment> for PlannerAssignmentDto {
    fn from(assignment: PlannerAssignment) -> Self {
        Self {
            id: assignment.id,
            lane_id: assignment.lane_id,
            player_uid: assignment.player_uid,
            last_known_name: assignment.last_known_name,
            current_name: assignment.current_name,
            state: assignment.state.as_str().to_string(),
            age: assignment.age,
            pinned: assignment.pinned,
            over_age: assignment.over_age,
            out_of_position: assignment.out_of_position,
            combined_score: assignment.combined_score,
            expected_combined_score: assignment.expected_combined_score,
            potential_combined_score: assignment.potential_combined_score,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerAssignmentLocationDto {
    pub team: String,
    pub string_id: i64,
    pub string_name: String,
    pub lane_id: String,
}

impl From<PlannerAssignmentLocation> for PlannerAssignmentLocationDto {
    fn from(location: PlannerAssignmentLocation) -> Self {
        Self {
            team: location.team.as_str().to_string(),
            string_id: location.string_id,
            string_name: location.string_name,
            lane_id: location.lane_id,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerSlotCandidateDto {
    pub player_uid: i64,
    pub name: String,
    pub current_club: String,
    pub age: Option<i64>,
    pub over_age: bool,
    pub ip_score: Option<u8>,
    pub oop_score: Option<u8>,
    pub combined_score: Option<u8>,
    pub expected_combined_score: Option<u8>,
    pub position_fit: &'static str,
    pub foot_fit: &'static str,
    pub assignment_location: Option<PlannerAssignmentLocationDto>,
}

impl From<PlannerSlotCandidate> for PlannerSlotCandidateDto {
    fn from(candidate: PlannerSlotCandidate) -> Self {
        Self {
            player_uid: candidate.player_uid,
            name: candidate.name,
            current_club: candidate.current_club,
            age: candidate.age,
            over_age: candidate.over_age,
            ip_score: candidate.ip_score,
            oop_score: candidate.oop_score,
            combined_score: candidate.combined_score,
            expected_combined_score: candidate.expected_combined_score,
            position_fit: candidate.position_fit.as_str(),
            foot_fit: candidate.foot_fit.as_str(),
            assignment_location: candidate
                .assignment_location
                .map(PlannerAssignmentLocationDto::from),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerStringDto {
    pub id: i64,
    pub string_order: i64,
    pub display_name: String,
    pub assignments: Vec<PlannerAssignmentDto>,
}

impl From<PlannerString> for PlannerStringDto {
    fn from(planner_string: PlannerString) -> Self {
        Self {
            id: planner_string.id,
            string_order: planner_string.string_order,
            display_name: planner_string.display_name,
            assignments: planner_string
                .assignments
                .into_iter()
                .map(PlannerAssignmentDto::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerDepthTeamDto {
    pub team: String,
    pub display_name: String,
    pub max_age: Option<i64>,
    pub strings: Vec<PlannerStringDto>,
}

impl From<PlannerDepthTeam> for PlannerDepthTeamDto {
    fn from(team: PlannerDepthTeam) -> Self {
        Self {
            team: team.team.as_str().to_string(),
            display_name: team.display_name,
            max_age: team.max_age,
            strings: team
                .strings
                .into_iter()
                .map(PlannerStringDto::from)
                .collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerDepthDto {
    pub tactic: PlannerTacticDto,
    pub teams: Vec<PlannerDepthTeamDto>,
}

impl From<PlannerDepth> for PlannerDepthDto {
    fn from(depth: PlannerDepth) -> Self {
        Self {
            tactic: depth.tactic.into(),
            teams: depth
                .teams
                .into_iter()
                .map(PlannerDepthTeamDto::from)
                .collect(),
        }
    }
}

/// Answers on a read connection when `has_defaults` finds every default `answer` would create,
/// and otherwise on the writer, where `answer` creates them. Creation is idempotent, so two
/// first reads that both miss create the defaults once.
fn answer_without_creating_defaults_on_a_reader<T>(
    db: &Db,
    has_defaults: impl FnOnce(&Connection) -> Result<bool, String>,
    answer: impl Fn(&Connection) -> Result<T, String>,
) -> Result<T, String> {
    {
        let conn = db.read()?;
        if has_defaults(&conn)? {
            return answer(&conn);
        }
    }
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    answer(&conn)
}

#[tauri::command]
pub async fn list_squad_players(
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    requested_fields: Option<Vec<String>>,
    db: State<'_, Db>,
) -> Result<SquadPlayersPageDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        read_squad_players(&db, offset, limit, sort_by, sort_dir, requested_fields)
    })
    .await
}

fn read_squad_players(
    db: &Db,
    offset: Option<u32>,
    limit: Option<u32>,
    sort_by: Option<String>,
    sort_dir: Option<String>,
    requested_fields: Option<Vec<String>>,
) -> Result<SquadPlayersPageDto, String> {
    let conn = db.read()?;
    let save_id = service::active_save_id(&conn)?;
    let offset = offset.unwrap_or(0) as usize;
    let limit = limit
        .map(|value| value as usize)
        .unwrap_or(DEFAULT_SQUAD_PAGE_LIMIT)
        .clamp(1, MAX_SQUAD_PAGE_LIMIT);
    let sort_by = match sort_by.as_deref() {
        None => SquadSortField::DEFAULT,
        Some(value) => SquadSortField::parse(value)?,
    };
    let sort_dir = match sort_dir.as_deref() {
        None => SquadSortDir::DEFAULT,
        Some(value) => SquadSortDir::parse(value)?,
    };
    let requested_fields = requested_fields.unwrap_or_default();
    Ok(squad_service::list_squad_players(
        &conn,
        save_id,
        offset,
        limit,
        sort_by,
        sort_dir,
        &requested_fields,
    )?
    .into())
}

fn get_planner_tactic_for_context(
    conn: &Connection,
    context: &service::SaveContext,
) -> Result<PlannerTactic, String> {
    service::ensure_save_context(conn, context)?;
    if tactic_service::tactic_exists(conn, context.id)? {
        return tactic_service::load_tactic(conn, context.id);
    }
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    service::ensure_save_context(&tx, context)?;
    let tactic = tactic_service::load_or_initialize_tactic_in_tx(&tx, context.id)?;
    tx.commit().map_err(|error| error.to_string())?;
    Ok(tactic)
}

fn get_planner_tactic_options_for_context(
    conn: &Connection,
    context: &service::SaveContext,
) -> Result<TacticOptions, String> {
    service::ensure_save_context(conn, context)?;
    Ok(tactic_service::get_tactic_options())
}

fn save_planner_tactic_for_context(
    conn: &Connection,
    context: &service::SaveContext,
    tactic: &PlannerTactic,
) -> Result<(), String> {
    let tx = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;
    service::ensure_save_context(&tx, context)?;
    tactic_service::save_tactic_in_tx(&tx, context.id, tactic)?;
    tx.commit().map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_planner_tactic(
    save_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<PlannerTacticDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let context = service::SaveContext {
            id: save_id,
            context_token,
        };
        read_planner_tactic(&db, &context)
    })
    .await
}

fn read_planner_tactic(
    db: &Db,
    context: &service::SaveContext,
) -> Result<PlannerTacticDto, String> {
    answer_without_creating_defaults_on_a_reader(
        db,
        |conn| tactic_service::tactic_exists(conn, context.id),
        |conn| get_planner_tactic_for_context(conn, context),
    )
    .map(Into::into)
}

#[tauri::command]
pub async fn get_planner_role_reference(
    phase: String,
    score_basis: String,
    db: State<'_, Db>,
) -> Result<PlannerRoleReferenceDto, String> {
    let db = db.inner().clone();
    blocking::run(move || read_planner_role_reference(&db, &phase, &score_basis)).await
}

fn read_planner_role_reference(
    db: &Db,
    phase: &str,
    score_basis: &str,
) -> Result<PlannerRoleReferenceDto, String> {
    let phase = RoleReferencePhase::parse(phase)?;
    let score_basis = RoleReferenceBasis::parse(score_basis)?;
    let conn = db.read()?;
    let save_id = service::active_save_id(&conn)?;
    Ok(role_reference_service::get_role_reference(&conn, save_id, phase, score_basis)?.into())
}

#[tauri::command]
pub async fn get_planner_tactic_options(
    save_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<TacticOptionsDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let context = service::SaveContext {
            id: save_id,
            context_token,
        };
        read_planner_tactic_options(&db, &context)
    })
    .await
}

fn read_planner_tactic_options(
    db: &Db,
    context: &service::SaveContext,
) -> Result<TacticOptionsDto, String> {
    let conn = db.read()?;
    Ok(get_planner_tactic_options_for_context(&conn, context)?.into())
}

#[tauri::command]
pub async fn save_planner_tactic(
    save_id: i64,
    context_token: String,
    tactic: PlannerTacticInputDto,
    db: State<'_, Db>,
) -> Result<PlannerTacticDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let context = service::SaveContext {
            id: save_id,
            context_token,
        };
        let tactic = PlannerTactic::from(tactic);
        save_planner_tactic_for_context(&conn, &context, &tactic)?;
        Ok(tactic.into())
    })
    .await
}

#[tauri::command]
pub async fn get_planner_depth(db: State<'_, Db>) -> Result<PlannerDepthDto, String> {
    let db = db.inner().clone();
    blocking::run(move || read_planner_depth(&db)).await
}

fn read_planner_depth(db: &Db) -> Result<PlannerDepthDto, String> {
    answer_without_creating_defaults_on_a_reader(
        db,
        |conn| depth_service::depth_exists(conn, service::active_save_id(conn)?),
        |conn| depth_service::get_depth(conn, service::active_save_id(conn)?),
    )
    .map(Into::into)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerTeamInputDto {
    pub team: String,
    pub display_name: String,
    pub max_age: Option<i64>,
}

impl From<PlannerTeamInputDto> for PlannerTeamInput {
    fn from(input: PlannerTeamInputDto) -> Self {
        Self {
            team: input.team,
            display_name: input.display_name,
            max_age: input.max_age,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerStaffingTargetRemovalImpactDto {
    pub job_id: String,
    pub job_label: String,
    pub slot_count: i64,
}

impl From<PlannerStaffingTargetRemovalImpact> for PlannerStaffingTargetRemovalImpactDto {
    fn from(impact: PlannerStaffingTargetRemovalImpact) -> Self {
        Self {
            job_id: impact.job_id,
            job_label: impact.job_label,
            slot_count: impact.slot_count,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerTeamRemovalImpactDto {
    pub team: String,
    pub display_name: String,
    pub assignment_count: i64,
    pub staffing_targets: Vec<PlannerStaffingTargetRemovalImpactDto>,
}

impl From<PlannerTeamRemovalImpact> for PlannerTeamRemovalImpactDto {
    fn from(impact: PlannerTeamRemovalImpact) -> Self {
        Self {
            team: impact.team.as_str().to_string(),
            display_name: impact.display_name,
            assignment_count: impact.assignment_count,
            staffing_targets: impact
                .staffing_targets
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[tauri::command]
pub async fn get_planner_team_removal_impacts(
    teams: Vec<PlannerTeamInputDto>,
    db: State<'_, Db>,
) -> Result<Vec<PlannerTeamRemovalImpactDto>, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let teams = teams
            .into_iter()
            .map(PlannerTeamInput::from)
            .collect::<Vec<_>>();
        read_planner_team_removal_impacts(&db, &teams)
    })
    .await
}

fn read_planner_team_removal_impacts(
    db: &Db,
    teams: &[PlannerTeamInput],
) -> Result<Vec<PlannerTeamRemovalImpactDto>, String> {
    answer_without_creating_defaults_on_a_reader(
        db,
        |conn| teams_service::team_settings_exist(conn, service::active_save_id(conn)?),
        |conn| {
            teams_service::planner_team_removal_impacts(conn, service::active_save_id(conn)?, teams)
        },
    )
    .map(|impacts| impacts.into_iter().map(Into::into).collect())
}

#[tauri::command]
pub async fn save_planner_teams(
    teams: Vec<PlannerTeamInputDto>,
    confirm_populated_removal: bool,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let save_id = service::active_save_id(&conn)?;
        let teams = teams
            .into_iter()
            .map(PlannerTeamInput::from)
            .collect::<Vec<_>>();
        let (_, snapshot_id) =
            teams_service::save_team_settings(&conn, save_id, &teams, confirm_populated_removal)?;
        Ok(depth_service::load_depth(&conn, save_id, snapshot_id)?.into())
    })
    .await
}

/// Refills the squads. Without `position_familiarity`, players are placed on role scores and
/// preferred foot alone, whatever positions they can play.
#[tauri::command]
pub async fn optimize_planner_depth(
    position_familiarity: bool,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let db = db.inner().clone();
    let position_rule = if position_familiarity {
        PositionRule::Familiarity
    } else {
        PositionRule::AnyPosition
    };
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let save_id = service::active_save_id(&conn)?;
        Ok(optimizer::optimize_depth(&conn, save_id, position_rule)?.into())
    })
    .await
}

#[tauri::command]
pub async fn get_planner_slot_candidates(
    team: String,
    lane_id: String,
    search: String,
    db: State<'_, Db>,
) -> Result<Vec<PlannerSlotCandidateDto>, String> {
    let db = db.inner().clone();
    blocking::run(move || read_planner_slot_candidates(&db, &team, &lane_id, &search)).await
}

fn read_planner_slot_candidates(
    db: &Db,
    team: &str,
    lane_id: &str,
    search: &str,
) -> Result<Vec<PlannerSlotCandidateDto>, String> {
    let team = PlannerTeam::parse(team)?;
    answer_without_creating_defaults_on_a_reader(
        db,
        |conn| depth_service::depth_exists(conn, service::active_save_id(conn)?),
        |conn| {
            let save_id = service::active_save_id(conn)?;
            depth_service::get_slot_candidates(conn, save_id, team, lane_id, search)
        },
    )
    .map(|candidates| {
        candidates
            .into_iter()
            .map(PlannerSlotCandidateDto::from)
            .collect()
    })
}

#[tauri::command]
pub async fn clear_planner_depth(
    confirmed: bool,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let save_id = service::active_save_id(&conn)?;
        let (_, snapshot_id) = depth_service::clear_all(&conn, save_id, confirmed)?;
        Ok(depth_service::load_depth(&conn, save_id, snapshot_id)?.into())
    })
    .await
}

#[tauri::command]
pub async fn clear_planner_assignment(
    string_id: i64,
    lane_id: String,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let save_id = service::active_save_id(&conn)?;
        let (_, snapshot_id) =
            depth_service::clear_assignment(&conn, save_id, string_id, &lane_id)?;
        Ok(depth_service::load_depth(&conn, save_id, snapshot_id)?.into())
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerUnassignedPlayerDto {
    pub player_uid: i64,
    pub name: String,
    pub age: Option<i64>,
    /// The tactic lane the player fits best now; absent when he suits none.
    pub lane_id: Option<String>,
    pub current_score: Option<u8>,
    pub expected_score: Option<u8>,
}

/// The managed club's players in no squad, best current fit first.
#[tauri::command]
pub async fn get_planner_unassigned_players(
    db: State<'_, Db>,
) -> Result<Vec<PlannerUnassignedPlayerDto>, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let save_id = service::active_save_id(&conn)?;
        Ok(optimizer::unassigned_players(&conn, save_id)?
            .into_iter()
            .map(|player| {
                let (lane_id, current_score, expected_score) = match player.best_lane {
                    Some(lane) => (
                        Some(lane.lane_id),
                        Some(lane.current_score),
                        Some(lane.expected_score),
                    ),
                    None => (None, None, None),
                };
                PlannerUnassignedPlayerDto {
                    player_uid: player.player_uid,
                    name: player.name,
                    age: player.age,
                    lane_id,
                    current_score,
                    expected_score,
                }
            })
            .collect())
    })
    .await
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerLoanCandidateDto {
    pub player_uid: i64,
    pub name: String,
    pub age: i64,
    /// The team and string that hold the player; absent when no squad does.
    pub team: Option<&'static str>,
    pub string_name: Option<String>,
    /// The tactic lane he is judged in.
    pub lane_id: String,
    pub current_score: u8,
    pub expected_score: u8,
    /// How much a loan would help him, from 0 to 100.
    pub benefit: u8,
    pub recommended: bool,
}

impl From<LoanCandidate> for PlannerLoanCandidateDto {
    fn from(candidate: LoanCandidate) -> Self {
        let (team, string_name) = match candidate.place {
            Some(place) => (Some(place.team.as_str()), Some(place.string_name)),
            None => (None, None),
        };
        Self {
            player_uid: candidate.player_uid,
            name: candidate.name,
            age: candidate.age,
            team,
            string_name,
            lane_id: candidate.lane_id,
            current_score: candidate.current_score,
            expected_score: candidate.expected_score,
            benefit: candidate.benefit,
            recommended: candidate.recommended,
        }
    }
}

/// The managed club's players aged 18 to 23 outside the senior squad, most to gain from a
/// loan first.
#[tauri::command]
pub async fn get_planner_loan_candidates(
    db: State<'_, Db>,
) -> Result<Vec<PlannerLoanCandidateDto>, String> {
    let db = db.inner().clone();
    blocking::run(move || read_planner_loan_candidates(&db)).await
}

fn read_planner_loan_candidates(db: &Db) -> Result<Vec<PlannerLoanCandidateDto>, String> {
    answer_without_creating_defaults_on_a_reader(
        db,
        |conn| depth_service::depth_exists(conn, service::active_save_id(conn)?),
        |conn| loans::loan_candidates(conn, service::active_save_id(conn)?),
    )
    .map(|candidates| candidates.into_iter().map(Into::into).collect())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerFmMoveDto {
    pub player_uid: i64,
    pub name: String,
    pub age: Option<i64>,
    /// The level of his FM team in the snapshot: `senior`, `reserve`, or `youth`; null when
    /// unknown, which the bridge settles from FM itself.
    pub from_level: Option<String>,
    pub to_team: &'static str,
    pub to_team_name: String,
    /// Older than the target squad's age limit; FM moves him anyway.
    pub over_age: bool,
}

impl From<FmSquadMove> for PlannerFmMoveDto {
    fn from(planned: FmSquadMove) -> Self {
        Self {
            player_uid: planned.player_uid,
            name: planned.name,
            age: planned.age,
            from_level: planned.from_level,
            to_team: planned.to_team.as_str(),
            to_team_name: planned.to_team_name,
            over_age: planned.over_age,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerFmMovesDto {
    pub moves: Vec<PlannerFmMoveDto>,
    /// Why the moves cannot be applied now; absent when they can.
    pub unavailable_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerFmMoveFailureDto {
    pub player_uid: i64,
    pub name: String,
    pub reason: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerFmMoveResultDto {
    pub moved: Vec<i64>,
    pub unchanged: Vec<i64>,
    pub failed: Vec<PlannerFmMoveFailureDto>,
    /// The error that stopped the moves early; FM may have made some of them first.
    pub error: Option<String>,
}

impl From<FmSquadMoveReport> for PlannerFmMoveResultDto {
    fn from(report: FmSquadMoveReport) -> Self {
        Self {
            moved: report.moved,
            unchanged: report.unchanged,
            failed: report
                .failed
                .into_iter()
                .map(|failure| PlannerFmMoveFailureDto {
                    player_uid: failure.player_uid,
                    name: failure.name,
                    reason: failure.reason,
                })
                .collect(),
            error: report.error,
        }
    }
}

/// The moves that would put each Planner-placed player at the managed club into the FM team
/// his squad stands for, and whether FM can make them now.
#[tauri::command]
pub async fn get_planner_fm_moves(db: State<'_, Db>) -> Result<PlannerFmMovesDto, String> {
    let db = db.inner().clone();
    blocking::run(move || read_planner_fm_moves(&db, bridge_squad_moves_unavailable)).await
}

pub(super) fn read_planner_fm_moves(
    db: &Db,
    bridge_unavailable: impl FnOnce() -> Option<String>,
) -> Result<PlannerFmMovesDto, String> {
    let (plan, context) = {
        let conn = db.read()?;
        let save_id = service::active_save_id(&conn)?;
        (
            fm_moves::plan_fm_squad_moves(&conn, save_id)?,
            fm_moves::capture_move_context(&conn),
        )
    };
    Ok(PlannerFmMovesDto {
        moves: plan.into_iter().map(Into::into).collect(),
        unavailable_reason: match context {
            Err(reason) => Some(reason),
            Ok(_) => bridge_unavailable(),
        },
    })
}

/// None when the bridge offers squad moves for its live scan; otherwise why not.
fn bridge_squad_moves_unavailable() -> Option<String> {
    let status = bridge::resolve_bridge_directory()
        .and_then(|directory| bridge::read_bridge_status(&directory));
    match status {
        Ok(status) if status.squad_moves_supported == Some(true) => None,
        _ => Some(bridge::SQUAD_MOVES_UNAVAILABLE.to_string()),
    }
}

/// Moves are quick; a bridge that has not answered in this time is not running.
const SQUAD_MOVE_WAIT: DumpWaitConfig = DumpWaitConfig {
    timeout: std::time::Duration::from_secs(45),
    poll_interval: std::time::Duration::from_millis(200),
};

/// Moves the chosen players of the current plan in FM and records the moves FM confirms.
#[tauri::command]
pub async fn apply_planner_fm_moves(
    player_uids: Vec<i64>,
    db: State<'_, Db>,
) -> Result<PlannerFmMoveResultDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        fm_moves::apply_fm_squad_moves(&db, &player_uids, |source, club, items| {
            bridge::request_squad_moves_from_local_app_data(source, club, items, SQUAD_MOVE_WAIT)
                .map_err(|error| error.to_string())
        })
        .map(Into::into)
    })
    .await
}

/// Where the Planner has a player.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadFmTrainingSlotDto {
    pub team: &'static str,
    pub team_name: String,
    pub string_name: String,
    /// The slot's placements in and out of possession, such as `AMR` and `MR`.
    pub ip_position: String,
    pub oop_position: String,
    /// The slot's own in-possession role, which he trains unless he is learning the
    /// out-of-possession position.
    pub in_possession_role: String,
}

/// A slot position a player trains to become fully familiar with it.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadFmTrainingLearningDto {
    pub position: String,
    /// His familiarity there out of 20; 0 when he has none.
    pub familiarity: i64,
    /// Whether it is the slot's out-of-possession position, which he trains in place of the
    /// slot's own in-possession role and position.
    pub out_of_possession: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadFmTrainingPlayerDto {
    pub player_uid: i64,
    pub name: String,
    pub age: Option<i64>,
    /// His Planner slot; absent for a player in no slot, whose roles FM keeps.
    pub slot: Option<SquadFmTrainingSlotDto>,
    /// The position his in-possession role is trained at; absent for a player in no slot.
    pub position: Option<String>,
    pub in_possession_role: Option<String>,
    pub out_of_possession_role: Option<String>,
    pub learning: Option<SquadFmTrainingLearningDto>,
    /// His Suggested Training; absent means his focus is cleared.
    pub focus: Option<String>,
}

impl From<FmTrainingChange> for SquadFmTrainingPlayerDto {
    fn from(planned: FmTrainingChange) -> Self {
        let mut player = Self {
            player_uid: planned.player_uid,
            name: planned.name,
            age: planned.age,
            slot: None,
            position: None,
            in_possession_role: None,
            out_of_possession_role: None,
            learning: None,
            focus: planned.focus,
        };
        if let Some(slot) = planned.slot {
            player.position = Some(slot.roles.position);
            player.in_possession_role = Some(slot.roles.in_possession);
            player.out_of_possession_role = Some(slot.roles.out_of_possession);
            player.learning = slot.learning.map(|learning| SquadFmTrainingLearningDto {
                position: learning.position,
                familiarity: learning.familiarity,
                out_of_possession: learning.out_of_possession,
            });
            player.slot = Some(SquadFmTrainingSlotDto {
                team: slot.team.as_str(),
                team_name: slot.team_name,
                string_name: slot.string_name,
                ip_position: slot.ip_position,
                oop_position: slot.oop_position,
                in_possession_role: slot.in_possession,
            });
        }
        player
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadFmTrainingDto {
    pub players: Vec<SquadFmTrainingPlayerDto>,
    /// Why the training cannot be set now; absent when it can.
    pub unavailable_reason: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquadFmTrainingResultDto {
    pub applied: Vec<i64>,
    pub failed: Vec<PlannerFmMoveFailureDto>,
    /// The error that stopped the request early; FM may have taken some changes first.
    pub error: Option<String>,
}

impl From<FmTrainingReport> for SquadFmTrainingResultDto {
    fn from(report: FmTrainingReport) -> Self {
        Self {
            applied: report.applied,
            failed: report
                .failed
                .into_iter()
                .map(|failure| PlannerFmMoveFailureDto {
                    player_uid: failure.player_uid,
                    name: failure.name,
                    reason: failure.reason,
                })
                .collect(),
            error: report.error,
        }
    }
}

/// The training Apply in FM would set for each of the managed club's players, and whether FM
/// can take it now.
#[tauri::command]
pub async fn get_squad_fm_training(db: State<'_, Db>) -> Result<SquadFmTrainingDto, String> {
    let db = db.inner().clone();
    blocking::run(move || read_squad_fm_training(&db, bridge_training_unavailable)).await
}

pub(super) fn read_squad_fm_training(
    db: &Db,
    bridge_unavailable: impl FnOnce() -> Option<String>,
) -> Result<SquadFmTrainingDto, String> {
    let (plan, context) = {
        let conn = db.read()?;
        let save_id = service::active_save_id(&conn)?;
        (
            fm_training::plan_fm_training(&conn, save_id)?,
            fm_training::capture_training_context(&conn),
        )
    };
    Ok(SquadFmTrainingDto {
        players: plan.into_iter().map(Into::into).collect(),
        unavailable_reason: match context {
            Err(reason) => Some(reason),
            Ok(_) => bridge_unavailable(),
        },
    })
}

/// None when the bridge offers training changes for its live scan; otherwise why not.
fn bridge_training_unavailable() -> Option<String> {
    match bridge::resolve_bridge_directory()
        .and_then(|directory| bridge::read_bridge_status(&directory))
    {
        Ok(status) => bridge::training_unavailable_reason(&status).map(str::to_string),
        Err(_) => Some(bridge::TRAINING_NEEDS_LOAD_DATA.to_string()),
    }
}

/// The bridge sends the events at once and reads them back within about seven seconds.
const TRAINING_WAIT: DumpWaitConfig = DumpWaitConfig {
    timeout: std::time::Duration::from_secs(45),
    poll_interval: std::time::Duration::from_millis(200),
};

/// Sets the chosen players' training in FM and reports what FM took.
#[tauri::command]
pub async fn apply_squad_fm_training(
    player_uids: Vec<i64>,
    db: State<'_, Db>,
) -> Result<SquadFmTrainingResultDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        fm_training::apply_fm_training(&db, &player_uids, |source, club, items| {
            bridge::request_training_from_local_app_data(source, club, items, TRAINING_WAIT)
                .map_err(|error| error.to_string())
        })
        .map(Into::into)
    })
    .await
}

#[tauri::command]
pub async fn set_planner_assignment_pinned(
    string_id: i64,
    lane_id: String,
    pinned: bool,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let save_id = service::active_save_id(&conn)?;
        let (_, snapshot_id) =
            depth_service::set_assignment_pinned(&conn, save_id, string_id, &lane_id, pinned)?;
        Ok(depth_service::load_depth(&conn, save_id, snapshot_id)?.into())
    })
    .await
}

#[tauri::command]
pub async fn assign_planner_player(
    string_id: i64,
    lane_id: String,
    player_uid: i64,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let save_id = service::active_save_id(&conn)?;
        let (_, snapshot_id) =
            depth_service::assign_player(&conn, save_id, string_id, &lane_id, player_uid)?;
        Ok(depth_service::load_depth(&conn, save_id, snapshot_id)?.into())
    })
    .await
}

#[tauri::command]
pub async fn move_planner_player(
    string_id: i64,
    lane_id: String,
    player_uid: i64,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        let save_id = service::active_save_id(&conn)?;
        let (_, snapshot_id) =
            depth_service::move_player(&conn, save_id, string_id, &lane_id, player_uid)?;
        Ok(depth_service::load_depth(&conn, save_id, snapshot_id)?.into())
    })
    .await
}

#[cfg(test)]
mod tests {
    use crate::db::migrations;

    use super::*;

    fn save_context(conn: &Connection, save_id: i64) -> service::SaveContext {
        conn.query_row(
            "SELECT context_token FROM saves WHERE id = ?1",
            [save_id],
            |row| {
                Ok(service::SaveContext {
                    id: save_id,
                    context_token: row.get(0)?,
                })
            },
        )
        .expect("read save context")
    }

    fn open_with_two_saves() -> (
        tempfile::TempDir,
        Connection,
        service::SaveContext,
        service::SaveContext,
    ) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let conn = Connection::open(temp_dir.path().join("planner-commands.db")).expect("open db");
        migrations::apply(&conn).expect("apply migrations");
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Active save', 1)",
            [],
        )
        .expect("insert active save");
        let active_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO saves (name, is_active) VALUES ('Inactive save', 0)",
            [],
        )
        .expect("insert inactive save");
        let inactive_id = conn.last_insert_rowid();
        let active = save_context(&conn, active_id);
        let inactive = save_context(&conn, inactive_id);
        (temp_dir, conn, active, inactive)
    }

    fn planner_row_counts(db: &Db) -> (i64, i64, i64) {
        let conn = db.read().expect("acquire reader");
        let count = |table: &str| -> i64 {
            conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("count planner rows")
        };
        (
            count("planner_tactic_lanes"),
            count("planner_teams"),
            count("planner_strings"),
        )
    }

    #[test]
    fn planner_reads_answer_on_a_reader_once_the_writer_created_the_defaults() {
        let (_temp_dir, db) = crate::db::open_with_golden_snapshot();
        let context = {
            let conn = db.0.lock().expect("lock writer");
            let context = service::capture_active_save_context(&conn).expect("active save");
            crate::features::managed_club::service::set_managed_club(
                &conn,
                context.id,
                "Loan FC",
                Some(1000),
            )
            .expect("configure managed club");
            let attributes = crate::features::scoring::catalog::DUMP_ATTRIBUTE_KEYS
                .iter()
                .map(|key| ((*key).to_string(), serde_json::Value::from(10)))
                .collect::<serde_json::Map<_, _>>();
            conn.execute(
                "UPDATE players SET attributes_json = ?1 WHERE uid = 77",
                [serde_json::Value::Object(attributes).to_string()],
            )
            .expect("give the golden player every attribute Squad training needs");
            context
        };

        let generation = || db.read().expect("acquire reader").generation();
        let fresh = generation();
        let depth = read_planner_depth(&db).expect("depth creates its defaults on the writer");
        assert_eq!(generation(), fresh + 1, "one write creates every default");
        assert_eq!(planner_row_counts(&db), (11, 3, 5));
        read_planner_depth(&db).expect("depth again");
        assert_eq!(
            generation(),
            fresh + 1,
            "existing defaults are not written again"
        );

        let senior_string = depth.teams[0].strings[0].id;
        depth_service::assign_player(
            &db.0.lock().expect("lock writer"),
            context.id,
            senior_string,
            "centre_forward",
            77,
        )
        .expect("assign the golden player");
        let kept = depth
            .teams
            .iter()
            .filter(|team| team.team != "youth")
            .map(|team| PlannerTeamInput {
                team: team.team.clone(),
                display_name: team.display_name.clone(),
                max_age: team.max_age,
            })
            .collect::<Vec<_>>();
        teams_service::save_team_settings(
            &db.0.lock().expect("lock writer"),
            context.id,
            &kept,
            false,
        )
        .expect("remove the youth team");
        let removed = generation();

        let (tactic, options, depth, reference, candidates, impacts, squad) =
            crate::db::read_while_writer_is_held(&db, move |db| {
                (
                    read_planner_tactic(db, &context),
                    read_planner_tactic_options(db, &context),
                    read_planner_depth(db),
                    read_planner_role_reference(db, "in_possession", "current"),
                    read_planner_slot_candidates(db, "senior", "goalkeeper", ""),
                    read_planner_team_removal_impacts(db, &kept[..1]),
                    read_squad_players(db, None, None, None, None, None),
                )
            });

        assert_eq!(tactic.expect("tactic on a reader").lanes.len(), 11);
        assert!(!options.expect("options on a reader").placements.is_empty());
        assert_eq!(
            depth
                .expect("depth on a reader")
                .teams
                .iter()
                .map(|team| team.team.as_str())
                .collect::<Vec<_>>(),
            vec!["senior", "reserves"],
            "a removed team stays removed"
        );
        assert_eq!(
            reference.expect("role reference on a reader").lanes.len(),
            11
        );
        assert_eq!(
            candidates
                .expect("slot candidates on a reader")
                .iter()
                .map(|candidate| candidate.player_uid)
                .collect::<Vec<_>>(),
            vec![77]
        );
        assert_eq!(
            impacts
                .expect("removal impacts on a reader")
                .iter()
                .map(|impact| impact.team.as_str())
                .collect::<Vec<_>>(),
            vec!["reserves"]
        );
        assert!(squad.expect("Squad on a reader").players[0]
            .suggested_training
            .is_some());
        assert_eq!(generation(), removed);
        assert_eq!(planner_row_counts(&db), (11, 2, 4));

        db.0.lock()
            .expect("lock writer")
            .execute("DELETE FROM planner_tactic_lanes", [])
            .expect("remove the tactic lanes");
        let squad = crate::db::read_while_writer_is_held(&db, |db| {
            read_squad_players(db, None, None, None, None, None)
        })
        .expect("Squad on a reader without tactic lanes");
        assert_eq!(squad.players[0].suggested_training, None);
        assert_eq!(planner_row_counts(&db), (0, 2, 4));
    }

    #[test]
    fn captured_context_commands_accept_inactive_save_and_isolate_persistence() {
        let (_temp_dir, conn, active, inactive) = open_with_two_saves();
        let active_tactic =
            get_planner_tactic_for_context(&conn, &active).expect("read active tactic");
        assert_eq!(active_tactic, tactic_service::default_tactic());
        assert_eq!(
            get_planner_tactic_options_for_context(&conn, &inactive)
                .expect("read inactive options")
                .placements,
            tactic_service::get_tactic_options().placements
        );

        let mut inactive_tactic =
            get_planner_tactic_for_context(&conn, &inactive).expect("read inactive tactic");
        inactive_tactic.lanes[0].ip_weight = 0.25;
        save_planner_tactic_for_context(&conn, &inactive, &inactive_tactic)
            .expect("save inactive tactic");

        assert_eq!(
            get_planner_tactic_for_context(&conn, &inactive).expect("reload inactive tactic"),
            inactive_tactic
        );
        assert_eq!(
            get_planner_tactic_for_context(&conn, &active).expect("reload active tactic"),
            active_tactic
        );
    }

    #[test]
    fn invalid_contexts_are_rejected_before_tactic_persistence_changes() {
        let (_temp_dir, conn, _active, inactive) = open_with_two_saves();
        let persisted =
            get_planner_tactic_for_context(&conn, &inactive).expect("initialize tactic");
        let mut attempted = persisted.clone();
        attempted.lanes[0].ip_weight = 0.25;

        let unknown = service::SaveContext {
            id: 999,
            context_token: "unknown-token".to_string(),
        };
        assert_eq!(
            get_planner_tactic_for_context(&conn, &unknown),
            Err("Save 999 not found".to_string())
        );
        assert_eq!(
            save_planner_tactic_for_context(&conn, &unknown, &attempted),
            Err("Save 999 not found".to_string())
        );
        assert_eq!(
            tactic_service::get_tactic(&conn, inactive.id).expect("read after unknown ID"),
            persisted
        );

        let wrong_token = service::SaveContext {
            id: inactive.id,
            context_token: "wrong-token".to_string(),
        };
        assert_eq!(
            get_planner_tactic_options_for_context(&conn, &wrong_token),
            Err("Save changed or no longer exists".to_string())
        );
        assert_eq!(
            save_planner_tactic_for_context(&conn, &wrong_token, &attempted),
            Err("Save changed or no longer exists".to_string())
        );
        assert_eq!(
            tactic_service::get_tactic(&conn, inactive.id).expect("read after wrong token"),
            persisted
        );
    }

    #[test]
    fn tactic_ipc_round_trips_slot_metrics_and_requires_the_field() {
        let (_temp_dir, conn, active, _inactive) = open_with_two_saves();
        let mut payload = serde_json::to_value(PlannerTacticDto::from(
            get_planner_tactic_for_context(&conn, &active).expect("read tactic"),
        ))
        .expect("serialize tactic");
        payload["lanes"][0]["customMetricKeys"] = serde_json::json!(["save_ratio", "goals"]);

        let input: PlannerTacticInputDto =
            serde_json::from_value(payload.clone()).expect("deserialize tactic input");
        let tactic = PlannerTactic::from(input);
        save_planner_tactic_for_context(&conn, &active, &tactic).expect("save tactic");
        let response = serde_json::to_value(PlannerTacticDto::from(
            get_planner_tactic_for_context(&conn, &active).expect("reload tactic"),
        ))
        .expect("serialize reloaded tactic");

        assert_eq!(
            response["lanes"][0]["customMetricKeys"],
            serde_json::json!(["save_ratio", "goals"])
        );
        assert_eq!(
            response["lanes"][1]["customMetricKeys"],
            serde_json::json!([])
        );

        payload["lanes"][0]
            .as_object_mut()
            .expect("lane object")
            .remove("customMetricKeys");
        let error = serde_json::from_value::<PlannerTacticInputDto>(payload)
            .err()
            .expect("reject a lane without customMetricKeys");
        assert!(error.to_string().contains("customMetricKeys"));
    }

    #[test]
    fn old_token_cannot_write_a_recreated_save_with_the_same_id() {
        let (_temp_dir, conn, _active, old_context) = open_with_two_saves();
        get_planner_tactic_for_context(&conn, &old_context).expect("initialize old tactic");
        conn.execute("DELETE FROM saves WHERE id = ?1", [old_context.id])
            .expect("delete old save");
        conn.execute(
            "INSERT INTO saves (id, name, is_active, context_token) VALUES (?1, 'Recreated save', 0, 'recreated-token')",
            [old_context.id],
        )
        .expect("recreate save");
        let recreated = save_context(&conn, old_context.id);
        let mut recreated_tactic =
            get_planner_tactic_for_context(&conn, &recreated).expect("initialize recreated tactic");
        recreated_tactic.lanes[0].ip_weight = 0.3;
        save_planner_tactic_for_context(&conn, &recreated, &recreated_tactic)
            .expect("save recreated tactic");

        let mut stale_write = recreated_tactic.clone();
        stale_write.lanes[0].ip_weight = 0.8;
        assert_eq!(
            save_planner_tactic_for_context(&conn, &old_context, &stale_write),
            Err("Save changed or no longer exists".to_string())
        );
        assert_eq!(
            tactic_service::get_tactic(&conn, recreated.id).expect("reload recreated tactic"),
            recreated_tactic
        );
    }
}
