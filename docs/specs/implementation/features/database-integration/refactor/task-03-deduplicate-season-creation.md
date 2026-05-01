# Task 03 - Deduplicate Season Creation Logic

## Overview

Refactor `create_season` to delegate to `create_season_tx`, eliminating ~40 lines of duplicated duplicate-check and insert logic. Make `create_season_tx` `pub(crate)` for the upcoming module split.

## Files to Modify

- `src-tauri/src/storage/mod.rs`

## Context

Two functions create season records with near-identical logic:

1. `create_season_tx` (private, ~line 357) — takes `&Transaction`, used by `import_season`. Does: derive label → check duplicate → insert → re-read `imported_at` → return `Season`.

2. `create_season` (public, ~line 514) — takes `&Connection`, used by tests and could be a command. Does: verify save exists → derive label → check duplicate → insert → re-read `imported_at` → return `Season`.

The only difference: `create_season` also verifies the save exists before proceeding. Everything after that check is identical to `create_season_tx`.

## Steps

- [ ] **Step 1: Change `create_season_tx` visibility to `pub(crate)`**

At ~line 357, change:

```rust
// BEFORE:
fn create_season_tx(
```

To:

```rust
// AFTER:
pub(crate) fn create_season_tx(
```

This is needed for the module split (Task 06) where `import.rs` will need to call this function across module boundaries.

- [ ] **Step 2: Replace `create_season` body**

Replace the entire `create_season` function body (~lines 514-569) with a version that delegates to `create_season_tx`:

```rust
/// Create a season record with auto-derived label.
/// Opens its own transaction and delegates to `create_season_tx`.
pub fn create_season(
    conn: &Connection,
    save_id: i64,
    in_game_date: &str,
) -> Result<Season, StorageError> {
    let tx = conn.unchecked_transaction()?;

    // Verify save exists
    let save_exists: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM saves WHERE id = ?1)",
        rusqlite::params![save_id],
        |row| row.get(0),
    )?;
    if !save_exists {
        return Err(StorageError::NotFound("Save not found.".to_string()));
    }

    let season = create_season_tx(&tx, save_id, in_game_date)?;
    tx.commit()?;
    Ok(season)
}
```

This reduces `create_season` from ~55 lines to ~20 lines by eliminating duplicated:
- `derive_season_label` call
- Duplicate season check query
- INSERT statement
- `imported_at` re-read
- `Season` construction

- [ ] **Step 3: Run all tests**

```bash
cd src-tauri && cargo test
```

Expected: All 219 tests pass. The behavioral contract is identical:
- Save existence check happens inside a transaction (previously outside, but single-user app = no TOCTOU concern)
- Duplicate check, insert, and re-read are performed by the same tested `create_season_tx` function
- `import_season` continues to use `create_season_tx` directly (unchanged)

## Dependencies

None. This task is independent of all other refactor tasks.

## Success Criteria

- All 219 tests pass without modification
- `create_season` body is ≤20 lines (delegates to `create_season_tx`)
- No duplicated SQL between `create_season` and `create_season_tx`
- `create_season_tx` is `pub(crate)` (visible within crate, not public API)

## Tests

### Test 1: Season creation behavior unchanged

**What to test:** All existing season CRUD tests pass — `create_season_basic`, `create_season_invalid_date_rejected`, `create_season_duplicate_date_rejected`, `create_season_save_not_found`.

**Feasibility:** ✅ `cargo test --lib storage -- create_season` covers all these paths.

### Test 2: Import still works

**What to test:** `import_season` (which calls `create_season_tx`) continues to work correctly.

**Feasibility:** ✅ `cargo test --lib storage -- import_season` covers this.
