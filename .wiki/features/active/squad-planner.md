# Squad Planner

## Status

Active

## Intent

Let the user model one FM26 tactic and organize the selected club family into Senior, Reserves, and Youth depth charts. Every team uses the same tactic and can have any number of ordered strings. The planner uses the role scores from the current snapshot and preserves planning state when Load Data replaces that snapshot.

## User-visible behavior

- A new `/planner` route and nav item open the Squad Planner.
- First use asks the user to choose a primary club from the active save's current snapshot.
- The primary club contributes Senior, Reserves, and Youth players according to `teamLevel`.
- The user can attach separate B-team or youth clubs to Reserves or Youth. Attached clubs contribute all their players unless the user selects a narrower team-level filter.
- One shared tactic defines 11 linked player lanes across separate In-Possession (IP) and Out-of-Possession (OOP) shapes and roles.
- The tactic starts from an editable 4-3-3 DM IP / 4-1-4-1 DM OOP shape with compatible general-purpose roles and a 50/50 IP/OOP score weight.
- Senior, Reserves, and Youth appear as team tabs. Each tab shows tactic lanes as rows and ordered strings as columns.
- Every team starts with one string. The user can add strings without a product-level maximum and remove any string while at least one remains.
- The user assigns a player through a slot-fit picker. Candidates come from the configured club family and rank by the selected lane's combined IP/OOP role score.
- One player can occupy only one planner cell across all teams and strings. Selecting an assigned player moves that player after explicit confirmation.
- Right-clicking a string header opens its actions. The same actions are available from a visible keyboard-operable header menu.
- Planner configuration and assignments persist per app save across app restarts and snapshot refreshes.

## Invariants

- Exactly one tactic is active per app save for MVP.
- The tactic always contains 11 stable player lanes. Each lane links one IP placement and role to one OOP placement and role.
- Each phase role must belong to that phase and support the lane's selected position.
- Senior, Reserves, and Youth use the same tactic and score weight.
- A player UID is unique across all assignments in one app save.
- Planner rows reference app saves, not snapshots. Load Data must not delete planner state.
- Assignments store the player UID and last-known name. If the UID is absent from the current snapshot, the cell remains occupied and visibly unresolved until the user clears or replaces it.
- Changing club-family sources or tactic roles never silently deletes assignments.
- Removing a populated string requires confirmation and deletes only that string's assignments.
- Rust validates every persisted planner mutation. React never opens SQLite or recomputes role scores.
- Missing role scores remain unknown and render as `—`; they never become zero.

## Non-goals

- Multiple named tactics or tactic libraries per app save.
- Importing a tactic from FM26 or writing a tactic back to the game.
- Team instructions, set pieces, duties, tactical-style presets, or an FM Visualiser clone.
- Automatic inference of B-team relationships from club names.
- Automatic lineup or string selection. The Squad Optimizer owns automated assignment.
- Custom string names, player comparison, transfer workflows, or squad-history tracking.
- Editing the snapshot's `teamLevel`, club, or loan data.

## Current-state map

- **Relevant components:** `AppNavRail` exposes Dashboard, Search, and Planner. The `/planner` route currently owns the no-snapshot state and club-family setup panel. Shared `Panel`, `Modal`, `Button`, `SelectField`, `EmptyState`, and `ScoreBadge` primitives cover most planned UI.
- **Data model:** `players` already stores `current_club`, `parent_club`, `team_level`, positions, and player UID. `player_role_scores` stores every IP/OOP role score for the current snapshot.
- **Persistence and migrations:** SQLite migration v6 is current. Snapshot replacement cascade-deletes players and role scores, while `planner_club_settings`, `planner_club_sources`, the shared tactic, strings, and assignments stay save-scoped without snapshot foreign keys.
- **Existing behavioral assumptions:** one app save is active; all player reads use its current snapshot; Load Data and save switching invalidate snapshot, planner, search, and profile query trees.
- **Architectural seams:** React feature code belongs in `src/features/planner`; Rust persistence and queries belong in `src-tauri/src/features/planner`; the route composes planner and snapshot context without cross-feature imports.
- **Test ownership:** Vitest + RTL own route and interaction behavior; Rust tests own migrations, validation, uniqueness, persistence, and score joins; Playwright smoke owns the browser Planner path with stubbed IPC.
- **Authoritative validation commands:** `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`; targeted Rust checks still run through `./scripts/dev check-rust` when useful during iteration.
- **Likely reuse points:** Planner tactic and club-family Rust services, `plannerKeys`, route-level IPC mocks, `ScoreBadge`, profile tabs, shared `Modal`, and Search keyboard-result patterns.
- **Known technical risks:** club relationships are absent from the dump; club identity is a name string; phase-slot linkage must remain clear during tactic edits; horizontally growing strings must remain usable at 1280x800.
- **Applicable repository patterns:** save-scoped Rust persistence, current-snapshot resolution without snapshot-owned planner rows, TanStack Query for IPC data, route composition across features, and behavioral tests at Rust and React boundaries.

## Feature architecture (this feature)

Planner state is save-scoped and survives replacement of the current snapshot.

- **Club family:** one primary club plus source rows assigned to Senior, Reserves, or Youth. A source stores a club name and an optional `teamLevel` filter. Default primary-club sources use the matching level; an attached B-team or youth club defaults to all levels.
- **Tactic:** one tactic per save with a global IP weight and 11 stable lanes. Each lane stores its IP pitch placement and role plus its OOP pitch placement and role. Pitch placements map to the existing FM position tags used by the scoring catalog.
- **Depth chart:** each team owns ordered string rows. Assignments point to a string and tactic lane, with a save-wide unique player UID and last-known player name. String order produces the displayed 1st, 2nd, and later ordinal labels.
- **Planner read model:** Rust resolves saved assignments against the active save's current snapshot, joins the chosen IP/OOP role scores, calls the existing combined-score helper, and returns bounded tactic, source, string, assignment, and candidate DTOs.
- **Candidate picker:** Rust searches only configured club-family sources, excludes or identifies already assigned players, and sorts for the selected lane. React owns search input, focus, confirmation, and presentation.
- **Cache invalidation:** planner mutations invalidate planner keys. Load Data and active-save changes also invalidate planner reads because current player details and scores may change, while persisted planner configuration remains intact.

