# Player Shortlist

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** d21c4902558e323535302f7bd43f8ba0c209a7b0f72c211082b559c40db1a63e

## Intent

Add **Shortlist** as the third tab beside **General** and **Moneyball** in Player Search (`/search`). Membership is automatic from the current Moneyball CSV cohort — no manual add, remove, or assignment. Shortlist lets a recruiter work a narrowed Moneyball cohort with the familiar attribute and General-role-score table experience.

Linear JAY-52 originally described manual curation; the developer explicitly replaced that wording with automatic current Moneyball-cohort membership. This ledger records the approved replacement intent and supersedes the older manual description.

## User-visible behavior

- Player Search shows three tabs: **General**, **Moneyball**, **Shortlist**. The existing Moneyball tab keeps the **Upload Moneyball CSV** action. Shortlist has no upload control.
- **General** and **Shortlist** use the same attribute-based metrics: identity, club/contract, ability/reputation, visible/hidden attributes, personality, position suitability, current role scores (`role.*`), potential role scores (`potential_role.*`), and `club_dna` — plus sortable/filterable General columns. Neither view exposes Moneyball statistics (`moneyball.*`) or Moneyball role scores (`moneyball_role.*`).
- **Moneyball** keeps the closed Moneyball catalog: identity/context, `moneyball.*` statistics, and `moneyball_role.*` roles with `filtered`/`fullCsv` pool selection. Shortlist does not offer the pool toggle.
- Switching tabs uses a keyboard-operable three-tab pattern (ArrowLeft/Right, Home/End, roving tabIndex) in **General, Moneyball, Shortlist** order — ArrowRight from General lands on Moneyball and End lands on Shortlist. Selecting any view clears all filters and resets to that destination's default sort/direction (`CA` descending for General and Shortlist; `moneyball.average_rating` descending for Moneyball), as `SearchPageContent` does now. Direct URLs still drop invalid filters via route validation (`parseSearchFilters`).
- Shortlist is read-only apart from existing Search interactions: virtualized paging, header sort, column picker/menus, resize, and whole-row activation.
- Shortlist keeps an independent persisted table layout. It follows the same ownership as `search`, `moneyball-search`, `squad`, and staff tables: ordered visible metric IDs plus clamped widths stored in `use-player-table-store.ts`. Default columns match General (`name`, `age`, `nationality`, `club`, `division`, `ca`, `pa`, `value` without duplicate `club`/`division` handling) and default sort is `CA` descending.
- Clicking a Shortlist row opens `/players/$uid?view=general`. Profile navigation maps `shortlist` to `general` because profiles only accept General or Moneyball analysis views. Global Ctrl+K and General row activation remain unchanged.
- Shortlist empty-state presentation is determined solely from existing `SearchPlayersPage` inputs (`total` and `filters.length`): an unfiltered Shortlist result with `total === 0` (`filters.length === 0`) shows a neutral “No shortlist yet” empty state that directs to the **Moneyball** tab for upload; any filtered Shortlist result with `total === 0` (`filters.length > 0`) shows the standard “No players match these filters” state, regardless of whether the underlying Moneyball cohort is empty — clearing filters then reveals the neutral guidance if the cohort is empty. A direct URL that carries a valid General filter follows this same precedence (`total === 0 && filters.length > 0` → filtered empty state). This is a presentation distinction, not cohort truth; no cohort-existence signal, companion unfiltered query, extra IPC field/shape, or DTO extension is added.
- With a cohort present, Shortlist paginates with the same bounded 50-row IPC pages and 200 max limit. Totals reflect the filtered cohort size where applicable and use the same read-only current-snapshot compact role model.

## Invariants

- One Moneyball cohort exists per effective current snapshot in `player_moneyball_stats` `(snapshot_id, player_uid)`. Shortlist membership follows that table exactly — no new shortlist persistence, table, column, migration, or file.
- Membership uses an inner join to current-snapshot `player_moneyball_stats`. It does not depend on `percentiles_json` readiness, Moneyball metric thresholds, or any derived cache.
- The existing cumulative contract applies unchanged: matched rows upsert `player_moneyball_stats`; omitted rows remain; a newly current snapshot has no shortlist until its own Moneyball import; snapshot deletion removes rows via FK cascade; empty or zero-match imports are no-ops for the cohort.
- Shortlist uses only General filter/sort/requested-field validation and the existing compact role read models (`player_role_metrics`, persisted `club_dna_scores` where applicable). No new formula, role catalog, or scoring path.
- No read-time writes. Completeness is validated before reads via `assert_read_models_complete`; no materialization or repair occurs at query time.
- Upload ownership stays in Moneyball. Shortlist does not add an upload IPC, dialog, or capability. Moneyball import still invalidates `searchKeys.all` and Shortlist refreshes from that same cache root.
- App default remains General or Moneyball only via `useMoneyballPreferences`. Shortlist is never a Settings default and cannot be set there. A valid `view` URL parameter wins; absent view uses the app-local preference; Shortlist requires an explicit `view=shortlist` URL.

## Non-goals

- Manual add, remove, assignment, ordering, status, notes, or recommendation integration for shortlisted players.
- New shortlist persistence, migration, identity source, or management command.
- Moneyball statistics or Moneyball role-score metrics inside Shortlist filters, columns, or sorts.
- Historical cohort browsing, trends, comparisons, or analytics beyond the current snapshot.
- Squad columns, Youth Tracker changes, Staff Shortlist changes, bridge or snapshot-selection changes, or a second Moneyball upload path.
- Duplicating the Moneyball upload action inside Shortlist.
- Changing Global search or making Shortlist a default-view option in Settings.

## Current-state map

