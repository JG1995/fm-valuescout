# Refine Suggested Training

## Status

Ready for final publication

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 6ef35da0bbe72779f92dabeeb1a352f4172d2bd0dca6c7203bfb7aaf92438346

Ledger state does not create or switch a branch. The recorded branch is intent only until an explicit `/skill:workflow-deliver-feature` invocation has a reviewed, accepted Delivery fingerprint.

## Intent

Refine the read-time Suggested Training recommendation so its focus matches a developing player's current age category. Keep the existing assigned-lane precedence, fallback-lane selection, and gain ranking. Return no suggestion for players who cannot develop or whose age is unknown or 29 or older (Linear [JAY-59](https://linear.app/jaycount/issue/JAY-59/refine-suggested-training)).

## User-visible behavior

- Suggested Training remains a Squad-only, read-only, non-sortable `string | null` DTO. The existing cell renders `null` as an accessible em dash.
- No suggestion is returned when `CA >= PA`, age is `null`, or age is 29 or older.
- Players under 21 rank Physical focuses only. Players aged 21 through 24 rank Technical focuses only. Players aged 25 through 28 rank Mental focuses only.
- The applicable outfield or goalkeeper inventory is selected first. Ranking then considers only focuses in the player's age category.
- Assigned lanes still take precedence. Eligible unassigned players still use the existing best-current-lane fallback.
- Missing attributes used only by ineligible focuses do not prevent a suggestion. Missing selected-category focus attributes or selected-lane role attributes return no suggestion.

## Invariants

- Per-player CA/PA and age gates run before assigned-lane lookup, fallback-lane selection, fallback-input loading, and training ranking.
- Preserve the existing +1 simulation, unrounded IP/OOP gain comparison, lane IP weight, strict inventory-order tie-break, outfield/GK lane selection, and best-current-lane fallback rule.
- Focus categories are exact:
  - Physical: Quickness; Agility and Balance; Strength; Endurance.
  - Technical: Free Kick Taking; Corner Taking; Penalty Taking; Long Throws; Shooting; Passing; Crossing; Ball Control; Aerial; GK Technique; GK Distribution (Long); GK Distribution (Short).
  - Mental: Defensive Positioning; Attacking Movement; Final Third; GK Reactions; GK Tactical; GK Sweeping.
- Suggestions remain derived during bounded `list_squad_players` reads. They create no persistence, migration, cache, IPC command, or mutation.
- The frontend DTO and table behavior remain unchanged.

## Non-goals

- No UI, DTO, schema, migration, table-layout, sort, training-action, category-display, selector, or evidence-UI change.
- No change to CA, PA, potential projection, tactic configuration, Planner assignments, or fallback-lane selection.
- No change to the existing focus names, attribute mappings, inventory order, or ranking math beyond category filtering.
- No ADR or BACKLOG change.

## Current-state map

- Relevant components:
  - `src-tauri/src/features/planner/suggested_training.rs` owns tuple-based outfield and goalkeeper focus inventories and the pure all-applicable-focus ranker. It currently requires every applicable inventory attribute, then simulates +1 and compares unrounded lane-weighted gains.
  - `src-tauri/src/features/planner/squad.rs` maps `players.age` to `SquadPlayer.age: Option<i64>`, derives suggestions during each bounded Squad page read, gates only `CA >= PA`, and loads fallback inputs for unassigned developing rows.
  - `src-tauri/src/features/planner/squad_tests.rs` covers assigned-lane precedence, best-current-lane fallback, CA/PA suppression, missing attributes, and fallback read constraints. `planner/test_support.rs::set_player_age` already sets nullable fixture ages.
  - `src/features/squad/types/squad-player.ts` and `squad-overview-panel.tsx` already mirror `suggestedTraining: string | null` and render its null as an accessible em dash. They are not changed.
  - `.wiki/ARCHITECTURE.md` currently records read-time Suggested Training, assigned-lane precedence, fallback selection, and the string-or-null contract. Commit 3 must extend that implemented-state description when the behavior lands.
- Data model: `players.age` is nullable and is already selected on every Squad row. `players.attributes_json`, current role metrics, tactic lanes, and Planner assignments remain existing read inputs.
- Persistence and migrations: none. Suggestions are computed per page request and never stored.
- Existing behavioral assumptions: a player assignment is unique per save; `lane_id == "goalkeeper"` selects the goalkeeper inventory; a missing required ranking input returns `None`; fallback reads current role metrics, position familiarity, preferred foot, and tactic order.
- Architectural seams: `list_squad_players` → `suggestion_for_player` → `suggest_for_lane`; the page first identifies fallback candidates, then loads their existing fallback inputs in one query.
- Project validation commands: `./scripts/dev check-rust` runs Rust format, Clippy, and Rust tests. `./scripts/dev check` adds the repository application gate. The CI `check` status runs the applicable Rust job for these source changes.
- Primary risks: a duplicated age mapping could drift; applying missing-attribute checks before category filtering could suppress valid suggestions; and omitting the new gates from the fallback candidate filter would parse or load fallback inputs for ineligible rows.