## Uncertainty register

### Known

- Dump schema v5 provides `currentClub` as text and `teamLevel` as `senior | reserve | youth | null`.
- Some FM clubs model B teams as separate clubs, so primary-club plus `teamLevel` filtering alone is incomplete.
- The dump has no stable club UID or affiliation relationship.
- FM26 tactics use separate IP and OOP formations and role sets. The same selected player links both phases.
- `combine_role_scores` already supports a caller-supplied IP weight and defaults to 50/50.

### Assumptions

- Club names are stable enough within one app save to persist explicit source mappings.
- One editable tactic per app save meets MVP needs.
- A 4-3-3 DM IP / 4-1-4-1 DM OOP starting shape gives a useful first-run state without restricting later edits.
- The number of user-created strings stays small enough for one save-scoped read without pagination.

### Decisions

- Use one tactic across Senior, Reserves, and Youth.
- Enforce one assignment per player across the whole planner.
- Use explicit club-family sources. Do not infer B-team relationships from names.
- Keep source mappings and assignments when their players or clubs disappear from a later snapshot; show a visible unresolved state.
- Support right-click as a shortcut, with the same string actions exposed through a visible accessible menu.
- Keep tactic creation focused on formations, phase roles, and IP/OOP weight. Defer instructions and automatic selection.

### Unknowns

- Live FM verification may reveal club-name or `teamLevel` edge cases that require source-selection copy or filtering changes, not a new persistence boundary — **blocks:** later validation only.

## Risks

### Configured club disappears

- **Trigger:** A custom database or later snapshot renames or removes a configured club.
- **Consequence:** The source can no longer resolve against the current snapshot.
- **Mitigation:** Keep the mapping and assignments, show a missing-source warning, and let the user replace the source explicitly.
- **Proof:** PR 1 club-family refresh tests plus final live-save validation.

### Assigned player leaves the pool

- **Trigger:** Load Data moves or removes an assigned player from the configured club family.
- **Consequence:** A planner cell could falsely appear empty or silently lose intent.
- **Mitigation:** Preserve UID and last-known name and mark the assignment outside-pool or unresolved.
- **Proof:** PR 2 commit 1 Rust replacement tests and commit 2 rendering tests.

### Invalid role and position pair

- **Trigger:** A tactic edit combines a role with an unsupported phase placement.
- **Consequence:** Candidate scores and assignment fit become invalid.
- **Mitigation:** Filter options in the editor and reject the complete tactic at the Rust mutation boundary.
- **Proof:** PR 1 tactic persistence and editor tests.

### Matrix overflow

- **Trigger:** The user adds enough string columns to exceed the viewport.
- **Consequence:** Lane identity or assignment cells become unreadable or unreachable.
- **Mitigation:** Keep lane labels sticky, use explicit horizontal scrolling, and preserve readable minimum cell width.
- **Proof:** PR 2 commits 2 and 4 tests, smoke path, and final 1280x800 manual validation.

## Walking skeleton

PR 1, commit 1: open Planner, choose Barcelona as the primary club, attach Barça Athletic to Reserves, switch away and back, and see the saved club-family configuration. This proves the new route, save-scoped persistence, snapshot-derived club choices, and React-Rust IPC path before tactic or assignment complexity lands.

## Delivery plan

### PR 1 — Create the club tactic

**Status:** Merged

**Provisional PR title:** `feat(planner): create dual-phase club tactic`

**Purpose:** Establish the save-scoped club family and the shared FM26 dual-phase tactic as an independently useful planner foundation. This PR is a merge boundary because it introduces the persistent model and validates the tactic interaction before squad assignments depend on it.

**Depends on:** Snapshot ingest, role scoring engine, and the current app shell.

#### Commit 1 — Configure club-family sources

**Status:** Completed — `31b091a`

**Work:** Add save-scoped club-family persistence, distinct-club and source-management IPC, the `/planner` shell, nav entry, no-snapshot Load Data guidance, first-use primary-club setup, and editable Reserves/Youth associated-club sources. Seed the primary club's three team-level sources and keep missing mappings visible after refresh.

**Out of scope for this commit:**

- Tactic persistence or editing.
- Strings, player assignments, candidate ranking, or combined scores.
- Automatic club-affiliation inference.

**Validation:** Start with a failing Rust persistence/isolation test and failing route interaction tests that prove an attached separate B club survives reload for one app save without leaking to another, and that `/planner` points to Load Data when the active save has no snapshot. Run affected Vitest and Rust tests, `./scripts/dev check`, and planner route smoke coverage for both first-run states.

**Provisional commit:** `feat(planner): configure club family sources`

#### Commit 2 — Persist the dual-phase tactic

**Status:** Completed — `88925cc`

**Work:** Add the save-scoped tactic and 11 stable lane model, seed the default IP/OOP shapes with compatible general-purpose roles, expose phase-compatible role and placement options from the Rust scoring catalog, validate complete tactics and role-position compatibility, and persist the IP weight.

**Out of scope for this commit:**

- Visual pitch editing.
- Squad strings or player assignments.
- Team instructions, multiple tactics, or tactic import.

**Validation:** RED Rust tests for 11-lane persistence, save isolation, invalid phase/role/position rejection, and weight bounds. Run affected Rust tests and `./scripts/dev check`.

**Provisional commit:** `feat(planner): persist dual-phase tactic`

#### Commit 3 — Add the dual-phase tactic editor

**Status:** Completed — `a6a761c`

**Work:** Add the planned IP, OOP, and side-by-side tactic views with editable pitch placements, phase-filtered role pickers, linked lane identity, IP/OOP weight control, complete loading/error states, and pointer plus keyboard operation. Save through the tactic IPC contract and show validation without losing the draft.

**Out of scope for this commit:**

- Squad strings, assignments, or candidate lists.
- Team instructions or the nine-zone FM Visualiser.
- Multiple tactic presets beyond the seeded starting shape.

**Validation:** RED component tests for linked phase edits, incompatible-role prevention, keyboard editing, weight changes, and failed-save draft retention. Run affected Vitest tests, `./scripts/dev check`, and Playwright smoke for tactic creation.

