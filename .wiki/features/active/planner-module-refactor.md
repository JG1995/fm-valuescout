# Planner Module Refactor

## Status

Active

## Intent

Reduce the size and mixed responsibilities of Planner depth and optimizer implementation files without changing Planner behavior or expanding its product boundary. Keep the Planner feature as the only home for depth, optimization, matrix presentation, and controls.

## User-visible behavior

- The `/planner` route continues to show the same three-team depth matrix, string menus, slot picker, Optimize squads action, and selected-team Clear Squad action.
- Keyboard paths, accessible names, confirmation dialogs, pending states, errors, success messages, and focus restoration remain unchanged.
- Existing Tauri command names, arguments, DTOs, SQLite rows and provenance, optimizer results, cache replacement, invalidation, and the browser smoke path remain unchanged.

## Invariants

- `features::planner::commands` and `src-tauri/src/lib.rs` retain the existing Planner command registration and command names.
- `PlannerDepthDto`, related Rust-to-TypeScript DTO mappings, and frontend Planner types remain behaviorally and structurally compatible.
- Planner persistence remains save-scoped. No migration, schema, provenance, transaction boundary, SQL behavior, or assignment uniqueness rule changes.
- `optimize_planner_depth` retains manual rows, replaces only optimizer rows, allocates in Senior, Reserves, Youth and string order, preserves the exact matcher objective, and rolls back on failure.
- `clear_planner_team` remains confirmed and selected-team only.
- Planner API fetchers stay in `src/features/planner/api/`; routes remain composition and query-loader wiring; Planner components do not import another feature.
- Successful Planner mutations continue to replace `plannerKeys.depth()` and invalidate `plannerKeys.slotCandidates()` where they do now.
- The existing matrix keyboard tab, header-menu, picker, and confirmation paths remain operable and retain their current accessible names.

## Non-goals

- New product behavior, endpoints, command names, DTOs, migrations, dependencies, traits, repositories, public abstractions, or a top-level optimizer feature.
- Changes to optimizer eligibility, matching, age, suitability, score, pool, priority, provenance, transaction, or rollback rules.
- Changes to query keys, cache policy, fetcher ownership, route behavior, smoke scope, visual design, or accessibility requirements.
- Cross-feature imports, route expansion, unrelated cleanup, or a rewrite of completed Planner and Optimizer history.

## Current-state map

- **Relevant components:** `src-tauri/src/features/planner/depth.rs` is a 2,214-line Planner module. It currently combines public depth read-model types, exact matching (`MatchGraph` and `match_lanes`), optimizer persistence, depth reads and assignment resolution, slot candidates, string and assignment mutations, and its Rust tests. `src/features/planner/components/planner-depth-matrix.tsx` is a 797-line feature component. It combines presentational matrix markup, string/header behavior, picker and focus coordination, string mutations, Optimize, and Clear Squad.
- **Data model:** `PlannerDepth`, `PlannerDepthTeam`, `PlannerString`, `PlannerAssignment`, `PlannerSlotCandidate`, `PlannerTeam`, and `AssignmentState` remain feature-local Rust values. `src-tauri/src/features/planner/commands.rs` maps them to the existing camel-case DTOs; `src/features/planner/types/depth.ts` mirrors the read model for the WebView.
- **Persistence and migrations:** Planner string and assignment rows are save-scoped. Architecture §5.5 records migration v7 provenance (`manual` or `optimizer`), snapshot replacement behavior, and the one-transaction optimizer contract. This feature does not modify `src-tauri/src/db/migrations.rs`.
- **Existing behavioral assumptions:** `optimize_depth` calls `ensure_depth`, retains manual assignment UIDs and lanes, deletes only optimizer rows in one transaction, allocates ordered strings from team-scoped candidates, commits, then reads depth. `clear_team` requires confirmation and deletes only the chosen team's assignments. `get_depth` preserves resolved, outside-pool, and unresolved assignment rendering.
- **Architectural seams:** `src-tauri/src/features/planner/mod.rs` already declares feature submodules; `commands.rs` consumes `depth` through its existing public functions and types. `src/app/routes/planner.tsx` is already thin loader and composition wiring. `planner-depth-matrix.tsx` already uses only Planner API wrappers and `plannerKeys`.
- **Test ownership:** Rust characterization and transaction coverage is currently under `depth.rs`'s `#[cfg(test)]` module. `src/app/routes/planner.test.tsx` covers matrix rendering, keyboard tabs, string header menus and focus, selected-team clearing, cache reconciliation, optimizer success/error/pending behavior, and no duplicate actions. `e2e/smoke.spec.ts` covers the Planner Optimize control path through `e2e/tauri-ipc-stub.ts`.
- **Authoritative validation commands:** `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` is unsupported and must not be reported as passed.
- **Likely reuse points:** Planner's current feature-local `tactic.rs`, `service.rs`, and `commands.rs` establish submodule and thin-command conventions. The original optimizer ledger at commit `8aa4e87` records the existing depth, transaction, cache, and control contracts that this refactor must preserve.
- **Known technical risks:** Moving private Rust helpers can accidentally change transaction scope, matcher ties, SQL parameter usage, or test visibility. Moving React state can accidentally duplicate mutation state, change query invalidation, or alter the accessible keyboard and focus contract.
- **Applicable repository patterns:** Architecture §§1, 2.1, 2.2, 2.3, 5.5, and 6.4; Rust feature modules in `src-tauri/src/features/planner/`; frontend feature API/query-key ownership in `src/features/planner/api/`; Planner route integration tests and stubbed smoke tests.

