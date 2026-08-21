# Planner Best-Role Reference

## Status

Active

## Intent

Give the user a read-only reference that assigns every current managed-club player to the tactic position and role where that player has the strongest individual fit. The reference ignores squad need, depth strings, existing Planner assignments, and competition from other players so it can answer development questions such as which role best matches a youth player's strengths.

## User-visible behavior

- The Planner toolbar has a secondary **Best role fit** action that opens an informational Modal.
- The Modal shows one selectable tactic pitch on the left and the players assigned to the selected tactic slot on the right.
- An **IP / OOP** toggle selects the tactic phase. Scoring uses only the selected phase's position, role, familiarity, and foot rule.
- A separate **Current / Potential** toggle selects the score basis that assigns each player to exactly one tactic slot.
- Every assigned player row shows the player name, adjusted current score, and adjusted potential score for that phase and slot.
- Clicking the Name, Current, or Potential table header sorts the selected slot's rows. Sorting changes row order only; it never reassigns a player.
- A separate **No eligible role** section lists managed-club players who have no eligible tactic slot for the selected phase and assignment basis.
- Switching phase or assignment basis recalculates exclusive assignments. The linked lane selection remains stable when that lane exists in both phase views.
- Empty, loading, and error states remain inside the Modal. Closing the Modal restores focus to **Best role fit**.

## Invariants

- Rust owns player scope, phase eligibility, current and potential score calculation, foot and familiarity adjustments, exclusive lane selection, and deterministic tie-breaking.
- The reference includes each player at the exact managed-club name in the current snapshot exactly once: under one tactic lane or under **No eligible role**.
- IP and OOP scoring are independent. An unsuitable linked position in the other phase cannot disqualify the selected phase.
- A selected-phase position familiarity below 12 is ineligible. Familiarity from 12 through 15 deducts five points. Qualified tactic placements use their base position for familiarity lookup.
- A Preferred foot mismatch deducts five points. A Strict mismatch makes the lane ineligible. Foot and familiarity deductions have a zero floor and do not rewrite stored role scores.
- Current assignment uses the adjusted stored role score. Potential assignment uses the adjusted score calculated from the existing CA-to-PA visible-attribute projection.
- Both displayed score columns use the selected phase and assigned lane's adjustments, independent of which basis selected the lane. A missing score renders as unavailable.
- Equal assignment scores keep the earlier persisted tactic lane. Table sorts use player UID as the final deterministic tie-break.
- The feature is read-only. It cannot change the tactic, Planner strings, Planner assignments, team settings, managed club, snapshots, players, or persisted role scores.
- The result is save- and current-snapshot-scoped and never crosses an active-save change.

## Non-goals

- Optimizing an XI, filling depth strings, balancing squad need, or changing the existing optimizer.
- Assigning a player to more than one best lane for the selected phase and basis.
- Training recommendations, training-plan mutations, comparison charts, search, filters, export, or player-profile links.
- New role formulas, potential-projection formulas, familiarity thresholds, foot penalties, or configurable tie-break rules.
- Persisting the selected phase, assignment basis, selected lane, sort, or Modal state after close.
- Mobile, web-client, or narrow-window layouts.

## Current-state map

- Relevant components: `src/features/planner/components/planner-depth-matrix.tsx` owns the Planner toolbar and Modal trigger state; `planner-tactic-pitch.tsx` renders the selectable phase pitch; `src/components/ui/modal/modal.tsx` owns focus trapping, dismissal, animation, and focus restoration; `src/components/ui/score-badge/score-badge.tsx` renders accessible score values.
- Data model: `PlannerTactic.lanes` links one IP and OOP position and role through a stable lane ID. `players` stores current club, preferred foot, positions, attributes, CA, and PA. `player_role_scores` stores current role scores by snapshot and player UID.
- Persistence and migrations: tactic rows are save-owned; players and role scores are snapshot-owned. This feature needs no schema change, migration, or persisted preference.
- Existing behavioral assumptions: the optimizer requires both linked phases, combines IP/OOP scores, applies one foot penalty, and applies one familiarity penalty per phase. This reference must reuse the same thresholds and penalty values while evaluating only one selected phase.
- Architectural seams: React calls typed adapters in `src/features/planner/api/`; Tauri commands in `src-tauri/src/features/planner/commands.rs` resolve the active save; Rust services query SQLite and return bounded presentation models; React Query caches results under `plannerKeys`.
- Project validation commands: `./scripts/dev format`, `./scripts/dev test [target...]`, `./scripts/dev check`, and `./scripts/dev smoke`.
- Primary risks: scoring drift from the optimizer, accidental dual-phase eligibility, ambiguous equal-score lanes, stale data after save or tactic changes, and an unreadable two-column Modal at the 1280×800 minimum window.

