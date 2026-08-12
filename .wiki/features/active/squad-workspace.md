# Squad Workspace

## Status

Active

## Intent

Turn the current Planner surface into a broader Squad workspace. Give the user one club-scoped player overview, keep the existing depth planner and tactic editor as separate pages inside that workspace, move club-family setup to the Dashboard, add dedicated CSV import entry points, and apply the two approved player-development actions across the configured squad.

## User-visible behavior

- The primary navigation label changes from **Planner** to **Squad** while the stable route remains `/planner`.
- The Squad route has URL-backed **Squad**, **Planner**, and **Tactic** pages. Squad is first and is the default.
- The existing depth matrix moves from **Squad** to **Planner** without changing its tactic, assignment, optimizer, or persistence behavior.
- **Club Setup** leaves the Squad tab strip and appears on the Dashboard. The existing Dashboard CSV import remains available.
- Squad Overview shows only current-snapshot players whose exact current-club name belongs to the configured club family. The union of primary, Reserves, and Youth source clubs is deduplicated by player UID.
- When no club family is configured, Squad Overview explains that setup is required and links to the Dashboard Club Setup section. A missing snapshot keeps the existing Load Data recovery path.
- The overview uses the Search table's fixed columns: Name, Age / DOB, Nationality, Club, Division, CA, PA, and Value. It has no filters. Every header sorts, and each player name links to `/players/$uid`.
- **Upload Moneyball CSV** and **Upload Youth Academy CSV** each open a modal with one drag-and-drop target and a keyboard-accessible browse action. Each action accepts only its named supported format, imports one file into the active save and current snapshot, and reports the existing total, stored, and skipped counts.
- The Dashboard's existing auto-detect CSV import remains in place for the developer-facing workflow.
- **Boost all CA** applies the existing closed CA action to eligible squad players sequentially. Players aged 20 or younger receive +5, players aged 21 through 28 receive +10, and players aged 29 or older receive no CA boost. Each eligible target remains capped by PA and 200.
- **Make all Wonderkids** applies the existing closed Wonderkid Mentality action sequentially. For each player, only known Ambition, Professionalism, and Determination values at or below 10 are rerolled to inclusive random values from 11 through 20. Unknown and already-high values remain unchanged.
- Both squad-wide actions require confirmation, prevent duplicate submission, and report updated, skipped, and failed player counts. A proven no-write player-local rejection does not undo earlier successes or stop later eligible players.
- If the active save or current snapshot changes, or a bridge or reconciliation result leaves FM state uncertain, the command stops before another FM write and reports the partial result plus a Load Data recovery path.

## Invariants

- The running FM26 process remains the source of truth, and the C# bridge remains the only process-memory writer.
- The bridge continues to receive one closed action for one player at a time. No arbitrary address, field, value, UID list, or general batch-write payload crosses the bridge protocol.
- The WebView cannot choose squad player UIDs, CA increments, target values, or Wonderkid random values. Rust derives the current club-family cohort and all eligibility from the active snapshot.
- Squad membership is exact current-club equality against the configured club-family source names. `team_level` does not restrict the overview, and one player appears once even when a primary club seeds several Planner teams.
- Squad reads and writes use only the active save's effective current snapshot and its immutable context token.
- A verified player result is reconciled into SQLite before the next player begins. Determination changes recompute that player's current role scores in the same transaction.
- Ineligible players, including players aged 29 or older for a CA boost, are skipped without an FM write. A bridge-proven player-local rejection with no write counts as failed and processing continues. Recovery-required bridge, rollback, verification, context, or reconciliation uncertainty stops processing before another bridge request.
- CSV imports remain bounded, exact-UID matched, and supplementary. They never create players, replace memory-owned fields, expose paths, or write a detected format that differs from the selected modal.
- React remains presentation-focused. Rust owns sorting, pagination, club-family membership, bulk eligibility, sequential orchestration, context checks, CSV format enforcement, and SQLite reconciliation.
- Existing Planner, Tactic, Dashboard import, Search, profile, Academy, save, snapshot, and bridge behavior stays unchanged unless this ledger states otherwise.

## Non-goals

- Renaming the `/planner` route or adding new top-level routes for the three Squad pages.
- Removing the Dashboard or its existing CSV import panel.
- Adding Squad filters, editable cells, row selection, comparison, export, custom columns, or historical snapshots.
- Changing Planner assignments, optimizer behavior, tactics, club-family persistence, or Academy eligibility.
- Adding arbitrary player editing, custom boost values, undo, parallel FM writes, an all-or-nothing batch transaction, or a new bridge batch protocol.
- Adding new CSV formats, CSV-only players, historical Moneyball seasons, import history, or a second identity source.
- Claiming that browser IPC stubs prove native drag-and-drop, native dialog, SQLite-file, WebView, or live-FM integration.

