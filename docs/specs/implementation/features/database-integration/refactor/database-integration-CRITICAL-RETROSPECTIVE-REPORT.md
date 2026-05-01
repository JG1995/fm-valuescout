# Critical Retrospective Report

## Scope

Feature-level review of the **database-integration** feature (9 tasks, 7 commits). Files covered:

- `src-tauri/src/storage/mod.rs` (1703 lines — 757 implementation + 945 tests)
- `src-tauri/src/commands/storage.rs` (95 lines)
- `src-tauri/src/commands/csv_parser.rs` (9 lines)
- `src-tauri/src/commands/mod.rs` (2 lines)
- `src-tauri/src/lib.rs` (49 lines)
- `src-tauri/tests/integration_storage.rs` (343 lines)
- `src-tauri/tests/edge_case_storage.rs` (398 lines)

## Executive Summary

The feature is functionally solid: 219 tests passing, correct schema design with hybrid queryable-columns + JSON-blob approach, and proper save-game isolation. The primary issues are structural: a 1700-line monolith mixing implementation and tests, dead code left over from the pre-implementation stub, duplicated logic between `create_season` and `create_season_tx`, and silent error swallowing via `filter_map(|r| r.ok())`. These are all fixable without behavioral changes.

## What Went Well

- **Schema design.** The hybrid approach (queryable columns for filter/sort + JSON blob for full data) is the right call for this domain — it gives SQL-queryable fields for list views while preserving every parsed attribute without schema migrations.
- **Save-game isolation.** Every table scopes to `save_id` either directly or via foreign keys. The orphan-player cleanup in `delete_season` shows genuine thought about data integrity.
- **Test coverage.** 219 tests across unit, integration, and edge-case files with meaningful assertions — not just "it doesn't crash."
- **Graceful JSON degradation.** When the JSON blob fails to deserialize, queryable fields are still available. This is a pragmatic hedge against schema drift or corruption.
- **Import transaction.** `import_season` correctly wraps everything in a single transaction with rollback on failure.

## Critical Findings

### 1. 1700-Line Monolith Mixing Implementation and Tests

**Current State:**
`src-tauri/src/storage/mod.rs`: 1703 lines total. Lines 1-757 are implementation; lines 759-1703 are `#[cfg(test)] mod tests`. There is no file-level separation of concerns — DDL, DTOs, error types, CRUD, import logic, retrieval logic, and 70+ test functions all share one file.

**Impact:**
- Navigation overhead: finding any function requires scrolling past 945 lines of tests or vice versa.
- Compilation bloat: the entire test module is parsed on every build even though it's cfg-gated.
- Review friction: any PR touching storage presents a diff mixed with test changes.

**Recommended Refactor:**
Split into module directory:
```
src-tauri/src/storage/
├── mod.rs          (re-exports: pub use ...)
├── schema.rs       (SCHEMA_DDL, init_schema, init_db, init_db_test)
├── error.rs        (StorageError + Display + From impls)
├── types.rs        (Save, Season, ImportResult, PlayerSeasonData, DbState)
├── saves.rs        (validate_save_name, create_save, list_saves, rename_save, delete_save)
├── seasons.rs      (derive_season_label, create_season, create_season_tx, list_seasons, rename_season, delete_season)
├── import.rs       (format_positions, import_season)
├── retrieval.rs    (row_to_player_season, get_players_for_season, get_player_career, get_latest_season)
```

Tests stay in their current location within `mod.rs` or move to `src-tauri/src/storage/tests.rs` — either is acceptable. The key win is production code separation.

**Effort:** Medium
**Priority:** P1 (important)

### 2. Dead Code: `save_import` Stub Still Present

**Current State:**
`src-tauri/src/storage/mod.rs:196-204` — the original stub function that was the starting point for this entire feature:

```rust
pub fn save_import(_players: Vec<ParsedPlayer>, _in_game_date: &str) -> Result<(), String> {
    Err("Storage is not yet implemented. Your data has not been saved.".to_string())
}
```

It is no longer called from anywhere — not from commands, not from tests, not from other storage functions.

**Impact:**
- Misleading for any reader who encounters it — suggests unimplemented functionality.
- Uses a different error type (`String`) than the rest of the module (`StorageError`), signaling it predates the current design.
- Violates the principle that deleted code leaves no trace.

