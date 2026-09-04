# Integrated Shortlists

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 9ac27f359b9986fbfcf39080c821196d8f88d1731eff3663a2c9da40dbc0f105

## Intent

Fold the two separate shortlist views into the core search pages. Player Search (`/search`) loses its standalone Player Shortlist tab and gains an `Upload Shortlist` button plus a shortlist on/off toggle on the General page. Staff Search (`/staff`) absorbs the My Club Staff Shortlist workspace and gains shortlist upload, `Configure Club Staff`, and `Optimize assignments` controls in its filter header plus a shortlist toggle beside the result count. Each area keeps exactly one save-owned persistent replacement list. Moneyball CSV imports stop affecting player shortlist membership.

## User-visible behavior

- `/search` shows two views: General and Moneyball. The Player Shortlist tab is gone.
- The General page shows an `Upload Shortlist` button immediately to the right of `Edit Filters`. It opens a CSV upload modal that follows the existing native-picker/modal patterns.
- Player shortlist CSV uses the established detection path supporting comma or semicolon delimiters. It requires the exact `Player UID` header with BOM support; header order is irrelevant. All other columns are ignored and no name column is required. The existing bounded-file rules apply: regular files only, 8 MiB limit, 10,000-row limit. Valid repeated UIDs are deduplicated. Blank, nonnumeric, zero/out-of-range, duplicate, and current-snapshot-unmatched rows are skipped. Malformed CSV or a missing header is rejected. The import atomically replaces the prior list only when at least one UID matches a current-snapshot player; a zero-match upload preserves the old list and returns an error. The summary reports total data rows, stored unique matched UIDs, and skipped rows (invalid, duplicate, and unmatched each count once).
- One player shortlist persists per app save across restart and snapshot replacement. Current search rows join by UID.
- A successful player upload turns shortlist filtering on. The on/off state lives in Search URL state, so it is shareable. Off shows all current Search players. On restricts results to current players whose UIDs sit in the saved player shortlist.
- General filters, sorting, columns, tactic columns, Club DNA, and paging compose with the shortlist restriction. Moneyball CSV imports no longer change player shortlist membership in any way.
- The player shortlist toggle sits next to the Search results count/sort line (`<count> players · sorted by ...`). The toggle is keyboard accessible and always shows the true on/off state. Upload stays beside Edit Filters.
- `/staff` core Staff Search owns the staff shortlist. The separate Staff Shortlist workspace/tab in My Club is gone.
- Staff shortlist keeps the existing semicolon CSV contract: `Unique ID`, `Preferred Job`, `Club Job`, and `Coaching Qualifications`. It keeps exact UID matching, replacement safety, metadata, Preferred Job behavior, Only unemployed behavior, staff assignment targets, Configure/Optimize behavior, immutable context guards, and optimizer allocation semantics.
- Staff Search puts shortlist upload, `Configure Club Staff` (renamed from `Configure slots`), and the existing `Optimize assignments` control in the filter header. Upload, Configure Club Staff, and Optimize assignments are always visible in the integrated Staff filter header. Configure remains usable because save-owned target configuration does not require shortlist rows. Optimize retains its existing typed no-shortlist result/setup feedback; no new disable rule is invented. Preferred Job and Only unemployed render only when shortlist filtering is ON.
- A successful staff upload turns shortlist filtering on. The staff shortlist toggle sits beside the Staff result count/sort line. Core Staff filter-editor rules also apply to shortlisted staff when filtering is on.
- Legacy shortlist URLs replace-normalize without inspecting persistence: `/search?view=shortlist` becomes General with shortlist filtering ON; `/staff?view=shortlist` and `/my-club?view=staff-shortlist` become `/staff` with shortlist filtering ON. With no saved list, the on state yields the existing setup/no-shortlist feedback and Upload remains available.
- When shortlist filtering is OFF, `/staff` uses the existing `staff-search` layout and normal Staff metrics. When ON, the same page uses the existing persisted `staff-shortlist` layout and `staffShortlistPresentation(preferredJob)`: All jobs retains its configurable CSV metadata layout; mapped jobs retain fixed identity/metadata plus mapped score and current sort behavior; Coach retains six outfield coaching scores without automatic score sorting; unrecognized jobs retain the current CA behavior. Core Staff filter-editor predicates compose in both modes. The `staff-shortlist` persisted layout is kept.
- Controls that require a shortlist stay visible in the integrated filter header. Preferred Job and Only unemployed render only when shortlist filtering is on. In the Staff header specifically, Upload, Configure Club Staff, and Optimize assignments are always visible; Configure stays usable without shortlist rows and Optimize keeps its existing typed no-shortlist result/setup feedback.
- Staff profiles and row navigation keep working from the integrated page.

## Invariants

- One player shortlist per app save; one staff shortlist per app save. Both are replacement lists, never merges.
- Zero-match uploads never clear the prior list. Both imports return an error in that case.
- Current-snapshot rows are authoritative. Saved shortlist rows join by UID against the effective current snapshot at read time.
- Shortlist filtering composes with existing filters, sorting, and paging. It never bypasses filter-editor validation.
- Shortlist toggle state is URL state in both Search and Staff Search. Saved list membership is not URL state.
- Staff CSV parsing stays semicolon-delimited with the exact four-column contract. Player CSV parsing follows the exact contract above: comma or semicolon via established detection, exact `Player UID` header with BOM support, order-irrelevant headers, all other columns ignored, 8 MiB and 10,000-row limits, dedupe of valid repeated UIDs, per-row skip classes, malformed-or-missing-header rejection, atomic replace on at least one current match, zero-match preservation, and the three-field summary.
- Staff assignment targets, Configure/Optimize behavior, immutable context guards, and optimizer allocation semantics do not change.
- The feature stays offline and desktop-only: thin React frontend, thick Rust backend, SQLite owned by Rust, bounded IPC.
- Moneyball imports never write player shortlist membership.

## Non-goals

- Manual add, remove, reorder, annotate, or rank shortlist entries.
- Historical shortlists, trends, comparisons, or analytics beyond the current snapshot.
- New scoring models, role catalogs, or attribute formulas.
- Changes to bridge scanning, snapshot selection, Youth Tracker, Squad, Planner, Academy, Settings, or Dashboard.
- Making the shortlist a Settings default view.
- Changing `.wiki/CONCEPT.md`, `.wiki/ARCHITECTURE.md`, or `.wiki/DESIGN.md` during planning. Those documents describe current state and get reconciled only after implementation makes the new behavior true.
- Release preparation. An explicit release skill owns that separately.

## Current-state map

