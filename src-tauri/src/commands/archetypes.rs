use tauri::State;

use crate::storage::{self, DbState, Archetype, MetricWeight};

// ── Archetype management ──────────────────────────────────────────────

#[tauri::command]
pub fn create_archetype_cmd(
    state: State<DbState>,
    name: String,
    role: String,
    metrics: Vec<MetricWeight>,
) -> Result<Archetype, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::create_archetype(&conn, &name, &role, &metrics).map_err(|e| e.into())
}

#[tauri::command]
pub fn list_archetypes_by_role(
    state: State<DbState>,
    role: String,
) -> Result<Vec<Archetype>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::list_archetypes(&conn, &role).map_err(|e| e.into())
}

#[tauri::command]
pub fn list_all_archetypes_cmd(state: State<DbState>) -> Result<Vec<Archetype>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::list_all_archetypes(&conn).map_err(|e| e.into())
}

#[tauri::command]
pub fn get_archetype_cmd(state: State<DbState>, id: i64) -> Result<Archetype, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::get_archetype(&conn, id).map_err(|e| e.into())
}

#[tauri::command]
pub fn update_archetype_cmd(
    state: State<DbState>,
    id: i64,
    name: String,
    metrics: Vec<MetricWeight>,
) -> Result<Archetype, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::update_archetype(&conn, id, &name, &metrics).map_err(|e| e.into())
}

#[tauri::command]
pub fn delete_archetype_cmd(state: State<DbState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::delete_archetype(&conn, id).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    // Command wrappers are thin wrappers around storage functions.
    // The storage layer (storage::archetypes) has comprehensive tests.
    // These tests just verify the command functions exist and compile.

    #[test]
    fn test_archetype_commands_compile() {
        // Verify all 6 command functions are defined with correct signatures
        let _create: fn(tauri::State<crate::storage::DbState>, String, String, Vec<crate::storage::MetricWeight>) -> Result<crate::storage::Archetype, String> = super::create_archetype_cmd;
        let _list_by_role: fn(tauri::State<crate::storage::DbState>, String) -> Result<Vec<crate::storage::Archetype>, String> = super::list_archetypes_by_role;
        let _list_all: fn(tauri::State<crate::storage::DbState>) -> Result<Vec<crate::storage::Archetype>, String> = super::list_all_archetypes_cmd;
        let _get: fn(tauri::State<crate::storage::DbState>, i64) -> Result<crate::storage::Archetype, String> = super::get_archetype_cmd;
        let _update: fn(tauri::State<crate::storage::DbState>, i64, String, Vec<crate::storage::MetricWeight>) -> Result<crate::storage::Archetype, String> = super::update_archetype_cmd;
        let _delete: fn(tauri::State<crate::storage::DbState>, i64) -> Result<(), String> = super::delete_archetype_cmd;
    }
}
