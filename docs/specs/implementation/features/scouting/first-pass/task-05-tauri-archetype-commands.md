# Task 05 - Tauri Archetype Commands

## Overview

Create Tauri command wrappers for archetype CRUD operations. These are the IPC bridge between the Svelte frontend and the Rust storage layer. Follow the exact pattern from `src-tauri/src/commands/storage.rs`.

## Files to Create/Modify

- Create: `src-tauri/src/commands/archetypes.rs` — Tauri command wrappers with tests
- Modify: `src-tauri/src/commands/mod.rs` — Add `pub mod archetypes;`
- Modify: `src-tauri/src/lib.rs` — Register commands in `invoke_handler`

## Context

### Existing Command Pattern (from `src-tauri/src/commands/storage.rs`)

Every command follows this pattern:

```rust
use tauri::State;
use crate::storage::{DbState, ...};

#[tauri::command]
pub fn command_name(state: State<DbState>, /* params */) -> Result<ReturnType, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    storage::function_name(&conn, /* params */).map_err(|e| e.into())
}
```

Key details:
- Takes `State<DbState>` as first param
- Locks the Mutex to get a connection reference
- Delegates to the storage function
- Converts `StorageError` to `String` via `.map_err(|e| e.into())`

### Test Pattern (from `src-tauri/src/storage/mod.rs`)

Tests use an in-memory SQLite DB:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use super::schema::init_schema;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_name() {
        let conn = setup_test_db();
        // ... test logic
    }
}
```

### Types Available for IPC

From `src-tauri/src/storage/`:
- `Archetype` — needs to be `Serialize, Deserialize` (from storage types)
- `MetricWeight` — needs to be `Serialize, Deserialize` (from storage types)

The frontend will send/receive these as JSON.

## Steps

- [ ] **Step 1: Write failing integration tests**

Create `src-tauri/src/commands/archetypes.rs` with a `#[cfg(test)] mod tests` block containing failing tests. The tests call the command functions directly (not via IPC):