## Current-state map

- Relevant components: `src/app/routes/planner.tsx` composes the Squad overview, CSV actions, Planner depth, and Tactic workspaces; `src/features/squad/` owns the overview query and table; `src/features/csv-import/` owns the shared context-bound importer and Squad modals; `src/features/planner/components/planner-workspace-tabs.tsx` owns tab semantics; `src/app/routes/index.tsx` composes Dashboard snapshot, Club Setup, and CSV panels; `src/app/components/app-nav-rail.tsx` labels the route Squad.
- Data model: save-scoped `planner_club_settings` and `planner_club_sources` define the primary and associated club names; current-snapshot `players.current_club` supplies exact membership; no schema change is required.
- Persistence and migrations: Planner club-family settings remain save-scoped. Moneyball enrichment remains snapshot-owned and Youth career enrichment remains save-owned. Player boosts reconcile current-snapshot player rows and affected role scores. No migration is planned.
- Existing behavior: `/planner?view=squad` is the configured club-family overview, `/planner?view=planner` is the depth matrix, and `/planner?view=tactic` is the tactic editor. Club Setup is Dashboard-only at `/#club-setup`; configured and unconfigured saves both default to Squad.
- Architectural seams: Rust `features/planner` owns club-family discovery and persistence; Rust `features/search` demonstrates paginated fixed-column player queries; Rust `features/csv_import` owns bounded imports; Rust `features/player` owns the two UID-only boost commands and snapshot reconciliation; Rust `features/memory_read` and the C# bridge own serialized action-specific FM writes.
- Frontend analogues: Search provides the fixed columns, sortable headers, row keyboard behavior, virtualization, and profile navigation; Modal provides focus trapping and restoration; the current CSV panel provides context-bound import state and safe result copy; player profiles provide confirmation and phase-specific boost feedback.
- Project validation commands: `./scripts/dev test`, `./scripts/dev check`, `./scripts/dev bridge-test`, and `./scripts/dev smoke`; `./scripts/dev format` applies formatting before checkpoint. `./scripts/dev mutate` remains unsupported.
- Primary risks: stale save or snapshot context during a long sequential operation; partial FM success before a later player fails; accidental widening of the closed write boundary; duplicated or inconsistent Search table behavior; native dropped-path handling that browser tests cannot prove; and route or cache state that survives the wrong save or snapshot.
- Advisory index: Repowise is unavailable in this checkout (`repowise: command not found`), so planning uses direct repository, test, configuration, and Git evidence.

## Feature architecture

The stable `/planner` route becomes the Squad composition root. It owns validated `view=squad|planner|tactic` state plus Squad Overview sort state, keeps Planner and Tactic local drafts mounted when hidden, and composes feature-owned panels without creating cross-feature imports. The Dashboard route composes the existing Planner Club Setup panel and gives it a stable link target.

Rust adds a bounded, paginated Squad Overview read model under the Planner club-family boundary. The query selects the active save's effective current snapshot, requires a configured club family, matches `players.current_club` against the distinct source-club union, applies only the fixed sortable columns, and returns Search-compatible scalar values without importing Search feature ownership.

The CSV import feature extracts its current context-bound import lifecycle so the Dashboard auto-detect panel and the new Squad modals can share it. Browse continues through the installed dialog plugin. Drag-and-drop uses the installed Tauri window API while the relevant modal is open. Rust accepts an optional expected format, validates the detected file format before any persistence, and retains auto-detection when the Dashboard does not provide an expected format.

Rust `features/player` owns squad-wide boost orchestration. A bulk command accepts only the closed operation, captures the active save, current snapshot token, and distinct configured club-family player cohort, then processes one player at a time through the existing prepare, bridge request, verified readback, and SQLite reconciliation path. The player boost gate prevents concurrent bulk or profile actions. Ineligible players increment `skipped`; bridge-proven player-local rejections with no write increment `failed` and processing continues; verified reconciliations increment `updated`. A recovery-required result or changed save or snapshot context stops the loop before the next bridge request and returns a truthful partial result plus its terminal recovery state. [ADR-0018](../../decisions/0018-squad-wide-player-boosts.md) owns this durable safety decision.

After a successful or partial bulk action, the Squad route invalidates snapshot, search, player, Planner, and Academy query roots once. Result state is bound to the initiating save and snapshot so navigation or context replacement cannot display stale feedback.

## Uncertainty register

### Known

