# Squad Optimizer

## Status

Active

## Intent

Fill every configured Planner team and ordered string with the best available players for the shared tactic. The optimizer must preserve the user's manual choices and make its own choices replaceable on a later run.

## User-visible behavior

- The Planner can optimize Senior, Reserves, and Youth in that order, then each team's strings from first to last.
- A string receives the assignment set with the highest total combined role score across its unfilled tactic lanes. It does not greedily fill one lane at a time.
- A player can fill a lane only when the player belongs to that team's configured club-family pool, meets the team's age rule, has suitability of at least 15 for both lane positions, and has a combined score.
- Senior has no age maximum. Reserves permit ages through 23. Youth permit ages through 18. A missing age excludes a player from Reserves and Youth.
- Missing IP or OOP role score makes a player ineligible for optimization. A present combined score of zero remains eligible and wins over a blank when it increases the number of filled lanes without reducing the total score.
- Manual assignments remain fixed. Manual assignments anywhere in the save reserve their player UID before automatic allocation, including a manual assignment in a later team or string.
- Re-running Optimize removes only assignments created by an earlier Optimize run, recomputes from the current snapshot, tactic, and club-family sources, and returns reconciled Planner depth.
- The selected Planner team has a **Clear Squad** action. It removes every assignment in that team only after an explicit destructive confirmation. It does not change other teams.
- Optimize and Clear Squad show pending, success, and error feedback. Their keyboard paths use the existing button and destructive Modal behavior.

## Invariants

- Planner assignments remain save-scoped and a player UID remains unique across all Planner teams and strings in that save.
- Existing assignment rows migrate to manual provenance. Manual assignment and move mutations write manual provenance.
- Optimize runs as one Rust-owned database transaction: retain manual rows, remove prior optimizer rows, allocate in strict priority order, persist the new optimizer rows, then return depth after the transaction commits.
- The optimizer never changes, removes, or repositions a manual assignment, even if that player is over-age, outside the current pool, position-ineligible, unresolved, or lacks a current combined score.
- Later automatic strings cannot exchange an earlier automatic string's result for a better later result. Manual rows reserve UIDs before this automatic priority order begins.
- The primary club remains a source for all three teams. An attached source remains available only to its configured target team.
- When IP and OOP positions differ, both suitability values must be at least 15. When they match, one suitability check is sufficient.
- The optimizer first maximizes a string's total combined score, then its number of filled lanes, then uses a stable deterministic tie-break. Blank lanes are valid output.
- React presents Rust-provided Planner depth and does not recompute eligibility, score, priority, or matching results.

## Non-goals

- Transfer-gap recommendations beyond visibly blank optimized slots.
- Formation comparison, custom optimizer constraints, user-configured priorities, string reordering, or custom string names.
- A manual-assignment locking UI. Provenance is persisted for optimizer behavior, not surfaced as a new editing control.
- Changing tactic ownership, role-score calculation, club-family configuration, or the existing manual picker behavior.
- Any new matching library or other dependency.

## Current-state map

- **Relevant components:** `src/app/routes/planner.tsx` composes the Planner. `src/features/planner/components/planner-depth-matrix.tsx` owns selected team, depth actions, string menus, destructive removal, and depth/candidate cache reconciliation. `src/features/planner/components/planner-slot-fit-picker.tsx` owns manual assign, move, and clear-slot interactions.
- **Data model:** migration v6 in `src-tauri/src/db/migrations.rs` creates save-scoped `planner_strings` and `planner_assignments`. The assignment table has `save_id`, `string_id`, `lane_id`, `player_uid`, and `last_known_name`; uniqueness already covers `(save_id, player_uid)` and `(string_id, lane_id)`.
- **Persistence and migrations:** `migrations::apply` uses ordered `PRAGMA user_version` migrations and one transaction per migration. Existing migration tests inspect fresh-schema columns and monotonic registration in `src-tauri/src/db/migrations.rs`.
- **Existing behavioral assumptions:** `depth::assign_player` and `depth::move_player` validate the target string and lane, enforce save-wide UID uniqueness, and store the current player name. `depth::get_depth` resolves retained rows against the current snapshot as `resolved`, `outside_pool`, or `unresolved` without deleting them.
- **Architectural seams:** Rust `features/planner/{depth.rs,commands.rs}` owns Planner mutations and bounded depth DTOs. `src-tauri/src/lib.rs` registers Planner commands. Frontend `features/planner/api/` reaches commands only through `src/lib/tauri-client.ts`; `plannerKeys` owns Planner cache keys.
- **Eligibility evidence:** `players.age` and `players.positions_json` exist in migration v2. `bridge/DUMP_SCHEMA.md` defines `age` as nullable and `positions` as a position-to-suitability map. `src-tauri/src/features/search/filter.rs` validates the FM position keys and uses `json_extract(positions_json, '$.<position>')` for exact suitability checks.
- **Score evidence:** `features/scoring/combine.rs::combine_role_scores` returns `None` when either phase score is missing and otherwise rounds the tactic-weighted score. `depth::get_slot_candidates` already loads the two role scores from `player_role_scores`, applies that helper, and orders candidates deterministically.
- **Priority evidence:** `depth::get_depth` already returns teams in Senior, Reserves, Youth order and strings by `string_order`. `tactic::TACTIC_LANE_COUNT` is 11 and `PlannerTactic` persists ordered lanes.
- **Test ownership:** Rust Planner service tests live in `src-tauri/src/features/planner/depth.rs`; migration tests live in `src-tauri/src/db/migrations.rs`. Planner route integration tests and the in-memory Planner IPC mock live in `src/app/routes/planner.test.tsx` and `src/testing/planner-ipc-mock.ts`. Browser smoke uses `e2e/smoke.spec.ts` and `e2e/tauri-ipc-stub.ts`.
- **Authoritative validation commands:** `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` is unsupported and is not evidence.
- **Likely reuse points:** `PlannerDepthDto` returns from every current Planner mutation; `Panel.actions`, `Button` loading labels, `Modal` destructive variant, `PlannerDepthMatrix` mutation structure, and Planner route test helpers are established UI seams.
- **Known technical risks:** snapshot replacement removes snapshot-owned rows, assignment rows survive it, and cached slot candidates carry assignment locations. The optimizer must therefore use current snapshot values but preserve manual retained rows and invalidate all candidate queries after a result changes assignments.
- **Applicable repository patterns:** `remove_planner_string` requires a confirmation flag in Rust and returns reconciled depth; the tactic editor shows local pending, success, and error status; the app top bar invalidates `plannerKeys.all` after Load Data and active-save changes.

## Feature architecture (this feature)

