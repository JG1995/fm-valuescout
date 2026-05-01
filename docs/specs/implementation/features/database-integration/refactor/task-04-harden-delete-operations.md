# Task 04 - Harden Delete Operations

## Overview

Simplify `delete_save` to trust SQLite's `ON DELETE CASCADE` (single statement instead of four), and add transaction wrapping to `delete_season` for atomic multi-step deletion.

## Files to Modify

- `src-tauri/src/storage/mod.rs`

## Context

### delete_save

The schema has `ON DELETE CASCADE` on all foreign keys:
- `saves(id)` ← `seasons(save_id) ON DELETE CASCADE`
- `saves(id)` ← `players(save_id) ON DELETE CASCADE`
- `seasons(id)` ← `player_seasons(season_id) ON DELETE CASCADE`
- `players(id)` ← `player_seasons(player_id) ON DELETE CASCADE`

`PRAGMA foreign_keys = ON` is set in `init_schema`.

The current `delete_save` manually deletes from `player_seasons`, `players`, `seasons`, and `saves` in four separate statements. The comment says "no cascade from saves → player_seasons directly" — true, but the cascade chain is `saves → players → player_seasons` and `saves → seasons → player_seasons`, which covers all tables.

A single `DELETE FROM saves WHERE id = ?1` cascades correctly:
1. Delete `saves` row
2. CASCADE deletes all `players` with that `save_id`
3. CASCADE deletes all `seasons` with that `save_id`
4. CASCADE from `seasons` deletes all `player_seasons` for those seasons
5. CASCADE from `players` deletes all `player_seasons` for those players

### delete_season

`delete_season` performs three operations:
1. Delete `player_seasons` for this season
2. Delete the `season`
3. Delete orphaned `players` (no remaining seasons)

If the process crashes between steps 1-3, the database is left in a partially-deleted state. Wrapping in a transaction ensures atomicity.

## Steps

- [ ] **Step 1: Simplify `delete_save`**

Replace the entire `delete_save` function body (~lines 312-337):

```rust
// BEFORE (4 statements):
pub fn delete_save(conn: &Connection, save_id: i64) -> Result<(), StorageError> {
    // Delete player_seasons first (no cascade from saves → player_seasons directly)
    conn.execute(
        "DELETE FROM player_seasons WHERE player_id IN (SELECT id FROM players WHERE save_id = ?1)",
        rusqlite::params![save_id],
    )?;
    // Delete players
    conn.execute(
        "DELETE FROM players WHERE save_id = ?1",
        rusqlite::params![save_id],
    )?;
    // Delete seasons
    conn.execute(
        "DELETE FROM seasons WHERE save_id = ?1",
        rusqlite::params![save_id],
    )?;
    // Delete save
    let rows = conn.execute(
        "DELETE FROM saves WHERE id = ?1",
        rusqlite::params![save_id],
    )?;
    if rows == 0 {
        return Err(StorageError::NotFound("Save not found.".to_string()));
    }
    Ok(())
}
```

With:

```rust
// AFTER (1 statement, CASCADE handles the rest):
/// Delete a save and all associated data (cascade: seasons, player_seasons, players).
pub fn delete_save(conn: &Connection, save_id: i64) -> Result<(), StorageError> {
    let rows = conn.execute(
        "DELETE FROM saves WHERE id = ?1",
        rusqlite::params![save_id],
    )?;
    if rows == 0 {
        return Err(StorageError::NotFound("Save not found.".to_string()));
    }
    Ok(())
}
```

- [ ] **Step 2: Add transaction wrapping to `delete_season`**

Replace the `delete_season` function body. Change all `conn.` references inside to `tx.` and wrap in a transaction:

```rust
// AFTER:
/// Delete a season, all associated player_seasons, and orphaned players.
/// All operations are atomic within a single transaction.
pub fn delete_season(conn: &Connection, season_id: i64) -> Result<(), StorageError> {
    let tx = conn.unchecked_transaction()?;

    let save_id: Option<i64> = tx.query_row(
        "SELECT save_id FROM seasons WHERE id = ?1",
        rusqlite::params![season_id],
        |row| row.get(0),
    ).ok();

    let save_id = match save_id {
        Some(sid) => sid,
        None => return Err(StorageError::NotFound("Season not found.".to_string())),
    };

    // Delete player_seasons for this season
    tx.execute(
        "DELETE FROM player_seasons WHERE season_id = ?1",
        rusqlite::params![season_id],
    )?;

    // Delete the season
    tx.execute(
        "DELETE FROM seasons WHERE id = ?1",
        rusqlite::params![season_id],
    )?;

    // Clean up orphaned players (players with no remaining seasons in this save)
    tx.execute(
        "DELETE FROM players WHERE save_id = :save_id AND id NOT IN \
         (SELECT DISTINCT player_id FROM player_seasons \
          JOIN seasons ON player_seasons.season_id = seasons.id \
          WHERE seasons.save_id = :save_id)",
        rusqlite::named_params!{":save_id": save_id},
    )?;

    tx.commit()?;
    Ok(())
}
```

Note: The early return `return Err(StorageError::NotFound(...))` drops `tx` without committing. Rust's `Transaction` drop impl rolls back automatically. Since no writes have occurred yet, this is harmless.

- [ ] **Step 3: Run all tests**

```bash
cd src-tauri && cargo test
```

Expected: All 219 tests pass. Key tests to verify:

- `delete_save_basic` — single save deleted
- `delete_save_cascades_seasons` — seasons cascade-deleted with save
- `delete_save_not_found` — error for nonexistent save
- Integration test `delete_save_cascades_all` — full cascade verification (seasons, players, player_seasons all gone)
- `delete_season_basic` — single season deleted
- `delete_season_not_found` — error for nonexistent season
- `delete_season_cleans_up_orphan_players` — orphan cleanup works
- Integration test `delete_season_cleans_orphans_preserves_shared` — shared players preserved

## Dependencies

None. This task is independent of all other refactor tasks.

## Success Criteria

- All 219 tests pass without modification
- `delete_save` is a single SQL statement (4 lines including error check)
- `delete_season` wraps all operations in a transaction
- Integration test `delete_save_cascades_all` passes (verifies CASCADE works)

## Tests

### Test 1: CASCADE correctness

**What to test:** Deleting a save removes all associated data.

**Feasibility:** ✅ `delete_save_cascades_all` integration test verifies all 4 tables are empty after delete.

### Test 2: Transaction atomicity

**What to test:** If `delete_season` fails mid-operation, no partial changes persist.

**Feasibility:** ⚠️ Difficult to test directly (would need to inject a failure mid-transaction). The type system guarantees: `tx.commit()` is the only success path; early returns and panics both drop `tx` without committing, triggering rollback.
