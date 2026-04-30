# Task 07 - Command Wiring and App Initialization

## Overview

Wire the storage functions to Tauri commands. Create `src-tauri/src/commands/storage.rs` with command wrappers, update `commands/mod.rs` to include the new module, and update `lib.rs` to initialize DbState, register new command handlers, and remove the `save_import` stub registration.

## Files to Create/Modify

- Create: `src-tauri/src/commands/storage.rs` — Tauri command wrappers
- Modify: `src-tauri/src/commands/mod.rs` — add `pub mod storage;`
- Modify: `src-tauri/src/commands/csv_parser.rs` — remove `save_import` command
- Modify: `src-tauri/src/lib.rs` — add DbState init, register new handlers, remove save_import

## Steps

- [ ] **Step 1: Create `commands/storage.rs`**

Create file `src-tauri/src/commands/storage.rs` with:

```rust
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
    storage::import_season(&conn, save_id, &players, &in_game_date).map_err(|e| e.into())
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
```

- [ ] **Step 2: Update `commands/mod.rs`**

Replace the entire contents of `src-tauri/src/commands/mod.rs` with:

```rust
pub mod csv_parser;
pub mod storage;
```

- [ ] **Step 3: Remove `save_import` from `commands/csv_parser.rs`**

Replace the entire contents of `src-tauri/src/commands/csv_parser.rs` with:

```rust
use crate::parser;
use crate::parser::types::ParseResult;

/// Parse a CSV file. Pure function — no side effects.
/// Returns ParseResult with players, skipped rows, warnings, and column status.
#[tauri::command]
pub fn parse_csv(file_path: String, _in_game_date: String) -> Result<ParseResult, String> {
    parser::parse_csv(&file_path)
}
```

- [ ] **Step 4: Update `lib.rs` with DbState init and new handlers**

Replace the entire contents of `src-tauri/src/lib.rs` with:

```rust
mod commands;
pub mod parser;
pub mod storage;
pub use parser::parse_csv;

use std::sync::Mutex;
use storage::DbState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()
                .map_err(|e| format!("Unable to access app data directory: {}", e))?;

            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| format!("Unable to create app data directory: {}", e))?;

            let db_path = app_data_dir.join("fm_valuescout.db");
            let db_path_str = db_path.to_string_lossy().to_string();

            let conn = storage::init_db(&db_path_str)
                .map_err(|e| format!("Unable to initialize database: {}", e))?;

            app.manage(DbState {
                conn: Mutex::new(conn),
            });

            Ok(())
        })
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Run `cargo check`**

Run: `cd src-tauri && cargo check`
Expected: Compiles without errors.

- [ ] **Step 6: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All tests pass (existing 113 tests — note: 2 old `save_import` stub tests are removed from storage/mod.rs, replaced by new tests in tasks 01-06).

## Dependencies

- Task 01 (StorageError, DbState, DTOs)
- Task 02 (init_db)
- Task 03 (save CRUD)
- Task 04 (season CRUD)
- Task 05 (import_season)
- Task 06 (data retrieval)

## Success Criteria

- `save_import` command completely removed from both `commands/csv_parser.rs` and `lib.rs`.
- All 12 new Tauri commands registered in `generate_handler![]`.
- DbState initialized in `.setup()` with proper error handling for app data directory.
- `cargo check` succeeds.
- All tests pass.

## Tests

### Test 1: Build succeeds

**What to test:** `cargo check` passes after wiring.

**Feasibility:** ✅ Can be tested

### Test 2: All existing tests pass

**What to test:** `cargo test` — 113 existing tests (minus 2 removed stub tests, plus new storage tests).

**Feasibility:** ✅ Can be tested

### Test 3: No dead code

**What to test:** `cargo check` shows no warnings about unused `save_import`.

**Feasibility:** ✅ Can be tested
