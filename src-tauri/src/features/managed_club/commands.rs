use serde::Serialize;
use tauri::State;

use crate::db::Db;
use crate::features::snapshot::service::active_save_id;

use super::service::{self, ManagedClubAvailability, ManagedClubStatus};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedClubStatusDto {
    pub club_name: Option<String>,
    pub status: &'static str,
    pub unclassified_player_count: i64,
}

impl From<ManagedClubStatus> for ManagedClubStatusDto {
    fn from(value: ManagedClubStatus) -> Self {
        Self {
            club_name: value.club_name,
            status: match value.availability {
                ManagedClubAvailability::Unconfigured => "unconfigured",
                ManagedClubAvailability::Available => "available",
                ManagedClubAvailability::Missing => "missing",
            },
            unclassified_player_count: value.unclassified_player_count,
        }
    }
}

#[tauri::command]
pub fn get_managed_club(db: State<'_, Db>) -> Result<ManagedClubStatusDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    Ok(service::get_managed_club(&conn, active_save_id(&conn)?)?.into())
}

#[tauri::command]
pub fn list_managed_club_options(db: State<'_, Db>) -> Result<Vec<String>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    service::list_managed_club_options(&conn, active_save_id(&conn)?)
}

#[tauri::command]
pub fn set_managed_club(
    club_name: String,
    db: State<'_, Db>,
) -> Result<ManagedClubStatusDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    Ok(service::set_managed_club(&conn, active_save_id(&conn)?, &club_name)?.into())
}
