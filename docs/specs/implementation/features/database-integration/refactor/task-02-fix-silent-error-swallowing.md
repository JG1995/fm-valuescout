# Task 02 - Fix Silent Error Swallowing

## Overview

Replace `.filter_map(|r| r.ok())` with `.collect::<Result<Vec<_>, _>>()?` in all four production query functions. This converts silent row-dropping into explicit error propagation.

## Files to Modify

- `src-tauri/src/storage/mod.rs`

## Context

Four production functions silently discard database rows that fail to map:

| Function | Line | Current Pattern |
|---|---|---|
| `list_saves` | ~279 | `.filter_map(\|r\| r.ok()).collect()` |
| `list_seasons` | ~586 | `.filter_map(\|r\| r.ok()).collect()` |
| `get_players_for_season` | ~699 | `.filter_map(\|r\| r.ok()).collect()` |
| `get_player_career` | ~728 | `.filter_map(\|r\| r.ok()).collect()` |

If any row fails to map (schema change, type mismatch, migration bug), it is silently dropped from results. The caller sees a shorter list with no indication of data loss.

Additionally, `get_players_for_season` and `get_player_career` have a redundant `.map_err(|e| StorageError::Database(e.to_string()))` before the `?`. Since we have `impl From<rusqlite::Error> for StorageError`, this `.map_err()` does the same thing as `?` alone. Remove it.

## Steps

- [ ] **Step 1: Fix `list_saves`**

In `src-tauri/src/storage/mod.rs`, find the `list_saves` function. Replace the query_map chain:

```rust
// BEFORE:
    let saves = stmt.query_map([], |row| {
        Ok(Save {
            id: row.get(0)?,
            name: row.get(1)?,
            managed_club: row.get(2)?,
            created_at: row.get(3)?,
            season_count: row.get(4)?,
            player_count: row.get(5)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(saves)
```

With:

```rust
// AFTER:
    let saves: Vec<Save> = stmt.query_map([], |row| {
        Ok(Save {
            id: row.get(0)?,
            name: row.get(1)?,
            managed_club: row.get(2)?,
            created_at: row.get(3)?,
            season_count: row.get(4)?,
            player_count: row.get(5)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(saves)
```

- [ ] **Step 2: Fix `list_seasons`**

Find the `list_seasons` function. Replace the query_map chain:

```rust
// BEFORE:
    let seasons = stmt.query_map(rusqlite::params![save_id], |row| {
        Ok(Season {
            id: row.get(0)?,
            save_id: row.get(1)?,
            in_game_date: row.get(2)?,
            label: row.get(3)?,
            imported_at: row.get(4)?,
        })
    })?.filter_map(|r| r.ok()).collect();

    Ok(seasons)
```

With:

```rust
// AFTER:
    let seasons: Vec<Season> = stmt.query_map(rusqlite::params![save_id], |row| {
        Ok(Season {
            id: row.get(0)?,
            save_id: row.get(1)?,
            in_game_date: row.get(2)?,
            label: row.get(3)?,
            imported_at: row.get(4)?,
        })
    })?.collect::<Result<Vec<_>, _>>()?;

    Ok(seasons)
```

- [ ] **Step 3: Fix `get_players_for_season`**

Find the `get_players_for_season` function. Replace the query_map chain:

```rust
// BEFORE:
    let players = stmt
        .query_map(rusqlite::params![season_id], |row| row_to_player_season(row))
        .map_err(|e| StorageError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(players)
```

With:

```rust
// AFTER:
    let players: Vec<PlayerSeasonData> = stmt
        .query_map(rusqlite::params![season_id], |row| row_to_player_season(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(players)
```

Note: The `.map_err(|e| StorageError::Database(e.to_string()))?` is also removed. The `From<rusqlite::Error> for StorageError` impl handles the conversion via `?` alone.

- [ ] **Step 4: Fix `get_player_career`**

Find the `get_player_career` function. Replace the query_map chain:

```rust
// BEFORE:
    let career = stmt
        .query_map(rusqlite::params![save_id, player_id], |row| row_to_player_season(row))
        .map_err(|e| StorageError::Database(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(career)
```

With:

```rust
// AFTER:
    let career: Vec<PlayerSeasonData> = stmt
        .query_map(rusqlite::params![save_id, player_id], |row| row_to_player_season(row))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(career)
```

- [ ] **Step 5: Run all tests**

```bash
cd src-tauri && cargo test
```

Expected: All 219 tests pass. The behavioral change only affects error paths — all existing tests exercise happy paths where row mapping succeeds.

- [ ] **Step 6: Verify no `filter_map` remains in production code**

```bash
cd src-tauri && grep -n "filter_map" src/storage/mod.rs | grep -v "#\[cfg(test)\]" | grep -v "mod tests" | grep -v "^\s*//"
```

Expected: No matches outside the test module. (The test module uses `filter_map` in its own code, which is acceptable.)

## Dependencies

None. This task is independent of all other refactor tasks.

## Success Criteria

- All 219 tests pass without modification
- No `.filter_map(|r| r.ok())` in production query paths (only in test code)
- No `.map_err(|e| StorageError::Database(e.to_string()))` in `get_players_for_season` or `get_player_career` (redundant with `From` impl)

## Tests

### Test 1: Happy path unchanged

**What to test:** All existing tests pass — the change only affects error paths.

**Feasibility:** ✅ `cargo test` verifies.

### Test 2: Error propagation works

**What to test:** If a row fails to map, the function returns an error instead of silently dropping the row.

**Feasibility:** ✅ Verified by code review — `collect::<Result<Vec<_>, _>>()?` propagates the first error. Testing this directly would require corrupting the schema mid-query, which is impractical without mocking. The type system guarantees the behavior.