**Provisional commit:** `feat(planner): add dual-phase tactic editor`

### PR 2 — Plan three-team squad depth

**Status:** Active

**Provisional PR title:** `feat(planner): plan three-team squad depth`

**Purpose:** Add the depth-chart model and user workflow after the tactic boundary has landed and proved stable. This split keeps the migration and interaction risk of tactic creation independently reviewable before assignments depend on lane identity.

**Depends on:** PR 1 merged to trunk.

**Merge boundary:** This PR completes the planner's squad-depth workflow on top of the merged tactic foundation. It is independently mergeable because every commit preserves the existing Planner route and gate while later UI commits consume stable Rust-owned contracts from the first commit.

#### Commit 1 — Persist squad depth assignments

**Status:** Completed — `1fb57c8`

**Work:** Add save-scoped ordered strings and assignments for the three fixed teams. Seed one string per team, enforce save-wide player uniqueness, retain last-known names across snapshot replacement, resolve current snapshot details and combined lane scores in Rust, and support add, remove, clear, assign, and move mutations.

**Out of scope for this commit:**

- Depth-chart matrix UI or player picker.
- Optimized or automatic assignments.

**Validation:** RED Rust tests for default strings, unlimited ordered additions, final-string protection, populated-string deletion, unique player moves, snapshot survival, combined scores, and save isolation. Prove that replacing club-family sources or changing tactic roles preserves assignments and returns the documented outside-pool or unresolved state instead of deleting them. Run affected Rust tests and `./scripts/dev check`.

**Provisional commit:** `feat(planner): persist squad depth assignments`

##### Implementation profile

**Assigned implementer:** Terra xhigh — `gpt-5.6-terra` at `xhigh`.

**Routing summary:** Capability Demand 6 routes to Terra. Effort Demand 6 would normally be medium, but the Terra xhigh hard floor applies because uniqueness and ordering span several mutations and stale assignments must survive snapshot replacement.

##### Review profile

**Assigned reviewer:** Terra xhigh — `gpt-5.6-terra` at `xhigh`, fresh context.

**Mandate:**

- Verify save-wide player uniqueness across assign, move, clear, string removal, and save isolation.
- Verify snapshot replacement preserves player UID and last-known name while truthfully marking unresolved or outside-pool state.
- Verify source or tactic changes never cascade-delete assignments.
- Verify final-string protection and populated-string removal affect only the selected string.
- Verify combined scores use the persisted tactic roles and weight and preserve missing scores as unknown.
- Verify Rust owns mutation validation and that no matrix or picker UI leaks into this commit.

##### Implementation packet

###### Governing requirements and invariants

- Planner rows reference app saves, not snapshots.
- One player UID can occupy only one assignment across all teams and strings in one save.
- Every team has at least one ordered string; populated removal deletes only that string after caller confirmation.
- Assignments retain player UID and last-known name through snapshot replacement.
- Missing current players, outside-pool players, and missing role scores remain explicit states; missing scores never become zero.
- Club-source and tactic mutations do not silently delete assignments.

###### Existing patterns to follow

- `src-tauri/src/db/migrations.rs` owns versioned SQLite migrations and fresh-schema tests.
- `src-tauri/src/features/planner/tactic.rs` shows save-scoped persistence, transactions, validation, DTO-independent domain types, and per-save tests.
- `src-tauri/src/features/planner/service.rs` shows current-snapshot joins, club-source resolution, and snapshot-replacement survival tests.
- `src-tauri/src/features/scoring/combine.rs::combine_role_scores` is the only combined-score implementation.
- `src-tauri/src/features/planner/commands.rs` and `src-tauri/src/lib.rs` own Planner DTO conversion and Tauri registration.
- No exact assignment or ordered-string analogue exists; preserve the existing Rust-owned planner boundary.

###### Expected change surface

- **Likely modified:** `src-tauri/src/db/migrations.rs`, `src-tauri/src/features/planner/mod.rs`, `src-tauri/src/features/planner/commands.rs`, and `src-tauri/src/lib.rs`.
- **Likely added:** one focused Rust planner depth module, such as `src-tauri/src/features/planner/depth.rs`.
- **Ownership boundaries:** SQLite writes, score joining, uniqueness, ordering, and state classification remain in Rust.
- **Do not change without replanning:** React Planner UI, bridge schema v5, role-score formula, tactic lane identity, or snapshot deletion semantics.

###### State and data design

- SQLite is authoritative. Save-scoped strings have fixed team identity and explicit order; assignments reference save, string, tactic lane, player UID, and last-known name.
- The read model resolves current player details and configured-pool membership from the active save's current snapshot without making snapshot rows assignment parents.
- Assign and move use one authoritative transaction so uniqueness cannot be bypassed by mutation order. Failed mutations leave persisted state unchanged.
- Snapshot replacement can change resolution, club-pool state, and scores but not assignment occupancy or last-known identity.

###### Expected interfaces

- Rust domain types represent team, ordered strings, assignment resolution state, and combined lane score.
- Planner commands expose one bounded depth read plus add, remove, clear, assign, and move operations. Exact signatures follow existing command conversion patterns after the domain model is proven.
- The read model returns enough stable identifiers for later matrix and picker commits without exposing raw database rows.

###### Execution order

1. Add RED migration and persistence tests for default strings, ordering, uniqueness, isolation, and snapshot survival.
2. Add the migration and depth domain types.
3. Implement transactional string and assignment mutations with boundary validation.
4. Implement the current-snapshot read model and combined-score join.
5. Add command DTOs and Tauri registration.
6. Add negative tests for final-string removal, invalid references, source/tactic preservation, and missing scores.
7. Run targeted Rust tests, `./scripts/dev format`, and `./scripts/dev check`.

###### Validation ladder

1. Targeted Rust depth and migration tests.
2. Full planner Rust module tests.
3. `./scripts/dev check-rust` when useful during iteration.
4. `./scripts/dev check` as the commit gate.

###### Stop conditions

- Stop if uniqueness cannot be enforced transactionally at the Rust mutation boundary, existing snapshot replacement deletes save-scoped planner rows, tactic lane identity is not stable enough for assignments, or the read model requires a bridge-schema change or UI work from later commits.

###### Allowed discretion