## Feature architecture

- `suggested_training.rs` will hold the smallest planner-visible category type and private typed focus catalog that associates each existing focus name and attribute mapping with one category. The current ranker remains the single ranking path.
- One age-to-category decision returns no category for null or 29-or-older ages. `squad.rs` uses that decision before either assignment or fallback lane work, and the ranker receives the selected category to restrict both evaluability and simulation.
- `squad.rs` retains ownership of page composition, CA/PA gating, assignment precedence, and fallback-input batching. It excludes all age-ineligible unassigned rows from fallback-input loading.
- Rust remains the sole business-rule owner. Existing React rendering consumes the unchanged nullable string and needs no duplicate tests.

## Uncertainty register

### Known

- `main` is clean at planning start. No active ledger or planned feature specification exists for this feature.
- `9c77ca2` added the current unassigned best-current-lane fallback after the earlier Suggested Training feature. That fallback is now part of the behavior this feature preserves.
- Linear JAY-59 and the developer-accepted null-age rule define the category map and gates. Current source provides the existing ranking and lane seams, but it lacks age categories and age gating.
- One PR is sufficient: both implementation commits are trunk-safe and form one narrowly related Rust read-model change. No ADR meets the repository threshold.

### Assumptions

- The existing full-attribute fixture and `set_player_age` helper can express the category and nullable-age contracts without new test infrastructure.
- The existing `./scripts/dev check-rust` gate discovers the inline planner and `squad_tests.rs` tests. There is no stable `scripts/dev` command for selecting one Rust test by name.

### Decisions

- Retain the approved two-commit implementation split. The typed catalog is a behavior-preserving, independently provable foundation; applying categories and gates is the complete behavior change.
- Make category metadata planner-visible only where the existing Squad-to-ranker seam needs it. Do not introduce a second ranker or a temporary duplicate API.
- Keep the existing inventory order as the tie-break within each category after filtering. Category membership does not reorder focuses.
- Gate null age as no suggestion. `Option<i64>` already represents this state and no sentinel is introduced.

### Unknowns

- None requiring a developer decision.

### Risks

- A review may find that the typed catalog changes existing unrestricted ranking. Preserve its exact test outcomes before Commit 3 changes behavior.
- A category filter that checks all inventory attributes first would violate the approved strict eligible-focus rule. Tests must prove an ineligible missing key does not suppress a result.
- A late age gate could leave ineligible unassigned players in `load_fallback_inputs`. Page-level tests must prove their exclusion.

## Walking skeleton

1. Convert the focus inventories into a categorized catalog while proving unrestricted ranking has not changed.
2. Apply one age-category decision before lane selection, load fallback inputs only for eligible unassigned players, and return the existing focus-or-dash result.

## Delivery plan

### PR 1 — Refine Suggested Training

**Status:** Ready for publication

**PR ref:** https://github.com/JG1995/fm-valuescout/pull/138

**Merge ref:** Not merged

**Branch:** `feature/refine-suggested-training`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict required GitHub Actions status `check`

**Feature close-out:** Current

**CI repair rounds:** 0

**Provisional PR title:** `feat(training): tailor suggested training to player age`

**Purpose:** Deliver categorized focus ranking and age-gated Squad suggestions in one reviewable, trunk-safe PR without changing the existing UI or data contract.

**Depends on:** None.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(training): record approved feature plan`

**Work:** Commit the independently reviewed ledger and TODO state on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, BACKLOG, architecture, ADRs, and Git state outside the fingerprint-authorized branch activation.

**Implementation packet:**

- Preserve the accepted review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/refine-suggested-training.md` — reviewed feature intent, current-state evidence, delivery boundary, and execution packets.
- `.wiki/TODO.md` — Active feature link and Linear JAY-59 link while preserving the existing Next item.

**Behavior and data flow:**

- Record one reviewed source of active feature intent before code changes. No runtime behavior changes.

**Ordered implementation steps:**

1. Verify the authorized branch and `main` base without changing Git state.
2. Confirm that only the reviewed ledger and TODO paths are selected.
3. Run the ledger classifier.
4. Inspect the exact planning diff before the independent checkpoint review and commit.

