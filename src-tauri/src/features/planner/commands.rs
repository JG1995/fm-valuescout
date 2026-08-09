use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::Db;
use crate::features::snapshot::service;

use super::depth::{
    self as depth_service, PlannerAssignment, PlannerAssignmentLocation, PlannerDepth,
    PlannerDepthTeam, PlannerSlotCandidate, PlannerString, PlannerTeam,
};
use super::optimizer;
use super::service::{self as planner_service, ClubFamily, ClubSourceInput};
use super::tactic::{self as tactic_service, PlannerTactic, TacticLane, TacticOptions};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClubSourceInputDto {
    pub team: String,
    pub club_name: String,
    pub team_level: Option<String>,
}

impl From<ClubSourceInputDto> for ClubSourceInput {
    fn from(input: ClubSourceInputDto) -> Self {
        Self {
            team: input.team,
            club_name: input.club_name,
            team_level: input.team_level,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClubSourceDto {
    pub id: i64,
    pub team: String,
    pub club_name: String,
    pub team_level: Option<String>,
    pub is_primary: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClubFamilyDto {
    pub primary_club: Option<String>,
    pub sources: Vec<ClubSourceDto>,
}

impl From<ClubFamily> for ClubFamilyDto {
    fn from(family: ClubFamily) -> Self {
        Self {
            primary_club: family.primary_club,
            sources: family
                .sources
                .into_iter()
                .map(|source| ClubSourceDto {
                    id: source.id,
                    team: source.team,
                    club_name: source.club_name,
                    team_level: source.team_level,
                    is_primary: source.is_primary,
                })
                .collect(),
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
                })
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
    pub combined_score: Option<u8>,
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
            combined_score: assignment.combined_score,
            potential_combined_score: assignment.potential_combined_score,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannerAssignmentLocationDto {
    pub team: String,
    pub string_id: i64,
    pub string_order: i64,
    pub lane_id: String,
}

impl From<PlannerAssignmentLocation> for PlannerAssignmentLocationDto {
    fn from(location: PlannerAssignmentLocation) -> Self {
        Self {
            team: location.team.as_str().to_string(),
            string_id: location.string_id,
            string_order: location.string_order,
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
    pub ip_score: Option<u8>,
    pub oop_score: Option<u8>,
    pub combined_score: Option<u8>,
    pub assignment_location: Option<PlannerAssignmentLocationDto>,
}

impl From<PlannerSlotCandidate> for PlannerSlotCandidateDto {
    fn from(candidate: PlannerSlotCandidate) -> Self {
        Self {
            player_uid: candidate.player_uid,
            name: candidate.name,
            current_club: candidate.current_club,
            ip_score: candidate.ip_score,
            oop_score: candidate.oop_score,
            combined_score: candidate.combined_score,
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
    pub assignments: Vec<PlannerAssignmentDto>,
}

impl From<PlannerString> for PlannerStringDto {
    fn from(planner_string: PlannerString) -> Self {
        Self {
            id: planner_string.id,
            string_order: planner_string.string_order,
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
    pub strings: Vec<PlannerStringDto>,
}

impl From<PlannerDepthTeam> for PlannerDepthTeamDto {
    fn from(team: PlannerDepthTeam) -> Self {
        Self {
            team: team.team.as_str().to_string(),
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

#[tauri::command]
pub fn get_planner_club_family(db: State<'_, Db>) -> Result<ClubFamilyDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    Ok(planner_service::get_club_family(&conn, save_id)?.into())
}

#[tauri::command]
pub fn list_planner_clubs(db: State<'_, Db>) -> Result<Vec<String>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    planner_service::list_clubs_for_snapshot(&conn, save_id)
}

#[tauri::command]
pub fn save_planner_club_family(
    primary_club: String,
    sources: Vec<ClubSourceInputDto>,
    db: State<'_, Db>,
) -> Result<ClubFamilyDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    let sources = sources
        .into_iter()
        .map(ClubSourceInput::from)
        .collect::<Vec<_>>();
    Ok(planner_service::save_club_family(&conn, save_id, &primary_club, &sources)?.into())
}

#[tauri::command]
pub fn get_planner_tactic(db: State<'_, Db>) -> Result<PlannerTacticDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    Ok(tactic_service::get_tactic(&conn, save_id)?.into())
}

#[tauri::command]
pub fn get_planner_tactic_options() -> TacticOptionsDto {
    tactic_service::get_tactic_options().into()
}

#[tauri::command]
pub fn save_planner_tactic(
    tactic: PlannerTacticInputDto,
    db: State<'_, Db>,
) -> Result<PlannerTacticDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    let tactic = PlannerTactic::from(tactic);
    tactic_service::save_tactic(&conn, save_id, &tactic)?;
    Ok(tactic.into())
}

#[tauri::command]
pub fn get_planner_depth(db: State<'_, Db>) -> Result<PlannerDepthDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    Ok(depth_service::get_depth(&conn, save_id)?.into())
}

#[tauri::command]
pub fn optimize_planner_depth(
    score_basis: String,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let score_basis = optimizer::ScoreBasis::parse(&score_basis)?;
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    Ok(optimizer::optimize_depth_with_basis(&conn, save_id, score_basis)?.into())
}

#[tauri::command]
pub fn get_planner_slot_candidates(
    team: String,
    lane_id: String,
    search: String,
    db: State<'_, Db>,
) -> Result<Vec<PlannerSlotCandidateDto>, String> {
    let team = PlannerTeam::parse(&team)?;
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    depth_service::get_slot_candidates(&conn, save_id, team, &lane_id, &search).map(|candidates| {
        candidates
            .into_iter()
            .map(PlannerSlotCandidateDto::from)
            .collect()
    })
}

#[tauri::command]
pub fn add_planner_string(team: String, db: State<'_, Db>) -> Result<PlannerDepthDto, String> {
    let team = PlannerTeam::parse(&team)?;
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    depth_service::add_string(&conn, save_id, team)?;
    Ok(depth_service::get_depth(&conn, save_id)?.into())
}

#[tauri::command]
pub fn remove_planner_string(
    string_id: i64,
    confirm_populated: bool,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    depth_service::remove_string(&conn, save_id, string_id, confirm_populated)?;
    Ok(depth_service::get_depth(&conn, save_id)?.into())
}

#[tauri::command]
pub fn clear_planner_depth(confirmed: bool, db: State<'_, Db>) -> Result<PlannerDepthDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    depth_service::clear_all(&conn, save_id, confirmed)?;
    Ok(depth_service::get_depth(&conn, save_id)?.into())
}

#[tauri::command]
pub fn clear_planner_assignment(
    string_id: i64,
    lane_id: String,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    depth_service::clear_assignment(&conn, save_id, string_id, &lane_id)?;
    Ok(depth_service::get_depth(&conn, save_id)?.into())
}

#[tauri::command]
pub fn assign_planner_player(
    string_id: i64,
    lane_id: String,
    player_uid: i64,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    depth_service::assign_player(&conn, save_id, string_id, &lane_id, player_uid)?;
    Ok(depth_service::get_depth(&conn, save_id)?.into())
}

#[tauri::command]
pub fn move_planner_player(
    string_id: i64,
    lane_id: String,
    player_uid: i64,
    db: State<'_, Db>,
) -> Result<PlannerDepthDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = service::active_save_id(&conn)?;
    depth_service::move_player(&conn, save_id, string_id, &lane_id, player_uid)?;
    Ok(depth_service::get_depth(&conn, save_id)?.into())
}