## Feature architecture

The feature stays inside the existing Planner boundary and adds no persistence. A small Planner-private fit module owns the reusable position suitability, familiarity penalty, and preferred-foot rules. The existing optimizer composes those rules across both phases without changing behavior. The new reference service applies the same rules to one phase at a time.

The frontend requests one phase and one assignment basis. Rust loads the active save's current tactic, current snapshot, exact managed-club cohort, current role scores, and potential-score inputs. For each player it calculates adjusted current and potential scores for every tactic lane in the selected phase, chooses the lane with the highest non-null selected-basis score, and keeps tactic order on ties. The response groups players by stable lane ID and returns players without a selected-basis lane in a separate collection. Each assigned row carries both adjusted scores for its chosen lane.

React Query keys the read by active save ID, phase, and assignment basis. The Modal owns only open state, selected lane, the two toggles, and bounded table sort state. It uses the existing tactic and role-option data to label the pitch and selected role. Local sorting is permitted because Rust has already reduced the current managed-club cohort to one bounded row per player and all domain decisions are complete.

## Uncertainty register

### Known

- The current app is on `main` at `bd3b47137ac7c2bfbbcc723dc58b6ca5e47d96a7`, and the worktree was clean during planning.
- `/my-club?view=planner` renders `PlannerDepthMatrix`; `/planner` is a compatibility redirect.
- The current Planner cohort is the current snapshot's exact managed-club name. Team categories and age rules belong to depth allocation and do not define this reference cohort.
- The tactic pitch already supports phase-specific positions, role labels, selected lanes, qualified placements, and keyboard-operable lane buttons.
- The shared Modal supports a 720px two-column variant through its existing `className` override and restores focus to its trigger.
- Repowise is unavailable in this checkout because the CLI is not installed. Direct repository files, tests, configuration, and Git are authoritative.

### Assumptions

- The current managed-club cohort is small enough to return as one read model. The product contract requires every player, so the first implementation does not truncate or paginate the result.
- The Modal defaults to IP, Current, the first available tactic lane, and Current descending sort.
- Changing the assignment basis resets the table sort to that basis descending. Changing only the phase keeps the selected sort when the column still exists.
- **No eligible role** uses the same Name, Current, and Potential columns, with unavailable score cells because no lane was selected for those players under the active basis.

### Decisions

- Use one exclusive best lane per player rather than ranking every eligible player under every tactic slot.
- Let the user switch the assignment basis between Current and Potential. Both score columns remain visible in either mode.
- Evaluate IP and OOP independently.
- Keep equal-score assignment deterministic through persisted tactic order rather than adding balancing logic that would reintroduce squad need.
- Show **No eligible role** as a separate right-panel section rather than silently omitting players.
- Keep sorting Modal-local and query-silent. Sorting is presentation, while assignment and scoring remain Rust-owned.
- Deliver through one short-lived branch and one PR because the refactor, read model, and Modal form one review surface and introduce no migration or risky foundation that needs a separate trunk merge.

### Unknowns

- The exact populated Modal density and scroll behavior at 1280×800 and 1600×900 remain unverified until implementation can run in a browser and, when available, the native Tauri/WebView.
- Representative full-club calculation timing is not measured. The design must remain simple until evidence shows a need for persistence, batching, or pagination.

### Risks