```rust
// src-tauri/src/commands/archetypes.rs

use tauri::State;

use crate::storage::{
    DbState, Archetype, MetricWeight,
    create_archetype, list_archetypes, list_all_archetypes,
    get_archetype, update_archetype, delete_archetype,
};

#[tauri::command]
pub fn create_archetype_cmd(
    state: State<DbState>,
    name: String,
    role: String,
    metrics: Vec<MetricWeight>,
) -> Result<Archetype, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    create_archetype(&conn, &name, &role, &metrics).map_err(|e| e.into())
}

#[tauri::command]
pub fn list_archetypes_by_role(
    state: State<DbState>,
    role: String,
) -> Result<Vec<Archetype>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    list_archetypes(&conn, &role).map_err(|e| e.into())
}

#[tauri::command]
pub fn list_all_archetypes_cmd(
    state: State<DbState>,
) -> Result<Vec<Archetype>, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    list_all_archetypes(&conn).map_err(|e| e.into())
}

#[tauri::command]
pub fn get_archetype_cmd(
    state: State<DbState>,
    id: i64,
) -> Result<Archetype, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    get_archetype(&conn, id).map_err(|e| e.into())
}

#[tauri::command]
pub fn update_archetype_cmd(
    state: State<DbState>,
    id: i64,
    name: String,
    metrics: Vec<MetricWeight>,
) -> Result<Archetype, String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    update_archetype(&conn, id, &name, &metrics).map_err(|e| e.into())
}

#[tauri::command]
pub fn delete_archetype_cmd(
    state: State<DbState>,
    id: i64,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| e.to_string())?;
    delete_archetype(&conn, id).map_err(|e| e.into())
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::types::DbState;
    use rusqlite::Connection;
    use std::sync::Mutex;

    fn setup_test_state() -> DbState {
        let conn = Connection::open_in_memory().unwrap();
        crate::storage::schema::init_schema(&conn).unwrap();
        DbState { conn: Mutex::new(conn) }
    }

    // Helper: create a MetricWeight for testing
    fn test_metric(name: &str, weight: f64) -> MetricWeight {
        MetricWeight {
            name: name.to_string(),
            weight,
        }
    }

    // Helper: wrap a DbState in tauri::State
    fn state_wrapper(state: &DbState) -> State<DbState> {
        State::new(state.clone())
    }

    // ── create_archetype_cmd tests ──────────────────────────────────────

    #[test]
    fn create_archetype_cmd_basic() {
        let state = setup_test_state();
        let metrics = vec![test_metric("pace", 0.8), test_metric("shooting", 0.6)];

        let result = create_archetype_cmd(
            state_wrapper(&state),
            "Goal Hunter".to_string(),
            "Striker".to_string(),
            metrics,
        );

        let archetype = result.unwrap();
        assert_eq!(archetype.name, "Goal Hunter");
        assert_eq!(archetype.role, "Striker");
        assert_eq!(archetype.metrics.len(), 2);
        assert!(archetype.id > 0);
    }

    #[test]
    fn create_archetype_cmd_empty_name_rejected() {
        let state = setup_test_state();
        let metrics = vec![test_metric("pace", 0.8)];

        let result = create_archetype_cmd(
            state_wrapper(&state),
            "".to_string(),
            "Striker".to_string(),
            metrics,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("empty")
            || result.unwrap_err().to_lowercase().contains("name"));
    }

    #[test]
    fn create_archetype_cmd_duplicate_rejected() {
        let state = setup_test_state();
        let metrics = vec![test_metric("pace", 0.8)];

        create_archetype_cmd(
            state_wrapper(&state),
            "Unique Archetype".to_string(),
            "Midfielder".to_string(),
            metrics.clone(),
        ).unwrap();

        let result = create_archetype_cmd(
            state_wrapper(&state),
            "Unique Archetype".to_string(),
            "Midfielder".to_string(),
            metrics,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("exist")
            || result.unwrap_err().to_lowercase().contains("duplicate"));
    }

    // ── list_archetypes_by_role tests ────────────────────────────────────

    #[test]
    fn list_archetypes_by_role_returns_matching() {
        let state = setup_test_state();
        let metrics = vec![test_metric("pace", 0.8)];

        create_archetype_cmd(
            state_wrapper(&state),
            "Quick Winger".to_string(),
            "Winger".to_string(),
            metrics.clone(),
        ).unwrap();

        create_archetype_cmd(
            state_wrapper(&state),
            "Slow Defender".to_string(),
            "Defender".to_string(),
            metrics.clone(),
        ).unwrap();

        let result = list_archetypes_by_role(state_wrapper(&state), "Winger".to_string());
        let archetypes = result.unwrap();

        assert_eq!(archetypes.len(), 1);
        assert_eq!(archetypes[0].name, "Quick Winger");
    }

    #[test]
    fn list_archetypes_by_role_empty_when_no_match() {
        let state = setup_test_state();
        let metrics = vec![test_metric("pace", 0.8)];

        create_archetype_cmd(
            state_wrapper(&state),
            "Some Archetype".to_string(),
            "Striker".to_string(),
            metrics,
        ).unwrap();

        let result = list_archetypes_by_role(state_wrapper(&state), "Goalkeeper".to_string());
        let archetypes = result.unwrap();

        assert!(archetypes.is_empty());
    }

    // ── list_all_archetypes_cmd tests ───────────────────────────────────

    #[test]
    fn list_all_archetypes_cmd_returns_all() {
        let state = setup_test_state();
        let metrics = vec![test_metric("pace", 0.8)];

        create_archetype_cmd(
            state_wrapper(&state),
            "Arch A".to_string(),
            "Role A".to_string(),
            metrics.clone(),
        ).unwrap();

        create_archetype_cmd(
            state_wrapper(&state),
            "Arch B".to_string(),
            "Role B".to_string(),
            metrics,
        ).unwrap();

        let result = list_all_archetypes_cmd(state_wrapper(&state));
        let archetypes = result.unwrap();

        assert_eq!(archetypes.len(), 2);
    }

    #[test]
    fn list_all_archetypes_cmd_empty_when_none_exist() {
        let state = setup_test_state();
        let result = list_all_archetypes_cmd(state_wrapper(&state));
        assert!(result.unwrap().is_empty());
    }

    // ── get_archetype_cmd tests ─────────────────────────────────────────

    #[test]
    fn get_archetype_cmd_returns_existing() {
        let state = setup_test_state();
        let metrics = vec![test_metric("pace", 0.8)];

        let created = create_archetype_cmd(
            state_wrapper(&state),
            "Target Archetype".to_string(),
            "Target Role".to_string(),
            metrics,
        ).unwrap();

        let result = get_archetype_cmd(state_wrapper(&state), created.id);
        let archetype = result.unwrap();

        assert_eq!(archetype.id, created.id);
        assert_eq!(archetype.name, "Target Archetype");
    }

    #[test]
    fn get_archetype_cmd_not_found() {
        let state = setup_test_state();
        let result = get_archetype_cmd(state_wrapper(&state), 9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("not found")
            || result.unwrap_err().to_lowercase().contains("does not exist"));
    }

    // ── update_archetype_cmd tests ───────────────────────────────────────

    #[test]
    fn update_archetype_cmd_basic() {
        let state = setup_test_state();
        let metrics = vec![test_metric("pace", 0.8)];

        let created = create_archetype_cmd(
            state_wrapper(&state),
            "Old Name".to_string(),
            "Role".to_string(),
            metrics.clone(),
        ).unwrap();

        let new_metrics = vec![test_metric("shooting", 0.9)];
        let result = update_archetype_cmd(
            state_wrapper(&state),
            created.id,
            "New Name".to_string(),
            new_metrics,
        );

        let updated = result.unwrap();
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.metrics.len(), 1);
        assert_eq!(updated.metrics[0].name, "shooting");
    }

    #[test]
    fn update_archetype_cmd_not_found() {
        let state = setup_test_state();
        let metrics = vec![test_metric("pace", 0.8)];

        let result = update_archetype_cmd(
            state_wrapper(&state),
            9999,
            "Name".to_string(),
            metrics,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("not found")
            || result.unwrap_err().to_lowercase().contains("does not exist"));
    }

    // ── delete_archetype_cmd tests ──────────────────────────────────────

    #[test]
    fn delete_archetype_cmd_basic() {
        let state = setup_test_state();
        let metrics = vec![test_metric("pace", 0.8)];

        let created = create_archetype_cmd(
            state_wrapper(&state),
            "To Delete".to_string(),
            "Role".to_string(),
            metrics,
        ).unwrap();

        let result = delete_archetype_cmd(state_wrapper(&state), created.id);
        result.unwrap();

        // Verify it's gone
        let get_result = get_archetype_cmd(state_wrapper(&state), created.id);
        assert!(get_result.is_err());
    }

    #[test]
    fn delete_archetype_cmd_not_found() {
        let state = setup_test_state();
        let result = delete_archetype_cmd(state_wrapper(&state), 9999);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_lowercase().contains("not found")
            || result.unwrap_err().to_lowercase().contains("does not exist"));
    }
}
```