- **Relevant components:** `src-tauri/src/features/search/query.rs` (`SearchView`, `SortField`, `search_players_in_view`, `search_players_with_roles`), `src-tauri/src/features/search/commands.rs` (`search_players`, `parse_search_view`, default sort), `src-tauri/src/features/search/filter.rs` (`compile_filters`, `compile_filters_for_moneyball`), `src-tauri/src/features/player_metrics/resolver.rs` (`MetricField::parse_for_moneyball`, `is_moneyball_search_field`, `parse_requested_fields_for_moneyball`), `src-tauri/src/features/player_metrics/compact.rs` (read-model joins and completeness), `src-tauri/src/db/migrations.rs` (`player_moneyball_stats` PK `(snapshot_id, player_uid)` and cascades, `player_role_metrics` compact 68/68 columns), `src-tauri/src/features/csv_import/service.rs` (cumulative upserts and full-cohort percentiles), `src/features/search/types/search-view.ts` (`SearchView`, `defaultSearchSort`, `parseSearchView`), `src/app/routes/search.tsx` (two-tab list, `validateSearch`, `loaderDeps`, `comparisonPool` scoping, `Upload Moneyball CSV`, `searchKeys.all` invalidation), `src/features/search/utils/filter-registry.ts` (`filterFieldsForView`, `getFilterField`), `src/features/search/utils/search-url-search.ts` (`parseSearchFilters`, `searchFiltersForUrl`), `src/features/search/types/search-sort.ts` (`DEFAULT_SEARCH_SORT_FIELD`, `defaultDirForSortField`), `src/features/search/components/search-results-panel.tsx` (tableId mapping, column metric resolution, `onPlayerActivate` profile navigation), `src/stores/use-player-table-store.ts` (`PLAYER_TABLE_LAYOUT_VERSION`, `PlayerTableId`, `defaultLayout`, `sanitizeLayout`, `migratePersistedState`), `src/testing/search-ipc-mock.ts` (`resolveSearchPlayersIpcMock`, `SearchPlayersPageIpcMockMode`), `e2e/tauri-ipc-stub.ts` and `e2e/smoke.spec.ts` (stubbed IPC smoke).
- **Data model:** `players` current snapshot plus `player_role_metrics` (one row per current player, 68 current and 68 potential nullable columns plus model versions) and `club_dna_scores` for General-family reads. `player_moneyball_stats` `(snapshot_id, player_uid)` with cascading snapshot/player FKs holds the Moneyball cohort; Moneyball-only reads join it with `percentiles_json IS NOT NULL`. Shortlist must restrict with `INNER JOIN player_moneyball_stats` without that predicate for membership-only filtering.
- **Persistence and migrations:** `player_moneyball_stats` schema already enforces cohort ownership; snapshot deletion removes current-format Moneyball rows. No migration is warranted for Shortlist. Adding one would create upgrade risk with no new enforceable state.
- **Existing behavioral assumptions:** `SearchView::General` and `SearchView::Moneyball` are the only parsed views; `search_players` defaults to `ca` descending for General and `moneyball.average_rating` descending for Moneyball. Moneyball filtering/sorting validates against `MONEYBALL_SEARCH_METRICS`; General validates against `PLAYER_METRICS`. `searchKeys.all` is invalidated after `SquadCsvImportModal` `onMoneyballImported` in both Search and My Club.
- **Architectural seams:** `SearchResultsPanel` owns committed/requested query observers, sort-replacement retention, virtual paging, and row activation. `searchPlayersQueryOptions` carries `searchView` and `comparisonPool` in the query key. Player Shortlist membership is a read-only `INNER JOIN player_moneyball_stats` on `(snapshot_id, player_uid)` scoped to the active save's current snapshot, without `percentiles_json` readiness.
- **Project validation commands:** `./scripts/dev test [target...]`, `./scripts/dev check-rust`, `./scripts/dev check`, `./scripts/dev smoke` (Chromium via `playwright.config.ts`; `pnpm exec playwright install chromium` prerequisite).
- **Primary risks:** Moneyball-only filters/columns persisting across a view switch and producing empty or errored queries; Shortlist accidentally reusing Moneyball resolvers or showing Moneyball fields; membership tied to `percentiles_json` so a cohort is invisible before full enrichment; layout migration losing existing `search`/`moneyball-search` widths; profile navigation passing `view=shortlist` to a profile that only knows General/Moneyball; neutral empty state duplicating the upload control; tab keyboard behavior regressing.

## Feature architecture

Rust adds `SearchView::Shortlist`. Its parse, request validation, and `SortField` default mirror General. Query execution validates `requested_fields` via `parse_requested_fields_for_moneyball(requested_fields, view == SearchView::Moneyball)` passing `false` for Shortlist, and validates filters/sort via `MetricField::parse_for_moneyball` with `moneyball = false` and the General `compile_filters` path. It uses the compact General role/attribute join path. The sole added predicate is `FROM players INNER JOIN player_moneyball_stats shortlist ON shortlist.snapshot_id = players.snapshot_id AND shortlist.player_uid = players.uid` scoped to the active save's `is_current = 1` snapshot. `parse_comparison_pool` accepts closed pool values for all command requests; the pool affects only Moneyball query behavior, while frontend URL state exposes it only for Moneyball. No `percentiles_json` predicate, no read-time `INSERT`/`UPDATE`, and no new write transaction.

No new IPC shape, Tauri command shape, or capability is required; `search_players` gains only a third accepted `search_view` string and a General-matching default sort. The Rust DTO remains `PlayerSummary`/`SearchPlayersPage`; no Moneyball percentile `statistics_json` path is added for Shortlist.

Frontend extends `SearchView` to `"general" | "moneyball" | "shortlist"` in `search-view.ts` (`parseSearchView`, `defaultSearchSort`, `isSearchView`-style guard). Route validation in `search.tsx` accepts the third value, keeps `comparisonPool` in URL state only for `moneyball` (backend `parse_comparison_pool` already accepts closed values for any view; only Moneyball query behavior uses it), clears all filters and resets to the destination default sort/direction on any tab change, and renders a three-tab `tablist` (`General` / `Moneyball` / `Shortlist`) in that order — ArrowRight from General lands on Moneyball and End lands on Shortlist — with ArrowLeft/Right/Home/End and roving tabIndex focus management. `filter-registry` maps Shortlist to the General metric catalog. Shortlist is a General-family view for `src/features/search/utils/dynamic-columns.ts`: `isVisibleSortField` and `dynamicColumnFields` must treat `shortlist` like `general` so General sorts including `ca`/`pa` and dynamic General fields (including current `role.*`, `potential_role.*`, `club_dna`, `attr.*`, `position`/`pos.*`) remain visible and Moneyball-only sorts (`moneyball.*`, `moneyball_role.*`) remain rejected. URL helpers (`search-url-search.ts`) drop invalid filters on direct URL loads via `parseSearchFilters`.

`SearchResultsPanel` maps `tableId` for Shortlist to an independent layout (`"shortlist"`). With `total === 0 && filters.length === 0` it renders the Shortlist-specific neutral empty state (“No shortlist yet”) with a link-like action directing to the Moneyball tab; with `total === 0 && filters.length > 0` it renders the standard “No players match these filters” empty state regardless of whether the underlying cohort is empty (clearing filters then reveals the neutral guidance if the cohort is empty; a direct URL with a valid General filter follows this same rule); with rows it renders the virtualized full-height table with General column metrics only. This distinction uses only existing `total` and `filters.length` from `SearchPlayersPage` — no cohort-existence signal, companion unfiltered query, extra IPC field/shape, or DTO extension. Row activation navigates with `view=general` for Shortlist (Moneyball passes through its own view). `use-player-table-store.ts` bumps `PLAYER_TABLE_LAYOUT_VERSION` to `6`, adds `PlayerTableId` `"shortlist"`, seeds its default from `DEFAULT_PLAYER_TABLE_COLUMN_IDS` via `withoutDuplicateIdentityColumns`, and migrates sanitization without touching existing `search` and `moneyball-search` layouts.

Moneyball import still invalidates `searchKeys.all`; Shortlist consumers already key on `searchView`/`view` so they refetch without a new invalidation root. Profile navigation guards Shortlist to General before `navigate({ to: "/players/$uid", search: { view } })`.

## Uncertainty register

### Known

- Current `query.rs` `SearchView` has two variants; `commands.rs` `parse_search_view` accepts only `general`/`moneyball` and `SortField::DEFAULT` is `ca`. `search_players_in_view` handles Moneyball with `INNER JOIN player_moneyball_stats ... AND moneyball.percentiles_json IS NOT NULL` and a separate role-scoring path for `moneyball_role.*` fields.
- `player_moneyball_stats` composite PK and FK cascades already own per-snapshot cohort membership; snapshot deletion removes rows. The cumulative import feature now upserts matched rows and retains omitted rows.
- Frontend `validateSearch` already clears sort when `isVisibleSortField(sort, filterRules, view)` is false and defaults direction via `defaultDirForSortField`. The tab keyboard handler already implements ArrowLeft/Right/Home/End for two tabs.
- `use-player-table-store.ts` persists per-table ordered `columnIds` and clamped widths with versioned migration; version `5` removed duplicate `club`/`division`.

### Assumptions