- Private Rust type names, one module versus a small service split, SQL query shape, and test organization inside the existing Planner boundary.

###### Prohibited discretion

- Changing save ownership, snapshot foreign-key behavior, score calculation, player uniqueness, final-string protection, or adding matrix/picker behavior.

##### Escalation conditions

- **Increase effort when:** The save-scoped model and mutation boundary are correct but edge paths, joins, or tests remain incomplete.
- **Increase model capability when:** The implementation cannot explain or enforce uniqueness, snapshot survival, or authoritative Rust ownership.
- **Replan when:** Stable tactic lanes cannot identify assignments, a new persisted/public contract is required outside Planner, or snapshot replacement must change.

##### Execution metadata

```yaml
execution_profile:
  planner: { model: gpt-5.6-sol, effort: high }
  implementer: { model: gpt-5.6-terra, effort: xhigh, confidence: 0.86 }
  capability_demand:
    residual_ambiguity: 1
    architectural_novelty: 1
    diagnostic_uncertainty: 0
    semantic_risk: 2
    context_synthesis: 2
    total: 6
    luna_punch_up_applied: false
    hard_floor: terra-xhigh-stale-state-and-cross-mutation-uniqueness
  effort_demand:
    implementation_breadth: 2
    branch_density: 3
    repository_discovery: 1
    validation_weakness: 0
    tool_coordination: 2
    adjustments: -2
    total: 6
  reviewer: { model: gpt-5.6-terra, effort: xhigh, context_mode: fresh }
  review_demand:
    missed_defect_consequence: 2
    hidden_interaction_complexity: 3
    validation_weakness: 0
    architectural_discretion: 2
    blast_radius: 2
    total: 9
    hard_floor: terra-xhigh-stale-state-and-cross-mutation-uniqueness
  review_mandate:
    - Verify uniqueness across every mutation and save boundary.
    - Verify snapshot replacement preserves truthful unresolved assignments.
    - Verify source and tactic changes cannot cascade-delete assignments.
    - Verify combined scores preserve unknown values.
    - Reject matrix or picker scope leakage.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - The model has the correct persistence boundary but misses mutation paths or tests.
  escalate_model_when:
    - The model misunderstands uniqueness, snapshot survival, or Rust authority.
  replan_when:
    - Stable lane identity or save-scoped persistence must change.
  adjudicator:
    model: gpt-5.6-sol
    effort: medium
    invoke_when:
      - A correction changes persistence ownership or an invariant.
      - Reviewer and implementer disagree about snapshot survival.
```

#### Commit 2 — Add the three-team depth matrix

**Status:** Completed — `6b4e36b`

**Work:** Render Senior, Reserves, and Youth tabs over one shared tactic matrix. Keep tactic lanes sticky, strings horizontally scrollable, cells keyboard reachable, and player identity plus combined score honest for missing, outside-pool, and unresolved assignments.

**Out of scope for this commit:**

- Assigning players from the UI.
- Adding or removing strings from the UI.
- Automatic gap analysis or optimization.

**Validation:** RED route/component tests for team switching, shared tactic rows, string order, horizontal overflow structure, score display, and unresolved/outside-pool states. Run affected Vitest tests and `./scripts/dev check`.

**Provisional commit:** `feat(planner): add three-team depth matrix`

##### Implementation profile

**Assigned implementer:** Luna xhigh — `gpt-5.6-luna` at `xhigh`.

**Routing summary:** Capability Demand 5 normally routes to Terra. The Luna punch-up applies because the architecture and Rust read model are settled, repository patterns are named, the UI change is reversible, and deterministic route tests catch contract failures. Effort Demand 10 reflects the number of display, interaction, and viewport states.

##### Review profile

**Assigned reviewer:** Terra High — `gpt-5.6-terra` at `high`, fresh context.

**Mandate:**

- Verify Senior, Reserves, and Youth render the same stable tactic lanes with team-specific strings.
- Verify missing score, unresolved player, and outside-pool states stay distinct and truthful.
- Verify every matrix cell and team tab is keyboard reachable and exposes equivalent information.
- Verify sticky lane labels and horizontal scrolling preserve readable cells at 1280x800.
- Verify the commit does not add picker mutations, string controls, or client-owned domain calculations.

##### Implementation packet

###### Governing requirements and invariants

- Three fixed team tabs share one tactic and score weight.
- Rows follow stable tactic lane order; columns follow Rust-owned string order.
- Missing role scores render as `—`; unresolved and outside-pool assignments remain occupied and visible.
- React presents the Rust read model and never recomputes combined scores or persistence state.

###### Existing patterns to follow

- `src/app/routes/planner.tsx` composes save, snapshot, club-family, and tactic query state.
- `src/app/routes/planner.test.tsx` owns Planner loading, refresh, error, keyboard, and draft-safety behavior.
- `src/features/player-profile/components/player-profile-tabs.tsx` provides the repository's keyboard tab pattern.
- `src/features/search/components/search-results-panel.tsx` provides sticky table headers and keyboard-focused rows.
- `src/components/ui/score-badge/score-badge.tsx` renders known and unknown scores consistently.
- `src/features/planner/components/planner-tactic-editor.tsx` and `planner-tactic-pitch.tsx` provide tactic lane labels and phase identity.
- No exact depth-matrix analogue exists.

###### Expected change surface

- **Likely modified:** `src/app/routes/planner.tsx`, `src/app/routes/planner.test.tsx`, `src/features/planner/api/planner-keys.ts`, and `src/testing/planner-ipc-mock.ts`.
- **Likely added:** Planner depth types, fetch/query-option modules, and one or more focused matrix components under `src/features/planner/`.
- **Ownership boundaries:** Route composition stays in `src/app/routes`; matrix presentation stays in `src/features/planner`; scores and assignment state come from Rust.
- **Do not change without replanning:** Rust persistence or mutation semantics, tactic editor behavior, candidate queries, string mutation UI, or shared design tokens.

###### State and data design

- TanStack Query owns the save-scoped depth read model. The selected team tab is local presentation state.
- Loading, route refresh, refetch error, empty/default strings, unresolved assignments, outside-pool assignments, and unknown scores each have explicit rendering.
- The matrix has no local authoritative assignment state and no optimistic mutations in this commit.