## Feature architecture (this feature)

The Rust work replaces the monolithic `depth.rs` implementation with sibling `planner::depth` and `planner::optimizer` internal capabilities. The depth module keeps Planner depth types, reads, assignment resolution, slot candidates, and string and assignment mutations. The optimizer module owns candidate loading, eligibility, the transaction, and the exact matcher. `commands.rs` can adjust its private Rust imports while every Tauri command and DTO stays unchanged. The split is structural only; it does not create a reusable optimizer outside Planner.

The React work retains `planner-depth-matrix.tsx` as the Planner composition and interaction coordinator. Planner-owned components take over the presentational depth table, Optimize controls, and Clear Squad controls. The coordinator remains the single owner of cross-control mutation coordination, picker/string state, cache updates, invalidation, and focus behavior so no second source of truth appears.

## Uncertainty register

### Known

- `commands.rs` imports `depth` types and functions, while `src-tauri/src/lib.rs` registers the existing Planner commands by their current paths.
- `planner-depth-matrix.tsx` uses `plannerKeys.depth()` for cache replacement and `plannerKeys.slotCandidates()` for invalidation after string removal, Optimize, and Clear Squad.
- Existing Rust tests cover exact matching, manual reservation, priority ordering, eligibility, source scoping, reruns, rollback, candidate ordering, confirmation, uniqueness, snapshot replacement, and save isolation.
- Existing route tests cover the matrix's user-facing interactions and error states; the existing smoke path only stubs and drives Optimize.

### Assumptions

- The current Rust test module can move into a test-only child module while keeping private optimizer helpers testable through Rust module visibility. This does not block planning; prove it with the targeted Rust tests before changing broader code.
- The presentational React controls can receive current coordinator callbacks and state without changing their accessible output or creating a new shared store. This does not block the first Rust commit.

### Decisions

- Deliver one short-lived PR with two ordered commits. They share one Planner maintainability review surface and neither has an independent merge boundary.
- Keep all new modules and components inside `features/planner`; do not create a cross-feature optimizer abstraction.
- Use existing tests as characterization evidence. Do not invent a RED behavioral change for a behavior-preserving structural move.
- Plan with Terra xhigh because the repository already supplies the architecture, feature boundary, command/data contracts, and useful Planner analogues. No generic Sol High planning pass is required: no contract is unsettled and no Sol condition applies.

### Unknowns

- No gating unknowns.
- The sibling `planner::depth` and `planner::optimizer` responsibility boundary is fixed. Final private child-file names and helper placement inside those capabilities are local implementation discretion. This blocks neither commit.

## Risks

### Optimizer behavior changes during private-module extraction

- **Trigger:** A moved helper changes the transaction scope, manual reservation, candidate source filtering, matcher comparison, or rollback path.
- **Consequence:** Planner can persist an incorrect allocation or replace protected manual work.
- **Mitigation:** Preserve the current command and `depth` function surfaces; move existing characterization tests with the behavior they protect; run targeted Rust tests before the full gate.
- **Proof:** Commit 1 targeted Rust coverage and `./scripts/dev check`; fresh Terra High review of the staged diff.