Note: The test block references `DbState::clone()` — you may need to implement `Clone` for `DbState` or refactor the test helper to work around it.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test --lib commands::archetypes`
Expected: **FAIL** — compilation errors because:
- `archetypes` module doesn't exist in commands
- `crate::storage::schema::init_schema` may not be pub
- Storage functions (`create_archetype`, etc.) don't exist yet
- `DbState` may not implement `Clone`

This confirms the tests are testing against missing code.

- [ ] **Step 3: Implement the commands**

Write the 6 command functions as shown in Step 1. The implementations are already included above — just ensure the file is complete with the test block.

- [ ] **Step 4: Register module in mod.rs**

In `src-tauri/src/commands/mod.rs`, change:

```rust
pub mod csv_parser;
pub mod storage;
```

to:

```rust
pub mod csv_parser;
pub mod storage;
pub mod archetypes;
```

- [ ] **Step 5: Register commands in invoke_handler**

In `src-tauri/src/lib.rs`, add the archetype commands to the `invoke_handler` macro. The full `invoke_handler` should be:

```rust
.invoke_handler(tauri::generate_handler![
    commands::csv_parser::parse_csv,
    commands::storage::create_save,
    commands::storage::list_saves,
    commands::storage::rename_save,
    commands::storage::delete_save,
    commands::storage::list_seasons,
    commands::storage::rename_season,
    commands::storage::delete_season,
    commands::storage::import_season,
    commands::storage::get_players_for_season,
    commands::storage::get_player_career,
    commands::storage::get_latest_season,
    commands::archetypes::create_archetype_cmd,
    commands::archetypes::list_archetypes_by_role,
    commands::archetypes::list_all_archetypes_cmd,
    commands::archetypes::get_archetype_cmd,
    commands::archetypes::update_archetype_cmd,
    commands::archetypes::delete_archetype_cmd,
])
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd src-tauri && cargo test --lib commands::archetypes`
Expected: **PASS** — all 11 integration tests pass.

- [ ] **Step 7: Run full test suite**

Run: `cd src-tauri && cargo test --lib`
Expected: **ALL PASS** — no regressions in existing tests.

## Dependencies

- Task 02 (archetype types) — `Archetype`, `MetricWeight` types defined in storage
- Task 03 (archetype CRUD) — Storage functions to wrap (`create_archetype`, etc.)
- Task 04 (seed data) — Not strictly required for compilation, but startup will seed data

## Success Criteria

- All 6 commands compile and register
- All 11 integration tests pass (create, list by role, list all, get, update, delete × happy/error paths)
- `cargo test --lib commands::archetypes` passes
- All existing tests still pass
- Frontend can call these commands via `invoke("command_name", { params })`

## Tests

### Test 1: Compilation

**What to test:** All commands compile without errors.
**Feasibility:** ✅ Can be tested — `cargo check`.

### Test 2: Command Integration Tests

**What to test:** All 6 commands work correctly with in-memory SQLite DB.
**Feasibility:** ✅ Can be tested — `cargo test --lib commands::archetypes`.
**Coverage:**
- `create_archetype_cmd` — basic create, empty name rejected, duplicate rejected
- `list_archetypes_by_role` — returns matching archetypes, empty when no match
- `list_all_archetypes_cmd` — returns all, empty when none exist
- `get_archetype_cmd` — returns existing, not found error
- `update_archetype_cmd` — basic update, not found error
- `delete_archetype_cmd` — basic delete, not found error

### Test 3: Registration

**What to test:** Commands are registered in `invoke_handler` and accessible from frontend.
**Feasibility:** ⚠️ Verified indirectly through integration tests in Step 6 — if commands register and return correct data, registration is confirmed.
