# Optional Planner Teams

## Status

Validation

## Intent

Let the user choose which of the three supported Planner team categories exist in each app save and give each available category a save-scoped display name. Planner depth, candidate selection, and both optimizer modes must use only the configured categories without allowing presentation names to replace the stable `senior`, `reserves`, and `youth` identities.

Linear source: [JAY-28 — Support optional teams in squad planning](https://linear.app/jaycount/issue/JAY-28/support-optional-teams-in-squad-planning).

## User-visible behavior

- A save starts with Senior, Reserves, and Youth available, matching current behavior.
- The user can keep any one, two, or three supported categories. No fourth or custom category can be added.
- The user can rename each available team. The custom display name appears in Planner tabs, matrix headings and accessible names, picker locations, action feedback, and confirmation copy.
- Removing an empty team removes it immediately after the user saves the team configuration.
- Removing a team with assignments requires a destructive confirmation that names the affected team and assignment count. Confirmation removes the team's strings and assignments.
- Re-adding a removed category creates one empty string and starts with its canonical Senior, Reserves, or Youth display name, which the user can edit before saving.
- Removing the selected team selects a remaining team. In the tabbed matrix layout, keyboard focus moves to that remaining team's tab.
- Loading Planner, reloading the app, switching saves, and refreshing or promoting snapshots do not recreate a removed team or copy another save's names or availability.
- Current-score and potential-score optimization skip unavailable categories entirely.

## Invariants

- The only internal team identities are `senior`, `reserves`, and `youth`. Display names never select candidate pools, age rules, source mappings, persistence rows, or optimizer order.
- Team configuration belongs to an app save, not a snapshot, dump, bridge scan, or memory-derived value.
- Each save has at least one and at most three available categories.
- Available categories remain in canonical Senior, Reserves, Youth order regardless of display name or the order submitted by the WebView.
- Each available team has at least one string. An unavailable team has no strings and no assignments.
- Removing a populated team is atomic: either its assignments, strings, and availability row are all removed, or none are.
- Removing a team never changes the tactic, club-family sources, other teams, Academy data, shortlist data, snapshots, or another save.
- The backend rejects picker and string commands for unavailable categories even if a caller bypasses the UI.
- The optimizer never loads candidates or writes assignments for an unavailable category in either score mode.
- Display names are trimmed, non-empty, at most 40 Unicode scalar values, and unique within a save under case-insensitive comparison.
- Existing supported databases migrate with all three categories available and the canonical display names, preserving every existing Planner string and assignment.

## Non-goals

- Arbitrary team categories, more than three teams, category reordering, or custom internal identifiers.
- Snapshot-derived, bridge-derived, or automatically changing team availability.
- Dormant assignments, dormant strings, undo, or recovery of assignments after team removal.
- Preserving a removed team's custom display name after deletion. Re-adding the category starts from its canonical name.
- Changing age eligibility, optimizer ranking, tactic ownership, string ordering, assignment provenance, or score calculation.
- Replacing club-family sources or deriving native FM membership. [JAY-27](https://linear.app/jaycount/issue/JAY-27/replace-club-family-setup-with-single-club-selection) owns that later change.
- Moving the managed-club selector or redesigning Dashboard Club Setup.
- Updating snapshot, dump, bridge, Academy, Staff, shortlist, or Squad Overview membership contracts.

## Current-state map

- **Relevant backend components:** `src-tauri/src/features/planner/depth.rs` defines `PlannerTeam`, `PLANNER_TEAMS`, `get_depth`, `ensure_depth`, string mutations, assignment mutations, picker reads, and Clear all. `optimizer.rs::optimize_depth_with_basis` iterates the fixed `PLANNER_TEAMS` array and loads current or potential candidates for every category. `commands.rs` maps the depth model to IPC DTOs, and `src-tauri/src/lib.rs` registers Planner commands.
- **Relevant frontend components:** `src/features/planner/components/planner-depth-matrix.tsx` owns the selected team, responsive team tabs, action mutations, focus restoration, and cache reconciliation. `planner-depth-table.tsx`, `planner-slot-fit-picker.tsx`, and `planner-clear-all-control.tsx` still use canonical hard-coded team labels in visible or accessible copy. `src/app/routes/planner.tsx` composes the depth matrix.
- **Frontend contracts and test doubles:** `src/features/planner/types/club-family.ts` owns the fixed category union; `types/depth.ts` owns the depth DTO. `src/testing/planner-ipc-mock.ts`, `src/testing/setup.ts`, `src/app/routes/planner.test.tsx`, `e2e/tauri-ipc-stub.ts`, and `e2e/smoke.spec.ts` assume all three teams and canonical labels.
- **Data model:** schema v27 stores team identity only on `planner_strings` and `planner_club_sources`. `planner_strings` has a save/team/order uniqueness constraint; `planner_assignments` cascades from strings and keeps player UIDs unique per save. There is no availability or display-name row.
- **Persistence and migrations:** `src-tauri/src/db/migrations.rs` owns the ordered `PRAGMA user_version` registry and migration tests. `depth.rs::ensure_depth` currently inserts a first string for every category whenever Planner depth loads, which would recreate a removed team.
- **Existing behavioral assumptions:** `get_depth` creates three empty team DTOs before reading strings. `add_string` accepts every parseable category. Slot candidates use the requested category's current `planner_club_sources`. Clear all removes every save assignment. Existing save and snapshot changes invalidate the Planner query root.
- **Architectural seams:** Rust owns persistence, validation, transactions, candidate eligibility, optimization, and bounded IPC DTOs. React owns local drafts, modal state, display labels, selection, focus, and Query cache reconciliation. Stable category identity is already shared across Rust and TypeScript.
- **Project validation commands:** `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev check-rust` and `./scripts/dev check-app` are the affected-layer gates. Bridge tests are outside this feature because the bridge does not change.
- **Advisory risk evidence:** the Repowise index matched `e713e2449059`. It marks migrations, Planner depth, optimizer, matrix, route tests, IPC stubs, and smoke as high-churn surfaces. Migration preservation, direct command guards, cache reconciliation, responsive focus, and end-to-end browser behavior therefore require explicit tests and review.
- **Primary risks:** a lazy initializer silently recreating removed teams; a partial removal leaving assignments or strings behind; current or potential optimization still loading an absent category; display names leaking into domain identity; stale selection or focus after removal; stale team data crossing a save change; and migration changes affecting unrelated save-owned tables.

## Feature architecture

### Save-scoped team configuration

Migration v28 adds `planner_teams`, keyed by `(save_id, team)`, with a constrained stable category and bounded `display_name`. Row presence defines availability. The migration inserts canonical rows for Senior, Reserves, and Youth for every existing save and does not rebuild or mutate `planner_strings` or `planner_assignments`.

New saves continue to initialize Planner lazily. The initializer inserts the three canonical team rows only when the save has no `planner_teams` rows. After configuration exists, it ensures a first string only for rows that are present. It never inserts one missing category in isolation, because a missing row is a user-selected absence.

A single complete-configuration command receives the one-to-three available category/name pairs plus an explicit populated-removal confirmation flag. Rust trims and validates names, rejects duplicate categories and display names, derives removed and added categories from current persisted state, verifies whether removed categories contain assignments, and applies the replacement in one transaction. Removing a category deletes its strings and relies on the existing assignment foreign-key cascade. Adding a category inserts its row and one first string. Updating a category changes only its display name.

### Depth and optimizer behavior

`get_planner_depth` returns only available categories in canonical order, and each returned team carries its persisted display name. Direct string and picker operations verify that their category remains available. Assignment commands continue to derive the category from a persisted string, so a removed team cannot be targeted after its strings are deleted.

Both optimizer score modes load the persisted available-category list once inside their transaction and iterate that list instead of `PLANNER_TEAMS`. Candidate source selection, age eligibility, lane matching, manual reservations, assignment provenance, and canonical priority remain unchanged for categories that are available.

### Planner presentation and management

The matrix derives tabs, combined headers, captions, picker locations, status text, and confirmation lists from the returned team rows. Stable category values remain keys and IPC arguments; display names are presentation only. Keyboard tab movement uses the available subset rather than the static three-item array.

The Squad depth toolbar adds a secondary **Manage teams** action. Its Modal uses local draft state and the existing Button, TextField, Modal, and destructive-confirmation patterns. It lists the three supported internal categories, lets the user include one to three, and shows a name field for each included category. The fixed category set makes the maximum structural rather than a numeric free-form limit.

On a successful save, React replaces the depth Query cache, invalidates all slot-candidate queries, closes stale picker or string-menu state, and clears outdated action feedback. If the selected category was removed, it chooses the next category in canonical order, falling back to the first remaining category. In the narrow layout it moves focus to that tab after the Modal closes; in the combined layout it restores focus to **Manage teams** because no team tab exists. Keying the matrix to the active save prevents local drafts, selection, and Modal state from crossing a save change.

### Relationship to JAY-27

JAY-28 does not depend on JAY-27. This feature owns which stable categories are available and how they are named. JAY-27 may later replace `planner_club_sources` with FM-derived membership, but every membership consumer must still accept the stable category and respect `planner_teams` availability. Snapshot refreshes can change members within an available category but cannot add, remove, or rename categories.

## Uncertainty register

### Known

- Linear JAY-28 requires user-managed availability, any category removable down to one, assignment deletion after confirmation, and save-scoped behavior independent of snapshots.
- The developer added editable per-save display names and a maximum of three teams.
- The current fixed category union, source mappings, age rules, and optimizer order already use stable lowercase identifiers.
- Existing supported saves have all three teams because `ensure_depth` creates missing strings for all categories.
- `planner_assignments` already cascades when its owning string is deleted.
- Active-save and effective-snapshot changes already invalidate the Planner query root.
- JAY-27 is Todo and related to JAY-28, but neither issue blocks the other in Linear.

### Assumptions

- The Planner depth toolbar is the least surprising management location because availability and display names affect only Planner depth in this feature. A later My Club or Settings feature may move the control without changing the Rust contract.
- Forty characters is sufficient for a user-facing team name and keeps tabs, headings, and confirmation copy bounded. Components still truncate where their existing layout requires it while preserving the full accessible name or title.
- Display names must be unique within one save so picker locations, destructive copy, and tabs remain unambiguous.
- Removing a team deletes its display-name row along with its depth structure. Re-adding uses the canonical default name and one empty string.
- The existing browser-stub smoke seam plus Rust migration/service tests provides proportionate automated evidence. A native WebView pass remains useful but is not required to design the first commit.

### Decisions

- Use a dedicated `planner_teams` table. Do not derive availability from `planner_strings`, because strings are depth structure and loading currently creates them as a side effect.
- Use row presence as the availability contract. Do not add a second boolean state or dormant team structure.
- Backfill all three categories for existing saves. This preserves every current Planner row and current behavior without inferring intent from snapshots.
- Use one complete-configuration mutation so add, rename, remove, string initialization, and assignment cleanup share one transaction and one cache reconciliation path.
- Require confirmation only when removed categories contain assignments, matching current populated-string safety. Empty team removal does not need a second destructive dialog.
- Keep canonical internal order and category-based age rules after renaming.
- Keep JAY-27 outside this PR. Record the availability seam in the final architecture so its future membership implementation consumes it.
- Use one PR. The backend contract, dynamic presentation, and management UI share one review surface and have no independently valuable publication boundary.
- No ADR is required. The persistence choice is local to the established Planner ownership boundary and is fully explained by this ledger, migration tests, and the final current-state documentation.

### Unknowns

- Whether a native desktop environment will be available for final keyboard and minimum-window verification. If unavailable, final handoff must state the gap and rely on component plus Playwright evidence without claiming a native pass.

### Risks

- Inserting canonical rows for each missing category instead of only for an entirely uninitialized save would undo user deletions.
- Deleting availability before strings or outside one transaction could leave stale depth or assignments.
- A direct `add_planner_string` or picker call could recreate or query an absent category unless Rust validates availability at that boundary.
- Potential optimization has a separate candidate-loading path and can regress independently from current-score optimization.
- Long, duplicate, or empty names can make tabs and destructive copy ambiguous unless both React and Rust validate them.
- Removing the selected team while a picker, menu, or responsive-layout transition is active can leave focus on a detached element.
- Adding the command without updating `src/testing/setup.ts` and both IPC stubs would make UI tests pass against an incomplete desktop contract.
- Migration registry and schema assertions are high-churn shared surfaces; a version or index expectation can fail outside Planner unless the v28 change is exact.

## Walking skeleton

The first two commits form the walking skeleton. A migrated save is changed through the Rust contract from Senior, Reserves, and Youth to renamed Senior and Youth only. `get_planner_depth` returns the two available teams and their display names, the removed Reserves strings and assignments are gone, both optimizer modes skip Reserves, and a focused Planner route test renders only the two renamed teams in tabs, the combined matrix, picker locations, and Clear all copy. This proves persistence, deletion safety, optimizer scope, IPC shape, and presentation before the management Modal is added.

## Delivery plan

### PR 1 — Support configurable Planner teams

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/optional-planner-teams`

**Base branch:** `main`

**Base ref:** `e713e2449059b162c392407c3b042cec3196e067`

**Publication provider:** GitHub (`JG1995/fm-valuescout`)

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict required `check` aggregate after all applicable frontend, browser, Rust, and release-validation jobs pass

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(planner): support configurable squad teams`

**Purpose:** Add one save-scoped team contract that safely controls Planner depth, naming, candidate access, optimization, and presentation for any one-to-three supported categories.

**Depends on:** The completed Squad Planner, Squad Optimizer, Planner Workspace Redesign, Potential Role Scores, and Planner Module Refactor already on `main`. JAY-27 is related but not a dependency.

**Merge to trunk when:** All three commits have cleared their commit gates and reviews, feature validation and close-out have passed, documentation reflects the implemented contract, and the final PR's required `check` aggregate is green.

#### Commit 1 — Persist save-scoped team settings

**Status:** Completed

**Provisional commit:** `feat(planner): persist save-scoped team settings`

**Work:** Add the explicit save-owned availability and display-name contract, make depth and both optimizer modes consume it, and provide one transactional IPC mutation for complete team configuration.

**Out of scope:**

- React rendering or team-management controls.
- Club-family source derivation or JAY-27 behavior.
- Changes to optimizer scoring, age limits, matching, or tactic state.
- Current-state architecture or design claims before the feature is implemented.

**Implementation packet:**

- Add migration v28 and focused upgrade/fresh-schema coverage before changing Planner behavior.
- Add a focused Planner-private team settings module rather than growing club-family `service.rs` or mixing the new responsibility into optimizer code.
- Make the depth initializer distinguish a save with no team configuration from a save with intentionally missing categories.
- Add the complete replacement service and command, including authoritative name, count, duplicate, availability, and confirmation validation.
- Route depth reads, string and picker guards, and both optimizer modes through the same persisted available-category list.
- Preserve current behavior for an untouched or migrated save with all three canonical teams.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — add `planner_teams`, v28 registration, all-existing-save backfill, schema/index expectations, and populated-v27 preservation coverage.
- `src-tauri/src/features/planner/teams.rs` — own default names, initialization, canonical available-team reads, input normalization, complete replacement, populated-removal checks, string cleanup, and first-string creation.
- `src-tauri/src/features/planner/teams_tests.rs` — prove initialization, validation, add/rename/remove, confirmation, rollback, save isolation, restoration, and preservation of unrelated Planner state.
- `src-tauri/src/features/planner/mod.rs` — register the focused module and tests.
- `src-tauri/src/features/planner/depth.rs` — return only configured teams with display names, stop recreating individual missing categories, and reject unavailable-team picker and string operations.
- `src-tauri/src/features/planner/depth_tests.rs` — update the default contract and add direct-command absence regressions without duplicating team-service tests.
- `src-tauri/src/features/planner/optimizer.rs` — iterate the persisted available subset once per optimization transaction for both score bases.
- `src-tauri/src/features/planner/optimizer_tests.rs` — prove current and potential modes neither load candidates nor create assignments for an absent category while preserving canonical priority among remaining teams.
- `src-tauri/src/features/planner/commands.rs` — add bounded input and output DTOs, include `displayName` in team depth DTOs, and expose the complete configuration mutation.
- `src-tauri/src/features/planner/test_support.rs` — initialize and query the new team contract in shared Planner fixtures.
- `src-tauri/src/lib.rs` — register the new command and no other capability.

**Behavior and data flow:**

- Migration opens a supported v27 database, creates the settings table, inserts three canonical rows per existing save, and leaves strings, assignments, tactics, club sources, snapshots, and other save-owned tables unchanged.
- `ensure_depth` initializes all three settings only when the save has zero settings, then creates missing first strings only for the returned available rows.
- The command accepts one-to-three category/name inputs and `confirmPopulatedRemoval`; Rust normalizes and validates them before any write.
- Inside one transaction, Rust identifies removed categories, counts their assignments, rejects unconfirmed populated removal without mutation, deletes removed strings so assignments cascade, upserts retained or added settings, creates one string for additions, and returns reconciled depth after commit.
- `get_depth` constructs only available team DTOs in canonical order and resolves existing assignments exactly as before.
- Picker and add-string entry points reject a stable category that lacks a settings row.
- The optimizer loads the available subset, removes prior optimizer rows, and applies its existing allocation pipeline only to that subset for `current` and `potential`.

**Ordered implementation steps:**

1. Add a failing v27 migration test that expects three canonical settings while preserving populated Planner and unrelated save state.
2. Add the v28 schema and registry entry until fresh and upgrade tests pass.
3. Add failing service tests for two-team rename/removal, confirmation, rollback, min/max and name validation, restoration, and save isolation.
4. Implement the settings module and integrate depth initialization, reads, and direct command guards until those tests pass.
5. Add failing current and potential optimizer tests with absent-team-only sentinel data that would fail if either loader queries the removed category.
6. Iterate the persisted subset in the optimizer and keep the existing matcher unchanged.
7. Add the IPC DTO and command registration, then run the affected Rust gate and the full repository gate.

**Tests and proof:**

- RED migration proof: v27 has no team settings; the test expects canonical backfill and unchanged string, assignment, tactic, source, snapshot, and other-save rows.
- GREEN migration proof: a fresh database exposes the constrained table and latest schema version 28; a populated v27 database retains all three existing teams and data.
- RED service proof: current `ensure_depth` recreates every category, and no complete settings mutation exists.
- GREEN service proof: Senior renamed to First Team plus Youth renamed to U19 persists as exactly two rows; unconfirmed populated Reserves removal changes nothing; confirmed removal deletes only Reserves strings and assignments; the final category cannot be removed; adding Reserves back creates one empty string.
- Boundary coverage: reject zero teams, more than the three supported identities, duplicate categories, unknown identities, blank or overlong names, and case-insensitive duplicate display names.
- Direct-call coverage: absent-team slot candidates and string creation fail without recreating the team.
- Optimizer coverage: both current and potential modes skip sentinel data reachable only through the absent team's source and create no assignment in that category.

**Patterns to verify:**

- Migration registration and populated upgrade tests in `src-tauri/src/db/migrations.rs`.
- Complete transactional replacement and validation in `planner/service.rs::save_club_family`, without coupling the new settings to club-family ownership.
- Lazy per-save initialization in `depth.rs::ensure_depth` and `tactic.rs::get_tactic`.
- Existing string deletion cascade and populated-string confirmation in `depth.rs::remove_string`.
- Current/potential shared allocation pipeline in `optimizer.rs::optimize_depth_with_basis`.
- Thin DTO and command mapping in `planner/commands.rs` plus command registration in `src-tauri/src/lib.rs`.

**Constraints and non-goals:**

- Do not rebuild existing Planner tables or add a new dependency.
- Do not store availability on snapshots or infer it from sources, players, strings, or memory fields.
- Do not pass display names into source, age, score, or allocation logic.
- Do not retain dormant strings or assignments.
- Keep every error safe and bounded at the IPC boundary.

**Dependencies and sequencing:**

- Starts from schema v27 and the current Planner module split at base ref `e713e2449059b162c392407c3b042cec3196e067`.
- Commit 2 depends on the added `displayName` DTO and variable-length team response.
- JAY-27 must not be implemented inside this commit. If it lands on `main` before build, re-inspect its membership seam and replan affected queries before changing code.

**Validation:** Run `./scripts/dev format src-tauri/src/lib.rs`, then `./scripts/dev check-rust`, then `./scripts/dev check`. GREEN evidence must include migration v28, direct absent-team guards, assignment cascade, save isolation, and both optimizer score modes.

**Stop conditions:** Stop and replan if supported v27 data contains a Planner category outside the three-value union; if row-presence initialization cannot distinguish a new save from an intentional deletion; if removal cannot be atomic without rebuilding existing Planner tables; if JAY-27 changes the category identity or source contract first; or if a potential optimizer path cannot be proven to skip absent-team data.

**Review mandate:**

- Trace v27-to-v28 migration with populated strings, assignments, tactics, club sources, snapshots, and two saves for loss or cross-save writes.
- Prove `ensure_depth` initializes only a zero-row save and never restores one missing category.
- Verify rejected validation and unconfirmed deletion leave the transaction unchanged.
- Verify strings cascade assignments and no orphaned rows or unavailable-team strings remain.
- Trace both optimizer loaders and direct picker/string commands through the availability guard.
- Check that display names remain presentation metadata and stable identifiers still own order, age rules, sources, and persistence.

#### Commit 2 — Render configured squad teams

**Status:** Completed

**Provisional commit:** `feat(planner): render configured squad teams`

**Work:** Make the existing depth matrix, picker, actions, captions, selection, and responsive focus behavior consume the backend's variable-length, renamed team DTO without yet adding the management Modal.

**Out of scope:**

- The user-facing add, remove, or rename form.
- Any new backend behavior beyond adapting frontend types to Commit 1's DTO.
- Club Setup, Squad Overview, JAY-27 membership, or design-system changes.

**Implementation packet:**

- Update the typed depth contract and shared test fixtures first so missing display names fail at compile time.
- Replace static render loops and label maps with the canonical order already returned by Rust.
- Keep stable category IDs for React keys, refs, query keys, and IPC calls while using display names for every visible and accessible location.
- Make team-tab keyboard movement and selected-team fallback operate on the available subset.
- Pass one team-name map into the picker and action components instead of creating local hard-coded maps.
- Key the matrix to the active save so transient state cannot cross saves.

**Files and responsibilities:**

- `src/features/planner/types/depth.ts` — add the persisted `displayName` to `PlannerDepthTeam`.
- `src/features/planner/types/club-family.ts` — retain the fixed identity union and export canonical default names only where missing-category presentation needs them later.
- `src/features/planner/components/planner-depth-matrix.tsx` — derive ordered teams, selection, keyboard cycling, label mapping, layout width, status, and focus targets from `depth.teams`.
- `src/features/planner/components/planner-depth-table.tsx` — use supplied display names for combined headings, captions, matrix names, cells, and string-removal context.
- `src/features/planner/components/planner-slot-fit-picker.tsx` — format current and prior assignment locations with the persisted display-name map.
- `src/features/planner/components/planner-clear-all-control.tsx` — receive the available display-name list and name only those teams in destructive copy.
- `src/app/routes/planner.tsx` — key `PlannerDepthMatrix` by active save ID to reset local selection and transient dialogs on save replacement.
- `src/testing/planner-ipc-mock.ts` and `src/testing/setup.ts` — make default and custom depth responses satisfy the new DTO without adding management behavior yet.
- `src/app/routes/planner.test.tsx` — add the two-team renamed rendering, action copy, picker location, keyboard order, save change, and responsive focus regressions.

**Behavior and data flow:**

- Query returns one-to-three team rows with stable category and display name.
- Matrix renders the returned list only and derives combined width from only those strings.
- Narrow layout tabs cycle through only returned categories; combined layout groups only returned categories.
- Picker cells continue to send the stable category, but all source and destination location copy resolves through the display-name map.
- Clear all still invokes the same all-save command, while its Modal lists only currently available display names.
- An active-save change remounts the matrix and consumes the newly fetched save-scoped names and availability.

**Ordered implementation steps:**

1. Add `displayName` to the TypeScript DTO and update the default mock until type checking passes.
2. Add a failing route fixture with renamed Senior and Youth only; assert no Reserves tab, header, caption, cell label, picker location, or confirmation copy.
3. Refactor matrix ordering, selection, refs, keyboard movement, and layout calculations to the returned subset.
4. Thread the display-name map through the table, picker, and Clear all control.
5. Add selected-team fallback, responsive focus, and active-save replacement regressions.
6. Run focused Planner tests, the frontend gate, and the full gate.

**Tests and proof:**

- RED route proof: hard-coded `PLANNER_TEAMS` and `TEAM_LABELS` still render Reserves and canonical names against a two-team renamed fixture.
- GREEN narrow-layout proof: only First Team and U19 tabs exist; Arrow, Home, and End keys cycle within that subset; no hidden Reserves panel exists.
- GREEN combined-layout proof: only two column groups render and the accessible caption names only First Team and U19.
- Picker proof: existing and target locations use renamed display names while IPC calls retain lowercase stable identities.
- Action proof: Clear all names only First Team and U19; optimizer success remains basis-specific and does not invent absent-team copy.
- Context proof: switching to a save with a different subset resets selected team and transient state; snapshot refresh keeps the same configured subset.
- Focus proof: selection falls back to a remaining category without focusing a detached tab during a responsive mode change.

**Patterns to verify:**

- Existing focus context, tab refs, and ResizeObserver flow in `PlannerDepthMatrix`.
- `PlannerDepthTable`'s semantic table, captions, `headers` relationships, and cell accessible names.
- `PlannerSlotFitPicker`'s source/destination confirmation and focus restoration.
- The active-save key already used by `PlannerTacticEditor` in `src/app/routes/planner.tsx`.
- Query mock cloning in `src/testing/planner-ipc-mock.ts` so tests do not share mutable names or team arrays.

**Constraints and non-goals:**

- Do not sort teams by display name or frontend constants.
- Do not recompute domain state or filter optimizer output in React.
- Do not add a second Query or Zustand copy of depth or team settings.
- Preserve the current semantic table, tab roles, Modal focus rules, minimum matrix column width, and bounded overflow.

**Dependencies and sequencing:**

- Depends on Commit 1's variable-length `PlannerDepthDto` and `displayName` field.
- Commit 3 depends on this commit's available-team selection and label plumbing.

**Validation:** Run `./scripts/dev format src/features/planner src/app/routes/planner.tsx src/app/routes/planner.test.tsx src/testing`, then `./scripts/dev test src/app/routes/planner.test.tsx`, then `./scripts/dev check-app`, then `./scripts/dev check`.

**Stop conditions:** Stop and replan if the backend response can contain zero teams; if a display name must become part of a Query key or IPC identity; if selected-team focus cannot be restored without changing the shared Modal contract; or if active-save replacement exposes stale local state outside the matrix boundary.

**Review mandate:**

- Search for every hard-coded Senior, Reserves, and Youth label in Planner depth presentation and retain only canonical defaults or unrelated club-source copy.
- Verify tabs, combined headers, captions, cells, picker locations, status, and confirmation copy use persisted names and omit absent teams.
- Exercise one-team, two-team, and three-team keyboard navigation and responsive mode transitions.
- Verify active-save changes cannot retain a removed category, custom name, open picker, menu, or stale focus target.
- Confirm stable category identifiers still drive keys, refs, mutations, and backend arguments.

#### Commit 3 — Add squad team management

**Status:** Completed

**Provisional commit:** `feat(planner): add squad team management`

**Work:** Add the accessible user flow for including, removing, restoring, and renaming the three supported team categories, with populated-removal confirmation and full cache and focus reconciliation.

**Out of scope:**

- A general Settings or My Club page.
- Drag reordering, arbitrary category creation, undo, dormant data, or snapshot automation.
- Club-family source controls or JAY-27.
- New shared form libraries or design primitives.

**Implementation packet:**

- Add one typed API wrapper around the complete backend mutation.
- Add a focused Planner-local management component using existing UI primitives and local draft state.
- Treat the fixed category list as the only add choices, enforce one selected category in the UI, and repeat authoritative validation in Rust.
- Derive assignment counts from the current depth only to decide which confirmation copy to show; the backend remains authoritative and rejects stale unconfirmed removal.
- On success, reconcile depth and candidates before moving selection or focus.
- Extend both test IPC seams and browser smoke with the exact command and result behavior.

**Files and responsibilities:**

- `src/features/planner/api/save-planner-teams.ts` — invoke `save_planner_teams` with the complete one-to-three settings list and populated-removal confirmation flag.
- `src/features/planner/components/planner-team-management.tsx` — own the Manage teams trigger, draft category inclusion and names, validation feedback, assignment-removal summary, destructive confirmation, pending state, error retention, and focus contract.
- `src/features/planner/components/planner-depth-matrix.tsx` — compose the new control, provide depth and assignment counts, close stale interaction state, reconcile Query caches, choose the fallback selected team, and schedule the required focus target.
- `src/features/planner/types/club-family.ts` — keep the stable category union and canonical names used when a removed category is restored.
- `src/features/planner/components/planner-club-family-panel.tsx` — keep the no-associated-club explanation accurate for configured team availability.
- `src/testing/planner-ipc-mock.ts` and `src/testing/setup.ts` — implement controllable success, validation failure, pending, and populated-removal behavior for the new command.
- `src/app/routes/planner.test.tsx` — prove add, remove, rename, validation, confirmation, rollback, cache invalidation, selection fallback, focus, and save isolation at the user-visible seam.
- `e2e/tauri-ipc-stub.ts` — model the same complete replacement semantics for browser smoke, including string and assignment cleanup.
- `e2e/smoke.spec.ts` — exercise rename plus populated removal in the narrow layout and restored-team rendering without asserting Rust or SQLite behavior.

**Behavior and data flow:**

- **Manage teams** opens a labelled Modal and focuses the first category control. Existing available categories and display names populate the draft; missing categories appear as addable canonical options.
- The user selects one-to-three supported categories and edits names. Save stays unavailable for zero categories or local name errors.
- When no removed category has assignments, Save sends the complete configuration directly.
- When one or more removed categories have assignments, Save shows a destructive confirmation listing each display name and assignment count. Confirm sends the same draft with the explicit backend confirmation flag.
- Pending submission prevents duplicate save, close, optimize, Clear all, and conflicting string actions.
- Success replaces `plannerKeys.depth()`, invalidates `plannerKeys.slotCandidates()`, closes the management flow, clears removed-team interaction state, reports success, and applies selected-team and focus fallback.
- Failure keeps the draft and relevant Modal open with an inline safe error. No optimistic deletion occurs.

**Ordered implementation steps:**

1. Add a failing route test for the Manage teams trigger and a renamed two-team success result.
2. Add the API wrapper and test-mock command shape.
3. Build the local draft Modal with category count and name validation using existing primitives.
4. Add failing populated-removal, rejected-removal, duplicate-submit, and rollback tests.
5. Implement destructive confirmation and authoritative mutation handling.
6. Add selection, focus, stale interaction cleanup, candidate invalidation, active-save isolation, and restoration tests.
7. Extend the Playwright IPC stub and add one complete browser management flow at supported desktop widths.
8. Run focused tests, smoke, frontend and full gates.

**Tests and proof:**

- RED UI proof: no Manage teams action or `save_planner_teams` IPC path exists.
- Rename proof: First Team and U19 replace canonical labels everywhere after one successful save and survive a depth refetch for the same save.
- Empty removal proof: an unassigned category disappears without a second confirmation and no absent tab or matrix group remains.
- Populated removal proof: the confirmation names the custom team and count; Cancel preserves the matrix and focus; backend failure preserves the draft and assignments; success removes the team and its players from all Planner locations.
- Minimum/maximum proof: the user cannot submit zero categories, and only the fixed three choices exist.
- Name proof: blank, over-40, and case-insensitive duplicate names show field-linked errors and do not invoke IPC.
- Selection proof: removing the selected middle or edge category picks the next canonical remaining category and focuses its tab in narrow mode; a one-team result remains selected.
- Cache proof: success replaces depth and forces the next picker open to refetch candidates; failure changes neither cache.
- Context proof: switching saves closes the Modal, discards its draft, and shows only the target save's settings.
- Smoke proof: browser stub renames Senior, removes populated Reserves after confirmation, omits it at narrow and combined widths, and restores Reserves with one empty string.

**Patterns to verify:**

- Local draft and failed-save retention in `PlannerTacticEditor` and current Club Setup controls.
- Destructive confirmation and focus restoration in `PlannerStringRemovalConfirmation` and `PlannerClearAllControl`.
- Query cache replacement plus candidate invalidation in existing string, assignment, Clear all, and optimizer mutations.
- Shared `TextField`, `Button`, and `Modal` APIs; do not add a form dependency for three fields.
- Playwright's stateful Planner stub and its explicit warning that smoke does not prove Rust or SQLite.

**Constraints and non-goals:**

- Keep one primary action in each Modal and use text plus structure, not color alone, for removal and error state.
- Every input has a visible label and field-linked error; every action is keyboard reachable; Escape and Cancel restore focus.
- Do not expose internal category identifiers as editable text.
- Do not optimistically remove teams or assignments before the Rust result.
- Do not add arbitrary names to the fixed category array or allow a fourth row.

**Dependencies and sequencing:**

- Depends on Commit 1's complete mutation and Commit 2's dynamic rendering, selection, and label plumbing.
- Completes the sole PR's implementation. Run `$workflow-finish-feature` after this commit clears checkpoint.

**Validation:** Run `./scripts/dev format src/features/planner src/app/routes/planner.tsx src/app/routes/planner.test.tsx src/testing e2e`, then `./scripts/dev test src/app/routes/planner.test.tsx`, then `./scripts/dev check-app`, then `./scripts/dev smoke`, then `./scripts/dev check`.

**Stop conditions:** Stop and replan if management requires a fourth category or custom internal identity; if the backend mutation cannot distinguish stale unconfirmed populated removal; if Modal focus restoration conflicts with the required selected-tab focus and cannot be resolved locally; if a save switch can submit the prior save's draft; or if the browser stub cannot model the command without diverging from the typed DTO.

**Review mandate:**

- Verify the WebView cannot submit zero, duplicate, unknown, blank, overlong, or fourth-team state and that Rust still rejects bypassed input.
- Trace Cancel, Escape, backend failure, pending duplicate clicks, and active-save change for draft, cache, assignment, and focus safety.
- Verify populated removal copy names only affected custom teams and that success deletes all of their visible assignment references.
- Confirm selection and focus fallback for removed first, middle, last, and sole-disallowed categories in narrow and combined layouts.
- Verify candidate invalidation and stale picker/menu cleanup after add, remove, and rename.
- Compare the finished UI with the existing Modal, field, toolbar, and destructive-action patterns and the 1280×800 design contract.

## Active work

**PR:** PR 1 — Support configurable Planner teams

**Commit:** Implementation complete; feature close-out remains

### RED proof

The focused Planner route and browser tests fail before the complete team-settings IPC command and management flow exist.

### Expected outcome

The Planner toolbar lets the user include, remove, and rename the three supported categories with accessible validation and populated-removal confirmation. Successful saves reconcile depth and candidates, close stale interaction state, and move selection and focus to a remaining canonical category when needed; rejected saves retain the draft and existing assignments.

### Explicit exclusions

- No general Settings or My Club page, arbitrary categories, reordering, undo, or dormant assignments.
- No Club Setup source changes, JAY-27 membership derivation, snapshot or bridge behavior, or new form dependency.
- No optimistic removal or display-name use as a stable category identity.

## Discoveries and replanning

- Linear JAY-28's six open questions are resolved by the developer: availability is user-managed; any supported category may be removed while one remains; persistence design is delegated; assignments are deleted; existing data has low personal value; and snapshots or memory reads do not manage teams.
- The added rename requirement makes `planner_strings` presence unsuitable as the source of truth because strings do not own category metadata and are created during depth reads.
- JAY-27 will replace membership derivation but not the stable category identity. The two features can land independently if JAY-27 later consumes `planner_teams` availability.
- Repowise was fresh at the planning base but reported no coverage map. Impacted-test selection therefore comes from direct source, current colocated tests, the route suite, and browser smoke rather than coverage-backed recommendations.
- Commit 1 review required complete team replacement to delete and reinsert settings inside the existing transaction so any legal display name remains saveable, and exact pre/post fixture tuples to prove v27 migration preservation; both are now covered.
- Commit 3 keeps the fixed category set in the management Modal and chooses focus from the post-save DOM, so removing a selected team remains safe when the responsive matrix changes from tabs to the combined layout.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Persist save-scoped team settings | Pending record | v28 persistence, availability guards, transactional replacement, cleanup, and optimizer integration complete | Sol Medium accepted after two correction rounds | Repowise refresh unavailable; direct source and test evidence used |
| PR 1 | Render configured squad teams | Pending record | Variable-length renamed team rendering across tabs, matrix headings and captions, picker locations, Clear all copy, keyboard subset navigation, and active-save remount; route coverage includes two-team, one-team, and save-replacement state | Sol Medium accepted after one correction round; no findings remain | None |
| PR 1 | Add squad team management | Pending record | Management Modal supports one-to-three category selection, save-scoped renaming, populated-removal confirmation, restoration, validation, cache and focus reconciliation, plus matching IPC doubles and browser smoke coverage | Sol Medium accepted after one correction round; no findings remain | Feature smoke is green for Planner management; the full smoke suite retains an unrelated pre-existing My Staff CA-boost timing failure |

## Final validation

1. Run `./scripts/dev format` and verify a second run makes no changes.
2. Run `./scripts/dev test` for the complete Vitest suite.
3. Run `./scripts/dev check` for Biome, TypeScript, secretlint, Rust format, Clippy, and Rust tests, including migration v28 and both optimizer modes.
4. Run `./scripts/dev smoke` for the stateful Planner management flow, absent-team rendering, current and potential optimizer controls, Clear all copy, and supported desktop widths.
5. Verify a migrated populated v27 database starts with all three canonical teams and retains existing strings, assignments, tactic, club sources, saves, snapshots, Academy data, and shortlist data.
6. Verify one-team, Senior-plus-Youth, and all-three configurations across app reload, active-save changes, and effective-snapshot refresh or promotion.
7. Verify renamed teams in tabs, combined headings, matrix and cell accessible names, picker locations, action feedback, and destructive confirmation copy.
8. Verify confirmed team removal deletes its assignments and strings, leaves other state unchanged, and never reappears after Planner reload.
9. Verify current and potential optimization never load candidates for or assign players to an absent category.
10. At feature close-out, run Repowise change-risk evidence against the exact recorded feature range if the index remains usable, then complete the required feature review and documentation reconciliation.
11. When a native WebView is available, verify keyboard-only management and selection fallback at 1280×800, 1600×900, and 1920×1080. If unavailable, record the gap without treating Playwright as native proof.

## Documentation impact

Complete during reconciliation. Expected current-state owners after implementation are `.wiki/ARCHITECTURE.md` for schema v28, save-scoped team data flow, IPC, depth, and optimizer behavior; `.wiki/DESIGN.md` for Manage teams, dynamic team labels, destructive confirmation, and focus behavior; `.wiki/TODO.md` for completion state; and the condensed completed feature record. No ADR is planned.