```text
React Planner controls
  → Planner feature API through invokeCommand
  → Rust Planner command
  → planner depth service transaction
      → preserve manual assignment provenance
      → replace prior optimizer rows
      → current snapshot + tactic + club-family eligibility
      → exact per-string matching in strict team/string order
  → reconciled PlannerDepthDto
  → depth cache replacement + slot-candidate invalidation
```

Rust remains the authority for provenance, eligibility, matching, priority, SQLite transactions, and the returned depth model. React only requests the two mutations, controls the selected-team confirmation flow, displays state, replaces the returned depth cache, and invalidates stale picker candidates.

The matcher belongs in the existing Planner depth module or a narrowly adjacent private Planner module. It uses a small exact dynamic-programming or equivalent maximum-weight bipartite matching implementation over at most 11 remaining lanes. It must not add a dependency. Its comparison order is total score, then filled-lane count, then the lexicographically lowest player-UID assignment by stable tactic-lane order; a blank sorts after a player UID for that final comparison.

## Uncertainty register

### Known

- `planner_assignments` is save-scoped and already enforces global UID uniqueness.
- `PlannerTeam` exposes the exact Senior, Reserves, Youth order. `PlannerString.string_order` provides each team's priority order.
- Planner sources already model the primary club for every team and attached clubs for only their configured team.
- `positions_json` stores the 0–20 position suitability values required for the 15-point eligibility threshold.
- Planner's existing TypeScript and Playwright fixtures are in-memory protocol stubs. They do not prove Rust persistence; Rust tests own that proof.
- Recallium searches were attempted four times by the main session and failed with `Search orchestrator not available`. Planning proceeds from repository evidence; no Recallium memory is saved because the ledger contains the durable plan.

### Assumptions

- **Clear Squad scope:** Clear Squad deletes all assignments, manual and optimizer-generated, from the currently selected Planner team only. It preserves the other two teams. The developer added this action after the first clarification round but did not specify whether it should affect one team or all three; selected-team scope is the smallest safe interpretation.
- A deterministic tie should favor lower player UID by stable lane order after score and fill-count ties. This does not express product preference; it only makes equal valid outcomes repeatable from the same snapshot.

### Decisions

- Use one PR with four atomic commits. Provenance is the migration boundary; the optimizer is a Rust-only verifiable outcome before UI exposure; Clear Squad is a separately useful destructive control; Optimize then completes the planned UI and smoke path.
- Persist two assignment provenances: manual and optimizer. Existing rows default to manual in the new migration.
- Use a private exact matcher bounded by the existing 11 tactic lanes. Do not add a matching dependency.
- Treat a score of zero as assignable and a missing combined score as non-assignable. This preserves the specified lexicographic score, fill, tie-break objective.
- Plan Clear Squad as a selected-team destructive action. A change to all-team clearing, provenance-only clearing, or a different scope requires replanning commits 3 and 4 before build.

### Unknowns

- Does **Clear Squad** mean the currently selected team, as planned, or all three teams? — blocks: commit 3 only; it does not block provenance or optimizer work.
- Native WebView verification at 1280×800 and 1600×900 remains environment-dependent, as it was at Squad Planner close-out. This blocks validation only if a UI behavior cannot be proven by Vitest and smoke; it does not block the first commit.

## Risks

### A migration misclassifies existing manual work

- **Trigger:** the provenance field lacks a safe default or manual assign/move writes the optimizer value.
- **Consequence:** a later Optimize run can replace user-created assignments.
- **Mitigation:** migration defaults every existing row to manual; Rust tests cover migration shape, legacy-row behavior, manual assign, and move.
- **Proof:** PR 1, commit 1 Rust migration and Planner depth tests.

### Greedy or global matching violates the stated allocation order

- **Trigger:** implementation selects lane-by-lane or trades an earlier string's result for a later string.
- **Consequence:** the displayed lineup is not the required best result for the stated priority.
- **Mitigation:** test exact matching against a greedy counterexample, process only one string at a time, and reserve every manual UID before automatic allocation.
- **Proof:** PR 1, commit 2 pure-matcher and service tests.

### Snapshot changes make retained state unsafe to optimize

- **Trigger:** optimization treats unresolved, outside-pool, over-age, or scoreless manual rows as invalid and deletes or moves them.
- **Consequence:** Load Data followed by Optimize destroys Planner intent.
- **Mitigation:** preserve manual rows without revalidating them; read only current eligible candidates for empty cells; test resolved, outside-pool, unresolved, and missing-age cases.
- **Proof:** PR 1, commits 1 and 2 Rust tests.

### Clear Squad clears too broadly or appears to succeed with stale picker data

- **Trigger:** the delete predicate ignores the selected team, the backend accepts an unconfirmed request, or candidate caches retain stale assignment locations.
- **Consequence:** manual work in another team disappears or a later picker gives false move guidance.
- **Mitigation:** scope deletion through save and target-team strings, require the confirmation flag, replace depth, invalidate `plannerKeys.slotCandidates()`, and test keyboard focus plus failure retention.
- **Proof:** PR 1, commit 3 route and Rust tests.

## Walking skeleton

Commit 1 makes retained Planner assignments distinguish manual from replaceable optimizer state. Commit 2 then exposes a testable Rust optimizer that recomputes and persists all automatic assignments but has no new control yet. This proves the migration, transaction, priority, eligibility, and matching contracts on trunk before UI work. Commit 3 exposes a safe selected-team reset. Commit 4 connects Optimize beside it and proves the browser path with stubbed IPC.

## Delivery plan

### PR 1 — Add squad optimization

**Status:** Active

**Provisional PR title:** `feat(planner): add squad optimization`

**Purpose:** Deliver optimizer behavior through a safe sequence that preserves existing manual Planner work, proves the Rust allocation algorithm before UI exposure, and ends with clear controls and smoke coverage.

**Depends on:** Completed Squad Planner (`.wiki/features/completed/squad-planner.md`), including migrations v4–v6, current snapshot ingest, role scoring, club-family sources, shared tactic lanes, and depth cache behavior.

**Merge boundary:** One short-lived trunk PR is sufficient. Its commits form independently safe outcomes: migration compatibility, backend optimization, selected-team clearing, and UI exposure. A second PR would not remove a shared review surface or establish a more useful trunk boundary.

#### Commit 1 — Persist assignment provenance

**Status:** Active

**Work:** Add save-scoped persisted provenance to Planner assignments. Existing rows become manual by migration default. New manual assign and move paths write manual provenance while retaining all current uniqueness, snapshot-resolution, and manual-picker behavior.

**Out of scope for this commit:**

- Optimizer matching, optimizer IPC, or optimizer-generated rows.
- Any new Planner control, assignment provenance indicator, or Clear Squad action.
- Changing slot-candidate eligibility, score calculation, club-family pools, tactic lanes, or snapshot ingest.