**Tests and proof:**

- Not applicable — independently reviewed planning documents only. The ledger classifier proves schema and active-work consistency. No repository documentation validator is configured.

**Patterns to verify:**

- `.wiki/features/active/README.md` schema 2 template and the current TODO Active/Next ownership pattern.

**Constraints and non-goals:**

- Do not change implementation, tests, configuration, BACKLOG, architecture, or plan decisions. Do not treat the ledger as branch authority.

**Dependencies and sequencing:**

- Requires cleared plan review, developer acceptance, a Delivery fingerprint, and exact branch activation before any commit.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/refine-suggested-training.md`

**Stop conditions:** Stop for an uncleared review, classifier error, unreviewed path, branch mismatch, or substantive post-review plan change.

**Review mandate:** Verify that the selected diff contains only the complete reviewed ledger and TODO outcome, preserves the Next item, and grants no implementation or Git authority.

#### Commit 2 — Classify suggested training focuses

**Status:** Completed

**Provisional commit:** `refactor(training): classify suggested training focuses`

**Work:** Replace tuple-only focus definitions with the smallest categorized focus metadata while preserving unrestricted ranking behavior.

**Size assessment:** Small Rust-only catalog refactor, within the soft target.

**Out of scope:**

- Age gates, category filtering, Squad page changes, fallback-input changes, persistence, DTOs, UI, and architecture documentation updates.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/planner/suggested_training.rs` — replace tuple-only entries with the minimum private/planner-visible typed category metadata; annotate the existing outfield and goalkeeper inventories; keep one existing ranking path; update nearby inline tests.

**Behavior and data flow:**

- The ranker still selects from every applicable focus and returns the same suggestion as before. The new catalog records category membership only; it does not filter, alter mappings, or change the public result.

**Ordered implementation steps:**

1. Add RED assertions for exact focus name, inventory order, mapped attributes, and category membership for both inventories, alongside existing unrestricted ranking outcomes.
2. Introduce the smallest typed catalog representation and migrate both inventories without creating a duplicate ranker or temporary API.
3. Keep current inventory selection, evaluability, simulation, gain comparison, and tie-breaking behavior unchanged.
4. Run the Rust gate.

**Tests and proof:**

- Inline `suggested_training.rs` tests prove the exact category map, existing focus names/order/attribute mappings, GK/outfield inventory selection, and a preserved unrestricted ranking result. They fail for a wrong category, mapping, order, or changed ranker result.
- No React or IPC test changes: this commit has no observable DTO or UI change.

**Patterns to verify:**

- The existing `suggest_for_lane` strict inventory evaluation, strict `>` tie-break, and all-zero behavior.

**Constraints and non-goals:**

- Keep metadata private or planner-visible only as required by the existing seam. Do not add configuration, category display, a second ranking entry point, or speculative extensibility.

**Dependencies and sequencing:**

- Follows the planning-artifact commit. Commit 3 consumes this catalog unchanged.

**Validation:** `./scripts/dev check-rust`

**Stop conditions:** Stop and replan if preserving unrestricted ranking requires duplicate catalog data or a new outward API, or if any existing mapping/order test changes without an approved product reason.

**Review mandate:** Verify the exact three-category map, both inventories' existing names/order/keys, unchanged all-focus ranking behavior, no widened visibility beyond the planner seam, and no unrelated changes.

#### Commit 3 — Tailor suggestions to player age

**Status:** Completed

**Provisional commit:** `feat(training): tailor suggestions to player age`

**Work:** Apply the approved age gates and category-filtered ranking to the read-time Squad suggestion while preserving assigned and fallback lane behavior.

**Size assessment:** Small Rust read-model change plus focused tests and intrinsic architecture documentation; within the soft target.

**Out of scope:**

- UI or DTO changes, migrations, persistence, IPC commands, sorting, table layout, training actions, category display, CA/PA/projection changes, tactic configuration, Planner assignments, and fallback-lane selection changes.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/planner/suggested_training.rs` — expose one age-to-category decision through the planner seam and filter evaluability and +1 simulation to the selected category while retaining the established ranking rule; its inline `#[cfg(test)]` module owns pure engine boundary, category, and missing-attribute tests.
- `src-tauri/src/features/planner/squad.rs` — use the single decision before assigned-lane or fallback work; preserve `CA >= PA` suppression; return `None` for null or 29-or-older age; exclude every ineligible unassigned row from fallback-input loading.
- `src-tauri/src/features/planner/squad_tests.rs` — own page-level integration and early-ordering/fallback proofs. Reuse the existing `set_player_age` helper without changing test infrastructure.
- `.wiki/ARCHITECTURE.md` — update the implemented Squad read-path description with age gates, category-filtered ranking, and early fallback exclusion when this behavior is implemented.

