# Task 01 - Quick Fixes: Delete Dead Code & Fix created_at

## Overview

Remove the dead `save_import` stub and fix `create_save` to return the actual `created_at` timestamp instead of an empty string. Both are trivial changes with no behavioral risk.

## Files to Modify

- `src-tauri/src/storage/mod.rs`

## Context

### Dead code

Lines 196-204 contain the original `save_import` stub — a placeholder that was implemented before the database-integration feature. It is no longer called from any command, test, or storage function. It uses a different error type (`String`) than the rest of the module (`StorageError`), signaling it predates the current design.

### Empty created_at

`create_save` (line 247-254) returns `created_at: String::new()` with a comment "Will be populated by list_saves". The actual timestamp is set by SQLite's `DEFAULT (datetime('now'))`. Every other create function (`create_season`, `create_season_tx`) re-reads the server-generated timestamp after insert. `create_save` should do the same.

## Steps

- [ ] **Step 1: Delete the `save_import` stub**

Delete lines 196-204 in `src-tauri/src/storage/mod.rs`:

```rust
// DELETE THESE LINES (196-204):
// ── Placeholder for future tasks ───────────────────────────────────────

/// Persist imported players to the database.
/// Currently an honest stub — returns an error until implemented.
/// Idempotent: skips players with same UID + in_game_date.
/// Will be implemented in a future task (DB write layer).
pub fn save_import(_players: Vec<ParsedPlayer>, _in_game_date: &str) -> Result<(), String> {
    Err("Storage is not yet implemented. Your data has not been saved.".to_string())
}
```

- [ ] **Step 2: Fix `create_save` to return populated `created_at`**

In `src-tauri/src/storage/mod.rs`, find the `create_save` function. After the line `let id = conn.last_insert_rowid();` and before the `Ok(Save { ... })`, add a re-read of `created_at`:

Replace the return block (currently lines ~246-255):

```rust
// BEFORE:
    Ok(Save {
        id,
        name,
        managed_club: None,
        created_at: String::new(), // Will be populated by list_saves
        season_count: 0,
        player_count: 0,
    })
```

With:

```rust
// AFTER:
    let created_at: String = conn.query_row(
        "SELECT created_at FROM saves WHERE id = ?1",
        rusqlite::params![id],
        |row| row.get(0),
    )?;

    Ok(Save {
        id,
        name,
        managed_club: None,
        created_at,
        season_count: 0,
        player_count: 0,
    })
```

- [ ] **Step 3: Run all tests**

```bash
cd src-tauri && cargo test
```

Expected: All 219 tests pass. Two categories to verify:

1. **Storage unit tests** — the `create_save_basic` test at ~line 949 checks `save.managed_club.is_none()` but does NOT check `created_at`. After this change, `created_at` will contain a timestamp string instead of `""`. No test asserts it's empty, so all tests pass.

2. **Integration tests** — `full_import_flow` and other tests create saves and never inspect `created_at` directly.

- [ ] **Step 4: Verify `save_import` has no callers**

```bash
cd src-tauri && grep -rn "save_import" src/
```

Expected: No matches. The function and all references were already gone (only the definition existed, no callers).

## Dependencies

None. This task is independent of all other refactor tasks.

## Success Criteria

- `save_import` function no longer exists anywhere in the codebase
- `create_save` returns a non-empty `created_at` string
- All 219 tests pass without modification
- `grep -rn "save_import" src-tauri/src/` returns no matches

## Tests

### Test 1: Dead code removed

**What to test:** No reference to `save_import` exists in source code.

**Feasibility:** ✅ Verifiable via `grep -rn "save_import" src-tauri/src/`

### Test 2: created_at populated on create

**What to test:** `create_save` returns a Save with non-empty `created_at`.

**Feasibility:** ✅ Existing tests pass (none assert empty). Visual inspection of return value confirms timestamp.