- A shared fit extraction can subtly change optimizer deductions, especially when IP and OOP use the same base position. Characterization tests must lock the existing behavior before extraction.
- A player can move between lanes when the basis changes. The UI must label the active basis and must not imply that table sorting caused the move.
- Potential scoring can be unavailable when required projected attributes are missing. The selected basis then cannot assign that player even when the other basis has a score.
- Identical tactic lanes can tie for many players. Earlier tactic order is deterministic but can leave a later identical lane empty; the feature must not balance ties because that would account for need.
- The Modal can contain two independent toggle groups, an interactive pitch, a sortable table, and an ineligible section. Keyboard order, focus visibility, headings, and internal scrolling require explicit tests.

## Walking skeleton

Extract the existing Planner fit rules without behavior change, then prove one Rust current/IP request that assigns every managed-club player to one lane or **No eligible role**, and finally expose that request through **Best role fit** with one selected pitch lane. The same command parameters and row model extend that path to OOP and Potential without another architecture.

## Delivery plan

### PR 1 — Add Planner best-role reference

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/planner-best-role-reference`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict `check`

**Feature close-out:** Not run; correction round 1 active after feature review

**CI repair rounds:** `0`

**Provisional PR title:** `feat(planner): add best-role reference`

**Purpose:** Add one read-only Planner reference that reuses the established fit rules, assigns every current managed-club player to one best phase-specific tactic lane by the selected score basis, and presents the result in an accessible sortable Modal.

**Depends on:** Current `main`, the implemented dual-phase tactic, current and potential role scoring, complete position familiarity, preferred-foot optimizer rules, exact managed-club scope, and the shared Modal and ScoreBadge primitives.

**Merge to trunk when:** all three commits are complete; focused, full, and browser validation pass; the feature-complete review and documentation reconciliation clear; the template-complete PR passes the required `check`; and no unresolved native limitation blocks acceptance.

#### Commit 1 — Share phase fit scoring rules

**Status:** Completed

**Provisional commit:** `refactor(planner): share tactic fit scoring rules`

**Work:** Extract the optimizer's position suitability, preferred-foot, and familiarity adjustments into one Planner-private module that supports both the existing linked-lane calculation and the new single-phase calculation without changing optimizer behavior.

**Out of scope:**

- The best-role read model, IPC command, Modal, query adapter, or any user-visible behavior.
- Changes to suitability thresholds, penalty values, score combination, team age rules, optimizer matching, transaction behavior, or persisted role scores.

**Implementation packet:**

- Create a narrow Planner-private fit module. It must express a single-phase fit score and the existing linked-lane allocation score from the same suitability, foot, and familiarity primitives.
- Preserve the optimizer's exact semantics: both linked positions require familiarity at least 12; a strict foot mismatch is ineligible; a preferred mismatch deducts five once; every phase below familiarity 16 deducts five, including two deductions when both phases use the same sub-16 base position; deductions saturate at zero.
- Replace the optimizer-local helpers only after characterization tests prove those behaviors. Keep the matcher, candidate loading, score combination, and persistence untouched.

**Files and responsibilities:**

- `src-tauri/src/features/planner/fit.rs` — Planner-private suitability and adjusted-fit functions for one phase and a linked lane.
- `src-tauri/src/features/planner/fit_tests.rs` — direct characterization of thresholds, qualified-placement normalization, preferred and strict foot rules, zero floor, and linked-lane penalty composition.
- `src-tauri/src/features/planner/optimizer.rs` — call the shared linked-lane function while preserving candidate and matching behavior.
- `src-tauri/src/features/planner/mod.rs` — register the private fit module and its test module.
- `src-tauri/src/features/planner/optimizer_tests.rs` — retain or tighten observable optimizer assertions only where needed to prove no behavior changed.

**Behavior and data flow:**

- Existing optimizer role scores continue through `combine_role_scores`, then through the shared linked-lane fit function, then into the unchanged exact matcher and assignment transaction.
- The new single-phase helper remains unused outside tests in this commit. It accepts one role score and one selected phase position plus the lane foot rule, and returns an adjusted score or ineligibility.
- No SQL, DTO, IPC, React Query, or UI path changes.

**Ordered implementation steps:**

1. Add RED characterization tests for the shared function contract and confirm failure because the fit module does not exist.
2. Extract the smallest shared fit primitives and turn the focused Rust proof GREEN.
3. Replace optimizer-local calls with the shared linked-lane function and run the existing optimizer suite through the project gate.
4. Refactor only while all characterization and optimizer proofs remain green.
5. Run formatting and the commit gate.

**Tests and proof:**

- A selected phase accepts familiarity 12, deducts five at 12 through 15, and has no familiarity penalty at 16.
- Qualified placements such as MCR and MCL read MC familiarity.
- Preferred foot mismatch deducts five once; strict mismatch returns no score; matching and two-footed values follow the current contract.
- Linked IP/OOP scoring still rejects either unsuitable phase and counts both familiarity penalties, including repeated base positions.
- Low scores saturate at zero.
- Existing optimizer tests still produce the same candidates and assignments.

**Patterns to verify:**

- `optimizer.rs::allocation_score`, `foot_matches`, and `is_suitable_for_lane` are the behavior being extracted.
- `tactic.rs::base_position` remains the only qualified-placement normalization rule.
- `optimizer_tests.rs` is the observable regression seam for exact allocation.

**Constraints and non-goals:**

- Keep the module private to `features::planner`; do not create a global scoring abstraction.
- Do not change public types, serialization, SQL, migrations, optimizer score bases, or test fixtures beyond what the extraction needs.
- Do not simplify the existing double familiarity deduction for identical phase positions; preserving behavior is the contract.

**Dependencies and sequencing:** None beyond current `main`. Commit 2 depends on the single-phase fit function from this commit.

**Validation:**

- `./scripts/dev format`
- `./scripts/dev check`

**Stop conditions:** Stop and replan if characterization shows the current optimizer behavior differs from the completed feature contracts, if qualified placements use a second normalization path, or if sharing the rule requires changing optimizer output or persistence.

**Review mandate:**

- Confirm exact optimizer behavior is preserved for suitability, foot, familiarity, and zero-floor cases.
- Confirm identical IP/OOP base positions retain two familiarity deductions below 16.
- Confirm the extraction does not change matching order, team eligibility, score combination, transaction scope, or visibility.
- Confirm the module remains Planner-private and introduces no speculative configuration.

#### Commit 2 — Rank players by their best tactic role

**Status:** Completed

**Provisional commit:** `feat(planner): rank players by best tactic role`

**Work:** Add the Rust read model and Tauri command that assign every current managed-club player to one best tactic lane for a requested phase and Current or Potential basis, while returning both adjusted scores and a separate no-eligible collection.

**Out of scope:**

- React components, query adapters, Planner toolbar changes, browser tests, or documentation that claims the Modal is implemented.
- Persistence, migrations, tactic edits, Planner assignments, optimizer mutations, team age rules, or multiple assignments per player.

**Implementation packet:**

- Add closed phase and assignment-basis enums at the Rust boundary. Reject unknown values before querying.
- Resolve the active save, current snapshot, exact managed-club name, and persisted tactic through existing Planner and snapshot seams.
- Load each exact managed-club player once with UID, name, preferred foot, position familiarity, attributes, CA, and PA. Load the selected phase's current role scores for the tactic roles without exposing raw rows through IPC.
- Calculate projected attributes once per player, then calculate current and potential role scores for every selected-phase tactic lane. Pass both through the single-phase fit rule.
- Select the greatest non-null adjusted score from the requested basis. Preserve persisted tactic order on equal scores. Put a player with no selected-basis score under **No eligible role** even when the other basis has a usable score.
- Return lane groups in tactic order. Return each assigned player with UID, name, adjusted current score, and adjusted potential score for the chosen lane. Return no-eligible players once with unavailable lane scores. Use name and UID only for deterministic baseline ordering; React owns requested table sorting.

**Files and responsibilities:**

- `src-tauri/src/features/planner/role_reference.rs` — phase and basis parsing, exact-club query, projection and role scoring, exclusive lane selection, deterministic grouping, and response-domain types.
- `src-tauri/src/features/planner/role_reference_tests.rs` — focused SQLite-backed service tests using current Planner fixtures.
- `src-tauri/src/features/planner/fit.rs` — consume the single-phase function established in commit 1; change it only if a test exposes a missing part of the approved contract.
- `src-tauri/src/features/planner/commands.rs` — typed serializable DTOs and `get_planner_role_reference` command mapping.
- `src-tauri/src/features/planner/mod.rs` — register the service and test module.
- `src-tauri/src/features/planner/test_support.rs` — add only small fixture setters needed by several new service tests.
- `src-tauri/src/lib.rs` — register the new Tauri command.

**Behavior and data flow:**

- React will send `{ phase, scoreBasis }` through one closed command.
- The command resolves the active save and delegates to the service.
- The service reads the current snapshot, exact managed-club cohort, tactic, current role scores, and potential inputs from SQLite; computes all phase-lane fits in memory; assigns each player once; and returns tactic-ordered lane groups plus no-eligible players.
- The command serializes camelCase DTOs with no database IDs other than player UID and no persistence side effects.
- An absent current snapshot or managed club returns the existing safe user-facing error path. An empty exact-club cohort returns empty groups and an empty no-eligible collection.

**Ordered implementation steps:**

1. Add a RED service test for exclusive current/IP assignment and one no-eligible player; confirm failure because the service is absent.
2. Implement the smallest current/IP service path and make the proof GREEN.
3. Add RED tests for OOP independence, Potential assignment, both returned score columns, tie order, penalties, missing selected-basis scores, exact-club scope, and empty state.
4. Extend the same service path without creating a second scorer or query architecture.
5. Add DTO mapping and command registration after the service contract is green.
6. Run formatting and the commit gate.

**Tests and proof:**

- Every exact managed-club player appears exactly once in one lane group or no-eligible output.
- A player can choose different lanes under IP versus OOP and Current versus Potential.
- OOP unfamiliarity cannot disqualify an IP assignment, and IP unfamiliarity cannot disqualify an OOP assignment.
- Current and potential adjusted values in a row come from the assigned phase and lane, not each column's independently best lane.
- Familiarity 11 is ineligible, 12 through 15 deduct five, 16 does not deduct, and sided placements normalize to the base position.
- Preferred and strict foot rules match the existing Planner contract.
- Equal selected-basis scores keep earlier tactic order; baseline row ordering resolves by case-insensitive name and UID.
- A missing selected-basis score sends the player to no-eligible even when the other displayed basis has a score.
- Other-club and non-current-snapshot players never appear. Team age limits and existing Planner assignments do not affect the cohort.
- Unknown phase or basis input fails safely at the command boundary; no read mutates SQLite.

**Patterns to verify:**

- `depth.rs::current_snapshot_id` and the read-only `tactic.rs::load_tactic` seam for snapshot and tactic reads.
- `optimizer.rs::load_current_optimizer_candidates` and `load_potential_optimizer_candidates` for current score loading and `project_attributes` plus `score_role` use. Copy the scoring contract, not the combined-score or age-filter behavior.
- `commands.rs` DTO conversions and `lib.rs` registration for stable IPC naming and camelCase serialization.
- `test_support.rs::open_with_snapshot` and Planner service tests for temporary SQLite fixtures.

**Constraints and non-goals:**

- Do not call or mutate the optimizer. Do not read Planner strings or assignments.
- Do not apply IP/OOP weights, combined scores, importance ranks, team order, string order, team availability, or age limits.
- Do not persist projected scores or introduce a response cap that omits managed-club players.
- Do not expose attributes, positions, preferred foot, raw database rows, or internal errors to React.

**Dependencies and sequencing:** Commit 1 must be complete so phase fit uses the shared Planner contract.

**Validation:**

- `./scripts/dev format`
- `./scripts/dev check`

**Stop conditions:** Stop and replan if the existing projection cannot produce both displayed scores without new persistence, if exact managed-club scope cannot include every player deterministically, if the service needs Planner assignment state, or if representative data shows the one-response cohort is too large for a responsive command.

**Review mandate:**

- Confirm phase independence and basis-selected exclusive assignment.
- Confirm current and potential displayed scores use the chosen lane's adjustments.
- Confirm exact managed-club/current-snapshot scope and one-row-per-player completeness.
- Confirm tie-breaking is tactic-order deterministic and contains no balancing or squad-need logic.
- Confirm no persistence, assignment, optimizer, age-rule, or combined-score behavior leaked into the read.
- Confirm invalid boundary values and missing data fail safely without exposing internals.

#### Commit 3 — Show the best-role reference Modal

**Status:** Active

**Provisional commit:** `feat(planner): show best-role reference`

**Work:** Add the typed React Query adapter, Planner toolbar action, two-toggle read-only Modal, selectable tactic pitch, sortable score table, no-eligible section, route regressions, browser smoke, and implemented design documentation.

**Out of scope:**

- Player-profile navigation, training actions, export, search, filters, persistence of Modal state, changes to the Tactic workspace, or changes to optimizer controls.
- Client-side scoring, eligibility, lane assignment, potential projection, or rebalancing after a table sort.

**Implementation packet:**

- Model closed phase, assignment basis, lane-group, assigned-player, and no-eligible response types in the Planner feature.
- Add one invoke adapter and query-options builder keyed by active save ID, phase, and basis. Enable it only while the Modal is open. Tactic saves and active-save changes already invalidate `plannerKeys.all`; confirm the new key participates.
- Add a secondary **Best role fit** button to the existing Planner action toolbar without competing with **Optimize squads**.
- Build one informational 720px Modal. Put accessible IP/OOP and Current/Potential segmented controls above a two-column layout. Reuse `PlannerTacticPitch` as the left-side lane selector for the active phase. Keep the selected stable lane when toggles change.
- On the right, identify the selected position and role, then render a semantic compact table with Name, Current, and Potential headers. Header clicks sort the selected lane's bounded rows with visible and `aria-sort` direction. Basis changes reset sorting to that basis descending; later header clicks do not refetch or reassign.
- Render scores with the table ScoreBadge and unavailable values as an em dash. Under a hairline-separated heading, render **No eligible role** only when non-empty, using the same columns and unavailable score cells.
- Keep loading, empty, request error, no players for the selected role, and no-eligible states explicit. Do not show stale results from the previous phase or basis as if current.
- Restore focus on close, keep every toggle and pitch slot keyboard-operable, trap focus through internal scroll regions, and preserve visible focus.

**Files and responsibilities:**

- `src/features/planner/types/role-reference.ts` — closed frontend response and interaction types.
- `src/features/planner/api/fetch-planner-role-reference.ts` — typed Tauri adapter.
- `src/features/planner/api/planner-role-reference-query-options.ts` — save/phase/basis query options and open-state enablement.
- `src/features/planner/api/planner-keys.ts` — role-reference cache root and parameterized key.
- `src/features/planner/components/planner-role-reference-modal.tsx` — toggles, pitch selection, loading/error/empty states, sortable semantic tables, and Modal composition.
- `src/features/planner/components/planner-depth-matrix.tsx` — toolbar trigger, open state, active-save context, and Modal mounting.
- `src/testing/planner-ipc-mock.ts` — deterministic role-reference fixture, request capture, pending/error controls, and reset behavior.
- `src/app/routes/my-club-squad.test.tsx` — route-level user-observable regressions.
- `e2e/smoke.spec.ts` — one browser-stubbed Planner reference workflow at the product edge.
- `.wiki/DESIGN.md` — record the implemented Planner reference interaction and Modal layout after the behavior exists.

**Behavior and data flow:**

- **Best role fit** opens with IP, Current, the first tactic lane, and Current descending sort.
- The open Modal requests one Rust grouping for active save, phase, and basis. Toggle changes select another keyed query and keep the lane ID when valid.
- The selected pitch lane chooses one already-computed Rust lane group. Local sort copies that group's bounded rows and orders only their presentation.
- Closing clears transient Modal interaction state and restores trigger focus. Reopening starts from defaults and may reuse matching current React Query data.
- Active-save replacement remounts `PlannerDepthMatrix`, closes stale local state, and ensures a new save-keyed request.

**Ordered implementation steps:**

1. Extend the IPC mock and add a RED route test that expects **Best role fit** to open an accessible Modal with the current/IP request.
2. Add the typed adapter, query key, and minimal Modal trigger and make the walking-skeleton test GREEN.
3. Add RED tests for lane selection, OOP and Potential requests, basis-driven reassignment, both score columns, header sorting without refetch, no-eligible rendering, error and empty states, Escape, and focus restoration.
4. Implement the smallest complete two-column Modal with existing primitives and make the focused route suite GREEN.
5. Add the browser smoke path for opening, toggling, selecting, sorting, and closing the reference.
6. Update `.wiki/DESIGN.md` only after the implemented behavior matches the plan.
7. Run formatting, focused tests, the full gate, and browser smoke.

**Tests and proof:**

- The button appears in the Planner toolbar and does not appear as a new workspace or route.
- Opening sends `in_possession` plus `current`; toggles send the exact OOP or Potential values and update labelled active states.
- Switching phase or basis keeps a valid selected lane, but changing basis resets sort to the corresponding score descending.
- Clicking Name, Current, or Potential toggles sorting, updates `aria-sort`, orders deterministically, and does not call IPC again or move players to another lane.
- The right table shows only the selected Rust lane group with player name and both adjusted scores.
- **No eligible role** is separate, includes each returned player once, and renders unavailable scores truthfully.
- Loading, selected-lane empty, whole-cohort empty, and error paths remain inside the Modal.
- Escape, close button, and backdrop dismissal close the informational Modal and restore focus to **Best role fit**.
- A save change cannot display or restore the prior save's grouping.
- Browser smoke proves the composed route and IPC-stub path; native Tauri/SQLite calculation remains distinct Rust and manual evidence.

**Patterns to verify:**

- `PlannerTacticEditor` for accessible segmented-control keyboard behavior and phase labels; do not extract a shared control unless two current callers genuinely benefit.
- `PlannerTacticPitch` for phase layout, stable lane selection, role labels, and linked placement handling.
- `PlannerDepthMatrix` for toolbar ordering, action feedback boundaries, and save-keyed local state.
- `PlannerSlotFitPicker` and `PlannerTeamManagement` for 720px Modal composition, internal scrolling, error ownership, and focus restoration.
- `PlayerRolesPanel` for sortable Current/Potential headers and ScoreBadge accessibility.
- Existing My Club route tests and `planner-ipc-mock.ts` for provider setup and IPC-state reset.

**Constraints and non-goals:**

- Use the shared Modal, Button, ScoreBadge, tactic pitch, tokens, and Lucide system. Add no dependency or custom overlay primitive.
- Keep the feature desktop-only at the documented 1280×800 minimum. Use one internal Modal scroll owner and avoid nested panels or decorative cards.
- Keep role assignment immutable in React. Do not regroup rows on header sort.
- Do not add URL state, Zustand state, persisted settings, or a new route.
- Show only the requested name and two scores in player rows.

**Dependencies and sequencing:** Commit 2's registered command and stable DTO contract must be complete. No external service or migration is required.

**Validation:**

- `./scripts/dev format`
- `./scripts/dev test src/app/routes/my-club-squad.test.tsx`
- `./scripts/dev test`
- `./scripts/dev check`
- `./scripts/dev smoke`

**Stop conditions:** Stop and replan if the existing pitch cannot serve as a read-only selector without changing Tactic editing behavior, if one internal scroll owner cannot fit the Modal at 1280×800, if sorting would require client-side domain reassignment, or if save/tactic invalidation cannot prevent stale phase or basis results.

**Review mandate:**

- Confirm React displays Rust assignments without recomputing fit or regrouping on sort.
- Confirm both toggles have correct accessible state, keyboard behavior, query keys, and stale-data handling.
- Confirm semantic header sorting, ScoreBadge accessible names, empty/error states, focus trapping, Escape/backdrop close, and trigger focus restoration.
- Confirm one internal scroll owner and readable two-column density at the documented desktop sizes.
- Confirm the toolbar hierarchy keeps Optimize primary and the reference secondary.
- Confirm route, cache, and active-save boundaries prevent cross-save results.
- Confirm `.wiki/DESIGN.md` describes only implemented behavior and no architecture document change is needed.

## Active work

**PR:** PR 1 — Add Planner best-role reference (correction round 1; Active)

**Commit:** Commit 3 — Show the best-role reference Modal (correction round 1; Active)

### RED proof

Add a RED route test that expects **Best role fit** to open an accessible Modal and request the current/IP grouping. The first run must fail because the typed adapter, query key, toolbar trigger, and Modal path do not exist. A plausible wrong implementation must fail when it refetches for sorting, loses the selected lane across toggles, shows stale phase/basis data, or omits the no-eligible section.

### Expected outcome

The Planner opens a read-only, two-column best-role reference Modal with IP/OOP and Current/Potential controls, a selectable tactic pitch, sortable score rows, and explicit loading, empty, error, and no-eligible states.

### Explicit exclusions

- No player-profile navigation, training actions, export, search, filters, URL state, persisted Modal state, or changes to the Tactic workspace or optimizer controls.
- No client-side scoring, eligibility, lane assignment, regrouping after sorting, or new overlay/dependency primitive.

## Discoveries and replanning

- Planning confirmed that the initial use case is role-training guidance, but the reference must also support broader current and future evaluation. Current/Potential therefore selects the assignment basis while both adjusted score columns remain visible.
- Planning confirmed that IP and OOP are independent and that players without an eligible lane need a separate visible section.
- Planning found that the current optimizer's hidden familiarity rule counts one five-point deduction per phase, even when both phase positions normalize to the same base position. Commit 1 must preserve that linked-lane behavior while the reference applies only one selected-phase deduction.
- Commit 2 uses the Planner-private read-only `load_tactic` seam so opening the reference cannot seed or edit a missing tactic; `get_tactic` remains the existing initialization path for Planner screens.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Planning | Pending record | Active ledger and TODO activation | Planning-only verification | None |
| PR 1 | Commit 1 — Share phase fit scoring rules | Pending record | Planner-private phase and linked-lane fit helpers; optimizer delegates to the shared linked-lane rule; focused characterization coverage | Sol Medium: clean after one correction round | None |
| PR 1 | Commit 2 — Rank players by their best tactic role | Pending record | Read-only Rust role-reference service, phase/basis parsing, exact current managed-club scope, projected current/potential adjusted scores, deterministic lane grouping, DTO, command registration, and SQLite-backed coverage | Sol Medium: clean after one correction round | Read-only `load_tactic` visibility seam avoids `get_tactic` default seeding; no persistence side effects |
| PR 1 | Commit 3 — Show the best-role reference Modal | Pending record | Typed React Query adapter, save/phase/basis cache key, Planner toolbar action, read-only two-column Modal with selectable tactic pitch, sortable score tables, no-eligible and explicit state handling, route regressions, browser smoke, and implemented design note | Sol Medium: clean after two correction rounds | One internal Modal scroll owner; native Tauri/WebView evidence remains outside browser smoke |

## Final validation

- `./scripts/dev format`
- `./scripts/dev test`
- `./scripts/dev check`
- `./scripts/dev smoke`
- `git diff --check <recorded-feature-base>...<recorded-feature-head>`
- Manual browser inspection at 1280×800 and 1600×900 with a populated exact-club cohort: verify two-column density, one internal Modal scroll owner, score readability, lane selection, both toggles, all three sorts, no-eligible visibility, and focus restoration.
- Native Tauri/WebView inspection at the same sizes when the supported Windows environment is available. If unavailable, record it as an evidence gap rather than a pass.
- Feature-complete review over the exact recorded implementation refs, followed by documentation reconciliation and archive through the feature workflow.
- `./scripts/dev mutate` remains unsupported and must not be reported as passed. `./scripts/dev bridge-test` is not required because the bridge contract does not change.

## Documentation impact

Planning creates this active ledger and activates the feature in `.wiki/TODO.md`. Commit 3 updates `.wiki/DESIGN.md` when the interaction is implemented. `.wiki/ARCHITECTURE.md` should need no change because the feature stays within the implemented React → Tauri → Rust → SQLite Planner boundary and adds no persistence. Reassess that conclusion during feature reconciliation.