- Relevant components:
  - `src/app/routes/search.tsx::SearchPageContent` — General/Moneyball/Shortlist tabs, `shortlist` table-layout branch, Shortlist empty-state copy.
  - `src/features/search/components/search-results-panel.tsx::SearchResultsPanel` — results count/sort line, filter bar placement beside `Edit Filters`.
  - `src/features/search/utils/search-url-search.ts` — filter/sort/combine URL parsing; no shortlist toggle key exists yet.
  - `src/features/search/types/search-view.ts` — `SearchView` includes `shortlist`.
  - `src/stores/use-player-table-store.ts` — independent `shortlist` layout plus `staff-shortlist` layout.
  - `src/app/routes/staff.tsx::StaffPageContent` — core search only; carries shortlist sort-state fields for a separate view.
  - `src/app/routes/my-club.tsx::MyClubStaffShortlistWorkspace` — Staff Shortlist workspace UI and assignment optimizer host.
  - `src/features/staff/components/staff-search-results-panel.tsx::StaffSearchResultsPanel` — core Staff table/layout.
  - `src/features/staff/components/staff-shortlist-import-modal.tsx` — staff CSV upload modal and summary contract.
  - `src/features/staff/components/staff-assignment-optimizer.tsx` — `Optimize assignments` control; `staff-assignment-target-modal.tsx` owns the `Configure slots` label.
  - `src/features/staff/utils/staff-url-search.ts` — `StaffView` includes `shortlist`; no shortlist-filter toggle key exists yet.
  - `src/features/staff/api/staff-keys.ts`, `fetch-staff.ts`, `staff-query-options.ts` — `shortlist` query scope and `list_staff_shortlist` fetch path.
  - `src/features/staff/utils/staff-shortlist-presentation.ts` — Preferred Job presentation mapping.
  - `src/testing/staff-ipc-mock.ts`, `src/testing/setup.ts` — shortlist mocks.
- Data model:
  - Player shortlist is Moneyball-backed: `SearchView::Shortlist` builds `FROM players INNER JOIN player_moneyball_stats` in `src-tauri/src/features/search/query.rs` (line ~509). No dedicated player shortlist table exists.
  - Staff shortlist persists in `staff_shortlist_entries(save_id, staff_uid, preferred_job, club_job, coaching_qualifications)` from migration v27 (`STAFF_SHORTLIST_SCHEMA_SQL` in `src-tauri/src/db/migrations.rs`). `list_staff_with_shortlist` in `src-tauri/src/features/staff/query.rs` already composes general Staff filters with shortlist predicates internally.
- Persistence and migrations:
  - Migration registry lives in `src-tauri/src/db/migrations.rs`. Latest version is v40 (`expand_compact_role_metrics_for_generic_oop`). A new v41 migration owns the player shortlist table. Staff needs no new table.
  - Player shortlist table must mirror the staff save-owned pattern: keyed by save, joined to the current snapshot by UID at read time, surviving snapshot replacement and restart.
- Existing behavioral assumptions:
  - Player Shortlist membership equals the current Moneyball cohort; upload ownership sits in the Moneyball tab; Shortlist has no upload control (see `.wiki/features/completed/player-shortlist.md`).
  - Staff Shortlist replaces per save on successful import; zero-match, invalid, stale, or failed imports keep the prior list; Preferred Job exact match and trimmed blank/`-` Club Job unemployment rule apply (see `.wiki/features/completed/staff-shortlist.md`).
  - Completed records for the optimizer, FM26 layout, My Club workspace, and staff workspace stay valid except where this ledger explicitly removes a separate view.
- Architectural seams:
  - Rust: `src-tauri/src/features/csv_import/commands.rs` owns the player CSV import command alongside a player-shortlist module and `mod.rs`; Tauri registration lives in `src-tauri/src/lib.rs`. `src-tauri/src/features/search/commands.rs` owns only shortlist-filter request parsing. Staff: `src-tauri/src/features/staff/{commands.rs,query.rs}`, `src-tauri/src/features/csv_import/{commands.rs,staff_shortlist.rs}` plus `parser.rs`, `service.rs`, `model.rs`.
  - Frontend: routes above, results panels, URL parsers, query keys, table store, import modals.
- Project validation commands:
  - `./scripts/dev test [target...]` for focused and affected tests.
  - `./scripts/dev check` as the full commit gate (Biome, TypeScript, secretlint, Rust format, Clippy, Rust tests).
  - `./scripts/dev check-fast`, `./scripts/dev check-app`, `./scripts/dev check-rust` for scoped gates.
  - `CI=1 ./scripts/dev smoke` for the Playwright product suite after Chromium install.
- Primary risks:
  - Player query rewrite drops Moneyball coupling but must keep General filters, tactic columns, Club DNA, and paging intact.
  - Staff core search must gain the shortlist predicate without breaking the existing `list_staff_shortlist` contract or optimizer semantics.
  - URL toggle keys are new in both parsers; legacy shortlist URLs replace-normalize to the integrated pages with filtering ON as stated above, without inspecting persistence.
  - Removing the My Club workspace must not strand assignment targets or optimizer state. Removal scope is exact: remove the Player `shortlist` table layout; retain the Staff `staff-shortlist` layout; remove only separate tabs/workspaces and obsolete scope wrappers.

## Feature architecture

- Rust owns persistence, CSV parsing, replacement safety, UID join, filter composition, sorting, and paging for both shortlists.
- Player adds migration v41 with a save-owned `(save_id, player_uid)` table. A new import command in `src-tauri/src/features/csv_import/commands.rs` (player-shortlist module, `mod.rs`; registered in `src-tauri/src/lib.rs`) implements the exact player CSV contract above and atomically replaces only on at least one current-snapshot match. The General search query gains an optional shortlist restriction joined by UID (`src-tauri/src/features/search/commands.rs` owns only the request parsing). The Moneyball-backed `SearchView::Shortlist` branch and its INNER JOIN to `player_moneyball_stats` is retained through the Commit 3 restriction as compatibility and retires in Commit 4 with the frontend move, not with storage.
- Staff reuses `staff_shortlist_entries` and the `list_staff_with_shortlist` composition. The existing `search_staff` Tauri command (`src-tauri/src/features/staff/commands.rs::search_staff`) and `list_staff` query (`src-tauri/src/features/staff/query.rs::list_staff`) gain shortlist on/off plus Preferred Job/unemployed parameters routed through reusable `list_staff_with_shortlist`; then `list_staff_shortlist` and its dedicated frontend fetch/query wrappers retire in Commit 6 once the integrated route consumes the flagged core path. Reusable query helpers, tests, and presentation utilities are retained. The CSV contract, replacement safety, and metadata do not change.
- React owns buttons, modals, toggles, URL toggle state, conditional Preferred Job and Only unemployed controls, table presentation, and disabled/setup states. No new state library or routing framework.
- URL state carries only the shortlist on/off flag plus existing filters and sorts. Saved membership never enters the URL.
- Contract removal ships with replacement behavior in the same commits: the Player Shortlist tab, Moneyball-backed membership, separate Staff Shortlist workspace/tab, obsolete route state, the Player `shortlist` persisted layout, empty-state copy, tests, mocks, and components retire only where no supported behavior remains. The Staff `staff-shortlist` persisted layout is retained. Staff profiles and row navigation are preserved.

## Uncertainty register

### Known

- Latest migration is v40. Player shortlist needs v41. Staff shortlist storage (v27) is settled.
- `list_staff_with_shortlist` already composes general Staff filters with shortlist predicates. The staff backend change is a flag on the core path, not a new query engine.
- Current player Shortlist is an INNER JOIN to `player_moneyball_stats`. That branch must be deleted, not kept as a fallback.
- `.wiki/CONCEPT.md` and `.wiki/ARCHITECTURE.md` describe the old separate views. They stay unchanged until implementation lands. `.wiki/DESIGN.md` likewise stays unchanged during planning; its reconciliation facts are recorded under Documentation impact for the close-out owner.
- No planned feature spec exists. There are no active features. The working tree was clean at dispatch.

