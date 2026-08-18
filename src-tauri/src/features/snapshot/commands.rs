use std::time::Instant;

use serde::Serialize;
use tauri::State;

use crate::db::Db;
use crate::features::memory_read::service::DumpWaitConfig;
use crate::features::player::boost_gate;

use super::ingest::SnapshotSummary;
use super::load_data::{self, LoadDataError, LoadDataResult};
use super::query;
use super::service::{self, SaveDeleteResult, SaveSummary, SnapshotDeleteResult, SnapshotMetadata};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSummaryDto {
    pub id: i64,
    pub context_token: String,
    pub name: String,
    pub is_active: bool,
    pub created_at_utc: String,
    pub updated_at_utc: String,
}

impl From<SaveSummary> for SaveSummaryDto {
    fn from(save: SaveSummary) -> Self {
        Self {
            id: save.id,
            context_token: save.context_token,
            name: save.name,
            is_active: save.is_active,
            created_at_utc: save.created_at_utc,
            updated_at_utc: save.updated_at_utc,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotMetadataDto {
    pub id: i64,
    pub context_token: String,
    pub save_id: i64,
    pub custom_name: Option<String>,
    pub game_date: Option<String>,
    pub game_date_source: String,
    pub player_count: i64,
    pub loaded_at_utc: String,
    pub is_current: bool,
}

impl From<SnapshotMetadata> for SnapshotMetadataDto {
    fn from(snapshot: SnapshotMetadata) -> Self {
        Self {
            id: snapshot.id,
            context_token: snapshot.context_token,
            save_id: snapshot.save_id,
            custom_name: snapshot.custom_name,
            game_date: snapshot.game_date,
            game_date_source: snapshot.game_date_source,
            player_count: snapshot.player_count,
            loaded_at_utc: snapshot.loaded_at_utc,
            is_current: snapshot.is_current,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDeleteResultDto {
    pub deleted_snapshot_id: i64,
    pub save_id: i64,
    pub current_snapshot_id: Option<i64>,
}

impl From<SnapshotDeleteResult> for SnapshotDeleteResultDto {
    fn from(result: SnapshotDeleteResult) -> Self {
        Self {
            deleted_snapshot_id: result.deleted_snapshot_id,
            save_id: result.save_id,
            current_snapshot_id: result.current_snapshot_id,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDeleteResultDto {
    pub deleted_save_id: i64,
    pub deleted_was_active: bool,
    pub active_save: SaveSummaryDto,
}

impl From<SaveDeleteResult> for SaveDeleteResultDto {
    fn from(result: SaveDeleteResult) -> Self {
        Self {
            deleted_save_id: result.deleted_save_id,
            deleted_was_active: result.deleted_was_active,
            active_save: SaveSummaryDto::from(result.active_save),
        }
    }
}

#[tauri::command]
pub fn list_saves(db: State<'_, Db>) -> Result<Vec<SaveSummaryDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let saves = service::list_saves(&conn)?;
    Ok(saves.into_iter().map(SaveSummaryDto::from).collect())
}

#[tauri::command]
pub fn create_save(name: String, db: State<'_, Db>) -> Result<SaveSummaryDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save = service::create_save(&conn, &name)?;
    Ok(SaveSummaryDto::from(save))
}

#[tauri::command]
pub fn rename_save(
    save_id: i64,
    name: String,
    db: State<'_, Db>,
) -> Result<SaveSummaryDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save = service::rename_save(&conn, save_id, &name)?;
    Ok(SaveSummaryDto::from(save))
}

#[tauri::command]
pub fn set_active_save(save_id: i64, db: State<'_, Db>) -> Result<SaveSummaryDto, String> {
    set_active_save_for_command(db.inner(), save_id).map(SaveSummaryDto::from)
}

pub(crate) fn set_active_save_for_command(db: &Db, save_id: i64) -> Result<SaveSummary, String> {
    let _boost_guard = boost_gate::acquire_boost_gate()?;
    let mut conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    service::set_active_save(&mut conn, save_id)
}

#[tauri::command]
pub fn list_snapshots(
    save_id: Option<i64>,
    db: State<'_, Db>,
) -> Result<Vec<SnapshotMetadataDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let snapshots = service::list_snapshot_metadata(&conn, save_id)?;
    Ok(snapshots
        .into_iter()
        .map(SnapshotMetadataDto::from)
        .collect())
}

#[tauri::command]
pub fn rename_snapshot(
    snapshot_id: i64,
    context_token: String,
    custom_name: Option<String>,
    db: State<'_, Db>,
) -> Result<SnapshotMetadataDto, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let snapshot =
        service::rename_snapshot(&conn, snapshot_id, &context_token, custom_name.as_deref())?;
    Ok(SnapshotMetadataDto::from(snapshot))
}

#[tauri::command]
pub fn delete_snapshot(
    snapshot_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<SnapshotDeleteResultDto, String> {
    let _boost_guard = boost_gate::acquire_boost_gate()?;
    let mut conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let result = service::delete_snapshot(&mut conn, snapshot_id, &context_token)?;
    Ok(SnapshotDeleteResultDto::from(result))
}

#[tauri::command]
pub fn delete_save(
    save_id: i64,
    context_token: String,
    db: State<'_, Db>,
) -> Result<SaveDeleteResultDto, String> {
    let _boost_guard = boost_gate::acquire_boost_gate()?;
    let mut conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let result = service::delete_save(&mut conn, save_id, &context_token)?;
    Ok(SaveDeleteResultDto::from(result))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotSummaryDto {
    pub id: i64,
    pub save_id: i64,
    pub schema_version: i64,
    pub generated_at_utc: String,
    pub game_version: String,
    pub supported_game_version: String,
    pub bridge_version: String,
    pub protocol_version: i64,
    pub game_date: Option<String>,
    pub game_date_source: String,
    pub scan_truncated: bool,
    pub max_accepted: Option<i64>,
    pub player_count: i64,
    pub loaded_at_utc: String,
}

impl From<SnapshotSummary> for SnapshotSummaryDto {
    fn from(snapshot: SnapshotSummary) -> Self {
        Self {
            id: snapshot.id,
            save_id: snapshot.save_id,
            schema_version: snapshot.schema_version,
            generated_at_utc: snapshot.generated_at_utc,
            game_version: snapshot.game_version,
            supported_game_version: snapshot.supported_game_version,
            bridge_version: snapshot.bridge_version,
            protocol_version: snapshot.protocol_version,
            game_date: snapshot.game_date,
            game_date_source: snapshot.game_date_source,
            scan_truncated: snapshot.scan_truncated,
            max_accepted: snapshot.max_accepted,
            player_count: snapshot.player_count,
            loaded_at_utc: snapshot.loaded_at_utc,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadDataTimingsDto {
    pub scan_ms: u64,
    pub ingest_ms: u64,
    pub total_ms: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadDataResultDto {
    pub request_id: String,
    pub players_found: Option<i32>,
    pub scan_truncated: Option<bool>,
    pub max_accepted: Option<i32>,
    pub stored_snapshot: SnapshotSummaryDto,
    pub effective_snapshot: SnapshotSummaryDto,
    pub timings: LoadDataTimingsDto,
}

impl From<load_data::LoadDataTimings> for LoadDataTimingsDto {
    fn from(timings: load_data::LoadDataTimings) -> Self {
        Self {
            scan_ms: timings.scan_ms,
            ingest_ms: timings.ingest_ms,
            total_ms: timings.total_ms,
        }
    }
}

impl From<LoadDataResult> for LoadDataResultDto {
    fn from(result: LoadDataResult) -> Self {
        Self {
            request_id: result.request_id,
            players_found: result.players_found,
            scan_truncated: result.scan_truncated,
            max_accepted: result.max_accepted,
            stored_snapshot: SnapshotSummaryDto::from(result.stored_snapshot),
            effective_snapshot: SnapshotSummaryDto::from(result.effective_snapshot),
            timings: LoadDataTimingsDto::from(result.timings),
        }
    }
}

#[tauri::command]
pub fn get_current_snapshot(db: State<'_, Db>) -> Result<Option<SnapshotSummaryDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let snapshot = query::get_current_snapshot(&conn)?;
    Ok(snapshot.map(SnapshotSummaryDto::from))
}

#[tauri::command]
pub fn load_data(
    max_accepted: Option<i32>,
    db: State<'_, Db>,
) -> Result<LoadDataResultDto, LoadDataError> {
    let _boost_guard = boost_gate::acquire_boost_gate().map_err(|message| LoadDataError::Scan {
        kind: "inProgress".to_string(),
        message,
    })?;
    let total_started = Instant::now();
    let save_context = {
        let conn = db.0.lock().map_err(|_| LoadDataError::Scan {
            kind: "internal".to_string(),
            message: "database lock poisoned".to_string(),
        })?;
        service::capture_active_save_context(&conn).map_err(|message| LoadDataError::Scan {
            kind: "internal".to_string(),
            message,
        })?
    };
    let scan_started = Instant::now();
    let (captured_dump_path, dump_result) =
        load_data::scan_dump_from_local_app_data(DumpWaitConfig::default(), max_accepted)?;
    let scan_ms = scan_started.elapsed().as_millis() as u64;
    let ingest_started = Instant::now();
    let mut conn = db.0.lock().map_err(|_| LoadDataError::Scan {
        kind: "internal".to_string(),
        message: "database lock poisoned".to_string(),
    })?;
    let mut result = load_data::load_data_after_scan_with_context(
        &mut conn,
        captured_dump_path.as_ref(),
        dump_result,
        &save_context,
    )?;
    result.timings = load_data::LoadDataTimings {
        scan_ms,
        ingest_ms: ingest_started.elapsed().as_millis() as u64,
        total_ms: total_started.elapsed().as_millis() as u64,
    };
    Ok(LoadDataResultDto::from(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_management_dtos_expose_metadata_and_invalidation_context_only() {
        let snapshot = SnapshotMetadataDto::from(SnapshotMetadata {
            id: 7,
            context_token: "snapshot-token".to_string(),
            save_id: 3,
            custom_name: Some("Pre-season".to_string()),
            game_date: Some("2026-08-01".to_string()),
            game_date_source: "memory".to_string(),
            player_count: 25,
            loaded_at_utc: "2026-08-11T10:00:00.000Z".to_string(),
            is_current: true,
        });
        let snapshot_value = serde_json::to_value(snapshot).expect("serialize snapshot metadata");
        assert_eq!(snapshot_value["contextToken"], "snapshot-token");
        assert_eq!(snapshot_value["isCurrent"], true);
        assert!(snapshot_value.get("players").is_none());

        let deleted = SnapshotDeleteResultDto::from(SnapshotDeleteResult {
            deleted_snapshot_id: 7,
            save_id: 3,
            current_snapshot_id: Some(6),
        });
        let deleted_value = serde_json::to_value(deleted).expect("serialize delete result");
        assert_eq!(deleted_value["deletedSnapshotId"], 7);
        assert_eq!(deleted_value["saveId"], 3);
        assert_eq!(deleted_value["currentSnapshotId"], 6);

        let deleted_save = SaveDeleteResultDto::from(SaveDeleteResult {
            deleted_save_id: 3,
            deleted_was_active: true,
            active_save: SaveSummary {
                id: 4,
                context_token: "fallback-token".to_string(),
                name: "Fallback".to_string(),
                is_active: true,
                created_at_utc: "2026-08-11T10:00:00.000Z".to_string(),
                updated_at_utc: "2026-08-11T10:00:00.000Z".to_string(),
            },
        });
        let deleted_save_value =
            serde_json::to_value(deleted_save).expect("serialize save delete result");
        assert_eq!(deleted_save_value["deletedSaveId"], 3);
        assert_eq!(deleted_save_value["deletedWasActive"], true);
        assert_eq!(deleted_save_value["activeSave"]["id"], 4);
    }
}