### Matrix decomposition breaks a UI lifecycle contract

- **Trigger:** A child component duplicates mutation state, misses a depth-cache replacement or candidate invalidation, or changes a dialog, pending, focus, tab, or ARIA path.
- **Consequence:** The depth table becomes stale, actions duplicate, or keyboard users lose an existing interaction.
- **Mitigation:** Keep coordination in `PlannerDepthMatrix`; pass existing state and callbacks to Planner-owned presentation components; retain the current route characterization tests and Optimize smoke path.
- **Proof:** Commit 2 route tests, `./scripts/dev check`, `./scripts/dev smoke`, and fresh Terra High review.

### Refactor expands beyond Planner

- **Trigger:** A proposed split requires API, route, DTO, schema, or cross-feature changes.
- **Consequence:** The maintenance PR gains an unplanned product or architectural boundary.
- **Mitigation:** Treat each as a replanning condition and stop before adding it.
- **Proof:** Both commit reviews verify the exact expected change surfaces and exclusions.

## Walking skeleton

Commit 1 moves the existing Rust depth and optimizer implementation plus its characterization tests into cohesive Planner-private modules while every current Planner command still compiles and the same Rust tests pass. It proves the refactoring seam on trunk before Commit 2 reduces the React matrix to Planner-owned composition of the same table and controls.

## Delivery plan

### PR 1 — Planner Module Refactor

**Status:** Active

**Provisional PR title:** `refactor(planner): decompose planner modules`

**Purpose:** Make the two oversized Planner implementations easier to navigate and review while preserving the complete current Planner contract.

**Depends on:** The completed Squad Planner and Squad Optimizer behavior recorded in `.wiki/features/completed/squad-planner.md` and `.wiki/features/completed/squad-optimizer.md`.

**Merge boundary:** One PR keeps the related Rust and React modularity work on one Planner maintainability review surface. Each ordered commit is independently trunk-safe and revertible; a second PR would not provide a separate behavior or merge boundary.

#### Commit 1 — Split Planner depth and optimizer modules

**Status:** Active

**Work:** Replace `src-tauri/src/features/planner/depth.rs` with cohesive sibling `planner::depth` and `planner::optimizer` modules. Keep depth reads and mutations under `depth`, move candidate loading, eligibility, the optimizer transaction, and the exact matcher under `optimizer`, adjust only private command imports, and separate the existing Rust characterization tests by capability.

**Out of scope for this commit:**

- React files, query keys, browser stubs, routes, UI components, or smoke tests.
- Command registration or names; DTO or type changes; migrations, SQL contract changes, provenance changes, traits, repositories, or public abstractions.
- Optimizer algorithm, eligibility, ordering, transaction, assignment, candidate, string, or clear-team behavior changes.

**Validation:** Run the moved, targeted Rust characterization tests for matching, optimizer lifecycle, reads, candidates, and mutations; then run `./scripts/dev check`. Existing tests are the characterization proof for this structural refactor. `./scripts/dev smoke` is not required because this commit does not alter browser code.

**Provisional commit:** `refactor(planner): split depth optimizer modules`

##### Implementation profile

**Assigned implementer:** Terra — `gpt-5.6-terra` at high

**Routing summary:** Capability Demand is 6 (residual ambiguity 1, architectural novelty 1, diagnostic uncertainty 0, semantic and consequence risk 2, context synthesis 2), which routes to Terra. This is an internal structural change, but it touches the current persisted transaction and exact-matching implementation. Effort Demand is 8 after adjustments (breadth 3, branch density 3, discovery 1, validation weakness 1, tool coordination 2; minus 3 for existing deterministic coverage, known ownership, and explicit order; plus 1 for backward-compatibility preservation), which routes to high. No implementation hard floor applies because no persistence or cache semantics change; Terra High is assigned by the independent demand scores. The Luna punch-up does not apply.

##### Review profile

**Assigned reviewer:** Terra — `gpt-5.6-terra` at high

**Context:** Fresh. The reviewer receives the commit contract, relevant feature context, packet, diff, validation, and repository access before implementation notes.

**Mandate:**