### Assumptions

- Recent PRs squash-merge into `main` with Conventional Commits titles under 72 characters. The ledger records squash and the repository PR template from that evidence.
- `Configure Club Staff` is a rename of the existing `Configure slots` label in `staff-assignment-target-modal.tsx` with no behavior change.
- One PR suffices. Player and staff changes share the "remove separate shortlist views" contract removal, so splitting would create two reviews of one seam rather than two independent merges.

### Decisions

- Approved intent in the dispatch is the decision record: button and toggle placement, CSV contracts, replacement safety, URL-state toggles, composition rules, Moneyball decoupling, header control grouping, conditional Preferred Job and Only unemployed, visible-but-disabled no-shortlist controls, and removal scope. This ledger does not relitigate them.
- No ADR. The change reuses established persistence, import, and query patterns from the completed staff shortlist record. No durable structural choice with meaningful alternatives meets the ADR threshold.
- BACKLOG is unchanged. No deferred item graduates or retires under this scope.

### Unknowns

- Exact new URL toggle key names in `search-url-search.ts` and `staff-url-search.ts`. Resolve from the existing parser analogues (`parseSearchCombine`, `parseStaffView`, sort/dir keys) at implementation time. Keys must round-trip, reject invalid values to off, and apply the exact legacy normalization above (`/search?view=shortlist` to General ON; `/staff?view=shortlist` and `/my-club?view=staff-shortlist` to `/staff` ON) without inspecting persistence.
- Exact new Rust command and query signatures for the player import (owned by `src-tauri/src/features/csv_import/commands.rs` with a player-shortlist module and `mod.rs`, registered in `src-tauri/src/lib.rs`) and the core staff shortlist flag (owned by `src-tauri/src/features/staff/commands.rs::search_staff` / `src-tauri/src/features/staff/query.rs::list_staff` routed through reusable `list_staff_with_shortlist`; `src-tauri/src/features/search/commands.rs` owns only shortlist-filter request parsing). Resolve signatures from `csv_import/commands.rs`, `staff_shortlist.rs`, and `staff/query.rs` analogues at implementation time.

### Risks

- Player query rewrite could drop tactic-column, Club DNA, or paging behavior. The packet requires proving each composes with the restriction.
- Staff integration could strand assignment targets or optimizer context guards owned by the My Club workspace. The packet requires proving Configure/Optimize behave identically from the new header.
- Toggle accessibility could regress. Both packets require a keyboard-operable toggle with a truthful on/off representation.
- Legacy `view=shortlist` links resolve by the exact replace-normalization above. Removal packets must implement that normalization and delete the Player `shortlist` layout key while retaining the Staff `staff-shortlist` layout key.

## Walking skeleton

Player migration v41 plus the bounded import/replacement command lands first and may sit unused; the General query restriction follows in Commit 3 with the legacy `SearchView::Shortlist` contract retained, then Commit 4 wires an `Upload Shortlist` button and URL toggle on `/search` and retires the legacy branch. Staff integration follows the same proven shape on its existing storage.

## Delivery plan

### PR 1 — feat(search): integrate player and staff shortlists

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feat/integrated-shortlists`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub required strict status `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(search): integrate player and staff shortlists`

**Purpose:** The sole review and merge boundary. Player and staff changes share one contract removal (no separate shortlist views), so one PR keeps the seam reviewable. Later commits depend on earlier merged work inside this PR only.

**Depends on:** None. Builds on completed player-shortlist, staff-shortlist, optimizer, workspace, Moneyball, tactic-column, and Club DNA records.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(shortlists): record approved feature plan`

**Work:** Commit the independently reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/integrated-shortlists.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — active feature state.
- `.wiki/BACKLOG.md` — no change.
- `.wiki/decisions/` — no new record; the ADR threshold is not met.

**Behavior and data flow:**

- Move planning truth into one reviewed active ledger and record the exact delivery sequence before implementation.

**Ordered implementation steps:**

1. Verify the active branch and base without changing Git state.
2. Confirm the worktree contains only the reviewed planning paths.
3. Run the ledger classifier and any repository documentation check.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The ledger classifier and documentation checks prove structural consistency.

**Patterns to verify:**

