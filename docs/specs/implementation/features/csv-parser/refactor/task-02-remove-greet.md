# Task 02 - Remove greet Scaffold Function

## Overview

The `greet` function in `src-tauri/src/lib.rs` is leftover from the Tauri scaffold template. It has nothing to do with the CSV parser feature and ships dead code to production.

## Files to Create/Modify

- Modify: `src-tauri/src/lib.rs` (26 lines)

## Steps

- [ ] **Step 1: Remove the greet function and its registration**

In `src-tauri/src/lib.rs`, delete the `greet` function (lines 7-13) and remove `greet,` from the `invoke_handler` (line 21).

The file should become:

```rust
mod commands;
pub mod parser;
pub use parser::parse_csv;
mod storage;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::csv_parser::parse_csv,
            commands::csv_parser::save_import,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors and no warnings about `greet`.

- [ ] **Step 3: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All existing tests pass. No test referenced `greet`.

## Dependencies

- None.

## Success Criteria

- No `greet` function exists anywhere in `src-tauri/src/`.
- No reference to `greet` in `invoke_handler`.
- `cargo check` passes.
- All existing tests pass.

## Tests

### Test 1: Compilation succeeds without greet

**What to test:** `cargo check` succeeds after removal.

**Feasibility:** ✅ Can be tested — `cargo check` is the verification.
