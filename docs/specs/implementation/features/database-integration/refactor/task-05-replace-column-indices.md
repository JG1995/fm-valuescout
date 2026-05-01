# Task 05 - Replace Hardcoded Column Indices

## Overview

Replace positional column indices (`row.get(0)`, `row.get(14)`, etc.) with named column references (`row.get("id")`, `row.get("data")`) in `row_to_player_season` and its associated SQL queries. This eliminates a fragile mapping that silently breaks if the SELECT column order changes.

## Files to Modify

- `src-tauri/src/storage/mod.rs`

## Context

`row_to_player_season` (line ~651) uses 16 positional indices to extract fields from a SQL result row. The column order is defined by the SELECT statements in `get_players_for_season` (line ~686) and `get_player_career` (line ~713). If anyone adds, removes, or reorders a column in these SELECTs, the indices silently read wrong fields — no compile error, just wrong data.

The function also has a gap: index 14 is `data` (JSON blob), but `contract_expires` at index 15 is extracted separately. The comment "Index 14 is the JSON data blob" is the only documentation of this mapping.

rusqlite supports `row.get("column_name")` via the `RowIndex` trait implemented for `&str`. This is zero-cost at runtime (resolves to index at first call per row).

## Steps

- [ ] **Step 1: Add `AS player_name` alias to SQL in `get_players_for_season`**

Find the SQL in `get_players_for_season` (the `conn.prepare(...)` call). Change `p.name` to `p.name AS player_name`:

```rust
// BEFORE:
    let mut stmt = conn.prepare(
        "SELECT ps.id, ps.player_id, ps.season_id, p.fm_uid, p.name,
                ps.club, ps.age, ps.nationality, ps.position, ps.minutes,
                ps.appearances_started, ps.appearances_sub, ps.wage_per_week,
                ps.transfer_value_high, ps.data, ps.contract_expires
         FROM player_seasons ps
         JOIN players p ON ps.player_id = p.id
         WHERE ps.season_id = ?1
         ORDER BY p.name ASC",
    )?;
```

```rust
// AFTER:
    let mut stmt = conn.prepare(
        "SELECT ps.id, ps.player_id, ps.season_id, p.fm_uid, p.name AS player_name,
                ps.club, ps.age, ps.nationality, ps.position, ps.minutes,
                ps.appearances_started, ps.appearances_sub, ps.wage_per_week,
                ps.transfer_value_high, ps.data, ps.contract_expires
         FROM player_seasons ps
         JOIN players p ON ps.player_id = p.id
         WHERE ps.season_id = ?1
         ORDER BY p.name ASC",
    )?;
```

- [ ] **Step 2: Add `AS player_name` alias to SQL in `get_player_career`**

Same change in `get_player_career`:

```rust
// BEFORE:
    let mut stmt = conn.prepare(
        "SELECT ps.id, ps.player_id, ps.season_id, p.fm_uid, p.name,
                ps.club, ps.age, ps.nationality, ps.position, ps.minutes,
                ps.appearances_started, ps.appearances_sub, ps.wage_per_week,
                ps.transfer_value_high, ps.data, ps.contract_expires
         FROM player_seasons ps
         JOIN players p ON ps.player_id = p.id
         JOIN seasons s ON ps.season_id = s.id
         WHERE p.save_id = ?1 AND ps.player_id = ?2
         ORDER BY s.in_game_date ASC",
    )?;
```

```rust
// AFTER:
    let mut stmt = conn.prepare(
        "SELECT ps.id, ps.player_id, ps.season_id, p.fm_uid, p.name AS player_name,
                ps.club, ps.age, ps.nationality, ps.position, ps.minutes,
                ps.appearances_started, ps.appearances_sub, ps.wage_per_week,
                ps.transfer_value_high, ps.data, ps.contract_expires
         FROM player_seasons ps
         JOIN players p ON ps.player_id = p.id
         JOIN seasons s ON ps.season_id = s.id
         WHERE p.save_id = ?1 AND ps.player_id = ?2
         ORDER BY s.in_game_date ASC",
    )?;
```