- The active-ledger template, current TODO/BACKLOG ownership rules, and relevant accepted ADR format.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, plan scope, packet order, or reviewed decisions.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/integrated-shortlists.md` plus the repository documentation check when one exists.

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files.

#### Commit 2 — Add save-owned player shortlist store and import

**Status:** Completed

**Provisional commit:** `feat(search): add save-owned player shortlist store`

**Work:** One coherent, revertible backend outcome: persistent player shortlist storage plus the bounded CSV replacement import. No query restriction and no frontend wiring yet; storage and import may land unused.

**Size assessment:** Roughly 120 to 200 changed non-test implementation lines across one migration and one import path. Within the soft target.

**Out of scope:**

- General query restriction, Moneyball removal, React changes, URL toggle state, staff changes, and `.wiki/CONCEPT.md` or `.wiki/ARCHITECTURE.md` reconciliation.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — new v41 migration `create_player_shortlist_entries` mirroring `STAFF_SHORTLIST_SCHEMA_SQL` (v27): save-owned `(save_id, player_uid)` composite primary key whose save-id prefix serves the per-save lookup (no separate save index), cascade on save delete. Verify save-id and snapshot-join semantics against the staff analogue and the snapshot service before writing.
- `src-tauri/src/features/csv_import/commands.rs` plus a player-shortlist module and `mod.rs`, following `staff_shortlist.rs` — exact player CSV contract: comma or semicolon via the established `detect_delimiter` path; exact `Player UID` header with BOM support, header order irrelevant; all other columns ignored, no name required; existing `MAX_CSV_BYTES` (8 MiB, regular files) and `MAX_CSV_ROWS`/`MAX_STAFF_SHORTLIST_ROWS`-analogue 10,000-row limits, where the 10,000 limit counts every data row before validity/deduplication (10,000 data rows accepted, 10,001 rejected even when rows would be skipped); deduplicate valid repeated UIDs; skip blank, nonnumeric, zero/out-of-range, duplicate, and current-snapshot-unmatched rows; reject malformed CSV or a missing header; match against the effective current snapshot; atomically replace only on at least one match; preserve the old list and return an error on zero matches; summary reports total data rows, stored unique matched UIDs, and skipped rows (invalid, duplicate, and unmatched each count once). Reuse the bounded-file, outside-the-lock parse, and in-transaction context-revalidation shape from `staff_shortlist.rs`.
- `src-tauri/src/lib.rs` — register the new import command beside `import_staff_shortlist_csv`.
- Rust unit tests beside the changed modules — import replacement safety (zero-match preserves, dedupe, per-class skip counts, summary fields) and save ownership across snapshot replacement.

**Behavior and data flow:**

- Entry: Tauri player shortlist import command with a trusted file path.
- Parse outside the database lock, capture active save and current-snapshot context, revalidate context in the transaction, and atomically replace matching rows. Zero-match import returns an error and leaves storage untouched.

**Ordered implementation steps:**

1. Add the smallest RED proof: zero-match import preserves the prior list; one-match import replaces with the exact summary fields.
2. Write migration v41 following the v27 analogue.
3. Implement the import path following `staff_shortlist.rs` with the exact contract above.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Observable behavior: replacement safety, save ownership, and the exact summary contract.
- Prove at the `csv_import` seam: pre-seed one saved UID, run a zero-match import, and assert the old list survives; run a one-match import with duplicates and unmatched rows and assert atomic replacement plus exact summary counts.
- Row-count boundary: 10,000 data rows accepted, 10,001 rejected, even when rows would be skipped — the limit counts rows before validity/deduplication.
- Forced insert-failure proof: fail the replacement insert and assert the delete/replace transaction rolls back to the exact prior list.
- Moneyball non-interference: a Moneyball import leaves pre-seeded `player_shortlist_entries` unchanged.
- Migration proof: forward registry/version registration of v41, fresh-install and populated-database upgrade, `(save_id, player_uid)` primary key, save-delete cascade, save isolation, and snapshot-replacement survival.
- Do not inventory unchanged Moneyball, staff, search-query, or snapshot tests.

**Patterns to verify:**

- `staff_shortlist.rs` for parse-outside-lock, context capture and revalidation, and atomic replace.
- `STAFF_SHORTLIST_SCHEMA_SQL` for save-owned key, index, and cascade shape.

**Constraints and non-goals:**

- No read-time writes. No query or UI change in this commit.
- Do not change staff code, React code, search query code, or current-state documents.

**Dependencies and sequencing:**

- Requires Commit 1 merged on the feature branch. No other prerequisite.

**Validation:** Focused `cargo test` for the touched modules, then affected `csv_import` package tests, then `./scripts/dev check-rust`. Full `./scripts/dev check` before checkpoint.

**Stop conditions:** Stop for a developer decision when the save-id or snapshot-join semantics in the staff analogue do not transfer cleanly to players.

**Review mandate:** Verify forward migration registration/version and cascade shape (no separate save index — the composite PK prefix covers it); exact CSV contract (delimiter detection, header, BOM, row-count boundary before validity/deduplication, limits, skip classes, rejection); atomic replacement with zero-match preservation and insert-failure rollback; summary fields; Moneyball non-interference; no read-time writes.

#### Commit 3 — Restrict General search to the player shortlist

**Status:** Completed

**Provisional commit:** `feat(search): restrict general search to shortlist`

**Work:** One coherent, revertible backend outcome: the optional General Search shortlist restriction flag. The existing Rust `SearchView::Shortlist` parse/query contract is intentionally retained in this commit as compatibility because the current React tab still reads it until Commit 4. No frontend wiring yet.

**Size assessment:** Roughly 60 to 120 changed non-test implementation lines in the search query and request parsing. Within the soft target.

**Out of scope:**

- React changes, URL toggle state, staff changes, and `.wiki/CONCEPT.md` or `.wiki/ARCHITECTURE.md` reconciliation.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/search/query.rs` — optional shortlist restriction on the General query path: INNER JOIN to the Commit 2 save-owned table by UID against the current snapshot, composing with existing General filters, sorts, tactic columns, Club DNA, and paging. The Moneyball-backed `SearchView::Shortlist` INNER JOIN to `player_moneyball_stats` is intentionally retained in this commit; it retires in Commit 4.
- `src-tauri/src/features/search/commands.rs` — owns only shortlist-filter request parsing for the restriction flag; verify Moneyball import code paths no longer touch player shortlist membership.
- Rust unit tests beside `search/query.rs` — restriction composition (restriction plus one General filter plus paging) and the retained legacy Moneyball-backed Shortlist contract.

**Behavior and data flow:**

- General search reads carrying a shortlist flag join saved UIDs to current-snapshot players, apply General predicates, then count, sort, and page.

**Ordered implementation steps:**

1. Add the smallest RED proof: General query with the flag returns only joined UIDs.
2. Implement the query restriction.
3. Retain `SearchView::Shortlist`, its parse/query branch, and its tests unchanged until Commit 4; add restriction coverage without touching the legacy contract.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Observable behavior: composed restriction alongside the retained legacy contract.
- Prove at the `search/query.rs` seam: query General with the flag plus one filter and assert only joined current-snapshot players return with correct totals; the existing Moneyball-backed shortlist query tests stay green unchanged in this commit.
- Do not inventory unchanged Moneyball, staff, or snapshot tests.

**Patterns to verify:**

- `list_staff_with_shortlist` for composing general filters with a shortlist INNER JOIN before count, sort, and paging.

**Constraints and non-goals:**

- No new scoring path. No Moneyball statistics inside the General query.
- Moneyball imports must not write player shortlist membership after this commit.
- Do not change staff code, React code, or current-state documents.

**Dependencies and sequencing:**

- Requires Commit 2 on the feature branch for the storage table.

**Validation:** Focused `cargo test` for the search module, then affected `search` package tests, then `./scripts/dev check-rust`. Full `./scripts/dev check` before checkpoint.

**Stop conditions:** Stop for a developer decision when the General query cannot compose the restriction with tactic columns or Club DNA without a wider rewrite.

**Review mandate:** Verify UID join against the current snapshot only; composition with filters, tactic columns, Club DNA, and paging; the old `SearchView::Shortlist` contract still fully serves the current tab (trunk-safe); no Moneyball write path remains; no React or staff change.

#### Commit 4 — Integrate player shortlist upload and toggle in Search

**Status:** Completed

**Provisional commit:** `feat(search): integrate player shortlist upload toggle`

**Work:** One coherent, revertible cross-language outcome: `Upload Shortlist` button plus modal, URL-backed on/off toggle beside the count/sort line, composition with General behavior, Moneyball decoupling in the UI, and atomic removal of the separate Shortlist tab together with the now-unused Rust `SearchView::Shortlist` parse/query branch (retained through Commit 3 as compatibility). Obsolete frontend state, layout, copy, tests, and mocks retire in the same commit.

**Size assessment:** Roughly 150 to 250 changed non-test implementation lines across the route, results panel, URL parser, table store, and modal. May exceed the soft target; keep it together because the button, modal, toggle, tab removal, and layout cleanup are one observable contract (any subset alone leaves a dead tab or a toggle without upload).

**Out of scope:**

- Staff changes, Rust changes beyond what Commits 2 and 3 exposed, and current-state document reconciliation.

**Implementation packet:**

**Files and responsibilities:**

