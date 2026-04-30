# Task 04 - Replace assign_stat with Macro-Driven Stat Dispatch

## Overview

The `assign_stat` function in `src-tauri/src/parser/mod.rs:418-487` is a 70-arm `match field_name` that must be kept in sync with `STAT_COLUMNS` and the stat structs in `types.rs`. Replace it with a macro that generates both the `STAT_COLUMNS` table and the assignment logic from a single declaration, eliminating the two-location synchronization problem.

## Files to Create/Modify

- Modify: `src-tauri/src/parser/mod.rs` — replace `STAT_COLUMNS` and `assign_stat` with macro-generated code
- Modify: `src-tauri/src/parser/types.rs` — no changes (structs remain the same)

## Background

Currently there are three places that must be updated for every new stat column:

1. `STAT_COLUMNS` array (line 37) — maps field name to CSV header name and allow_negative flag
2. `assign_stat` match (line 418) — maps field name to struct field assignment
3. Stat structs in `types.rs` — the actual `Option<f64>` fields

This task consolidates #1 and #2 into a single macro invocation. #3 remains manual (struct fields must be declared), but the macro ensures every stat listed in `STAT_COLUMNS` has a corresponding assignment arm — if you add a stat to the table and forget to add the match arm, the macro won't compile.

## Steps

- [ ] **Step 1: Write the stat definition macro**

In `src-tauri/src/parser/mod.rs`, replace the `STAT_COLUMNS` constant and `assign_stat` function (lines 16-111 and 418-487) with a single macro:

```rust
/// Macro that generates:
/// 1. STAT_COLUMNS — the table of (field_name, csv_name, allow_negative)
/// 2. assign_stat  — the match that routes stat values to struct fields
///
/// Every stat is declared exactly once. Adding a new stat requires one line here
/// and the corresponding field in the stat struct in types.rs.
macro_rules! define_stats {
    ($($field:literal => { csv: $csv:literal, neg: $neg:expr, path: $path:expr }),* $(,)?) => {
        const STAT_COLUMNS: &[(&str, ColumnDef)] = &[
            $(
                ($field, ColumnDef { csv_name: $csv, allow_negative: $neg }),
            )*
        ];

        fn assign_stat(player: &mut ParsedPlayer, field_name: &str, value: Option<f64>) {
            match field_name {
                $(
                    $field => { let target: &mut Option<f64> = $path(player); *target = value; }
                )*
                _ => {} // Unknown stat name, skip
            }
        }
    };
}
```

- [ ] **Step 2: Invoke the macro with all stat definitions**

Replace the old `STAT_COLUMNS` const and `assign_stat` fn body with the macro invocation. Each line maps one stat:

```rust
define_stats! {
    // Attacking
    "goals" => { csv: "Goals", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.goals },
    "goals_from_outside_box" => { csv: "Goals From Outside The Box", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.goals_from_outside_box },
    "xg" => { csv: "xG", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.xg },
    "np_xg" => { csv: "NP-xG", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.np_xg },
    "xg_overperformance" => { csv: "xG-OP", neg: true, path: |p: &mut ParsedPlayer| &mut p.attacking.xg_overperformance },
    "xg_per_shot" => { csv: "xG/shot", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.xg_per_shot },
    "shots" => { csv: "Shots", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.shots },
    "shots_from_outside_box_per_90" => { csv: "Shots From Outside The Box Per 90 minutes", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.shots_from_outside_box_per_90 },
    "shots_on_target" => { csv: "ShT", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.shots_on_target },
    "penalties_taken" => { csv: "Pens", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.penalties_taken },
    "penalties_scored" => { csv: "Pens S", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.penalties_scored },
    "free_kick_shots" => { csv: "Free Kick Shots", neg: false, path: |p: &mut ParsedPlayer| &mut p.attacking.free_kick_shots },
    // Chance creation
    "assists" => { csv: "Assists", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.assists },
    "xa" => { csv: "xA", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.xa },
    "chances_created_per_90" => { csv: "Ch C/90", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.chances_created_per_90 },
    "clear_cut_chances" => { csv: "CCC", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.clear_cut_chances },
    "key_passes" => { csv: "Key", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.key_passes },
    "open_play_key_passes_per_90" => { csv: "OP-KP/90", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.open_play_key_passes_per_90 },
    "crosses_attempted" => { csv: "Cr A", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.crosses_attempted },
    "crosses_completed" => { csv: "Cr C", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.crosses_completed },
    "open_play_crosses_attempted" => { csv: "OP-Crs A", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.open_play_crosses_attempted },
    "open_play_crosses_completed" => { csv: "OP-Crs C", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.open_play_crosses_completed },
    "passes_attempted" => { csv: "Pas A", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.passes_attempted },
    "passes_completed" => { csv: "Pas C", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.passes_completed },
    "progressive_passes" => { csv: "PsP", neg: false, path: |p: &mut ParsedPlayer| &mut p.chance_creation.progressive_passes },
    // Movement
    "dribbles" => { csv: "Drb", neg: false, path: |p: &mut ParsedPlayer| &mut p.movement.dribbles },
    "distance_km" => { csv: "Distance", neg: false, path: |p: &mut ParsedPlayer| &mut p.movement.distance_km },
    "sprints_per_90" => { csv: "Sprints/90", neg: false, path: |p: &mut ParsedPlayer| &mut p.movement.sprints_per_90 },
    "possession_lost_per_90" => { csv: "Poss Lost/90", neg: false, path: |p: &mut ParsedPlayer| &mut p.movement.possession_lost_per_90 },
    // Defending
    "tackles_attempted" => { csv: "Tck A", neg: false, path: |p: &mut ParsedPlayer| &mut p.defending.tackles_attempted },
    "tackles_completed" => { csv: "Tck C", neg: false, path: |p: &mut ParsedPlayer| &mut p.defending.tackles_completed },
    "key_tackles" => { csv: "K Tck", neg: false, path: |p: &mut ParsedPlayer| &mut p.defending.key_tackles },
    "interceptions" => { csv: "Itc", neg: false, path: |p: &mut ParsedPlayer| &mut p.defending.interceptions },
    "possession_won_per_90" => { csv: "Poss Won/90", neg: false, path: |p: &mut ParsedPlayer| &mut p.defending.possession_won_per_90 },
    "pressures_attempted" => { csv: "Pres A", neg: false, path: |p: &mut ParsedPlayer| &mut p.defending.pressures_attempted },
    "pressures_completed" => { csv: "Pres C", neg: false, path: |p: &mut ParsedPlayer| &mut p.defending.pressures_completed },
    "blocks" => { csv: "Blk", neg: false, path: |p: &mut ParsedPlayer| &mut p.defending.blocks },
    "shots_blocked" => { csv: "Shts Blckd", neg: false, path: |p: &mut ParsedPlayer| &mut p.defending.shots_blocked },
    "clearances" => { csv: "Clearances", neg: false, path: |p: &mut ParsedPlayer| &mut p.defending.clearances },
    // Aerial
    "aerial_challenges_attempted" => { csv: "Hdrs A", neg: false, path: |p: &mut ParsedPlayer| &mut p.aerial.aerial_challenges_attempted },
    "aerial_challenges_won" => { csv: "Hdrs", neg: false, path: |p: &mut ParsedPlayer| &mut p.aerial.aerial_challenges_won },
    "aerial_challenges_lost_per_90" => { csv: "Hdrs L/90", neg: false, path: |p: &mut ParsedPlayer| &mut p.aerial.aerial_challenges_lost_per_90 },
    "key_headers_per_90" => { csv: "K Hdrs/90", neg: false, path: |p: &mut ParsedPlayer| &mut p.aerial.key_headers_per_90 },
    // Goalkeeping
    "clean_sheets" => { csv: "Clean Sheets", neg: false, path: |p: &mut ParsedPlayer| &mut p.goalkeeping.clean_sheets },
    "goals_conceded" => { csv: "Goals Conceded", neg: false, path: |p: &mut ParsedPlayer| &mut p.goalkeeping.goals_conceded },
    "saves_per_90" => { csv: "Saves/90", neg: false, path: |p: &mut ParsedPlayer| &mut p.goalkeeping.saves_per_90 },
    "expected_save_pct" => { csv: "xSv %", neg: false, path: |p: &mut ParsedPlayer| &mut p.goalkeeping.expected_save_pct },
    "expected_goals_prevented" => { csv: "xGP", neg: true, path: |p: &mut ParsedPlayer| &mut p.goalkeeping.expected_goals_prevented },
    "saves_held" => { csv: "Svh", neg: false, path: |p: &mut ParsedPlayer| &mut p.goalkeeping.saves_held },
    "saves_parried" => { csv: "Svp", neg: false, path: |p: &mut ParsedPlayer| &mut p.goalkeeping.saves_parried },
    "saves_tipped" => { csv: "Svt", neg: false, path: |p: &mut ParsedPlayer| &mut p.goalkeeping.saves_tipped },
    "penalties_faced" => { csv: "Pens Faced", neg: false, path: |p: &mut ParsedPlayer| &mut p.goalkeeping.penalties_faced },
    "penalties_saved" => { csv: "Pens Saved", neg: false, path: |p: &mut ParsedPlayer| &mut p.goalkeeping.penalties_saved },
    // Discipline
    "fouls_made" => { csv: "Fouls Made", neg: false, path: |p: &mut ParsedPlayer| &mut p.discipline.fouls_made },
    "fouls_against" => { csv: "Fouls Against", neg: false, path: |p: &mut ParsedPlayer| &mut p.discipline.fouls_against },
    "yellow_cards" => { csv: "Yel", neg: false, path: |p: &mut ParsedPlayer| &mut p.discipline.yellow_cards },
    "red_cards" => { csv: "Red cards", neg: false, path: |p: &mut ParsedPlayer| &mut p.discipline.red_cards },
    "offsides" => { csv: "Off", neg: false, path: |p: &mut ParsedPlayer| &mut p.discipline.offsides },
    "mistakes_leading_to_goal" => { csv: "MLG", neg: false, path: |p: &mut ParsedPlayer| &mut p.discipline.mistakes_leading_to_goal },
    // Match outcome
    "average_rating" => { csv: "Rating", neg: false, path: |p: &mut ParsedPlayer| &mut p.match_outcome.average_rating },
    "player_of_the_match" => { csv: "PoM", neg: false, path: |p: &mut ParsedPlayer| &mut p.match_outcome.player_of_the_match },
    "games_won" => { csv: "Games Won", neg: false, path: |p: &mut ParsedPlayer| &mut p.match_outcome.games_won },
    "games_drawn" => { csv: "Games Drawn", neg: false, path: |p: &mut ParsedPlayer| &mut p.match_outcome.games_drawn },
    "games_lost" => { csv: "Games Lost", neg: false, path: |p: &mut ParsedPlayer| &mut p.match_outcome.games_lost },
    "team_goals" => { csv: "Team Goals", neg: false, path: |p: &mut ParsedPlayer| &mut p.match_outcome.team_goals },
}
```