###### Expected interfaces

- Frontend types mirror the Rust depth DTO without deriving domain state.
- One query option under `plannerKeys` loads all three teams for the active save.
- Matrix components receive tactic lanes, one selected team, ordered strings, and resolved assignment cells as props.

###### Execution order

1. Add RED route/component tests for team tabs, shared lane order, string columns, and truthful assignment states.
2. Add DTO types and the depth query path.
3. Add keyboard-operable team tabs.
4. Render the sticky-lane, horizontally scrollable matrix with `ScoreBadge`.
5. Add loading, error, unresolved, outside-pool, and unknown-score states.
6. Run targeted Planner tests, `./scripts/dev format`, and `./scripts/dev check`.

###### Validation ladder

1. Targeted `src/app/routes/planner.test.tsx` tests.
2. Full Vitest Planner and shared UI tests.
3. `./scripts/dev check-app` when useful during iteration.
4. `./scripts/dev check` as the commit gate.
5. Manual 1280x800 keyboard and overflow check if automated layout evidence remains incomplete.

###### Stop conditions

- Stop if the Rust read model cannot distinguish unresolved from outside-pool state, lane or string identifiers are unstable, truthful rendering requires frontend score calculation, or picker/string mutation work becomes necessary.

###### Allowed discretion

- Component decomposition, local names, semantic table versus grid markup when accessibility remains correct, and test organization.

###### Prohibited discretion

- Adding mutations, recomputing scores, hiding unresolved assignments, changing tactic ownership, or altering persistence/read-model contracts.

##### Escalation conditions

- **Increase effort when:** The approved read model and component boundaries are correct but state coverage, keyboard behavior, or layout verification is incomplete.
- **Increase model capability when:** The implementation reconstructs domain state in React or cannot preserve truthful assignment states.
- **Replan when:** A new Rust or persisted contract is required or matrix viability changes the candidate-picker or PR boundary.

##### Execution metadata

```yaml
execution_profile:
  planner: { model: gpt-5.6-sol, effort: high }
  implementer: { model: gpt-5.6-luna, effort: xhigh, confidence: 0.88 }
  capability_demand:
    residual_ambiguity: 1
    architectural_novelty: 1
    diagnostic_uncertainty: 0
    semantic_risk: 1
    context_synthesis: 2
    total: 5
    luna_punch_up_applied: true
    hard_floor: none
  effort_demand:
    implementation_breadth: 3
    branch_density: 3
    repository_discovery: 2
    validation_weakness: 2
    tool_coordination: 2
    adjustments: -2
    total: 10
  reviewer: { model: gpt-5.6-terra, effort: high, context_mode: fresh }
  review_demand:
    missed_defect_consequence: 1
    hidden_interaction_complexity: 2
    validation_weakness: 1
    architectural_discretion: 1
    blast_radius: 1
    total: 6
    hard_floor: terra-high-ai-discretion
  review_mandate:
    - Verify shared lane identity and team-specific string state.
    - Verify unresolved, outside-pool, and unknown-score rendering.
    - Verify keyboard reachability and target viewport overflow.
    - Reject client-owned domain logic and later-commit scope.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - The UI architecture is correct but state, layout, or keyboard paths are incomplete.
  escalate_model_when:
    - The implementer reconstructs Rust-owned domain state in React.
  replan_when:
    - The matrix requires a new persisted contract or later-commit mutation work.
  adjudicator:
    model: gpt-5.6-sol
    effort: medium
    invoke_when:
      - A correction changes the read model or PR boundary.
```

#### Commit 3 — Assign players by slot fit

**Status:** Completed — `b60e2aa`

**Work:** Add the searchable slot-fit picker backed by the configured club family. Rank candidates by combined score for the selected tactic lane, show IP/OOP evidence and current assignment location, support assignment and confirmed moves, and restore focus to the originating cell.

**Out of scope for this commit:**

- Transfer search outside the configured club family.
- Automated lineup selection or multi-slot optimization.
- Drag-only interaction.

**Validation:** RED Rust query tests and component tests for the target team's source union, separate B-club candidates, the All club family option, score ordering, null-score display, uniqueness, confirmed moves, cancellation, and focus restoration. Run affected Vitest and Rust tests and `./scripts/dev check`.

**Provisional commit:** `feat(planner): assign players by slot fit`

##### Implementation profile

**Assigned implementer:** Terra xhigh — `gpt-5.6-terra` at `xhigh`.

**Routing summary:** Capability Demand 6 routes to Terra. Effort Demand 10 and the Terra xhigh hard floor apply because candidate ranking, asynchronous search, confirmation, persistence, cache reconciliation, and save-wide uniqueness interact.

##### Review profile

**Assigned reviewer:** Sol xhigh — `gpt-5.6-sol` at `xhigh`, fresh context. Review Demand 10 selects Sol, while xhigh effort preserves the Terra xhigh hard floor for persistence, cache, UI reconciliation, and uniqueness across mutations.

**Mandate:**

- Verify candidate scope is the configured club-family union for the target team, including separate attached clubs and optional all-level sources.
- Verify Rust owns filtering, score ordering, assignment-location evidence, and uniqueness.
- Verify assigning an unassigned player and moving an assigned player reconcile the matrix and picker without stale duplicates.
- Verify cancellation and failed mutations preserve the current assignment and restore focus to the originating cell.
- Verify null score evidence remains unknown and ordering does not misrepresent it as zero.
- Verify pointer and keyboard users receive equivalent search, selection, confirmation, and focus behavior.

##### Implementation packet

###### Governing requirements and invariants

- Candidates come only from the selected team's configured club-family sources and rank by the selected tactic lane's combined IP/OOP score.
- A player UID remains unique across every planner cell; moving an assigned player requires explicit confirmation.
- Missing scores remain unknown. Current assignment location and IP/OOP evidence are truthful.
- Rust owns candidate scope, score calculation, and assignment mutations. React owns query text, focus, confirmation, and presentation.

###### Existing patterns to follow