- Verify `commands.rs` retains every existing Tauri command function, name, and DTO conversion, and `lib.rs` retains its current command registration; only the private call to `optimize_depth` may move from `planner::depth` to `planner::optimizer`.
- Trace the optimizer transaction to confirm manual retention, optimizer-only replacement, ordered allocation, persistence, rollback, and post-commit depth read retain their current behavior.
- Challenge the extracted matcher against its total-score, filled-lane, zero-score, blank-lane, and stable UID tie-break characterization tests.
- Verify team source scope, age/position/score eligibility, assignment uniqueness, and resolved/outside-pool/unresolved read behavior remain in Planner and unchanged.
- Verify test separation preserves, rather than weakens, the existing transaction and behavioral coverage; reject a test-only refactor that stops exercising real SQLite behavior.
- Reject migrations, public abstractions, new dependencies, cross-feature imports, or unrelated changes.

##### Implementation packet

###### Governing requirements and invariants

- Preserve the current command-facing Planner depth types and behavior consumed by `src-tauri/src/features/planner/commands.rs`. Only the private Rust module path for `optimize_depth` may change.
- Preserve the SQLite schema, migration version, assignment provenance, SQL parameterization, transaction semantics, and all Planner behavior listed in this ledger's invariants.
- Keep exact matcher and persistence logic internal to Planner. Do not make an optimizer feature or reusable public framework.

###### Existing patterns to follow

- `src-tauri/src/features/planner/mod.rs` for feature-local submodule declarations.
- `src-tauri/src/features/planner/tactic.rs` and `service.rs` for cohesive Planner-local Rust modules.
- `src-tauri/src/features/planner/commands.rs` for the command-to-domain mapping that must remain unchanged.
- `src-tauri/src/features/planner/depth.rs` tests for existing real SQLite setup and characterization coverage.
- The original optimizer plan at Git commit `8aa4e87` for manual-retention, transaction, cache, and optimizer boundaries. No useful pre-existing refactor analogue splits this exact mixed file; use the established Planner module layout rather than inventing a new abstraction.

###### Expected change surface

- **Likely modified:** `src-tauri/src/features/planner/mod.rs`, `src-tauri/src/features/planner/commands.rs` for private module imports, and the current `depth.rs` implementation as it becomes the depth module root.
- **Likely added:** private files below `src-tauri/src/features/planner/depth/` for depth implementation and tests, plus `src-tauri/src/features/planner/optimizer/` for candidate loading, eligibility, transaction orchestration, exact matching, and optimizer tests.
- **Ownership boundaries:** The new modules stay private to Planner. `commands.rs` remains the IPC boundary; `db/migrations.rs` remains the migration owner; `tactic.rs` remains tactic ownership.
- **Do not change without replanning:** command registration, DTO shape, SQL/schema/provenance, transaction and optimizer behavior, Planner persistence scope, external visibility, or any frontend file.

###### State and data design

- SQLite remains the source of truth for strings and assignments. Current snapshots remain the source for player identity, club pool, positions, and role scores.
- The optimizer continues to reserve manual player UIDs, replace only persisted optimizer assignments within one transaction, and return the existing `PlannerDepth` after commit.
- Read paths continue to resolve assignments without changing stored intent. There is no draft, cache, or frontend state in this commit.

###### Expected interfaces

- Keep depth reads, slot candidates, string and assignment mutations, `PlannerTeam`, and depth read-model types under `planner::depth`. Move only `optimize_depth` and its private supporting logic to `planner::optimizer`; update `commands.rs` to call the new private path without changing the Tauri command or DTO contract.
- Keep current command DTO mappings and Tauri function signatures unchanged.
- Internal optimizer, read, mutation, and test helpers can move behind private child modules. Do not introduce a trait, repository, new public interface, or generic optimizer API.

###### Execution order

1. Run the current targeted Rust characterization tests to establish the pre-move baseline.
2. Create sibling private `planner::depth` and `planner::optimizer` module structures and update only their internal Planner wiring.
3. Keep depth types, reads, assignment resolution, slot candidates, and mutations under `depth`; move candidate loading, eligibility, exact matching, and optimization transaction work under `optimizer`; separate test-only characterization coverage by capability.
4. Adjust only internal imports and visibility needed for the unchanged command boundary and tests.
5. Run targeted tests after each cohesive move, then `./scripts/dev format` and `./scripts/dev check`.

###### Validation ladder

1. Targeted Rust tests for `matcher_`, `optimizer_`, candidates, strings, assignments, snapshot replacement, and save isolation in the extracted Planner test module.
2. The affected Planner Rust test module as a whole.
3. Existing command/DTO compilation through the Rust test and clippy targets inside `./scripts/dev check`.
4. `./scripts/dev check`.
5. No browser integration or smoke command is required for this Rust-only commit.