**Behavior and data flow:**

- For each mapped Squad row, CA/PA and nullable-age eligibility resolve before lane choice. If no category is eligible, the row returns `None` without assignment lookup, fallback selection, ranking, or fallback-input loading. An eligible row retains its assigned lane or existing best-current fallback, selects the lane's outfield/GK inventory, filters it to one category, checks only selected-category and lane-role attributes, and returns the unchanged focus string or `None`.

**Ordered implementation steps:**

1. Add RED pure-engine proof in `suggested_training.rs` for the category boundaries and strict selected-category evaluability, then add page-level proof in `squad_tests.rs` for the gates and early fallback exclusion.
2. Add an isolated CA/PA-gate fixture for an assigned player with `CA >= PA`, an age-eligible value, and an allowed nonblank unknown `planner_assignments.lane_id`. Assert `suggested_training: None`, not `Unknown tactic lane`.
3. Add isolated age-gate fixtures for assigned developing players with `CA < PA`, allowed nonblank unknown `planner_assignments.lane_id` values, and the two supported age-ineligible states: `null` and 29. Assert each returns `suggested_training: None`, not `Unknown tactic lane`.
4. Add an isolated early-fallback-load fixture for an unassigned developing player with `CA < PA`, an age-ineligible value, and corrupt fallback-only input. Assert it returns `None` without loading or parsing that input.
5. Add the single age-to-category decision and thread its result through the existing Squad-to-ranker seam. Do not duplicate age ranges.
6. Filter ranker input before evaluability and simulation. Keep +1 behavior, unrounded gain, lane weight, and strict inventory-order tie-break unchanged.
7. Restrict `fallback_uids` to rows that pass both CA/PA and age eligibility, then preserve existing assigned and fallback results for eligible controls.
8. Update the current-state architecture sentence with the shipped rule and run the recorded validation.

**Tests and proof:**

- Inline `suggested_training.rs` tests prove ages 20/21, 24/25, and 28/29; 29 and null age return no category; and outfield and goalkeeper fixtures select only their applicable category.
- Inline ranking tests prove a missing attribute in an ineligible category still produces a suggestion, while a missing selected-category focus attribute or selected-lane role attribute returns `None`.
- `squad_tests.rs` proves page-level integration: eligible assigned-lane precedence and eligible unassigned fallback return the expected category-valid focus. Its isolated poisoned fixtures prove the CA/PA gate with an assigned player that has `CA >= PA`, an age-eligible value, and an allowed nonblank unknown `planner_assignments.lane_id`; the age gate with assigned developing players that have `CA < PA`, the direct page-seam states `null` and 29, and allowed nonblank unknown assignment lanes; and early fallback exclusion with an unassigned developing age-ineligible player that has corrupt fallback-only input. Each gate fixture returns `suggested_training: None`, not `Unknown tactic lane`; the fallback fixture returns `None` without loading or parsing its corrupt input.
- Existing `set_player_age` is deliberately retained because it models the nullable active-snapshot field. Keep the null and 29 page-seam rows in the same focused gate proof; do not add redundant React or separate page tests for pure engine boundaries.

**Patterns to verify:**

- `suggestion_for_player`, `fallback_uids`, `load_fallback_inputs`, `best_fallback_lane`, and the existing write-denial and fallback test helpers in `squad_tests.rs`.

**Constraints and non-goals:**

- Preserve string-or-null DTO serialization and existing em-dash rendering without editing frontend files. Preserve bounded paging and one batched fallback-input query. Do not load fallback data for an ineligible row, persist a suggestion, or alter fallback ranking.

**Dependencies and sequencing:**

- Requires Commit 2's categorized catalog. This is the final implementation commit in the sole PR.

**Validation:** `./scripts/dev check-rust` followed by `./scripts/dev check`

**Stop conditions:** Stop and replan if category filtering cannot remain in the pure Rust ranker, a gate cannot run before lane/fallback work, preservation requires a DTO or frontend contract change, or current fallback behavior proves incompatible with the approved early-exclusion rule.