**Validation:** Start with a migration/legacy-row RED test that proves rows created before the new field remain manual. Add depth service tests for manual assign and move provenance while preserving the current unique UID and unresolved-row behavior. Run targeted Rust tests, `./scripts/dev test` for unaffected frontend contract confirmation, and `./scripts/dev check`. Run `./scripts/dev smoke` only as the existing browser regression gate; no browser behavior changes in this commit.

**Provisional commit:** `feat(planner): persist assignment provenance`

##### Implementation profile

**Assigned implementer:** Terra — `gpt-5.6-terra` at `xhigh`

**Routing summary:** Capability Demand is 5 (ambiguity 0, novelty 1, diagnostic uncertainty 0, semantic risk 2, context synthesis 2), which routes to Terra. The Terra xhigh hard floor applies because the commit migrates existing data and provenance must remain consistent across assign and move mutations. Effort Demand is 6 after adjustments; the hard floor raises execution to Terra xhigh. The Luna punch-up does not apply.

##### Review profile

**Assigned reviewer:** Terra — `gpt-5.6-terra` at `xhigh`

**Context:** Fresh. The reviewer receives this commit contract, the relevant feature context, packet, staged diff, validation evidence, and repository access before implementation notes.

**Mandate:**

- Verify the new migration is monotonic and makes pre-existing assignment rows manual without rebuilding or deleting the table.
- Verify both manual assignment entry points write manual provenance and no existing mutation accidentally writes optimizer provenance.
- Verify save-wide UID and cell uniqueness remain enforced after the schema change.
- Verify `get_depth` still retains and resolves manual rows through snapshot replacement and source changes.
- Verify tests would fail if a legacy row became optimizer-generated or a moved manual row became replaceable.

##### Implementation packet

###### Governing requirements and invariants

- Existing assignment rows migrate/default to manual provenance.
- Manual assignment and move mutations become manual provenance.
- Player UID uniqueness across all Planner teams and strings remains save-scoped.
- This commit preserves all current Planner-visible behavior.

###### Existing patterns to follow

- `src-tauri/src/db/migrations.rs::Migration`, `migrations::all`, `migrations::apply`, and the v6 Planner-depth schema tests define the migration pattern.
- `src-tauri/src/features/planner/depth.rs::{assign_player,move_player,insert_assignment}` define every current row-writing path.
- `depth.rs` tests `enforces_player_uniqueness_and_moves_in_one_save` and `preserves_assignment_as_unresolved_when_snapshot_replaces_player` are the closest behavior tests.
- No existing provenance field or provenance migration exists. Add only the narrow Planner-specific representation required by this contract.

###### Expected change surface

- **Likely modified:** `src-tauri/src/db/migrations.rs`, `src-tauri/src/features/planner/depth.rs`.
- **Likely added:** None.
- **Ownership boundaries:** Rust owns migration and provenance. No React type or UI needs the private field in this commit.
- **Do not change without replanning:** existing assignment DTO shape, snapshot deletion rules, club-family source ownership, tactic persistence, or cross-save UID isolation.

###### State and data design

- The database row is the source of truth for provenance.
- Add a non-null constrained provenance value with manual as the migration default so rows created by v6 are preserved as manual during v7 application.
- `assign_player` and `move_player` persist manual. `clear_assignment` keeps its existing delete semantics.
- `get_depth` continues to resolve row identity, pool state, and combined score from the current snapshot. Provenance remains internal until a later product requirement needs it in the read model.
- No cache, loading, error, or replacement behavior changes in this commit because React does not receive a changed DTO.

###### Expected interfaces

- Extend the Rust Planner assignment persistence model only as required to distinguish manual from optimizer rows.
- Keep current `PlannerDepthDto` and all registered Planner command signatures unchanged.
- Do not invent a public provenance API; commit 2 consumes the persisted state inside Rust.

###### Execution order

1. Write the smallest migration and legacy-row RED test.
2. Add the next monotonic Planner migration and update schema-registration and column tests.
3. Thread manual provenance through the private insert path and both manual mutations.
4. Add targeted behavior tests for assign and move, then rerun existing Planner depth tests.
5. Run the validation ladder and review the diff against unchanged external contracts.

###### Validation ladder

1. Targeted `depth.rs` and `migrations.rs` Rust tests for legacy rows, manual assign, manual move, uniqueness, and retained resolution.
2. Existing Planner depth Rust tests.
3. `./scripts/dev test` to confirm the unchanged frontend Planner contract.
4. `./scripts/dev check`.
5. `./scripts/dev smoke` as unchanged browser regression coverage.
6. No manual proof is required unless migration tests cannot create a v6-shaped database state.

###### Stop conditions

- Stop and replan if `PRAGMA user_version` cannot express an additive default-safe migration, current data must be rewritten or deleted, current mutations do not share a safe insertion seam, or existing assignments require a new public DTO contract.

###### Allowed discretion

- Private enum or value naming, query organization, test fixture structure, and local helper extraction that preserve the stated database and command contracts.

###### Prohibited discretion

- Defaulting legacy rows to optimizer provenance, exposing a new UI control, changing UID uniqueness, changing manual assignment availability, or altering snapshot/pool resolution.

##### Escalation conditions

- **Increase effort when:** The migration design is correct but existing fixture construction, query details, or regression coverage remains incomplete.
- **Increase model capability when:** The implementer cannot preserve legacy manual meaning or misunderstands which mutation owns provenance.
- **Replan when:** The persisted assignment contract needs a destructive rewrite, a current manual path cannot express manual provenance, or a DTO/API boundary must change.

##### Execution metadata

```yaml
execution_profile:
  planner:
    model: gpt-5.6-terra
    effort: xhigh
  implementer:
    model: gpt-5.6-terra
    effort: xhigh
    confidence: null
  capability_demand:
    residual_ambiguity: 0
    architectural_novelty: 1
    diagnostic_uncertainty: 0
    semantic_risk: 2
    context_synthesis: 2
    total: 5
    luna_punch_up_applied: false
    hard_floor: Terra xhigh — existing assignment data migrates and provenance spans assign and move
  effort_demand:
    implementation_breadth: 2
    branch_density: 2
    repository_discovery: 1
    validation_weakness: 1
    tool_coordination: 1
    adjustments: -1
    total: 6
    hard_floor: Terra xhigh — migration compatibility must be preserved
  reviewer:
    model: gpt-5.6-terra
    effort: xhigh
    context_mode: fresh
  review_demand:
    missed_defect_consequence: 2
    hidden_interaction_complexity: 2
    validation_weakness: 1
    architectural_discretion: 1
    blast_radius: 2
    total: 8
    hard_floor: Terra xhigh — existing-data migration
  review_mandate:
    - Verify legacy rows migrate to manual provenance.
    - Verify assign and move persist manual provenance without weakening uniqueness.
    - Verify retained rows still resolve across snapshot replacement.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - The migration is correct but test fixture or query coverage misses a bounded path.
  escalate_model_when:
    - The implementer misunderstands legacy-manual ownership or mutation provenance.
  replan_when:
    - The migration or public Planner contract must change materially.
  adjudicator:
    model: gpt-5.6-sol
    effort: high
    invoke_when:
      - Reviewer and implementer disagree about legacy assignment semantics.
      - A high-severity migration finding remains disputed.
      - A correction changes the feature plan.
```

