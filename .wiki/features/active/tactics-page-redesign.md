# Tactics Page Redesign

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** bd0767c438e788e0db1560643e56bfa8773de559f7914bd35a0a46b4db9ae84c

Provisional: recorded for structural consistency only — it confers no delivery authority until a fresh complete plan review passes and the developer accepts it.

The previously accepted fingerprint `0404c69ef4ddf846a4a20f646fed2bf223381c4138d21c0eaee32bbc5794016d` was invalidated by this material replanning (unique-placement swap contract plus one-time tactic reset, replacing draft-overflow support). The fingerprint recorded here confers no delivery authority until a fresh complete plan review passes and the developer accepts it.

## Intent

Replace the Tactic workspace's two side-by-side IP/OOP pitch boards and bottom settings shelf with one phase-aware tactical pitch, a persistent Tactical XI panel, and a persistent Selected Slot inspector beside the pitch (Linear JAY-57). The workspace keeps the current one-tactic-per-save persistence model, every existing setting and edit callback, and the supported viewport contracts, and it adds a real 90-degree landscape pitch orientation at viewport widths >= 1920px. Placement editing follows a unique-placement swap contract (no duplicate or overflow draft states), and existing tactics are discarded once via a one-time reset to the normal defaults. This is redesign-only.

## User-visible behavior

- One tactical pitch replaces the two side-by-side boards. IP mode shows all 11 slots at their IP positions and roles. OOP mode shows all 11 slots at their OOP positions and roles. Both mode shows both markers for each same underlying slot, clearly distinguished, with a connector only where the slot's canonical full qualified placement identity changes (DCR → DCL gets a connector; legacy-equivalent ST → STC does not) and a readable IP role → OOP role transition.
- A persistent Tactical XI panel lists all 11 existing lanes with their IP → OOP role transition. Selection from the list or from the pitch owns the same `selectedLaneId`.
- A persistent Selected Slot inspector sits beside the pitch. It reuses every existing setting and edit callback: importance rank, preferred foot, foot preference, IP/OOP weight, IP position/role, and OOP position/role. Position pickers offer exact qualified placements (for example AMCR/AMC/AMCL, DCR/DC/DCL, DMCR/DM/DMCL, MCR/MC/MCL, STCR/STC/STCL). Selecting an already occupied placement swaps the two lanes' placements in the edited phase only; the other phase is unchanged. Lane identity and non-placement settings (importance rank, weights, foot rules) stay on their original lanes. A swap preserves each affected lane's role where it stays compatible with the new placement and clears it for re-selection where it becomes incompatible. Failed-save draft retention, validation, invalidation, read-only/context behavior, and Save tactic stay unchanged.
- Existing saved tactics are reset once to the normal defaults on upgrade; planner assignments and all other save data are preserved. No duplicate or overflow draft states can be created by edits.
- Below 1920px viewport width the pitch keeps its portrait attack-up orientation. At viewport widths >= 1920px the pitch uses a real 90-degree landscape orientation. Marker text stays upright in both orientations.
- At 3440×1440 the Tactic workspace is centered or max-width constrained so side panels and pitch do not stretch indefinitely. The global `content-max-width: none` contract does not change.
- At 1280×800 the workspace may scroll vertically but must not overflow horizontally. At 1600×900 and 1920×1080 it fits cleanly. At 1920 the landscape orientation is active.

## Invariants

- One tactic per app save, shared by all teams. `save_planner_tactic` validates and replaces the complete lane set. Backend work is bounded to one one-time data migration: `reset_planner_tactic_lanes` deletes every `planner_tactic_lanes` row once (closest analogue: v36 `reset_staff_assignment_targets`), and the existing lazy seed (`get_tactic` → `default_tactic`) recreates the normal defaults per save. No schema change, no new IPC, no new command. Planner assignments live in the separate `planner_assignments` table keyed by stable lane IDs and are untouched, as is all other save data.
- Eleven ordered, save-scoped `planner_tactic_lanes` rows with linked IP/OOP positions and roles, 0–1 IP weight, optional unique importance rank 1–11, and a preferred-foot rule. A qualified placement appears only once per phase — enforced at edit time by placement swaps and kept at save time by validation.
- Invalid or incomplete role-position pairs cannot save. Failed saves retain the draft. The save flow invalidates depth, slot candidates, and role references.
- Selection is one shared `selectedLaneId` across pitch markers, XI panel rows, and inspector. Focus, hover, and selection emphasize the linked counterpart.
- Keyboard traversal and DOM/tab order follow the current visual pitch order in each orientation (stable left-to-right/top-to-bottom with deterministic ties), never raw lane IDs. Attack direction is explicitly described and stays understandable and accessible in both orientations. Marker text never rotates; the DOM is never rotated with a CSS transform.
- Linear position order governs lane lists: goalkeeper band toward striker band, IP position as the primary order. Visual pitch grids keep their attack geometry; DOM/tab order follows the current visual pitch order in each orientation, not raw lane IDs.
- No cross-feature imports. Planner components compose in the My Club route. `src/lib/tauri-client.ts` stays the sole invoke site.

## Non-goals

- Tactic names, tactic selection, formation presets, multiple tactics, ongoing resets, broad save deletion, and new IPC. The only backend work is the one-time tactic reset migration plus removal of the now-obsolete legacy-placement compatibility code. Generated screenshot controls that imply those features are directional only.
- Rendering, preserving, or migrating duplicate-placement/overflow draft states and legacy overcapacity. Edits can no longer create them; the reset removes stored ones.
- Drag-and-drop, movement-line toggles, tactical insight cards, and summary analytics from the concept material. They are not in JAY-57 acceptance criteria.
- Changes to the global `content-max-width: none` contract. Squad tables need full width.
- New dependencies, new abstractions, and new media-query hooks unless direct CSS responsive behavior cannot meet the tested contract.
- Duplicate unit/component/e2e coverage and extensive visual snapshot infrastructure. Rework the existing route and Playwright smoke tests where they already own behavior.
- Current-state documentation edits inside implementation commits. `.wiki/ARCHITECTURE.md` and `.wiki/DESIGN.md` describe the old two-pitch layout and go stale during implementation; reconciliation happens at feature close-out after behavior exists.

## Current-state map

- Relevant components: `src/features/planner/components/planner-tactic-editor.tsx` (375 lines; owns draft, view, selection, save mutation, and layout), `src/features/planner/components/planner-tactic-pitch.tsx` (453 lines; grid-band renderer with `PITCH_ROWS`, `PitchBoard`, `LaneButton`, and position-layout helpers; rotates nothing), `src/features/planner/components/planner-tactic-inspector.tsx` (242 lines; already owns all required controls), `src/features/planner/components/tactic-context-boundary.tsx` (context guards and read-only behavior).
- Shared pitch consumer: `planner-tactic-pitch.tsx` exports the reusable portrait, single-phase `PlannerTacticPitch`, also consumed by `src/features/planner/components/planner-role-reference-modal.tsx` (Best role fit modal, single `phase` prop; outside JAY-57 and stays portrait single-phase).
- Data model: `src/features/planner/types/tactic.ts` has all needed slot fields (`laneId`, `ipWeight`, `importanceRank`, `preferredFoot`, `footPreference`, `ipPosition`, `ipRoleId`, `oopPosition`, `oopRoleId`). Lane identity comes from `TACTIC_LANE_IDS` in `src/utils/tactic-ids.ts`. No model change is needed.
- Persistence and migrations: Rust owns one save-scoped tactic; `get_planner_tactic` seeds a 4-3-3 DM IP / 4-1-4-1 DM OOP tactic and `save_planner_tactic` replaces the lane set. A one-time `reset_planner_tactic_lanes` data migration (Commit 5, analogue of v36) is the only planned migration; until it lands, the implemented state has no tactic migration. Planner assignments live in the separate `planner_assignments` table keyed by stable lane IDs and survive a tactic reset.
- Existing behavioral assumptions: the editor renders `visiblePhases(view).map` into `lg:grid-cols-2`, then one bottom inspector. `PlannerTacticInspector` receives `phases={visiblePhases(view)}` and all update callbacks. `phasePosition`, `phaseRoleId`, `phasePositionLabel`, `roleLabel`, and `linkedPositionDescription` in `src/features/planner/utils/tactic-editor.ts` (331 lines) already describe linked slots.
- Architectural seams: the My Club route (`src/app/routes/my-club.tsx`, tactic workspace block) mounts `PlannerTacticEditor` under `TacticContextBoundary` with `hidden`-prop workspace preservation. Planner tactic, depth, slot candidates, and role references share the Planner Query tree. Styling uses existing Tailwind v4 tokens in `src/styles/global.css`; responsive behavior currently uses Tailwind breakpoints only.
- Project validation commands: `./scripts/dev test <pattern>`, `./scripts/dev check-app`, `./scripts/dev check`, `./scripts/dev smoke` (Chromium via `pnpm exec playwright install chromium`).
- Primary risks: the uncommitted overflow-based canvas work (staged in `planner-tactic-pitch.tsx`, `my-club-squad.test.tsx`, `e2e/smoke.spec.ts`) is abandoned and must be restored to HEAD before implementation resumes, or the new simplified canvas will collide with it; the swap contract must clear incompatible roles without stranding lane identity or non-placement settings; the one-time reset must preserve planner assignments; landscape geometry at 1920×1080 must fit pitch, XI panel, and inspector without document scrolling; Both-mode connectors must stay readable when many slots change position.

## Feature architecture