**Review mandate:** Verify that CA/PA, null age, and 29+ gates precede assigned-lane lookup and fallback work. The isolated poisoned fixtures must prove independently that: an assigned player with `CA >= PA`, an age-eligible value, and an allowed nonblank unknown `planner_assignments.lane_id` returns `None`, not `Unknown tactic lane`; assigned developing players with `CA < PA`, `null` and 29 age values, and allowed nonblank unknown assignment lanes each return `None`, not `Unknown tactic lane`; and an unassigned developing age-ineligible player with corrupt fallback-only input returns `None` without loading or parsing it. Confirm the focused page seam directly covers both null and 29 without redundant page tests, one age mapping owns every range, only selected-category keys affect evaluability, both GK and outfield category results are exact, assigned/fallback selection and +1/unrounded/weight/order behavior survive, no persistence, IPC, DTO, UI, sort, or layout change occurred, and the architecture update describes only implemented behavior.

## Active work

**PR:** PR 1 — Refine Suggested Training

**Active work:** None — documentation close-out

**Commit:** None — documentation close-out

### RED or removal proof

Not applicable — all three planned packets completed deterministic validation and independent checkpoint review. Full feature validation, feature review, and documentation reconciliation are complete. The reviewed close-out is ready for final PR publication.

### Expected outcome

The reviewed close-out is ready for final PR publication.

### Explicit exclusions

Release preparation and unrelated implementation or documentation.

## Discoveries and replanning

- Planning confirmed that the fallback added by `9c77ca2` loads inputs only for unassigned `CA < PA` rows. Commit 3 added the approved nullable-age and under-29 eligibility condition to that existing selection, not a redesign of fallback behavior.
- Feature validation on implementation HEAD `4b0cab2c56b0fba33222d3b11990700a85824b6c` passed `./scripts/dev check-rust` and `./scripts/dev check`; 809 tests passed and 2 were ignored. Rust primary LSP was clean, and `git diff --check main...HEAD` passed.
- Fresh feature review found no CRITICAL or HIGH findings. The Test portfolio passed and implementation and project ownership conformed. The deferred MEDIUM documentation finding is resolved: only CA/PA- and age-eligible unassigned rows enter the one batched fallback-input load; ineligible rows bypass fallback-input loading and parsing.
- No source evidence invalidated the approved two-implementation-commit split.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Refine Suggested Training | Commit 1 — Record the approved feature plan | 06e5266409112bad31560b44fa2514d0cab5afbf | Recorded the reviewed schema 2 ledger and TODO activation without changing executable behavior. | `ledger_state.py`: runnable; `delivery_state.py`: runnable; `git diff --cached --check`: passed; pre-commit `check-fast`: passed. | Not applicable | Clear | 0 | None. |
| PR 1 — Refine Suggested Training | Commit 2 — Classify suggested training focuses | 4346ca4e6ccae3fdcb1f5b5fce1bdd972be6a882 | Replaced tuple-only inventories with one typed, categorized focus catalog while preserving unrestricted ranking behavior. | RED failed on absent category metadata; 10 focused tests passed; `./scripts/dev check-rust`: 805 passed, 2 ignored; Rust LSP and `git diff --check`: passed. | Pass | Clear | 1 | Review correction added complete-length catalog proof, removed a redundant category test, and narrowed catalog visibility. |
| PR 1 — Refine Suggested Training | Commit 3 — Tailor suggestions to player age | 4b0cab2c56b0fba33222d3b11990700a85824b6c | Applied exact age-category ranking and early CA/PA and age gates while preserving eligible assigned and fallback lane behavior and the string-or-null read contract. | Initial `check-rust` failed while age-aware call sites were incomplete; `./scripts/dev check-rust` and `./scripts/dev check`: 809 passed, 2 ignored; Rust LSP and `git diff --check`: passed. | Pass | Clear | 0 | MEDIUM deferred to feature close-out and resolved in the architecture reconciliation: ineligible unassigned rows are excluded before fallback-input loading and parsing. The reported RED was a compile failure rather than behavioral proof; final focused and full-suite tests prove the shipped contract. |

## Final validation

- `./scripts/dev check-rust` after each Rust implementation commit proves Rust format, Clippy, and the discovered Rust test suite.
- `./scripts/dev check` after Commit 3 and at feature validation proves the full repository gate.
- No browser smoke is required because this feature leaves frontend behavior and rendering unchanged.

## Documentation impact

Documentation reconciliation is complete. `.wiki/ARCHITECTURE.md` records that only CA/PA- and age-eligible unassigned rows enter the one batched fallback-input load and that ineligible rows bypass fallback-input loading and parsing. `.wiki/TODO.md` marks JAY-59 complete and links the completed record. No other documentation, ADR, or debug report is required. The orchestrator owns the archive move: after inspection, move this complete ledger to `.wiki/features/completed/refine-suggested-training.md`.
