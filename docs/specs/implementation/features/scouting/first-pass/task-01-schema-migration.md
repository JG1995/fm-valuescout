# Task 01 - Schema Migration: Archetypes Table

## Overview

Add the `archetypes` table to the existing SQLite schema. This table stores user-defined scoring archetypes (name, role, metric weights). The migration must be idempotent and follow the existing `SCHEMA_DDL` pattern in `src-tauri/src/storage/schema.rs`.

## Files to Create/Modify

- Modify: `src-tauri/src/storage/schema.rs` — Add `archetypes` table DDL to `SCHEMA_DDL` constant
- Test: `src-tauri/src/storage/mod.rs` — Add schema tests for the new table

## Context

### Existing Pattern (from `src-tauri/src/storage/schema.rs`)

Schema DDL is a single `const SCHEMA_DDL: &str` containing all `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF NOT EXISTS` statements. The `init_schema()` function runs `conn.execute_batch(SCHEMA_DDL)`. This is called once at app startup in `src-tauri/src/lib.rs`.

### Existing Tables

- `saves` — Save games
- `seasons` — Season snapshots within saves
- `players` — Player records (keyed by save + fm_uid)
- `player_seasons` — Player data per season (includes `data` JSON blob of `ParsedPlayer`)

### Archetype Table Design

From the design spec:
- **id**: Primary key
- **name**: Archetype name (e.g., "Ball-Playing Goalkeeper")
- **role**: FM position role this archetype applies to (maps to `Role` enum: GK, D, WB, DM, M, AM, ST)
- **metrics_json**: JSON array of `{ metric_key: string, weight: number, inverted: boolean }` — the weighted metrics for scoring
- **is_default**: Boolean flag — true for built-in archetypes, false for user-created
- **created_at**: Creation timestamp
- **updated_at**: Last modification timestamp

Constraints:
- `(name, role)` must be unique — no two archetypes with same name for same role
- `name` must be non-empty (enforced in application code, not schema)
- `role` must be non-null

## Steps

- [ ] **Step 1: Write the failing test**

Add this test to the `#[cfg(test)] mod tests` block in `src-tauri/src/storage/mod.rs`, after the existing `schema_creates_all_tables` test (around line 146):

```rust
#[test]
fn schema_creates_archetypes_table() {
    let conn = setup_test_db();
    let tables: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    ).unwrap()
        .query_map([], |row| row.get(0)).unwrap()
        .filter_map(|r| r.ok())
        .collect();
    assert!(tables.contains(&"archetypes".to_string()));
}

#[test]
fn schema_creates_archetypes_indexes() {
    let conn = setup_test_db();
    let indexes: Vec<String> = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%' ORDER BY name"
    ).unwrap()
        .query_map([], |row| row.get(0)).unwrap()
        .filter_map(|r| r.ok())
        .collect();
    assert!(indexes.contains(&"idx_archetypes_role".to_string()));
}

#[test]
fn archetypes_unique_name_role_constraint() {
    let conn = setup_test_db();
    conn.execute(
        "INSERT INTO archetypes (name, role, metrics_json, is_default) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["Test Arch", "GK", "[]", true],
    ).unwrap();
    let result = conn.execute(
        "INSERT INTO archetypes (name, role, metrics_json, is_default) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params!["Test Arch", "GK", "[]", false],
    );
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test schema_creates_archetypes_table --lib -- --nocapture`
Expected: FAIL — `archetypes` table does not exist yet.

Run: `cd src-tauri && cargo test archetypes_unique_name_role --lib -- --nocapture`
Expected: FAIL — `archetypes` table does not exist yet.

- [ ] **Step 3: Add the archetypes DDL to SCHEMA_DDL**

In `src-tauri/src/storage/schema.rs`, append the following to the `SCHEMA_DDL` constant, after the existing index definitions (after line 53):

```sql
CREATE TABLE IF NOT EXISTS archetypes (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    name         TEXT    NOT NULL,
    role         TEXT    NOT NULL,
    metrics_json TEXT    NOT NULL,
    is_default   INTEGER NOT NULL DEFAULT 0,
    created_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    updated_at   TEXT    NOT NULL DEFAULT (datetime('now')),
    UNIQUE(name, role)
);

CREATE INDEX IF NOT EXISTS idx_archetypes_role ON archetypes(role);
```

The full `SCHEMA_DDL` constant should now contain the original 4 tables + indexes, plus this new table and index.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test schema --lib -- --nocapture`
Expected: ALL PASS — including existing schema tests and the new archetypes tests.

- [ ] **Step 5: Run full test suite to verify no regressions**

Run: `cd src-tauri && cargo test --lib`
Expected: ALL PASS — all existing tests still pass.

## Dependencies

- None — this is the foundation task.

## Success Criteria

- `archetypes` table is created by `init_schema()`
- Unique constraint on `(name, role)` works
- Index `idx_archetypes_role` exists
- All existing tests still pass
- `schema_is_idempotent` test still passes (double `init_schema` does not fail)

## Tests

### Test 1: Table creation

**What to test:** `archetypes` table appears in `sqlite_master` after schema init.
**Feasibility:** ✅ Can be tested — directly query `sqlite_master`.

### Test 2: Index creation

**What to test:** `idx_archetypes_role` index exists.
**Feasibility:** ✅ Can be tested — directly query `sqlite_master`.

### Test 3: Unique constraint

**What to test:** Inserting duplicate `(name, role)` pair fails.
**Feasibility:** ✅ Can be tested — attempt duplicate insert, assert error.

### Test 4: Idempotency

**What to test:** Running `init_schema` twice does not fail (existing test covers this).
**Feasibility:** ✅ Already tested by `schema_is_idempotent`.
