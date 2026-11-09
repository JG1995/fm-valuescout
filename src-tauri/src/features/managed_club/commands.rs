use serde::Serialize;
use tauri::State;

use crate::blocking;
use crate::db::Db;
use crate::features::snapshot::service::active_save_id;

use super::service::{self, ManagedClubAvailability, ManagedClubStatus};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedClubStatusDto {
    pub club_name: Option<String>,
    pub club_uid: Option<i64>,
    pub status: &'static str,
    pub unclassified_player_count: i64,
}

impl From<ManagedClubStatus> for ManagedClubStatusDto {
    fn from(value: ManagedClubStatus) -> Self {
        Self {
            club_name: value.club_name,
            club_uid: value.club_uid,
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
pub async fn get_managed_club(db: State<'_, Db>) -> Result<ManagedClubStatusDto, String> {
    let db = db.inner().clone();
    blocking::run(move || read_managed_club(&db)).await
}

fn read_managed_club(db: &Db) -> Result<ManagedClubStatusDto, String> {
    let conn = db.read()?;
    Ok(service::get_managed_club(&conn, active_save_id(&conn)?)?.into())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedClubOptionDto {
    pub club_name: String,
    pub club_uid: Option<i64>,
}

impl From<service::ManagedClubOption> for ManagedClubOptionDto {
    fn from(value: service::ManagedClubOption) -> Self {
        Self {
            club_name: value.club_name,
            club_uid: value.club_uid,
        }
    }
}

#[tauri::command]
pub async fn list_managed_club_options(
    db: State<'_, Db>,
) -> Result<Vec<ManagedClubOptionDto>, String> {
    let db = db.inner().clone();
    blocking::run(move || read_managed_club_options(&db)).await
}

fn read_managed_club_options(db: &Db) -> Result<Vec<ManagedClubOptionDto>, String> {
    let conn = db.read()?;
    Ok(
        service::list_managed_club_options(&conn, active_save_id(&conn)?)?
            .into_iter()
            .map(ManagedClubOptionDto::from)
            .collect(),
    )
}

#[tauri::command]
pub async fn set_managed_club(
    club_name: String,
    club_uid: Option<i64>,
    db: State<'_, Db>,
) -> Result<ManagedClubStatusDto, String> {
    let db = db.inner().clone();
    blocking::run(move || {
        let conn =
            db.0.lock()
                .map_err(|_| "database lock poisoned".to_string())?;
        Ok(service::set_managed_club(&conn, active_save_id(&conn)?, &club_name, club_uid)?.into())
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_club_reads_answer_on_a_reader() {
        let (_temp_dir, db) = crate::db::open_with_golden_snapshot();
        {
            let conn = db.0.lock().expect("lock writer");
            let save_id = active_save_id(&conn).expect("active save");
            service::set_managed_club(&conn, save_id, "Loan FC", Some(1000))
                .expect("configure managed club");
        }

        let (status, options) = crate::db::read_while_writer_is_held(&db, |db| {
            (read_managed_club(db), read_managed_club_options(db))
        });

        let status = status.expect("managed club on a reader");
        assert_eq!(status.club_name.as_deref(), Some("Loan FC"));
        assert_eq!(status.status, "available");
        assert!(options
            .expect("options on a reader")
            .iter()
            .any(|option| option.club_name == "Loan FC"));
    }
}
