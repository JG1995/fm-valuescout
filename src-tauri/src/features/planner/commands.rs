use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::Db;
use crate::features::snapshot::service;

use super::service::{self as planner_service, ClubFamily, ClubSourceInput};

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