- Spotlighting Shortlist through the existing `search_players` command with a third `search_view` value is sufficient. No new command or IPC DTO is needed because Shortlist selects General columns from the same `players`/`player_role_metrics` read models filtered by cohort membership.
- `percentiles_json` can remain populated by the existing percentile transaction but must not gate Shortlist membership. `assert_read_models_complete` rejects incomplete compact models before reads; individual nullable metrics can show `—` only after that completeness gate passes. No `—` fallback is assumed for an incomplete model.
- Reusing `searchKeys.all` invalidation after Moneyball import is sufficient to refresh Shortlist totals and pages without a dedicated shortlist key.

### Decisions

- Use exactly one PR. The Rust predicate change and the frontend tab/layout/empty-state integration share one review surface and no independently publishable seam justifies a second PR. A walking skeleton that can load a filtered Shortlist page after a cumulative Moneyball import is the first mergeable proof.
- Implement Shortlist membership as `INNER JOIN player_moneyball_stats` on `(snapshot_id, uid)` without `percentiles_json IS NOT NULL`. This matches the approved cohort truth and keeps snapshot replacement behavior symmetric.
- Give Shortlist an independent `shortlist` layout in `use-player-table-store.ts` and bump `PLAYER_TABLE_LAYOUT_VERSION` from `5` to `6`. Other layouts remain stable; migration resets the Shortlist layout only on malformed stored state.
- Default Shortlist sort to `ca` descending and default columns to General defaults. Do not add Shortlist to `useMoneyballPreferences`.
- Route view scoping: `parse_comparison_pool` accepts closed pool values (`filtered`/`fullCsv`) for all command requests; the pool affects only Moneyball query behavior, while frontend URL state exposes `comparisonPool` only when `view === "moneyball"`. `shortlist` is a General-family view that uses the General default branch and ignores the pool for its query.
- Profile mapping: `shortlist` → `general` before navigation because `/players/$uid` only accepts General or Moneyball analysis views.
- No ADR. The work extends the existing `SearchView` seam, the existing table-layout ownership pattern, and the existing import cascade. No competing durable alternative meets the ADR threshold.

### Unknowns

- Whether Playwright stub smoke plus temporary-SQLite Rust tests can cover native WebView focus restoration at 1280×800 and 1600×900 without Windows Tauri verification. That remains an explicit manual gap if unavailable.
- Whether the representative snapshot used for Windows verification contains a realistic overlapping Moneyball import to observe cumulative Shortlist membership before teardown.

### Risks

- Persisting Moneyball-only `filters` or `sort` into a Shortlist URL would fail `parseSearchFilters` validation or return zero rows. Selecting any view must clear all filters and reset to that destination's default sort/direction in one navigation.
- An unscrubbed `requested_fields` containing `moneyball.*` or `moneyball_role.*` would bypass Shortlist's General contract if the resolver is not switched to General-only validation.
- Adding `percentiles_json IS NOT NULL` to the Shortlist join would hide newly upserted members until the next percentile recomputation and break the invariant that cohort presence equals Shortlist presence.
- A table-layout migration that mutates `search` or `moneyball-search` would lose user custom widths and need a restore loop.
- Allowing `view=shortlist` to flow into `onPlayerActivate` would surface an unknown view in the profile loader and violate the General/Moneyball profile contract.

## Walking skeleton

Load/ingest a current snapshot that has no Moneyball import, open unfiltered Shortlist (third tab in **General, Moneyball, Shortlist**; ArrowRight from General lands on Moneyball and End lands on Shortlist) and verify the neutral “No shortlist yet” empty state directing to the **Moneyball** tab while the current snapshot remains loaded. Then import a Moneyball CSV from the Moneyball tab (second tab), switch back to Shortlist, and verify only cohort members appear with General columns (`CA` descending), attribute and General role-score filters via the compact-table experience, and that clicking a row opens the profile in General analysis view.

## Delivery plan

### PR 1 — Add Player Shortlist tab to Player Search

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/player-shortlist`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub strict required status `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(search): add Player Shortlist tab`

**Purpose:** Deliver the complete Shortlist surface — Rust cohort-filtered General search, three-tab navigation, independent persisted layout, Shortlist-specific empty state, and General-mapped profile navigation — as one trunk-safe review boundary.

**Depends on:** Linear JAY-52 approved intent and the completed Player Search, Compact Metrics, Configurable Player Tables, Moneyball Views, Moneyball Role Scores, and Cumulative Moneyball Imports foundations already on `main`.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(shortlist): record approved feature plan`

**Work:** Commit the independently reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/player-shortlist.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — Active work link to this ledger; leave the gender investigation in Next.
- `.wiki/BACKLOG.md` — deliberately unchanged because no accepted deferred scope changes.
- `.wiki/features/planned/player-shortlist.md` — not present, so no promoted spec removal.
- `.wiki/decisions/` — deliberately unchanged because no ADR meets the threshold.

**Behavior and data flow:**

- Move planning truth from this provisional diff into one reviewed active ledger and record the exact delivery sequence before implementation.

**Ordered implementation steps:**

1. Verify the active branch is `feature/player-shortlist` and the base is `main` without changing Git state.
2. Confirm the worktree contains only the reviewed planning paths.
3. Run the ledger classifier and any repository documentation check.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The ledger classifier and documentation checks prove structural consistency.

**Patterns to verify:**