- `src/app/routes/search.tsx::SearchPageContent` — remove the `shortlist` view from the tab list, view state, and route validation; add `Upload Shortlist` immediately right of `Edit Filters` via the settled after-Edit action seam in `src/features/search/components/search-filter-bar.tsx` and `search-filter-strip.tsx` (existing tactic controls keep their current placement; `Upload Shortlist` renders immediately to the right of `Edit Filters`); successful upload turns filtering on; legacy `/search?view=shortlist` replace-normalizes to General with shortlist filtering ON without inspecting persistence (verify against `search.test.tsx` expectations).
- `src-tauri/src/features/search/commands.rs` (`parse_search_view`, sort/comparison-pool view matches) and `src-tauri/src/features/search/query.rs` — remove the now-unused `SearchView::Shortlist` parse arm and Moneyball-backed query branch in the same commit, once no frontend reader remains. This commit owns the affected Rust tests.
- `src/features/search/components/search-results-panel.tsx::SearchResultsPanel` — shortlist toggle next to the `<count> players · sorted by ...` line; keyboard accessible with a truthful on/off representation; off shows all current players, on restricts to saved UIDs; existing General filters, sorting, columns, tactic columns, Club DNA, and paging compose unchanged.
- `src/features/search/utils/search-url-search.ts` — new shortlist on/off URL key with strict parse (invalid resolves to off) and serialization; follow `parseSearchCombine` and sort/dir key analogues.
- `src/features/search/types/search-view.ts` — remove `shortlist` from `SearchView` only if no supported behavior reads it after this commit.
- `src/stores/use-player-table-store.ts` — remove the independent `shortlist` layout and its sanitization branch; General layout owns the integrated table.
- `src/features/search/components/player-shortlist-import-modal.tsx` — player shortlist upload modal following `staff-shortlist-import-modal.tsx` patterns and native-picker conventions: success summary, zero-match error, context-change guards.
- `src/features/search/api/import-player-shortlist.ts` — modal IPC invocation for the player upload (or the exact equivalent owner if the established search-api naming requires it).
- `src/features/search/types/player-shortlist-import-summary.ts` — typed shortlist summary shape (or the exact equivalent owner).
- Query keys and fetch path for the General query with the shortlist flag; Moneyball upload no longer invalidates or writes player shortlist state.
- Tests: extend `src/app/routes/search.test.tsx`, `src/features/search/utils/search-url-search.test.ts`, and `src/stores/use-player-table-store.test.ts` for toggle round-trip, upload-turns-on, off-shows-all, and composed filtering. Remove Shortlist-tab tests, empty-state copy tests (`No shortlist yet`), and shortlist mocks in `src/testing/search-ipc-mock.ts` (plus `src/testing/setup.ts` wiring) that assert the removed contract; update `e2e/tauri-ipc-stub.ts` and `e2e/smoke.spec.ts` only where they assert the retired tab/contract — retarget or remove only tests and fixtures that assert retired routes/contracts. Retire or rewrite the Moneyball-backed shortlist query tests in `search/query.rs` (`shortlist_returns_only_current_moneyball_members_*`, `shortlist_respects_snapshot_isolation_and_empty_cohort`, `shortlist_comparison_pool_*`, `shortlist_rejects_moneyball_only_typed_inputs_*`, `tactic_shortlist_value_and_sort`) where they assert the removed contract. Keep any that still prove General composition by retargeting them to the new join.

**Behavior and data flow:**

- Click `Upload Shortlist`, pick a CSV in the native picker, parse `Player UID` in Rust, replace on match, and return a stored/total/skipped summary. The modal closes, filtering turns on, the URL updates, and the General query refetches with the restriction. Toggle off keeps the saved list and shows all current players. Toggle on with an empty or missing list shows the standard filtered empty state and the existing setup guidance, never the removed tab copy.

**Ordered implementation steps:**

1. Add the smallest RED proof: toggle key round-trips through the URL parser; upload success sets it on; Shortlist tab no longer renders.
2. Wire the button, modal, and toggle with the Commits 2 and 3 IPC surface.
3. Remove the tab, route state, layout, copy, and mocks that own no surviving behavior.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Observable behavior: upload turns filtering on; toggle composes with one General filter; Moneyball upload leaves membership untouched.
- RED proof at the parser and route seams: unknown toggle value parses to off; toggling updates the URL and refetches; `/search?view=shortlist` replace-normalizes to General with filtering ON.
- Absence proof for the removed tab: assert the Shortlist tab, its empty-state copy, and its layout key are gone. Add it only because tab reintroduction is plausible and observable.
- Do not inventory unchanged Moneyball, staff, planner, or snapshot tests.

**Patterns to verify:**

- `staff-shortlist-import-modal.tsx` for modal, summary, error, and context-guard shape.
- Existing filter-bar and count/sort line placement in `search-results-panel.tsx` for button and toggle positioning.
- `dynamic-columns.ts` and tactic-column toggles for column composition with the restriction.

**Constraints and non-goals:**

- Toggle is keyboard accessible with a truthful accessible name and state. Upload stays beside Edit Filters; toggle stays beside the count/sort line.
- Profiles and row navigation keep working; Shortlist row activation maps to the General profile view as before.
- Do not change staff code or current-state documents.

**Dependencies and sequencing:**

- Requires Commit 3 on the feature branch for the restriction IPC; builds on the Commit 2 storage table.

**Validation:** Focused Vitest for the parser, route, and store tests, then affected search suite, then focused `cargo test` for the touched Rust search modules plus `./scripts/dev check-rust`. Full `./scripts/dev check` before checkpoint.

**Stop conditions:** Stop for a developer decision when the modal cannot follow the staff analogue without a wider shared-component refactor.

**Review mandate:** Verify toggle placement (after-Edit seam; tactic controls unmoved) and keyboard access; truthful on/off representation; URL round-trip with invalid-to-off; upload-turns-on; exact `/search?view=shortlist` replace-normalization; composition with filters, sorts, tactic columns, Club DNA, and paging; Moneyball decoupling complete; Rust `SearchView::Shortlist` parse/query branch fully removed with no remaining reader; Player `shortlist` layout, tab, copy, tests, and mocks removed exactly where no supported behavior remains; no unrelated Search behavior changed.

#### Commit 5 — Compose shortlist filter into core staff search

**Status:** Completed

**Provisional commit:** `feat(staff): compose shortlist filter into search`

**Work:** One coherent, revertible Rust-only backend outcome: the core Staff Search query path accepts a shortlist-filter flag reusing the existing shortlist composition, with the semicolon CSV contract, replacement safety, metadata, Preferred Job, unemployment, and optimizer semantics preserved. No React/API change; the old `list_staff_shortlist` command/query path remains temporarily for the current frontend.

**Size assessment:** Roughly 80 to 150 changed non-test implementation lines in the staff query and commands. Within the soft target.

**Out of scope:**