**Recommended Refactor:**
Delete the function entirely.

**Effort:** Low
**Priority:** P0 (critical) — it takes 30 seconds and removes confusion.

### 3. Duplicated Logic: `create_season` vs `create_season_tx`

**Current State:**
Two functions at `src-tauri/src/storage/mod.rs:357-401` (`create_season_tx`) and `src-tauri/src/storage/mod.rs:514-569` (`create_season`) that do the same thing:
1. Validate save exists
2. Derive season label
3. Check for duplicate season (identical query)
4. INSERT into seasons
5. Re-read `imported_at`
6. Return `Season`

The only differences: `create_season_tx` takes a `&Transaction` and skips the save-exists check (the caller `import_season` already does it). `create_season` takes `&Connection` and does the save check itself.

**Impact:**
- Bug fixes must be applied in two places (the duplicate-check query was already copied verbatim).
- `create_season` is `pub` but is not wired to any Tauri command — it's only used by tests. This is a test-only convenience function that got promoted to public API.

**Recommended Refactor:**
Make `create_season` call the transactional version internally, or extract the shared logic (duplicate check, insert, re-read) into a single helper. Since `create_season` is test-only, consider making it `#[cfg(test)]` or moving it to a test helper module.

```rust
pub fn create_season(conn: &Connection, save_id: i64, in_game_date: &str) -> Result<Season, StorageError> {
    let tx = conn.unchecked_transaction()?;
    // save-exists check (unique to this path)
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

**Effort:** Low
**Priority:** P1 (important)

### 4. Silent Error Swallowing: `filter_map(|r| r.ok())`

**Current State:**
Four production call sites use `.filter_map(|r| r.ok())` to collect query results:
- `list_saves` (`mod.rs:279`)
- `list_seasons` (`mod.rs:586`)
- `get_players_for_season` (`mod.rs:699`)
- `get_player_career` (`mod.rs:728`)

This silently discards any row that fails to map. A schema change, added column, or migration bug that causes a type mismatch in one row will silently drop that row from results. The caller will see a shorter list with no indication of data loss.

**Impact:**
- The worst kind of failure: plausible-looking output that's missing data.
- Debugging is extremely difficult — there's no log, no error, no warning.
- The `get_players_for_season` case is particularly risky because `row_to_player_season` does JSON deserialization that can fail in non-obvious ways.

**Recommended Refactor:**
For `list_saves` and `list_seasons` (simple row mapping), use `collect::<Result<Vec<_>, _>>()?` — any mapping failure becomes a proper error:

```rust
let saves: Vec<Save> = stmt.query_map([], |row| {
    Ok(Save { ... })
})?.collect::<Result<Vec<_>, _>>()?;
```

For `get_players_for_season` and `get_player_career`, where graceful degradation on individual bad JSON blobs is intentional, propagate row-mapping failures but handle JSON failures explicitly in `row_to_player_season` (which already does this correctly — the JSON error becomes `data: None`, not a row error).

**Effort:** Low
**Priority:** P1 (important) — this is a silent data-loss vector.

### 5. Hardcoded Column Index in `row_to_player_season`

**Current State:**
`src-tauri/src/storage/mod.rs:651-673` — `row_to_player_season` uses positional column indices (0-15) to extract fields from the query result:

```rust
let data_json: String = row.get(14)?;  // Index 14 = data column
// ... then uses indices 0-13, 15 for the remaining fields
```

The comment at line 652 says "Index 14 is the JSON data blob" but the SELECT statements at lines 686-694 and 713-721 must match this exact column order. If anyone adds, removes, or reorders a column in the SELECT, the indices silently read wrong fields.

**Impact:**
- Adding a column to the query requires updating 16 index references.
- Reordering columns causes silent misreads (wrong field in wrong slot).
- The `contract_expires` at index 15 has a gap from the `data` at index 14, which is the reason for the separate extraction — but it's fragile.

**Recommended Refactor:**
Use `row.get("column_name")` with named column references, or use rusqlite's `FromRow` derive. The query already uses table-qualified names, so column names are unambiguous. This is a straightforward mechanical change.

```rust
fn row_to_player_season(row: &rusqlite::Row) -> rusqlite::Result<PlayerSeasonData> {
    let data_json: String = row.get("data")?;
    let data = serde_json::from_str::<ParsedPlayer>(&data_json).ok();
    Ok(PlayerSeasonData {
        id: row.get("id")?,
        player_id: row.get("player_id")?,
        // ...
    })
}
```

This requires using column aliases in the SELECT if there are ambiguous names across joins. The current queries join `player_seasons ps` with `players p`, so `name` from players needs aliasing to `player_name`.

**Effort:** Low
**Priority:** P2 (nice to have) — the current code works and is tested, but it's a maintenance trap.

### 6. `delete_save` Manually Cascades Despite Schema CASCADE

**Current State:**
`src-tauri/src/storage/mod.rs:312-337` — `delete_save` manually deletes from `player_seasons`, `players`, `seasons`, and `saves` in four separate statements. The comment at line 313 says "no cascade from saves → player_seasons directly" which is true — the FK chain is `saves → players → player_seasons` and `saves → seasons → player_seasons`.

However, with `PRAGMA foreign_keys = ON` (set in `init_schema` at line 71), deleting from `saves` would cascade to `players` and `seasons`, and then cascading from `seasons` would delete `player_seasons`. The manual deletion is belt-and-suspenders.

**Impact:**
- Four SQL statements where one would suffice.
- The manual ordering is correct but must be maintained if the schema changes.
- Not a bug — just unnecessary complexity.

**Recommended Refactor:**
Trust the CASCADE and simplify to:

```rust
pub fn delete_save(conn: &Connection, save_id: i64) -> Result<(), StorageError> {
    let rows = conn.execute("DELETE FROM saves WHERE id = ?1", rusqlite::params![save_id])?;
    if rows == 0 {
        return Err(StorageError::NotFound("Save not found.".to_string()));
    }
    Ok(())
}
```

This works because `PRAGMA foreign_keys = ON` is set at init, and the schema has `ON DELETE CASCADE` on all foreign keys. The existing integration test `delete_save_cascades_all` would verify this continues to work.

**Effort:** Low
**Priority:** P2 (nice to have)

### 7. `create_save` Returns Empty `created_at` String

**Current State:**
`src-tauri/src/storage/mod.rs:247-254` — `create_save` returns a `Save` with `created_at: String::new()`. The comment says "Will be populated by list_saves." This means the immediate return from `create_save` has a different shape than what you get from `list_saves`.

**Impact:**
- Any caller that creates a save and immediately reads `created_at` gets `""`.
- The frontend must know to re-fetch or ignore `created_at` on creation.
- Inconsistent API: sometimes `created_at` is populated, sometimes empty.

**Recommended Refactor:**
Re-read the `created_at` after insert (same pattern used in `create_season` at line 556-560):

```rust
let created_at: String = conn.query_row(
    "SELECT created_at FROM saves WHERE id = ?1",
    rusqlite::params![id],
    |row| row.get(0),
)?;
```

**Effort:** Low
**Priority:** P2 (nice to have) — known issue documented as acceptable, but the fix is trivial.

### 8. Missing Transaction Wrapping in `delete_save` and `delete_season`

**Current State:**
`delete_save` (lines 312-337) and `delete_season` (lines 609-644) execute multiple SQL statements without transaction wrapping. If the process crashes mid-deletion, the database can be left in a partially-deleted state.

**Impact:**
- `delete_season` deletes `player_seasons`, then `seasons`, then orphans. A crash after step 1 but before step 3 leaves orphaned data that's harder to clean up.
- Low probability in a single-user desktop app, but the fix is trivial.

**Recommended Refactor:**
Wrap in a transaction:

```rust
pub fn delete_season(conn: &Connection, season_id: i64) -> Result<(), StorageError> {
    let tx = conn.unchecked_transaction()?;
    // ... existing logic using &tx instead of conn ...
    tx.commit()?;
    Ok(())
}
```

**Effort:** Low
**Priority:** P2 (nice to have) — user opted to skip this per handoff notes.

### 9. Command Wrappers Have Identical Boilerplate

**Current State:**
`src-tauri/src/commands/storage.rs` — all 12 command functions follow the exact same pattern:

```rust
let conn = state.conn.lock().map_err(|e| e.to_string())?;
storage::some_function(&conn, args).map_err(|e| e.into())
```

The `StorageError → String` conversion happens at the Tauri boundary, which is correct. But 12 functions repeating the same lock-then-delegate pattern is boilerplate.

**Impact:**
- Not a functional problem — the pattern is correct.
- If the error conversion logic changes, it must change in 12 places.
- Verbose but each function is short (3-5 lines), so the cost is limited.

**Recommended Refactor:**
This is acceptable as-is. A macro would reduce repetition but adds indirection for minimal gain at 12 functions. Flagged as an observation, not a recommendation to change.

**Effort:** N/A
**Priority:** Not recommended for change.

## What Should Have Been Done Before

1. **Module structure before implementation.** Starting with a single `mod.rs` made sense for a greenfield feature, but the module should have been split into a directory structure once it exceeded ~300 lines. The 1700-line result is a predictable consequence of not having a "split point" checkpoint.

2. **Delete the stub before building the replacement.** The `save_import` stub should have been deleted the moment `import_season` was implemented. Leaving it created a "which one do I use?" ambiguity that persisted through code review.

3. **Named column references from the start.** Using `row.get("name")` instead of `row.get(4)` is a best practice for any query that selects more than 2-3 columns. It costs nothing at runtime and prevents an entire class of bugs.

## Refactor Priorities

### Phase 1 (Must Fix)

1. **Finding #2** — Delete `save_import` stub (dead code)
2. **Finding #4** — Replace `filter_map(|r| r.ok())` with proper error propagation in production queries

### Phase 2 (Should Fix)

1. **Finding #1** — Split `storage/mod.rs` into module directory
2. **Finding #3** — Deduplicate `create_season` / `create_season_tx`
3. **Finding #5** — Replace hardcoded column indices with named references

### Phase 3 (Nice to Have)

1. **Finding #6** — Simplify `delete_save` to trust CASCADE
2. **Finding #7** — Fix `create_save` to return populated `created_at`
3. **Finding #8** — Add transaction wrapping to delete operations

## Implementation Notes

- **Phase 1 changes are zero-risk.** Deleting dead code and changing `filter_map` to `collect::<Result<...>>` cannot change behavior for any currently-passing test (the tests verify the happy path, which won't produce mapping errors).
- **Phase 2 module split** should be done in a single commit with no behavioral changes. The `mod.rs` becomes a re-export hub. All existing tests continue to use `use super::*` so they don't need import changes.
- **Named column references** (Finding #5) requires adding aliases in the SELECT statements where column names collide across joins. The key collision is `p.name` → needs aliasing to `player_name` to match the `PlayerSeasonData` field name.
- **Deduplicating create_season** (Finding #3) — the `create_season` function can delegate to `create_season_tx` by opening its own transaction. This preserves the public API for tests while eliminating the duplicated duplicate-check and insert logic.

## Files Affected

- `src-tauri/src/storage/mod.rs` — split into module directory, delete dead code, fix error handling, replace indices
- `src-tauri/src/storage/schema.rs` — new file (extracted from mod.rs)
- `src-tauri/src/storage/error.rs` — new file (extracted from mod.rs)
- `src-tauri/src/storage/types.rs` — new file (extracted from mod.rs)
- `src-tauri/src/storage/saves.rs` — new file (extracted from mod.rs)
- `src-tauri/src/storage/seasons.rs` — new file (extracted from mod.rs)
- `src-tauri/src/storage/import.rs` — new file (extracted from mod.rs)
- `src-tauri/src/storage/retrieval.rs` — new file (extracted from mod.rs)
- `src-tauri/src/commands/storage.rs` — no changes needed
- `src-tauri/tests/integration_storage.rs` — no changes needed
- `src-tauri/tests/edge_case_storage.rs` — no changes needed

## Success Criteria

- [ ] All 219 existing tests pass without modification
- [ ] `save_import` function no longer exists
- [ ] No `filter_map(|r| r.ok())` in production query paths (tests are exempt)
- [ ] `storage/mod.rs` is under 100 lines (re-exports only)
- [ ] No production file exceeds 200 lines
- [ ] `create_season` delegates to `create_season_tx` (no duplicated SQL)
- [ ] `row_to_player_season` uses named column references, not positional indices