- The active-ledger template, current TODO/BACKLOG ownership rules, and relevant accepted ledger format.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, plan scope, packet order, or reviewed decisions.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/player-shortlist.md` plus the repository documentation check when one exists.

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files.

#### Commit 2 — Add Shortlist view and cohort-filtered General query

**Status:** Active

**Provisional commit:** `feat(search): add Shortlist view and cohort query`

**Work:** Introduce the third `SearchView::Shortlist` variant, General-gated validation for Shortlist requests, and a cohort-restricted General query path backed by the current-snapshot Moneyball table.

**Size assessment:** Estimated 120–200 changed non-test implementation lines in Rust. Within the soft target; `SearchView`, parsing, validation, and the joined cohort query form one revertible query contract.

**Out of scope:**

- Frontend tabs, table layout, empty-state copy, Moneyball import UI, Squad/Staff changes, and additional migrations.

**Implementation packet:**

- Extend the search backend so Shortlist uses General filter/sort/requested-field validation and compact attribute-role read models but restricts rows to current-snapshot `player_moneyball_stats` members without relying on percentile readiness or adding read-time writes.

**Files and responsibilities:**

- `src-tauri/src/features/search/query.rs` — add `SearchView::Shortlist`, helper `is_general_family_view` if needed, update `SearchView` branching in `search_players_in_view` and `search_players_with_roles` guard, add the Shortlist `INNER JOIN player_moneyball_stats shortlist` predicate for the General-family cohort path, keep `assert_read_models_complete` and `player_metrics_join` behavior unchanged.
- `src-tauri/src/features/search/commands.rs` — extend `parse_search_view` for `shortlist`, extract a small private pure sort parser/helper (e.g., `parse_search_sort_for_view` or `resolve_sort_for_view`) from the existing `search_players` match so the command boundary owns raw-string validation, set `SortField::DEFAULT` (`ca` descending) as the Shortlist default when `sort_by` is absent, map accepted explicit General sorts (including representative `pa`/`attr.*`/`role.*`/`potential_role.*`/`club_dna`) through the General `SortField::parse` path and reject Moneyball-only sorts (`moneyball.*`, `moneyball_role.*`), add Shortlist to the General-family default branch for `parse_comparison_pool` (accepts closed values for any view; Shortlist query ignores the pool, which affects only Moneyball behavior). Existing `parse_search_view`/`parse_comparison_pool` unit tests are extended to cover the new `shortlist` branch.
- `src-tauri/src/features/search/filter.rs` — no shape change; verify Shortlist uses the General compilation path.
- `src-tauri/src/features/player_metrics/resolver.rs` — no shape change; Shortlist uses `MetricField::parse_for_moneyball` with `moneyball = false` via `parse_requested_fields_for_moneyball(requested_fields, false)` and must reject `moneyball.*` and `moneyball_role.*` inputs (e.g., `moneyball.goals`, `moneyball_role.wbl_wbr_wing_back_ip`).
- `src-tauri/src/db/migrations.rs` — deliberately retained unchanged; cohort ownership already exists.
- `src-tauri/src/features/search/commands.rs` tests — own command-boundary proof for raw `search_view`/`sort_by` string parsing (see Tests and proof).
- `src-tauri/src/features/search/query.rs` tests — prove cohort filtering, snapshot isolation, and typed `SortField`/`requested_fields`/`filters` SQL behavior after parsing; modify only the minimum necessary existing Moneyball join tests to keep the `percentiles_json IS NOT NULL` predicate scoped to Moneyball. Query tests do not claim proof of raw-string sort parsing.

**Behavior and data flow:**

- `commands::search_players(search_view = "shortlist", sort_by = …, ...)` parses raw `search_view` and `sort_by` strings at the IPC boundary before any typed `SortField` reaches `query.rs`. `parse_search_view("shortlist")` accepts the new view, the extracted sort helper returns `SortField::DEFAULT` (`ca` descending) when `sort_by` is absent for Shortlist, accepts explicit General sorts (including representative `pa` and current `role.*`/`potential_role.*`/`club_dna`/`attr.*`), and rejects Moneyball-only `moneyball.goals`/`moneyball_role.wbl_wbr_wing_back_ip` with a typed error. `query.rs` then validates typed `SortField`, `filters`, and `requested_fields` through the same General path as `view=general`; invalid Moneyball-only fields return a typed error rather than producing a misaligned SQL expression.
- The SQL `FROM` clause adds `INNER JOIN player_moneyball_stats shortlist ON shortlist.snapshot_id = players.snapshot_id AND shortlist.player_uid = players.uid` scoped to `players.snapshot_id = ?1` (the active save's `is_current = 1` snapshot). No `percentiles_json IS NOT NULL` condition is added for Shortlist.
- `WHERE` compilation appends any General `CompiledFilter` SQL with the same parameter indexing as General. Sort and dynamic-value `SELECT` expressions use `MetricField::sql_expression` / `sql_expression_with_club_dna` without Moneyball context.
- A snapshot with no Moneyball rows returns `total = 0` and no pagination rows. A snapshot with cohort rows returns only those rows, preserving committed/requested query semantics in the frontend.

**Ordered implementation steps:**

1. RED: add `src-tauri/src/features/search/commands.rs` unit tests for the extracted pure sort parser: raw `search_view="shortlist"` parses, `sort_by=None` for Shortlist resolves to `SortField::DEFAULT` (`ca` descending), explicit General sorts including representative `pa` and current `role.deep_lying_playmaker_ip` are accepted, and `moneyball.goals`/`moneyball_role.wbl_wbr_wing_back_ip` are rejected with the exact General-validation error. Current code fails because `shortlist` is not a known view.
2. RED: add a temporary-SQLite integration test `search_shortlist_returns_only_current_moneyball_members_with_general_metrics` in `query.rs` that ingests one snapshot with four players, adds Moneyball rows for two current players, then calls `search_players_in_view` with `view = Shortlist`, a typed General sort (`ca` desc), and General requested fields. Current code fails because `shortlist` is not a known view.
3. RED: extend the success test to add a third Moneyball row with `percentiles_json = NULL` and assert Shortlist still returns that member while Moneyball `IS NOT NULL` would exclude it.
4. RED: add negative cases for Moneyball-only `filters`/`requested_fields`/`sort_by` (`moneyball.goals`, `moneyball_role.wbl_wbr_wing_back_ip`) against `view = Shortlist` at both layers: command-boundary raw-string sort rejection in `commands.rs` and typed `SortField`/`requested_fields`/`filters` rejection in `query.rs`, both expecting the exact General-validation error string.
5. GREEN: add `SearchView::Shortlist` and `parse_search_view("shortlist")` plus the extracted sort helper used by `search_players` so Shortlist defaults to `ca` descending and maps explicit General sorts correctly.
6. GREEN: branch the query path so Shortlist uses the General resolver/compile path and the membership inner join without percentile readiness; keep Moneyball's `percentiles_json IS NOT NULL` join untouched.
7. GREEN: handle `comparisonPool` scoping: add Shortlist to the General-family default branch so `parse_comparison_pool` still accepts closed values for any view; Shortlist query ignores the pool (Moneyball-only behavior branch). Do not add a rejection for Shortlist.
8. REFACTOR: normalize any duplicated join string or validation helper without changing observable semantics.
9. Run targeted Rust validation then the full commit gate.

**Tests and proof:**

- `src-tauri/src/features/search/commands.rs` unit tests (pure helper, no DB) — RED then GREEN: (a) `parse_search_view(Some("shortlist"))` parses; (b) `sort_by=None` for Shortlist resolves to `SortField::DEFAULT` (`ca` descending) via the helper; (c) accepted explicit General sort including representative `pa` and current `role.deep_lying_playmaker_ip` parses through the General path; (d) `moneyball.goals` and `moneyball_role.wbl_wbr_wing_back_ip` are rejected with the exact General-validation error string. Proves raw-string command-boundary behavior before any `query.rs` call.
- `src-tauri/src/features/search/query.rs` integration tests (temporary database via `db::migrations::apply` and `features/snapshot/ingest` helpers):
  - RED then GREEN: (a) cohort membership including null-percentiles row with General filtering/sorting: ingested snapshot with four players, Moneyball rows for UIDs 1 and 3 plus a third row with `percentiles_json = NULL` for UID 2 → Shortlist returns exactly those three members; General returns 4; filter `role.deep_lying_playmaker_ip > 50` and typed sort `ca` desc compose via General resolvers; `requested_fields = ["role.deep_lying_playmaker_ip", "attr.Acceleration", "ca"]` round-trips. Prevents Shortlist collapsing to General and percentile-gated invisibility.
  - (b) active/current snapshot isolation and empty cohort: second save/snapshot with its own cohort does not leak into the active shortlist; snapshot with no Moneyball rows returns `total = 0`. Prevents cross-snapshot membership.
  - (c) closed validation rejects representative Moneyball-only typed inputs at the query layer: `filters`/`requested_fields`/typed `sort_by` containing `moneyball.goals` or `moneyball_role.wbl_wbr_wing_back_ip` against `view = Shortlist` returns the exact General-validation error string. Prevents Moneyball score leakage through the SQL path. Do not claim these query tests prove raw `sort_by` string parsing — that is owned by `commands.rs` above.
- Retain existing General and Moneyball search tests, comparator direction tests, and filter registry unit tests. Do not duplicate exhaustive operator coverage already proved for General. Do not duplicate migration, cumulative-import cascade, or import-persistence tests — those remain the owner of FK-cascade and import lifecycle proof.
- No snapshots, fixtures, or mocks beyond the existing inline CSV/statistics helpers and temporary-DB harness.

**Patterns to verify:**

- `SearchView::General` vs `SearchView::Moneyball` branching at `src-tauri/src/features/search/query.rs:360` and the `percentiles_json IS NOT NULL` join at `~query.rs:501` for Moneyball scope isolation.
- `commands.rs:parse_search_view` / `parse_comparison_pool` closed-input validation.
- `player_metrics::resolver::is_moneyball_search_field` and `MetricField::parse_for_moneyball` for Moneyball rejections in General-family views.
- `player_metrics::compact::assert_read_models_complete` completeness gate without read-time mutation.

**Constraints and non-goals:**

- Do not add a migration, table, column, index, background job, or second command.
- Do not duplicate Moneyball percentile `statistics_json` plumbing for Shortlist.
- Do not gate Shortlist membership on `percentiles_json`.
- Do not allow `moneyball.*` or `moneyball_role.*` fields in Shortlist requests.
- Preserve existing General and Moneyball behavior, limits (50 default / 200 max), and error strings.

**Dependencies and sequencing:**

- Depends on Commit 1. Establishes the authoritative backend contract consumed by the frontend tab and empty-state work.

**Validation:** `./scripts/dev check-rust`, then `./scripts/dev check`.

**Stop conditions:** Stop and replan if Shortlist cannot validate through the General resolver without a schema migration; if cohort membership cannot be expressed as a read-only inner join scoped to the effective current snapshot inside the existing query path; if snapshot cascade semantics diverge from Moneyball storage; or if implementation would require a new IPC DTO, dual-query merging, or read-time writes.

**Review mandate:**

- Trace the Shortlist SQL string for exact `INNER JOIN player_moneyball_stats` scoping, parameter index correctness, and absence of `percentiles_json IS NOT NULL`.
- Verify every Shortlist error path for Moneyball-only fields returns the General-validation error without interpolating raw client strings.
- Verify General path still rejects unknown fields and caps `MAX_REQUESTED_FIELDS` and `MAX_FILTER_RULES`.
- Verify completeness validation runs before any dynamic value read.
- Verify no new branch leaks Moneyball percentiles or role-score calculation into Shortlist.
- Verify snapshot isolation uses the active save's `is_current = 1` snapshot consistently.
- Verify Moneyball views retain their existing percentile-gated join.
- Verify the change is readable as one revertible query contract in review.

#### Commit 3 — Wire Shortlist tab, layout, empty state, and profile mapping

**Status:** Pending

**Provisional commit:** `feat(search): wire Shortlist tab and table layout`

**Work:** Expose Shortlist as the third Search tab, give it an independent persisted table layout with migrated routing for existing stored preferences, render the Shortlist-specific empty state, and ensure profile navigation maps Shortlist to General.

**Size assessment:** Estimated 140–220 changed non-test implementation lines across existing React store, route, and panel composition. Slightly above the soft target is acceptable if the tab, layout, empty-state, and navigation ownership form one coherent user-visible integration.

**Out of scope:**

- Rust query or schema changes, Moneyball import duplication in Shortlist, Squad/Staff changes, and new ADRs.

**Implementation packet:**

- Reuse `SquadCsvImportModal` and `useCsvImport` ownership unchanged; only Shortlist presentation and route state change.

**Files and responsibilities:**

- `src/features/search/types/search-view.ts` — extend `SearchView` to `"general" | "moneyball" | "shortlist"`, update `parseSearchView`, `isSearchView`-style guards, and `defaultSearchSort` so Shortlist returns `"ca"`.
- `src/features/search/types/search-sort.ts` — no conceptual change; confirm `defaultDirForSortField` already covers General columns used by Shortlist.
- `src/features/search/utils/filter-registry.ts` — route Shortlist through General catalog (`PLAYER_METRICS`) via `filterFieldsForView` / `getFilterField`.
- `src/features/search/utils/dynamic-columns.ts` — treat Shortlist as a General-family view: extend `isVisibleSortField` so `ca`/`pa` remain visible for Shortlist (they are currently gated to `view === "general"`) and `dynamicColumnFields`/`isVisibleSortField` otherwise dispatch Shortlist through the same General branch as `general` (visible General sorts including `ca`/`pa` and dynamic `role.*`/`potential_role.*`/`club_dna`/`attr.*`/`pos.*`/`position`, rejected Moneyball sorts `moneyball.*`/`moneyball_role.*`).
- `src/features/search/utils/search-url-search.ts` — retain `parseOneFilterRule` / `parseSearchFilters` dropping invalid filters on direct URL loads; switching tabs clears all filters and resets to the destination default sort/direction so no filter preservation is added.
- `src/app/routes/search.tsx` — expand `validateSearch` to accept `shortlist` (still dropping invalid URL filters via `parseSearchFilters`), derive `view` from URL or `useMoneyballPreferences` without adding Shortlist as a default, keep `comparisonPool` in URL state only when `view === "moneyball"` (no backend rejection), render the three-tab `tablist` in **General, Moneyball, Shortlist** order with ArrowLeft/Right/Home/End where ArrowRight from General lands on Moneyball and End lands on Shortlist, focus management, and per-tab `aria-selected`/`tabIndex`, clear all filters and reset to `defaultSearchSort(nextView)`/`defaultDirForSortField` on any tab change, explicitly resolve all three layout IDs on filter apply (`general` → `search`, `moneyball` → `moneyball-search`, `shortlist` → `shortlist`) so a Shortlist filter-added column mutates only `layouts.shortlist` and leaves `search`/`moneyball-search` unchanged (current code maps only Moneyball vs search), keep `SquadCsvImportModal` behind `view === "moneyball"` only, preserve `searchKeys.all` invalidation after import, and ensure `SearchPageContent` does not render stale Shortlist feedback from a prior save/snapshot generation.
- `src/features/search/components/search-results-panel.tsx` — map `view === "shortlist"` to `tableId = "shortlist"`, use `getPlayerMetric` for column resolution (no Moneyball metrics), wire `SearchResultsVirtualTable` for Shortlist paging, render presentation-distinguished empty states solely from existing `total` and `filters.length`: when `total === 0 && filters.length === 0` show the neutral “No shortlist yet — upload a Moneyball CSV in the Moneyball tab” state with a Moneyball-tab action, when `total === 0 && filters.length > 0` show the standard “No players match these filters” state regardless of whether the underlying cohort is empty (clearing filters then reveals the neutral guidance if the cohort is empty; a direct URL with a valid General filter follows this same rule), keep `onPlayerActivate` mapping `shortlist` to `general` before `navigate({ to: "/players/$uid", search: { view } })`. No cohort-existence signal, companion query, or DTO field is added.
- `src/stores/use-player-table-store.ts` — bump `PLAYER_TABLE_LAYOUT_VERSION` to `6`, extend `PlayerTableId` with `"shortlist"`, add `shortlist` to `defaultLayout`/`defaultPlayerTableLayouts` using `withoutDuplicateIdentityColumns(DEFAULT_PLAYER_TABLE_COLUMN_IDS)`, extend `sanitizeLayout`/`sanitizePersistedState`/`migratePersistedState` so version `5` state is upgraded without mutating stored `search` or `moneyball-search` layouts; keep existing Staff and Squad branches untouched.
- `src/features/search/utils/dynamic-columns.test.ts` — add Shortlist coverage: `isVisibleSortField("ca"/"pa", …, "shortlist")` and representative dynamic General field (`role.*`/`attr.*`/`club_dna`) remain visible, `moneyball.*`/`moneyball_role.*` remain rejected, `dynamicColumnFields` for Shortlist mirrors General.
- `src/testing/search-ipc-mock.ts` — deliberately retained unchanged because its `parsePaging`/`resolveSearchPlayersIpcMock` already discriminates by `filters`/`sortBy`/`requestedFields` without branching on `searchView`; no Shortlist-specific resolver change is needed and adding one would create an unowned view-specific mock seam. Keep existing `pendingReplacement` style modes compatible with the third tab for focused route tests without view-specific branching.
- `e2e/tauri-ipc-stub.ts` — extend the stub to return a Shortlist cohort by `searchView` (General, Moneyball, Shortlist) without requiring a stubbed upload to mutate cohort unless the existing seam already does so trivially. `e2e/smoke.spec.ts` — extend smoke as presentation evidence to prove third-tab rendering (General, Moneyball, Shortlist), no duplicate upload in Shortlist, keyboard navigation (ArrowRight General→Moneyball, End→Shortlist), and profile mapping, without asserting import persistence.

**Behavior and data flow:**

- `validateSearch` accepts `general` | `moneyball` | `shortlist`; an absent or invalid `view` still falls back to `useMoneyballPreferences.defaultAnalysisView` (General or Moneyball only). When `view !== "moneyball"` the resulting URL state has `comparisonPool = undefined` irrespective of any stale URL value (backend `parse_comparison_pool` still accepts closed values for any view; only Moneyball query uses it).
- Tab activation calls `updateSearch({ view: next, comparisonPool: next === "moneyball" ? "filtered" : undefined, sort: defaultSearchSort(next), dir: defaultDirForSortField(defaultSearchSort(next)), filters: [] })` clearing all filters and resetting to the destination default sort/direction as `SearchPageContent` does now. The existing committed/requested observer pattern retains sort-replacement semantics for in-view sort changes; tab switches are full filter/sort replacements.
- `SearchFilterBar` `onApply` explicitly resolves layout ID by view (`general` → `search`, `moneyball` → `moneyball-search`, `shortlist` → `shortlist`) and calls `addColumns(layoutId, rules.map(r => r.field))` so a Shortlist filter adds its field only to `layouts.shortlist` and leaves `search`/`moneyball-search` unchanged. `isVisibleSortField` and `dynamicColumnFields` treat Shortlist as General-family so `ca`/`pa` and dynamic General fields remain sortable/columnizable and Moneyball-only fields remain rejected.
- `SearchResultsPanel` resolves `tableId` → layout, `requestedFields` from visible columns, and `searchPlayersQueryOptions` with `searchView = view` directly. With `total === 0 && filters.length === 0` the panel replaces the virtual table with the Shortlist-specific neutral empty state (“No shortlist yet”) containing a keyboard-focusable control that navigates to `view=moneyball` (no implicit upload). With `total === 0 && filters.length > 0` it replaces with the existing “No players match these filters” empty state regardless of whether the underlying cohort is empty — clearing filters then reveals the neutral guidance if the cohort is empty, and a direct URL carrying a valid General filter follows this same precedence. With rows it renders the shared full-height virtual table. The distinction uses only existing `total` and `filters.length`; no cohort-existence signal, companion unfiltered query, or DTO field is introduced.
- `usePlayerTableStore` persists the Shortlist layout independently; changing it does not fire layout effects for other tables.
- Row activation for Shortlist calls `navigate({ to: "/players/$uid", params: { uid: String(player.uid) }, search: { view: "general" } })`. General and Moneyball rows keep their existing `view` passthrough.

**Ordered implementation steps:**

1. RED: extend `src/app/routes/search.test.tsx` to assert `view=shortlist` URL validation accepts `view=shortlist`, drops invalid `moneyball.*`/`moneyball_role.wbl_wbr_wing_back_ip` filters on direct URL via `parseSearchFilters`, clearing all filters and resetting to `defaultSearchSort`/`defaultDirForSortField` on any tab switch, and keeps `comparisonPool` out of URL when `view !== "moneyball"`.
2. RED: extend `src/stores/use-player-table-store.test.ts` to prove version `5` persisted state migrates to version `6` with a fresh `shortlist` layout and preserved `search`/`moneyball-search` columnIds/widths.
3. RED: extend `src/app/routes/search.test.tsx` (panel empty/profile/tab/query behavior) to assert both Shortlist zero-result presentation branches solely from `total` and `filters.length`: (a) `total === 0 && filters.length === 0` shows the neutral “No shortlist yet” empty state with a Moneyball-tab action; (b) `total === 0 && filters.length > 0` shows the standard “No players match these filters” state regardless of cohort emptiness (including a direct URL that carries a valid General filter such as `role.*` or `attr.*` that yields `total === 0` — clearing filters then reveals the neutral guidance if the stubbed cohort is empty); (c) Shortlist row activation navigates with `view=general`; (d) Moneyball upload button remains absent in Shortlist; (e) three tabs render in **General, Moneyball, Shortlist** order with ArrowRight General→Moneyball and End→Shortlist keyboard behavior; (f) applying a Shortlist filter via `SearchFilterBar` `onApply` adds its field only to `layouts.shortlist` and leaves `layouts.search`/`layouts.moneyball-search` unchanged (proof for the three-layout `addColumns` fix); (g) `src/features/search/utils/dynamic-columns.test.ts` Shortlist sort visibility: `isVisibleSortField("ca", …, "shortlist")`/`"pa"` and representative dynamic General fields remain true while `moneyball.*`/`moneyball_role.*` remain false.
4. RED: extend Playwright smoke as presentation evidence so `page.goto("/search?view=shortlist")` with stubbed `searchView`-aware cohort shows the neutral “No shortlist yet” state for `total === 0 && filters.length === 0`, shows the standard “No players match these filters” state for `total === 0 && filters.length > 0` (including a direct URL with a valid General filter), and shows table rows when the stub returns Shortlist members; prove no duplicate upload and prove ArrowRight from General lands on Moneyball and End lands on Shortlist with keyboard focus (stub returns cohort by `searchView` without requiring a stubbed upload to mutate or a companion unfiltered query).
5. GREEN: implement the three-tab type/route changes, clearing all filters and resetting to the destination default sort/direction on any tab switch, and focus/aria handling.
6. GREEN: fix `src/features/search/utils/dynamic-columns.ts` so Shortlist is General-family for `isVisibleSortField`/`dynamicColumnFields` (including `ca`/`pa` visibility).
7. GREEN: add the `shortlist` table layout with version `6` migration, explicitly resolve three layout IDs on filter apply (`general` → `search`, `moneyball` → `moneyball-search`, `shortlist` → `shortlist`), and add the Shortlist-specific panel branching.
8. GREEN: guard the profile mapping and keep the upload action behind the Moneyball tab.
9. REFACTOR: consolidate any duplicated `defaultSearchSort` or `filterFieldsForView` branching without changing URL or layout keys.
10. Run focused route/store/panel/dynamic-columns tests, the frontend/full gate, and smoke in the recorded order.

**Tests and proof:**

- Modify or extend focused frontend coverage:
  - Route validation test (`src/app/routes/search.test.tsx`) — prove `view=shortlist` is accepted, direct URL with `moneyball.goals`/`moneyball_role.wbl_wbr_wing_back_ip` filters is dropped via `parseSearchFilters`, `comparisonPool` is stripped when `view !== "moneyball"`, and selecting any view clears all filters and resets to `defaultSearchSort`/`defaultDirForSortField`. Plausible regression is a Moneyball filter surviving into Shortlist and producing a 500 or empty query.
  - Store migration test (`src/stores/use-player-table-store.test.ts` or inline store harness) — seed `localStorage["fm-valuescout-player-table-layouts"]` with a `version: 5` payload containing custom `search` and `moneyball-search` widths, then assert the hydrated store versions to `6`, creates `layouts.shortlist` from `withoutDuplicateIdentityColumns(DEFAULT_PLAYER_TABLE_COLUMN_IDS)`, and preserves both prior layouts byte-for-byte.
  - Layout-isolation test (`src/app/routes/search.test.tsx` with `usePlayerTableStore`) — seed `search`/`moneyball-search` with known columns, trigger `SearchFilterBar` `onApply` with a Shortlist filter (e.g., `role.deep_lying_playmaker_ip` or `attr.Acceleration`) while `view === "shortlist"`, then assert the field appears only in `layouts.shortlist.columnIds` and `layouts.search`/`layouts.moneyball-search` are unchanged. Prevents Shortlist `addColumns` from mutating General's layout.
  - Dynamic-columns test (`src/features/search/utils/dynamic-columns.test.ts`) — prove Shortlist as General-family: `isVisibleSortField("ca", …, "shortlist")` and `isVisibleSortField("pa", …, "shortlist")` are true (they are currently false outside General), representative dynamic General sorts (`role.deep_lying_playmaker_ip`, `attr.Acceleration`, `club_dna`, `potential_role.goalkeeper_ip`) remain true, and Moneyball-only sorts (`moneyball.goals`, `moneyball_role.wbl_wbr_wing_back_ip`) remain false; `dynamicColumnFields` for representative filters mirrors General.
  - Panel empty/profile test (`src/app/routes/search.test.tsx` covering panel empty/profile/tab/query behavior via `mockIPC`) — prove both Shortlist zero-result presentation branches solely from `total` and `filters.length`: stub `search_players` to return `total: 0` with `filters.length === 0` and assert the neutral “No shortlist yet” heading and Moneyball-tab control; stub `total: 0` with `filters.length > 0` (valid General filter such as `attr.Acceleration` or `role.deep_lying_playmaker_ip`) and assert the standard “No players match these filters” state regardless of cohort emptiness (including the direct-URL case where a valid General filter yields `total === 0`); stub `total: 5` for Shortlist and assert the virtual table caption, columns, and row activation. Mock `useNavigate` to prove Shortlist activation passes `view: "general"` and does not pass `"shortlist"`. No cohort-existence query or DTO field is introduced.
  - Tab keyboard test (`src/app/routes/search.test.tsx` harness) — with focus on the General tab, dispatch `ArrowRight` (lands on Moneyball), `ArrowRight` again (lands on Shortlist), `End` (Shortlist), `Home` (General), `ArrowLeft` and assert `tabRefs` focus movement and `aria-selected` rotation across three tabs in **General, Moneyball, Shortlist** order. Prevents two-tab logic from wrapping incorrectly.
  - Smoke proof (`e2e/smoke.spec.ts` with `e2e/tauri-ipc-stub.ts` returning Shortlist cohort by `searchView`) — prove as presentation evidence both zero-result branches solely from `total`/`filters.length`: `/search?view=shortlist` with `total === 0 && filters.length === 0` shows the neutral “No shortlist yet” state, `/search?view=shortlist` with a valid General filter and `total === 0` (e.g., `/search?view=shortlist&filters=...`) shows the standard “No players match these filters” state regardless of cohort emptiness; the Upload button is absent in both Shortlist empty states; keyboard navigation exposes three tabs in General, Moneyball, Shortlist order with ArrowRight General→Moneyball and End→Shortlist; with stubbed Shortlist cohort the table shows rows without duplication; no stubbed upload mutation, companion query, or DTO extension is required.
- Retain `src/testing/search-ipc-mock.ts` General/Moneyball fixtures unchanged; do not add view-specific mock branching — its resolver is deliberately retained without a `shortlist` view discriminant because it discriminates only by `filters`/`sortBy`/`requestedFields` (see Files and responsibilities). Remove only replacement-specific assertions made obsolete by the three-tab model; retain picker cancellation, pending lock, context generation, path-redaction, and keyboard-close coverage.
- No snapshots or heavy fixtures. Do not add a second mock protocol or a second IPC command.

**Patterns to verify:**

- Existing two-tab keyboard/ARIA pattern in `src/app/routes/search.tsx` tablist (`role="tablist"`, `role="tab"`, `aria-selected`, `tabIndex`, `ArrowRight/Left/Home/End`).
- Existing route-owned `searchKeys.all` invalidation after `SquadCsvImportModal` `onMoneyballImported`.
- Existing `usePlayerTableStore` `defaultLayout` / `sanitizeLayout` / `migratePersistedState` versioning at `src/stores/use-player-table-store.ts:14`.
- Existing `SearchResultsPanel` committed/requested observer and `VirtualizedPlayerTable` paging contract.

**Constraints and non-goals:**

- Do not create a new IPC command, table, or Tauri capability.
- Do not show Moneyball metrics or `moneyball_role.*` fields in Shortlist filters, columns, or sort menus.
- Do not expose an upload control in Shortlist or make Shortlist a Settings default.
- Do not interpolate raw client strings into SQL or store unsanitized layout values.
- Keep the main table scroller as the single scroll owner and preserve horizontal overflow behavior at 1280×800 and 1600×900.

**Dependencies and sequencing:**

- Depends on Commit 2 so the Shortlist view has a backed query path before frontend wiring claims membership. This is the final implementation commit and moves the feature to Validation after checkpoint completion.

**Validation:** `./scripts/dev test src/app/routes/search.test.tsx src/stores/use-player-table-store.test.ts src/features/search/utils/dynamic-columns.test.ts`, then `./scripts/dev check`, then `./scripts/dev smoke`.

**Stop conditions:** Stop and replan if Shortlist cannot own an independent layout without touching existing persisted user preferences; if clearing all filters and resetting to the destination default sort/direction on tab switch cannot be performed without leaving stale URL state; if Shortlist cannot be made General-family for `isVisibleSortField`/`dynamicColumnFields` without regressing General or Moneyball sort visibility; if applying a Shortlist filter cannot be isolated to `layouts.shortlist` without mutating `search`/`moneyball-search`; if profile navigation would need a new profile view variant; or if Shortlist presentation would require a second IPC path.

**Review mandate:**

- Verify tab count is three and keyboard/ARIA behavior covers ArrowLeft/Right/Home/End without focus loss.
- Verify selecting any view clears all filters and resets to the destination default sort/direction, and `comparisonPool` is exposed in URL only for Moneyball (backend accepts closed values for any view but only Moneyball query uses it).
- Verify Shortlist requests are validated against General resolvers and rejected for any Moneyball-only field before SQL construction, and that `isVisibleSortField` treats Shortlist as General-family so `ca`/`pa` and dynamic General sorts (`role.*`, `potential_role.*`, `club_dna`, `attr.*`) remain visible while `moneyball.*`/`moneyball_role.*` remain rejected.
- Verify the Shortlist layout migrates to version `6` without mutating `search` or `moneyball-search` persisted widths or order.
- Verify applying a Shortlist filter via `SearchFilterBar` `onApply` adds its field only to `layouts.shortlist` and leaves `search`/`moneyball-search` unchanged (three layout IDs explicitly resolved).
- Verify Shortlist empty-state presentation uses only `total` and `filters.length`: `total === 0 && filters.length === 0` shows the neutral “No shortlist yet” guidance/action to Moneyball, `total === 0 && filters.length > 0` shows the standard “No players match these filters” state regardless of cohort emptiness (clearing filters then reveals the neutral guidance if the cohort is empty; direct URL with a valid General filter follows this same rule), and never renders the upload control inside Shortlist; no cohort-existence query or DTO field is introduced.
- Verify Shortlist row activation maps to `view=general` before navigation.
- Verify no Moneyball columns or import controls leak into Shortlist `SearchResultsPanel` rendering.
- Verify existing smoke and route tests remain presentation-scoped and do not imply native Tauri or SQLite proof.

## Active work

**PR:** PR 1 — Add Player Shortlist tab to Player Search

**Commit:** Add Shortlist view and cohort-filtered General query

### RED or removal proof

Add command-boundary tests that fail because `shortlist` is not a parsed view, then add temporary-SQLite query tests that fail because `SearchView::Shortlist` and its cohort-restricted General query path do not exist.

### Expected outcome

Rust accepts `search_view = "shortlist"`, validates it through General sort/filter/requested-field rules, and returns only current-snapshot Moneyball cohort members through a read-only join that does not depend on `percentiles_json`.

### Explicit exclusions

- Frontend tabs, table layout, empty-state presentation, new persistence or migrations, new IPC commands or DTOs, read-time writes, and Moneyball statistics or role scores in Shortlist.

## Discoveries and replanning

- Planning confirmed that `src-tauri/src/features/search/query.rs:SearchView` and `src-tauri/src/features/search/commands.rs:parse_search_view` are the single seams for a third view, and that the closed resolver contract in `src-tauri/src/features/player_metrics/resolver.rs` already rejects Moneyball-only inputs when called with `moneyball = false`.
- Planning confirmed that `player_moneyball_stats` already owns one row per `(snapshot_id, player_uid)` with FK cascades, so Shortlist needs no migration and its cohort follows the cumulative import lifecycle: deletion cascades rows only for the deleted snapshot while the visible cohort follows whichever snapshot becomes current after selection/promotion; a newly current snapshot with no own Moneyball rows has no shortlist.
- Planning confirmed that Moneyball-only reads filter on `percentiles_json IS NOT NULL` while Shortlist membership must omit that predicate. The Moneyball predicate stays scoped to Moneyball paths only.
- Planning selected one PR because the Rust join change and the three-tab/layout/empty-state integration share one acceptance surface; no independent foundation or separately mergeable artifact justifies a second PR. The provisional branch is `feature/player-shortlist` from `main` with GitHub squash publication, strict `check`, and `Not run` close-out until the final feature PR.
- Linear JAY-52 manual shortlist language was explicitly superseded by automatic current Moneyball-cohort membership. This ledger records that replacement as a decision.
- Correction 2026-09-02 (HIGH, first planning-artifact commit review): `SearchPlayersPage` exposes only filtered `total`, so `total === 0 && filters.length > 0` cannot distinguish an empty cohort from a present cohort with zero matches; no cohort-existence signal, companion unfiltered query, or DTO field is planned. Corrected ledger to define presentation precedence solely from existing inputs (`total` and `filters.length`) — unfiltered `total === 0` (`filters.length === 0`) → neutral “No shortlist yet” guidance/action to Moneyball, any filtered `total === 0` (`filters.length > 0`) → standard “No players match these filters” regardless of cohort emptiness (clearing filters reveals neutral if cohort empty; direct URL with valid General filter follows same rule). Reconciled every contradictory statement in Intent/User-visible behavior, Feature architecture (`SearchResultsPanel`), Commit 3 files/data flow/steps/tests/review mandate, smoke claims, and Final validation smoke claims. No cohort-existence query, extra IPC field/shape, companion request, persistence, or dual-query behavior added — presentation distinction only. Left `Delivery fingerprint` as `pending-review` because packet changes invalidate the prior delivery grant and require a new reviewed fingerprint.
- Correction 2026-09-02 (MEDIUM, second bounded review): removed invalid `## Abandonment record` Active-ledger section (schema 2 permits it only after explicit abandonment); plan review found `ledger_state.py` did not reject this invalid section, so schema-template inspection remains required.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Add Player Shortlist tab to Player Search | Commit 1 — Record the approved feature plan | Pending record | Recorded the reviewed schema 2 ledger and activated the feature in TODO without changing executable behavior. | `ledger_state.py`: runnable; `delivery_state.py`: runnable; documentation links and `git diff --cached --check`: passed. | Not applicable | Clear | 0 | None. |