- `PlannerTacticEditor` keeps ownership of draft state, `TacticView` switching, shared `selectedLaneId`, validation, failed-save draft retention, and the save mutation with its existing invalidation set. No ownership moves.
- Draft editing owns the unique-placement swap contract. A pure helper beside `updatePhaseLane` in `src/features/planner/utils/tactic-editor.ts` (new, unit-tested) resolves a placement pick: an unoccupied qualified placement applies directly; an occupied one swaps the two lanes' placements in the edited phase only, leaving the other phase untouched. Lane identity and non-placement settings (importance rank, weights, foot rules) stay on their original lanes. Each affected lane keeps its role where `rolesForPhase` still lists it for the new placement (base-position tags: AMC sides → AMC, DC/DM/MC sides → their base, STCR/STC/STCL → catalog ST) and clears it to `""` where it becomes incompatible, so the existing save validation (`Choose a compatible … role`) stands unchanged. `updatePosition` in `planner-tactic-editor.tsx` wires the helper; no other edit path can create duplicates, and save validation remains the backstop.
- The one-time tactic reset is a data-only migration. New `reset_planner_tactic_lanes` SQL (`DELETE FROM planner_tactic_lanes;`, closest analogue v36 `reset_staff_assignment_targets`) registered as the next migration version; the existing lazy seed recreates normal defaults per save on next read. The same commit removes the now-unreachable legacy-placement compatibility code (`normalize_legacy_placements`, `normalize_legacy_phase`, and the legacy-load tests) because no legacy rows can exist after the reset. No new IPC, no new command, no schema change; `planner_assignments` and all other save data are untouched.
- The one-time tactic reset is a data-only migration. New `reset_planner_tactic_lanes` SQL (`DELETE FROM planner_tactic_lanes;`, closest analogue v36 `reset_staff_assignment_targets`) registered as the next migration version; the existing lazy seed recreates normal defaults per save on next read. The same commit removes the now-unreachable legacy-placement compatibility code (`normalize_legacy_placements`, `normalize_legacy_phase`, and the legacy-load tests) because no legacy rows can exist after the reset. No new IPC, no new command, no schema change; `planner_assignments` and all other save data are untouched.
- The shared `PlannerTacticPitch` stays the reusable portrait, single-phase canvas used by both current consumers: `PlannerTacticEditor` (through the workspace-only phase-aware wrapper) and the Best role fit modal (`src/features/planner/components/planner-role-reference-modal.tsx`, single `phase` prop). It derives each rendered marker's normalized portrait coordinate from a direct qualified-placement table over the current `phasePosition(lane, phase)`, projects it with the pure geometry helper, draws SVG/HTML pitch markings and conditional connectors, and renders markers from the existing `phaseRoleId`, `phasePositionLabel`, and `roleLabel` helpers. There are no duplicate/overflow branches: uniqueness is guaranteed by the swap contract, the reset, and save validation, so repeated-placement layout, overflow rows, spacers, and sort offsets are removed, not reimplemented. Both-mode connectors compare canonical full qualified placement identity (DCR → DCL connects; legacy-equivalent ST → STC does not). It introduces no scoring, persistence, or mutation logic, and it never gains an orientation source: the modal stays portrait and single-phase and is explicitly outside JAY-57.
- A workspace-only `PlannerPhaseAwareTacticPitch` wrapper composes the same shared canvas logic for `TacticView`: one marker per lane in IP/OOP mode, two visually distinct phase markers per lane in Both mode. Only `PlannerTacticEditor` switches to it. No duplicate renderer logic lives in the wrapper.
- A new generic orientation-aware normalized coordinate projection owns the portrait-to-landscape mapping as a pure function over normalized coordinates, directly useful to every supported placement value. The workspace phase-aware wrapper owns the single orientation source: one colocated `matchMedia("(min-width: 1920px)")` read with change subscription and cleanup inside the wrapper component; no dependency and no shared hook. Landscape is a clockwise 90-degree rotation, so portrait attack-up becomes landscape attack-right, with an explicit accessible direction description.
- The Tactical XI panel owns lane listing and selection only. It reads the same draft lanes and writes the same `selectedLaneId` as the pitch. Each row shows the existing `linkedPositionDescription` transition.
- `PlannerTacticInspector` is first adapted in place into the persistent **Selected Slot** presentation (region/heading renamed from Selected position settings to Selected Slot; both IP and OOP position/role controls always exposed regardless of pitch view; IP/OOP weight kept visually associated with both phases) while it remains in its current location, reusing all existing callbacks, controls, validation, failed-save, and read-only behavior. The editor then moves the now-complete inspector from the bottom shelf into a persistent aside beside the pitch.
- Ultrawide containment applies only to the Tactic workspace block in the My Club route. No global token, shell, or Squad surface changes.

## Uncertainty register

### Known

- Current HEAD at replan time is `8d9e142fa4000aa2c31ebc93f4e96d9d3b70b15a` on `feat/tactics-page-redesign`. Commits 1 (planning, `119b51910fa2eaea3163f9716a68be933356cc1e`) and 2 (geometry, HEAD) are completed and preserved; the Commit 2 geometry helper and its unit tests are overflow-independent and stay valid.
- The uncommitted normalized-canvas work staged in `src/features/planner/components/planner-tactic-pitch.tsx`, `src/app/routes/my-club-squad.test.tsx`, and `e2e/smoke.spec.ts` implements the rejected overflow contract (`OVERFLOW_SORT_GAP`, `OVERFLOW_SPACER_PCT`, overflow rows/spacers/sort offsets, duplicate-row smoke assertions). It is abandoned: the supervisor restores those three paths to HEAD with developer approval before implementation resumes, and the future writer starts from the HEAD file versions. The worktree otherwise matches HEAD plus this ledger edit.
- The existing main UI test surface for tactic behavior is `src/app/routes/my-club-squad.test.tsx` (tactic command-bar order, pitch attack-to-goalkeeper order, save flow). The existing browser contract is `e2e/smoke.spec.ts` (`planner tactic editor saves a linked phase adjustment`, `planner tactic workspace fits its supported desktop viewports`). `tactic-context-boundary.test.tsx` covers context guards that do not change.
- `PlannerTacticInspector` already owns every required control, so the inspector move needs no control work.
- No Rust, migration, IPC, or schema work exists in this feature. The persistence model is untouched.

### Assumptions

- Headless Chromium accepts a 3440×1440 Playwright viewport for the ultrawide assertions. The worker verifies this in the containment commit and reports a gap if the runner clamps it.
- `jsdom` has no `matchMedia`; unit coverage proves the pure projection/order function plus a small `matchMedia` stub, while real orientation behavior (initial 1920px load and live 1919↔1920 crossing) is proved through `./scripts/dev smoke`.
- Resetting every save's tactic settings to the normal defaults is acceptable: the user explicitly authorized discarding existing tactics. Planner assignments and all other save data survive because they live in separate tables keyed by stable lane IDs, and the reset commit proves it with backend tests.

### Decisions