#### Commit 2 — Add the Rust squad optimizer

**Status:** Pending

**Work:** Add a Rust-owned Planner optimizer mutation and IPC command. In one transaction, preserve manual rows, delete prior optimizer rows, read the current snapshot/tactic/club-family sources, allocate exact per-string maximum-weight matches in strict priority order, persist optimizer rows, and return reconciled depth.

**Out of scope for this commit:**

- An Optimize button, frontend API fetcher, cache update, or new browser interaction.
- Clear Squad behavior or Button styling.
- Changes to manual picker eligibility or an assignment-provenance UI.
- Transfer recommendations, custom constraints, string reordering, or any matching dependency.

**Validation:** Start with a pure matching RED test where greedy lane selection loses to the global string optimum. Add service tests for strict team/string priority, all manual UIDs reserved before automatic allocation, rerun replacement, team age limits including missing age, both-position suitability, missing-score exclusion, zero-score fill preference, deterministic ties, source scope, and transaction failure preservation. Run targeted Rust tests, `./scripts/dev check`, and `./scripts/dev test`; run `./scripts/dev smoke` only as unchanged browser regression coverage.

**Provisional commit:** `feat(planner): optimize depth assignments`

##### Implementation profile

**Assigned implementer:** Terra — `gpt-5.6-terra` at `xhigh`

**Routing summary:** Capability Demand is 7 (ambiguity 1, novelty 2, diagnostic uncertainty 0, semantic risk 2, context synthesis 2), which routes to Terra. Terra xhigh is mandatory because uniqueness and ordering span existing manual and new optimizer mutations, persisted state is replaced atomically, and this command crosses Planner persistence and IPC. Effort Demand is 10 after adjustments, which independently routes to xhigh. The Luna punch-up does not apply.

##### Review profile

**Assigned reviewer:** Terra — `gpt-5.6-terra` at `high`

**Context:** Fresh. The reviewer receives this commit contract, the relevant feature context, packet, staged diff, validation evidence, and repository access before implementation notes.

**Mandate:**

- Challenge the transaction boundary: manual retention, optimizer-row replacement, rollback on failure, and depth read after commit.
- Reconstruct strict Senior → Reserves → Youth then string-order allocation and verify manual UIDs are reserved before it.
- Verify eligibility includes source scope, age, both required position suitability values, and complete combined scores.
- Verify the matcher is exact per string, honors score then fill count then stable tie-break, and does not greedily consume a candidate.
- Verify reruns replace only optimizer rows, never manual rows, and blank lanes remain blank when no eligible player exists.
- Verify command registration and returned DTO reuse the existing Planner command boundary without moving domain computation to React.

##### Implementation packet

###### Governing requirements and invariants

- Optimize every existing configured team and every string in strict Senior, Reserves, Youth, then ascending string order.
- Preserve manual rows regardless of current eligibility or resolvability, and reserve all their UIDs before automatic allocation.
- Replace only prior optimizer rows atomically.
- Maximize combined role score over all unfilled lanes within each string. Then maximize number of assigned lanes. Then use the recorded stable tie-break.
- Require current club-source membership, team age eligibility, suitability of at least 15 for each distinct lane position, and `Some(combine_role_scores(...))`.

###### Existing patterns to follow

- `depth.rs::{get_depth,get_slot_candidates,ensure_depth,current_snapshot_id,find_lane,insert_assignment}` supplies the Planner read, source, score, lane, and persistence seams.
- `depth.rs::PlannerTeam` and `get_depth` provide the required team and string ordering.
- `tactic.rs::TACTIC_LANE_COUNT` bounds matching to 11; `PlannerTactic` supplies stable lane order and the current positions/role IDs.
- `scoring/combine.rs::combine_role_scores` is the only combined-score authority.
- `search/filter.rs::json_extract_expr` and position-suitability tests provide the repository's exact JSON-position query precedent.
- `commands.rs` Planner mutations all obtain the active save through `snapshot::service::active_save_id`, call the Rust service, and return `PlannerDepthDto`. `src-tauri/src/lib.rs` registers each command.
- No useful in-repository exact matching implementation exists. Add only a private bounded Planner matcher; do not add a dependency.

###### Expected change surface

- **Likely modified:** `src-tauri/src/features/planner/depth.rs`, `src-tauri/src/features/planner/commands.rs`, `src-tauri/src/lib.rs`.
- **Likely added:** None, unless a focused private Rust matcher module improves cohesion without creating a cross-feature abstraction.
- **Ownership boundaries:** Rust owns all eligibility, matching, transaction, provenance selection, and DTO production. The command returns existing Planner depth shape.
- **Do not change without replanning:** tactic schema, scoring algorithm, club-family source semantics, current snapshot replacement rules, Planner depth DTO compatibility, or frontend ownership.

###### State and data design

- Source of truth is the current save's SQLite Planner rows plus the current snapshot, shared tactic, and configured club sources.
- First load the required tactic and depth topology. Inside the optimizer transaction, collect manual rows and reserve their UIDs; delete only optimizer rows; then evaluate current candidates while allocating each string in priority order.
- Candidate eligibility reads `players.age`, `players.positions_json`, and both persisted role scores from the current snapshot. Primary and attached source scope follows the existing `planner_club_sources` query pattern.
- The matcher sees only empty lanes and unreserved candidates for one string. It stores no future state outside the transaction. After it selects a string, its selected UIDs become unavailable to later automatic strings.
- Persisted optimizer rows retain the player name as current assignment rows do. `get_depth` remains the reconciliation read model after commit.
- Failure rolls back the delete and all optimizer inserts. Load Data remains responsible for invalidating all Planner queries; this backend-only commit introduces no frontend cache mutation.

###### Expected interfaces

- Add one Rust Planner optimization service operation and one registered no-free-form-input Planner command that returns `PlannerDepthDto`, matching current mutation responses.
- Reuse current Planner depth DTO types. Do not expose the private matching candidates or provenance solely for frontend reconstruction.
- Add a private assignment-provenance selector and bounded exact-matching result representation only if the existing depth module cannot express them clearly.

###### Execution order