###### Stop conditions

- Stop and return to planning if preserving `commands.rs` requires a command, DTO, or public path change.
- Stop if module separation exposes a required schema, migration, transaction, or optimizer-contract change.
- Stop if an existing characterization test cannot run against real Planner persistence after the move, or if a test failure indicates behavior changed rather than a path/import adjustment.
- Stop if this commit requires React/cache/stub changes, a cross-feature dependency, new abstraction, or new dependency.

###### Allowed discretion

- Private module and helper names, internal import placement, private visibility, test submodule organization, and move order that preserve all current behavior and listed boundaries.

###### Prohibited discretion

- Command/DTO/migration changes; public APIs; traits/repositories; matching or transaction rules; SQL behavior; persistence ownership; frontend changes; and validation reduction.

##### Escalation conditions

- **Increase effort when:** The module boundary is correct but compiler, test, or import failures expose missed private visibility, test setup, or transaction paths.
- **Increase model capability when:** The implementer proposes a new public optimizer abstraction, misreads an existing persistence invariant, or changes behavior to make the split easier.
- **Replan when:** A known command, DTO, transaction, schema, matcher invariant, validation seam, or PR boundary cannot remain unchanged.

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
    residual_ambiguity: 1
    architectural_novelty: 1
    diagnostic_uncertainty: 0
    semantic_risk: 2
    context_synthesis: 2
    total: 6
    luna_punch_up_applied: false
    hard_floor: none
  effort_demand:
    implementation_breadth: 3
    branch_density: 3
    repository_discovery: 1
    validation_weakness: 1
    tool_coordination: 2
    adjustments: -2
    adjustment_detail:
      - minus_existing_deterministic_coverage
      - minus_known_files_and_ownership
      - minus_explicit_execution_order
      - plus_backward_compatibility_preservation
    total: 8
  reviewer:
    model: gpt-5.6-terra
    effort: high
    context_mode: fresh
  review_demand:
    missed_defect_consequence: 2
    hidden_interaction_complexity: 3
    validation_weakness: 1
    architectural_discretion: 1
    blast_radius: 2
    total: 9
    hard_floor: terra_high_for_persistence_and_state_lifecycle_review
  review_mandate:
    - Preserve Planner command registration and DTO mappings.
    - Preserve optimizer transaction and rollback semantics.
    - Preserve exact matcher objective and stable ties.
    - Preserve candidate and assignment read behavior.
    - Preserve real characterization coverage after test extraction.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - The design is correct but module visibility, imports, or tests expose missed execution paths.
  escalate_model_when:
    - The implementer misunderstands Planner ownership, public surface, or a persistence invariant.
  replan_when:
    - A command, DTO, schema, transaction, matcher, validation seam, or PR boundary must change.
  adjudicator:
    model: gpt-5.6-sol
    effort: medium
    invoke_when:
      - Reviewer and implementer disagree about a Planner architecture boundary.
      - A high-severity persistence or transaction finding remains disputed.
      - A correction would change the feature plan.
