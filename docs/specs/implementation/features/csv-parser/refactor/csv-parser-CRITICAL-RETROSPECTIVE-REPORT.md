# Critical Retrospective Report

## Scope

Feature-level review of the CSV Parser feature. All Rust implementation files under `src-tauri/src/parser/`, `src-tauri/src/commands/`, `src-tauri/src/storage/`, `src-tauri/src/lib.rs`. Design spec at `docs/specs/design/features/csv-parser/2026-04-29-fm-valuescout-csv-parser-design-spec.md`. Implementation plan at `docs/specs/implementation/features/csv-parser/first-pass/`.

**Files covered:**

| File | Lines | Role |
|---|---|---|
| `src-tauri/src/parser/mod.rs` | 574 | Parse orchestration, STAT_COLUMNS, assign_stat |
| `src-tauri/src/parser/fields.rs` | 607 | Individual field parsers |
| `src-tauri/src/parser/types.rs` | 465 | Core types, stat structs |
| `src-tauri/src/parser/headers.rs` | 199 | BOM strip, delimiter detect, column map |
| `src-tauri/src/parser/positions.rs` | 174 | Position string parser |
| `src-tauri/src/parser/metrics.rs` | 215 | Per-90 and ratio computation |
| `src-tauri/src/parser/countries.rs` | 261 | Static country code lookup |
| `src-tauri/src/commands/csv_parser.rs` | 17 | Tauri command handlers |
| `src-tauri/src/storage/mod.rs` | 34 | Persistence stub |
| `src-tauri/src/lib.rs` | 26 | App entry, command registration |

**Total:** 2,572 lines of Rust. 113 `#[test]` functions.

## Executive Summary

The CSV parser is a solid first feature with correct architecture, clean module boundaries, and good test coverage. The main problems are two oversized files (`mod.rs` at 574 lines, `fields.rs` at 607 lines) driven by mechanical repetition rather than genuine complexity, a struct-heavy `assign_stat` match that will be a maintenance drag every time FM changes a column name, and a task-tracking system that was abandoned mid-development. The `save_import` function is a stub that silently discards data, which is the highest-risk gap.

## What Went Well

- **Module structure matches the design spec exactly.** The planned architecture (`types.rs`, `headers.rs`, `fields.rs`, `positions.rs`, `metrics.rs`, `countries.rs`) was implemented without deviation. This is rare and valuable — the spec was actionable and the implementation followed it.
- **Thin command layer.** `commands/csv_parser.rs` at 17 lines is exactly right. No business logic leaked across the Tauri boundary.
- **Skip-and-collect error handling.** The hard-reject / soft-degrade pattern in `parse_csv` (lines 189–362 of `mod.rs`) correctly distinguishes between row-killing failures and field-level degradation. This was a good design decision and it's implemented correctly.
- **Good test coverage for field parsers.** `fields.rs` has 26 tests covering identity, physical, financial, date/time, and stat parsing with edge cases. `headers.rs` has 11 tests. `positions.rs` has 8 tests. The integration tests (edge cases + sample CSV) provide end-to-end confidence.
- **`parse_csv` is genuinely pure.** No side effects, no global state, no database writes. The design spec's hard constraint is honored.
- **Country code lookup uses `phf` for compile-time lookup.** Correct choice for a static table.

## Critical Findings

### 1. Repetitive Soft-Field Extraction Pattern in `parse_csv`

**Current State:**

`src-tauri/src/parser/mod.rs:248-362` — approximately 115 lines of nearly identical code following this pattern:

```rust
if let Some(idx) = col_nation {
    if let Some(raw) = record.get(idx) {
        player.nationality = parse_nationality(raw);
    }
}
```

This pattern repeats for: nationality, second nationality, club, age, height, left foot, right foot, CA, PA, transfer value, wage, contract expires, appearances, minutes — 15 fields, each with the same `if let Some(idx)` + `if let Some(raw)` wrapper. Fields that produce warnings (footedness, transfer value, wage) add another 8–12 lines each for warning construction.

