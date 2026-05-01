# Task 05 - Tauri Archetype Commands

## Overview

Create Tauri command wrappers for archetype CRUD operations. These are the IPC bridge between the Svelte frontend and the Rust storage layer. Follow the exact pattern from `src-tauri/src/commands/storage.rs`.

## Files to Create/Modify

- Create: `src-tauri/src/commands/archetypes.rs` — Tauri command wrappers
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

### Types Available for IPC

From `src-tauri/src/storage/archetypes.rs`:
- `Archetype` — already `Serialize, Deserialize`
- `MetricWeight` — already `Serialize, Deserialize`

The frontend will send/receive these as JSON.

## Steps

- [ ] **Step 1: Create the commands file**

Create `src-tauri/src/commands/archetypes.rs`:

```rust
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
```

- [ ] **Step 2: Register the module**

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

- [ ] **Step 3: Register commands in invoke_handler**

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

- [ ] **Step 4: Run compilation check**

Run: `cd src-tauri && cargo check`
Expected: SUCCESS — no errors, no warnings.

- [ ] **Step 5: Run full test suite**

Run: `cd src-tauri && cargo test --lib`
Expected: ALL PASS — no regressions.

## Dependencies

- Task 02 (archetype types) — `Archetype`, `MetricWeight` types
- Task 03 (archetype CRUD) — Storage functions to wrap
- Task 04 (seed data) — Not strictly required for compilation, but startup will seed data

## Success Criteria

- All 6 commands compile and register
- `cargo check` passes
- All existing tests still pass
- Frontend can call these commands via `invoke("command_name", { params })`

## Tests

### Test 1: Compilation

**What to test:** All commands compile without errors.
**Feasibility:** ✅ Can be tested — `cargo check`.

### Test 2: Registration

**What to test:** Commands are registered in `invoke_handler` and accessible from frontend.
**Feasibility:** ⚠️ Dependent on running the Tauri app — verified in Task 06 integration.