- React/frontend API changes including `staffSearchQueryOptions` and other frontend fetch/key files are excluded; player changes, new staff storage, and current-state document reconciliation are excluded. The required Rust `commands.rs::search_staff` and query changes remain in scope.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/staff/query.rs::list_staff` plus reusable `list_staff_with_shortlist` — extend the core `list_staff` query path with shortlist on/off plus Preferred Job/unemployed parameters that apply the existing INNER JOIN to `staff_shortlist_entries` plus Preferred Job and unemployment predicates before count, sort, and paging. Preserve CSV metadata columns and mapped score columns/sort behavior exactly as `list_staff_shortlist` returns them today.
- `src-tauri/src/features/staff/commands.rs::search_staff` — expose the flag through bounded IPC. The existing `list_staff_shortlist` command and the dedicated `query.rs::list_staff_shortlist` wrapper are intentionally retained in this commit as compatibility for the still-current frontend; they retire in Commit 6. Retain reusable query helpers, tests, and presentation utilities. No `staffSearchQueryOptions` or other React/API change in this commit.
- `src-tauri/src/features/csv_import/{commands.rs,staff_shortlist.rs}` — no contract change. Verify replacement safety, exact UID matching, and the four-column semicolon contract still hold; successful upload still turns filtering on from the frontend, not from Rust.
- Rust tests beside `staff/query.rs` — flagged core search equals the standalone shortlist query for the same inputs; Preferred Job and unemployment predicates apply before paging; core filter-editor rules apply to shortlisted staff; setup states (no list, list with no current-snapshot match) report as today.

**Behavior and data flow:**

- Core search reads carry an optional shortlist flag with optional Preferred Job and unemployment inputs. When on, the query INNER JOINs save-owned entries to the effective current snapshot by UID, applies Preferred Job exact match and the trimmed blank/`-` unemployment rule, then applies core Staff filters, then counts, sorts, and pages. Staff assignment targets, Configure/Optimize inputs, and optimizer allocation read the same entries with unchanged semantics.

**Ordered implementation steps:**

1. Add the smallest RED proof: flagged core search returns the same rows, metadata, and totals as `list_staff_shortlist` for identical inputs.
2. Extend the `search_staff` command / `list_staff` query with shortlist on/off plus Preferred Job/unemployed parameters, routed through reusable `list_staff_with_shortlist`; make no React/API change.
3. Expose the parameters through commands.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Observable behavior: flagged core search and standalone shortlist query agree; Preferred Job and unemployment compose with one core Staff filter.
- Prove at the `staff/query.rs` seam. Keep the existing shortlist tests (`shortlist_joins_active_save_metadata_and_filters_before_paging`, setup-state tests) green or retargeted; add one equivalence test for the flagged core path.
- Do not inventory unchanged player, Moneyball, or snapshot tests.

**Patterns to verify:**

- The existing `list_staff_with_shortlist` predicate order: shortlist JOIN, then shortlist predicates, then general filters, then count, sort, paging.
- `assignment_optimizer_query.rs` for untouched allocation semantics.

**Constraints and non-goals:**

- No CSV contract change. No optimizer, target, or context-guard change. No new table or migration.
- Core Staff filter-editor rules apply to shortlisted staff when filtering is on.

**Dependencies and sequencing:**

- Requires Commit 1. Independent of Commits 2 through 4 except for shared PR sequencing; implement after Commit 4 so staff work builds on a green tree, or in either order if the tree stays green.

**Validation:** Focused `cargo test` for the staff module, then affected staff and csv_import tests, then `./scripts/dev check-rust`. Full `./scripts/dev check` before checkpoint.

**Stop conditions:** Stop for a developer decision when the flagged core path cannot reuse the `list_staff_with_shortlist` predicate order without a wider rewrite.

**Review mandate:** Verify flag reuse rather than a duplicated query; predicate order before paging; metadata preserved; Preferred Job exact match and unemployment rule unchanged; setup states reported as today; optimizer and target semantics untouched; `list_staff_shortlist` intentionally retained as compatibility until Commit 6 (trunk-safe), with reusable helpers, tests, and presentation utilities kept.

#### Commit 6 — Integrate shortlist controls into Staff Search page

**Status:** Active

**Provisional commit:** `feat(staff): integrate shortlist into staff search`

**Work:** One coherent, revertible frontend outcome: shortlist upload, `Configure Club Staff`, and `Optimize assignments` always visible in the Staff Search filter header; conditional Preferred Job and Only unemployed; URL-backed toggle beside the count/sort line; settled `staffShortlistPresentation(preferredJob)` mapping; atomic move of the frontend off the now-retired path — retire the now-unused Rust `list_staff_shortlist` command plus dedicated frontend wrappers in the same commit; removal of the My Club Staff Shortlist workspace/tab with its obsolete route state, copy, tests, mocks, and components. The Staff `staff-shortlist` persisted layout is retained. Every commit stays trunk-safe.

**Size assessment:** Roughly 150 to 250 changed non-test implementation lines across the staff route, results panel, URL parser, table store, and workspace removal. May exceed the soft target; keep it together because header controls, conditional filters, toggle, and workspace removal are one observable contract (removing the workspace without the integrated header strands the optimizer).

**Out of scope:**

- Player changes, unrelated Rust/behavior changes beyond the Commit 5 surface and the exact compatibility removals named below, and current-state document reconciliation. The exact compatibility removals (`commands.rs::list_staff_shortlist`, its `src-tauri/src/lib.rs` registration, and the dedicated `query.rs::list_staff_shortlist` wrapper) are in scope for this commit.

**Implementation packet:**

**Files and responsibilities:**

- `src/features/staff/components/staff-filter-bar.tsx` — integrated action slot hosting Upload, `Configure Club Staff` (rename `Configure slots` in `staff-assignment-target-modal.tsx`), and `Optimize assignments`, always visible in the Staff Search filter header; conditional shortlist metadata filters (Preferred Job and Only unemployed render only when filtering is ON).
- `src/app/routes/staff.tsx::StaffPageContent` — filter header always shows shortlist upload, `Configure Club Staff`, and `Optimize assignments` via that slot; successful upload turns filtering on; toggle beside the result count/sort line; core filter-editor rules apply to shortlisted staff; Configure stays usable without shortlist rows because save-owned target configuration does not require them; Optimize keeps its existing typed no-shortlist result/setup feedback with no new disable rule; legacy `/staff?view=shortlist` replace-normalizes to `/staff` with filtering ON without inspecting persistence (this route never intercepts `/my-club`), and the on-with-no-list state shows existing setup/no-shortlist feedback with Upload available.
- `src/app/routes/staff.tsx::StaffPageContent` owns the shortlist orchestration currently in `my-club.tsx` around the active save/current snapshot/managed club/Planner depth context: obtain active save ID + context token, snapshot ID + context token, the managed-club value, and the enabled Planner team identities/display names; retain route-owned shortlist import revision and import-pending state; build `shortlistContextKey`, `StaffAssignmentContext`, and the assignment context key from those exact values; derive `contextUnavailable` from save/snapshot/managed-club/Planner refresh or errors, the shared `playerResultContextMutationKey`, and import pending; pass those to `StaffAssignmentOptimizer`, preserving its own request-generation/result-acceptance guards; on successful staff import invalidate `staffKeys.all`, reset Preferred Job/Only unemployed, increment the import revision, and never restore stale feedback. Name the query loader/prefetch changes needed in `staff.tsx` (extend `loaderDeps`/`loader` and route-composition invalidation alongside `staffSearchQueryOptions`) so the integrated page carries the same current-owner guarantees the workspace had.
- `src/features/staff/components/staff-search-results-panel.tsx::StaffSearchResultsPanel` — core table remains the page. When filtering is OFF it uses the existing `staff-search` layout and normal Staff metrics; when ON it uses the existing persisted `staff-shortlist` layout and `staffShortlistPresentation(preferredJob)` (All jobs keeps its configurable CSV metadata layout; mapped jobs keep fixed identity/metadata plus mapped score and current sort; Coach keeps six outfield coaching scores without automatic score sorting; unrecognized jobs keep the current CA behavior) from `staff-shortlist-presentation.ts`.
- `src/features/staff/utils/staff-url-search.ts` — new shortlist on/off URL key plus Preferred Job and unemployment serialization following existing parser analogues; invalid resolves to off.
- `src/features/staff/api/{staff-keys.ts,fetch-staff.ts,staff-query-options.ts}` — this commit owns `staffSearchQueryOptions` and the frontend fetch/key changes onto the flagged core search query path via the Commit 5 parameters; retire the `shortlist` scope, `fetchStaffShortlist`, and `list_staff_shortlist` wrappers once the integrated route consumes the flagged core path. Retain reusable query helpers, tests, and presentation utilities.
- `src-tauri/src/features/staff/commands.rs::list_staff_shortlist`, its registration in `src-tauri/src/lib.rs`, and the dedicated `src-tauri/src/features/staff/query.rs::list_staff_shortlist` wrapper — atomically remove all three in this commit after the route and tests use the flagged core path. Retain the shared `list_staff_with_shortlist` logic and retarget its tests to the flagged core path.
- `src/app/routes/my-club.tsx::MyClubStaffShortlistWorkspace` — own the legacy `/my-club?view=staff-shortlist` replace-normalization to `/staff` with filtering ON without inspecting persistence, alongside removal of the workspace and tab; `src/features/my-club/components/my-club-workspace-tabs.tsx` — remove the Staff Shortlist tab entry; verify assignment targets and optimizer mount cleanly from the Staff Search header with identical context guards.
- `src/stores/use-player-table-store.ts` — retain the `staff-shortlist` persisted layout; remove only the Player `shortlist` layout (owned by Commit 4) and obsolete scope wrappers.
- `src/features/staff/components/{staff-shortlist-import-modal.tsx,staff-assignment-optimizer.tsx,staff-assignment-results.tsx,staff-assignment-target-modal.tsx}` — reuse modals and optimizer in place; remove only components that own no surviving behavior.
- Tests: extend `src/app/routes/staff.test.tsx`, `src/app/routes/legacy-club-routes.test.tsx`, `src/features/staff/utils/staff-url-search.test.ts`, and optimizer tests for header controls, conditional filters, toggle round-trip, and upload-turns-on. Prove both exact replace-normalizations without persistence inspection: `/staff?view=shortlist` in `staff.tsx` and `/my-club?view=staff-shortlist` in `my-club.tsx`, each to `/staff` with filtering ON. Retarget the valuable context-change, delayed-result, and pending-mutation tests from `src/app/routes/my-club-squad.test.tsx` to `staff.test.tsx`/existing optimizer tests — do not delete them with the workspace tests. Remove workspace/tab tests, obsolete empty-state copy, and `src/testing/staff-ipc-mock.ts` shortlist paths (plus `src/testing/setup.ts` wiring) that assert the removed contract; update `e2e/tauri-ipc-stub.ts` and `e2e/smoke.spec.ts` only where they assert the retired workspace/contract — retarget or remove only tests and fixtures that assert retired routes/contracts. Preserve staff profiles and row navigation tests.

**Behavior and data flow:**

- The filter header always shows upload, Configure, and Optimize. Upload success replaces the save-owned list, turns filtering on, updates the URL, and refetches the flagged core search. Preferred Job and Only unemployed appear only when filtering is ON and compose with core filters. With no list, the on state shows the existing setup/no-shortlist feedback with Upload available; Configure stays usable and Optimize returns its existing typed no-shortlist result/setup feedback. Row activation still opens Staff Profiles.

**Ordered implementation steps:**

1. Add the smallest RED proof: toggle key round-trips; Preferred Job and Only unemployed render only when filtering is on; workspace tab no longer renders.
2. Move header controls and conditional filters with the Commit 5 IPC surface.
3. Remove the workspace, route state, copy, tests, and mocks that own no surviving behavior; keep the `staff-shortlist` persisted layout.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- Observable behavior: upload turns filtering on; Preferred Job plus one core Staff filter composes; Configure/Optimize behave as in the workspace.
- RED proof at parser and route seams: invalid toggle resolves to off; on-with-no-list state shows the existing setup message with Upload available and Optimize typed feedback; both `/staff?view=shortlist` and `/my-club?view=staff-shortlist` replace-normalize to `/staff` with filtering ON without inspecting persistence (covered in `staff.test.tsx` / `legacy-club-routes.test.tsx`).
- Absence proof for the removed workspace: assert the My Club shortlist tab and its empty-state copy are gone. Add it because workspace reintroduction is plausible and observable.
- Do not inventory unchanged player, planner, or snapshot tests.

**Patterns to verify:**

- `staff-shortlist-import-modal.tsx` for upload, summary, replacement warning, and context guards.
- `staff-assignment-optimizer.tsx` and target modal for unchanged Configure/Optimize behavior.
- `staff-shortlist-presentation.ts` for Preferred Job column and sort mapping.

**Constraints and non-goals:**

- Toggle is keyboard accessible with a truthful on/off representation. Rename only: `Configure slots` becomes `Configure Club Staff` with no behavior change.
- Staff profiles and row navigation keep working.
- Do not change player code or current-state documents.

**Dependencies and sequencing:**

- Requires Commit 5 on the feature branch for the flagged core search IPC.

**Validation:** Focused Vitest for the staff URL parser, route, and optimizer tests, then the affected staff suite, then focused `cargo test` for the retired-command seam plus `./scripts/dev check-rust`. Full `./scripts/dev check` before checkpoint. `CI=1 ./scripts/dev smoke` at feature validation before close-out.

**Stop conditions:** Stop for a developer decision when assignment targets cannot mount outside My Club without a wider refactor.

**Review mandate:** Verify always-visible Upload, Configure Club Staff, and Optimize assignments via the integrated action slot; Configure usable without rows; Optimize existing typed no-shortlist feedback with no invented disable rule; conditional Preferred Job and Only unemployed; settled `staffShortlistPresentation` mapping with the retained `staff-shortlist` layout; toggle placement, keyboard access, and truthful state; URL round-trip with invalid-to-off; exact legacy replace-normalizations (`/staff?view=shortlist` owned by `staff.tsx`, `/my-club?view=staff-shortlist` owned by `my-club.tsx`, both to `/staff` ON without persistence inspection); upload-turns-on with `staffKeys.all` invalidation, Preferred Job/Only-unemployed reset, import-revision increment, and no stale feedback; immutable context ownership (`shortlistContextKey`, `StaffAssignmentContext`, assignment context key, `contextUnavailable`) identical to the workspace; core filter-editor rules apply to shortlisted staff; Configure/Optimize identical to workspace behavior; now-unused `commands.rs::list_staff_shortlist`, its `src-tauri/src/lib.rs` registration, and the dedicated `query.rs::list_staff_shortlist` wrapper plus dedicated frontend wrappers fully retired with no remaining reader while shared `list_staff_with_shortlist` logic is retained and its tests retargeted; workspace, tab entry, route state, copy, tests, and mocks removed exactly where no supported behavior remains; context-change/delayed-result/pending-mutation tests retargeted, not deleted; Player `shortlist` layout removed and Staff `staff-shortlist` layout retained; profiles and row navigation preserved.

## Active work

**PR:** feat(search): integrate player and staff shortlists

**Commit:** Integrate shortlist controls into Staff Search page

### RED or removal proof

Add focused frontend proofs that the Staff shortlist URL flag round-trips, metadata filters appear only when enabled, the My Club shortlist tab is absent, and both legacy URLs replace-normalize to integrated Staff Search.

### Expected outcome

`/staff` owns shortlist upload, configuration, optimization, conditional metadata filters, URL-backed filtering and retained shortlist presentation; the My Club workspace and dedicated backend/frontend query paths retire atomically.

### Explicit exclusions

Player changes, current-state documentation, unrelated Staff behavior, and unrelated refactors.

## Discoveries and replanning

Orchestrator correction round settled the Staff presentation mapping (OFF uses `staff-search` layout and normal metrics; ON uses the persisted `staff-shortlist` layout with the exact `staffShortlistPresentation(preferredJob)` cases), the exact legacy replace-normalization without persistence inspection, the exact player CSV contract and summary fields, the settled `search_staff`/`list_staff` extension routed through `list_staff_with_shortlist` with `list_staff_shortlist` retirement in Commit 6, always-visible Staff header controls, and the Commit 2/3 backend split. No implementation evidence contradicts the delivery boundary.

Correction round 1 (plan-review verdict) made Commits 3/4 and 5/6 trunk-safe by retaining the old backend contracts (`SearchView::Shortlist`, `list_staff_shortlist`) through the flag commits and retiring each atomically with its frontend move; hardened Commit 2 proof (row-count boundary before validity/deduplication, insert-failure rollback, Moneyball non-interference, forward-migration/cascade/isolation/snapshot-survival proof, no separate save index); settled frontend ownership (`search-filter-bar.tsx`/`search-filter-strip.tsx` after-Edit seam, `player-shortlist-import-modal.tsx` + import API + summary types, `staff-filter-bar.tsx` action slot, `my-club-workspace-tabs.tsx`, testing/e2e assets); assigned immutable shortlist context ownership to `staff.tsx::StaffPageContent`; and recorded the exact DESIGN reconciliation facts for the workflow-core close-out owner. One PR and six commits preserved; Delivery fingerprint stays pending review.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — feat(search): integrate player and staff shortlists | Commit 1 — Record the approved feature plan | d5b958618df579b99b0c778a55178ca034995f43 | Recorded the reviewed schema 2 ledger and activated the feature in TODO without changing executable behavior. | `ledger_state.py`; `delivery_state.py`; `git diff --cached --check`; pre-commit `check-fast` — passed. | Not applicable | Clear | 0 | None |
| PR 1 — feat(search): integrate player and staff shortlists | Commit 2 — Add save-owned player shortlist store and import | 905f716e368450adaacb49e2d1fe8cc62a5eecc4 | Added migration v41 and a bounded save-owned player shortlist CSV import with context revalidation, atomic replacement, zero-match preservation, and exact summary counts. | `cargo test player_shortlist`; `cargo test csv_import`; `./scripts/dev check-rust`; `./scripts/dev check` (768 passed, 2 ignored); Rust LSP; `git diff --check` — passed. | Pass | Clear | 0 | MEDIUM deferred to feature close-out: remove the module-level dead-code suppression and test-only internal skip counters if the final portfolio confirms they add no contract value. The one-column delimiter fallback is required for the exact UID-only header. |
| PR 1 — feat(search): integrate player and staff shortlists | Commit 3 — Restrict General search to the player shortlist | abe2c2d496256ca3e6037f5e29e99ac9f3c00cf1 | Added a General-only, save-scoped shortlist restriction through bounded IPC while preserving the legacy Moneyball-backed Shortlist contract for the current frontend. | `cargo test --lib features::search::` (102 passed); `./scripts/dev check-rust`; `./scripts/dev check` (769 passed, 2 ignored); Rust LSP; `git diff --check` — passed. | Pass | Clear | 0 | Existing tactic and Club DNA tests provide stronger composition coverage through the unchanged shared query pipeline; no duplicate flagged variants added. |
| PR 1 — feat(search): integrate player and staff shortlists | Commit 4 — Integrate player shortlist upload and toggle in Search | 1b47488b482f7081fd319e6277a37b7eafb6a561 | Integrated save-owned shortlist upload and URL filtering into General Search, removed the separate Player Shortlist tab/layout and Rust legacy branch, and preserved Moneyball isolation and profile navigation. | Search route 70 passed; focused parser/key/store/modal tests 51 passed; Rust Search 98 passed; `CI=1 ./scripts/dev smoke` 54 passed; `./scripts/dev check` (765 passed, 2 ignored); `git diff --check` — passed. | Pass | Clear | 2 | Corrected zero-result switch access, true replace-normalization, immutable token guards, complete summary feedback, Playwright Moneyball fidelity, redundant tests, and filtered Moneyball empty-state precedence. |
| PR 1 — feat(search): integrate player and staff shortlists | Commit 5 — Compose shortlist filter into core staff search | Pending record | Extended core Staff Search with optional shortlist, Preferred Job, and unemployment parameters through the shared shortlist query composition while retaining the legacy command for the current frontend. | `cargo test --lib features::staff::` (68 passed); combined bind proof 1 passed; `./scripts/dev check-rust` (766 passed, 2 ignored); `./scripts/dev check`; Rust LSP; `git diff --check` — passed. | Pass | Clear | 1 | Reviewer-required combined bind proof consolidated shortlist, Preferred Job, unemployment, core filter, metadata, and paging into one query test. |

## Final validation

- `./scripts/dev test` for the affected search, staff, csv_import, and store suites.
- `./scripts/dev check` as the commit gate for every implementation commit.
- `CI=1 ./scripts/dev smoke` for the assembled product path before feature close-out.
- Manual evidence only where automation cannot prove the contract: native-picker upload, keyboard toggle operation, and restart/snapshot-replacement persistence for both lists.
- Close-out reconciliation covers `.wiki/CONCEPT.md`, `.wiki/ARCHITECTURE.md`, `.wiki/DESIGN.md`, `.wiki/TODO.md`, and the completed feature record (see Documentation impact for the exact DESIGN facts).

## Documentation impact

Complete during reconciliation. After implementation makes the new behavior true, reconcile `.wiki/CONCEPT.md`, `.wiki/ARCHITECTURE.md`, `.wiki/DESIGN.md`, `.wiki/TODO.md`, and the completed feature record. No ADR is planned. No BACKLOG change is planned. No normal implementation packet is reserved for close-out: workflow-core owns the close-out commit, so this section plus Final validation state the exact DESIGN facts the close-out owner must reconcile and cannot omit:

- Status paragraph (implemented surfaces): the automatic Moneyball-cohort Player Shortlist view with General metrics and independent layout becomes the save-owned persistent replacement list with `Upload Shortlist` and URL toggle on `/search` General; the save-owned Staff Shortlist workspace entry becomes the integrated Staff Search header (Upload, `Configure Club Staff`, `Optimize assignments`) with URL toggle.
- Compact Filter Strip / Filter Tag / Filter Editor: the settled after-Edit action seam (`Upload Shortlist` immediately right of `Edit Filters`, tactic controls unmoved) and the Staff integrated action slot with conditional Preferred Job / Only unemployed.
- Search tactic columns: tactic columns compose with the General shortlist restriction.
- Modal: the new `player-shortlist-import-modal.tsx` alongside the staff import modal (summary, zero-match error, context-change guards).
- Empty, Loading, and Error States: removed Player Shortlist tab copy (`No shortlist yet`) and My Club Staff Shortlist workspace copy replaced by on-with-no-list setup/no-shortlist feedback with Upload available.
- Staff workspace layout and Squad workspace layout: Staff Shortlist leaves the My Club workspace; `staff-shortlist` persisted layout retained on `/staff`, Player `shortlist` layout removed.
- Current-state docs remain unchanged during planning; reconciliation happens only after implementation makes the new behavior true.