- The active commit's Rust depth module and commands own assignments, moves, resolution state, and combined scores.
- `src-tauri/src/features/planner/service.rs` resolves team club sources against the current snapshot.
- `src/features/search/components/global-player-search.tsx` shows debounced searchable selection, active-option keyboard navigation, and focus handling.
- `src/components/ui/modal/modal.tsx` provides focus trapping, Escape handling, and trigger focus restoration.
- `src/features/planner/components/planner-club-family-panel.tsx` shows Planner mutations and `plannerKeys` invalidation.
- `src/app/routes/planner.test.tsx` and `src/testing/planner-ipc-mock.ts` own route-level interaction evidence.
- No exact confirmed-move picker analogue exists.

###### Expected change surface

- **Likely modified:** Rust Planner depth/commands, `src-tauri/src/lib.rs`, `src/features/planner/api/planner-keys.ts`, Planner matrix components, `src/app/routes/planner.test.tsx`, and `src/testing/planner-ipc-mock.ts`.
- **Likely added:** candidate DTO/query modules, assignment mutation modules, and a focused slot-fit picker component.
- **Ownership boundaries:** Rust selects and ranks candidates and performs moves; React holds transient picker and confirmation state only.
- **Do not change without replanning:** club-source persistence, tactic roles/weight, search feature APIs, string management, or optimizer behavior.

###### State and data design

- Query state is keyed by the active save, team, lane, and normalized search input. Candidate results include current assignment location and nullable phase/combined scores.
- The originating matrix cell owns picker launch context. Cancel and error preserve persisted assignments and restore focus.
- Successful assign or move invalidates or updates the complete Planner depth query so one authoritative read removes stale duplicates.
- Confirmation is local UI state; the Rust move command remains authoritative and transactional.

###### Expected interfaces

- A Rust candidate query accepts team, lane, and bounded search input and returns ranked candidate DTOs with assignment location and score evidence.
- Assignment and move commands use stable string/lane/player identifiers from the depth contract.
- Frontend query and mutation wrappers use `invokeCommand`; the picker consumes typed DTOs and never imports Search feature internals.

###### Execution order

1. Add RED Rust tests for source unions, separate clubs, all-level filters, score order, unknown scores, and assignment-location evidence.
2. Implement the candidate query and command DTOs.
3. Add RED component tests for opening, keyboard search, cancel, assign, confirmed move, failed mutation, and focus restoration.
4. Add typed frontend query/mutation wrappers and the picker UI.
5. Reconcile Planner cache after success and preserve state after failure.
6. Run targeted Rust and Vitest tests, `./scripts/dev format`, and `./scripts/dev check`.

###### Validation ladder

1. Targeted Rust candidate and mutation tests.
2. Targeted Planner route/component tests.
3. Full affected Rust and Vitest suites.
4. `./scripts/dev check` as the commit gate.
5. Keyboard-only manual check when focus behavior cannot be fully observed in jsdom.

###### Stop conditions

- Stop if source resolution cannot express the planned club-family union, candidate ranking needs a new scoring model, moves cannot remain atomic, the matrix contract lacks stable origin identifiers, or the picker requires Search feature coupling or optimizer scope.

###### Allowed discretion

- Search debounce threshold, private component decomposition, internal query result mapping, and test organization within existing contracts.

###### Prohibited discretion

- Client-side ranking or uniqueness, implicit moves without confirmation, cross-feature imports from Search, silent treatment of null scores, or string/optimizer behavior.

##### Escalation conditions

- **Increase effort when:** Ownership is correct but search edges, cache updates, focus paths, or tests are incomplete.
- **Increase model capability when:** The implementation moves ranking or uniqueness into React, cannot explain assignment reconciliation, or patches stale duplicates locally.
- **Replan when:** Candidate scope, scoring, stable identifiers, or transactional move semantics require a new feature boundary or persisted contract.

##### Execution metadata

```yaml
execution_profile:
  planner: { model: gpt-5.6-sol, effort: high }
  implementer: { model: gpt-5.6-terra, effort: xhigh, confidence: 0.84 }
  capability_demand:
    residual_ambiguity: 1
    architectural_novelty: 1
    diagnostic_uncertainty: 0
    semantic_risk: 2
    context_synthesis: 2
    total: 6
    luna_punch_up_applied: false
    hard_floor: terra-xhigh-persistence-cache-ui-reconciliation
  effort_demand:
    implementation_breadth: 3
    branch_density: 3
    repository_discovery: 2
    validation_weakness: 1
    tool_coordination: 3
    adjustments: -2
    total: 10
  reviewer: { model: gpt-5.6-sol, effort: xhigh, context_mode: fresh }
  review_demand:
    missed_defect_consequence: 2
    hidden_interaction_complexity: 3
    validation_weakness: 1
    architectural_discretion: 2
    blast_radius: 2
    total: 10
    hard_floor: terra-xhigh-persistence-cache-ui-reconciliation
  review_mandate:
    - Verify club-family candidate scope and score ordering in Rust.
    - Verify uniqueness and confirmed moves across all cells.
    - Verify cache reconciliation cannot show stale duplicates.
    - Verify cancel, failure, and focus restoration.
    - Verify unknown scores and later-feature exclusions.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - The architecture is correct but async, focus, or cache paths are incomplete.
  escalate_model_when:
    - The implementer misunderstands Rust authority, uniqueness, or reconciliation.
  replan_when:
    - Candidate scope or atomic moves require a new boundary or contract.
  adjudicator:
    model: gpt-5.6-sol
    effort: medium
    invoke_when:
      - A correction changes assignment semantics or feature boundaries.
      - Reviewer and implementer disagree about cache or persistence authority.
```

#### Commit 4 — Manage squad string columns

**Status:** Active

**Work:** Add strings from the matrix header, expose add/remove actions through right-click and the visible header menu, renumber ordinal labels after removal, confirm destructive removal of populated strings, and complete the browser smoke path for all three teams.

**Out of scope for this commit:**

- Custom string names or reorder controls.
- A fixed maximum string count.
- Optimizer controls or gap recommendations.

**Validation:** RED component tests for pointer and keyboard menus, add-after behavior, ordinal renumbering, last-string protection, populated removal confirmation, cancellation, and focus return. Run affected Vitest tests, `./scripts/dev check`, and the complete Planner Playwright smoke flow.

**Provisional commit:** `feat(planner): manage squad string columns`

##### Implementation profile

