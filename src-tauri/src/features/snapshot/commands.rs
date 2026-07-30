use serde::Serialize;
use tauri::State;

use crate::db::Db;
use crate::features::memory_read::service::DumpWaitConfig;

use super::ingest::SnapshotSummary;
use super::load_data::{self, LoadDataError, LoadDataResult};
use super::query::{self, PlayerSanityRow, DEFAULT_SANITY_LIMIT, MAX_SANITY_LIMIT};
use super::service::{self, active_save_id, SaveSummary};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSummaryDto {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
    pub created_at_utc: String,
    pub updated_at_utc: String,
}

impl From<SaveSummary> for SaveSummaryDto {
    fn from(save: SaveSummary) -> Self {
        Self {
            id: save.id,
            name: save.name,
            is_active: save.is_active,
            created_at_utc: save.created_at_utc,
            updated_at_utc: save.updated_at_utc,
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
    let mut conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let save = service::set_active_save(&mut conn, save_id)?;
    Ok(SaveSummaryDto::from(save))
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
pub struct LoadDataResultDto {
    pub request_id: String,
    pub players_found: Option<i32>,
    pub scan_truncated: Option<bool>,
    pub max_accepted: Option<i32>,
    pub snapshot: SnapshotSummaryDto,
}

impl From<LoadDataResult> for LoadDataResultDto {
    fn from(result: LoadDataResult) -> Self {
        Self {
            request_id: result.request_id,
            players_found: result.players_found,
            scan_truncated: result.scan_truncated,
            max_accepted: result.max_accepted,
            snapshot: SnapshotSummaryDto::from(result.snapshot),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerSanityRowDto {
    pub name: String,
    pub ca: i64,
    pub club: Option<String>,
}

impl From<PlayerSanityRow> for PlayerSanityRowDto {
    fn from(row: PlayerSanityRow) -> Self {
        Self {
            name: row.name,
            ca: row.ca,
            club: row.club,
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
pub fn list_sanity_players(
    limit: Option<u32>,
    db: State<'_, Db>,
) -> Result<Vec<PlayerSanityRowDto>, String> {
    let conn =
        db.0.lock()
            .map_err(|_| "database lock poisoned".to_string())?;
    let limit = limit
        .map(|value| value as usize)
        .unwrap_or(DEFAULT_SANITY_LIMIT)
        .min(MAX_SANITY_LIMIT);
    let players = query::list_sanity_players(&conn, limit)?;
    Ok(players.into_iter().map(PlayerSanityRowDto::from).collect())
}

#[tauri::command]
pub fn load_data(
    max_accepted: Option<i32>,
    db: State<'_, Db>,
) -> Result<LoadDataResultDto, LoadDataError> {
    let save_id = {
        let conn = db.0.lock().map_err(|_| LoadDataError::Scan {
            kind: "internal".to_string(),
            message: "database lock poisoned".to_string(),
        })?;
        active_save_id(&conn).map_err(|message| LoadDataError::Scan {
            kind: "internal".to_string(),
            message,
        })?
    };
    let (bridge_directory, dump_result) =
        load_data::scan_dump_from_local_app_data(DumpWaitConfig::default(), max_accepted)?;
    let mut conn = db.0.lock().map_err(|_| LoadDataError::Scan {
        kind: "internal".to_string(),
        message: "database lock poisoned".to_string(),
    })?;
    let result =
        load_data::load_data_after_scan(&mut conn, &bridge_directory, dump_result, save_id)?;
    Ok(LoadDataResultDto::from(result))
}
