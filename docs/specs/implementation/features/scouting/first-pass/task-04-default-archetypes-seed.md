# Task 04 - Default Archetypes Seed Data

## Overview

Create a function that seeds the database with default archetypes from `docs/notes/metrics.md`. This function runs once at app startup (or when the table is empty) and inserts all built-in archetypes marked as `is_default = true`.

## Files to Create/Modify

- Create: `src-tauri/src/storage/seed_archetypes.rs` — Seed function and default data
- Modify: `src-tauri/src/storage/mod.rs` — Add module and re-export
- Modify: `src-tauri/src/lib.rs` — Call seed function after DB init

## Context

### Default Archetypes (from `docs/notes/metrics.md`)

The metrics.md file defines archetypes per position with in-possession and out-of-possession metric weights. Per the design spec, these are combined into a single archetype (not separate in/out selections).

**Note on archetype naming from metrics.md:** Some positions have different archetype names for in-possession vs out-of-possession. The design spec says we combine these into a single archetype. When the names differ, we use the in-possession name for the archetype and combine both metric sets.

**Role mapping from metrics.md positions to `Role` enum:**
- Goalkeeper → `"GK"`
- Center Back → `"D"` (with sides C)
- Right/Left Back → `"D"` (with sides R/L)
- Defensive Midfielder → `"DM"`
- Right/Left Wingback → `"WB"`
- Central Midfielder → `"M"`
- Left/Right Winger → `"AM"` (wingers map to AM in FM's role system — but actually the Role enum is: GK, D, WB, DM, M, AM, ST. Check: the FM position strings use "AM" for wingers? No — check the design spec which says "Winger" is a separate archetype group. Since the Role enum only has GK, D, WB, DM, M, AM, ST, winger archetypes would use role `"AM"` (or we need to check how FM positions work in the parser.)

**Important clarification:** Looking at the design spec, it lists archetype groups: GK, CB, FB, DM, WB, CM, Winger, AM, ST. But the `Role` enum has: GK, D, WB, DM, M, AM, ST. The archetype groups CB and FB both map to `D` role, and "Winger" maps to... we need a way to distinguish winger from other AM. However, the archetype `role` field is just a string in the DB, so we can use a finer-grained role mapping:
- GK → `"GK"`
- CB → `"CB"`
- FB → `"FB"`
- DM → `"DM"`
- WB → `"WB"`
- CM → `"CM"`
- Winger → `"W"`
- AM → `"AM"`
- ST → `"ST"`

This is fine because the `role` column is `TEXT` — it doesn't need to match the parser's `Role` enum. The archetype role is used to match against a player's positions for scoring eligibility.

**Metric key mapping:** The metric names in metrics.md (e.g., "Pass Completion Ratio") need to be mapped to `ParsedPlayer` field names (e.g., `"pass_completion_rate"`). The complete mapping is defined in this task.

## Steps

- [ ] **Step 1: Define the metric key mapping**

Create `src-tauri/src/storage/seed_archetypes.rs` with the human-readable name to `ParsedPlayer` field name mapping and seed function. Start with the tests:

```rust
use rusqlite::Connection;

use super::archetypes::{Archetype, MetricWeight, create_archetype};
use super::error::StorageError;
use super::schema::init_schema;

/// Map human-readable metric names (from metrics.md) to ParsedPlayer field names.
/// These are the keys stored in `MetricWeight.metric_key` and used by the
/// TypeScript scoring engine to extract values from player data.
///
/// IMPORTANT: Keys must exactly match field names in `ParsedPlayer` structs
/// defined in `src-tauri/src/parser/types.rs`. Some metrics.md names describe
/// concepts that don't have a direct field — we use the closest available field.
///
/// Known limitations (no ParsedPlayer field exists):
/// - "Shots on Target Ratio" — use `attacking.shots_on_target_per_90` as proxy
/// - "Conversion Rate" — no direct field; use `attacking.goals_per_90` as proxy
/// - "Average Minutes per Goal" — no direct field; use `attacking.goals_per_90` inverted
/// - "Save Ratio" — no direct field; use `goalkeeping.expected_save_pct` as proxy
/// - Many total fields used where per-90 was specified (e.g., `crosses_completed` not `crosses_completed_per_90`)
pub fn metric_key_for_name(name: &str) -> Option<&'static str> {
    Some(match name {
        // Attacking — use actual ParsedPlayer field names
        "Goals per 90" => "attacking.goals_per_90",
        "Goals" => "attacking.goals",
        "xG per 90" => "attacking.xg_per_90",
        "Non Penalty Expected Goals per 90" | "NPxG per 90" => "attacking.np_xg_per_90",
        "Expected Goals per Shot" => "attacking.xg_per_shot",
        "Shots per 90" => "attacking.shots_per_90",
        // No shots_on_target_ratio exists; use per-90 as scoring proxy
        "Shots on Target Ratio" => "attacking.shots_on_target_per_90",
        // No conversion_rate exists; use goals_per_90 as proxy
        "Conversion Rate" => "attacking.goals_per_90",
        // No avg_minutes_per_goal exists; use goals_per_90 (inverted=true in archetype)
        "Average Minutes per Goal" => "attacking.goals_per_90",

        // Chance Creation — many are totals, not per-90
        "Assists per 90" => "chance_creation.assists_per_90",
        "Expected Assists per 90" | "xA per 90" => "chance_creation.xa_per_90",
        "Chances Created per 90" => "chance_creation.chances_created_per_90",
        "Key Passes per 90" => "chance_creation.key_passes_per_90",
        // These are totals, not per-90 — metric name says per-90 but field is total
        "Crosses Completed per 90" => "chance_creation.crosses_completed",
        "Open Play Crosses Completed per 90" => "chance_creation.open_play_crosses_completed",

        // Passing / Progression
        "Pass Completion Ratio" => "chance_creation.pass_completion_rate",
        // These are totals, not per-90
        "Passes Attempted per 90" => "chance_creation.passes_attempted",
        "Passes Completed per 90" => "chance_creation.passes_completed",
        "Progressive Passes per 90" => "chance_creation.progressive_passes_per_90",

        // Movement
        "Dribbles Made per 90" => "movement.dribbles_per_90",
        "Distance Covered per 90" => "movement.distance_per_90",
        "High Intensity Sprints per 90" => "movement.sprints_per_90",
        "Possession Lost per 90" => "movement.possession_lost_per_90",

        // Defending — many are totals or per-90 as defined
        "Tackles per 90" => "defending.tackles_per_90",
        "Tackle Completion Ratio" => "defending.tackle_completion_rate",
        "Interceptions per 90" => "defending.interceptions_per_90",
        "Possession Won per 90" => "defending.possession_won_per_90",
        // totals, not per-90
        "Pressures Completed per 90" => "defending.pressures_completed",
        "Blocks per 90" => "defending.blocks",
        "Clearances per 90" => "defending.clearances_per_90",
        "Tackles Attempted per 90" => "defending.tackles_attempted",

        // Aerial
        "Headers Won Ratio" => "aerial.aerial_challenge_rate",
        "Key Headers per 90" => "aerial.key_headers_per_90",

        // Goalkeeping — note: expected_goals_prevented is total, saves_held is total
        "Expected Goals Prevented per 90" => "goalkeeping.expected_goals_prevented",
        // No save_ratio exists; use expected_save_pct as closest proxy
        "Save Ratio" => "goalkeeping.expected_save_pct",
        // saves_held is total
        "Saves Held per 90" => "goalkeeping.saves_held",

        // Discipline
        "Fouls Made per 90" => "discipline.fouls_made_per_90",
        "Fouls Against per 90" => "discipline.fouls_against_per_90",
        // mistakes_leading_to_goal is total
        "Mistakes Leading to Goal" => "discipline.mistakes_leading_to_goal",

        // Match Impact
        "Average Rating" => "match_outcome.average_rating",

        _ => return None,
    }),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn metric_key_mapping_known_metrics() {
        assert_eq!(metric_key_for_name("Goals per 90"), Some("attacking.goals_per_90"));
        assert_eq!(metric_key_for_name("Pass Completion Ratio"), Some("chance_creation.pass_completion_rate"));
        assert_eq!(metric_key_for_name("Progressive Passes per 90"), Some("chance_creation.progressive_passes_per_90"));
    }

    #[test]
    fn metric_key_mapping_unknown_returns_none() {
        assert_eq!(metric_key_for_name("Unknown Metric"), None);
    }

    #[test]
    fn seed_archetypes_creates_defaults() {
        let conn = setup_test_db();
        seed_default_archetypes(&conn).unwrap();
        // Check we have archetypes for each role
        let gk_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM archetypes WHERE role = 'GK'", [],
            |r| r.get(0),
        ).unwrap();
        assert!(gk_count >= 2, "Expected at least 2 GK archetypes, got {}", gk_count);

        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM archetypes WHERE is_default = 1", [],
            |r| r.get(0),
        ).unwrap();
        assert!(total >= 18, "Expected at least 18 default archetypes, got {}", total);
    }

    #[test]
    fn seed_archetypes_is_idempotent() {
        let conn = setup_test_db();
        seed_default_archetypes(&conn).unwrap();
        seed_default_archetypes(&conn).unwrap(); // Second call should not fail or duplicate
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM archetypes WHERE is_default = 1", [],
            |r| r.get(0),
        ).unwrap();
        // Count should be the same after double-seed
        assert!(total >= 18);
    }

    #[test]
    fn seed_archetypes_weights_sum_to_one() {
        let conn = setup_test_db();
        seed_default_archetypes(&conn).unwrap();
        let mut stmt = conn.prepare(
            "SELECT metrics_json FROM archetypes WHERE is_default = 1"
        ).unwrap();
        let rows: Vec<String> = stmt.query_map([], |r| r.get(0)).unwrap()
            .filter_map(|r| r.ok()).collect();
        for json in &rows {
            let metrics: Vec<MetricWeight> = serde_json::from_str(json).unwrap();
            let sum: f64 = metrics.iter().map(|m| m.weight).sum();
            assert!((sum - 1.0).abs() < 0.01, "Weights should sum to ~1.0, got {}", sum);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test --lib storage::seed_archetypes`
Expected: FAIL — `seed_default_archetypes` not defined yet.

- [ ] **Step 3: Implement the seed function**

Add the `seed_default_archetypes` function to `src-tauri/src/storage/seed_archetypes.rs`, between the `metric_key_for_name` function and the `#[cfg(test)]` block:

```rust
/// Helper to build a MetricWeight from human-readable name.
/// Panics in debug if metric name not found (programming error in defaults).
fn mw(name: &str, weight: f64, inverted: bool) -> MetricWeight {
    let key = metric_key_for_name(name)
        .unwrap_or_else(|| panic!("Unknown metric name in defaults: '{}'", name));
    MetricWeight {
        metric_key: key.to_string(),
        weight,
        inverted,
    }
}

/// All default archetypes as (name, role, metrics) tuples.
/// Derived from docs/notes/metrics.md.
fn default_archetypes() -> Vec<(&'static str, &'static str, Vec<MetricWeight>)> {
    vec![
        // ── Goalkeeper ──────────────────────────────────────────────
        ("Traditional Goalkeeper", "GK", vec![
            mw("Pass Completion Ratio", 0.70, false),
            mw("Passes Attempted per 90", 0.30, false),
            mw("Expected Goals Prevented per 90", 0.50, false),
            mw("Save Ratio", 0.30, false),
            mw("Saves Held per 90", 0.20, false),
        ]),
        ("Ball-Playing Goalkeeper", "GK", vec![
            mw("Progressive Passes per 90", 0.40, false),
            mw("Pass Completion Ratio", 0.30, false),
            mw("Passes Attempted per 90", 0.20, false),
            mw("Expected Assists per 90", 0.10, false),
            mw("Expected Goals Prevented per 90", 0.40, false),
            mw("Interceptions per 90", 0.30, false),
            mw("High Intensity Sprints per 90", 0.20, false),
            mw("Distance Covered per 90", 0.10, false),
        ]),

        // ── Center Back ─────────────────────────────────────────────
        ("Traditional Center Back", "CB", vec![
            mw("Pass Completion Ratio", 0.60, false),
            mw("Passes Completed per 90", 0.40, false),
            mw("Headers Won Ratio", 0.30, false),
            mw("Interceptions per 90", 0.25, false),
            mw("Possession Won per 90", 0.20, false),
            mw("Blocks per 90", 0.15, false),
            mw("Clearances per 90", 0.10, false),
        ]),
        ("Ball-Playing Center Back", "CB", vec![
            mw("Progressive Passes per 90", 0.45, false),
            mw("Pass Completion Ratio", 0.25, false),
            mw("Passes Attempted per 90", 0.20, false),
            mw("Key Passes per 90", 0.10, false),
            mw("Headers Won Ratio", 0.30, false),
            mw("Interceptions per 90", 0.25, false),
            mw("Possession Won per 90", 0.20, false),
            mw("Blocks per 90", 0.15, false),
            mw("Clearances per 90", 0.10, false),
        ]),

        // ── Full Back ───────────────────────────────────────────────
        ("Full Back", "FB", vec![
            mw("Pass Completion Ratio", 0.40, false),
            mw("Passes Attempted per 90", 0.30, false),
            mw("Progressive Passes per 90", 0.20, false),
            mw("Distance Covered per 90", 0.10, false),
            mw("Pressures Completed per 90", 0.35, false),
            mw("Possession Won per 90", 0.25, false),
            mw("High Intensity Sprints per 90", 0.25, false),
            mw("Tackles per 90", 0.15, false),
        ]),
        ("Offensive Full Back", "FB", vec![
            mw("Expected Assists per 90", 0.35, false),
            mw("Open Play Crosses Completed per 90", 0.25, false),
            mw("Dribbles Made per 90", 0.20, false),
            mw("Key Passes per 90", 0.20, false),
            mw("Tackle Completion Ratio", 0.35, false),
            mw("Interceptions per 90", 0.25, false),
            mw("Headers Won Ratio", 0.20, false),
            mw("Blocks per 90", 0.20, false),
        ]),

        // ── Defensive Midfielder ────────────────────────────────────
        ("Defensive Midfielder", "DM", vec![
            mw("Pass Completion Ratio", 0.45, false),
            mw("Passes Completed per 90", 0.30, false),
            mw("Progressive Passes per 90", 0.15, false),
            mw("Possession Lost per 90", 0.10, true),
            mw("Pressures Completed per 90", 0.35, false),
            mw("Possession Won per 90", 0.30, false),
            mw("Tackles per 90", 0.20, false),
            mw("Interceptions per 90", 0.15, false),
        ]),
        ("Playmaker", "DM", vec![
            mw("Progressive Passes per 90", 0.40, false),
            mw("Expected Assists per 90", 0.20, false),
            mw("Passes Attempted per 90", 0.20, false),
            mw("Key Passes per 90", 0.20, false),
            mw("Interceptions per 90", 0.45, false),
            mw("Distance Covered per 90", 0.25, false),
            mw("Fouls Made per 90", 0.20, true),
            mw("Mistakes Leading to Goal", 0.10, true),
        ]),

        // ── Wing Back ───────────────────────────────────────────────
        ("Wing Back", "WB", vec![
            mw("Crosses Completed per 90", 0.35, false),
            mw("Passes Attempted per 90", 0.25, false),
            mw("Progressive Passes per 90", 0.20, false),
            mw("Distance Covered per 90", 0.20, false),
            mw("Pressures Completed per 90", 0.40, false),
            mw("Possession Won per 90", 0.30, false),
            mw("High Intensity Sprints per 90", 0.20, false),
            mw("Tackles Attempted per 90", 0.10, false),
        ]),
        ("Offensive Wing Back", "WB", vec![
            mw("Expected Assists per 90", 0.35, false),
            mw("Dribbles Made per 90", 0.25, false),
            mw("Chances Created per 90", 0.20, false),
            mw("Open Play Crosses Completed per 90", 0.20, false),
            mw("Distance Covered per 90", 0.40, false),
            mw("Interceptions per 90", 0.30, false),
            mw("Tackle Completion Ratio", 0.20, false),
            mw("Blocks per 90", 0.10, false),
        ]),

        // ── Central Midfielder ──────────────────────────────────────
        ("All-Rounder", "CM", vec![
            mw("Pass Completion Ratio", 0.30, false),
            mw("Passes Attempted per 90", 0.25, false),
            mw("Progressive Passes per 90", 0.25, false),
            mw("Distance Covered per 90", 0.20, false),
            mw("Tackles per 90", 0.30, false),
            mw("Interceptions per 90", 0.25, false),
            mw("Pressures Completed per 90", 0.25, false),
            mw("Possession Won per 90", 0.20, false),
        ]),
        ("Box-to-Box", "CM", vec![
            mw("Expected Goals per 90", 0.30, false),
            mw("Distance Covered per 90", 0.25, false),
            mw("Shots per 90", 0.25, false),
            mw("Progressive Passes per 90", 0.20, false),
            mw("Tackles per 90", 0.30, false),
            mw("Interceptions per 90", 0.25, false),
            mw("Pressures Completed per 90", 0.25, false),
            mw("Possession Won per 90", 0.20, false),
        ]),
        ("Playmaker", "CM", vec![
            mw("Expected Assists per 90", 0.35, false),
            mw("Progressive Passes per 90", 0.30, false),
            mw("Key Passes per 90", 0.20, false),
            mw("Pass Completion Ratio", 0.15, false),
            mw("Interceptions per 90", 0.40, false),
            mw("Tackle Completion Ratio", 0.30, false),
            mw("Blocks per 90", 0.20, false),
            mw("Distance Covered per 90", 0.10, false),
        ]),

        // ── Winger ──────────────────────────────────────────────────
        ("Traditional Winger", "W", vec![
            mw("Crosses Completed per 90", 0.40, false),
            mw("Dribbles Made per 90", 0.30, false),
            mw("Key Passes per 90", 0.20, false),
            mw("Pass Completion Ratio", 0.10, false),
            mw("High Intensity Sprints per 90", 0.40, false),
            mw("Expected Goals per 90", 0.30, false),
            mw("Fouls Against per 90", 0.20, false),
        ]),
        ("Goalscoring Winger", "W", vec![
            mw("Non Penalty Expected Goals per 90", 0.45, false),
            mw("Shots on Target Ratio", 0.25, false),
            mw("Conversion Rate", 0.20, false),
            mw("Goals per 90", 0.10, false),
            mw("High Intensity Sprints per 90", 0.40, false),
            mw("Expected Goals per 90", 0.30, false),
            mw("Fouls Against per 90", 0.20, false),
        ]),
        ("Inside Forward", "W", vec![
            mw("xG per 90", 0.30, false),
            mw("Expected Assists per 90", 0.25, false),
            mw("Dribbles Made per 90", 0.25, false),
            mw("Key Passes per 90", 0.20, false),
            mw("Distance Covered per 90", 0.35, false),
            mw("Pressures Completed per 90", 0.30, false),
            mw("Tackles per 90", 0.20, false),
            mw("Interceptions per 90", 0.15, false),
        ]),

        // ── Attacking Midfielder ────────────────────────────────────
        ("Running Attacking Mid", "AM", vec![
            mw("Non Penalty Expected Goals per 90", 0.55, false),
            mw("Shots per 90", 0.25, false),
            mw("Distance Covered per 90", 0.20, false),
            mw("Goals per 90", 0.15, false),
            mw("High Intensity Sprints per 90", 0.40, false),
            mw("Fouls Against per 90", 0.20, false),
        ]),
        ("Playmaker", "AM", vec![
            mw("Expected Assists per 90", 0.35, false),
            mw("Key Passes per 90", 0.30, false),
            mw("Progressive Passes per 90", 0.20, false),
            mw("Pass Completion Ratio", 0.15, false),
            mw("Pressures Completed per 90", 0.40, false),
            mw("Distance Covered per 90", 0.30, false),
            mw("Tackles per 90", 0.15, false),
            mw("Interceptions per 90", 0.15, false),
        ]),

        // ── Striker ─────────────────────────────────────────────────
        ("Creative Forward", "ST", vec![
            mw("Expected Assists per 90", 0.35, false),
            mw("Key Passes per 90", 0.25, false),
            mw("Progressive Passes per 90", 0.20, false),
            mw("Pass Completion Ratio", 0.10, false),
            mw("Dribbles Made per 90", 0.10, false),
            mw("High Intensity Sprints per 90", 0.40, false),
            mw("Expected Goals per 90", 0.30, false),
            mw("Fouls Against per 90", 0.20, false),
        ]),
        ("Goalscoring Forward", "ST", vec![
            mw("Non Penalty Expected Goals per 90", 0.45, false),
            mw("Expected Goals per Shot", 0.25, false),
            mw("Shots on Target Ratio", 0.15, false),
            mw("Conversion Rate", 0.10, false),
            mw("Average Minutes per Goal", 0.05, true),
            mw("Pressures Completed per 90", 0.40, false),
            mw("Possession Won per 90", 0.30, false),
            mw("Distance Covered per 90", 0.20, false),
            mw("Tackles per 90", 0.10, false),
        ]),
    ]
}

/// Seed the database with default archetypes.
/// Only inserts if the archetypes table has no default rows.
/// Uses INSERT OR IGNORE to handle the UNIQUE(name, role) constraint idempotently.
pub fn seed_default_archetypes(conn: &Connection) -> Result<(), StorageError> {
    // Check if defaults already exist
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM archetypes WHERE is_default = 1",
        [],
        |row| row.get(0),
    )?;
    if count > 0 {
        return Ok(()); // Already seeded
    }

    for (name, role, metrics) in default_archetypes() {
        // Use the archetypes module's create function, but bypass the unique
        // check by using INSERT OR IGNORE directly for seed data
        let mut normalized = metrics.clone();
        crate::storage::archetypes::normalize_weights(&mut normalized);
        let metrics_json = serde_json::to_string(&normalized)
            .map_err(|e| StorageError::Database(format!("Failed to serialize metrics: {}", e)))?;

        conn.execute(
            "INSERT OR IGNORE INTO archetypes (name, role, metrics_json, is_default) VALUES (?1, ?2, ?3, 1)",
            rusqlite::params![name, role, metrics_json],
        )?;
    }

    Ok(())
}
```

- [ ] **Step 4: Register the module**

In `src-tauri/src/storage/mod.rs`, add:

```rust
pub mod seed_archetypes;
pub use seed_archetypes::seed_default_archetypes;
```

- [ ] **Step 5: Call seed at startup**

In `src-tauri/src/lib.rs`, inside the `.setup(|app| { ... })` closure, after `app.manage(DbState { ... })` and before `Ok(())`, add:

```rust
// Seed default archetypes (idempotent — no-op if already seeded)
{
    let conn_inner = app.state::<DbState>();
    let conn = conn_inner.conn.lock().map_err(|e| format!("Lock error: {}", e))?;
    storage::seed_default_archetypes(&conn)
        .map_err(|e| format!("Failed to seed default archetypes: {}", e))?;
}
```

- [ ] **Step 6: Run tests**

Run: `cd src-tauri && cargo test --lib storage::seed_archetypes`
Expected: ALL PASS.

Run: `cd src-tauri && cargo test --lib`
Expected: ALL PASS — no regressions.

## Dependencies

- Task 01 (schema migration) — archetypes table must exist
- Task 02 (archetype types) — `MetricWeight`, validation functions
- Task 03 (archetype CRUD) — `normalize_weights` function

## Success Criteria

- At least 18 default archetypes are seeded (2 per position group × 9 groups, some have 3)
- Weights for each archetype sum to ~1.0
- Seeding is idempotent — running twice produces same count
- All archetype metrics use valid `ParsedPlayer` field key format (`category.field_name`)
- App startup seeds defaults when table is empty

## Tests

### Test 1: Metric key mapping

**What to test:** Known metric names map to valid field keys; unknown names return None.
**Feasibility:** ✅ Can be tested — pure function.

### Test 2: Seed creates defaults

**What to test:** After seeding, archetypes table has ≥18 defaults across all roles.
**Feasibility:** ✅ Can be tested — in-memory SQLite.

### Test 3: Idempotency

**What to test:** Seeding twice doesn't duplicate rows.
**Feasibility:** ✅ Can be tested — check count after double-seed.

### Test 4: Weight normalization

**What to test:** All seeded archetypes have weights summing to ~1.0.
**Feasibility:** ✅ Can be tested — read metrics_json, sum weights.