1. Add pure matcher RED tests, including a case where greedy selection loses the best total and a zero-score candidate beats a blank only on the fill-count objective.
2. Build current-snapshot candidate loading and eligibility tests from existing Planner fixtures and JSON position evidence.
3. Add the atomic optimizer service using commit 1 provenance, priority iteration, and the matcher.
4. Add command DTO wiring and `lib.rs` registration after service behavior is covered.
5. Add rerun, rollback, and retained-manual regression tests, then run the validation ladder.

###### Validation ladder

1. Targeted Rust matcher and `depth.rs` tests for matching, priority, manual reservation, eligibility, rerun, and rollback.
2. Existing Planner depth, tactic, club-family, migration, and scoring-combine Rust tests.
3. `./scripts/dev test` for unchanged frontend contracts.
4. `./scripts/dev check`.
5. `./scripts/dev smoke` as unchanged browser regression coverage.
6. No manual native run is required before a UI control exists.

###### Stop conditions

- Stop and replan if a string can exceed the known 11-lane bound, the existing snapshot cannot supply age or position suitability reliably, the score helper cannot distinguish missing from zero, a single transaction cannot encompass replacement and allocation, or a new public read model becomes necessary.

###### Allowed discretion

- Private query and matcher decomposition, storage layout for private candidate records, exact tie-comparison implementation, and test fixture arrangement that keep the recorded lexicographic objective.

###### Prohibited discretion

- Greedy matching, a new dependency, weakening strict priority, treating missing score as zero, assigning a missing-age Reserves/Youth player, changing manual rows, or moving matching logic to React.

##### Escalation conditions

- **Increase effort when:** The service design is sound but transaction mechanics, matching comparisons, or adversarial test cases are incomplete.
- **Increase model capability when:** The implementer weakens an invariant, chooses a non-exact matcher, misreads source scope, or treats automatic and manual rows alike.
- **Replan when:** the 11-lane bound is false, a required eligibility input is unavailable, atomic replacement is impossible, or the existing Planner mutation/read boundary cannot carry the result.

##### Execution metadata

```yaml
execution_profile:
  planner:
    model: gpt-5.6-terra
    effort: xhigh
  implementer:
    model: gpt-5.6-terra
    effort: xhigh
    confidence: null
  capability_demand:
    residual_ambiguity: 1
    architectural_novelty: 2
    diagnostic_uncertainty: 0
    semantic_risk: 2
    context_synthesis: 2
    total: 7
    luna_punch_up_applied: false
    hard_floor: Terra xhigh — persisted replacement, global uniqueness, and strict ordering cross mutations
  effort_demand:
    implementation_breadth: 3
    branch_density: 3
    repository_discovery: 2
    validation_weakness: 1
    tool_coordination: 1
    adjustments: 0
    total: 10
    hard_floor: Terra xhigh — exact transaction and matching behavior
  reviewer:
    model: gpt-5.6-terra
    effort: high
    context_mode: fresh
  review_demand:
    missed_defect_consequence: 2
    hidden_interaction_complexity: 2
    validation_weakness: 1
    architectural_discretion: 2
    blast_radius: 2
    total: 9
    hard_floor: Terra High — persisted matching and ordering require an engineering review; atomic Rust tests keep this below xhigh
  review_mandate:
    - Verify atomic preservation of manual rows and replacement of optimizer rows.
    - Verify exact per-string matching and strict cross-string priority.
    - Verify every eligibility and score edge case has a precise test.
    - Verify the command returns reconciled Rust-owned depth.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - Matcher or transaction coverage misses bounded combinations despite correct ownership.
  escalate_model_when:
    - The implementer misunderstands exactness, priority, provenance, or source authority.
  replan_when:
    - The 11-lane bound, required data, transaction boundary, or Planner API contract changes.
  adjudicator:
    model: gpt-5.6-sol
    effort: high
    invoke_when:
      - Reviewer and implementer disagree about matching or transaction semantics.
      - A high-severity invariant finding remains disputed.
      - A correction changes the feature plan.
```

#### Commit 3 — Add selected-team Clear Squad

**Status:** Pending

**Work:** Add the confirmed selected-team Clear Squad mutation through Rust, Planner API, and the depth matrix. The action clears all assignments for the selected team after a destructive confirmation, preserves other teams, reconciles depth and candidates, and exposes the design-system destructive Button variant with pending, success, and error feedback.

**Out of scope for this commit:**

- Optimize UI control or browser smoke interaction for Optimize.
- Any optimizer algorithm or change to its provenance behavior.
- Changing string removal, single-slot clearing, club-family scope, tactic controls, or assignment provenance UI.
- A global all-team clear, provenance-only clear, undo history, or a new toast framework.

**Validation:** Start with a Rust RED test that an unconfirmed request and an incorrect team scope cannot delete assignments. Add route RED coverage for confirmation, keyboard operation, pending/error/success feedback, preserved other-team cells, query reconciliation, candidate invalidation, and focus return. Extend `planner-ipc-mock.ts` for the new command. Run targeted frontend and Rust tests, `./scripts/dev test`, `./scripts/dev check`, and the existing `./scripts/dev smoke` regression suite.

**Provisional commit:** `feat(planner): clear selected team`

##### Implementation profile

**Assigned implementer:** Terra — `gpt-5.6-terra` at `high`

**Routing summary:** Capability Demand is 5 (ambiguity 0, novelty 1, diagnostic uncertainty 0, semantic risk 2, context synthesis 2), which would route to Luna. Terra High is applied because this destructive mutation crosses Rust, IPC, React state, and cache reconciliation. The xhigh lifecycle floor is lowered with repository-specific evidence: `remove_string`, the existing destructive Modal, Planner cache reconciliation, and fast Rust/route tests are exact analogues. Effort Demand is 8 after adjustments and independently routes to high. The Luna punch-up does not apply.

##### Review profile

**Assigned reviewer:** Terra — `gpt-5.6-terra` at `high`

**Context:** Fresh. The reviewer receives this commit contract, the relevant feature context, packet, staged diff, validation evidence, and repository access before implementation notes.

**Mandate:**

- Verify one confirmed Clear Squad request can delete only the selected save's selected-team strings and does not touch either other Planner team.
- Verify the Rust boundary rejects an unconfirmed destructive request and validates the team value.
- Verify the UI uses `Panel.actions`, a destructive Button, and the existing destructive Modal with the affected team named.
- Verify pending, success, and error states are truthful, keyboard reachable, and preserve assignments after failure or cancellation.
- Verify success replaces or refetches depth and invalidates every cached slot-candidate location.
- Verify the action includes manual and optimizer rows, as the confirmed selected-team scope requires.

##### Implementation packet

###### Governing requirements and invariants

- Clear Squad removes all assignments in the currently selected team only after explicit confirmation.
- It preserves all assignments in the other two teams and preserves tactic, strings, sources, and snapshot data.
- The backend must require confirmation rather than rely only on UI state.
- A successful mutation returns reconciled depth and invalidates candidate locations; a failed or canceled mutation leaves visible assignments unchanged.