- [ ] **Step 3: Replace positional indices in `row_to_player_season`**

Replace the entire `row_to_player_season` function:

```rust
// BEFORE:
fn row_to_player_season(row: &rusqlite::Row) -> rusqlite::Result<PlayerSeasonData> {
    // Index 14 is the JSON data blob — extract first for graceful deserialization
    let data_json: String = row.get(14)?;
    let data = serde_json::from_str::<ParsedPlayer>(&data_json).ok();

    Ok(PlayerSeasonData {
        id: row.get(0)?,
        player_id: row.get(1)?,
        season_id: row.get(2)?,
        fm_uid: row.get(3)?,
        player_name: row.get(4)?,
        club: row.get(5)?,
        age: row.get(6)?,
        nationality: row.get(7)?,
        position: row.get(8)?,
        minutes: row.get(9)?,
        appearances_started: row.get(10)?,
        appearances_sub: row.get(11)?,
        wage_per_week: row.get(12)?,
        transfer_value_high: row.get(13)?,
        contract_expires: row.get(15)?,
        data,
    })
}
```

With:

```rust
// AFTER:
/// Deserialize a database row into PlayerSeasonData.
/// Uses named column references — robust against SELECT column reordering.
/// Handles JSON blob deserialization with graceful degradation (None on failure).
fn row_to_player_season(row: &rusqlite::Row) -> rusqlite::Result<PlayerSeasonData> {
    let data_json: String = row.get("data")?;
    let data = serde_json::from_str::<ParsedPlayer>(&data_json).ok();

    Ok(PlayerSeasonData {
        id: row.get("id")?,
        player_id: row.get("player_id")?,
        season_id: row.get("season_id")?,
        fm_uid: row.get("fm_uid")?,
        player_name: row.get("player_name")?,
        club: row.get("club")?,
        age: row.get("age")?,
        nationality: row.get("nationality")?,
        position: row.get("position")?,
        minutes: row.get("minutes")?,
        appearances_started: row.get("appearances_started")?,
        appearances_sub: row.get("appearances_sub")?,
        wage_per_week: row.get("wage_per_week")?,
        transfer_value_high: row.get("transfer_value_high")?,
        contract_expires: row.get("contract_expires")?,
        data,
    })
}
```

- [ ] **Step 4: Run all tests**

```bash
cd src-tauri && cargo test
```

Expected: All 219 tests pass. The named references resolve to the same columns as the positional indices. Key tests:

- `get_players_for_season_basic` — verifies all fields map correctly
- `get_player_career_basic` — verifies career data across seasons
- `get_players_for_season_json_failure_graceful` — verifies graceful JSON degradation still works
- Integration test `full_import_flow` — end-to-end field verification

- [ ] **Step 5: Verify no positional indices remain in `row_to_player_season`**

```bash
cd src-tauri && grep -n "row.get([0-9]" src/storage/mod.rs | grep -v "#\[cfg(test)\]" | grep -v "mod tests"
```

Expected: No matches outside the test module. (Tests may use positional indices in their own query code, which is acceptable.)

## Dependencies

None. This task is independent of all other refactor tasks.

## Success Criteria

- All 219 tests pass without modification
- `row_to_player_season` uses only named column references (no `row.get(N)`)
- Both SQL queries use `p.name AS player_name` alias
- No positional column indices in production retrieval code

## Tests

### Test 1: Named references resolve correctly

**What to test:** All 16 fields in `PlayerSeasonData` map to the correct columns.

**Feasibility:** ✅ `get_players_for_season_basic` test verifies player name, fm_uid, club, and data. `import_season_extracts_queryable_columns` test verifies all queryable columns match. These existing tests cover the mapping exhaustively.