## Final validation

Required automated evidence before feature review:

- `./scripts/dev test`
- `./scripts/dev check-rust`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Rust integration proof that Shortlist returns only current-snapshot `player_moneyball_stats` members with General metrics, is independent of `percentiles_json`, rejects any `moneyball.*` or `moneyball_role.*` requested field/filter/sort (with raw `search_view`/`sort_by` string parsing proved at the `commands.rs` boundary and typed `SortField`/`requested_fields`/`filters` behavior proved in `query.rs`), and preserves snapshot isolation and deletion behavior.
- Frontend proof that the three-tab set is keyboard-operable, selecting any view clears all filters and resets to that destination's default sort/direction, the `shortlist` layout migrates to version `6` without touching existing layouts, Shortlist filter apply mutates only `layouts.shortlist`, Shortlist as General-family keeps `ca`/`pa` and dynamic General sorts visible while rejecting Moneyball-only sorts, Shortlist empty-state presentation uses only `total` and `filters.length` — `total === 0 && filters.length === 0` shows the neutral “No shortlist yet” guidance/action to Moneyball and `total === 0 && filters.length > 0` shows the standard “No players match these filters” state regardless of cohort emptiness (clearing filters reveals the neutral guidance if the cohort is empty; direct URL with a valid General filter follows this same rule; no cohort-existence query or DTO field), and row activation maps to General before profile navigation.