- Unique-placement swap contract replaces draft-overflow support. The user picks exact qualified placements with one lane per qualified placement per phase; selecting an occupied placement swaps the two lanes' placements in the edited phase only, preserving lane identity and non-placement settings on their original lanes. Approved developer intent (user's swap interpretation, accepted with 'Great'). Consequence: the renderer, layout helpers, tests, and smoke assertions for repeated/overflow drafts are removed, not fixed; save validation stays as the backstop.
- Role compatibility on swaps uses base positions (AMC sides → AMC, DC/DM/MC sides → their base, STCR/STC/STCL → catalog ST, per the repository's `basePosition`/`rolesForPhase`). Compatible roles are preserved; incompatible ones are cleared to `""` so the existing `Choose a compatible … role` validation stands. Consequence: a swap can leave a lane temporarily unsavable until its role is re-chosen; failed-save draft retention is unchanged.
- One-time tactic reset via a data-only migration (`DELETE FROM planner_tactic_lanes`, analogue of v36), with normal default recreation through the existing lazy seed and removal of the now-unreachable legacy-placement normalization. No new IPC, no new command, no schema change, no ongoing resets. Approved developer intent (existing tactics may be thrown away). Consequence: every save's tactic settings reset once on upgrade; planner assignments and all other save data are preserved.

- One phase-aware pitch replaces the two-board grid, with per-mode marker rules and conditional connectors. Approved developer intent. Consequence: the grid-band renderer and its shared card-width logic are removed in the pitch commit.
- The Tactical XI panel lists all 11 lanes with IP → OOP transitions and shares `selectedLaneId` with the pitch. Approved developer intent. Consequence: selection state stays in the editor; the panel adds no parallel state.
- The inspector is adapted in place into the persistent Selected Slot presentation (both-phase controls, renamed heading) and then moved beside the pitch, with every control and callback reused. Approved developer intent. Consequence: `planner-tactic-inspector.tsx` gains control-presentation edits in the first step and layout-only composition in the second; control logic never moves.
- Landscape orientation activates at viewport widths >= 1920px through one colocated `matchMedia("(min-width: 1920px)")` source inside the workspace phase-aware wrapper component (initial read, change subscription, cleanup; no dependency or shared hook) driving the generic projection, with SVG/HTML markings; landscape is a clockwise 90-degree rotation so portrait attack-up becomes landscape attack-right with an explicit accessible direction description; marker text stays upright and the DOM is never CSS-transform rotated; DOM/tab order follows the current visual pitch order in each orientation, not raw lane IDs. Approved developer intent. Consequence: portrait behavior below 1920px must remain pixel-compatible with current attack-up geometry.
- Ultrawide containment is scoped to the Tactic workspace; the global `content-max-width: none` contract stands. Approved developer intent. Consequence: Squad and other surfaces are unaffected.
- One PR with an ordered atomic commit sequence. Repository evidence shows no independently mergeable trunk-safe seam that needs a separate merge boundary: every commit builds on the same editor surface and no migration, dependency, or protocol change exists. Consequence: a single review and merge boundary.
- No ADR. The work recombines existing Planner boundaries with approved responsive breakpoints and adds no durable structural alternative. If implementation exposes a consequential durable decision, the worker stops and reports instead of improvising an ADR path.

### Unknowns

- Exact landscape column widths that fit pitch, XI panel, and inspector at 1920×1080 without document scrolling. The containment commit resolves this against the smoke fit matrix; compact marker density is the bounded fallback.
- Salvage value of the abandoned staged canvas work beyond the retained Commit 2 helper. The plan assumes none: the staged diff bakes in the rejected overflow contract, so the canvas commit starts from HEAD file versions rather than revising the staged files.

### Risks

- Staged-file collision. The abandoned overflow work stays staged until the supervisor restores the three paths to HEAD. Mitigation: the replan-recording commit (Commit 3) makes restoration an explicit precondition; the worker verifies a clean status read-only and stops otherwise.
- Swap-role stranding. A swap that clears an incompatible role leaves the draft unsavable until re-chosen. Mitigation: the existing validation message names the lane and phase, the inspector role picker already defaults to an empty `Choose a role` state, and the swap commit proves both the preserve and clear paths with unit tests plus a route-level swap flow.
- Reset data-loss scope. The migration deletes every save's tactic rows. Mitigation: user-explicit authorization, table separation (`planner_assignments` untouched), and backend tests proving assignments survive and defaults reseed; no broader deletion is in scope and any wider scope stops the commit.
- Simplified-canvas regression. Removing the grid-band renderer and the staged overflow branches changes marker geometry. Mitigation: rewritten unique-placement-only route and smoke geometry assertions fail on the old layout and pass on the canvas; modal coverage is retained unchanged.
- Both-mode connector clutter when many slots change position. Mitigation: connectors render only where the canonical full qualified placement identity changes (DCR → DCL connects; legacy-equivalent ST → STC does not), and the XI panel carries the full transition text.
- Smoke viewport runtime growth from added widths. Mitigation: extend the existing viewport test in place instead of adding new specs.
- Shared pitch consumer regression (Best role fit modal). Mitigation: the canvas commit preserves the `PlannerTacticPitch` component/API with the modal file deliberately unchanged, the consolidation commit keeps the modal on the shared portrait single-phase path while only the editor moves to the wrapper, the landscape commit scopes `matchMedia` to the wrapper only, and retained route/smoke coverage guards the modal in those commits.

## Walking skeleton

Establish the edit-time contract first with the pure placement-swap helper and its unit tests, then run the one-time tactic reset migration with backend proof that assignments survive and defaults reseed. Only then migrate the shared per-phase board internals onto a simplified normalized-coordinate canvas (unique qualified placements only, no overflow branches) behind the unchanged one-board/two-board composition, and consolidate IP/OOP/Both onto one workspace phase-aware wrapper with Both-mode transitions. That proves contract, data, geometry, and composition incrementally before the XI panel, in-place Selected Slot controls, beside-pitch composition with fit guards, containment, and orientation land.

## Delivery plan

### PR 1 — Redesign the tactic workspace

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feat/tactics-page-redesign`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict status `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(tactics): redesign the tactic workspace`

**Purpose:** Deliver the complete JAY-57 workspace redesign in one review boundary: unique-placement swaps, a one-time tactic reset to defaults, one phase-aware pitch, Tactical XI panel, beside-pitch inspector, landscape orientation at >= 1920px, ultrawide containment, and viewport-fit proof.

**Depends on:** None.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(tactics): record approved workspace redesign plan`

**Work:** Commit the reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/tactics-page-redesign.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — active feature state with a link to this ledger; the existing Next item is retained.

**Behavior and data flow:**

- Move planning truth into one reviewed active ledger and record the exact delivery sequence before implementation.

**Ordered implementation steps:**

1. Verify the active branch (`feat/tactics-page-redesign`) and base (`main`) without changing Git state.
2. Confirm the worktree contains only the reviewed planning paths.
3. Run the ledger classifier.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The ledger classifier proves structural consistency.

**Patterns to verify:**

- The active-ledger template, current TODO ownership rules, and schema 2 status vocabulary.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, BACKLOG, ADRs, plan scope, packet order, or reviewed decisions.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/tactics-page-redesign.md`

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files.

#### Commit 2 — Add orientation-aware pitch geometry projection

**Status:** Completed

**Provisional commit:** `feat(tactics): add orientation-aware pitch geometry`

**Work:** Add the pure normalized coordinate projection that maps portrait attack-up slot coordinates into the active orientation, with unit tests. No UI change yet.

**Size assessment:** About 80 changed non-test implementation lines in one new util file. Within the soft target.

**Out of scope:**

- Pitch rendering changes, XI panel, inspector move, responsive behavior, persistence, and test-asset changes outside the new unit test.

**Implementation packet:**

- Give the worker a complete handoff for a pure helper with no component wiring.

**Files and responsibilities:**

- `src/features/planner/utils/tactic-pitch-geometry.ts` (new) — generic pure `projectPosition(coordinate, orientation)` over normalized portrait coordinates plus the normalized portrait coordinate table directly useful to every supported placement value. No lane, component, Query, or IPC imports; no lane-ID coordinates.
- `src/features/planner/utils/tactic-pitch-geometry.test.ts` (new) — RED proof first: portrait identity mapping, clockwise 90-degree landscape mapping (portrait attack-up becomes landscape attack-right), coordinate bounds, and visual pitch order (stable left-to-right/top-to-bottom with deterministic ties) for representative coordinates. Projection/order only; upright labels are proved in smoke, not here.

**Behavior and data flow:**

- Pure function in, projected coordinates out. No state, no side effects, no persistence. The helper stays generic over coordinates; the pitch commit derives each marker's portrait coordinate from the current `phasePosition(lane, phase)` plus the existing dynamic `phasePositionLayout`, preserving stable distinct placement for repeated/overflow draft positions. Later pitch commits consume it; this commit wires nothing.

**Ordered implementation steps:**

1. Add the failing unit test for portrait identity and landscape projection.
2. Implement the minimum pure projection that turns the test GREEN.
3. Refactor only while the focused proof stays green.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- New `tactic-pitch-geometry.test.ts` is the RED→GREEN proof: it fails on the missing module, then passes. It guards portrait identity, clockwise landscape rotation direction, coordinate bounds, and projected visual order. No existing test asset changes.

**Patterns to verify:**

- Existing placement vocabulary in `src/features/planner/utils/tactic-editor.ts` (qualified placements, canonical ST → STC) for naming the portrait coordinate table; no lane logic enters the helper.

**Constraints and non-goals:**

- No component, styling, IPC, Rust, migration, or dependency changes. No orientation subscription here; the single colocated `matchMedia("(min-width: 1920px)")` source is wired in the landscape commit. The helper takes coordinates only, never lane IDs.

**Dependencies and sequencing:**

- Requires Commit 1. Blocks the canvas migration (Commit 6).

**Validation:** `./scripts/dev test tactic-pitch-geometry`, then `./scripts/dev check`.

**Stop conditions:** Stop if the projection needs domain data beyond lane positions and orientation (that belongs in the pitch component, not the pure helper), or if portrait coordinates cannot stay compatible with current attack-up geometry.

**Review mandate:** Verify purity (no lane, component, store, or IPC imports; coordinates only), coordinate correctness over supported placement values, projected visual order, test value of each case, and that no UI or unrelated file changed.

#### Commit 3 — Record the approved replan

**Status:** Completed

**Provisional commit:** `docs(tactics): record approved workspace replan`

**Work:** Commit the reviewed replanning artifacts on the feature branch before new implementation: the swap-contract plus one-time-reset ledger, the reorganized packet sequence, the TODO reconciliation, and the invalidation of the previous Delivery fingerprint.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, staged-file restoration mechanics beyond the recorded precondition, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted replan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/tactics-page-redesign.md` — replanned feature intent, swap/reset contract, delivery plan, and packets.
- `.wiki/TODO.md` — reconciled active feature summary with a link to this ledger.

**Behavior and data flow:**

- Move replanning truth into one reviewed active ledger and record the exact new delivery sequence before implementation resumes.

**Ordered implementation steps:**

1. Verify the active branch (`feat/tactics-page-redesign`) and base (`main`) without changing Git state.
2. Confirm the supervisor has restored the three abandoned staged paths (`src/features/planner/components/planner-tactic-pitch.tsx`, `src/app/routes/my-club-squad.test.tsx`, `e2e/smoke.spec.ts`) to HEAD with developer approval, so the worktree contains only HEAD plus the reviewed planning paths. Stop otherwise.
3. Run the ledger classifier.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The ledger classifier proves structural consistency.

**Patterns to verify:**

- The active-ledger template, current TODO ownership rules, and schema 2 status vocabulary.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, BACKLOG, ADRs, plan scope, packet order, or reviewed decisions. Do not revive the abandoned staged overflow work.

**Dependencies and sequencing:**

- Requires an accepted replan-review verdict, developer acceptance, a provisional Delivery fingerprint, exact branch activation, and the staged-file restoration precondition. Blocks Commit 4 (first new implementation).

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/tactics-page-redesign.md`

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, a branch mismatch, or unrestored abandoned staged files.

**Review mandate:** Verify that the staged diff contains the complete reviewed replanning outcome and no implementation or unrelated files, that completed Commits 1–2 are preserved byte-for-byte in intent, and that the restoration precondition is recorded.

#### Commit 4 — Enforce unique-placement swaps in draft editing

**Status:** Completed

**Provisional commit:** `feat(tactics): enforce unique-placement swaps`

**Work:** Establish the edit-time uniqueness contract before any renderer relies on it: a pure placement-swap helper plus `updatePosition` wiring so picking an occupied qualified placement swaps the two lanes' placements in the edited phase, preserves compatible roles, and clears incompatible roles for re-selection. No renderer, layout, persistence, or IPC change.

**Size assessment:** About 60 changed non-test implementation lines (one pure helper plus `updatePosition` wiring). Within the soft target.

**Out of scope:**

- Renderer changes, canvas migration, XI panel, inspector presentation or move, orientation switching, containment, persistence, IPC, Rust, migrations, and new dependencies.

**Implementation packet:**

- Add the pure swap helper with unit tests, then wire the single existing edit path through it.

**Files and responsibilities:**

- `src/features/planner/utils/tactic-editor.ts` — pure `swapPhasePlacement`-style helper beside `updatePhaseLane`: given the draft lanes, an editing lane ID, a phase, and the picked qualified placement, return the next lanes. An unoccupied placement applies directly; an occupied one exchanges the two lanes' placements in that phase only. Lane identity and non-placement settings stay on their original lanes. Each affected lane keeps its role where `rolesForPhase` still lists it for the new placement and resets it to `""` where it does not. The other phase is untouched. No component, Query, or IPC imports.
- `src/features/planner/components/planner-tactic-editor.tsx` — route `updatePosition` through the helper; every other update path stays identical.
- `src/features/planner/utils/tactic-editor.test.ts` (new) — RED→GREEN proof: unoccupied pick applies directly; occupied pick swaps exactly the two lanes' placements in the edited phase and leaves the other phase unchanged; ranks, weights, and foot rules stay on their original lanes; compatible roles survive on both lanes; an incompatible role clears to `""` on the affected lane only.
- `src/app/routes/my-club-squad.test.tsx` — replace the obsolete `blocks two lanes from using the same sided placement` route test (HEAD: selecting occupied MCL asserts the `MCL is already used in the In-Possession phase.` alert plus a disabled Save) with swap proof: picking the occupied placement swaps the two pitch markers, leaves the other phase unchanged, and surfaces the re-selection state for a cleared role, while the save flow still rejects the incomplete draft with the existing message. The neighboring seam tests are audited and retained: the MCR save round-trip above and `treats legacy ST and canonical STC as the same placement` below still describe the unchanged validation contract.

**Behavior and data flow:**

- Pure helper in, next lanes out. The editor's draft, validation, failed-save retention, save mutation, and invalidation set are untouched. After this commit no edit path can construct a duplicate-placement draft; save validation remains the backstop for corrupted loads.

**Ordered implementation steps:**

1. Add the failing swap unit tests and confirm RED.
2. Implement the helper until the proof turns GREEN.
3. Wire `updatePosition` through the helper and extend the route assertion until GREEN.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- New `tactic-editor.test.ts` swap cases plus the focused route swap assertion are the RED→GREEN proof: they fail on direct-assignment editing and pass with swaps. They cover direct apply, occupied swap with other-phase stability, non-placement setting stability, role preservation, and role clearing with the existing validation message. The obsolete duplicate-error route test is replaced, not kept; every other tactic route assertion is unaffected and stays green.

**Patterns to verify:**

- Existing `updatePhaseLane`, `rolesForPhase`, and `basePosition` vocabulary for the compatibility rule; the inspector's empty-role `Choose a role` state for the cleared path.

**Constraints and non-goals:**

- No renderer, layout, persistence, IPC, or dependency change. No new placement vocabulary: the helper consumes the existing `options.placements` values. Do not move ranks, weights, or foot rules between lanes.

**Dependencies and sequencing:**

- Requires Commit 3 (replan recorded, staged files restored). Blocks Commit 6 (simplified canvas relies on edit-time uniqueness).

**Validation:** `./scripts/dev test tactic-editor`, then `./scripts/dev test my-club-squad`, then `./scripts/dev check`.

**Stop conditions:** Stop if role compatibility needs more than base-position tags (report instead of inventing a new rule), or if any second edit path can construct duplicates (report instead of widening scope).

**Review mandate:** Verify swap-only semantics (exactly two lanes change, edited phase only), lane-identity and setting stability, role preserve/clear correctness against `rolesForPhase`, untouched save/validation behavior, unit-test value of each case, and that the diff stays within the helper, the `updatePosition` wiring, and owned tests.

#### Commit 5 — Reset stored tactics to defaults once

**Status:** Completed

**Provisional commit:** `feat(tactics): reset stored tactics to defaults`

**Work:** Discard existing stored tactics once via a data-only migration and remove the now-unreachable legacy-placement compatibility code. Normal defaults are recreated through the existing lazy seed; planner assignments and all other save data are preserved. No new IPC, no new command, no schema change, no frontend change.

**Size assessment:** About 40 changed non-test Rust lines (one migration constant plus registry entry) with net-negative compatibility code. Within the soft target.

**Out of scope:**

- Renderer changes, swap-contract changes, frontend edits, new IPC or commands, schema changes, ongoing resets, and any deletion outside `planner_tactic_lanes`.

**Implementation packet:**

- Add the reset migration, delete the dead normalization path with its tests, and prove assignment survival plus default recreation.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — new `TACTIC_RESET_SQL` constant (`DELETE FROM planner_tactic_lanes;`, closest analogue v36 `STAFF_ASSIGNMENT_TARGETS_RESET_SQL`) registered as migration version 42 (current highest is v41 `create_player_shortlist_entries`) with the registry's usual version assertions updated. Data-only; no table or index change.
- `src-tauri/src/db/migrations.rs` tests module (beside `migrates_populated_v35_by_clearing_only_staff_assignment_targets`) — new `migrates_populated_v41_by_resetting_only_tactic_lanes`: build the DB through v41 (`filter(version <= 41)`, set `user_version`), seed two saves with divergent tactic lanes (full v41 column set including importance ranks and foot preferences), `planner_strings` plus `planner_assignments` rows on stable lane IDs, and a named unrelated retained record (`planner_teams` rows, mirroring the v35 test). Run `apply(&conn)` and assert `user_version` 42, tactic rows cleared, each save lazily reseeds `default_tactic()` on `get_tactic`, a recreated tactic saves cleanly, then run `apply(&conn)` again and assert the recreated tactic, the assignments, and the `planner_teams` record are preserved. This test — not a vague tactic-service migration run — owns the reset proof.
- `src-tauri/src/features/planner/tactic.rs` — remove `normalize_legacy_placements` and `normalize_legacy_phase` (unreachable: no legacy rows can exist after the reset) and their call in `load_tactic`. Keep `canonical_placement` (ST → STC equivalence still guards saves), uniqueness validation, and role validation unchanged.
- `src-tauri/src/features/planner/tactic.rs` — remove `normalize_legacy_placements` and `normalize_legacy_phase` (unreachable: no legacy rows can exist after the reset) and their call in `load_tactic`. Keep `canonical_placement` (ST → STC equivalence still guards saves), uniqueness validation, and role validation unchanged.
- `src-tauri/src/features/planner/tactic.rs` tests — remove only the legacy-load tests (`legacy_repeated_base_positions_keep_their_existing_visual_order`, `legacy_groups_larger_than_three_load_but_require_resolution_before_save`). No reset proof lives here; it is owned by the migrations.rs test above. Remaining validation tests (duplicate rejection, role compatibility, weights, ranks, foot rules) stay green unchanged.

**Behavior and data flow:**

- The migration runs once per database on upgrade. The next `get_tactic` read per save finds no rows and seeds `default_tactic()` through the unchanged lazy path. Save, validation, and invalidation behavior are untouched.

**Ordered implementation steps:**

1. Add the failing reset tests (assignments survive, defaults reseed) and confirm RED.
2. Add the migration and remove the dead normalization path until the proof turns GREEN.
3. Refactor only while the focused proof stays green.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- New migration tests are the RED→GREEN proof: they fail without the reset (legacy rows persist) and pass with it (defaults reseeded, assignments preserved, duplicate saves still rejected). Existing tactic validation tests stay green unchanged.

**Patterns to verify:**

- The v36 reset migration as the closest analogue (data-only `DELETE`, registry pattern, version assertions); `load_or_initialize_tactic_in_tx` as the unchanged reseed path.

**Constraints and non-goals:**

- Exactly one `DELETE FROM planner_tactic_lanes;` statement in the migration. No `WHERE` scoping that could leave half-reset saves, no new command or IPC surface, no frontend change in this commit, no second reset mechanism.

**Dependencies and sequencing:**

- Requires Commit 3 (replan recorded). Independent of Commit 4 (different seam: backend versus frontend); blocks Commit 6 (canvas assumes no legacy rows).

**Validation:** `./scripts/dev check-rust` (owns the v42 migration test and the tactic-service suite), then `./scripts/dev check`.

**Stop conditions:** Stop if the reset needs IPC or frontend coordination (report instead of adding it), if any non-tactic table loses rows (report as data-loss scope breach), or if the migration registry pattern cannot express a data-only reset (report instead of improvising).

**Review mandate:** Verify the migration deletes only `planner_tactic_lanes` rows, normalization removal is complete with no remaining callers, assignment and save-data preservation is proved (not asserted), duplicate-save rejection still holds, and the diff stays within the migration registry, the tactic service, and owned backend tests.

#### Commit 6 — Migrate the pitch to a simplified normalized canvas

**Status:** Active

**Provisional commit:** `feat(tactics): migrate pitch to normalized canvas`

**Work:** Replace the per-phase grid-band board internals of the reusable `PlannerTacticPitch` with a simplified normalized-coordinate single-phase canvas (unique qualified placements only; no duplicate/overflow branches) while preserving its component/API and current editor behavior: IP and OOP views still render one phase board each and Both still renders two boards (`visiblePhases(view).map` into `lg:grid-cols-2` stays), and the Best role fit modal keeps consuming the same portrait single-phase component. Start from the HEAD file versions; the abandoned staged overflow work is not a base. Reuse the current phase helpers, preserve selection/highlight/accessibility and edited positions. Remove obsolete grid helpers only together with their final callers.

**Size assessment:** About 200 changed non-test implementation lines in `planner-tactic-pitch.tsx` — smaller than the abandoned attempt because duplicate/overflow branches are deleted rather than reimplemented. At the soft target boundary; splitting the swap would leave half-migrated geometry on trunk.

**Out of scope:**

- One-canvas consolidation (Commit 7), Both-mode connectors and transitions (Commit 8), XI panel, inspector move, orientation switching, ultrawide containment, persistence, IPC, Rust, and new dependencies.

**Implementation packet:**

- Migrate the renderer internals behind the unchanged board composition and the retained shared component/API for unique placements only.

**Files and responsibilities:**

- `src/features/planner/components/planner-tactic-pitch.tsx` — single-phase normalized canvas behind the retained reusable `PlannerTacticPitch` component/API. Derive each rendered marker's portrait coordinate from a direct qualified-placement table over the current `phasePosition(lane, phase)`, then project with the Commit 2 helper (portrait path used here). Reuse `LaneButton` semantics, `phaseRoleId`, `phasePositionLabel`, `roleLabel`, `phaseDescription`, and `linkedPositionDescription`. Keep the exported per-phase board API both consumers use. Remove `PITCH_ROWS`, `PitchBoard`, the shared card-width helpers, and every duplicate/overflow branch (overflow rows, spacers, sort offsets) only together with their last caller.
- `src/features/planner/utils/tactic-editor.ts` — remove the duplicate/overflow fallback machinery: `positionPlacement` and `CENTRAL_COLUMNS` go entirely, and `phasePositionLayout` becomes a direct qualified-placement lookup (unique placements are guaranteed by Commits 4–5 plus save validation). Trim the duplicate-count and `(row N)` suffix branches in `phasePositionLabel`; all consumers considered: the pitch is rewritten here, the inspector and the Best role fit modal render unique placements through the existing qualified-placement fast path with identical labels (verified by their retained coverage), and validation/depth text flows through the unchanged `phaseDescription`/`linkedPositionDescription` vocabulary. Export the existing private `canonicalPlacement` (ST → STC) so the pitch connector rule in Commit 8 shares the exact canonical identity the validation path already uses — no duplicated helper. Remove owned helpers with their last callers; no fake compatibility retention.
- `src/features/planner/components/planner-role-reference-modal.tsx` — deliberately unchanged. Its existing route/smoke coverage is retained as proof that the shared-component migration did not regress the modal consumer (that coverage lives in the same focused `my-club-squad` route test where applicable).
- `src/app/routes/my-club-squad.test.tsx` — explicitly remove or rewrite the five HEAD route tests that encode unsupported repeated/overflow geometry: `keeps repeated positions distinguishable without numeric labels` (HEAD:2982), `arranges repeated positions in stable central slots regardless of role` (HEAD:2995), `keeps every position button when a base position has more than three lanes` (HEAD:3180), `follows visible row order when a wide position overflows` (HEAD:3219), and `keeps a three-slot minimum when every tactic row has at most two positions` (HEAD:3267). Retain only supported unique-placement proof rewritten for the canvas (board count per view, attack-to-goalkeeper order, slot geometry, edited qualified position moving its marker). The abandoned staged duplicate/overflow assertions are not revived.
- `e2e/smoke.spec.ts` — rewrite the owned selectors and geometry assertions in `planner tactic editor saves a linked phase adjustment` for unique placements; the abandoned staged overflow assertions are not revived.

**Behavior and data flow:**

- The editor passes the same `lanes`, `options`, `selectedLaneId`, highlight state, and callbacks into the same per-phase board composition. Only the board internals change: markers are positioned from normalized portrait coordinates instead of grid bands. Selection, highlight, draft, validation, and save flow are untouched.

**Ordered implementation steps:**

1. Rewrite the smallest failing route geometry assertion to the unique-placement canvas contract and confirm RED.
2. Migrate the board internals to the normalized canvas until the proof turns GREEN.
3. Remove obsolete grid-band and duplicate/overflow helpers together with their final callers.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Rewritten route geometry assertions plus the rewritten smoke save test (proved via exact `./scripts/dev smoke`) are the behavior-preservation proof: they fail on the grid-band layout and pass on the canvas. They cover one board for IP/OOP views, two boards for Both, an edited qualified position moving its marker, and the preserved linked-phase save flow. `tactic-context-boundary.test.tsx` and the role-reference modal's existing route/smoke coverage are deliberately retained unchanged as proof for the second consumer.

**Patterns to verify:**

- Current `LaneButton` accessible naming (`IP:`/`OOP:` phase prefix with position and role) and the DESIGN linear position order for lane ordering.

**Constraints and non-goals:**

- Portrait attack-up geometry only; no orientation switching yet. No marker-set, composition, control, persistence, or IPC change. No drag-and-drop, movement-line toggles, insight cards, or analytics. No overflow or duplicate-placement rendering under any circumstance.

**Dependencies and sequencing:**

- Requires Commits 2 (projection helper, portrait path used), 3 (clean HEAD file versions), 4 (edit-time uniqueness), and 5 (no legacy rows). Blocks Commit 7 (one-canvas consolidation).

**Validation:** `./scripts/dev test my-club-squad`, then exact `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop if any existing setting, callback, validation message, or save behavior needs a contract change to fit the canvas, if a grid or overflow helper cannot be removed with its final caller (report the leftover instead of leaving dead code), or if the worktree does not present the expected clean-HEAD-plus-planning file state (stop and report; only the supervisor restores files after developer approval — the worker never mutates Git state).

**Review mandate:** Verify board-count preservation per view, marker geometry equivalence with the retired grid for unique placements, absence of every duplicate/overflow branch and constant, accessible naming, removal completeness for retired helpers, untouched selection/highlight/save behavior, absence of persistence or IPC drift, and that the diff stays within the pitch surface plus its owned tests.

#### Commit 7 — Consolidate onto one phase-aware canvas

**Status:** Pending

**Provisional commit:** `feat(tactics): consolidate one phase-aware canvas`

**Work:** Consolidate IP/OOP/Both onto one workspace-only phase-aware wrapper (`PlannerPhaseAwareTacticPitch`) composing the same shared canvas logic. IP and OOP modes render one marker per lane; Both mode renders two visually distinct phase markers per lane on the same pitch. No connectors yet and no duplicate renderer logic. Only `PlannerTacticEditor` switches to the wrapper; the Best role fit modal stays on the shared portrait single-phase `PlannerTacticPitch` (outside JAY-57). Preserve one shared selection/highlight state. Update the route and smoke contracts for one pitch.

**Size assessment:** About 150 changed non-test implementation lines across the pitch and editor composition. Within the soft target.

**Out of scope:**

- Both-mode connectors and role transitions (Commit 8), XI panel, inspector move, orientation switching, ultrawide containment, persistence, IPC, Rust, and new dependencies.

**Implementation packet:**

- Move the per-mode marker sets into the workspace wrapper composing the shared canvas logic behind the existing view state.

**Files and responsibilities:**

- `src/features/planner/components/planner-tactic-pitch.tsx` — shared canvas logic reusable by both the single-phase component and the workspace wrapper (marker-set selection by `TacticView`: one marker per lane in IP/OOP mode, two markers per lane in Both mode). Both-mode markers stay distinguished by phase treatment and accessible name. Marker coordinates derive from the direct qualified-placement table through the Commit 2 portrait projection. No duplicate renderer logic.
- `src/features/planner/components/planner-phase-aware-tactic-pitch.tsx` (new) — smallest workspace-only phase-aware wrapper composing the shared canvas logic for `TacticView`. Owns no geometry, scoring, persistence, or mutation logic.
- `src/features/planner/components/planner-tactic-editor.tsx` — render the workspace wrapper instead of the `visiblePhases(view).map` board composition. Only the editor switches; selection, highlight, and callback wiring stay identical.
- `src/features/planner/components/planner-role-reference-modal.tsx` — deliberately unchanged on the shared portrait single-phase `PlannerTacticPitch`; its existing route/smoke coverage is retained as proof (same focused `my-club-squad` route test where applicable).
- `src/app/routes/my-club-squad.test.tsx` — update tactic pitch assertions owned by this behavior to the one-pitch contract (single pitch per view, per-mode marker sets, attack-to-goalkeeper order).
- `e2e/smoke.spec.ts` — update `planner tactic editor saves a linked phase adjustment` to the one-pitch selectors.

**Behavior and data flow:**

- The editor passes the same lanes, options, selection, highlight state, and callbacks into the workspace wrapper; only the board composition collapses to one canvas. Selection and highlight write the same shared `selectedLaneId`. No data-flow change outside the canvas. The modal consumer keeps the unchanged shared component path.

**Ordered implementation steps:**

1. Update the smallest failing route assertion to the one-pitch contract and confirm RED.
2. Consolidate the composition onto the single canvas until the proof turns GREEN.
3. Remove the retired two-board composition and any remaining grid callers together.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Updated route pitch assertions plus the updated smoke save test (proved via exact `./scripts/dev smoke`) are the RED→GREEN proof: they fail on the two-board composition and pass on one canvas. They cover per-mode marker sets, Both-mode dual markers without connectors, and the preserved linked-phase save flow. No new unit test; projection and order stay covered by `tactic-pitch-geometry.test.ts`. The modal's existing route/smoke coverage is retained unchanged as proof the shared component/API still serves the second consumer.

**Patterns to verify:**

- Current `LaneButton` accessible naming in both modes and the DESIGN linear position order for lane ordering.

**Constraints and non-goals:**

- Portrait attack-up geometry only; no orientation switching yet. No connectors, no transitions text, no inspector or editor-layout change beyond the canvas composition. No persistence or IPC change.

**Dependencies and sequencing:**

- Requires Commit 6 (simplified normalized canvas with the per-phase board API). Blocks Commit 8 (Both-mode transitions layer onto this canvas).

**Validation:** `./scripts/dev test my-club-squad`, then `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop if any existing setting, callback, validation message, or save behavior needs a contract change to fit the single canvas, or if Both-mode dual markers cannot stay distinguishable without connectors (report instead of pulling Commit 8 scope forward).

**Review mandate:** Verify per-mode marker correctness, Both-mode marker distinctness without connectors, accessible naming in both modes, removal completeness for the retired composition, shared selection/highlight behavior, and that the diff stays within the pitch and editor composition plus owned tests.

#### Commit 8 — Add Both-mode tactical transitions

**Status:** Pending

**Provisional commit:** `feat(tactics): add Both-mode tactical transitions`

**Work:** Add Both-mode tactical transitions on the consolidated canvas: draw connectors only when the canonical full qualified placement identity changes (`DCR → DCL` connects; legacy-equivalent `ST → STC` does not) and expose a readable IP role → OOP role transition for each slot. Keep this separate from the one-canvas mode consolidation.

**Size assessment:** About 120 changed non-test implementation lines in the pitch component. Within the soft target.

**Out of scope:**

- Canvas consolidation (Commit 7), XI panel, inspector move, orientation switching, containment, persistence, IPC, Rust, and new dependencies.

**Implementation packet:**

- Add conditional connectors plus transition text to Both mode only.

**Files and responsibilities:**

- `src/features/planner/components/planner-tactic-pitch.tsx` — Both-mode connectors comparing canonical full qualified placement identity via the shared `canonicalPlacement` helper exported from `tactic-editor.ts` in Commit 6 (the same rule validation uses: ST → STC), plus a readable IP role → OOP role transition per slot reusing `roleLabel` and `linkedPositionDescription` vocabulary.
- `src/app/routes/my-club-squad.test.tsx` — focused route proof for connector conditionality (canonical change connects; legacy-equivalent pair does not) and the readable transition text.
- `e2e/smoke.spec.ts` — extend the existing tactic smoke flow with connector visibility/geometry assertions for a canonical changed placement and a legacy-equivalent placement.

**Behavior and data flow:**

- IP and OOP modes are untouched. In Both mode the canvas draws the same dual markers as Commit 7 plus connectors and transition text. No state, callback, or persistence change.

**Ordered implementation steps:**

1. Add the failing connector-conditionality assertion and confirm RED.
2. Add the connectors and transition text until the proof turns GREEN.
3. Refactor only while the focused proof stays green.
4. Run targeted and commit-level validation in the recorded order.

**Tests and proof:**

- Focused route assertions plus the extended existing tactic smoke flow are the RED→GREEN proof: the route assertions fail without the connectors and pass with them, covering the canonical-identity rule (`DCR → DCL` connects; legacy-equivalent `ST → STC` does not) and the readable IP role → OOP role transition; the extended smoke flow asserts connector visibility and geometry for a canonical changed placement and the absence of a connector for a legacy-equivalent placement. No duplicate projection coverage.

**Patterns to verify:**

- Shared `canonicalPlacement` in `src/features/planner/utils/tactic-editor.ts` (exported in Commit 6; ST → STC equivalence) for the connector rule — the pitch connector and save validation resolve identity through the same helper.

**Constraints and non-goals:**

- No marker-set, composition, orientation, or responsive change. No new analytics surfaces; the XI panel (Commit 9) carries the full transition text. No persistence or IPC change.

**Dependencies and sequencing:**

- Requires Commit 7 (consolidated canvas with Both-mode dual markers). Blocks Commit 13 (landscape projects these connectors).

**Validation:** `./scripts/dev test my-club-squad`, then exact `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop if Both-mode transitions cannot stay readable without new analytics surfaces, or if the canonical-identity rule needs a persistence or vocabulary change.

**Review mandate:** Verify connector conditionality against canonical identity, transition-text accuracy, Both-mode-only scoping, absence of IP/OOP-mode change, and that no file outside the pitch component and its owned test changed.

#### Commit 9 — Add persistent Tactical XI panel

**Status:** Pending

**Provisional commit:** `feat(tactics): add persistent Tactical XI panel`

**Work:** Add a persistent Tactical XI panel beside the pitch that lists all 11 lanes with their IP → OOP role transition and shares the editor's `selectedLaneId` in both directions.

**Size assessment:** About 120 changed non-test implementation lines (one new panel file plus editor wiring). Within the soft target.

**Out of scope:**

- Pitch rendering changes, inspector move, orientation switching, containment, persistence, and IPC changes.

**Implementation packet:**

- Add a selection-only panel with no parallel state and wire it to the existing selection ownership.

**Files and responsibilities:**

- `src/features/planner/components/planner-tactic-lane-list.tsx` (new) — lane list panel. Renders all 11 lanes in linear position order using `orderedTacticLanes` and `linkedPositionDescription`. Rows are ordinary buttons; the selected row uses `aria-pressed`, matching pitch markers. No listbox, no `aria-selected`, no `aria-current`. Row activation calls the editor's existing selection callback.
- `src/features/planner/components/planner-tactic-editor.tsx` — mount the panel beside the pitch and pass the existing `selectedLaneId` state and selection callback. No state, validation, or mutation change.
- `src/app/routes/my-club-squad.test.tsx` — extend tactic coverage for list↔pitch selection sync and the 11-row transition rendering.

**Behavior and data flow:**

- Panel rows read the same draft lanes as the pitch. Activating a row or a marker writes the same `selectedLaneId`; the inspector and highlight state follow unchanged. No new state or data flow.

**Ordered implementation steps:**

1. Add the failing selection-sync assertion and confirm RED.
2. Add the panel and editor wiring until the proof turns GREEN.
3. Refactor only while the focused proof stays green.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- New selection-sync assertions are the RED→GREEN proof: selecting a panel row (ordinary button with `aria-pressed`) updates the inspector and pitch marker state, and selecting a marker updates the panel row's pressed state. They fail without the shared wiring and pass with it.

**Patterns to verify:**

- Existing editor selection callbacks and the DESIGN micro-label pattern for the IP → OOP transition text.

**Constraints and non-goals:**

- No pitch, inspector-control, persistence, or responsive changes. The panel is a list of ordinary buttons, not an analytics surface and not a listbox; no scores, insight cards, summary statistics, `aria-selected`, or `aria-current`.

**Dependencies and sequencing:**

- Requires Commit 7 (single-canvas selection API). Blocks Commit 11 (panel occupies the beside-pitch composition the inspector joins).

**Validation:** `./scripts/dev test my-club-squad`, then `./scripts/dev check`.

**Stop conditions:** Stop if shared selection needs a second source of truth or a state-model change; selection must remain one `selectedLaneId` in the editor.

**Review mandate:** Verify single selection ownership, 11-row completeness in lane order, transition-text accuracy against `linkedPositionDescription`, keyboard operability of ordinary-button rows, accessible pressed state (`aria-pressed`, no listbox/`aria-selected`/`aria-current`), and that no pitch or persistence file changed.

#### Commit 10 — Expose persistent Selected Slot controls

**Status:** Pending

**Provisional commit:** `feat(tactics): expose persistent Selected Slot controls`

**Work:** Adapt the existing `PlannerTacticInspector` in its current location into the persistent **Selected Slot** inspector presentation: always expose both IP and OOP position/role controls regardless of pitch view (the ticket says the persistent inspector exposes all fields), rename the region/heading from Selected position settings to Selected Slot, and keep the IP/OOP weight visually associated with both phases. Position picks route through the Commit 4 swap contract. Reuse all existing callbacks, controls, validation, failed-save, and read-only behavior. Do not move it beside the pitch yet.

**Size assessment:** About 100 changed non-test implementation lines in the inspector presentation. Within the soft target.

**Out of scope:**

- Inspector relocation (Commit 11), pitch changes, XI panel changes, orientation switching, containment, persistence, and IPC changes.

**Implementation packet:**

- Adapt the inspector presentation in place; the editor composition does not move yet and control logic does not change.

**Files and responsibilities:**

- `src/features/planner/components/planner-tactic-inspector.tsx` — persistent Selected Slot presentation in place: both IP and OOP position/role controls always rendered regardless of pitch view, region/heading renamed from Selected position settings to Selected Slot, IP/OOP weight kept visually associated with both phases. All existing callbacks, controls, validation, failed-save, and read-only behavior reused; no relocation.
- `src/features/planner/components/planner-tactic-editor.tsx` — narrowly change the inspector's `phases={visiblePhases(view)}` to always both phases so the in-place Selected Slot presentation exposes IP and OOP controls in every pitch view. Callback wiring and control logic unchanged; relocation stays in Commit 11.
- `src/app/routes/my-club-squad.test.tsx` — focused route proof that both IP and OOP position/role controls render in every pitch view (IP, OOP, Both), plus updated accessible-name assertions for the Selected Slot heading.
- `e2e/smoke.spec.ts` — update only the existing inspector-region locator(s) owned by the rename (`Selected position settings` → `Selected Slot`); no layout or fit assertion changes here.

**Behavior and data flow:**

- Identical data flow: same selected lane, same draft updates, same validation error, same save mutation and invalidation. Only the inspector's control presentation changes: both phases always exposed, heading renamed, weight associated with both phases. Location and composition are untouched.

**Ordered implementation steps:**

1. Add the failing both-phases-in-every-view assertion and confirm RED.
2. Adapt the inspector presentation until the proof turns GREEN.
3. Refactor only while the focused proof stays green.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- The new both-phases-in-every-view route assertions plus updated accessible-name assertions are the RED→GREEN proof, with the preserved save-flow assertions proving no control or callback regressed. They fail when either phase's controls are missing in any view and pass with the persistent Selected Slot presentation. The updated smoke inspector-region locator(s) are part of the same RED→GREEN proof (proved via exact `./scripts/dev smoke`): they fail after the rename without the locator update and pass with it.

**Patterns to verify:**

- Existing Tailwind breakpoint usage in the editor and the DESIGN inspector-width token for the aside width.

**Constraints and non-goals:**

- No control-logic, validation, draft-retention, read-only, relocation, or mutation change. Presentation only: both phases exposed, heading renamed, weight associated with both phases.

**Dependencies and sequencing:**

- Requires Commit 7 (established canvas/view contract). Blocks Commit 11 (the now-complete inspector moves beside the pitch there).

**Validation:** `./scripts/dev test my-club-squad`, then exact `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop if exposing both phases in every view requires a callback, validation, or state-model change (report instead of improvising).

**Review mandate:** Verify both IP and OOP position/role controls render in every pitch view, the Selected Slot heading/accessible names are consistent, the weight control stays visually associated with both phases, every callback/validation/failed-save/read-only behavior is reused, nothing relocated, the updated smoke inspector-region locator(s) use Selected Slot with exact `./scripts/dev smoke` green, and the diff is inspector-presentation only plus its owned route test plus the rename-owned smoke locator(s).

#### Commit 11 — Compose the responsive tactic workspace

**Status:** Pending

**Provisional commit:** `feat(tactics): compose the responsive tactic workspace`

**Work:** Move the now-complete Selected Slot inspector beside the pitch AND guard normal desktop fit at 1280×800 and 1600×900 in one atomic packet: recompose the workspace grid so the pitch/XI area and the inspector aside sit side by side at wide breakpoints and stack below `lg`, with responsive stacking, min-width, and overflow ownership (vertical scrolling allowed at 1280×800, no horizontal overflow). The existing viewport contract must remain green in this commit; run full smoke. Do not include ultrawide containment or landscape orientation.

**Size assessment:** About 140 changed non-test implementation lines (editor recomposition plus responsive tightening). Within the soft target.

**Out of scope:**

- Ultrawide containment (Commit 12), landscape orientation (Commit 13), inspector control-presentation changes (Commit 10), pitch geometry changes, persistence, global style or token changes.

**Implementation packet:**

- Move the complete inspector beside the pitch and tighten responsive behavior to the tested normal-desktop fit contract in one atomic packet.

**Files and responsibilities:**

- `src/features/planner/components/planner-tactic-editor.tsx` — recompose the workspace grid (pitch/XI area plus inspector aside side by side at wide breakpoints, stacking below `lg`) and tighten responsive behavior (stacking, min-width guards, overflow ownership) so the fit matrix holds in portrait orientation. Keep the command bar and all inspector callbacks/`phases` wiring identical.
- `src/app/routes/my-club-squad.test.tsx` — update the command-bar/pitches/shelf order test to the beside-pitch composition and keep the preserved save-flow assertions proving no control or callback regressed.
- `e2e/smoke.spec.ts` — keep the existing `planner tactic workspace fits its supported desktop viewports` rows for 1280×800, 1600×900, and 1920×1080 green against the finished beside-pitch composition, plus layout-specific beside-pitch bounding-box assertions at the wide rows (1600×900 and 1920×1080, or the exact appropriate wide rows from the current fit test): the inspector bounding box sits beside the pitch/Tactical XI area (horizontally adjacent with vertical overlap), failing for a bottom-shelf inspector; no rename-locator changes here (owned by Commit 10); no new widths here.

**Behavior and data flow:**

- Composition and layout only. Same selected lane, same draft updates, same validation error, same save mutation and invalidation. Only the visual placement and responsive fit change.

**Ordered implementation steps:**

1. Update the layout-order assertion to the beside-pitch composition and extend the existing smoke fit rows with the beside-pitch bounding-box assertions; confirm RED where placement is bottom-shelf or fit overflows or breaks.
2. Recompose the editor grid and add the responsive tightening until the proof turns GREEN (bounding-box beside-pitch geometry plus fit rows).
3. Refactor only while the focused proof stays green.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- The updated layout-order route test plus the existing smoke fit rows for 1280×800 (vertical scroll allowed, no horizontal overflow), 1600×900, and 1920×1080, plus the beside-pitch bounding-box assertions comparing the inspector to the pitch/Tactical XI area at 1600×900 and 1920×1080 (or the exact appropriate wide rows from the current fit test), are the RED→GREEN proof (proved via exact `./scripts/dev smoke`): the bounding-box assertions fail on the bottom-shelf layout and pass on the beside-pitch composition, while the retained no-horizontal-overflow assertions (all rows) and vertical-fit assertions (1600×900, 1920×1080) stay green. No new specs; the rows are owned here.

**Patterns to verify:**

- Existing Tailwind breakpoint usage in the editor and the DESIGN inspector-width token for the aside width.

**Constraints and non-goals:**

- Do not touch ultrawide widths, orientation switching, the global `content-max-width: none` token, `src/styles/global.css` tokens, Squad surfaces, or shell layout. Vertical scroll at 1280×800 is allowed; horizontal overflow is not.

**Dependencies and sequencing:**

- Requires Commits 9 and 10 (panel occupies the beside-pitch area; inspector controls are complete). Blocks Commits 12 and 13 (containment constrains and landscape layers onto this composition).

**Validation:** `./scripts/dev test my-club-squad`, then `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop if 1920×1080 cannot fit without shrinking markers below readability (escalate density options), if the fit needs global style changes (report instead of leaking scope), or if the beside-pitch bounding-box assertions cannot fail on the bottom-shelf layout while keeping the no-horizontal-overflow and vertical-fit assertions green (report instead of weakening the proof).

**Review mandate:** Verify the inspector moved with every control and callback untouched, the beside-pitch bounding-box assertions prove the inspector sits beside the pitch/Tactical XI area at 1600×900 and 1920×1080 (or the exact appropriate wide rows), 1280 horizontal-fit with allowed vertical scroll, clean 1600/1920 fit, stacking and min-width behavior, no rename-locator changes, no global style change, and that the diff is editor-composition plus owned tests only.

#### Commit 12 — Contain the tactic workspace on ultrawide

**Status:** Pending

**Provisional commit:** `feat(tactics): contain tactic workspace on ultrawide`

**Work:** Add tactic-workspace-only ultrawide containment and prove bounded/centered behavior at 3440×1440. Do not touch global max-width, Squad, shell, orientation, or normal desktop composition beyond what containment requires. Use the existing smoke fit test.

**Size assessment:** About 40 changed non-test implementation lines (workspace wrapper). Within the soft target.

**Out of scope:**

- Normal desktop fit (Commit 11), landscape orientation (Commit 13), pitch geometry changes, control changes, persistence, global styles, and Squad or shell changes.

**Implementation packet:**

- Scope containment to the tactic workspace block and prove it at the ultrawide width.

**Files and responsibilities:**

- `src/app/routes/my-club.tsx` — constrain and center only the tactic workspace block (the `hidden={activeWorkspace !== "tactic"}` container) at ultrawide widths. No shell, Squad, or Planner surface changes.
- `e2e/smoke.spec.ts` — extend `planner tactic workspace fits its supported desktop viewports` with 3440×1440 containment assertions (workspace bounded and centered, no indefinite stretch).

**Behavior and data flow:**

- Layout only. No state, callback, validation, or persistence change. Containment is a workspace-scoped style boundary.

**Ordered implementation steps:**

1. Extend the smoke fit matrix with the 3440×1440 containment assertions and confirm RED.
2. Add the workspace-scoped containment until the proof turns GREEN.
3. Refactor only while the focused proof stays green.
4. Run targeted and commit-level validation in the recorded order.

**Tests and proof:**

- The extended smoke fit matrix row for 3440×1440 (proved via `./scripts/dev smoke`) is the RED→GREEN proof: it fails on the unconstrained workspace (stretch or horizontal overflow) and passes with scoped containment. Existing viewport rows guard against regressions.

**Patterns to verify:**

- Existing `hidden`-prop workspace preservation in `my-club.tsx` and Tailwind arbitrary or `min-*` breakpoint variants already used in the codebase.

**Constraints and non-goals:**

- Do not touch the global `content-max-width: none` token, `src/styles/global.css` tokens, Squad surfaces, shell layout, orientation behavior, or normal desktop composition beyond what containment requires.

**Dependencies and sequencing:**

- Requires Commit 11 (final beside-pitch composition to constrain). Blocks nothing; Commit 13 follows independently.

**Validation:** `./scripts/dev test my-club-squad`, then `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop if containment leaks onto Squad or global styles, or if headless Chromium clamps the 3440 viewport (report a validation gap instead of faking the assertion).

**Review mandate:** Verify containment scoping (tactic workspace only), centering behavior, no global style change, smoke assertion quality, and that no behavior file changed.

#### Commit 13 — Orient the pitch landscape at wide viewports

**Status:** Pending

**Provisional commit:** `feat(tactics): orient the pitch landscape at wide view`

**Work:** Activate the real 90-degree landscape pitch orientation at viewport widths >= 1920px in the workspace phase-aware wrapper using the Commit 2 projection with SVG/HTML pitch markings and connectors. Portrait attack-up orientation stays below 1920px. Marker text stays upright; the DOM is never rotated. Keyboard/DOM order and attack direction stay understandable and accessible in both orientations. The `matchMedia` orientation source belongs only to the workspace phase-aware component, never the modal or shared single-phase component.

**Size assessment:** About 150 changed non-test implementation lines (projection wiring, markings/connectors, orientation styles, accessible attack-direction semantics). Within the soft target.

**Out of scope:**

- Control, persistence, IPC, containment, global style, and dependency changes. No new hook file.

**Implementation packet:**

- Wire the tested projection into the canvas and prove orientation through geometry assertions, not snapshots.

**Files and responsibilities:**

- `src/features/planner/components/planner-phase-aware-tactic-pitch.tsx` — own the single orientation source with one colocated `matchMedia("(min-width: 1920px)")` read, change subscription, and cleanup (no dependency, no shared hook); select portrait or landscape projection by that source and pass the active orientation into the shared canvas logic. The shared portrait single-phase component gains no orientation source.
- `src/features/planner/components/planner-tactic-pitch.tsx` — draw orientation-aware SVG/HTML markings and Both-mode connectors in projected space from the wrapper-supplied orientation, keep marker labels upright with no rotated DOM, render an explicit accessible direction description (portrait attack-up, landscape attack-right via clockwise rotation), and compute DOM/tab order from the current visual pitch order in each orientation (stable left-to-right/top-to-bottom with deterministic ties), never raw lane IDs.
- `src/features/planner/components/planner-tactic-editor.tsx` — orientation-responsive arrangement only if the wrapper cannot own it alone; prefer wrapper-local changes.
- `src/features/planner/components/planner-role-reference-modal.tsx` — deliberately unchanged: no orientation source, portrait single-phase only; its existing route/smoke coverage is retained as proof (same focused `my-club-squad` route test where applicable).
- `e2e/smoke.spec.ts` — landscape assertions in the tactic workspace test (proved via `./scripts/dev smoke`): initial 1920px load plus live crossing 1919↔1920, attack direction description (attack-up versus attack-right), projected marker geometry (width/height relationship inverts versus portrait), upright marker labels with no rotated DOM, and tab/DOM order matching the current visual pitch order in each orientation.

**Behavior and data flow:**

- The workspace wrapper reads the active orientation from its colocated `matchMedia` source; the canvas derives each marker's portrait coordinate from the direct qualified-placement table over the current `phasePosition(lane, phase)`, maps it through `projectPosition` with that orientation, and renders markings, markers, and connectors from projected coordinates. Both-mode connectors compare canonical full qualified placement identity. Selection, highlight, draft, validation, and save flow are untouched. The modal path never subscribes to orientation.

**Ordered implementation steps:**

1. Add the failing landscape geometry assertions and confirm RED against portrait-only output.
2. Wire the projection and orientation-aware markings until the proof turns GREEN.
3. Confirm portrait assertions still pass below 1920px.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- The landscape smoke assertions (proved via `./scripts/dev smoke`) are the RED→GREEN proof: they fail when the canvas renders portrait geometry at >= 1920px and pass with the landscape projection. They cover initial 1920px load, live 1919↔1920 crossing, attack direction, projected marker geometry, upright labels with no rotated DOM, and tab/DOM order. Portrait-mode assertions from earlier commits guard the below-1920 contract. Pure geometry unit tests prove projection/order only, not upright labels. No visual snapshot infrastructure.

**Patterns to verify:**

- The Commit 2 projection contract, existing `LaneButton` focus/hover semantics, and Tailwind responsive variants for orientation layout.

**Constraints and non-goals:**

- No CSS-transform rotation of the DOM. No new dependency and no shared hook file: the single colocated `matchMedia("(min-width: 1920px)")` source lives inside the pitch component with initial read, change subscription, and cleanup. DOM/tab order derives from the current visual pitch order in each orientation, never raw lane IDs.

**Dependencies and sequencing:**

- Requires Commits 2 (projection), 8 (connectors), and 11 (fit foundation). Last implementation commit.

**Validation:** `./scripts/dev test my-club-squad`, then exact `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop if the projection cannot keep marker text upright, if keyboard order diverges from visible order in landscape, if attack direction is not perceivable to assistive technology in both orientations, or if a shared hook or dependency looks required (report instead of adding it).

**Review mandate:** Verify the single colocated `matchMedia` source lives only in the workspace wrapper (subscription/cleanup, no new hook or dependency; modal and shared single-phase component have no orientation source), projection wiring, marking/connector geometry in both orientations (canonical connector identity), absence of DOM rotation, upright marker text, tab/DOM order matching the current visual pitch order in each orientation, explicit attack-direction description (attack-up versus attack-right), portrait compatibility, retained modal route/smoke coverage, and that selection and save behavior are untouched.

## Active work

**PR:** 1

**Commit:** 6

### RED or removal proof

Rewrite the smallest route geometry assertion for the unique-placement normalized canvas; remove the retired duplicate/overflow geometry contracts with their implementation. Prove the preserved save flow through the existing smoke test.

### Expected outcome

Reusable portrait single-phase canvas with qualified-placement coordinates, unchanged board composition and modal API, and no duplicate/overflow machinery.

### Explicit exclusions

One-canvas consolidation, connectors, XI panel, inspector changes, orientation, containment, persistence, IPC, and Rust.

## Discoveries and replanning

- 2026-09-05: delivery resumed under fingerprint `bd0767c438e788e0db1560643e56bfa8773de559f7914bd35a0a46b4db9ae84c`. The developer explicitly approved discarding the abandoned canvas attempt. The safety tool required preserving those exact three paths in a recovery stash; they were removed from the worktree without reapplying them. Commit 4 started from clean HEAD. Its sole review finding was resolved by replacing the canonical self-selection test with a distinct-lane ST/STC occupant swap proof; literal occupancy matching makes that test fail.

- 2026-09-05: the developer accepted the reviewed replan and explicitly requested its commit before deciding whether to discard the abandoned implementation. Commit 3 therefore records only the ledger and TODO; the three source files are preserved unstaged, pending separate discard approval. This changes the planning-recording order only. No new implementation, publication, or delivery authority is inferred.

- 2026-09-05: fresh plan review and correction review cleared all six blocking packet findings; a final evidence check verified the complete ledger and TODO diff. Recommendation: Accept, with the recorded MEDIUM and NITPICK advisories still open. The revised fingerprint remains provisional until developer acceptance and a new delivery invocation. No implementation or Git mutation was performed during replanning.

- 2026-09-05: bounded correction round 1 (six HIGH findings, readiness only — no new acceptance or delivery authority). Commit 4 explicitly replaces the obsolete duplicate-error route test with swap proof; Commit 5 grounds the reset proof in a v41→v42 migrations.rs test beside the v36 analogue; Commit 6 owns the `tactic-editor.ts` duplicate/overflow cleanup, the five named HEAD route-test removals, and the shared `canonicalPlacement` export consumed by Commit 8; Commit 10 narrows the editor change to always-both-phases for the inspector; Commit 6 no longer orders the worker to restore files. Open advisory notes (recorded, not fixed in this round): MEDIUM stale no-Rust/no-migration claims and intended-worktree TODO gaps; MEDIUM Commit 13 optional editor-ownership line despite the wrapper owning orientation; NITPICK duplicated reset phrasing; NITPICK overstated 'could not be fixed' wording in the replan entry.
- 2026-09-05: material replan on explicit developer authority. The user rejects draft-overflow and legacy-overcapacity support (the five-AMC cross-band overlap could not be fixed within the old contract) and authorizes unique-placement swaps plus discarding existing tactics. Consequences: the uncommitted overflow canvas work in `planner-tactic-pitch.tsx`, `my-club-squad.test.tsx`, and `e2e/smoke.spec.ts` is abandoned (restore to HEAD before resuming); new Commit 4 establishes the edit-time swap contract, new Commit 5 runs the one-time reset migration and removes dead legacy normalization, and new Commit 6 rebuilds the canvas for unique placements only. Packet sequence grows from ten to thirteen commits on the same single PR/branch/base/squash boundary. Completed Commits 1–2 and their proofs are preserved unchanged; the accepted fingerprint `0404c69e…` is invalidated and replaced provisionally pending fresh review and developer acceptance.

- 2026-09-05: the developer challenged the six-implementation-commit breakdown as not the largest sensible atomic split and prefers more small, manageable commits. Replanned before acceptance into nine implementation commits (ten commits including planning): behavior-preserving normalized-canvas migration, one-canvas consolidation, Both-mode transitions, XI panel, inspector move, normal desktop fit, ultrawide containment, landscape orientation. Scope, decisions, geometry/connector/orientation contracts, publication fields, minimalism, and validation standards are unchanged. The Delivery fingerprint is pending a new complete review.
- 2026-09-05: bounded correction after a fresh complete review, recorded without widening scope. It resolves four reviewer findings while keeping nine implementation commits and one PR: (a) HIGH shared pitch consumer — `PlannerTacticPitch` is also consumed by `planner-role-reference-modal.tsx`, so Commit 3 retains it as the reusable portrait single-phase component/API with the modal file deliberately unchanged, Commit 4 introduces the workspace-only `PlannerPhaseAwareTacticPitch` wrapper composing the same canvas logic with only the editor switching, and Commit 10 scopes `matchMedia` to the wrapper only; the modal stays portrait single-phase and is explicitly outside JAY-57; (b) HIGH nondeterministic Commit 5 smoke — Commit 5 now requires exact `./scripts/dev smoke`, extending the existing tactic smoke flow with connector visibility/geometry assertions for a canonical changed placement and a legacy-equivalent placement; (c) HIGH Commit 7/8 split not trunk-safe — new Commit 7 exposes the persistent Selected Slot controls in place (both-phase controls in every view, renamed heading, weight associated with both phases) and new Commit 8 moves the complete inspector beside the pitch with the 1280×800/1600×900 fit guards in one atomic packet, with Commit 9 (ultrawide) and Commit 10 (landscape) unchanged in scope; (d) MEDIUM redundant full frontend runs — Commit 3 and Commit 10 use focused `./scripts/dev test my-club-squad`, exact `./scripts/dev smoke` where browser behavior matters, then `./scripts/dev check`. Packet responsibilities and order changed, so the Delivery fingerprint stays pending a new complete review.
- 2026-09-05: bounded correction keeping nine implementation commits and one PR: (a) HIGH Commit 7 trunk-safety — Commit 7 lists `e2e/smoke.spec.ts` for the rename-owned inspector-region locator(s) (`Selected position settings` → `Selected Slot`) with exact `./scripts/dev smoke` before `./scripts/dev check`, and Commit 8 keeps only layout-specific smoke changes; (b) HIGH Commit 8 beside-pitch proof — its viewport-smoke packet gains beside-pitch bounding-box assertions (inspector beside pitch/Tactical XI area at 1600×900 and 1920×1080) that fail for a bottom-shelf inspector while retaining no-horizontal-overflow and vertical-fit assertions. Scope, architecture, commit order, and nine implementation commits unchanged; Delivery fingerprint stays pending a new complete review.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Redesign the tactic workspace | Commit 1 — Record the approved feature plan | 119b51910fa2eaea3163f9716a68be933356cc1e | Recorded accepted ledger and active TODO link. | Ledger and delivery classifiers passed; check-fast and staged whitespace check passed. | Not applicable | Clear | 0 | None. |
| PR 1 — Redesign the tactic workspace | Commit 2 — Add orientation-aware pitch geometry projection | 8d9e142fa4000aa2c31ebc93f4e96d9d3b70b15a | Added pure coordinate projection, supported-placement table, and visual ordering without UI wiring. | Expected missing-module RED; focused 7/7 GREEN; full check and staged whitespace passed. | Pass | Clear | 0 | Worker accidentally applied an unrelated retained stash; developer restored the exact eight affected tracked paths. Verified recovery before validation and review. |
| PR 1 — Redesign the tactic workspace | Commit 3 — Record the approved replan | e47d709 | Recorded the reviewed unique-placement swap and one-time reset plan with the revised commit sequence and TODO summary. | Ledger and delivery classifiers passed; check-fast and staged whitespace check passed. | Not applicable | Accepted findings — independent plan review cleared blockers; recorded MEDIUM and NITPICK advisories remain open; developer accepted the reviewed plan. | 1 | Developer requested planning commit before source-discard decision; abandoned implementation preserved unstaged. |
| PR 1 — Redesign the tactic workspace | Commit 4 — Enforce unique-placement swaps in draft editing | edf6173f8a3817aeb0ed7354f64d116b78ed4160 | Added pure phase-placement swaps and wired the editor; preserved compatible roles, cleared incompatible roles, and replaced obsolete duplicate-rejection proof. | Unit 6/6, route 133/133, full check including 799 Rust tests passed; canonical literal-match mutation failed as expected; staged whitespace clean. | Pass | Clear | 1 | None. |
| PR 1 — Redesign the tactic workspace | Commit 5 — Reset stored tactics to defaults once | Pending record | Registered data-only v42 tactic reset; removed legacy normalization and its two load tests. | Migration 60/60 and tactic 14/14 tests; check-rust and full check including 798 Rust tests passed; staged whitespace clean. | Pass | Clear | 0 | Older v8/v28 migration tests now prove their surviving contracts without asserting tactic preservation after the authorized reset; orphaned normalization constant removed. |

## Final validation

- `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/tactics-page-redesign.md` — ledger structural consistency.
- `./scripts/dev test` — full frontend suite, including the new projection and swap unit tests, reworked route assertions, and the retained `tactic-context-boundary.test.tsx`.
- `./scripts/dev smoke` (after `pnpm exec playwright install chromium` if needed) — full Playwright product suite, including the reworked tactic save test, the extended viewport fit matrix (1280×800, 1600×900, 1920×1080, 3440×1440), and landscape geometry assertions.
- `./scripts/dev check` — full commit gate (Biome, TypeScript, secretlint, Rust format, Clippy, Rust tests; includes the reset-migration backend tests).
- `git diff --cached --check` plus staged-diff inspection before every commit.
- Manual viewport pass at 1280×800, 1600×900, 1920×1080, and 3440×1440 where a matching display is available; report as a gap where headless-only.

## Documentation impact

No documentation change in this planning commit beyond the ledger and TODO state. Implementation commits in this PR must not edit `.wiki/ARCHITECTURE.md` or `.wiki/DESIGN.md`: both describe the old two-pitch layout (dual boards, bottom settings shelf) and go stale while the redesign lands. Reconciliation of the Planner tactic, pitch geometry, and selected-position settings sections happens at feature close-out after behavior exists. No ADR is warranted; no debug report is expected.