```

#### Commit 2 — Split Planner depth matrix controls

**Status:** Pending

**Work:** Decompose `planner-depth-matrix.tsx` into Planner-owned presentational table, Optimize controls, and Clear Squad control components. Retain `PlannerDepthMatrix` as the composition point for selected-team, picker, string-menu, removal, focus, mutation coordination, depth-cache replacement, and candidate invalidation behavior.

**Out of scope for this commit:**

- Rust code, command registration, DTOs, migrations, optimizer rules, or browser IPC stub behavior.
- New API fetchers, query keys, cache stores, state libraries, hooks, dependencies, design-system primitives, routes, or cross-feature imports.
- Any visual, wording, keyboard, focus, accessibility, confirmation, pending, success, error, query cache, or smoke-path behavior change.

**Validation:** Run focused Planner route tests for matrix rendering, keyboard tabs, header menus, selected-team Clear Squad, and Optimize states; then run `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`. Existing route and smoke tests are characterization evidence; no invented RED behavior is appropriate for this refactor.

**Provisional commit:** `refactor(planner): split depth matrix controls`

##### Implementation profile

**Assigned implementer:** Terra — `gpt-5.6-terra` at high

**Routing summary:** Capability Demand is 5 (residual ambiguity 1, architectural novelty 1, diagnostic uncertainty 0, semantic and consequence risk 1, context synthesis 2), which would initially route to Luna. Terra High is the hard floor because cache invalidation is central to the current controls and several asynchronous mutation, error, pending, confirmation, and focus states interact. Effort Demand is 7 after adjustments (breadth 3, branch density 3, discovery 1, validation weakness 1, tool coordination 2; minus 3 for deterministic route coverage, known ownership, and explicit order), which independently routes to high. The Luna punch-up does not apply because the cache/lifecycle hard floor is active.

##### Review profile

**Assigned reviewer:** Terra — `gpt-5.6-terra` at high

**Context:** Fresh. The reviewer receives the commit contract, relevant feature context, packet, diff, validation, and repository access before implementation notes.

**Mandate:**

- Verify API fetchers remain in `features/planner/api`, `PlannerDepthMatrix` does not create a second Query cache or state owner, and no component imports across features.
- Trace Optimize and Clear Squad success and error paths to confirm depth-cache replacement, slot-candidate invalidation, messages, and duplicate-action prevention match the current contract.
- Verify selected-team clear scope, destructive confirmation, cancel/error behavior, pending labels, and dialog accessibility remain unchanged.
- Verify tab keyboard navigation, matrix cell accessible names and state labels, horizontal overflow, header menu and context-menu behavior, and origin-focus restoration remain reachable through the extracted presentation boundary.
- Verify the Planner route remains loader/composition-only and existing route tests and Optimize smoke path still exercise the real component behavior rather than mocks of extracted components.
- Reject new shared abstractions, dependencies, API/route changes, UI redesign, or any carry-over of backend work.

##### Implementation packet

###### Governing requirements and invariants

- Preserve every current Planner UI contract and the cache, query-key, and API ownership rules in this ledger.
- Keep `PlannerDepthMatrix` as the coordinator for state shared by table, picker, string actions, Optimize, and Clear Squad.
- Keep extracted components feature-local and presentational. They receive existing state/callbacks; they do not call new APIs or reconstruct optimizer logic.

###### Existing patterns to follow

- `src/features/planner/components/planner-depth-matrix.tsx` for the exact current markup, local state, query mutations, ARIA labels, focus timers, and cache updates.
- `src/features/planner/components/planner-slot-fit-picker.tsx` for Planner-local component ownership and callback composition.
- `src/features/planner/api/planner-keys.ts`, `optimize-planner-depth.ts`, and `clear-planner-team.ts` for unchanged cache-key and IPC wrapper ownership.
- `src/app/routes/planner.tsx` for thin loader and feature composition.
- `src/app/routes/planner.test.tsx` for integration-style characterizations of keyboard, focus, Clear Squad, and Optimize behavior.
- `e2e/smoke.spec.ts` and `e2e/tauri-ipc-stub.ts` for the existing Optimize browser smoke boundary. No useful analogue exists for this exact extraction; use the existing Planner feature composition rather than introducing a shared control framework.

###### Expected change surface

- **Likely modified:** `src/features/planner/components/planner-depth-matrix.tsx` and `src/app/routes/planner.test.tsx` only when imports or test names must follow the extracted, still user-observable composition.
- **Likely added:** Planner-local `planner-depth-table.tsx`, `planner-optimizer-controls.tsx`, and `planner-clear-team-control.tsx` components, or equivalently named kebab-case Planner-local files with the same three responsibilities.
- **Ownership boundaries:** API wrappers and query keys remain in `src/features/planner/api/`; `src/app/routes/planner.tsx` remains thin; the new components stay under `src/features/planner/components/`; shared UI primitives remain unchanged.
- **Do not change without replanning:** command/DTO/API/query-key contracts, cache reconciliation/invalidation, route behavior, browser stub contract, matrix behavior, accessibility semantics, or feature-import boundaries.

###### State and data design

- TanStack Query remains the source of displayed Planner depth. Successful mutations keep replacing `plannerKeys.depth()` and invalidating `plannerKeys.slotCandidates()` at the same successful boundaries.
- `PlannerDepthMatrix` retains selected team, picker, string/menu/removal, focus, and cross-control coordination state. It passes derived presentation data and callbacks to extracted child components.
- Optimizer and clear error/status/pending states retain their current visible behavior and mutual action-disabling behavior. No state moves to Zustand, the route, or a new cache.

###### Expected interfaces

- Preserve the `PlannerDepthMatrix` props used by `src/app/routes/planner.tsx`.
- Add only feature-private component prop types needed to pass the current data and callbacks between `PlannerDepthMatrix`, the presentational table, Optimizer controls, and Clear Squad control.
- Keep `optimizePlannerDepth`, `clearPlannerTeam`, `plannerKeys`, and all existing frontend DTO types at their current paths and signatures.

###### Execution order

1. Run focused existing route tests for matrix interaction, Clear Squad, and Optimize as the baseline.
2. Extract the presentational table while preserving its existing accessible markup and callback surface.
3. Extract Optimize and Clear Squad presentation around the coordinator's existing mutation and cache lifecycle state; retain their current control coordination.
4. Keep picker, string actions, selected team, focus timers, and Query coordination in `PlannerDepthMatrix`; adjust only feature-local imports and prop wiring.
5. Run focused route characterizations, then `./scripts/dev test`, `./scripts/dev format`, `./scripts/dev check`, and `./scripts/dev smoke`.

###### Validation ladder

1. Focused `src/app/routes/planner.test.tsx` cases for shared lanes, keyboard tabs, header menus, focus restoration, selected-team clear success/error/pending, and optimizer success/error/pending.
2. `./scripts/dev test` for the frontend suite.
3. Type checking and the Rust/frontend quality targets through `./scripts/dev check`.
4. `./scripts/dev check`.
5. `./scripts/dev smoke` for the existing Planner Optimize control path through the IPC stub.
6. No native WebView check is required beyond the existing documented limitation; smoke remains a browser-with-stub proof, not Rust/SQLite proof.

###### Stop conditions

- Stop and return to planning if the component split requires a new API, query key, cache owner, route state, shared store, dependency, command, DTO, or cross-feature import.
- Stop if the existing route tests cannot continue to exercise actual Planner behavior, or if accessible names, keyboard/focus paths, confirmation behavior, or cache reconciliation change.
- Stop if preserving control coordination requires a user-visible behavior change or a browser stub contract change.
- Stop if any Rust, persistence, or optimizer change becomes necessary.

###### Allowed discretion

- Kebab-case component file names, private prop types, placement of existing formatting helpers, and component markup extraction that retains the exact current Planner contract.

###### Prohibited discretion

- Query/API ownership, cache keys, depth data ownership, command behavior, control semantics, visible text, keyboard or focus behavior, visual redesign, shared abstractions, and cross-feature imports.

##### Escalation conditions

- **Increase effort when:** The component boundary is correct but TypeScript, route tests, or smoke expose missed prop, pending, focus, or cache lifecycle paths.
- **Increase model capability when:** The implementer moves Query or domain state into a child/route/store incorrectly, duplicates cache state, or proposes a cross-feature abstraction.
- **Replan when:** An API/query-key/cache contract, accessibility invariant, command/DTO boundary, smoke scope, or PR boundary must change.

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
    residual_ambiguity: 1
    architectural_novelty: 1
    diagnostic_uncertainty: 0
    semantic_risk: 1
    context_synthesis: 2
    total: 5
    luna_punch_up_applied: false
    hard_floor: terra_high_for_cache_invalidation_and_async_lifecycle
  effort_demand:
    implementation_breadth: 3
    branch_density: 3
    repository_discovery: 1
    validation_weakness: 1
    tool_coordination: 2
    adjustments: -3
    adjustment_detail:
      - minus_existing_deterministic_coverage
      - minus_known_files_and_ownership
      - minus_explicit_execution_order
    total: 7
  reviewer:
    model: gpt-5.6-terra
    effort: high
    context_mode: fresh
  review_demand:
    missed_defect_consequence: 1
    hidden_interaction_complexity: 3
    validation_weakness: 1
    architectural_discretion: 1
    blast_radius: 2
    total: 8
    hard_floor: terra_high_for_cache_invalidation_and_state_lifecycle_review
  review_mandate:
    - Preserve feature-local API and Query ownership.
    - Preserve depth-cache replacement and candidate invalidation.
    - Preserve selected-team clear confirmation and pending/error behavior.
    - Preserve tab, table, menu, picker, focus, and accessible-name paths.
    - Preserve thin routes and the existing Planner smoke boundary.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - The ownership is correct but validation reveals missed cache, pending, focus, or interaction paths.
  escalate_model_when:
    - The implementer misunderstands Planner state ownership, Query cache behavior, or feature boundaries.
  replan_when:
    - An API, query-key, cache, accessibility, command/DTO, smoke, or PR boundary must change.
  adjudicator:
    model: gpt-5.6-sol
    effort: medium
    invoke_when:
      - Reviewer and implementer disagree about Planner component ownership.
      - A high-severity cache or accessibility finding remains disputed.
      - A correction would change the feature plan.
```