###### Existing patterns to follow

- `depth.rs::remove_string` already validates a destructive confirmation in Rust, executes a scoped transaction, and returns refreshed Planner depth through `commands.rs`.
- `PlannerDepthMatrix` owns `selectedTeam`, current destructive string-removal Modal state, errors, focus restoration, `Panel`, and candidate invalidation.
- `src/components/ui/panel/panel.tsx` supports right-aligned `actions`; `Modal` implements destructive confirmation and keyboard focus behavior.
- `.wiki/DESIGN.md` specifies a destructive Button variant that is intentionally not implemented until the first destructive action. `src/components/ui/button/button.tsx` and its tests are its exact implementation seam.
- `planner-slot-fit-picker.tsx` and Planner route tests demonstrate mutation error retention, candidate invalidation, and origin focus restoration.

###### Expected change surface

- **Likely modified:** `src-tauri/src/features/planner/depth.rs`, `src-tauri/src/features/planner/commands.rs`, `src-tauri/src/lib.rs`, `src/features/planner/components/planner-depth-matrix.tsx`, `src/components/ui/button/button.tsx`, `src/components/ui/button/button.test.tsx`, `src/app/routes/planner.test.tsx`, `src/testing/planner-ipc-mock.ts`.
- **Likely added:** one Planner API mutation wrapper under `src/features/planner/api/`.
- **Ownership boundaries:** Rust owns confirmation validation and deletion scope. React owns the selected-team control, modal, status text, focus, and Query updates. The Button stays a shared visual primitive only.
- **Do not change without replanning:** Clear Squad's selected-team scope, provenance inclusion, string/tactic/source persistence, Modal semantics, or Planner cache ownership.

###### State and data design

- The selected team is existing local state in `PlannerDepthMatrix`; it is not new persisted state.
- The destructive Modal names the selected team and asks for confirmation. The mutation passes the selected team and a confirmation signal to Rust, mirroring the current populated-string safety pattern.
- Rust deletes assignment rows belonging to target-team strings for the active save in one transaction, then returns `PlannerDepthDto` after commit.
- On success React updates the depth query from the response, invalidates `plannerKeys.slotCandidates()`, shows a concise success status, closes the confirmation, and returns focus to the action trigger. On error it retains the modal or controlled error state without claiming success. Pending disables duplicate confirmation.
- Add the `destructive` Button visual variant using only existing error tokens. Do not add a notification dependency; use the tactic editor's local status pattern.

###### Expected interfaces

- Add one Rust Planner team-clear service operation and one validated registered mutation command returning `PlannerDepthDto`, following current Planner mutation DTOs.
- Add one feature API wrapper through `invokeCommand` and the existing Planner key family.
- Extend the in-memory Planner IPC mock with the same request validation and depth result semantics required by route tests.
- Extend the shared Button's existing union with its documented destructive variant; do not create a Planner-specific button.

###### Execution order

1. Write the backend scope/confirmation RED test and implement the transaction.
2. Add command registration and a focused Planner API wrapper.
3. Add the documented shared destructive Button variant with a small component test.
4. Add the selected-team Panel action, destructive Modal, statuses, cache reconciliation, and focus handling.
5. Extend Planner mock and route tests for cancellation, failure, selected-team success, cross-team preservation, and stale candidate invalidation.
6. Run the validation ladder and inspect the action's keyboard path.

###### Validation ladder

1. Targeted Rust Planner clear-team tests and Button component tests.
2. Planner route integration tests through `mockIPC`.
3. Full `./scripts/dev test`.
4. `./scripts/dev check`.
5. `./scripts/dev smoke` as browser regression coverage; commit 4 adds the new dedicated smoke interaction.
6. Manual keyboard confirmation only if automated modal focus coverage cannot prove the intended path.

###### Stop conditions

- Stop and replan if team scope cannot be derived safely from the existing selected team and string records, confirmation cannot be validated server-side, candidate locations cannot be invalidated, a global destructive action is required, or Button/Modal contracts need a broader design-system change.

###### Allowed discretion

- Exact status copy, private query helper organization, trigger placement within `Panel.actions`, and test fixture names that preserve the selected-team contract.

###### Prohibited discretion

- Clearing all teams, clearing only optimizer rows, deleting strings/tactic/sources, bypassing server-side confirmation, adding an undo/history system, or introducing a new notification dependency.

##### Escalation conditions

- **Increase effort when:** The confirmed scope is correct but UI state, focus restoration, or candidate invalidation misses a bounded path.
- **Increase model capability when:** The implementer broadens deletion scope, relies only on frontend confirmation, or misunderstands persisted/cache ownership.
- **Replan when:** Clear Squad scope changes, deletion cannot remain transactional and server-confirmed, or the shared control contract must materially change.

##### Execution metadata

```yaml
execution_profile:
  planner:
    model: gpt-5.6-terra
    effort: xhigh
  implementer:
    model: gpt-5.6-terra
    effort: high
    confidence: null
  capability_demand:
    residual_ambiguity: 0
    architectural_novelty: 1
    diagnostic_uncertainty: 0
    semantic_risk: 2
    context_synthesis: 2
    total: 5
    luna_punch_up_applied: false
    hard_floor: Terra High — destructive cross-layer mutation; exact repository analogues and deterministic tests justify lowering the xhigh lifecycle floor
  effort_demand:
    implementation_breadth: 2
    branch_density: 2
    repository_discovery: 1
    validation_weakness: 1
    tool_coordination: 2
    adjustments: 0
    total: 8
    hard_floor: Terra High — exact remove-string, Modal, and cache-reconciliation patterns bound the work
  reviewer:
    model: gpt-5.6-terra
    effort: high
    context_mode: fresh
  review_demand:
    missed_defect_consequence: 2
    hidden_interaction_complexity: 2
    validation_weakness: 1
    architectural_discretion: 1
    blast_radius: 2
    total: 8
    hard_floor: Terra High — persisted deletion and cross-layer cache lifecycle
  review_mandate:
    - Verify confirmation and selected-team deletion boundaries.
    - Verify other teams and Planner configuration remain intact.
    - Verify keyboard, pending, failure, success, and cache reconciliation states.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - Known scope is correct but UI lifecycle or invalidation coverage is incomplete.
  escalate_model_when:
    - The implementer misunderstands destructive scope or confirmation authority.
  replan_when:
    - Clear Squad scope, transaction semantics, or shared UI contracts change.
  adjudicator:
    model: gpt-5.6-sol
    effort: high
    invoke_when:
      - Reviewer and implementer disagree about destructive scope.
      - A high-severity deletion finding remains disputed.
      - A correction changes the feature plan.
```

#### Commit 4 — Add Optimize controls and smoke coverage