**Assigned implementer:** Terra High — `gpt-5.6-terra` at `high`.

**Routing summary:** Capability Demand 6 routes to Terra. Effort Demand 8 maps to high. Luna punch-up does not apply because populated-string removal is destructive and ordering plus focus behavior spans persistence, cache, and UI.

##### Review profile

**Assigned reviewer:** Terra High — `gpt-5.6-terra` at `high`, fresh context.

**Routing summary:** Review Demand 8 maps to Terra High under the revised review ladder. Rust already enforces final-string protection and isolated deletion, while targeted component tests and the browser smoke path cover the UI reconciliation contract.

**Mandate:**

- Verify add/remove mutations preserve stable order and renumber only displayed ordinals.
- Verify the last string cannot be removed through pointer, keyboard, context-menu, or direct command paths.
- Verify populated removal requires confirmation, cancellation is lossless, and confirmed removal deletes only that string's assignments.
- Verify right-click is only a shortcut and the visible keyboard menu exposes identical actions.
- Verify mutation errors preserve the matrix and restore focus to the originating header control.
- Verify the final smoke path covers all three teams without adding custom names, reordering, limits, or optimizer controls.

##### Implementation packet

###### Governing requirements and invariants

- Every team retains at least one string.
- Users can add strings without a product maximum and remove populated strings only after explicit confirmation.
- Removing one string deletes only its assignments; remaining strings retain stable order and display fresh ordinal labels.
- Pointer and keyboard users receive equivalent actions and focus restoration.

###### Existing patterns to follow

- The active Rust depth module owns add/remove validation, ordering, and transactional deletion.
- `src/components/ui/modal/modal.tsx` provides confirmation, Escape, and trigger-focus behavior.
- `src/features/planner/components/planner-tactic-editor.tsx` shows roving keyboard selection and explicit status/error messaging.
- `src/features/search/components/search-filter-editor-modal.tsx` shows visible action entry points around modal state.
- `src/app/routes/planner.test.tsx`, `src/testing/planner-ipc-mock.ts`, `e2e/tauri-ipc-stub.ts`, and `e2e/smoke.spec.ts` own Planner interaction and browser contracts.
- No existing context-menu abstraction exists; use native events and existing UI primitives instead of adding a general menu system.

###### Expected change surface

- **Likely modified:** Planner depth command wrappers, matrix/header components, `src/app/routes/planner.test.tsx`, `src/testing/planner-ipc-mock.ts`, `e2e/tauri-ipc-stub.ts`, and `e2e/smoke.spec.ts`.
- **Likely added:** focused add/remove mutation wrappers and a local string-actions menu or confirmation component if existing primitives cannot keep the matrix component clear.
- **Ownership boundaries:** Rust enforces last-string and deletion semantics; React owns menus, confirmation, focus, and ordinal presentation.
- **Do not change without replanning:** database schema, assignment uniqueness, custom naming/reordering, candidate ranking, optimizer behavior, or shared menu abstractions.

###### State and data design

- Persisted string order and assignments remain authoritative in Rust. Display ordinals derive directly from returned order.
- The open-menu string, pending removal, confirmation, and focus target are local UI state.
- Successful mutations refresh the complete Planner depth query. Failed or canceled mutations preserve current matrix data and focus context.

###### Expected interfaces

- Existing add/remove Rust commands are wrapped in typed Planner mutation modules.
- Header controls expose visible actions and an `onContextMenu` shortcut to the same local action state.
- The confirmation contract includes whether the target string is populated; Rust still rejects invalid direct calls.

###### Execution order

1. Add RED component tests for add, ordinal updates, visible keyboard menu, right-click parity, final-string protection, populated confirmation, cancel, failure, and focus return.
2. Add typed mutation wrappers and cache reconciliation.
3. Add accessible header controls and context-menu shortcut.
4. Add populated-removal confirmation and error handling.
5. Extend the IPC mock, browser stub, and Planner smoke path across all teams.
6. Run targeted Vitest tests, `./scripts/dev format`, `./scripts/dev check`, and `./scripts/dev smoke`.

###### Validation ladder

1. Targeted Planner route/component tests.
2. Full Vitest suite for affected frontend behavior.
3. `./scripts/dev check` as the commit gate.
4. `./scripts/dev smoke` for the complete Planner path.
5. Manual keyboard and right-click parity check at 1280x800 if browser automation leaves a focus or layout gap.

###### Stop conditions

- Stop if the Rust boundary does not enforce final-string protection or isolated deletion, accessible parity requires a new shared menu system, stable order requires schema redesign, or custom reordering/naming becomes necessary.

###### Allowed discretion

- Local menu placement, private component split, confirmation copy, and test organization within the existing UI system.

###### Prohibited discretion

- Client-only deletion guards, implicit populated deletion, fixed string limits, custom names or reordering, drag-only controls, or optimizer additions.

##### Escalation conditions

- **Increase effort when:** The mutation contract is correct but menu parity, focus, error, smoke, or layout paths are incomplete.
- **Increase model capability when:** The implementation relies on client-only safety, loses assignment state on failure, or introduces an unnecessary shared abstraction.
- **Replan when:** Ordering or deletion semantics need schema changes, a new cross-feature menu boundary, or behavior assigned to the optimizer.

##### Execution metadata

```yaml
execution_profile:
  planner: { model: gpt-5.6-sol, effort: high }
  implementer: { model: gpt-5.6-terra, effort: high, confidence: 0.87 }
  capability_demand:
    residual_ambiguity: 1
    architectural_novelty: 1
    diagnostic_uncertainty: 0
    semantic_risk: 2
    context_synthesis: 2
    total: 6
    luna_punch_up_applied: false
    hard_floor: terra-high-state-and-persistence
  effort_demand:
    implementation_breadth: 2
    branch_density: 3
    repository_discovery: 1
    validation_weakness: 1
    tool_coordination: 2
    adjustments: -1
    total: 8
  reviewer: { model: gpt-5.6-terra, effort: high, context_mode: fresh }
  review_demand:
    missed_defect_consequence: 2
    hidden_interaction_complexity: 2
    validation_weakness: 1
    architectural_discretion: 1
    blast_radius: 2
    total: 8
    hard_floor: terra-high-state-and-persistence
  review_mandate:
    - Verify isolated deletion, final-string protection, and ordinal updates.
    - Verify populated confirmation and lossless cancel or failure.
    - Verify pointer and keyboard action parity plus focus restoration.
    - Verify final smoke scope and non-goals.
  evidence_threshold:
    require_violated_requirement: true
    require_concrete_execution_path: true
    require_observable_consequence: true
    require_reproduction_or_precise_missing_test: true
    ignore_style_only_findings: true
  escalate_effort_when:
    - The contract is correct but interaction, error, or smoke paths are incomplete.
  escalate_model_when:
    - The implementer relies on client-only safety or misunderstands deletion scope.
  replan_when:
    - Ordering, deletion, or accessible menu behavior needs a new boundary.
  adjudicator:
    model: gpt-5.6-sol
    effort: medium
    invoke_when:
      - A correction changes deletion semantics or the feature plan.
```

