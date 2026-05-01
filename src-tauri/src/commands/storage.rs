use tauri::State;

use crate::storage::{
    self, DbState, ImportResult, PlayerSeasonData, Save, Season,
};
use crate::parser::types::ParsedPlayer;

// ── Save management ────────────────────────────────────────────────────

#[tauri::command]
pub fn create_save(state: State<DbState>, name: String) -> Result<Save, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::create_save(&conn, &name).map_err(|e| e.into())
}

#[tauri::command]
pub fn list_saves(state: State<DbState>) -> Result<Vec<Save>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::list_saves(&conn).map_err(|e| e.into())
}

#[tauri::command]
pub fn rename_save(state: State<DbState>, save_id: i64, name: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::rename_save(&conn, save_id, &name).map_err(|e| e.into())
}

#[tauri::command]
pub fn delete_save(state: State<DbState>, save_id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::delete_save(&conn, save_id).map_err(|e| e.into())
}

// ── Season management ──────────────────────────────────────────────────

#[tauri::command]
pub fn list_seasons(state: State<DbState>, save_id: i64) -> Result<Vec<Season>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::list_seasons(&conn, save_id).map_err(|e| e.into())
}

#[tauri::command]
pub fn rename_season(state: State<DbState>, season_id: i64, name: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::rename_season(&conn, season_id, &name).map_err(|e| e.into())
}

#[tauri::command]
pub fn delete_season(state: State<DbState>, season_id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::delete_season(&conn, season_id).map_err(|e| e.into())
}

// ── Import ─────────────────────────────────────────────────────────────

#[tauri::command]
pub fn import_season(
    state: State<DbState>,
    save_id: i64,
    players: Vec<ParsedPlayer>,
    in_game_date: String,
) -> Result<ImportResult, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::import_season(&conn, save_id, &in_game_date, players).map_err(|e| e.into())
}

// ── Data retrieval ─────────────────────────────────────────────────────

#[tauri::command]
pub fn get_players_for_season(
    state: State<DbState>,
    season_id: i64,
) -> Result<Vec<PlayerSeasonData>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::get_players_for_season(&conn, season_id).map_err(|e| e.into())
}

#[tauri::command]
pub fn get_player_career(
    state: State<DbState>,
    save_id: i64,
    player_id: i64,
) -> Result<Vec<PlayerSeasonData>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::get_player_career(&conn, save_id, player_id).map_err(|e| e.into())
}

#[tauri::command]
pub fn get_latest_season(
    state: State<DbState>,
    save_id: i64,
) -> Result<Option<Season>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::get_latest_season(&conn, save_id).map_err(|e| e.into())
}