**Status:** Pending

**Work:** Expose the Rust optimizer through the Planner feature API and a primary **Optimize** action beside the existing selected-team **Clear Squad** action. Add status and cache reconciliation, update stubs, and add browser smoke coverage for the final Planner control path.

**Out of scope for this commit:**

- Modifying the optimizer algorithm, migration, matching rules, age/suitability policy, or Clear Squad scope.
- Transfer-gap explanations beyond the blanks returned in Planner depth.
- New visual systems, new dependencies, new routes, manual-lock UI, or real Tauri WebView automation.

**Validation:** Start with a route RED test proving Optimize calls the Planner command, prevents duplicate runs while pending, displays success/error, replaces depth, and invalidates cached slot candidates. Add a smoke RED path using the existing browser IPC stub that reaches Planner, invokes Optimize, and exposes its reconciled matrix state. Update the stub protocol for Optimize and Clear Squad only as the smoke interaction needs. Run targeted Vitest, `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`.

**Provisional commit:** `feat(planner): add optimizer controls`

##### Implementation profile

**Assigned implementer:** Luna — `gpt-5.6-luna` at `high`

**Routing summary:** Capability Demand is 3 (ambiguity 0, novelty 0, diagnostic uncertainty 0, semantic risk 1, context synthesis 2), which routes to Luna. Commit 2 already fixes the Rust command and DTO, so this commit follows exact Planner mutation, Query reconciliation, route-test, and smoke-stub patterns rather than designing persistence. Effort Demand is 7 after adjustments and routes to high. No hard floor or Luna punch-up applies.

##### Review profile

**Assigned reviewer:** Terra — `gpt-5.6-terra` at `high`

**Context:** Fresh. The reviewer receives this commit contract, the relevant feature context, packet, staged diff, validation evidence, and repository access before implementation notes.

**Mandate:**

- Verify Optimize uses the existing API/invoke boundary and returns Rust-owned reconciled depth rather than frontend matching.
- Verify the primary action is keyboard reachable, has phase-specific pending text, prevents duplicate execution, and reports success/error accurately.
- Verify Optimize changes every configured team/string through the backend result, while Clear Squad remains scoped to only the selected team.
- Verify depth replacement and all slot-candidate invalidation occur after Optimize and do not leave stale assignment locations.
- Verify smoke tests exercise a user-visible control path but do not claim to prove Rust, SQLite, or the native WebView.
- Verify blank cells render as the existing gap signal and no deferred transfer-recommendation UI appears.

##### Implementation packet

###### Governing requirements and invariants

- Optimize triggers the Rust mutation that recomputes every team and string.
- React displays the returned depth and invalidates candidate data. It does not implement matching or eligibility.
- Optimize is the panel's primary action. Clear Squad stays destructive and applies only to the selected team.
- Pending, success, and error feedback must be visible and usable from keyboard navigation.
- Blank post-optimization cells remain honest empty cells rather than invented recommendations.

###### Existing patterns to follow

- Planner API wrappers such as `assign-planner-player.ts` and `remove-planner-string.ts` call `invokeCommand` and return `PlannerDepth`.
- `PlannerDepthMatrix` uses `useMutation`, `queryClient.setQueryData(plannerKeys.depth(), nextDepth)`, `invalidateQueries({ queryKey: plannerKeys.slotCandidates() })`, controlled errors, and `Panel.actions` support.
- `PlannerTacticEditor` provides the local pending, success, and error status pattern.
- `Panel`, `Button`, `Modal`, `plannerKeys`, `planner-ipc-mock.ts`, `planner.test.tsx`, `e2e/smoke.spec.ts`, and `e2e/tauri-ipc-stub.ts` are the exact UI and test seams.
- No existing optimizer frontend API or smoke test exists. Do not invent a client-side algorithm analogue.

###### Expected change surface

- **Likely modified:** `src/features/planner/components/planner-depth-matrix.tsx`, `src/app/routes/planner.test.tsx`, `src/testing/planner-ipc-mock.ts`, `e2e/tauri-ipc-stub.ts`, `e2e/smoke.spec.ts`.
- **Likely added:** one Planner optimizer API wrapper under `src/features/planner/api/`.
- **Ownership boundaries:** React owns mutation presentation, Query reconciliation, and browser stubs. Rust behavior from commit 2 is not changed. Clear Squad implementation from commit 3 is reused, not redefined.
- **Do not change without replanning:** optimizer invariants, return DTO shape, Clear Squad scope, Planner cache-key ownership, Design-system primary/destructive hierarchy, or smoke ownership boundaries.

###### State and data design

- The optimizer has no new persisted frontend state. `useMutation` owns pending/error state and Planner depth query remains the source of displayed data.
- On success, replace the depth cache from the command response, invalidate all slot-candidate keys, and render a concise success status. On error, retain prior depth and show the command error. Disable the Optimize trigger while pending.
- Clear Squad continues to read `selectedTeam` from `PlannerDepthMatrix`. Both actions live in the same `Panel.actions` row: Optimize primary, Clear Squad destructive secondary.
- Browser stubs model only command-level UI results required for smoke. Rust tests remain the proof of migration, database transaction, eligibility, and exact matching.

###### Expected interfaces

- Add one Planner frontend mutation wrapper returning existing `PlannerDepth` through `invokeCommand`.
- Reuse `plannerKeys.depth()` and `plannerKeys.slotCandidates()`; do not add a duplicate optimizer cache store.
- Extend the existing IPC test doubles for the registered optimizer command and clear command if the final smoke path invokes both.

###### Execution order

1. Write route RED coverage for Optimize pending, success, error, depth replacement, and candidate invalidation.
2. Add the narrow Planner API wrapper and `useMutation` control in the existing depth matrix Panel actions.
3. Compose primary Optimize with the already-built destructive Clear Squad action and use existing status patterns.
4. Extend the Planner mock and add smoke RED coverage through the Vite IPC stub.
5. Run the full validation ladder and inspect keyboard action order in tests.

###### Validation ladder

1. Targeted Planner route integration tests through `mockIPC`.
2. Existing Planner route and Button tests.
3. `./scripts/dev test`.
4. `./scripts/dev check`.
5. `./scripts/dev smoke` with the new Optimize control path.
6. If the environment supports it, manually check the native Planner at 1280×800 and 1600×900; otherwise record this as unavailable rather than claiming it passed.

###### Stop conditions

- Stop and replan if the backend result cannot reconcile the existing depth model, selected-team Clear Squad cannot coexist in `Panel.actions` without changing scope, candidate invalidation has no reliable test seam, or browser stubs need to reimplement optimizer business logic.

###### Allowed discretion

- Exact local status copy, icon choice from installed Lucide icons, control ordering within the documented actions row, and focused route/smoke test names.

###### Prohibited discretion