**Impact:**
- Adding a new field requires copy-pasting ~8 lines and changing 3 identifiers. Easy to get wrong.
- The repetition hides the interesting logic (which field, which parser, which warning) inside boilerplate.
- Makes the 287-line `parse_csv` function appear more complex than it is — the actual control flow is simple, it's just verbose.

**Recommended Refactor:**

Extract a helper that encapsulates the `Option<index> → record.get → parser call` pattern:

```rust
fn extract_field<T>(
    record: &csv::StringRecord,
    col_index: Option<usize>,
    parse: impl Fn(&str) -> T,
) -> Option<T> {
    col_index.and_then(|i| record.get(i).map(parse))
}
```

For fields that produce warnings, a second helper:

```rust
fn extract_field_with_warning<T>(
    record: &csv::StringRecord,
    col_index: Option<usize>,
    row_number: usize,
    field_name: &str,
    parse: impl Fn(&str) -> (T, Option<String>),
    warnings: &mut Vec<ParseWarning>,
) -> Option<T>
```

This would collapse the 115-line block to ~30 lines of meaningful assignments.

**Effort:** Low
**Priority:** P1 (important)

### 2. `assign_stat` Match Statement — Fragile, Verbose, and Error-Prone

**Current State:**

`src-tauri/src/parser/mod.rs:418-487` — a 70-line `match field_name` that maps string literals to struct field assignments:

```rust
fn assign_stat(player: &mut ParsedPlayer, field_name: &str, value: Option<f64>) {
    match field_name {
        "goals" => player.attacking.goals = value,
        "goals_from_outside_box" => player.attacking.goals_from_outside_box = value,
        // ... 65 more arms ...
        _ => {} // Unknown stat name, skip
    }
}
```

**Impact:**
- **Every new FM column requires two coordinated changes:** add an entry to `STAT_COLUMNS` (line 37) AND add a match arm in `assign_stat` (line 418). Miss either one and the stat is silently ignored.
- The string literal in the match arm must exactly match the string literal in `STAT_COLUMNS`. No compile-time enforcement.
- 70 match arms is a code review nightmare — the diff for adding one stat is 2 lines in 2 different locations.
- The `_ => {}` catch-all silently swallows typos. A misspelled field name in `STAT_COLUMNS` produces no error and no warning.

**Recommended Refactor:**

Option A (minimal): Replace the string match with a macro that generates both the `STAT_COLUMNS` entry and the assignment in one declaration. This eliminates the two-location coordination problem.

Option B (structural): Replace the 8 stat structs with a single `HashMap<String, Option<f64>>` for raw stats, and only promote to typed accessors when the data is consumed downstream. This eliminates `assign_stat` entirely — stat assignment becomes a single `HashMap::insert`. The typed accessors can be generated with a macro.

Option B is the better long-term choice because FM column sets change between versions. A HashMap absorbs new columns without code changes; the current struct approach requires code changes for every new column.

**Effort:** Medium (Option A), Medium-High (Option B)
**Priority:** P1 (important)

### 3. `types.rs` at 465 Lines — Stat Structs Dominate

**Current State:**

`src-tauri/src/parser/types.rs` — 8 stat structs (AttackingStats, ChanceCreationStats, MovementStats, DefendingStats, AerialStats, GoalkeepingStats, DisciplineStats, MatchOutcomeStats) plus ParsedPlayer, ParseResult, and supporting types. Each stat struct is a list of `pub Option<f64>` fields with serde annotations.

**Impact:**
- The file is 90% field declarations with no logic. It's a data definition file, not a code file. This is acceptable in isolation, but it must be updated in lockstep with `STAT_COLUMNS` and `assign_stat` — three places for every stat change.
- The 8 struct separation is a reasonable organizational choice, but it creates the maintenance problem described in Finding #2.

**Recommended Refactor:**

This finding is a consequence of Finding #2. If the stat structs are replaced with a HashMap (Finding #2, Option B), this file drops to ~100 lines containing only the domain types that actually have behavior (Position, Role, Side, Footedness, Nationality, SkippedRow, ParseWarning, ParseResult). The stat structs disappear entirely.