Manual/native evidence target:

- On a supported Windows Tauri build at 1280×800 and 1600×900, import a Moneyball CSV from Moneyball, switch to Shortlist, confirm General columns, `CA` desc, attribute/role filter composition, and bounded virtual paging. Confirm an unfiltered Shortlist with `total === 0` shows the neutral “No shortlist yet” guidance to the Moneyball tab, a filtered Shortlist with `total === 0` (including a direct URL carrying a valid General filter) shows the standard “No players match these filters” state regardless of cohort emptiness and that clearing filters then reveals the neutral guidance if the cohort is empty, and that Moneyball-only filters do not persist into Shortlist after tab switches. Confirm a Shortlist row click opens the profile in General analysis view. Confirm imported and omitted players still follow the cumulative cohort after an overlapping import by checking Shortlist row presence, not by inspecting Moneyball percentiles.
- If Windows Tauri or representative-export verification is unavailable, record that exact manual validation gap. Stub IPC smoke and temporary-DB Rust tests do not prove native WebView focus, packaged IPC, or real-file persistence for this feature.

`./scripts/dev bridge-test` is not planned because bridge source and file protocol are unchanged. Run it and replan if implementation crosses that boundary. `./scripts/dev mutate` remains unsupported and must not be reported as passed.

## Documentation impact

Reconciliation complete before final publication only. `ARCHITECTURE.md` and `DESIGN.md` will be updated to describe the third Search tab in **General, Moneyball, Shortlist** order, Shortlist membership as current-snapshot Moneyball cohort filtering on General read models, independent `shortlist` layout ownership (`shortlist`, version 5→6), and the neutral empty-state contract. `CONCEPT.md` will be assessed and updated to include the implemented Player Shortlist beside current General/Moneyball Search wording. No ADR is warranted: the feature extends the existing `SearchView` seam and the established player-table layout pattern without a competing durable alternative.
