use tauri::State;
use rusqlite::Connection;

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
    use super::*;
    use tauri::test::{mock_builder, mock_context, noop_assets, get_ipc_response, INVOKE_KEY};
    use tauri::webview::InvokeRequest;

    // ── Test helpers ───────────────────────────────────────────────────

    /// Create a fresh in-memory database with schema, wrapped in DbState.
    fn setup_test_state() -> DbState {
        let conn = Connection::open_in_memory().unwrap();
        storage::init_db_test(&conn).unwrap();
        DbState {
            conn: std::sync::Arc::new(std::sync::Mutex::new(conn)),
        }
    }

    /// Construct a MetricWeight with inverted=false.
    fn test_metric(key: &str, weight: f64) -> MetricWeight {
        MetricWeight {
            metric_key: key.to_string(),
            weight,
            inverted: false,
        }
    }

    /// Build a test app with the given DbState and return the webview window.
    fn build_test_app(state: DbState) -> tauri::WebviewWindow<tauri::test::MockRuntime> {
        let app = mock_builder()
            .manage(state)
            .invoke_handler(tauri::generate_handler![
                create_archetype_cmd,
                list_archetypes_by_role,
                list_all_archetypes_cmd,
                get_archetype_cmd,
                update_archetype_cmd,
                delete_archetype_cmd,
            ])
            .build(mock_context(noop_assets()))
            .expect("failed to build test app");

        tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("failed to build webview")
    }

    /// Call a command via IPC and return the deserialized response.
    fn invoke_cmd<T: serde::de::DeserializeOwned>(
        webview: &impl AsRef<tauri::Webview<tauri::test::MockRuntime>>,
        cmd: &str,
        args: serde_json::Value,
    ) -> Result<T, String> {
        let request = InvokeRequest {
            cmd: cmd.to_string(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: "http://tauri.localhost".parse().unwrap(),
            body: args.into(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        };

        let response = get_ipc_response(webview, request)
            .map_err(|e| serde_json::from_value::<String>(e.clone()).unwrap_or_else(|_| e.to_string()))?;

        response.deserialize().map_err(|e| e.to_string())
    }

    // ── create_archetype_cmd ────────────────────────────────────────────

    #[test]
    fn create_archetype_basic() {
        let state = setup_test_state();
        let webview = build_test_app(state);
        let metrics = vec![test_metric("goals_per_90", 0.6), test_metric("assists_per_90", 0.4)];

        let result: Archetype = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": "Goal Poacher", "role": "ST", "metrics": metrics.clone() })).unwrap();

        assert!(result.id > 0);
        assert_eq!(result.name, "Goal Poacher");
        assert_eq!(result.role, "ST");
        assert_eq!(result.metrics.len(), 2);
        assert!(!result.is_default);
        // Weights normalized to sum to 1.0
        let sum: f64 = result.metrics.iter().map(|m| m.weight).sum::<f64>();
        assert!((sum - 1.0).abs() < 1e-9, "weights should normalize to 1.0, got {}", sum);
    }

    #[test]
    fn create_archetype_empty_name_rejected() {
        let state = setup_test_state();
        let webview = build_test_app(state);
        let metrics = vec![test_metric("goals_per_90", 1.0)];

        let result: Result<Archetype, String> = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": "", "role": "ST", "metrics": metrics.clone() }));

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_lowercase().contains("cannot be empty"), "expected 'cannot be empty', got: {}", err);
    }

    #[test]
    fn create_archetype_duplicate_rejected() {
        let state = setup_test_state();
        let webview = build_test_app(state);
        let metrics = vec![test_metric("goals_per_90", 1.0)];

        let _: Archetype = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": "Poacher", "role": "ST", "metrics": metrics.clone() })).unwrap();

        let result: Result<Archetype, String> = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": "Poacher", "role": "ST", "metrics": metrics.clone() }));
        assert!(result.is_err(), "duplicate (name, role) should be rejected");
    }

    // ── list_archetypes_by_role ─────────────────────────────────────────

    #[test]
    fn list_archetypes_by_role_returns_matching() {
        let state = setup_test_state();
        let webview = build_test_app(state);
        let metrics = vec![test_metric("tackles_per_90", 1.0)];

        for (name, role) in [
            ("CB Anchor", "D"),
            ("Ball-Winning Mid", "DM"),
            ("FB Bomber", "D"),
        ] {
            let _: Archetype = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": name, "role": role, "metrics": metrics.clone() })).unwrap();
        }

        let result: Vec<Archetype> = invoke_cmd(&webview, "list_archetypes_by_role", serde_json::json!({ "role": "D" })).unwrap();

        assert_eq!(result.len(), 2, "should return 2 D archetypes");
        assert!(result.iter().all(|a| a.role == "D"), "all returned archetypes should have role D");
    }

    #[test]
    fn list_archetypes_by_role_empty_when_no_match() {
        let state = setup_test_state();
        let webview = build_test_app(state);

        let result: Vec<Archetype> = invoke_cmd(&webview, "list_archetypes_by_role", serde_json::json!({ "role": "GK" })).unwrap();

        assert!(result.is_empty(), "should return empty vec when no archetypes for role");
    }

    // ── list_all_archetypes_cmd ──────────────────────────────────────────

    #[test]
    fn list_all_archetypes_returns_all() {
        let state = setup_test_state();
        let webview = build_test_app(state);
        let metrics = vec![test_metric("crossing_per_90", 1.0)];

        for (name, role) in [
            ("Classic WB", "WB"),
            ("Target Forward", "ST"),
            ("Deep lying Playmaker", "DM"),
        ] {
            let _: Archetype = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": name, "role": role, "metrics": metrics.clone() })).unwrap();
        }

        let result: Vec<Archetype> = invoke_cmd(&webview, "list_all_archetypes_cmd", serde_json::json!({})).unwrap();

        assert_eq!(result.len(), 3, "should return all 3 archetypes");
    }

    #[test]
    fn list_all_archetypes_empty_when_none_exist() {
        let state = setup_test_state();
        let webview = build_test_app(state);

        let result: Vec<Archetype> = invoke_cmd(&webview, "list_all_archetypes_cmd", serde_json::json!({})).unwrap();

        assert!(result.is_empty(), "should return empty vec when no archetypes exist");
    }

    // ── get_archetype_cmd ───────────────────────────────────────────────

    #[test]
    fn get_archetype_returns_existing() {
        let state = setup_test_state();
        let webview = build_test_app(state);
        let metrics = vec![test_metric("clean_sheets_per_90", 1.0)];

        let created: Archetype = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": "GK Sweeper", "role": "GK", "metrics": metrics.clone() })).unwrap();

        let result: Archetype = invoke_cmd(&webview, "get_archetype_cmd", serde_json::json!({ "id": created.id })).unwrap();

        assert_eq!(result.id, created.id);
        assert_eq!(result.name, "GK Sweeper");
        assert_eq!(result.role, "GK");
    }

    #[test]
    fn get_archetype_not_found() {
        let state = setup_test_state();
        let webview = build_test_app(state);

        let result: Result<Archetype, String> = invoke_cmd(&webview, "get_archetype_cmd", serde_json::json!({ "id": 9999i64 }));

        assert!(result.is_err(), "invalid id should return error");
        let err = result.unwrap_err();
        assert!(err.contains("not found"), "error should mention 'not found', got: {}", err);
    }

    // ── update_archetype_cmd ────────────────────────────────────────────

    #[test]
    fn update_archetype_basic() {
        let state = setup_test_state();
        let webview = build_test_app(state);
        let metrics = vec![test_metric("key_passes_per_90", 1.0)];

        let created: Archetype = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": "Old Name", "role": "AM", "metrics": metrics.clone() })).unwrap();

        let new_metrics = vec![test_metric("dribbles_per_90", 0.7), test_metric("goals_per_90", 0.3)];
        let updated: Archetype = invoke_cmd(&webview, "update_archetype_cmd", serde_json::json!({ "id": created.id, "name": "New Name", "metrics": new_metrics.clone() })).unwrap();

        assert_eq!(updated.id, created.id);
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.role, "AM"); // role preserved
        assert_eq!(updated.metrics.len(), 2);
    }

    #[test]
    fn update_archetype_not_found() {
        let state = setup_test_state();
        let webview = build_test_app(state);
        let metrics = vec![test_metric("passes_per_90", 1.0)];

        let result: Result<Archetype, String> = invoke_cmd(&webview, "update_archetype_cmd", serde_json::json!({ "id": 9999i64, "name": "Fake", "metrics": metrics.clone() }));

        assert!(result.is_err(), "invalid id should return error");
        let err = result.unwrap_err();
        assert!(err.contains("not found"), "error should mention 'not found', got: {}", err);
    }

    // ── delete_archetype_cmd ────────────────────────────────────────────

    #[test]
    fn delete_archetype_basic() {
        let state = setup_test_state();
        let webview = build_test_app(state);
        let metrics = vec![test_metric("tackles_per_90", 1.0)];

        // Create 2 archetypes for same role so deletion is allowed
        let a1: Archetype = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": "Defender A", "role": "D", "metrics": metrics.clone() })).unwrap();

        let _: Archetype = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": "Defender B", "role": "D", "metrics": metrics.clone() })).unwrap();

        let result: Result<(), String> = invoke_cmd(&webview, "delete_archetype_cmd", serde_json::json!({ "id": a1.id }));
        assert!(result.is_ok(), "should delete archetype: {:?}", result);

        // Verify it's gone
        let get_result: Result<Archetype, String> = invoke_cmd(&webview, "get_archetype_cmd", serde_json::json!({ "id": a1.id }));
        assert!(get_result.is_err(), "deleted archetype should not be fetchable");
    }

    #[test]
    fn delete_archetype_not_found() {
        let state = setup_test_state();
        let webview = build_test_app(state);

        let result: Result<(), String> = invoke_cmd(&webview, "delete_archetype_cmd", serde_json::json!({ "id": 9999i64 }));

        assert!(result.is_err(), "invalid id should return error");
        let err = result.unwrap_err();
        assert!(err.contains("not found"), "error should mention 'not found', got: {}", err);
    }

    #[test]
    fn delete_archetype_last_for_role_rejected() {
        let state = setup_test_state();
        let webview = build_test_app(state);
        let metrics = vec![test_metric("tackles_per_90", 1.0)];

        // Create only one archetype for this role
        let a1: Archetype = invoke_cmd(&webview, "create_archetype_cmd", serde_json::json!({ "name": "Only DM", "role": "DM", "metrics": metrics.clone() })).unwrap();

        // Attempt to delete the only archetype for this role — should fail
        let result: Result<(), String> = invoke_cmd(&webview, "delete_archetype_cmd", serde_json::json!({ "id": a1.id }));

        assert!(result.is_err(), "deleting last archetype for role should be rejected");
        let err = result.unwrap_err();
        assert!(err.to_lowercase().contains("last archetype"), "error should mention 'last archetype', got: {}", err);
    }

    // ── Compilation test (preserved) ────────────────────────────────────

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