- The route and workspace shell, club-family model, fixed Search columns, CSV parser and persistence, Modal primitive, two closed boost operations, player-level reconciliation, and cross-feature cache invalidation already exist.
- The current CA rule is +5 through age 21 and +10 from age 22 without an upper age limit. This feature intentionally corrects it to +5 through age 20, +10 from age 21 through 28, and no boost from age 29 in both profile and squad flows.
- The existing bridge protocol and operation gate process one player action at a time and already verify expected live values before writing.
- Browser smoke uses a Tauri IPC stub and does not prove native file drop paths, native dialog behavior, SQLite persistence, or live FM writes.

### Assumptions

- The current fixed Search columns are the required Squad Overview columns because the developer asked for a table very similar to Search and did not request another column set.
- The stable `/planner` path can remain for bookmarks and generated routing while user-facing labels change to Squad.
- The installed Tauri v2 window API exposes one or more dropped filesystem paths without a new dependency. Implementation must verify this against the installed package before editing capabilities or configuration.
- A club-family union is the intended squad scope. Team-specific Planner source assignment affects depth pools but does not duplicate or exclude a player from the overview when the player's club appears anywhere in the configured family.

### Decisions

- Age 20 or younger receives +5 CA, age 21 through 28 receives +10, and age 29 or older is ineligible. The same rule applies to profile and squad-wide actions.
- Bulk boosts use sequential best-effort processing with updated, skipped, and failed counts.
- Proven no-write player-local rejections do not stop later players. Active-context loss, an unconfirmed or timed-out result, uncertain rollback, FM-success/SQLite-failure, and other recovery-required outcomes stop before another write.
- The existing Dashboard CSV import remains. Squad adds two explicit expected-format entry points.
- One PR owns the feature because navigation, Squad Overview, imports, and bulk actions form one coherent user surface and can be reviewed as fine-grained commits without a risky migration or protocol foundation that must merge separately.

### Unknowns

- A native Windows Tauri session must confirm that both dropped and browsed paths reach the same modal import path and that event subscription is scoped to the open modal.
- A supported live FM26.3.2 session is required to prove the final assembled sequential bulk path. If unavailable, feature close-out must record the gap and cannot substitute bridge fakes or browser IPC stubs.

### Risks

- A large squad makes the sequential action visibly long. The UI must expose progress or a truthful pending state and keep every write serialized; parallelism is out of scope.
- FM may accept a write before SQLite reconciliation fails. The summary must not call that player updated locally and must direct the user to Load Data.
- Treating every failure as continuable could authorize a new FM write after the app has lost proof that FM and SQLite agree. The orchestrator must preserve the existing recovery-required boundary.
- Reusing Search behavior through cross-feature imports would violate the frontend dependency contract. Extract only a genuinely shared presentation seam, or keep a bounded Squad-owned table.
- Leaving the Dashboard importer while adding Squad importers can create two visible entry points. This duplication is explicitly accepted until the Dashboard leaves the user-facing app.

## Walking skeleton

Commits 1 and 2 form the walking skeleton: the navigation opens Squad, the tabs read Squad / Planner / Tactic, Club Setup works from the Dashboard, and the default Squad page lists only configured club-family players with sorting and profile links. CSV and bulk actions then extend that proven surface without changing its ownership.

## Delivery plan