## Active work

**PR:** PR 2 — Plan three-team squad depth

**Commit:** Manage squad string columns

### RED test (active commit)

Add string-header controls; verify pointer and keyboard menus, add-after behavior, ordinal renumbering, last-string protection, populated-removal confirmation, cancellation, failure retention, focus return, and the Planner smoke flow across all three teams.

### Expected outcome

The Planner adds and removes ordered squad strings through equivalent visible and context-menu actions, protects the final string in Rust, confirms populated removal, restores focus, and completes the three-team browser smoke path.

### Explicit exclusions

- Do not add custom string names or reorder controls.
- Do not add a fixed maximum string count.
- Do not add optimizer controls or gap recommendations.

### Assigned profiles

- **Implementation:** Terra High — `gpt-5.6-terra` at `high`.
- **Review:** Terra High — `gpt-5.6-terra` at `high`, fresh context.

### Current blockers

- None.

### Discoveries that may require replanning

- None. The active packet's stop conditions cover final-string protection, isolated deletion, accessible action parity, ordering, and persistence boundaries.

## Discoveries and replanning

- **Planned:** Identify the managed club from current snapshot data. **Discovered:** Dump schema v5 has no manager or affiliation identity, and B teams can be separate clubs whose players report `teamLevel = senior`. **Why it matters:** Automatic affiliation would be unreliable. **Change:** Feature architecture changed to an explicit save-scoped club-family mapping. **Affected work:** PR 1 club setup and every later candidate-source query. **Routing impact:** Persistence and candidate commits require at least Terra because the authoritative source boundary spans save state and current snapshots.
- **Planned:** Route reviews with the original model ladder and use Sol Medium for feature completion. **Discovered:** The original ladder promoted routine cross-layer review too quickly and carried implementation effort into review. **Why it matters:** It spent Sol and xhigh usage without a matching increase in consequence or uncertainty. **Change:** Commit 4 review moves from Terra xhigh to Terra High because Rust already owns the destructive invariants and deterministic UI and smoke checks cover the integration; feature-complete review moves to the fixed Sol High profile. **Affected work:** PR 2 commit 4 and feature close-out only; completed review history is unchanged. **Routing impact:** Lower routine commit-review usage and stronger fixed feature-close-out review.

## Completed work

| PR | Commit | Hash | Notes | Implementer | Reviewer | Deviations |
| --- | --- | --- | --- | --- | --- | --- |
| PR 1 | Configure club-family sources | `31b091a` | Added migration v4, save-scoped source persistence and validation, Planner route/setup UI, IPC, cache invalidation, and first-use smoke coverage. | unknown (pre-routing) | unknown (pre-routing) | Explicit club-family mapping replaced unreliable inferred affiliation during planning. |
| PR 1 | Persist the dual-phase tactic | `88925cc` | Added migration v5, save-scoped 11-lane tactic persistence, catalog-backed options, Rust validation, tactic IPC, and route loading/status coverage. | unknown (pre-routing) | unknown (pre-routing) | None recorded. |
| PR 1 | Add the dual-phase tactic editor | `a6a761c` | Added linked IP/OOP/Both views, editable pitch lanes, compatible role filtering, keyboard controls, weight editing, save/error handling, and planner smoke coverage. | unknown (pre-routing) | unknown (pre-routing) | None recorded. |
| PR 2 | Persist squad depth assignments | `1fb57c8` | Added migration v6, save-scoped strings and assignments, snapshot-aware assignment state and scores, transactional depth mutations, and Planner IPC. | Terra xhigh | Terra xhigh | Reindexed strings after deletion to preserve contiguous display order. |
| PR 2 | Add the three-team depth matrix | `6b4e36b` | Added the typed depth query and keyboard-operable Senior, Reserves, and Youth matrix with ordered strings, sticky lane headers, horizontal overflow, and truthful assignment states. | Luna xhigh | Terra High | Added depth-query invalidation after tactic saves so Rust-computed scores refresh with the active tactic. |
| PR 2 | Assign players by slot fit | `b60e2aa` | Added Rust-ranked club-family slot candidates, typed picker mutations, confirmation, and focus-safe matrix reconciliation. | Terra xhigh | Sol xhigh | Review added complete candidate-query invalidation and an occupied-cell clear-first flow; the browser IPC stub gained depth data for the existing Planner smoke path. |

## Final validation

At feature end: `./scripts/dev test`, `./scripts/dev check`, `./scripts/dev smoke`, feature-complete reviewer pass, manual 1280x800 and 1600x900 layout check, keyboard-only tactic and squad workflow, and manual Windows verification against one same-club reserve model plus one separate B-club model when representative saves are available.

### Feature review profile

- **Reviewer:** Sol High — `gpt-5.6-sol` at `high`, fresh context.
- **Mandate:** Verify end-to-end club setup → tactic → three-team depth → candidate assignment → string management; cross-commit identity and cache behavior; snapshot survival; uniqueness; truthful unknown/unresolved state; keyboard equivalence; architecture and documentation consistency; and absence of temporary compatibility paths.

## Documentation impact

- Planning adds the Squad Planner interaction contract to `DESIGN.md`.
- Feature completion must update `ARCHITECTURE.md` with migrations, planner persistence, IPC/read paths, invalidation, and route ownership.
- Feature completion must archive this ledger and advance `TODO.md` to Squad Optimizer.
