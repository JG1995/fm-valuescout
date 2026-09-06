# Squad Planner Redesign

## Status

Ready for final publication

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 35ca73a4749aaf88b945a3cc6ed47fdeeb4d5073a281d1c1407b8b007491356b

## Intent

Replace the Squad Planner depth matrix with an information-dense squad-depth board that renders directly from Manage Teams configuration (Linear JAY-58). Each enabled squad shows its own independently named and ordered strings under one sticky tactical-slot band, with compact assignment cards that show player name and Current → Potential suitability. Manage Teams becomes the single owner of all structural configuration: squad enable/disable, squad display names, and string add/remove/rename/reorder.

## User-visible behavior

- The Planner shows one squad-depth board with 11 tactical-slot rows. Each row names its IP and OOP role and position for that slot.
- Each enabled squad renders as one horizontally grouped section with its configured display name and one column per configured string in configured order. Different enabled squads can show different string counts and names. All enabled squads render simultaneously in one board at all supported widths; the board owns horizontal overflow and no team tabs exist.
- Every string column carries its configured name. Every cell is a bounded compact card with the player name and Current → Potential suitability, or an explicit Assign action when empty.
- Manual assignment, move with confirmation, and removal stay as they are. Best role fit, current and potential optimization, Manage Teams, and Clear all stay in the single action toolbar.
- Disabled squads do not appear. The tactical-slot column stays sticky during horizontal overflow. Ultrawide width reveals more strings instead of stretching cards. The board keeps the existing dark and gold design and the My Club shell.

## Invariants

- One tactic per app save, shared by all teams. Eleven ordered, save-scoped tactic lanes. No tactic change in this feature.
- Player UIDs stay unique across a save. Assignments resolve against the active snapshot as resolved, outside-pool, or unresolved. Missing scores render as `—`.
- Disabling a populated squad requires confirmation and deletes only that squad's strings, assignments, and related staffing targets. This matches the implemented `save_team_settings` behavior.
- Removing a populated string requires target-specific confirmation and deletes only assignments in that string. Renames and reorders keep stable string IDs, so assignments survive them.
- At least one enabled squad and at least one string per enabled squad always remain.
- Rust owns persistence, validation, ordering, and removal scope. React owns presentation and interaction state only.
- No cross-feature imports. `src/lib/tauri-client.ts` stays the sole invoke site.

## Non-goals

- Tactic editing, optimizer rules, scoring, picker behavior, Best role fit reference, and Clear all semantics do not change.
- Mockup-only chrome is excluded: footer player totals, squad string or player counts, saved-state and last-updated bars, page banners, per-section add-string buttons, per-section overflow menus, and unrequested stats or counts.
- `ARCHITECTURE.md` and `DESIGN.md` stay unchanged during planning and implementation. They describe current behavior and change only at feature close-out after the new behavior exists.
- No new dependencies, no new abstractions, and no ADR. The work extends the established `planner_teams` save-owned configuration boundary, which has no durable alternative.

## Current-state map