### PR 1 — Build the Squad workspace

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(squad): add club overview and squad actions`

**Branch:** `feature/squad-workspace`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Build-feature-loop profile:** Terra Max — the highest packet owns sequential live-FM writes with partial outcomes and current-snapshot reconciliation.

**Purpose:** Deliver the complete user-facing Squad workspace in one cohesive review boundary while retaining atomic commits for navigation, reads, imports, the corrected CA rule, and each bulk action.

**Depends on:** Current `main` at planning base `508772d`; completed Planner, Search, CSV enrichment, snapshot history, player profile, and player-development boost foundations.

**Merge to trunk when:** Every commit is completed or removed with a recorded reason, the full feature validation passes, the Sol xhigh feature review clears, documentation is reconciled, the PR is published from the repository template, and GitHub's strict `check` succeeds.

#### Commit 1 — Reorganize the Squad workspace

**Status:** Completed

**Provisional commit:** `feat(squad): reorganize squad navigation`

**Work:** Rename the primary navigation surface to Squad, change the URL-backed pages to Squad / Planner / Tactic, move the current depth matrix to Planner, move Club Setup to the Dashboard, and make Squad the default with the required no-snapshot and no-club recovery states.

**Out of scope:**

- Squad player rows, sorting, CSV modals, CA rule changes, or bulk boosts.
- Planner depth, tactic, club-family persistence, Dashboard CSV import, or route-path changes.

**Implementation packet:**

- Owners and files: `src/app/components/app-nav-rail.tsx`; `src/app/routes/planner.tsx`; `src/app/routes/index.tsx`; `src/features/planner/components/planner-workspace-tabs.tsx`; existing Planner Club Setup composition; route and app-shell tests; affected Playwright smoke and IPC fixtures.
- Existing patterns to verify: URL-backed Planner workspace validation and focus behavior; hidden mounted panels; Dashboard Suspense composition; route-owned cross-feature invalidation; the Dashboard snapshot-management link and focus patterns.
- Constraints and invariants: keep `/planner`; preserve Planner and Tactic drafts while hidden; use exact labels Squad, Planner, and Tactic; keep Dashboard CSV import; direct no-club users to a stable Dashboard Club Setup target; do not change persistence or queries.
- Dependencies and ordering: none. This establishes the composition shell that later commits extend.

**Implementation profile:** Luna Max — the route and component boundaries already exist, and this packet is a bounded UI composition change with strong route analogues.

**Review profile:** Sol Medium — review must verify navigation, URL normalization, focus, default and empty states, and that the move preserves existing Planner and Dashboard behavior.

**Validation:**

- `./scripts/dev test src/app/routes/planner.test.tsx src/app/routes/index.test.tsx src/app/app-shell-routing.test.tsx`
- `./scripts/dev check`
- `./scripts/dev smoke`

Expected evidence: the targeted route tests prove the new labels, URLs, defaults, hidden panels, Dashboard Club Setup, and no-club link; the gate passes; smoke proves browser navigation and existing Planner journeys under the new labels.

**Stop conditions:** Replan if keeping `/planner` prevents the requested page semantics, if Club Setup cannot move without changing save-scoped ownership, or if the Dashboard route cannot host the existing panel without cross-feature import violations.

**Review mandate:**

- Verify the primary rail exposes Squad and no user-facing Planner rail label remains.
- Verify `view=squad|planner|tactic`, keyboard tab order, Home/End behavior, and URL replacement remain accessible.
- Verify Squad defaults even without club configuration and links to Dashboard Club Setup.
- Verify Planner depth and Tactic components retain local state while hidden.
- Verify Dashboard CSV import, snapshot management, save switching, and invalidation remain intact.
- Verify no route path, schema, command, or Planner domain behavior changed.

#### Commit 2 — List configured squad players

**Status:** Completed

**Provisional commit:** `feat(squad): list configured club players`

**Work:** Add the bounded club-family player query and render the filter-free Squad Overview table with fixed Search columns, sortable headers, current-snapshot counts, keyboard behavior, and player-profile name links.

**Out of scope:**

- CSV imports, boost actions, Search filters, custom columns, row selection, historical snapshots, or Planner depth changes.

**Implementation packet:**

- Owners and files: a Squad frontend feature under `src/features/squad/`; Squad route composition; a Planner-owned Rust squad query and DTO near `src-tauri/src/features/planner/`; Tauri command registration; query/IPC mocks; Rust query tests; route tests; affected smoke fixtures and journeys.
- Existing patterns to verify: Search fixed columns, sort defaults, pagination, virtualization threshold, formatters, header `aria-sort`, row keyboard traversal, and player-profile navigation; Planner club-family union queries; Academy club-family eligibility; current-snapshot selector and immutable context rules.
- Constraints and invariants: exact source-club union; distinct player UID; no filters; default CA descending; Rust-owned sort and pagination; one bounded page per IPC call; Name is a real link; no direct import from Search feature internals; nulls remain `—`.
- Dependencies and ordering: Commit 1 provides the Squad page and no-club state.

**Implementation profile:** Terra xhigh — the outcome is settled, but the packet requires a new cross-layer paginated query and careful project-fit judgment around sharing versus duplicating Search table presentation.

**Review profile:** Sol High — the read contract crosses Router, Query, IPC, club-family ownership, current-snapshot selection, SQL sorting, pagination, accessibility, and profile navigation.

**Validation:**

- `./scripts/dev test src/app/routes/planner.test.tsx src/features/squad`
- `./scripts/dev check`
- `./scripts/dev smoke`

Expected evidence: Rust tests prove exact family membership, deduplication, save/snapshot isolation, every fixed sort field and direction, paging, absent configuration, and empty current results; frontend tests prove columns, sorting, counts, keyboard access, and profile links; smoke proves the configured Squad journey.

**Stop conditions:** Replan if the table requires an unbounded IPC payload, if the current snapshot cannot be selected through the existing helper, if sharing presentation requires a broad Search refactor, or if membership semantics conflict with Planner and Academy's exact-club contract.

**Review mandate:**

- Verify the query cannot return players outside the active save's effective current snapshot.
- Verify the source-club union includes primary and attached clubs once per UID and ignores `team_level`.
- Verify sort inputs are allowlisted and parameterized, pagination is deterministic, and null ordering matches Search.
- Verify every requested fixed column renders with shared formatting and no filters appear.
- Verify Name links and keyboard row behavior reach the correct profile without nested activation defects.
- Verify no cross-feature frontend import or duplicated authoritative membership rule appears.

#### Commit 3 — Import squad CSV enrichment

**Status:** Completed

**Provisional commit:** `feat(import): add squad CSV import modals`

**Work:** Add dedicated Moneyball and Youth Academy import buttons to Squad Overview, share the existing context-bound import lifecycle, support one dropped or browsed CSV path per modal, enforce the selected format before persistence, and retain the Dashboard auto-detect importer.

**Out of scope:**

- Removing or redesigning Dashboard import, adding formats, multi-file import, import history, CSV-only players, or changing enrichment ownership.

**Implementation packet:**

- Owners and files: `src/features/csv-import/` state, API, and modal components; Squad Overview composition; `src-tauri/src/features/csv_import/commands.rs` and service validation; current Dashboard importer; Tauri event integration through the installed API; frontend, Rust, and smoke tests.
- Existing patterns to verify: `CsvImportPanel`, `importStateForContext`, native dialog selection, `Modal`, context-generation guards, safe error copy, Youth Academy invalidation, bounded Rust parsing, and expected-format detection before writes.
- Constraints and invariants: one `.csv` path; both drag/drop and browse; no path display or persistence; exact selected format must be checked before any row changes; Dashboard sends no expected format and keeps auto-detection; dropped event subscription exists only while the modal is open; save/snapshot changes clear results and prevent stale writes.
- Dependencies and ordering: Commit 2 supplies the Squad action surface and snapshot/club states.

**Implementation profile:** Terra xhigh — the UI is defined, but native drop events, modal lifecycle, stale-context handling, and trust-boundary format enforcement require material local judgment.

**Review profile:** Sol High — review must cover native-path handling, capability scope, input validation, atomic persistence, context replacement, accessibility, and both Dashboard and Squad entry points.

**Validation:**

- `./scripts/dev test src/features/csv-import src/app/routes/planner.test.tsx src/app/routes/index.test.tsx`
- `./scripts/dev check`
- `./scripts/dev smoke`

Expected evidence: frontend tests prove modal focus, drop/browse equivalence, cancellation, wrong-format errors, pending and result states, context clearing, and Dashboard retention; Rust tests prove expected-format mismatch writes nothing; smoke proves both Squad buttons and the unchanged Dashboard path with IPC stubs.

**Stop conditions:** Stop for a technical decision if the installed Tauri API cannot supply dropped filesystem paths without a new plugin or broader capability. Replan if format enforcement can occur only after persistence, if dropped paths must be exposed to the DOM or logs, or if Dashboard auto-detection cannot remain backward-compatible.

**Review mandate:**

- Verify each Squad button names and enforces its expected format before database writes.
- Verify dropped and browsed files enter one guarded import path and only one file is accepted.
- Verify event listeners are scoped and cleaned up, focus returns correctly, and keyboard-only browse remains complete.
- Verify paths and raw errors never appear in UI, logs, results, or persistence.
- Verify stale save/snapshot context leaves prior enrichment unchanged and clears stale feedback.
- Verify Dashboard auto-detect import and Youth Academy invalidation still work.
- Verify browser smoke limitations are reported rather than presented as native proof.

#### Commit 4 — Correct CA boost age eligibility

**Status:** Completed

**Provisional commit:** `fix(player): correct CA boost age eligibility`

**Work:** Change the authoritative and previewed CA eligibility so age 20 or younger receives +5, age 21 through 28 receives +10, and age 29 or older cannot receive a CA boost. Preserve PA/200 caps and all other guarded profile behavior.

**Out of scope:**

- Squad-wide orchestration, bridge protocol changes, custom increments, or non-age eligibility changes.

**Implementation packet:**

- Owners and files: `src-tauri/src/features/player/service.rs`; player service tests; `src/features/player-profile/components/player-development-boosts-panel.tsx`; player-profile tests and affected smoke expectations.
- Existing patterns to verify: current profile preview, Rust eligibility derivation, bridge request construction, CA caps, repeat boosts, and verified reconciliation.
- Constraints and invariants: one rule in Rust; React only previews the same eligibility; WebView still sends UID only; age 20 maps to +5, age 21 maps to +10, age 28 maps to +10, and age 29 is ineligible before any bridge request; caps and error states remain unchanged.
- Dependencies and ordering: none functionally, but it precedes the squad-wide CA action so that action starts from the corrected shared rule.

**Implementation profile:** Luna Max — this is a small, settled correction at two existing seams with direct boundary tests.

**Review profile:** Sol High — the diff is small, but a missed frontend/backend mismatch would authorize a different live-FM write than the confirmation presents.

**Validation:**

- `./scripts/dev test src/features/player-profile src/app/routes/players.\$uid.test.tsx`
- `./scripts/dev check`
- `./scripts/dev smoke`

Expected evidence: Rust and frontend boundary tests cover ages 20, 21, 28, and 29, preserve caps and disabled states, prove age 29 cannot submit a bridge request, and prove each eligible confirmation matches the UID-only command result.

**Stop conditions:** Replan if another layer independently derives the old age rule, if changing the rule requires a bridge protocol change, or if profile and Rust behavior cannot be made identical without moving authority into React.

**Review mandate:**

- Verify age 20 produces +5, ages 21 and 28 produce +10, and age 29 is disabled in Rust and profile copy.
- Verify age 29 cannot trigger an individual bridge request.
- Verify CA remains capped by PA and 200 and at-limit players remain ineligible.
- Verify React cannot supply the increment or target.
- Verify existing error, confirmation, repeat-boost, and reconciliation behavior remains intact.
- Verify tests would fail for the old `age <= 21` boundary and its missing upper age limit.

#### Commit 5 — Boost squad current ability

**Status:** Active

**Provisional commit:** `feat(squad): boost squad current ability`

**Work:** Add the confirmed **Boost all CA** action, derive and freeze the current club-family cohort in Rust, process eligible players sequentially through the existing closed boost path, and return truthful updated, skipped, failed, and terminal recovery outcomes.

**Out of scope:**

- Wonderkid bulk behavior, parallel writes, all-or-nothing rollback, arbitrary player selection, new bridge operations, or progress cancellation.

**Implementation packet:**

- Owners and files: Rust `features/player` bulk orchestration and command DTOs; Planner club-family service reuse; player command registration; Squad frontend API, types, confirmation/result UI, and context guards; cross-feature query invalidation; Rust, route, and smoke tests.
- Existing patterns to verify: `PLAYER_BOOST_GATE`; UID-only prepare/request/reconcile flow; immutable save/snapshot context checks; `planner::service::get_club_family`; profile confirmation and error copy; Load Data and boost invalidation roots; context-bound mutation feedback.
- Constraints and invariants: the WebView sends no UID list or increment; distinct club-family players only; corrected age rule; one bridge request at a time; age 29 or older and other ineligible states are skipped before the bridge; only a bridge-proven player-local no-write rejection continues; recovery-required uncertainty and context loss stop before the next request; each success commits before continuing; no claim of batch atomicity.
- Dependencies and ordering: Commits 1 and 2 supply the surface and cohort read contract; Commit 4 supplies the corrected shared age rule; ADR-0018 governs the safety boundary.

**Implementation profile:** Terra Max — this settled outcome combines live-process writes, long-running sequential orchestration, immutable context, partial failure, per-player SQLite reconciliation, and cross-feature cache effects.

**Review profile:** Sol xhigh — a defect could write the wrong FM player, continue under stale ownership, misreport partial success, or leave SQLite and FM observably inconsistent.

**Validation:**

- `./scripts/dev test src/features/squad src/app/routes/planner.test.tsx`
- `./scripts/dev check`
- `./scripts/dev bridge-test`
- `./scripts/dev smoke`

Expected evidence: Rust tests cover distinct cohort capture, ages 20/21/28/29, caps, age-ineligible bridge exclusion, skip/fail/continue accounting, profile-action mutual exclusion, per-player commit order, and the full continuation matrix. A proven no-write player-local rejection continues, while active-context replacement, unconfirmed or timed-out results, uncertain rollback, recovery-required bridge failures, and FM-success/SQLite-failure stop before player two reaches the bridge. Frontend and smoke tests cover confirmation, pending lockout, summary copy, recovery, and invalidation.

**Stop conditions:** Replan if the existing one-player bridge path cannot be safely reused, if a required write would bypass the boost gate, if context cannot be revalidated before each request, if the command needs a WebView-supplied UID list, or if truthful partial outcomes cannot distinguish updated, skipped, failed, and recovery-stopped states.

**Review mandate:**

- Verify the authoritative cohort comes from the active current snapshot and distinct configured source clubs.
- Verify no arbitrary batch payload, UID list, increment, target, address, or field crosses the WebView or bridge boundary.
- Verify the gate serializes profile and squad actions and no Db mutex is held while waiting for the bridge.
- Verify ineligible players never reach the bridge and proven no-write player-local rejections do not stop later players.
- Verify active-context loss and every recovery-required bridge, rollback, verification, or reconciliation outcome stop before another write and preserve truthful counts from earlier players.
- Verify every verified success reconciles before the next player and failures do not masquerade as local success.
- Verify the final invalidation refreshes snapshot, Search, profiles, Planner/Squad, and Academy once without stale feedback.

#### Commit 6 — Apply squad Wonderkid Mentality

**Status:** Pending

**Provisional commit:** `feat(squad): apply Wonderkid Mentality to squad`

**Work:** Add the confirmed **Make all Wonderkids** action on the established bulk orchestrator, applying only the existing eligible mentality fields and reporting sequential updated, skipped, failed, and terminal recovery outcomes.

**Out of scope:**

- New mentality fields, deterministic or user-selected targets, rerolling values above 10, parallelism, undo, or a general editor.

**Implementation packet:**

- Owners and files: established Rust player bulk orchestrator and DTOs; existing Wonderkid prepare/reconcile path; Squad frontend action, confirmation, and result copy; affected role-score invalidation and tests; smoke journey.
- Existing patterns to verify: player-profile Wonderkid preview and confirmation; bridge inclusive 11–20 randomness; null and above-10 preservation; Determination role-score recomputation; CA bulk accounting and context-stop behavior.
- Constraints and invariants: known Ambition, Professionalism, or Determination at or below 10 makes a player eligible; only eligible fields change; unknown and above-10 fields remain unchanged; one player at a time; result counts use the same meanings as CA; the WebView supplies neither UIDs nor values.
- Dependencies and ordering: Commit 5 establishes and reviews the shared sequential bulk orchestration and feedback contract.

**Implementation profile:** Terra xhigh — the bulk framework is established, but the packet still crosses random verified bridge results, JSON reconciliation, Determination score rewrites, and partial failure states.

**Review profile:** Sol xhigh — review must verify that the established safety boundary remains closed and that partial mentality writes and score reconciliation stay truthful across the full squad.

**Validation:**

- `./scripts/dev test src/features/squad src/app/routes/planner.test.tsx src/features/player-profile`
- `./scripts/dev check`
- `./scripts/dev bridge-test`
- `./scripts/dev smoke`

Expected evidence: Rust and bridge tests prove eligibility, untouched null/high fields, inclusive targets, rollback/readback behavior, Determination score recomputation, skip/fail/continue accounting, and reuse of the CA bulk continuation and recovery-stop matrix. Frontend and smoke tests prove confirmation, action lockout, summary feedback, and recovery.

**Stop conditions:** Replan if bulk reuse changes the closed Wonderkid payload, if a player can be marked updated without a verified eligible field change, if Determination results cannot remain transactionally aligned with role scores, or if any implementation exposes or accepts target values from React.

**Review mandate:**

- Verify only known values at or below 10 change and every target is 11 through 20 inclusive.
- Verify unknown and already-high values remain untouched for every player.
- Verify no WebView-controlled UID or target values enter the command or bridge request.
- Verify only proven no-write player-local rejections continue; recovery-required and context errors stop; counts remain truthful after partial success.
- Verify Determination changes recompute only the affected player's current role scores in the same SQLite transaction.
- Verify CA and Wonderkid actions cannot overlap each other or a profile boost.
- Verify confirmation and result copy do not imply that skipped or failed players changed.

## Active work

**PR:** PR 1 — Build the Squad workspace

**Commit:** Commit 5 — Boost squad current ability

### RED proof

Add focused Rust and Squad route tests first. They must freeze a distinct configured club-family cohort at the active current snapshot; cover ages 20, 21, 28, and 29, caps, and per-player bridge exclusion; prove sequential success, skip, and continue accounting; and distinguish a proven no-write player-local rejection from every outcome that must stop before player two.

### Expected outcome

A confirmed Squad CA action sends no player selection or increment from the WebView. Rust derives the cohort and corrected increment, runs each eligible player through the shared locked boost path in order, reconciles each verified result before moving on, reports accurate partial results, and invalidates shared reads once when complete or stopped.

### Explicit exclusions

Wonderkid bulk behavior, parallel writes, all-or-nothing rollback, arbitrary player selection, new bridge operations, progress cancellation, Search filters, custom columns, row selection, historical snapshots, Planner-depth changes, push, and publication do not belong in this active commit.

## Discoveries and replanning

- 2026-08-12: The developer corrected the lower CA boundary from the delivered `age <= 21` rule to +5 at age 20 or younger and +10 starting at age 21. Commit 4 owns the profile and Rust correction before bulk CA reuses it.
- 2026-08-12: The developer added an upper CA eligibility boundary: players aged 29 or older receive no CA boost on either the profile or Squad action. Commit 4 now covers both age boundaries, and Commit 5 must skip those squad players before the bridge.
- 2026-08-12: The developer accepted sequential best-effort bulk execution. Proven no-write player-local rejections continue; active-context loss and recovery-required uncertainty stop before another write.
- 2026-08-12: Commit review found that the initial plan treated every player-specific bridge or reconciliation failure as continuable. The plan now preserves ADR-0017's fail-closed recovery boundary and requires a fatal-versus-continuable test matrix for both squad actions.
- 2026-08-12: The Dashboard is expected to leave the user-facing app later, but this feature keeps its current CSV import while adding explicit Squad import entry points.
- 2026-08-12: Commit 2 keeps overview reads under the existing Planner invalidation root with a nested `['planner', 'squad']` key. Its 50-row pages stay below the table virtualization threshold while Rust retains the 200-row hard limit.
- 2026-08-12: Commit 3 uses the installed Tauri WebView drop API without a new capability. Each open modal captures its save/snapshot generation, so a delayed drop from a replaced context is ignored before IPC.
- 2026-08-12: Commit 3 serializes all modal intake while a CSV persistence request is pending. Later single-file drops and multi-file validation errors are ignored until the active request settles, so they cannot overwrite truthful pending state or reopen another import path.
- 2026-08-12: Commit 4 aligned the shared frontend IPC double with the corrected age-20 boundary and added submitted age-21 result parity coverage after review found that preview-only coverage could mask a conflicting test result.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Reorganize the Squad workspace | Pending record | Squad navigation, URL-backed workspace composition, Dashboard Club Setup, and recovery links | Sol Medium — Accept | None |
| PR 1 | Commit 2 — List configured squad players | Pending record | Bounded exact club-family overview query, sortable fixed-column table, paging, and profile links | Sol High — Accept | None |
| PR 1 | Commit 3 — Import squad CSV enrichment | Pending record | Format-bound Squad CSV modals, guarded browse/drop intake, expected-format persistence enforcement, and retained Dashboard auto-detection | Sol High — Accept | None |
| PR 1 | Commit 4 — Correct CA boost age eligibility | Pending record | Corrected CA age boundaries, pre-bridge age exclusion, and profile result parity | Sol High — Accept | None |

## Final validation

**Feature review profile:** Sol xhigh — final review must cross-check route ownership, club-family scoping, native file intake, expected-format persistence, the corrected CA age eligibility, serialized live-FM writes, partial outcomes, context stops, per-player reconciliation, and cross-feature invalidation across all commits.

Required automated evidence before feature review:

- `./scripts/dev format`
- `./scripts/dev test`
- `./scripts/dev check`
- `./scripts/dev bridge-test`
- `./scripts/dev smoke`
- `git diff --check 508772d...HEAD`
- Rust proof for current-snapshot club-family union, paging and every sort field, expected CSV format mismatch with no writes, corrected ages 20/21/28/29, age-ineligible bridge exclusion, distinct bulk cohorts, gate serialization, skip/fail/continue counts, the fatal-versus-continuable failure matrix, context-change stop, and per-player reconciliation including Determination role scores.
- Frontend proof for Squad navigation and URL state, Dashboard Club Setup, no-club recovery, fixed columns and sorting, profile links, both modal file paths, context-bound import and boost feedback, confirmation, pending lockout, and cache invalidation.
- Bridge proof for the unchanged two action-specific operations, expected-value revalidation, inclusive Wonderkid targets, verified readback, and rollback reporting.

Manual native evidence target:

- In a Windows Tauri session, open each Squad import modal, import the correct format once by drop and once by browse, reject the opposite format without persistence, and switch save or snapshot while a modal result is visible.
- In a supported FM26.3.2 session after a fresh Load Data, configure a bounded club family, run both squad-wide actions, confirm age 20 receives +5, ages 21 through 28 receive +10, and age 29 or older receives no CA boost. Confirm the individual CA button is unavailable for age 29 or older, confirm only mentality values at or below 10 change, and reload data to compare FM with the reconciled Squad and profile views.
- If either native environment is unavailable, record the exact gap during `$workflow-finish-feature`. Browser IPC stubs, Rust tests, and C# fakes must not be presented as proof of the missing native path.

`./scripts/dev mutate` remains unsupported and must not be reported as passed.

## Documentation impact

During implementation and feature reconciliation:

- Update `.wiki/ARCHITECTURE.md` when the Squad query, Dashboard Club Setup ownership, CSV expected-format/drop path, corrected age rule, and bulk orchestration are implemented.
- Update `.wiki/DESIGN.md` when the Squad / Planner / Tactic layout, overview table, import modals, Dashboard Club Setup, and bulk confirmations are implemented.
- Update `.wiki/CONCEPT.md` only if the implemented squad-wide use changes the stated product boundary beyond the same two approved actions.
- Keep [ADR-0018](../../decisions/0018-squad-wide-player-boosts.md) aligned with the accepted sequential safety boundary. Do not rewrite completed feature history; add narrow downstream pointers only if reconciliation needs them.
- Move this ledger to `features/completed/` and move the feature from Active to Completed in `.wiki/TODO.md` only after final validation, feature review, and documentation reconciliation.
