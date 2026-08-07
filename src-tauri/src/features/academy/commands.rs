use std::collections::BTreeMap;

use serde::Serialize;
use tauri::State;

use crate::db::Db;
use crate::features::snapshot::service as snapshot_service;

use super::service::{self, AcademyCandidate, AcademyClass, AcademyClassDetail, AcademyMember};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademyClassDto {
    pub id: i64,
    pub class_year: i64,
    pub is_automatic: bool,
    pub member_count: i64,
}

impl From<AcademyClass> for AcademyClassDto {
    fn from(academy_class: AcademyClass) -> Self {
        Self {
            id: academy_class.id,
            class_year: academy_class.class_year,
            is_automatic: academy_class.is_automatic,
            member_count: academy_class.member_count,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademyCandidateDto {
    pub player_uid: i64,
    pub name: String,
    pub age: Option<i64>,
    pub positions: BTreeMap<String, i64>,
    pub current_club: String,
}

impl From<AcademyCandidate> for AcademyCandidateDto {
    fn from(candidate: AcademyCandidate) -> Self {
        Self {
            player_uid: candidate.player_uid,
            name: candidate.name,
            age: candidate.age,
            positions: candidate.positions,
            current_club: candidate.current_club,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademyMemberDto {
    pub player_uid: i64,
    pub last_known_name: String,
    pub current_name: Option<String>,
    pub state: String,
    pub age: Option<i64>,
    pub nationalities: Vec<String>,
    pub positions: BTreeMap<String, i64>,
    pub current_club: Option<String>,
    pub parent_club: Option<String>,
    pub team_level: Option<String>,
    pub pa: Option<i64>,
    pub determination: Option<i64>,
    pub height_cm: Option<i64>,
    pub preferred_foot: Option<String>,
    pub senior_league_appearances: Option<i64>,
    pub goals: Option<i64>,
    pub assists: Option<i64>,
    pub international_caps: Option<i64>,
    pub sale_fee_gbp: Option<i64>,
    pub is_released: Option<bool>,
    pub is_graduate: Option<bool>,
}

impl From<AcademyMember> for AcademyMemberDto {
    fn from(member: AcademyMember) -> Self {
        Self {
            player_uid: member.player_uid,
            last_known_name: member.last_known_name,
            current_name: member.current_name,
            state: member.state.as_str().to_string(),
            age: member.age,
            nationalities: member.nationalities,
            positions: member.positions,
            current_club: member.current_club,
            parent_club: member.parent_club,
            team_level: member.team_level,
            pa: member.pa,
            determination: member.determination,
            height_cm: member.height_cm,
            preferred_foot: member.preferred_foot,
            senior_league_appearances: member.senior_league_appearances,
            goals: member.goals,
            assists: member.assists,
            international_caps: member.international_caps,
            sale_fee_gbp: member.sale_fee_gbp,
            is_released: member.is_released,
            is_graduate: member.is_graduate,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcademyClassDetailDto {
    pub id: i64,
    pub class_year: i64,
    pub is_automatic: bool,
    pub members: Vec<AcademyMemberDto>,
}

impl From<AcademyClassDetail> for AcademyClassDetailDto {
    fn from(detail: AcademyClassDetail) -> Self {
        Self {
            id: detail.id,
            class_year: detail.class_year,
            is_automatic: detail.is_automatic,
            members: detail
                .members
                .into_iter()
                .map(AcademyMemberDto::from)
                .collect(),
        }
    }
}

#[tauri::command]
pub fn list_academy_classes(db: State<'_, Db>) -> Result<Vec<AcademyClassDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = snapshot_service::active_save_id(&conn)?;
    service::list_classes(&conn, save_id)
        .map(|classes| classes.into_iter().map(AcademyClassDto::from).collect())
}

#[tauri::command]
pub fn get_academy_class(
    class_id: i64,
    db: State<'_, Db>,
) -> Result<AcademyClassDetailDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = snapshot_service::active_save_id(&conn)?;
    service::get_class(&conn, save_id, class_id).map(AcademyClassDetailDto::from)
}

#[tauri::command]
pub fn create_academy_class(class_year: i64, db: State<'_, Db>) -> Result<AcademyClassDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = snapshot_service::active_save_id(&conn)?;
    service::create_class(&conn, save_id, class_year).map(AcademyClassDto::from)
}

#[tauri::command]
pub fn delete_academy_class(
    class_id: i64,
    confirmed: bool,
    db: State<'_, Db>,
) -> Result<(), String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = snapshot_service::active_save_id(&conn)?;
    service::delete_class(&conn, save_id, class_id, confirmed)
}

#[tauri::command]
pub fn list_academy_candidates(
    search: String,
    db: State<'_, Db>,
) -> Result<Vec<AcademyCandidateDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = snapshot_service::active_save_id(&conn)?;
    service::list_candidates(&conn, save_id, &search).map(|candidates| {
        candidates
            .into_iter()
            .map(AcademyCandidateDto::from)
            .collect()
    })
}

#[tauri::command]
pub fn assign_academy_member(
    class_id: i64,
    player_uid: i64,
    db: State<'_, Db>,
) -> Result<(), String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = snapshot_service::active_save_id(&conn)?;
    service::assign_member(&conn, save_id, class_id, player_uid)
}

#[tauri::command]
pub fn remove_academy_member(
    class_id: i64,
    player_uid: i64,
    db: State<'_, Db>,
) -> Result<(), String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save_id = snapshot_service::active_save_id(&conn)?;
    service::remove_member(&conn, save_id, class_id, player_uid)
}