## Active work

**PR:** Planner Module Refactor

**Commit:** Split Planner depth and optimizer modules

### RED test (active commit)

No new RED test is appropriate. This commit is a behavior-preserving move of existing Rust code and test code, not a behavioral change. The existing matcher, optimizer transaction, candidate, assignment, snapshot, and save-isolation tests are characterization evidence; a new deliberately failing behavioral test would invent a contract. Run the targeted baseline before moving code and keep those tests green after each move.

### Expected outcome

The existing Planner depth and optimizer command behavior compiles and passes its current characterization tests after the oversized source and test block become cohesive private Planner modules.

### Explicit exclusions

- No frontend, route, query, stub, smoke, schema, migration, command, DTO, or product behavior change.
- No new dependency, trait, repository, public abstraction, top-level optimizer feature, or optimizer rule change.

### Assigned profiles

- **Implementation:** Terra High — `gpt-5.6-terra` at `high`
- **Review:** Terra High — `gpt-5.6-terra` at `high`, fresh context

### Current blockers

- None.

### Discoveries that may require replanning

- None. The implementation must stop rather than change a command, DTO, schema, transaction, or optimizer invariant to make the module split compile.

## Discoveries and replanning

- **Plan formed:** The feature uses one PR and two commits: a Rust-private refactor followed by a React-private refactor. **Repository discovery:** `depth.rs` combines exact matching, transactions, read resolution, mutations, and tests; the matrix combines table markup, string behavior, control mutations, focus, and cache updates. `commands.rs`, `lib.rs`, Planner API files, route tests, and the existing Optimize smoke path supply stable seams. **Why it matters:** each commit can be reviewed and merged safely without changing the Planner product contract. **Architecture change:** none; this applies the existing feature-local module boundaries. **Affected later work:** Commit 2 depends only on Commit 1 landing cleanly in the same PR, not on a changed interface. **Routing impact:** Terra High implementation and fresh Terra High review for each commit; Sol is reserved for specified replanning or final feature review.