- Relevant components: `src/features/planner/components/planner-depth-matrix.tsx` (737 lines; toolbar, tabs, string-menu state, add/remove/clear/optimize mutations), `src/features/planner/components/planner-depth-table.tsx` (591 lines; table renderer, ordinal string headers, ellipsis add/remove menu, `PlannerStringRemovalConfirmation`), `src/features/planner/components/planner-team-management.tsx` (439 lines; enable/disable plus display-name draft, populated-removal confirmation, cache reconciliation), `src/features/planner/components/planner-slot-fit-picker.tsx` (411 lines; assign, move, clear; unchanged).
- Data model: frontend `src/features/planner/types/depth.ts` (`PlannerString` has `id`, `stringOrder`, `assignments`; no name) and `src/features/planner/types/team.ts` (`PLANNER_TEAMS`, `PLANNER_TEAM_NAMES`). Backend `src-tauri/src/features/planner/depth.rs` (`PlannerString` has `id`, `string_order`; `add_string` appends next order; `remove_string` deletes one string's assignments and renumbers survivors) and `teams.rs` (`save_team_settings` replaces the one-to-three category configuration in one transaction with populated-removal guard and staffing-target cleanup).
- Persistence and migrations: `planner_strings(save_id, team, string_order)` has no name column. `planner_teams(save_id, team, display_name)` owns availability plus display names. Highest migration is v42 (`reset_planner_tactic_lanes`); the next version is v43. Commands registered in `src-tauri/src/lib.rs` include `add_planner_string`, `remove_planner_string`, `save_planner_teams`, `get_planner_team_removal_impacts`, and `get_planner_depth`.
- Existing behavioral assumptions: `get_planner_depth` returns only available teams in canonical order with display names. The matrix switches between per-team tabs and a combined table by measured width; this feature retires that tab fallback and renders all enabled squads in one board at every width. Optimizer allocates per ordered string IDs and is name-agnostic. String headers use a local `ordinal()` helper duplicated in `planner-depth-table.tsx` and `planner-slot-fit-picker.tsx`.
- Architectural seams: the My Club route composes Planner with `hidden`-prop workspace preservation. Planner tactic, depth, slot candidates, and role references share the Planner Query tree. Depth and candidate caches reconcile after team saves, string mutations, tactic saves, Load Data, save changes, and managed-club saves.
- Project validation commands: `./scripts/dev test <pattern>`, `./scripts/dev check-app`, `./scripts/dev check`, `./scripts/dev smoke` (Chromium via `pnpm exec playwright install chromium`).
- Primary risks: the v43 migration test seeds multi-string teams and asserts exact ordinal labels including the 11th/12th/13th boundary (orders 10–12), and the new Commit 6 board commit keeps a route assertion on migrated labels; every string writer (`ensure_depth`, `add_string`, restored-team creation in `save_team_settings`) must use the shared ordinal helper so no blank `display_name` persists in any trunk-safe intermediate commit; the structural save must keep string IDs stable across renames and reorders and validate IDs in the same transaction before any mutation; the board must keep sticky context and bounded widths at 1280×800 through 3440×1440 with enough strings to overflow at 1280.

## Feature architecture

- Rust owns one structural transaction. The extended team-settings save accepts the full enabled-team set plus each team's ordered string set with stable IDs, and it replaces teams and strings atomically: renames and reorders keep IDs and assignments, removals delete only that string's assignments, and populated squad or string removal without confirmation fails with a target-specific error. New strings (`id: None`) follow one rule: a blank or whitespace-only `display_name` makes Rust derive the next ordinal default label before validation; a non-blank name is trimmed, validated, and preserved. Retained IDs must never carry a blank name. String-name rules mirror the team-name precedent: non-empty after trim, at most 40 characters, unique per team case-insensitively, enforced in Rust beside the existing `normalize_inputs`. Before any mutation in the same transaction, Rust validates the full string set: it rejects duplicate retained IDs, unknown IDs, IDs from another save, and IDs declared under the wrong team; it derives removal only from the validated current set. Reorder uses a collision-safe two-phase write (first to a temporary non-colliding order range, then to final contiguous orders) so normal 0↔1 swaps succeed under `UNIQUE(save_id, team, string_order)`.
- The board is presentation only. It reads `PlannerDepth` (teams with named strings, tactic lanes) and renders the sticky slot band, grouped squad sections, named string columns, compact cards, and Assign actions. It owns no scoring, persistence, or mutation logic. Assignment, move, clear, optimize, Best role fit, Manage Teams, and Clear all keep their existing components and mutations.
- Manage Teams owns every structural edit. Its draft edits enable/disable, display names, string add/remove/rename/reorder per enabled team, prefills the next ordinal default for new strings (aligned with the server rule that blank input derives the default), and surfaces target-specific confirmations for populated squad or string removals before resubmitting with confirmation.
- The board is one horizontally grouped surface with no tabs and no measured combined-vs-tab fallback. It renders every enabled squad simultaneously at all supported widths, owns horizontal overflow with a sticky tactical-slot band, and uses bounded fixed string/card widths. The string-header add/remove menu, its add/remove commands, and their mocks and tests retire in Commit 5 with the last callers, while `PlannerDepthTable` stays as a static renderer with stored names and assignment cells. The static table (including the measured combined-vs-tab fallback) is replaced by the board in Commit 6, which also lands the complete responsive contract. No compatibility path remains after Commit 5.

## Uncertainty register

### Known

- Local `main` is `f90a14d74de892684cb0e6907a2a273a650a9672` and does not contain `origin/main` ref `51d857b82181d54dee0ca8776a8cbf9f1508ac93`. The current branch tree is byte-identical to `origin/main` (`git diff --stat origin/main HEAD` is empty), but activation requires a local fast-forward of `main` first. No fetch, pull, or branch switch happened during planning.
- The directional mockup is `/mnt/c/Users/jonas/Downloads/ChatGPT Image Sep 6, 2026, 12_56_04 PM.png`. It shows a tactical-slot left band, nested squad and string headers, compact bordered player cards, explicit Assign slots, and a single toolbar. Its footer stats, saved-state bar, banner, per-section add buttons, and overflow menus are out of scope.
- The existing route suite is `src/app/routes/my-club-squad.test.tsx` (6457 lines; owns depth-matrix behavior including string add/remove, tabs, and toolbar). The browser contract is `e2e/smoke.spec.ts` (4121 lines; owns Planner navigation and tactic viewport fits). IPC doubles live in `src/testing/planner-ipc-mock.ts`, `src/testing/setup.ts`, and `e2e/tauri-ipc-stub.ts`.

### Assumptions

- String counts stay small (single digits per team), so ordinal default labels and per-team uniqueness checks need no paging or performance work.
- Headless Chromium accepts the 3440×1440 Playwright viewport for the board assertions, as it did for the tactic containment work.

### Decisions

- One PR. Repository evidence shows no independently mergeable trunk-safe seam: the migration, transaction, management UI, and board all build on the same Planner surface, and past Planner persistence work (v28 team settings, v42 tactic reset) shipped inside its feature PR. A migration alone would carry no reviewable behavior.
- String names are unique per team case-insensitively. This mirrors the `planner_teams` NOCASE uniqueness rule and keeps string headers and accessible names distinct. Consequence: the structural save rejects duplicate string names within one team with a field-level error.
- Every empty cell shows one explicit Assign action. This keeps keyboard parity with occupied cells and matches the mockup's intentional empty-slot actions. Consequence: no passive "Empty slot" text without an action.
- `save_planner_teams` carries the full structure (teams plus strings) instead of adding parallel string commands. Consequence: `add_planner_string` and `remove_planner_string` retire in new Commit 5, the atomic commit that removes the string-header structural UI and retires every legacy production and test caller while retaining `PlannerDepthTable` as a static renderer, and Manage Teams submits one transactional save. Commit 3 keeps both commands, their service helpers, API files, mocks, stubs, and tests fully working; it extends the existing Manage Teams caller with a behavior-preserving payload adapter that round-trips current strings unchanged and supplies one valid ordinal default string for a newly restored team.
- One board, no tabs. The board renders every enabled squad simultaneously in one horizontally grouped surface at all supported widths and owns horizontal overflow with a sticky tactical-slot band, using horizontal scrolling only. Consequence: new Commit 5 removes the string-header structural UI and retires the legacy string path while retaining the static `PlannerDepthTable` (including its tab fallback) untouched in layout behavior; new Commit 6 replaces that static table with the board, retires the measured combined-vs-tab fallback (`combinedMatrixMinimumWidth`, width measurement, tab state and refs, tab focus handling), and lands the whole responsive contract (board-owned overflow, sticky slot context, fixed bounded card/string widths) in the same atomic outcome. New Commit 6 does not add Tactic-style `max-w` centering and does not edit `src/app/routes/my-club.tsx`. There is no later polish packet.
- New-string semantics are exact. For `id: None`, a blank or whitespace-only `display_name` makes Rust derive the next ordinal default; a non-blank name is trimmed, validated, and preserved. Existing retained IDs may never have blank names. Consequence: Manage Teams prefills the ordinal default for usability, and the server rule stays authoritative for blank input.
- No ADR. The change extends the save-owned Planner configuration boundary that `optional-planner-teams` established. No durable alternative exists.

### Unknowns

- None. No developer question remains open.

### Risks

- Backfill mismatch. A wrong ordinal expression would rename every existing string header. Mitigation: the v43 migration test seeds multi-string teams and asserts exact ordinal labels, and the board commit keeps a route assertion on migrated labels.
- ID instability on reorder. Rewriting IDs instead of orders would orphan assignments. Mitigation: the transaction updates only `string_order` and `display_name` for retained IDs, proved by a rename/reorder survival test with assignments present.
- Stale-rule drift. Rust validation and the Manage Teams draft could disagree on name rules. Mitigation: the management commit asserts the same three rules (non-empty after trim, 40-character bound, per-team uniqueness) on both sides with one shared error path each, and the transaction commit proves blank server input derives the ordinal default while custom names survive.
- ID trust-boundary bypass. An unvalidated string ID could move or delete another save's data. Mitigation: same-transaction pre-write validation rejects duplicate, unknown, cross-save, and wrong-team IDs before any mutation, proved by focused Rust tests.
- Reorder unique-constraint failure. Naive in-place order updates fail 0↔1 swaps under `UNIQUE(save_id, team, string_order)`. Mitigation: the collision-safe two-phase reorder, proved by an assigned 0↔1 reorder test with ID and assignment preservation.
- Board overflow regression. Grouped sections with many strings could break sticky context or force card stretch. Mitigation: new Commit 6 lands bounded fixed card widths, board-owned horizontal overflow with a sticky slot band, and smoke viewport assertions from 1280×800 through 3440×1440 on a fixture with enough strings to overflow at 1280, proving 3440×1440 reveals more fixed-width columns without card growth or page-level overflow.

## Walking skeleton

Land the v43 string-name persistence with ordinal backfill and ordinal defaults on every string writer, then extend the team-settings transaction to own teams plus strings with stable IDs and same-transaction ID validation alongside the retained legacy mutations, then move string editing into Manage Teams additively, then retire the legacy string path with the header UI while the table stays as a static renderer, then replace that static table with the one board including its whole responsive contract. Each step keeps trunk green: persistence without consumers and no blank names, transaction without callers with legacy commands still working, management UI behind the existing modal with legacy header UI still working, static table with Manage Teams as sole structural owner, and the board replacing the table with board-owned overflow and bounded widths in the same atomic outcome.

## Delivery plan

### PR 1 — Redesign Squad Planner around a configurable squad-depth board

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feat/squad-planner-redesign`

**Base branch:** `main`

**Base ref precondition:** local `main` must contain `51d857b82181d54dee0ca8776a8cbf9f1508ac93` after fast-forward synchronization. Planning verified local `main` at `f90a14d74de892684cb0e6907a2a273a650a9672`, which does not contain that ref. Delivery stops at activation until the base synchronizes. Ledger state alone creates no branch.

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict status `check`

**Feature close-out:** Current

**CI repair rounds:** 0

**Provisional PR title:** `feat(planner): redesign Squad Planner around squad-depth board`

**Purpose:** Deliver the complete JAY-58 board in one review boundary: named-string persistence, one structural transaction alongside the retained legacy path, Manage Teams string ownership, legacy-path retirement with a static table, and the depth-board renderer with its whole responsive contract.

**Depends on:** None.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(planner): record approved squad-depth board plan`

**Work:** Commit the reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/squad-planner-redesign.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — active feature state with a link to this ledger.

**Behavior and data flow:**

- Move planning truth into one reviewed active ledger and record the exact delivery sequence before implementation.

**Ordered implementation steps:**

1. Verify the active branch (`feat/squad-planner-redesign`) and that synchronized local `main` contains `51d857b82181d54dee0ca8776a8cbf9f1508ac93`, without changing Git state. Stop otherwise.
2. Confirm the worktree contains only the reviewed planning paths.
3. Run the ledger classifier.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The ledger classifier proves structural consistency.

**Patterns to verify:**

- The active-ledger template, current TODO ownership rules, and schema 2 status vocabulary.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, BACKLOG, ARCHITECTURE, DESIGN, plan scope, packet order, or reviewed decisions.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, exact branch activation, and the base precondition above.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/squad-planner-redesign.md`

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, a branch mismatch, or a base that does not contain the recorded ref.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files.

#### Commit 2 — Persist named planner strings with ordinal backfill

**Status:** Completed

**Provisional commit:** `feat(planner): persist named planner strings`

**Work:** Add a `display_name` to planner strings with a v43 migration that backfills every existing string to its current ordinal label, apply the shared ordinal default in every current string writer, and expose the name through the depth read path. No UI change yet.

**Size assessment:** About 60 changed non-test implementation lines (one migration, one struct field, DTO mapping). Within the soft target.

**Out of scope:**

- Structural save changes, Manage Teams edits, board rendering, command retirement, and optimizer changes. Later commits own those.

**Implementation packet:**

- Give the worker a complete persistence handoff with no frontend wiring.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — new `PLANNER_STRING_NAMES_SQL` (`ALTER TABLE planner_strings ADD COLUMN display_name TEXT NOT NULL DEFAULT ''` plus an `UPDATE` that backfills each row to its ordinal label from `string_order` within its save and team, with the 11th/12th/13th exception) registered as migration v43 with registry assertions extended. Rust validation owns the name rules; no new SQL CHECK.
- `src-tauri/src/features/planner/depth.rs` — new pure `ordinal_label(string_order)` helper shared by all writers; `PlannerString` gains `display_name`; `load_depth` selects it; ordering stays by `string_order`. `ensure_depth` writes the ordinal label on every lazy string insert. `add_string` writes the ordinal label for the appended order and returns it on the struct. The helper mirrors the frontend `ordinal()` in `planner-depth-table.tsx`.
- `src-tauri/src/features/planner/teams.rs` — restored-team creation in `save_team_settings` writes the ordinal label for the order-0 string it inserts for a newly available team.
- `src-tauri/src/features/planner/commands.rs` — `PlannerStringDto` gains `displayName`, mapped from every `PlannerString` construction site so no blank name escapes. No command signature changes.
- `src/features/planner/types/depth.ts` — `PlannerString` gains `displayName` (required, non-blank). No component change; the current UI ignores the field. Typed IPC and testing fixtures (`src/testing/planner-ipc-mock.ts`, `src/testing/setup.ts`, `e2e/tauri-ipc-stub.ts`) carry the new field where they construct string DTOs.
- `src/app/routes/my-club-squad.test.tsx` — update every required `PlannerString` literal to carry the matching stored `displayName` (ordinal labels for seeded rows); no blank names. No behavior or assertion change in this commit.
- `src-tauri/src/features/planner/squad_tests.rs` and `src-tauri/src/features/planner/role_reference_tests.rs` — update active-schema raw SQL fixtures that insert into `planner_strings` to include the non-blank `display_name` value matching the seeded order; no blank names.
- `src-tauri/src/db/migrations.rs` tests module — new `migrates_v42_strings_to_ordinal_display_names`: build through v42, seed one save with thirteen senior strings (orders 0–12), two reserves strings, and assignments on stable IDs, run `apply`, assert `user_version` 43, exact labels for orders 0–2 (`1st string`, `2nd string`, `3rd string`) and the migration boundary orders 10–12 (`11th string`, `12th string`, `13th string`), no blank `display_name` rows, and untouched assignments and teams.
- `src-tauri/src/features/planner/depth_tests.rs` — extend depth read coverage to assert returned display names round-trip, plus focused proofs: lazy `ensure_depth` initialization creates the ordinal-labeled string, direct `add_string` returns the next ordinal label including one writer proof at the 11th/12th/13th boundary (orders 10–12), and a team restored through `save_team_settings` after migration gets the ordinal-labeled order-0 string.

**Behavior and data flow:**

- Migration runs once per database on upgrade. Every string writer uses the shared `ordinal_label` helper, so no `display_name` stays blank in this or any later trunk-safe commit. Reads return the stored name; writes still use order-only paths until Commit 3, with labels attached. Existing headers render unchanged because backfilled labels equal current ordinal text.

**Ordered implementation steps:**

1. Add the failing migration test with multi-string teams and confirm RED.
2. Add the migration, the shared helper, the struct/DTO mapping, and the ordinal default in `ensure_depth`, `add_string`, and restored-team creation until the proof turns GREEN.
3. Refactor only while the focused proof stays green.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- New migration test is the RED→GREEN proof: it fails without v43 (missing column) and passes with exact ordinal labels including the 11th/12th/13th boundary, no blank names, and preserved assignments. Focused writer proofs (lazy initialization, direct add including the 11th/12th/13th boundary, team restoration after migration) fail with blank or missing labels and pass with exact ordinal defaults. Required `PlannerString` literals in `src/app/routes/my-club-squad.test.tsx` and active-schema raw SQL fixtures in `src-tauri/src/features/planner/squad_tests.rs` and `src-tauri/src/features/planner/role_reference_tests.rs` carry non-blank names. Existing depth tests stay green unchanged.

**Patterns to verify:**

- The v28 `planner_teams` backfill pattern (INSERT...SELECT per save) for save-scoped seeding; the v42 reset test for version-assertion style.

**Constraints and non-goals:**

- Exactly one new migration version. No other table changes, no IPC changes, no frontend behavior change, no uniqueness index (Rust owns name rules in Commit 3). No blank `display_name` may persist in any trunk-safe intermediate commit.

**Dependencies and sequencing:**

- Requires Commit 1. Blocks Commit 3 (transaction writes the new column).

**Validation:** `./scripts/dev check-rust` (owns the v43 migration test and planner suite), then `./scripts/dev check`.

**Stop conditions:** Stop if the ordinal backfill needs per-row logic that SQL cannot express (report instead of shipping wrong labels), or if any non-string table loses rows (report as data-loss scope breach).

**Review mandate:** Verify the migration touches only `planner_strings`, backfill labels match the frontend ordinal rule including the 11th/12th/13th boundary, every creation path (`ensure_depth`, `add_string`, restored-team creation) writes the shared ordinal label with no blank rows, assignments and teams are proved preserved, DTO, type, fixture, route-literal (`src/app/routes/my-club-squad.test.tsx`), and raw SQL fixture (`squad_tests.rs`, `role_reference_tests.rs`) changes carry non-blank names without behavior change, and the diff stays within the migration registry, the depth and teams services, DTOs, types, fixtures, and owned tests.

#### Commit 3 — Own full squad structure in one team-settings transaction

**Status:** Completed

**Provisional commit:** `feat(planner): save squad structure transactionally`

**Work:** Extend the team-settings save to accept each enabled team's ordered string set with stable IDs, applying renames, reorders, adds, and removals in one transaction with same-transaction ID validation and a collision-safe reorder, plus target-specific populated-removal guards for squads and strings. This adds the structural save alongside the retained legacy mutations. Keep the single-string add/remove commands, service helpers, API files, mocks, stubs, and their tests fully working; the new payload has one caller (the Manage Teams adapter) and no retired path.

**Size assessment:** About 170 changed non-test implementation lines (extended input normalization, pre-write ID validation, transactional string replacement with two-phase reorder, impact reads, payload adapter). Within the soft target.

**Out of scope:**

- Manage Teams UI string editing, board rendering, picker or optimizer changes. The new payload has exactly one caller (the behavior-preserving Manage Teams adapter); the legacy add/remove path keeps its UI and tests untouched. No command retirement in this commit.

**Implementation packet:**

- Add one structural save alongside the retained legacy mutations plus string-aware impact reads, while keeping the legacy path green.

**Files and responsibilities:**

- `src-tauri/src/features/planner/teams.rs` — extend `PlannerTeamInput` with `strings: Vec<PlannerStringInput>` (`id: Option<i64>`, `display_name: String`); extend normalization with the exact new-string rule (for `id: None`, a blank or whitespace-only name derives the Commit 2 `ordinal_label` default from the next order before validation; a non-blank name is trimmed, validated, and preserved; retained IDs must never carry a blank name) and require at least one string per enabled team, non-empty names of at most 40 characters, and case-insensitive uniqueness within the team. `save_team_settings` validates the full string set in the same transaction before any mutation: it rejects duplicate retained IDs, unknown IDs, IDs from another save, and IDs declared under the wrong team, and it derives removal only from the validated current set. It then applies the full replacement in its existing transaction: retained IDs keep assignments with only `string_order`/`display_name` updated via a collision-safe two-phase reorder (first to a temporary non-colliding order range, then to final contiguous orders), removed strings delete only their own assignments, removed squads keep the existing team-level cleanup, and renumbering reuses the established per-team order compaction. Populated squad or string removal without confirmation fails with an error naming each affected display name and assignment count.
- `src-tauri/src/features/planner/depth.rs` — no service change in this commit; `add_string` and `remove_string` stay as the working legacy path with their order-compaction logic untouched.
- `src-tauri/src/features/planner/commands.rs` — extend `PlannerTeamInputDto` with the string set; extend `PlannerTeamRemovalImpactDto` with per-string impacts (`stringId`, `displayName`, `assignmentCount`) for strings absent from the desired set. `add_planner_string` and `remove_planner_string` stay registered and working.
- `src-tauri/src/lib.rs` — no registration change in this commit.
- `src/features/planner/api/save-planner-teams.ts` — extend `PlannerTeamSettingInput` with the string set. `src/features/planner/api/add-planner-string.ts` and `src/features/planner/api/remove-planner-string.ts` stay and keep their callers.
- `src/features/planner/components/planner-team-management.tsx` — behavior-preserving payload adapter only: the existing team-only draft round-trips each currently enabled team's strings unchanged (stable IDs, stored names, current order, read from the depth query cache) and supplies one valid ordinal default string for a newly restored team, so the extended save accepts the current UI with no visible change.
- `src/features/planner/types/team-removal-impact.ts` and `src/features/planner/api/fetch-planner-team-removal-impacts.ts` — carry the per-string impacts.
- Backend tests in `teams_tests.rs` (including its `PlannerTeamInput` helper) — RED→GREEN proof: rename plus reorder with assignments present keeps IDs and assignments; string removal deletes only that string's assignments; populated string removal without confirmation fails naming the string; blank new-string input derives the next ordinal default while a custom non-blank name is trimmed and preserved; retained blank names fail; empty, overlong, and duplicate names fail; last-string removal fails; duplicate retained IDs, unknown IDs, cross-save IDs, and wrong-team IDs all fail before any mutation; assigned 0↔1 reorder keeps IDs and assignments. Update the `teams_tests.rs` helper to construct required `PlannerTeamInput.strings`.
- `src-tauri/src/features/planner/depth_tests.rs` and `src-tauri/src/features/planner/optimizer_tests.rs` — update direct required-`PlannerTeamInput.strings` callers to construct the new string set; no behavior change to the exercised paths.
- `src/app/routes/my-club-squad.test.tsx` — update existing save-payload expectations to assert the adapter round-trips retained IDs, stored names, and current orders unchanged, and that restoring a team submits one `id: None` ordinal-default string; proved by a focused adapter case.
- `src/testing/planner-ipc-mock.ts` and `src/testing/setup.ts` — keep the `add_planner_string` and `remove_planner_string` doubles working; extend the `save_planner_teams` double with the string set. `e2e/tauri-ipc-stub.ts` mirrors the change with both legacy commands intact.

**Behavior and data flow:**

- One `save_planner_teams` call validates the whole structure (including string-ID trust checks) in the same transaction, then replaces teams and strings atomically and returns the reloaded depth. The removal-impact read lets the UI confirm populated removals before resubmitting with confirmation. The legacy `add_planner_string` and `remove_planner_string` commands keep working with no caller change. Optimizer, picker, and depth reads are untouched and stay name-agnostic.

**Ordered implementation steps:**

1. Add the failing structural-save tests (rename/reorder survival, scoped string deletion, target-specific guard, blank-derived ordinal default, custom-name preservation, name rules, ID trust-boundary rejections, 0↔1 reorder) and confirm RED.
2. Implement the extended normalization, same-transaction ID validation, two-phase reorder transaction, impact reads, payload adapter, and command input changes until the proof turns GREEN.
3. Keep the legacy service functions, commands, registrations, API files, doubles, and their tests working with no caller change.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- New `teams_tests.rs` cases are the RED→GREEN proof: they fail on the order-only save and pass with stable IDs, scoped deletion, named guards, blank-derived ordinal defaults, preserved custom names, rejected names, rejected untrusted IDs, and assigned 0↔1 reorder. Direct required-`PlannerTeamInput.strings` callers in `depth_tests.rs`, `optimizer_tests.rs`, and the `teams_tests.rs` helper are updated to construct the string set. A focused adapter proof in `src/app/routes/my-club-squad.test.tsx` shows retained IDs/names/orders round-trip unchanged and restoring a team submits one `id: None` ordinal-default string. Legacy-command presence is proved by the retained doubles and the passing suite with the current UI and all tests compiling and working unchanged.

**Patterns to verify:**

- Existing `save_team_settings` transaction and `planner_team_removal_impacts` as the direct analogues; `normalize_inputs` team-name rules as the string-rule source.

**Constraints and non-goals:**

- No UI behavior change in this commit beyond the invisible payload adapter. No staffing-target rule change beyond the existing team-removal cleanup. No command retirement, no API file deletion, no mock or test removal. No new command surface besides the extended inputs and impacts.

**Dependencies and sequencing:**

- Requires Commit 2 (writes the name column). Blocks Commit 4 (Manage Teams calls the new payload).

**Validation:** `./scripts/dev check-rust`, then `./scripts/dev test planner`, then `./scripts/dev check`.

**Stop conditions:** Stop if the transaction needs a second round trip to stay atomic (report instead of splitting it), if retained-ID updates cannot preserve assignments (report as data-loss risk), or if the impact read cannot name populated strings (report instead of a generic confirmation).

**Review mandate:** Verify stable IDs across rename and reorder with assignments present, deletion scope limited to the removed string or squad, guard errors naming each affected display name and count, blank new-string input deriving the Commit 2 ordinal label with custom names preserved and retained blanks rejected, duplicate/unknown/cross-save/wrong-team IDs rejected before any mutation, 0↔1 reorder succeeding under the unique constraint, direct `PlannerTeamInput.strings` callers in `depth_tests.rs`, `optimizer_tests.rs`, and the `teams_tests.rs` helper updated, adapter round-trip plus restored-team `id: None` ordinal-default proof present in `src/app/routes/my-club-squad.test.tsx`, legacy commands with remaining callers, registrations, doubles, and tests intact and green, current UI and tests compiling and working, and the diff staying within the teams service, commands, API, adapter, mocks, and owned tests.

#### Commit 4 — Manage strings inside Manage Teams

**Status:** Completed

**Provisional commit:** `feat(planner): manage strings in Manage Teams`

**Work:** Move all string add, remove, rename, and reorder editing into the Manage Teams modal, one section per enabled team, with target-specific confirmations for populated removals and the same cache and focus reconciliation the team save already uses.

**Size assessment:** About 150 changed non-test implementation lines (modal string sections, draft logic, confirmation flow). Within the soft target.

**Out of scope:**

- Board rendering, header-menu code removal (the header menu retires in new Commit 5), picker or optimizer changes.

**Implementation packet:**

- Turn the team-only modal into the single structural editor.

**Files and responsibilities:**

- `src/features/planner/components/planner-team-management.tsx` — per enabled team, a string section listing ordered strings with rename fields, move-up/move-down reorder buttons, remove buttons, and an add-string action that prefills the next ordinal default (aligned with the server rule: a cleared blank still derives the default in Rust). Draft validation applies the three name rules per team with field-level errors. Saving first requests removal impacts; when populated squads or strings would be removed, the modal shows a target-specific confirmation naming each display name with its assignment count (and staffing targets for squads) before resubmitting with confirmation. Success keeps the existing `onSaved` reconciliation (depth replacement, candidate invalidation, picker and string-menu reset, focus move).
- `src/features/planner/utils/string-label.ts` (new) — shared `ordinalStringLabel` and `nextOrdinalDefault` used by the modal; replaces the duplicated local `ordinal()` copies in a later cleanup owned by Commit 5.
- `src/app/routes/my-club-squad.test.tsx` — add Manage Teams string tests only, with all header-menu code, mocks, and tests retained unchanged and green: rename preserves assignments, reorder moves columns, populated string removal confirms with the named count and deletes only that string's assignments, new strings prefill the next ordinal default, and invalid names block saving. No legacy code or assertion deletion in this commit.
- `e2e/smoke.spec.ts` — extend the existing team-management flow (`planner team management renames, removes, and restores a populated team`) with a string rename and reorder pass through the modal.

**Behavior and data flow:**

- Draft edits stay local until save. One `savePlannerTeams` call submits teams plus strings; the impact read gates confirmation; success reconciles depth and candidate caches exactly as team saves do today.

**Ordered implementation steps:**

1. Write the failing route tests for modal string rename, reorder, confirmed removal, ordinal default, and name-rule blocking, and confirm RED.
2. Implement the modal string sections, draft logic, and confirmation flow until the proof turns GREEN.
3. Refactor only while the focused proof stays green.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- New route tests are the RED→GREEN proof: they fail on the team-only modal and pass with assignments surviving rename and reorder, scoped deletion after named confirmation, correct ordinal defaults, and blocked invalid names. The extended smoke flow proves the IPC path. All header-menu code, mocks, and tests stay unchanged and green in this commit; every legacy deletion moves to new Commit 5.

**Patterns to verify:**

- Existing modal draft, `validateDraft`, impact-check, and `reconcileTeamSettings` flow as the unmodified skeleton; existing destructive-modal focus and pending-label behavior.

**Constraints and non-goals:**

- No board change. No change to team enable/disable rules, staffing-target cleanup, or the single-string-per-team minimum. No new IPC.

**Dependencies and sequencing:**

- Requires Commit 3 (calls the structural payload). Blocks new Commit 5 (board assumes Manage Teams owns structure; legacy header add/remove still works until new Commit 5 retires it).

**Validation:** `./scripts/dev test my-club-squad`, then `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop if the confirmation copy cannot name each populated string (report instead of a generic confirm), or if draft state and server rules disagree on validity (report instead of duplicating rules in the UI).

**Review mandate:** Verify assignments survive rename and reorder, removal deletes only the named string's assignments, confirmation names each affected string and count, ordinal defaults match backend labels, invalid names block with field errors, focus and cache reconciliation match the existing team-save path, all header-menu code, mocks, and tests are retained unchanged and green with no legacy deletion, and the diff stays within the modal, the shared label util, and owned tests.

#### Commit 5 — Retire legacy string path; Manage Teams owns structure

**Status:** Completed

**Provisional commit:** `feat(planner): retire legacy string path`

**Work:** Make Manage Teams the sole structural owner: remove the string-header add/remove UI and retire all legacy string service functions, commands, registrations, frontend APIs, mocks/stubs, and direct tests, while retaining `PlannerDepthTable` as a static renderer with stored names and assignment cells.

**Size assessment:** About 150 changed non-test implementation lines (header-UI removal, legacy command retirement, static-table retention). Within the soft target; the board replacement stays in Commit 6 so each commit is independently trunk-safe.

**Out of scope:**

- Board rendering, tab-fallback retirement, width bounds, picker, optimizer, Best role fit, Manage Teams, or Clear all logic changes. The static table keeps its current layout behavior including tabs.

**Implementation packet:**

- Retire the legacy string path with its last callers; keep the static table rendering stored names.

**Files and responsibilities:**

- `src/features/planner/components/planner-depth-table.tsx` — retained as a static renderer: remove the `PlannerStringHeader` ellipsis add/remove menu and `PlannerStringRemovalConfirmation`, render each string column with its stored `displayName` and existing assignment cells, keep tabs and layout behavior unchanged. Consolidate the two local `ordinal()` copies onto the Commit 4 `string-label.ts` util where still referenced.
- `src/features/planner/components/planner-depth-matrix.tsx` — remove string-menu state, header refs, and add/remove mutation wiring now owned by Manage Teams; keep the measured combined-vs-tab fallback (`combinedMatrixMinimumWidth`, width measurement, tab state and refs, tab focus handling, `tablist`/`tab`/`tabpanel` roles), toolbar (Best role fit, optimizers, Manage Teams, Clear all), picker, status, and reconciliation untouched.
- Retire with the last callers in this same commit: `add_string` and `remove_string` service functions in `src-tauri/src/features/planner/depth.rs` (order-compaction logic already lives in the structural save), `add_planner_string` and `remove_planner_string` commands in `src-tauri/src/features/planner/commands.rs` and their registrations in `src-tauri/src/lib.rs`, `src/features/planner/api/add-planner-string.ts` and `src/features/planner/api/remove-planner-string.ts`, the `add_planner_string` and `remove_planner_string` doubles in `src/testing/planner-ipc-mock.ts` and `src/testing/setup.ts` and `e2e/tauri-ipc-stub.ts`. No compatibility path remains after this commit.
- `src-tauri/src/features/planner/depth_tests.rs` and `src-tauri/src/features/planner/optimizer_tests.rs` — rewrite still-valid final-contract setup/assertions through `save_team_settings`; delete only tests exclusive to retired direct add/remove service behavior.
- `src/app/routes/my-club-squad.test.tsx` — delete the header-menu string add/remove tests and `add_planner_string`/`remove_planner_string` mock-call assertions retired with the header UI and commands; keep all static-table structure assertions (stored-name headers, assignment cells, tabs) green. No board assertions in this commit.
- `e2e/smoke.spec.ts` — rewrite the `planner depth adds strings for Senior, Reserves, and Youth` smoke test to the Manage Teams string flow; keep navigation, tactic viewport, and combined-vs-tab matrix tests unchanged.

**Behavior and data flow:**

- After this commit Manage Teams is the only structural writer via `save_planner_teams`. The static table reads the same `PlannerDepth` cache and renders stored names with assignment, move, and clear reconciling the same caches. Tactic saves still invalidate depth and candidates.

**Ordered implementation steps:**

1. Rewrite the still-valid `depth_tests.rs` and `optimizer_tests.rs` setup/assertions through `save_team_settings` and confirm the retired-behavior-only tests are the sole deletions.
2. Remove the header-menu UI and retire the legacy commands, helpers, API files, doubles, and stubs with their last callers until the focused proof turns GREEN.
3. Consolidate ordinal helpers.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Manage Teams string tests from Commit 4 plus the retained static-table assertions are the GREEN proof: they pass with stored names rendered, assignments intact, and no remaining legacy caller. Legacy-command absence is proved by the deleted doubles, commands, and registrations with the passing suite holding no remaining caller. Updated smoke selectors prove the IPC path.

**Patterns to verify:**

- Existing `AssignmentScores`, `phaseDescription`, `linkedPositionDescription`, and picker-target shapes as reused contracts; existing dark and gold tokens with no new colors.

**Constraints and non-goals:**

- No scoring, persistence, mutation, toolbar-logic, tab, or layout-behavior changes. No per-section add buttons or overflow menus (Manage Teams owns structure). No footer stats, squad string or player counts, banners, or saved-state bars. No new dependencies.

**Dependencies and sequencing:**

- Requires Commit 4 (Manage Teams already edits strings additively). Blocks Commit 6 (board replaces the static table).

**Validation:** `./scripts/dev test my-club-squad`, then `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop if any retired-behavior test still proves final-contract behavior through the legacy path (rewrite it through `save_team_settings` instead of deleting it), or if the static table needs data the depth DTO does not carry (report instead of adding IPC).

**Review mandate:** Verify the string-header add/remove menu and removal confirmation are gone, the static table renders stored names with assignment cells and unchanged tabs, Manage Teams is the sole structural writer, `add_string`/`remove_string` helpers, both commands, both registrations, both API files, all doubles/stubs in `src/testing/planner-ipc-mock.ts`, `src/testing/setup.ts`, and `e2e/tauri-ipc-stub.ts`, header-menu tests, mock-call assertions, and retired-only `depth_tests.rs`/`optimizer_tests.rs` cases are retired with no remaining caller or registration, still-valid `depth_tests.rs`/`optimizer_tests.rs` setup/assertions run through `save_team_settings`, and the diff stays within the table, the matrix coordinator, retirement sites, and owned tests.

#### Commit 6 — Render the squad-depth board with bounded overflow

**Status:** Completed

**Provisional commit:** `feat(planner): render the squad-depth board`

**Work:** Replace the static table with the complete simultaneous all-squad board including its whole responsive contract in the same atomic outcome: one horizontally grouped surface rendering every enabled squad simultaneously at all supported widths, with a sticky tactical-slot band carrying IP and OOP role and position per slot, named string columns, bounded compact cards with name and Current → Potential suitability, explicit Assign actions for empty cells, board-owned horizontal overflow, sticky slot context, fixed bounded card/string widths, a fixture that overflows at 1280×800, and proof that 3440×1440 reveals more fixed-width columns without card growth or page-level overflow. No later polish packet.

**Size assessment:** About 250 changed non-test implementation lines (one new board component replacing one retired table file, matrix fallback removal, bounded fixed widths). At the soft target boundary; splitting the renderer from its responsive contract would leave broken overflow on trunk.

**Out of scope:**

- Picker, optimizer, Best role fit, Manage Teams, or Clear all logic changes. Layout, structure, and proof only beyond the renderer swap.

**Implementation packet:**

- Swap the static table for the board, retire the tab fallback, and land the responsive contract atomically.

**Files and responsibilities:**

- `src/features/planner/components/planner-squad-board.tsx` (new) — board renderer consuming `teamDepths`, `tactic`, and `options`: sticky slot band (position chip, role name, IP and OOP lines from the existing `phaseDescription` and `linkedPositionDescription` helpers), one section per enabled squad (configured display name only, no string or player counts; one column per string in order with its configured name), compact cards at bounded fixed widths reusing `AssignmentScores` semantics (name plus Current → Potential badges, outside-pool and unresolved states; minimum plus maximum so cards never stretch and never grow with viewport, in existing Tailwind tokens), one Assign button per empty cell opening the existing picker with the same `PlannerSlotTarget`, board-owned `overflow-x-auto` with the slot band sticky. Removed squads render nothing. No `max-w` centering, no route change, no `src/app/routes/my-club.tsx` edit.
- `src/features/planner/components/planner-depth-matrix.tsx` — render the board instead of `PlannerDepthTable`; retire the measured combined-vs-tab fallback (`combinedMatrixMinimumWidth`, width measurement, tab state and refs, tab focus handling, `tablist`/`tab`/`tabpanel` roles) so all enabled squads render simultaneously at every width with horizontal scrolling only; keep toolbar (Best role fit, optimizers, Manage Teams, Clear all), picker, status, and reconciliation untouched.
- Delete `src/features/planner/components/planner-depth-table.tsx` (static table) with its last caller.
- `src/app/routes/my-club-squad.test.tsx` — replace static-table structure assertions (columnheader ordinals, team tabs) with board assertions: slot band with IP/OOP text, grouped sections with configured string names in order, different string counts per squad, compact cards with Current → Potential scores, Assign actions opening the picker, all enabled squads visible simultaneously, absent disabled squads, migrated ordinal labels, and stable-width assertions for bounded cards where jsdom can prove them.
- `e2e/smoke.spec.ts` — update Planner board selectors from table headers to section and string-column headings; keep navigation and tactic viewport tests unchanged; retire the combined-vs-tab matrix test (`planner depth groups all teams when the matrix fits`) and replace it with a simultaneous-squads board assertion; extend Planner viewport coverage on a fixture with enough strings to overflow at 1280 reusing the tactic workspace viewport-fit pattern: no page-level horizontal overflow at 1280×800, sticky slot band visible after board-level horizontal scroll, fixed card widths unchanged between 1280 and 3440×1440, and more fixed-width string columns visible at 3440×1440.

**Behavior and data flow:**

- Board reads the same `PlannerDepth` cache the static table read. Cell activation opens the unchanged `PlannerSlotFitPicker`; assignment, move, and clear reconcile the same caches. Tactic saves still invalidate depth and candidates because roles and weights change card scores. Pure layout contract beyond the renderer swap: no state, query, mutation, or data-flow change; wide screens show more fixed-width strings with horizontal scrolling only.

**Ordered implementation steps:**

1. Write the failing board route tests (slot band, named string columns, card scores, Assign actions, simultaneous squads, absent disabled squads) and the failing smoke viewport assertions (sticky band after board scroll, no page-level horizontal overflow at 1280, fixed card widths across viewports, more columns visible at 3440×1440) and confirm RED.
2. Implement the board, matrix rewiring (including tab-fallback retirement), and fixed min/max width bounds until the proof turns GREEN.
3. Delete the retired static table file with its last caller.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- New board route tests are the RED→GREEN proof: they fail on the static table renderer and pass on simultaneously grouped sections, ordered named strings, truthful card states, working Assign actions, and no disabled-squad output. New smoke viewport assertions are the RED→GREEN proof: they fail on stretching, card growth, or lost sticky context and pass with bounded fixed cards, sticky slot column, board-level overflow at 1280, and more fixed-width columns visible at 3440×1440 without page-level overflow. Route tests guard the width contract where jsdom applies. Updated smoke selectors prove the IPC path.

**Patterns to verify:**

- Existing `AssignmentScores`, `phaseDescription`, `linkedPositionDescription`, and picker-target shapes as reused contracts; existing dark and gold tokens with no new colors; the current matrix `max-h-[min(70vh,720px)]` vertical bound.

**Constraints and non-goals:**

- No scoring, persistence, mutation, or toolbar-logic changes. No per-section add buttons or overflow menus (Manage Teams owns structure). No footer stats, squad string or player counts, banners, saved-state bars, content, color, token, global-layout, route, or shell change. No new breakpoint hook unless direct CSS cannot meet the tested contract. No new dependencies.

**Dependencies and sequencing:**

- Requires Commit 5 (replaces the static table; Manage Teams is already the sole structural owner). Nothing follows; this closes the implementation sequence.

**Validation:** `./scripts/dev test my-club-squad`, then exact `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop if the board needs data the depth DTO does not carry (report instead of adding IPC), if disabled squads leak at any width (report instead of filtering in the UI), if headless Chromium clamps the 3440×1440 viewport (report as a validation gap instead of weakening the assertions), or if width bounds leak onto Squad or shell surfaces (report instead of widening scope).

**Review mandate:** Verify slot rows show IP and OOP role and position, sections and string columns match configuration exactly with no string or player counts, cards show name plus Current → Potential with truthful unresolved/outside-pool/unknown states, every empty cell offers Assign with keyboard reach, all enabled squads render simultaneously with no tabs or measured fallback remaining and horizontal scrolling only, disabled squads are absent, cards keep bounded fixed widths at 1280×800 and 3440×1440 with no card growth, the slot band stays sticky during board-level horizontal scroll, the 1280 fixture overflows inside the board only, 3440 reveals more fixed-width columns, page-level horizontal overflow stays absent at 1280×800, the retired static table has no remaining caller, `src/app/routes/my-club.tsx` is untouched, and the diff stays within the board, the matrix coordinator, and owned tests.

## Active work

None — implementation complete; feature validation and close-out remain.

## Discoveries and replanning

- Planning found local `main` at `f90a14d` without `origin/main` ref `51d857b`. No synchronization happened during planning. The ledger records the base precondition and stops delivery at activation until local `main` fast-forwards to contain that ref.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Redesign Squad Planner around a configurable squad-depth board | Commit 1 — Record the approved feature plan | 4e53fb047353405d1758a3125f2951cda19924df | Recorded the accepted ledger and active TODO link on the authorized feature branch. | Ledger and delivery classifiers passed; staged whitespace check passed. | Not applicable | Clear | 0 | None. |
| PR 1 — Redesign Squad Planner around a configurable squad-depth board | Commit 2 — Persist named planner strings with ordinal backfill | 33167621cf0826d8612bd9e5e78f823f1d5680f6 | Added v43 string-name persistence, exact ordinal backfill, shared writer defaults, and typed DTO fixtures without changing UI behavior. | Expected migration RED; check-rust and full check passed with 801 Rust tests, 0 failed, 2 intentionally ignored; LSP and whitespace checks clean. | Pass | Clear | 0 | None. |
| PR 1 — Redesign Squad Planner around a configurable squad-depth board | Commit 3 — Own full squad structure in one team-settings transaction | e8694a26a735fc1ff321eb6800ea0110847c67f0 | Extended the team-settings transaction and adapter to validate and persist complete ordered string structure with stable IDs, scoped removals, and retained legacy mutations. | Structural RED produced four expected failures; planner 22/22, route 133/133, check-rust 809 passed, full check passed, and focused Planner smoke 1/1 passed. | Pass | Clear | 1 | Corrected target-name assertions and the E2E impact shape. Full smoke also exposed one reproducible unrelated Squad CA-boost progress failure outside this packet; focused Planner smoke passed. |
| PR 1 — Redesign Squad Planner around a configurable squad-depth board | Commit 4 — Manage strings inside Manage Teams | 44e1aac45a6f81c547b5ab45f73f7c6fae4cd4fc | Added ordered string add, rename, reorder, and removal controls to Manage Teams with frontend validation and target-specific team, string, and mixed confirmations. | Expected five-test RED; route 139/139, smoke 56/56, full check with 809 Rust tests, LSP, and whitespace checks passed. | Pass | Clear | 1 | Corrected mixed-removal confirmation wording and added direct combined-impact proof. |
| PR 1 — Redesign Squad Planner around a configurable squad-depth board | Commit 5 — Retire legacy string path; Manage Teams owns structure | 2a64ffc9ea764a00528e3a39427610d6e7ca157e | Removed the header mutation UI and every legacy add/remove service, command, API, registration, double, stub, and exclusive test; retained the static named-string table and tab fallback. | Contract-removal search found only migration-history text; route 131/131, smoke 56/56, full check with 805 Rust tests, LSP, and whitespace checks passed. | Pass | Clear | 1 | Corrected responsive focus proof and added post-save custom stored-name rendering with assignment preservation. |
| PR 1 — Redesign Squad Planner around a configurable squad-depth board | Commit 6 — Render the squad-depth board with bounded overflow | eb4cf64b59ff133f3758a92dbfb9d7650721f66b | Replaced the static table and measured tabs with one simultaneous all-squad board using sticky tactical context, named fixed-width string columns, compact assignment cards, and explicit Assign actions. | Board RED confirmed absent semantics; route 129/129, smoke 57/57 including 1280×800 and 3440×1440 proof, full check with 805 Rust tests, LSP, and whitespace checks passed. | Pass | Clear | 1 | Corrected assignment cells to use compact card surfaces and removed primary text from data cells with direct visual-token proof. |

## Final validation

- `./scripts/dev test my-club-squad` — board, management, picker, toolbar, and tactic-cache behavior green.
- `./scripts/dev test` — full frontend suite green with retired table and command tests removed.
- `./scripts/dev check` — full gate green including v43 migration tests, structural-save tests, Biome, TypeScript, secretlint, Clippy, and Rust tests.
- `./scripts/dev smoke` — Chromium suite green including board selectors, management string flow, and viewport fits at 1280×800 and 3440×1440.
- `git diff --check` clean on the implementation range.

## Close-out evidence

Full frontend validation passed with 853 tests across 76 files. Exact Chromium smoke passed 57/57, including direct assigned and empty card-width measurements at 1280×800 and 3440×1440. The full quality gate passed with 805 Rust tests, 0 failed, and 2 intentionally ignored. Independent feature review cleared the exact implementation range `51d857b82181d54dee0ca8776a8cbf9f1508ac93..eb4cf64b59ff133f3758a92dbfb9d7650721f66b` after one test-only correction round. The stale cross-route Planner tab assertion now targets the simultaneous board header.

## Documentation impact

Feature close-out reconciled `.wiki/ARCHITECTURE.md` and `.wiki/DESIGN.md` with migration v43, transactional named-string ownership, Manage Teams structural editing, retired legacy commands, and the simultaneous overflow board. `.wiki/TODO.md` no longer lists the completed feature. No ADR, BACKLOG change, debug report, or temporary `.work/` artifact is warranted.