- Client-side matching, silently hiding blanks, altering Clear Squad scope, treating smoke as Rust proof, adding a dependency, or adding deferred recommendation/constraint UI.

##### Escalation conditions

- **Increase effort when:** Backend ownership is correct but mutation status, cache invalidation, smoke setup, or keyboard coverage misses a bounded case.
- **Increase model capability when:** The implementer moves domain logic to React, invents a second source of truth, or changes Clear Squad/optimizer invariants.
- **Replan when:** the returned depth contract, cache ownership, selected-team clear decision, browser-test boundary, or any optimizer invariant changes.

##### Execution metadata

```yaml
execution_profile:
  planner:
    model: gpt-5.6-terra
    effort: xhigh
  implementer:
    model: gpt-5.6-luna
    effort: high
    confidence: null
  capability_demand:
    residual_ambiguity: 0
    architectural_novelty: 0
    diagnostic_uncertainty: 0
    semantic_risk: 1
    context_synthesis: 2
    total: 3
    luna_punch_up_applied: false
    hard_floor: none — the persisted command contract is complete before this frontend-only integration commit
  effort_demand:
    implementation_breadth: 2
    branch_density: 2
    repository_discovery: 1
    validation_weakness: 1
    tool_coordination: 2
    adjustments: -1
    total: 7
    hard_floor: none
  reviewer:
    model: gpt-5.6-terra
    effort: high
    context_mode: fresh
  review_demand:
    missed_defect_consequence: 2
    hidden_interaction_complexity: 2
    validation_weakness: 1
    architectural_discretion: 1
    blast_radius: 2
    total: 8
    hard_floor: Terra High — frontend/backend integration and cache lifecycle
  review_mandate:
    - Verify the UI invokes Rust-owned optimization and reconciles returned depth.
    - Verify action state, keyboard path, and Clear Squad coexistence.
    - Verify candidate invalidation and smoke scope remain truthful.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - Existing contracts are correct but state or smoke coverage is incomplete.
  escalate_model_when:
    - The implementer creates client-side domain authority or changes an invariant.
  replan_when:
    - The DTO, cache ownership, clear scope, or optimizer contract changes.
  adjudicator:
    model: gpt-5.6-sol
    effort: high
    invoke_when:
      - Reviewer and implementer disagree about frontend/backend authority.
      - A high-severity lifecycle finding remains disputed.
      - A correction changes the feature plan.
```

## Active work

**PR:** PR 1 — Add squad optimization

**Commit:** Commit 1 — Persist assignment provenance

### RED test (active commit)

Create a v6-shaped Planner assignment, apply the new migration, and assert that the row is manual. Then prove that assigning and moving a player writes manual provenance. These tests fail if a future Optimize run would be allowed to replace prior user work; they do not merely inspect that a column exists.

### Expected outcome

The current Planner continues to behave identically, but every existing and newly manual assignment is durably distinguishable from a future optimizer row. The next commit can replace only optimizer-created rows without guessing intent.

### Explicit exclusions

- No optimizer command or allocation logic.
- No frontend provenance field or control.
- No Clear Squad action.
- No change to existing manual picker, club-family, tactic, score, or snapshot behavior.

### Assigned profiles

- **Implementation:** Terra xhigh — `gpt-5.6-terra`.
- **Review:** Terra xhigh — `gpt-5.6-terra`, fresh context.

### Current blockers

- None.

### Discoveries that may require replanning

- Native WebView checks remain environment-dependent, but the active migration commit has complete Rust-owned validation paths.

## Discoveries and replanning

- **Plan formed:** The planning pass proposed one PR with provenance, Rust optimizer, Clear Squad, and frontend Optimize commits. **Repository discovery:** migration v6, `depth.rs` mutation seams, `PlannerDepthDto`, `Panel.actions`, destructive Modal, candidate invalidation, and both test double layers exist. **Why it matters:** The shape is trunk-safe without an extra PR. **Architecture change:** None; it follows established Planner ownership. **Affected later work:** all four commits use the identified seams. **Routing impact:** Keep Terra xhigh for the existing-data migration and exact persisted optimizer, use Terra High for the bounded destructive cross-layer action and algorithm review, and use Luna High for the pattern-driven frontend controls. Sol remains reserved for the fixed feature-complete review or a true replanning condition.
- **Plan formed:** Clear Squad scope was not specified when the action was added. **Planning assumption:** It clears all assignments in the selected team only, after destructive confirmation, preserving other teams. **Why it matters:** It fixes the provisional Rust delete predicate and UI selected-team contract. **Architecture change:** None. **Affected later work:** commits 3 and 4. **Routing impact:** Confirm or replan those commits if the developer intended all-team clearing.

## Completed work

| PR | Commit | Hash | Notes | Implementer | Reviewer | Deviations |
| --- | --- | --- | --- | --- | --- | --- |
| — | — | — | No implementation has started. | — | — | — |

## Final validation

1. Run targeted Rust optimizer, provenance, migration, and clear-team tests. Include adversarial exact-matching, strict-priority, manual-reservation, score/age/position, rerun, and rollback cases.
2. Run the full Rust and frontend suite with `./scripts/dev test` and `./scripts/dev check`.
3. Run `./scripts/dev smoke` and verify the Planner control path with stubbed IPC, including Optimize status and the selected-team Clear Squad confirmation when its stub coverage exists.
4. Confirm keyboard-only access to Optimize, Clear Squad, the destructive confirmation, and the reconciled matrix. Confirm the matrix still scrolls horizontally and presents blank cells honestly.
5. Where the environment permits, manually inspect the native Planner at 1280×800 and 1600×900. If unavailable, record the limitation; Playwright smoke does not prove the native WebView, Rust commands, SQLite, migrations, or matching.
6. Dispatch the fixed fresh-context feature-complete reviewer: **Sol High**. It must assess end-to-end intent, cross-commit migration/optimizer/clear interactions, manual preservation through Load Data, cache lifecycle, temporary compatibility behavior, test honesty, and documentation accuracy.

### Feature review profile

- **Reviewer:** Sol High — `gpt-5.6-sol` at `high`, fresh context. This profile is fixed for every feature-complete review.
- **Mandate:** End-to-end user intent, cross-commit interaction, strict allocation and manual-preservation invariants, assignment provenance, destructive selected-team scope, stale candidate cache behavior, blank-gap honesty, duplicate abstractions, temporary compatibility layers, and documentation accuracy.

## Documentation impact

- This active ledger is the delivery-plan authority until the feature completes.
- At feature completion, reconcile the implemented optimizer and Clear Squad behavior with `CONCEPT.md`, `ARCHITECTURE.md`, and `DESIGN.md` only where implementation changes their durable claims. Then condense and archive this ledger under `.wiki/features/completed/` through `workflow-finish-feature`.