## Completed work

| PR | Commit | Hash | Notes | Implementer | Reviewer | Deviations |
| --- | --- | --- | --- | --- | --- | --- |
| — | — | — | No implementation commits yet | — | — | — |

## Final validation

1. Run `./scripts/dev test` for all current frontend characterizations.
2. Run `./scripts/dev check`, including Rust format, clippy, and tests plus frontend lint, types, and secret scan.
3. Run `./scripts/dev smoke` and verify the existing Planner Optimize smoke path remains available through the Vite/IPC stub boundary.
4. Review the Planner route test coverage for keyboard-only team tabs, header menu/context-menu operation, focus return, slot picker, selected-team Clear Squad confirmation and failure handling, and optimizer success/error/pending behavior.
5. Confirm no migration, schema, command registration, command name, DTO, query-key, cache semantics, API ownership, route thinness, cross-feature import, visual, or accessible behavior changed.
6. Dispatch the fixed fresh-context feature-complete reviewer: Sol High — `gpt-5.6-sol` at `high`. The review checks cross-commit structural coherence, preserved Planner behavior, cache and lifecycle paths, temporary compatibility layers, duplicate abstractions, test honesty, and documentation accuracy.

### Feature review profile

- **Reviewer:** Sol High — `gpt-5.6-sol` at `high`, fresh context. This profile is fixed for every feature-complete review.
- **Mandate:** End-to-end intent, cross-commit integration, feature invariants, duplicated abstractions, lifecycle paths, temporary compatibility layers, and documentation accuracy.

## Documentation impact

This active ledger and `.wiki/TODO.md` record the maintenance work. No current-state architecture, product, design, schema, or completed-feature documentation update is expected because the delivered behavior and contracts must not change. Reconcile and archive the ledger only after feature completion.