- [ ] **Step 3: Clean up duplicate comments**

While editing `mod.rs`, also clean up the duplicate comment block noted in the retrospective (lines 30-35). Remove the duplicated "Stat column definitions" comment, keeping only one.

- [ ] **Step 4: Verify compilation**

Run: `cd src-tauri && cargo check`
Expected: Compiles with no errors. The macro expansion produces identical `STAT_COLUMNS` and `assign_stat` as before.

- [ ] **Step 5: Run all tests**

Run: `cd src-tauri && cargo test`
Expected: All 113+ tests pass. The behavior is identical — only the code organization changed.

- [ ] **Step 6: Run the integration test against the sample CSV**

Run: `cd src-tauri && cargo test integration_csv_parser`
Expected: All integration tests pass, confirming the full pipeline still produces correct results.

## Dependencies

- Task 03 should be completed first since both tasks modify `mod.rs`. Completing Task 03 first means this task operates on the already-slimmer file and avoids merge conflicts.

## Success Criteria

- `STAT_COLUMNS` and `assign_stat` are both generated from a single `define_stats!` macro invocation.
- Adding a new stat requires exactly one line in the macro invocation and one field in the stat struct — no other changes.
- The `_ => {}` catch-all in the generated match is acceptable — unknown field names are simply ignored (same behavior as before).
- All existing tests pass without modification.
- `mod.rs` is under 450 lines.

## Tests

### Test 1: All existing tests pass

**What to test:** The refactored stat dispatch produces identical results.

**Feasibility:** ✅ Can be tested — `cargo test` exercises the full stat pipeline including integration tests against the real 257-row CSV.

### Test 2: Compilation catches field name typos

**What to test:** If a stat name in the macro doesn't match a struct field, compilation fails.

**Feasibility:** ✅ Verified by design — the closure `|p: &mut ParsedPlayer| &mut p.attacking.nonexistent` would fail to compile.