If keeping the structs, consider deriving them from a single macro invocation to guarantee field names stay synchronized with `STAT_COLUMNS`.

**Effort:** Covered by Finding #2
**Priority:** P1 (important)

### 4. `save_import` Silently Discards All Data

**Current State:**

`src-tauri/src/storage/mod.rs:6-14`:

```rust
pub fn save_import(players: Vec<ParsedPlayer>, _in_game_date: &str) -> Result<(), String> {
    // TODO: Implement actual persistence when storage layer is built.
    if players.is_empty() {
        return Ok(());
    }
    // Stub: accept all players, no actual storage
    Ok(())
}
```

The Tauri command in `lib.rs:22` calls this stub and returns success to the frontend. The user clicks "Save", sees a success response, and all their parsed data is silently discarded.

**Impact:**
- **This is the highest-risk gap in the codebase.** The `save_import` command promises persistence and delivers nothing. There is no indication to the user that data was not saved.
- The design spec (invariant #8) requires idempotent persistence. The stub violates this contract.
- Any frontend code built against this API will appear to work correctly until the user closes the app and finds their data gone.

**Recommended Refactor:**

Either implement persistence (out of scope for this refactor — it's a separate feature) or make the stub honest:

```rust
pub fn save_import(_players: Vec<ParsedPlayer>, _in_game_date: &str) -> Result<(), String> {
    Err("Storage is not yet implemented. Your data has not been saved.".to_string())
}
```

This way the frontend receives an error, the user sees a clear message, and no one is misled.

**Effort:** Low
**Priority:** P0 (critical)

### 5. `greet` Function Left from Scaffold

**Current State:**

`src-tauri/src/lib.rs:11-13`:

```rust
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
```

Registered on line 21: `greet,`. This is the Tauri scaffold default, not part of the CSV parser feature.

**Impact:**
- Minor code hygiene issue. Dead code that ships to production.
- Signals incomplete cleanup after scaffold.

**Recommended Refactor:**

Remove the `greet` function and its registration in `invoke_handler`.

**Effort:** Low
**Priority:** P2 (nice to have)

### 6. Task Tracking Abandoned After Task 01

**Current State:**

`docs/specs/implementation/features/csv-parser/first-pass/INDEX.md` shows:
- Task 01: `[x]` (completed)
- Tasks 02–10: `[ ]` (not completed)

But `git log --oneline` shows commits for all 10 tasks:
```
5b4f3f8 test(parser): add edge case tests for CSV parser          (Task 10)
13b8a47 test(parser): add integration tests with sample CSV        (Task 09)
917c6b5 feat(tauri): add parse_csv and save_import commands        (Task 08)
dd8bdbe feat(parser): implement CSV parser orchestration pipeline   (Task 07)
f8fce4a feat(parser): add computed metrics (per-90 and ratios)     (Task 06)
7f1467e feat(parser): add field parsers for identity...            (Task 05)
90420fc feat(parser): add header parser with BOM stripping...      (Task 04)
db5d637 feat(parser): add country code lookup table with phf       (Task 02)
03cdf75 feat(parser): add core types and stat structs              (Task 01)
```

None of the task files contain development logs or post-implementation notes.

**Impact:**
- The task tracking system is unreliable. A new contributor reading INDEX.md would believe Tasks 02–10 are not implemented.
- No development logs means no record of decisions, surprises, or issues encountered during implementation. The next feature loses this institutional knowledge.
- Makes it impossible to audit whether the implementation matches the task specifications.

**Recommended Refactor:**

Update INDEX.md to reflect actual completion state. Add brief development logs to each task file noting what was done, any deviations from spec, and any issues. This is a documentation task, not a code task.

**Effort:** Low
**Priority:** P2 (nice to have)

### 7. Duplicate Comment Block in `mod.rs`

**Current State:**

`src-tauri/src/parser/mod.rs:16-36`:

```rust
/// Column name constants for all stat fields.

/// Maps internal field name → CSV header name (for lookup).

struct ColumnDef {
    csv_name: &'static str,
    allow_negative: bool,
}

// ── Stat column definitions ────────────────────────────────────────────
// Each stat has a CSV header name and whether negative values are allowed.

// ── Stat column definitions ────────────────────────────────────────────
// Each stat has a CSV header name and whether negative values are allowed.
```

Lines 30-31 and 34-35 are identical comment blocks. Lines 16-17 are dangling doc comments that don't attach to anything.

**Impact:**
- Minor. Suggests the code was written in haste without a final review pass.

**Recommended Refactor:**

Remove the duplicate comment and clean up the dangling doc comments. Keep one comment block before `STAT_COLUMNS`.

**Effort:** Low
**Priority:** P2 (nice to have)

## What Should Have Been Done Before

1. **Stat schema as data, not code.** The three-point synchronization problem (STAT_COLUMNS → types.rs structs → assign_stat match) existed from the first design. Defining stats in a single declarative source (macro, config file, or build script) from the start would have eliminated the entire class of bugs where a stat is added in one place but not another.

2. **Persistence layer before or alongside the parser.** The parser produces data that goes nowhere. The `save_import` stub means the feature cannot be used end-to-end. Building persistence first (even a trivial file-based store) would have validated the `ParsedPlayer` shape against actual storage constraints.

3. **Task file discipline during implementation.** The task files were created as planning artifacts and then ignored during execution. If the task files are the contract, they should be updated as the work proceeds. If they're not going to be maintained, they shouldn't exist — an out-of-date tracking document is worse than no tracking document.

## Refactor Priorities

### Phase 1 (Must Fix)

1. **Finding #4** — Make `save_import` stub honest (return error instead of silent success). One-line change.
2. **Finding #5** — Remove `greet` scaffold code from `lib.rs`. Two-line change.

### Phase 2 (Should Fix)

1. **Finding #1** — Extract `extract_field` / `extract_field_with_warning` helpers to collapse the repetitive soft-field block in `parse_csv`.
2. **Finding #2** — Replace `assign_stat` match with a data-driven approach. Decide between macro-generated match (Option A) or HashMap migration (Option B). This is the most impactful refactor for long-term maintainability.
3. **Finding #3** — Consequence of #2. Resolve together.

### Phase 3 (Nice to Have)

1. **Finding #6** — Update INDEX.md and add development logs to task files.
2. **Finding #7** — Clean up duplicate comments and dangling doc comments.

## Implementation Notes

- Finding #2 (assign_stat refactor) is the linchpin. It should be designed first, because it determines whether Finding #3 is resolved automatically (Option B) or requires a separate macro (Option A).
- Option B (HashMap for stats) is a bigger change but pays off when FM changes column sets between versions. The stat structs provide type safety that isn't currently being leveraged — nothing in the codebase iterates over stat fields by name except `assign_stat` itself, which is just doing what a HashMap does natively.
- The `extract_field` helper (Finding #1) is independent of Finding #2 and can be done in any phase.
- Phase 1 changes are safe to ship immediately with zero regression risk.

## Files Affected

- `src-tauri/src/storage/mod.rs` — make stub honest (Phase 1)
- `src-tauri/src/lib.rs` — remove greet (Phase 1)
- `src-tauri/src/parser/mod.rs` — extract helpers, replace assign_stat, clean comments (Phase 2 & 3)
- `src-tauri/src/parser/types.rs` — stat struct changes or removal (Phase 2)
- `docs/specs/implementation/features/csv-parser/first-pass/INDEX.md` — update completion status (Phase 3)
- `docs/specs/implementation/features/csv-parser/first-pass/task-*.md` — add development logs (Phase 3)

## Success Criteria

- [ ] `save_import` returns an error when called, not silent success
- [ ] `greet` function and its registration removed
- [ ] `parse_csv` function body reduced by at least 80 lines through helper extraction
- [ ] `assign_stat` eliminated or reduced to generated code — no hand-maintained 70-arm match
- [ ] Adding a new stat column requires a change in exactly one location
- [ ] All 113 existing tests continue to pass after refactor
- [ ] INDEX.md reflects actual completion state of all 10 tasks
