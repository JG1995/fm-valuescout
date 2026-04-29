# Task 08 — Tauri Commands & Wiring

## Overview

Create the Tauri command handlers (`parse_csv`, `save_import`) in `src-tauri/src/commands/`, wire them into `lib.rs`, and create the storage module stub. The `parse_csv` command exposes the parser pipeline to the frontend via Tauri IPC. The `save_import` command persists data (storage implementation is stubbed for now).

## Files to Create/Modify

- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/csv_parser.rs`
- Create: `src-tauri/src/storage/mod.rs`
- Modify: `src-tauri/src/lib.rs`

## Steps

- [ ] **Step 1: Create commands module**

Create `src-tauri/src/commands/mod.rs`:

```rust
pub mod csv_parser;
```

- [ ] **Step 2: Create csv_parser command handler**

Create `src-tauri/src/commands/csv_parser.rs`:

```rust
use crate::parser;
use crate::parser::types::{ParseResult, ParsedPlayer};
use crate::storage;

/// Parse a CSV file. Pure function — no side effects.
/// Returns ParseResult with players, skipped rows, warnings, and column status.
#[tauri::command]
pub fn parse_csv(file_path: String, in_game_date: String) -> Result<ParseResult, String> {
    parser::parse_csv(&file_path, &in_game_date)
}

/// Persist parsed players to the database.
/// Idempotent: players with the same UID + in_game_date are skipped silently.
#[tauri::command]
pub fn save_import(players: Vec<ParsedPlayer>, in_game_date: String) -> Result<(), String> {
    storage::save_import(players, &in_game_date)
}
```

- [ ] **Step 3: Create storage stub**

Create `src-tauri/src/storage/mod.rs`:

```rust
use crate::parser::types::ParsedPlayer;

/// Persist imported players to the database.
/// Currently a stub — will be implemented with the persistence layer.
/// Idempotent: skips players with same UID + in_game_date.
pub fn save_import(players: Vec<ParsedPlayer>, _in_game_date: &str) -> Result<(), String> {
    // TODO: Implement actual persistence when storage layer is built.
    // For now, just validate the input and return success.
    if players.is_empty() {
        return Ok(());
    }
    // Stub: accept all players, no actual storage
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::types::{Position, Role, Side};

    #[test]
    fn save_import_accepts_empty() {
        assert!(save_import(vec![], "2026-01-01").is_ok());
    }

    #[test]
    fn save_import_stub_accepts_players() {
        let players = vec![ParsedPlayer::empty(1, "Test".to_string(), vec![Position {
            role: Role::ST,
            sides: vec![Side::C],
        }])];
        assert!(save_import(players, "2026-01-01").is_ok());
    }
}
```

- [ ] **Step 4: Update lib.rs to register commands**

Replace the contents of `src-tauri/src/lib.rs`:

```rust
mod commands;
mod parser;
mod storage;

// Keep the greet command for reference during development
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::csv_parser::parse_csv,
            commands::csv_parser::save_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Run full test suite**

Run: `cd src-tauri && cargo test`
Expected: All tests pass — types, countries, positions, headers, fields, metrics, parser orchestration, and storage stub.

- [ ] **Step 6: Verify Tauri build compiles**

Run: `cd src-tauri && cargo build`
Expected: Compiles without errors. No warnings about unused code.

## Dependencies

- Task 07 (parser orchestration must be complete)

## Success Criteria

- `cargo test` passes with all modules.
- `cargo build` succeeds.
- Two Tauri commands registered: `parse_csv`, `save_import`.
- Storage module is a stub that compiles and passes its 2 tests.
- `lib.rs` registers all commands in the Tauri builder.

## Tests

### Test 1: Storage stub accepts empty player list
**Feasibility:** ✅ Can be tested

### Test 2: Storage stub accepts player list
**Feasibility:** ✅ Can be tested

### Implicit: All prior module tests still pass after wiring
**Feasibility:** ✅ Verified by `cargo test`
