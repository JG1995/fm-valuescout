# Player Table Sort Performance

## Status

Ready for final publication

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** fdf0376491139cf1da0cf2e84d5fa5a0d0921dab9c10f7583f5ad2f304e1ce79

## Intent

Keep Search and Squad player tables visible and truthful during sort-only replacement, and make the common scalar and persisted-score sorts acceptably responsive on a representative approximately 250,000-player save without changing authoritative ordering or bounded data ownership.

## User-visible behavior

- Changing only the sort in Search or Squad keeps the last successful table visible until the replacement succeeds.
- The visible header, summary, rows, total, and activation identity always describe one committed request. A requested header never appears over retained rows.
- Scalar, current-role, and Club DNA replacements say that they are sorting. A potential-role replacement says that it is calculating and sorting because a cold request can populate the lazy cache.
- Retained rows stay readable but cannot activate, receive pending keyboard focus, or navigate until the requested result commits.
- A failed sort replacement keeps the committed table and offers retry. A first request with no committed result shows truthful loading, then the table or an initial error with retry.
- A filter, combine, view, comparison-pool, visible-column projection, active-save, current-snapshot, or managed-club change clears or unmounts the prior result table. Rows are retained only for sort replacement inside the same mounted context.
- Before an app-owned Load Data, active-save switch or deletion, current-snapshot deletion or resulting promotion, or managed-club change invokes Tauri, the app cancels and then removes exactly `searchKeys.playerPages()` and `squadKeys.playerPages()` and blocks their result controllers. All Search and Squad player pages are removed; Search suggestions and unrelated Planner queries remain. TanStack Query ignores a canceled command's late result.
- Search and Squad preserve both directions, deterministic UID ties, bounded virtual pages, URL-owned sort state, and exact active-save/current-snapshot behavior.
- Search Name and CA remain performance controls. Search PA, Age, Value, current role, warm potential role, and Club DNA plus Squad CA over the configured managed-club cohort, current role, warm potential role, and Club DNA are the targeted paths.
- Manual performance validation is pass/fail by developer judgment on a representative approximately 250,000-player save. The handoff or PR checklist records only `Pass` or `Fail`.

## Invariants

- React remains presentation-focused. Rust and SQLite own cohort selection, global ordering, score semantics, and migrations.
- IPC returns bounded pages. The feature never sends the complete player set to React or a Web Worker and never sorts the complete set on the client.
- TanStack Query owns IPC result data. URL search parameters own Search and Squad sort state. Local React state stores only the committed request descriptor, never rows or totals.
- Each result controller uses committed and requested first-page observers. Identical keys deduplicate through TanStack Query.
- Route loaders do not start Search or Squad result queries. Virtual page zero uses the exact committed first-page options and deduplicates against the committed observer.
- Retained rows and their header use one committed descriptor. Only success for the latest requested descriptor can promote it. Completion of a superseded sort cannot commit over a newer sort.
- Search may use the existing active-save `{ id, contextToken }` and current snapshot ID/save ID to discriminate a mounted context. Squad may also use existing managed-club club/status. These values stay frontend-only when used in result keys.
- Supported context changes are app-composed. App-layer owners inject one async pre-mutation callback from `src/app/player-result-context.ts`; feature mutations await it before Tauri, and result controllers remain blocked until context owners settle. Feature modules do not import sibling feature keys or the app coordinator, and shared modules do not import features. App modules may retain existing cross-feature composition, but only `src/app/player-result-context.ts` composes both `searchKeys.playerPages()` and `squadKeys.playerPages()` roots for the new cancellation/removal operation.
- A canceled Tauri command may finish. TanStack Query must ignore its late fulfillment. If focused tests disprove this behavior, stop and replan to explicit request/response generation binding.
- Context changes clear rows. Sort-only transitions inside the same mounted context retain committed rows.
- Retained rows cannot activate, receive pending keyboard focus, or navigate while replacement is unresolved.
- SQLite queries preserve null and missing behavior, both directions, deterministic `uid ASC` ties, page totals, and complete cohorts before global potential ordering.
- Search reads only the active save's effective current snapshot. Squad reads only that snapshot's exact configured managed-club cohort.
- The one `rusqlite` driver and `Db(Mutex<Connection>)` remain. No pool or second database driver is added.
- ADR-0019 remains authoritative for the sparse, versioned, nullable, lazy, disposable potential-role cache. Cold global potential sorting completes its cohort before ordering.
- ADR-0024 remains authoritative for eager persisted Club DNA rows and Club DNA null-last ordering.
- Seven targeted player indexes are the complete migration scope: directional PA, Age, and Value indexes plus managed-club membership. Name and CA indexes remain unchanged.
- Automated tests own correctness. Performance judgment is manual and does not enter `./scripts/dev check`.

## Non-goals

- Per-response generation metadata, snapshot or managed-club tokens, new IPC arguments, same-read identity protocols, reused-ID guarantees, or exhaustive owner-state matrices.
- Protection from external database mutation or numeric-ID reuse outside supported app mutations.
- Cancellable Rust work, progress channels, a connection pool, a second SQLite driver, or eager all-role potential calculation during Load Data.
- Replacing TanStack Virtual, changing the database engine, or moving sorting and cohort computation into the WebView.
- Precomputing every potential role, broadly normalizing JSON metrics, or adding indexes for every selectable metric.
- Production abstractions created only to inspect query plans or measure performance.
- Performance thresholds, recorded durations, percentages, samples, machine reports, or committed performance evidence.
- New performance targets for Nationality, Club or Division display sorting, JSON attributes, personality, Position, or bounded Moneyball role sorting.
- Changing filters, metric catalogs, layouts, scoring formulas, score versions, potential invalidation, Club DNA lifecycle, snapshot selection, or managed-club membership semantics.
- Updating current-state documents before implementation makes the new state true.

## Current-state map

- `src/app/routes/search.tsx` and `src/app/routes/my-club.tsx` currently start first-page result reads in route loaders with empty `requestedFields`; `SearchResultsPanel` and `SquadOverviewPanel` issue another first-page read with the visible projection through `useSuspenseQuery`.
- `src/features/search/components/search-results-panel.tsx::SearchResultsPanel` and `src/features/squad/components/squad-overview-panel.tsx::SquadOverviewPanel` currently derive visible fields and own the panel read. My Club also keys the Squad panel by sort and direction, so sorting replaces the panel.
- `src/features/search/api/search-keys.ts::searchKeys.players(...)` and `src/features/squad/api/squad-keys.ts::squadKeys.players(...)` are parameterized page-key factories, not usable prefixes. They currently include each page request under `searchKeys.all` or `squadKeys.all`, but neither key factory owns a stable player-page prefix or a mounted-context discriminator.
- `src/components/player-table/virtualized-player-table.tsx::ConfigurableVirtualizedTable` owns virtual page reads and row activation/focus behavior. Missing page data renders placeholders, and loaded rows remain activatable.
- `src/app/components/app-top-bar.tsx::AppTopBar` owns Load Data composition and active-save invalidation. `src/features/memory-read/hooks/use-load-data.ts::useLoadData` owns the mutation lifecycle. `src/features/snapshot/components/active-save-select.tsx::ActiveSaveSelect` owns active-save switching.
- `src/features/snapshot/components/save-switcher.tsx::SaveSwitcher` receives the active-save deletion result. `src/features/snapshot/components/snapshot-history-panel.tsx::SnapshotHistoryPanel` receives current-snapshot deletion and resulting promotion. `src/app/routes/settings.tsx::invalidateCurrentContext` owns their product-query invalidation callback.
- `src/features/managed-club/components/managed-club-selector.tsx::ManagedClubSelector` owns the managed-club mutation; `src/app/routes/my-club.tsx::onManagedClubSaved` owns downstream invalidation. My Club also validly imports `searchKeys` and `squadKeys` for its existing app-owned Club DNA invalidation through `searchKeys.all` and `squadKeys.all`.
- Those app-owned mutation seams currently invalidate after success or settlement. They do not cancel and remove Search/Squad result queries before mutation, and routes do not share a result-controller blocking signal.
- Rust Search: `src-tauri/src/features/search/query.rs::search_players_in_view` validates fields, resolves the effective current snapshot, materializes potential roles used by filters or sort, builds count/page SQL inline, and orders the requested page. `query_page_uids` can repeat page-only materialization for a globally complete potential sort role.
- Rust Squad: `src-tauri/src/features/planner/squad.rs::list_squad_players` resolves the exact current managed-club cohort, enumerates club UIDs before potential materialization, and builds count/page SQL inline.
- `src-tauri/src/features/player_metrics/resolver.rs::MetricSource` expresses current role, potential role, and Club DNA values as correlated scalar subqueries. Club DNA has exact definition/model bindings and null-last ordering.
- `src-tauri/src/features/player_metrics/potential_cache.rs::materialize_snapshot_roles` calls `has_missing_role_rows`, whose nested existence check scans candidates. The cache primary key is `(snapshot_id, uid, role_id)` and `PROJECTION_MODEL_VERSION` is 2.
- `src-tauri/src/db/migrations.rs` is at v32. Existing player indexes cover `(snapshot_id, name COLLATE NOCASE)` and `(snapshot_id, ca DESC)`. Current-role, potential-role, and Club-DNA relations have identity-prefix indexes.
- `src-tauri/src/db/mod.rs::Db(pub Mutex<Connection>)` serializes commands on one connection, so duplicate or inefficient reads queue.
- Rust correctness tests in `src-tauri/src/features/search/query.rs` and `src-tauri/src/features/planner/squad_tests.rs` cover role/potential ordering, bounded pages, nulls, ties, totals, snapshot isolation, managed-club membership, and Club DNA. Frontend tests in `src/app/routes/search.test.tsx` and `src/app/routes/my-club-squad.test.tsx` cover URL sorting, headers, requested fields, virtual paging, and profile navigation.
- The stable validation surface is `./scripts/dev test <target...>`, `./scripts/dev check-app`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke`. Performance testing is manual and outside these gates.
- Current `main` and `origin/main` are synchronized at `f8b511693cc879c2f64f4e267637e3b8744007a0`. There is no planned feature spec to promote. BACKLOG remains unchanged.
- Primary risks are late canceled results entering cache, result controllers remounting during an app-owned context mutation, duplicate page-zero reads, replacement failures losing committed ownership, relation joins dropping missing scores, and potential completeness checks accepting the wrong cohort or model version.

## Feature architecture

### Direct context boundary and replacement controller

Each feature key factory owns one exact stable player-page prefix: `searchKeys.playerPages()` returns the Search player-page root, and every `searchKeys.players(...)` key extends it; `squadKeys.playerPages()` returns the Squad player-page root, and every `squadKeys.players(...)` key extends it. Only the app-layer coordinator in `src/app/player-result-context.ts` composes both `searchKeys.playerPages()` and `squadKeys.playerPages()` roots for the new cancellation/removal operation. Its async clearing operation awaits cancellation of both exact prefixes, then removes both exact prefixes. Other app modules may retain existing cross-feature composition for their current responsibilities. The coordinator owns no rows, generation identity, or global result state. The neutral shared module `src/components/player-table/player-result-context.ts` owns only the TanStack mutation key and any neutral callback type needed by both app and feature code; it imports no feature module.

App-layer owners create and inject the async pre-mutation callback. `AppTopBar` passes it to `useLoadData` and `ActiveSaveSelect`; `settings.tsx` passes it through `SnapshotPanelsWithErrorBoundary` to active-save and current-snapshot deletion; and `my-club.tsx` passes it to `ManagedClubSelector`. Feature hooks and components never import sibling feature keys or the app coordinator. Each supported context-changing mutation uses the neutral mutation key, awaits the injected callback inside `mutationFn`, and only then invokes Tauri. Active-save deletion and current-snapshot deletion use the current target flags to select that key and callback path. Inactive-save deletion and non-current-snapshot deletion omit the shared key and do not call the callback. Existing post-success or settled invalidation remains responsible for refreshing the new owners.

Search and My Club use `useIsMutating` with the neutral shared key plus their existing context-owner fetch state to block or unmount result controllers through both the mutation and owner refresh. Search suggestions and unrelated Planner queries stay outside the player-page prefixes and remain cached. A context command may finish after Query cancellation, but its canceled result must not repopulate the removed cache.

No backend identity contract changes. Search can key a mounted controller with the active save's existing ID/context token and the current snapshot's existing ID/save ID. Squad can add the existing managed-club club/status. A change to filters, combine, view, comparison pool, sorted requested fields, or those existing context values remounts the controller and clears rows.

Inside one mounted context, each panel stores only a committed request descriptor. One `useQuery` observes its committed first-page options and one observes the latest requested sort's first-page options. Identical initial keys deduplicate. Sort A→B keeps A visible and busy. B failure retains A and offers retry. B success promotes only if B is still the latest request; A→B→C cannot let late B replace C. Promotion swaps descriptor, header, summary, rows, total, and virtual-page options together. Page zero uses the exact committed options.

While retained rows are stale, the shared virtual table clears pending focus and denies pointer activation, Enter activation, row tab stops, Arrow-key pending focus, and delayed focus completion. Context changes do not use this retention path; they clear or unmount rows.

### Target query architecture

Migration v33 retains the Name and CA indexes. It adds `idx_players_snapshot_pa_asc_uid`, `idx_players_snapshot_pa_desc_uid`, `idx_players_snapshot_age_asc_uid`, `idx_players_snapshot_age_desc_uid`, `idx_players_snapshot_value_asc_uid`, `idx_players_snapshot_value_desc_uid`, and `idx_players_snapshot_current_club_uid` with the exact columns and directions recorded in Commit 2.

Current-role, potential-role, and Club-DNA ordering use relation-driven plans owned directly by Search and Squad. Focused query tests can inspect local SQL or index use when practical, but production SQL does not gain an inspection abstraction. Each plan keeps the exact request cohort and preserves absent or nullable score rows.

Warm potential completeness uses exact-version count equality. Search compares selected-role cache rows with snapshot players. Squad compares selected-role cache rows joined to the exact snapshot/managed-club cohort with that cohort count. A mismatch enters the existing bounded materializer and rechecks before ordering. When the globally complete potential sort role is also visible, page-only materialization excludes only that role. Other visible potential roles remain page-lazy. Cold requests remain correct and lazy.

## Uncertainty register

### Known

- The app-owned mutation seams and current key ownership listed in the current-state map exist in current source.
- TanStack Query owns result data and supports prefix cancellation/removal. Current `searchKeys.players(...)` and `squadKeys.players(...)` are parameterized factories, so Commit 3 must add stable `searchKeys.playerPages()` and `squadKeys.playerPages()` roots for exact cancellation/removal. The Tauri promise itself is not guaranteed to stop.
- The repository uses GitHub, `.github/pull_request_template.md`, squash merges, and the strict required GitHub Actions status `check`.
- Name and CA already have indexes. The approved migration adds exactly seven player indexes.
- There is no planned feature spec to remove. BACKLOG stays unchanged.

### Assumptions

- TanStack Query cancellation followed by removal ignores a later fulfillment from the already-started Tauri promise. Commit 3 must prove this with a deferred result at the current integration seam.
- The existing active-save, snapshot, and managed-club fields are sufficient to discriminate mounted contexts after supported app mutations clear old result queries.
- The committed/requested two-observer controller is the direct way to retain rows without copying Query data.

### Decisions

- Deliver one PR with six commits. The migration, UI ownership correction, and relation rewrites form one user outcome and do not need separate publication boundaries.
- Keep schema 2, detailed atomic packets, normal checks, and fresh review for every non-trivial commit.
- Use manual pass/fail product testing on a representative approximately 250,000-player save for performance judgment. Do not commit performance evidence or add a measurement command.
- Remove duplicate route-loader/panel result ownership and use the committed/requested two-observer controller with exact page-zero deduplication.
- Clear rows at supported context boundaries through pre-mutation cancel/remove plus result-controller blocking. Do not add response generation metadata unless focused late-result proof fails.
- Retain existing Name and CA controls. Add only the six directional scalar indexes and one managed-club membership index.
- Use relation-driven current-role, warm potential-role, and Club-DNA ordering while preserving family-specific null and missing behavior.
- Keep potential scoring lazy. Optimize warm completeness and remove only the redundant selected sort-role page pass.

### Unknowns

- Focused deferred-result tests have not yet proved that cancellation/removal suppresses late Tauri fulfillment in the current Query integration.
- The exact responsiveness of the selected query changes on the developer's representative save remains unknown until final manual validation.
- Focused local SQL tests may show that an intended index is not selected. That is a query-design input, not a mandate for more indexes.

### Risks

- A late canceled result can restore stale rows unless Query cancellation/removal works as assumed.
- A controller can remount while a supported context mutation is in flight unless all named seams share the blocking mutation key.
- A replacement error can lose the committed observer or expose the requested header over retained rows.
- Page zero can duplicate an expensive command if options differ between the controller and virtual table.
- A relation join can drop null, missing, stale, or no-definition rows.
- A completeness count can accept the wrong potential model version or outside-club rows.
- Seven indexes add storage and ingest maintenance even though performance acceptance is manual.

## Walking skeleton

Commit 2 adds the narrow index foundation. Commit 3 then proves the direct UI architecture: one result owner, pre-mutation context clearing, late-result suppression, and truthful sort-only retention. Commits 4–6 replace correlated score ordering in dependency order, with Commit 5 preserving cold lazy potential correctness and Commit 6 ending in manual product validation.

## Delivery plan

### PR 1 — Improve player table sort performance

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/player-table-sort-performance

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** GitHub required strict status `check`

**Feature close-out:** Current

**CI repair rounds:** 0

**Provisional PR title:** `perf(tables): improve player sort responsiveness`

**Purpose:** Deliver truthful sort replacement and a narrow index/query portfolio in one reviewable product change.

**Depends on:** Synchronized `main` at `f8b511693cc879c2f64f4e267637e3b8744007a0`; ADRs 0005, 0015, 0019, 0024, and 0025; no earlier feature PR and no planned spec.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(tables): record sort performance plan`

**Work:** Commit the independently reviewed ledger, TODO activation, and ADR-0025 before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, BACKLOG, current-state documents, Git mutations beyond the later authorized exact commit, and unrelated decisions.

**Implementation packet:**

- Preserve the accepted review outcome. Commit only the exact reviewed planning paths after branch and base verification.

**Files and responsibilities:**

- `.wiki/features/active/player-table-sort-performance.md` — accepted intent, architecture, packets, risks, and delivery authority.
- `.wiki/TODO.md` — simplified active feature summary and ledger link.
- `.wiki/decisions/0025-selective-index-driven-player-sorts.md` — durable simplified decision.
- `.wiki/decisions/README.md` — retain the existing ADR-0025 index row unchanged unless its title needs adjustment.
- `.wiki/BACKLOG.md` — deliberately unchanged.

**Behavior and data flow:**

- Record one active planning source and one durable decision before implementation. Make no implemented-state claim.

**Ordered implementation steps:**

1. Verify the exact feature branch and recorded base without changing scope.
2. Confirm only the approved planning paths differ and BACKLOG is unchanged.
3. Run both ledger classifiers plus Markdown and diff checks.
4. Stage only the reviewed planning paths, inspect the complete staged diff, and run the staged diff check for checkpoint review.

**Tests and proof:**

- Not applicable — independently reviewed planning documents only. Classifiers and Markdown/diff checks prove structural consistency. No tests, fixtures, mocks, snapshots, helpers, or compatibility paths change.

**Patterns to verify:**

- Schema 2 template, TODO ownership, ADR format/index, and accepted publication metadata.

**Constraints and non-goals:**

- Do not alter implementation, tests, scripts, BACKLOG, current-state documents, packet order, or reviewed decisions.

**Dependencies and sequencing:**

- Requires a clear complete plan review, developer acceptance, a valid Delivery fingerprint, and exact branch activation through delivery.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/player-table-sort-performance.md`; `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/player-table-sort-performance.md .`; `git diff --check -- .wiki/features/active/player-table-sort-performance.md .wiki/TODO.md .wiki/decisions/0025-selective-index-driven-player-sorts.md .wiki/decisions/README.md`; after exact staging, `git diff --cached --check`

**Stop conditions:** Stop on uncleared review, classifier error beyond expected `Pending review`, an unreviewed path, substantive post-review change, invalid fingerprint, or branch/base mismatch.

**Review mandate:**

1. Verify the exact planning scope and unchanged BACKLOG.
2. Verify schema 2 structure, one active packet, and six execution-ready packets.
3. Verify solo-project calibration, six-commit order, and ADR agreement.
4. Verify `Pending review` and no implementation or current-state claim.

#### Commit 2 — Add seven targeted player indexes

**Status:** Completed

**Provisional commit:** `perf(database): index targeted player sorts`

**Work:** Add migration v33 with six directional UID-complete PA/Age/Value indexes and one managed-club membership index while retaining Name/CA indexes.

**Size assessment:** About 70–110 changed non-test migration lines. Within the soft target; migration and query tests are excluded.

**Out of scope:**

- Name/CA replacement, score-relation rewrites, score indexes, JSON/Position/display indexes, frontend behavior, and performance tooling.

**Implementation packet:**

- Implement exactly seven indexes from ADR-0025. Use focused real-query assertions for semantics and inspect local index use when practical without adding a production query-plan abstraction.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — add v33; retain existing Name/CA indexes; add `(snapshot_id, pa ASC, uid ASC)`, `(snapshot_id, pa DESC, uid ASC)`, equivalent Age and `market_value_gbp` pairs, and `(snapshot_id, current_club, uid)`; update registry/latest version and fresh/v32 upgrade tests.
- `src-tauri/src/features/search/query.rs` tests — characterize both-direction PA/Age/Value results with duplicates, nulls, ties, totals, and pages; inspect intended local index use where stable and practical.
- `src-tauri/src/features/planner/squad_tests.rs` — characterize exact managed-club cohort membership, isolation, totals, pages, and ordering; inspect membership-index use where practical.

**Behavior and data flow:**

- Migration applies once through `PRAGMA user_version`. Name/CA remain unchanged. PA/Age/Value gain direction-matched indexes with `uid ASC` ties, and Squad gains exact snapshot/club membership lookup. Result semantics do not change.

**Ordered implementation steps:**

1. Add RED fresh-schema and v32→v33 tests for the exact seven indexes, directions, UID suffixes, retained Name/CA indexes, and data preservation.
2. Add RED query assertions for duplicates, nulls, ties, totals, pages, and managed-club isolation.
3. Add v33 and update the exact registry/index inventories.
4. Add local SQL/index-use assertions only where they are stable without production extraction.
5. Run Rust and full gates.

**Tests and proof:**

- RED: v33 and the seven indexes are absent.
- GREEN: fresh and upgraded databases reach v33 with data intact; index metadata proves exact columns/directions; Name/CA indexes remain; real Search/Squad queries preserve both directions, nulls, ties, totals, pages, and cohort isolation.
- Modify migration and existing focused Rust tests. Delete no Name/CA protection. Deliberately retain current migration/query fixtures because they protect supported upgrade and ordering contracts. Add no mocks, snapshots, performance fixtures, or commands.

**Patterns to verify:**

- Migration v32 registration and inventory, current scalar order builders, and existing `PRAGMA index_xinfo` tests.

**Constraints and non-goals:**

- Do not modify prior migration SQL, drop or replace Name/CA, add dependencies, force production index selection, or add more than seven indexes.

**Dependencies and sequencing:**

- Depends on Commit 1 only.

**Validation:** `./scripts/dev check-rust`; `./scripts/dev check`

**Stop conditions:** Stop on data loss, changed null/tie/page semantics, an unstable test that depends on incidental planner text, or evidence that the approved index definition cannot serve its exact query shape without another migration or query rewrite.

**Review mandate:**

1. Verify prior migrations stay immutable and v33 adds exactly seven approved indexes.
2. Verify PA/Age/Value directions and `uid ASC` suffixes plus managed-club membership columns.
3. Verify Name/CA indexes remain unchanged.
4. Verify fresh and v32 upgrade paths preserve data and exact inventory.
5. Verify both-direction null, tie, cohort, total, and page correctness.
6. Verify no score, UI, or performance-tooling scope enters the commit.

#### Commit 3 — Retain rows and clear context-bound results

**Status:** Completed

**Provisional commit:** `fix(tables): retain rows while sorting`

**Work:** Remove duplicate result ownership, add committed/requested sort replacement, and clear Search/Squad results before supported app-owned context mutations.

**Size assessment:** About 330–470 changed non-test frontend lines. This exceeds the soft target because app-layer cross-feature clearing, all supported mutation seams, route blocking, duplicate-owner removal, both replacement controllers, and the shared activation boundary form one atomic stale-row safety contract. Splitting them would leave a trunk state that can retain or remount rows across a supported context change. Tests are excluded.

**Out of scope:**

- Rust DTOs or SQL, response generation metadata, snapshot/managed-club tokens, new IPC arguments, external database mutation, reused numeric IDs outside supported mutations, background work, and global row state.

**Implementation packet:**

- Keep result data in Query and app composition in the app layer. Each feature key factory owns its exact stable player-page prefix. Add `src/app/player-result-context.ts` as the only module that composes both `searchKeys.playerPages()` and `squadKeys.playerPages()` roots for the new cancellation/removal operation. Other app modules may retain existing cross-feature composition for their current responsibilities. Keep only the neutral mutation key and optional neutral callback type in shared player-table code. App owners inject the async pre-mutation callback into feature mutations; feature code awaits it inside `mutationFn` before Tauri and never imports sibling keys or the app coordinator. Configure save and snapshot deletion from the current target flags so only active-save and current-snapshot deletion enter the shared transition. Prove exact clearing, callback-before-Tauri order, conditional deletion, route blocking, and late-result suppression. If the late-result proof fails, stop and replan instead of adding generation binding here.

**Files and responsibilities:**

- `src/features/search/api/search-keys.ts` and `src/features/search/api/search-players-query-options.ts` — add exact stable `searchKeys.playerPages()`, make every parameterized `searchKeys.players(...)` key extend it, and accept existing active-save `{ id, contextToken }` and current-snapshot ID/save ID as frontend-only mounted-context discrimination. Keep suggestions outside the prefix and IPC arguments unchanged.
- `src/features/squad/api/squad-keys.ts` and `src/features/squad/api/squad-players-query-options.ts` — add exact stable `squadKeys.playerPages()`, make every parameterized `squadKeys.players(...)` key extend it, and accept the same existing save/snapshot fields plus managed-club club/status. Keep unrelated Planner queries outside the prefix and IPC arguments unchanged.
- `src/components/player-table/player-result-context.ts` — export the neutral player-result-context mutation key and only a neutral async callback type if it removes repeated prop typing. Do not import any feature or app module and do not own rows, identity, clearing, or global state.
- `src/app/player-result-context.ts` — export the async query-clearing operation. It receives the app `QueryClient`, awaits cancellation of exact `searchKeys.playerPages()` and `squadKeys.playerPages()`, then removes both exact prefixes. It may import both feature key factories but owns no mutation, rows, generation identity, or result state.
- `src/app/player-result-context.test.ts` — prove both exact prefixes are canceled before either is removed, all Search and Squad player pages are removed, Search suggestions and unrelated Planner queries remain, and late fulfillment of canceled player-page work does not restore cache data.
- `src/app/components/app-top-bar.tsx` — create one app callback around the coordinator and pass it to `useLoadData` and `ActiveSaveSelect`; retain existing settled/success invalidation.
- `src/features/memory-read/hooks/use-load-data.ts` — accept the required injected async callback, use the neutral mutation key, and await the callback inside `mutationFn` before `loadData`; do not import Search/Squad keys or the app coordinator.
- `src/features/snapshot/components/active-save-select.tsx` — accept the injected callback, use the neutral mutation key, and await it inside `mutationFn` before `setActiveSave`; retain snapshot and shell invalidation.
- `src/app/routes/settings.tsx` and `src/features/snapshot/components/snapshot-panels-with-error-boundary.tsx` — create the app callback in Settings and pass it through the existing composition wrapper to `SaveSwitcher` and `SnapshotHistoryPanel`; keep the existing current-context invalidation callback separate and unchanged in purpose.
- `src/features/snapshot/components/save-switcher.tsx` — pass the current delete target's active flag and injected callback into `SaveDeletionModal`. Configure that deletion mutation with the neutral shared key only when the current target is active; only that async `mutationFn` path awaits the callback before `deleteSave`. Inactive-save deletion omits the shared key and invokes Tauri directly. Retain result-driven post-success invalidation.
- `src/features/snapshot/components/snapshot-history-panel.tsx` — pass the current delete target's `isCurrent` flag and injected callback into `SnapshotDeletionModal`. Configure the mutation with the neutral key and callback path only for the current snapshot; non-current deletion omits both. Await clearing before `deleteSnapshot` so any resulting promotion occurs after old player pages are gone. Retain post-success history/current invalidation.
- `src/app/routes/my-club.tsx` and `src/features/managed-club/components/managed-club-selector.tsx` — create/pass the app callback; use the neutral mutation key in `ManagedClubSelector`; await the callback inside `mutationFn` before `setManagedClub`; retain My Club's existing app-owned `searchKeys.all` and `squadKeys.all` Club DNA/downstream invalidation in place. That invalidation is not the new player-page cancellation coordinator and must not move. Feature code imports neither sibling keys nor the app coordinator.
- `src/app/routes/search.tsx` — remove Search result loader ownership and result Suspense. Use `useIsMutating` with the neutral key plus existing save/current-snapshot fetch state to block or unmount the controller through mutation and owner refresh. Clear/remount on non-sort identity or context change.
- `src/app/routes/my-club.tsx` — remove only Squad result loader ownership and sort-key remount/Suspense while retaining unrelated workspace loader responsibilities. Use the same neutral `useIsMutating` signal plus existing saves/current-snapshot/managed-club fetch state to block Squad through mutation and owner refresh.
- `src/features/search/components/search-results-panel.tsx` and `src/features/squad/components/squad-overview-panel.tsx` — own committed/requested descriptors and two `useQuery` observers; implement initial, replacement, error/retry, rapid-sort, empty, and atomic promotion states.
- `src/components/player-table/virtualized-player-table.tsx` — accept replacement busy/activation state; clear or suppress pending focus, pointer, Enter, row tab stops, Arrow focus, and delayed activation while retained rows are stale.
- `src/testing/search-ipc-mock.ts`, `src/testing/squad-ipc-mock.ts`, and existing route test helpers — add deterministic deferred, reject, retry, resolution, and invocation controls without response identity fields.
- `src/app/routes/search.test.tsx`, `src/app/routes/my-club-squad.test.tsx`, `src/app/app-top-bar.test.tsx`, and `src/app/routes/settings.test.tsx` — prove Search/Squad replacement, neutral-key route blocking through owner refresh, Load Data, active-save switch, and managed-club callback-before-Tauri order, late-result rendering suppression, page-zero deduplication, virtual pages, and activation/focus denial.
- `src/features/snapshot/components/snapshot-panels.test.tsx` — prove callback-before-Tauri order and shared mutation-key use for active-save and current-snapshot deletion, including resulting promotion; prove inactive-save and non-current-snapshot deletion use neither the callback nor shared key.
- `src/features/search/api/search-keys.test.ts` and `src/features/squad/api/squad-keys.test.ts` — prove stable player-page roots, every parameterized key extending its own root, existing-context discrimination, canonical request identity, exact page-zero equality, and unchanged mocked IPC arguments.

**Behavior and data flow:**

- `AppTopBar`, Settings, and My Club create an async callback around the app coordinator and inject it into their feature-owned mutations. A supported mutation starts under the neutral shared mutation key, awaits the callback inside `mutationFn`, and only then invokes Tauri. The coordinator awaits cancellation of both exact player-page prefixes, then removes both. Active-save and current-snapshot target flags select this path before delete; inactive-save and non-current-snapshot delete keep their current direct mutation path without the shared key. Routes observe the neutral key with `useIsMutating` and combine it with existing owner-fetch state, so result controllers stay blocked through mutation and owner refresh. Search suggestions and unrelated Planner queries remain. Inside one mounted context, A remains visible during requested B; B error retains A; Retry can commit B; rapid B→C permits only C to promote. Promotion swaps header, summary, rows, total, and page options together.

**Ordered implementation steps:**

1. Add RED key/options tests for the two exact stable roots, every parameterized `players(...)` key extending its feature-owned root, existing-context discrimination, page-zero equality, and unchanged IPC arguments.
2. Add RED `src/app/player-result-context.test.ts` coverage with a real test `QueryClient`: seed multiple Search and Squad pages, Search suggestions, unrelated Planner queries, and deferred player-page work; prove cancellation of both roots completes before removal, exact cache preservation/removal, and late fulfillment suppression.
3. Add RED mutation tests for Load Data, active-save switch, managed-club save, active-save deletion, and current-snapshot deletion. Assert that the injected callback resolves before the Tauri mock is called and that the mutation is visible under the neutral key. Assert that inactive-save and non-current-snapshot deletion neither call the callback nor appear under that key.
4. Add the neutral shared key/type and the app coordinator. Add exact roots to their feature factories. AppTopBar, Settings, and My Club create callbacks around the coordinator and pass them through the named existing composition seams.
5. Update each feature mutation to await its injected callback inside `mutationFn`. For delete modals, use the current target's `isActive` or `isCurrent` flag to conditionally include the neutral mutation key and callback branch; do not add global state or infer the condition from the mutation result.
6. Add RED replacement and route tests for initial success/error, A→B busy retention, replacement error/retry, A→B→C supersession, atomic promotion, virtual pages, every activation/focus path, and controller blocking until owner refresh settles. Then remove route result ownership/Suspense/remount assumptions and add the two-observer controllers.
7. Add shared busy/error and activation/focus behavior. Remove obsolete loader-result, sort-key remount, and Suspense expectations/helpers. Run focused tests, app gate, smoke, and full gate.

**Tests and proof:**

- RED: current loaders and panels can duplicate first-page commands; sorting replaces the panel; no app coordinator exists; feature mutations invoke Tauri without an injected pre-mutation callback; and routes do not observe one neutral transition key.
- GREEN: feature key factories own exact stable prefixes; the app coordinator cancels both exact prefixes before removing either; app owners inject one async callback; every supported mutation exposes the neutral key and awaits callback completion before Tauri; inactive/non-current deletion exposes neither; all player pages disappear while Search suggestions and unrelated Planner queries remain; deferred fulfillment cannot restore cache data or rendered rows; routes stay blocked through mutation and owner refresh; IPC arguments remain unchanged; initial/page-zero options invoke once; committed replacement, retry, rapid supersession, atomic promotion, context clearing, and activation denial all hold.
- Add the coordinator test. Modify the named frontend owners, feature components/hooks, mocks, and focused tests. Delete obsolete loader/Suspense/remount assertions and helpers. Deliberately retain URL, empty/setup, layout, virtual paging, ordinary page error, profile navigation/back, current context, inactive deletion, and non-current deletion tests because those supported contracts remain. Add no snapshots, backend fixtures, global store, or generation helper.

**Patterns to verify:**

- Biome restricted-import zones; `.wiki/ARCHITECTURE.md` shared → features → app dependency direction; app-route composition; TanStack Query v5 `useQuery`, `useMutation`, `useIsMutating`, `cancelQueries`, and `removeQueries`; current target flags and post-success invalidation; ADR-0005 Query ownership; and existing layout-stable table overlays.

**Constraints and non-goals:**

- Feature modules must not import sibling feature keys or `src/app/player-result-context.ts`; shared modules must not import features. App modules may retain existing cross-feature composition, including My Club's app-owned `searchKeys.all` and `squadKeys.all` Club DNA/downstream invalidation. Only `src/app/player-result-context.ts` may compose both `searchKeys.playerPages()` and `squadKeys.playerPages()` roots for the new cancellation/removal operation. Do not move the existing My Club invalidation into that coordinator. Do not add `placeholderData`, copied rows/totals, Zustand cache, data-fetching effects, requested headers over retained rows, stale navigation, non-sort retention, response identity protocols, extra IPC arguments, broad `searchKeys.all` or `plannerKeys.all` cancellation/removal, global transition state, or a fallback generation mechanism. Inactive-save and non-current-snapshot deletion must not use the neutral mutation key or callback.

**Dependencies and sequencing:**

- Depends on Commit 2 only for delivery order.

**Validation:** `./scripts/dev test src/app/player-result-context.test.ts src/features/search/api/search-keys.test.ts src/features/squad/api/squad-keys.test.ts src/app/app-top-bar.test.tsx src/app/routes/settings.test.tsx src/features/snapshot/components/snapshot-panels.test.tsx src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx`; `./scripts/dev check-app`; `./scripts/dev smoke`; `./scripts/dev check`

**Stop conditions:** Stop and replan to explicit request/response generation binding if a supported app-owned context change can reproduce stale-generation rows after cancellation/removal, including late fulfillment. Also stop if cross-feature clearing cannot remain app-owned, a feature or shared module must violate Biome import zones, current target flags cannot select deletion behavior before Tauri, callback-before-Tauri ordering or route blocking cannot be proved, Query data must be copied, page-zero deduplication fails, a new IPC field/argument is needed, or activation safety requires virtualizer replacement.

**Review mandate:**

1. Verify each `players(...)` key extends its feature-owned exact prefix and only `src/app/player-result-context.ts` composes both `searchKeys.playerPages()` and `squadKeys.playerPages()` roots for the new cancellation/removal operation.
2. Verify AppTopBar, Settings, and My Club inject the async callback; feature modules do not import sibling feature keys or the app coordinator; shared modules do not import features; My Club retains its existing app-owned `searchKeys.all` and `squadKeys.all` Club DNA/downstream invalidation; and every supported `mutationFn` awaits the callback before Tauri.
3. Verify active/current delete uses the neutral key and callback while inactive/non-current delete uses neither, based on current target flags without global state.
4. Verify the coordinator cancels both exact prefixes before removal, preserves suggestions and unrelated Planner queries, and late fulfillment cannot repopulate cache or UI.
5. Verify `useIsMutating` plus existing owner-fetch state blocks both routes through mutation and refresh, while context/non-sort changes clear rows and sort-only changes retain them.
6. Verify committed/requested ownership, rapid-sort guard, atomic promotion, page-zero deduplication, truthful busy/error/retry states, and unchanged IPC arguments.
7. Verify pointer, Enter, Arrow, tab stop, pending-focus, and delayed activation remain disabled for retained stale rows.
8. Verify obsolete duplicate ownership and test assets are removed without weakening retained URL, paging, navigation, context, or conditional-deletion contracts.

#### Commit 4 — Drive current-role sorts from score rows

**Status:** Completed

**Provisional commit:** `perf(tables): drive current role sorts from scores`

**Work:** Replace correlated current-role ordering with relation-driven Search and Squad queries using the existing role-score relation and index.

**Size assessment:** About 130–190 changed non-test Rust lines. Within the soft target; tests are excluded.

**Out of scope:**

- New role index, potential/Club-DNA rewrites, filters, scoring, ingest, frontend behavior, and measurement infrastructure.

**Implementation packet:**

- Add the minimum closed current-role sort metadata in the resolver and let each query owner build its direct relation-driven ordering. Preserve missing and nullable rows and all current filter/display behavior.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/resolver.rs::MetricSource` and focused tests — expose validated current-role sort identity without a public or generic query-plan abstraction.
- `src-tauri/src/features/search/query.rs::search_players_in_view` and tests — order the exact Search cohort from `player_role_scores` while retaining players without an exact score row.
- `src-tauri/src/features/planner/squad.rs::list_squad_players` and `src-tauri/src/features/planner/squad_tests.rs` — apply the same relation-driven order to the exact managed-club cohort.

**Behavior and data flow:**

- Validated role identity selects exact `(snapshot_id, role_id)` score rows. A missing-row-preserving relation orders score by requested direction with `uid ASC` ties and returns the existing bounded page and total.

**Ordered implementation steps:**

1. Add RED Search/Squad tests that characterize current correlated ordering and require a relation-driven shape while covering missing, null, duplicate, and tie rows.
2. Add the minimum resolver metadata for an exact current-role sort source.
3. Implement Search relation ordering, then Squad relation ordering, without changing display/filter expressions.
4. Remove only obsolete correlated-sort assertions.
5. Run Rust and full gates.

**Tests and proof:**

- RED: current ordering uses correlated current-role scalar lookup.
- GREEN: both surfaces use the existing current-role relation/index where practical; preserve both directions, missing/null rows, UID ties, filters, totals, bounded pages, and save/snapshot/club isolation.
- Modify named Rust source/tests. Delete only assertions tied to correlated sort structure. Deliberately retain score/filter/display/ingest/potential/Club-DNA/Moneyball tests and fixtures because their contracts do not change. Add no mocks, snapshots, or performance assets.

**Patterns to verify:**

- `MetricSource::CurrentRole`, `idx_player_role_scores_snapshot_role`, current count/page query construction, and existing missing-score tests.

**Constraints and non-goals:**

- No new index, changed score identity, dropped player, altered filter, forced planner directive, unvalidated identifier, or shared statement-plan abstraction.

**Dependencies and sequencing:**

- Depends on Commits 2–3.

**Validation:** `./scripts/dev check-rust`; `./scripts/dev check`

**Stop conditions:** Stop on changed missing/null/public behavior, cohort leakage, need for another index, need to alter score persistence, or a query design that cannot keep bounded pages and exact totals.

**Review mandate:**

1. Verify relation-driven Search and Squad ordering uses exact role identity and the existing relation/index.
2. Verify exact save/snapshot/club cohorts and filter isolation.
3. Verify missing/null retention, both directions, UID ties, totals, and pages.
4. Verify display, filter, scoring, and ingest behavior remain unchanged.
5. Verify no new index, plan abstraction, or frontend scope.

#### Commit 5 — Streamline warm potential-role sorts

**Status:** Completed

**Provisional commit:** `perf(tables): streamline potential role sorts`

**Work:** Add exact warm completeness checks, relation-driven potential ordering, and precise removal of the redundant selected-role page pass while preserving cold lazy correctness.

**Size assessment:** About 170–220 changed non-test Rust lines. The completeness, cold fallback, ordering, and selected-role pass removal are one correctness boundary and may slightly exceed the soft target. Tests are excluded.

**Out of scope:**

- Eager potential scoring, schema or model-version change, background work, pool, display-only page-lazy removal, current-role/Club-DNA changes, frontend work, and performance tooling.

**Implementation packet:**

- Replace warm nested missing probes with exact-version count equality, retain bounded cold materialization and recheck, drive global ordering from exact-version rows, and skip only the displayed global sort role's redundant page materialization.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/potential_cache.rs::has_missing_role_rows`, `materialize_snapshot_roles`, and focused tests — add exact Search snapshot and Squad cohort count helpers that include nullable scores and exact model version; retain cold recheck.
- `src-tauri/src/features/player_metrics/resolver.rs::MetricSource` and tests — expose exact validated potential sort identity/version.
- `src-tauri/src/features/search/query.rs::search_players_in_view`, `query_page_uids`, and tests — use count completeness, cold fallback, relation ordering, and selected sort-role pass removal.
- `src-tauri/src/features/planner/squad.rs::list_squad_players` and `src-tauri/src/features/planner/squad_tests.rs` — apply exact cohort completeness and relation ordering without losing managed-club membership.

**Behavior and data flow:**

- Search compares exact-version selected-role rows with snapshot players. Squad compares exact-version rows joined to the current snapshot/managed-club cohort with that cohort. Equal counts include nullable scores because primary-key uniqueness gives one row per UID. Inequality runs the existing bounded materializer and rechecks. The complete relation orders globally. A distinct visible potential role stays page-lazy.

**Ordered implementation steps:**

1. Add RED completeness tests for nullable, missing, stale-version, and managed-club membership-transition cases.
2. Add RED cold Search/Squad tests that remove selected-role rows and require complete correct order, total, page, exact model version, and no writes to unrelated roles.
3. Add RED warm instrumentation proving no materialization when complete and no redundant selected sort-role page pass while a second visible role stays lazy.
4. Implement exact count helpers and post-materialization recheck.
5. Add relation-driven Search/Squad ordering and exclude only the selected global sort role from page materialization.
6. Remove obsolete nested missing-probe helpers/assertions and run Rust/full gates.

**Tests and proof:**

- RED: the warm path uses nested probes/correlated ordering and repeats page materialization.
- GREEN: complete nullable cohorts bypass writes; stale/missing/transition cohorts rebuild and recheck; cold requests lazily complete before global ordering; both surfaces preserve directions, nulls, ties, totals, pages, and isolation; the selected visible sort role skips only its redundant pass; another visible potential role remains page-lazy.
- Modify cache/resolver/query owners and existing tests. Delete obsolete missing-probe helpers/assertions. Deliberately retain projection, scoring, batch, resume, invalidation, display-only, and boost tests because cold lazy ownership remains supported. Add no frontend mocks, snapshots, or performance assets.

**Patterns to verify:**

- ADR-0019, potential primary key/version, `materialize_snapshot_roles`, page UID materialization, and existing cold potential tests.

**Constraints and non-goals:**

- Count nullable rows with exact version and cohort. Do not infer completeness from non-null scores, unrelated roles, or frontend state. Preserve cold laziness and unrelated display-role materialization.

**Dependencies and sequencing:**

- Depends on Commit 4's sort-source direction.

**Validation:** `./scripts/dev check-rust`; `./scripts/dev check`

**Stop conditions:** Stop if count equality is unsound, cold completion changes relation contents or ordering, membership transitions fail, another visible role loses laziness, the selected role still repeats materialization, or another index/schema/background mechanism becomes necessary.

**Review mandate:**

1. Verify exact-version count proof covers nullable, stale, missing, and managed-club transition cases.
2. Verify cold materialization remains bounded, lazy, complete, and rechecked before ordering.
3. Verify relation-driven warm ordering preserves exact cohorts without correlated sort lookup.
4. Verify both directions, nulls, ties, totals, pages, and unrelated-role writes.
5. Verify only the selected global sort role skips its redundant page pass.
6. Verify other visible potential roles remain page-lazy and existing invalidation/version ownership remains unchanged.

#### Commit 6 — Drive Club DNA sorts from score rows

**Status:** Completed

**Provisional commit:** `perf(tables): drive Club DNA sorts from scores`

**Work:** Replace correlated Club DNA ordering with exact relation-driven Search and Squad queries, then complete final automated and manual product validation.

**Size assessment:** About 120–180 changed non-test Rust lines. Within the soft target; tests and manual validation are excluded.

**Out of scope:**

- Club DNA writer/schema/index/version changes, definition UX, unrelated filters, current/potential changes, performance infrastructure, and quantified performance claims.

**Implementation packet:**

- Drive ordering from exact persisted Club DNA rows while preserving missing, stale, no-definition, computed-null, and null-last behavior. After automated correctness passes, run the simple representative-save manual checklist and record only `Pass` or `Fail` in the delivery handoff or PR checklist.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/resolver.rs::MetricSource`, `ClubDnaSqlBindings`, and tests — expose exact definition/model sort identity without changing scoring ownership.
- `src-tauri/src/features/search/query.rs::search_players_in_view` and existing Club DNA tests — relation-driven exact-definition/model ordering with missing-row retention and null-last.
- `src-tauri/src/features/planner/squad.rs::list_squad_players` and `src-tauri/src/features/planner/squad_tests.rs` — apply the same relation order to the exact managed-club cohort.
- Delivery handoff or PR checklist — record only `Pass` or `Fail` for the manual product validation; do not add a repository evidence file.

**Behavior and data flow:**

- Query resolves the current definition once, joins exact `(snapshot_id, definition_version, score_model_version)` rows to the cohort, retains absent/stale/null/no-definition players, and orders null last, score by requested direction, then `uid ASC`. Reads remain bounded and do not write.

**Ordered implementation steps:**

1. Add RED Search/Squad tests that reject correlated Club DNA sort while retaining missing, stale, no-definition, null, duplicate, and tie cases.
2. Add exact sort-source metadata and Search relation ordering.
3. Add Squad relation ordering and prove score-row counts remain unchanged.
4. Remove only obsolete correlated-sort assertions and run focused Rust/full correctness gates.
5. On a representative approximately 250,000-player save, exercise both directions for Search Name/CA controls, PA/Age/Value/current role/warm potential/Club DNA, and Squad CA over the configured managed-club cohort, current role, warm potential role, and Club DNA.
6. Confirm tables remain visible only for sort replacement, context changes clear them, rows/order/totals remain credible, and targeted sorts are acceptably responsive by developer judgment. Record only `Pass` or `Fail` in the handoff or PR checklist.
7. Run smoke and the final full gate. Set feature status to Validation only after automated checks and manual result pass.

**Tests and proof:**

- RED: current ordering uses correlated Club DNA scalar lookup.
- GREEN: relation ordering uses exact definition/model identity without read writes; both surfaces preserve null-last, missing, stale, no-definition, duplicates, both directions, UID ties, exact cohorts, totals, and bounded pages. Existing frontend replacement and context tests remain green. Manual performance and interaction validation records only pass/fail.
- Modify resolver/query owners and existing Rust tests. Delete only correlated-sort assertions. Deliberately retain eager writer, formula, filter, UI, null/stale, read-only, and migration tests because those contracts remain. Add no mocks, snapshots, fixtures, reports, or architecture docs for performance testing.

**Patterns to verify:**

- ADR-0024, `ClubDnaSqlBindings`, the existing Club DNA relation index, and current null-last tests.

**Constraints and non-goals:**

- No read writes, identity/index change, weakened null-last behavior, performance evidence file, duration/percentage/sample recording, or quantified improvement claim.

**Dependencies and sequencing:**

- Depends on Commits 2–5. Final implementation packet.

**Validation:** `./scripts/dev check-rust`; `./scripts/dev check`; `./scripts/dev smoke`; manual representative-save checklist with only `Pass` or `Fail` recorded outside committed repository evidence

**Stop conditions:** Stop on correlated lookup, read writes, changed null/missing contract, failed automated correctness, manual `Fail`, need for another index/architecture, or any request to claim a quantified gain without separate approved evidence.

**Review mandate:**

1. Verify exact definition/model relation ordering with no correlated lookup or read write.
2. Verify null-last, missing, stale, no-definition, duplicate, direction, tie, cohort, total, and page behavior.
3. Verify writer, schema, index, formula, and filter ownership remain unchanged.
4. Verify final automated gates cover SQL semantics, cold potential, context cancellation, replacement, virtual pages, and activation.
5. Verify the manual checklist covers both directions for Search Name/CA, PA/Age/Value, current role, warm potential role, and Club DNA plus Squad CA over the configured managed-club cohort, current role, warm potential role, and Club DNA.
6. Verify the handoff records only `Pass` or `Fail` and makes no quantified performance claim.

## Discoveries and replanning

- Current-source inspection confirmed duplicate Search/Squad result ownership in route loaders and panels, keyed Squad sort remounting, the single Db mutex, v32 indexes, correlated persisted-score expressions, and the redundant selected potential-role page pass.
- Current-source inspection also confirmed the exact supported mutation seams: `AppTopBar`/`useLoadData`, `ActiveSaveSelect`, `SaveSwitcher` active-save deletion, `SnapshotHistoryPanel` current-snapshot deletion/resulting promotion through `settings.tsx::invalidateCurrentContext`, and `ManagedClubSelector` through `my-club.tsx::onManagedClubSaved`.
- Current `searchKeys.players(...)` and `squadKeys.players(...)` are parameterized factories, not usable prefixes. Commit 3 adds direct stable `searchKeys.playerPages()` and `squadKeys.playerPages()` roots and makes each parameterized factory extend its own root. Only the app coordinator composes both roots for the new cancellation/removal operation; app owners inject that operation into feature mutations. Current mutations invalidate after success or settlement but do not yet cancel/remove player-page queries before mutation. My Club's existing app-owned `searchKeys.all` and `squadKeys.all` Club DNA/downstream invalidation remains separate and in place.
- Developer-directed solo-project replan supersedes correction rounds 1–3. It removes statement-plan extraction, performance tooling, permanent performance evidence, immutable response-generation requirements, snapshot/managed-club token additions, same-read identity protocols, and exhaustive context matrices.
- Correction round 1 resolves the plan reviewer's cross-feature ownership finding. Commit 3 now places the new two-root Search/Squad player-page cancellation/removal operation in `src/app/player-result-context.ts`, keeps only a neutral mutation key/type in shared player-table code, injects the async callback from AppTopBar, Settings, and My Club, and defines active/current-only deletion from current target flags. Feature mutations import neither sibling keys nor the app coordinator. My Club retains its existing app-owned `searchKeys.all` and `squadKeys.all` Club DNA/downstream invalidation outside the coordinator.
- Correction round 2 narrows the ownership wording to the new operation: only `src/app/player-result-context.ts` composes both player-page roots for cancellation/removal. App modules may retain existing cross-feature composition, and My Club's current Club DNA/downstream invalidation must not move.
- The approved replacement is direct: use existing context fields; cancel and then remove exactly `searchKeys.playerPages()` and `squadKeys.playerPages()` before supported app mutations invoke Tauri; preserve Search suggestions and unrelated Planner queries; block controllers during the transition; and prove late-result suppression with focused deferred tests. Explicit request/response generation binding is deferred unless that proof fails or a supported context mutation reproduces stale-generation rows.
- The replan preserves the one-PR schema-2 structure, planning-only first packet, seven-index scope, committed/requested observers, duplicate-owner removal, relation-driven current/potential/Club-DNA sorts, warm potential optimization, cold lazy correctness, bounded IPC, one SQLite connection, exact SQL semantics, normal checks, review, and manual representative-save judgment.
- Delivery now has exactly six commits. Every implementation packet has a new fingerprint because packet scope, proof, dependencies, validation, stop conditions, and review concerns changed materially.
- The exact planning scope remains `.wiki/features/active/player-table-sort-performance.md`, `.wiki/TODO.md`, `.wiki/decisions/0025-selective-index-driven-player-sorts.md`, and `.wiki/decisions/README.md` only if its existing row needs adjustment. BACKLOG and planned specs remain unchanged.
- Delivery fingerprint remains `fdf0376491139cf1da0cf2e84d5fa5a0d0921dab9c10f7583f5ad2f304e1ce79`.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Improve player table sort performance | Commit 1 — Record the approved feature plan | a1bf86feeddfc89fc0c3b0d3328ac9bc971b8ed7 | Recorded the accepted schema 2 ledger, TODO activation, ADR-0025, and ADR index entry; BACKLOG stayed unchanged. | Both classifiers were runnable; Markdown/LSP diagnostics and exact staged diff checks passed. | Not applicable | Clear | 0 | None |
| PR 1 — Improve player table sort performance | Commit 2 — Add seven targeted player indexes | 5985da86ab02abbb80df7c252f5e418167483a73 | Added migration v33 with exactly six directional PA, Age, and Value indexes plus one managed-club membership index; retained Name and CA indexes and unchanged query behavior. | `./scripts/dev check-rust` passed with 584 tests and 2 ignored; `./scripts/dev check` passed; Rust LSP and staged diff checks passed. | Pass | Clear | 0 | None |
| PR 1 — Improve player table sort performance | Commit 3 — Retain rows and clear context-bound results | 1ed1f17bae6f0a8c401301a7e82d53062c014adc | Added exact player-page roots and app-owned cancellation/removal, injected context transitions, committed/requested result controllers, truthful sort-only retention, projection/context clearing, and stale-row activation denial. | Focused frontend validation passed 231 tests; `./scripts/dev check-app`, 49-test smoke, `./scripts/dev check`, LSP, and staged diff checks passed. | Pass | Clear | 2 | None |
| PR 1 — Improve player table sort performance | Commit 4 — Drive current-role sorts from score rows | 812aa46f0e0dffc68dd55c3d6de8f588c2eb942a | Replaced correlated current-role sort lookup with exact missing-preserving Search and Squad joins to validated `player_role_scores` identities, including the potential-display page path. | `./scripts/dev check-rust` passed with 591 tests and 2 ignored; `./scripts/dev check`, Rust LSP, and staged diff checks passed. | Pass | Clear | 1 | None |
| PR 1 — Improve player table sort performance | Commit 5 — Streamline warm potential-role sorts | db65160bdd0a6e5a598efdcc2606679db2894454 | Replaced nested warm probes with exact-version completeness counts, retained bounded cold recheck, added exact relation ordering, and skipped only the selected role's redundant page pass. | `./scripts/dev check-rust` passed with 597 tests and 2 ignored; `./scripts/dev check`, Rust LSP, and staged diff checks passed. | Pass | Clear | 0 | None |
| PR 1 — Improve player table sort performance | Commit 6 — Drive Club DNA sorts from score rows | 0111f0835a25bb24211e1254635c027a09c83a39 | Replaced correlated Club DNA sort lookup with exact missing-preserving, null-last Search and Squad score relations while keeping reads bounded and read-only. | `./scripts/dev check-rust` passed with 599 tests and 2 ignored; `./scripts/dev check` passed; exact smoke rerun passed 49 tests; manual representative-save acceptance: Pass. | Pass | Clear | 0 | None |

## Final validation

- `./scripts/dev test src/app/player-result-context.test.ts src/features/search/api/search-keys.test.ts src/features/squad/api/squad-keys.test.ts src/app/app-top-bar.test.tsx src/app/routes/settings.test.tsx src/features/snapshot/components/snapshot-panels.test.tsx src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx`
- `./scripts/dev check-rust`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Automated acceptance covers migration inventory and upgrade, SQL null/missing/tie/total/page semantics, exact cohorts, cold potential completion and ordering, no unrelated potential writes, feature-owned exact player-page prefixes, app-coordinator cancellation then removal, injected callback-before-Tauri ordering, active/current-only conditional deletion, preservation of Search suggestions and unrelated Planner queries, route blocking through owner refresh, late-result suppression, sort replacement/error/retry/rapid requests, page-zero deduplication, bounded virtual pages, atomic promotion, and stale-row activation/focus denial.
- Manual acceptance on a representative approximately 250,000-player save exercises both directions for Search Name/CA controls, PA/Age/Value/current role/warm potential/Club DNA, and Squad CA over the configured managed-club cohort, current role, warm potential role, and Club DNA. Confirm the table stays visible only during sort replacement, context changes clear it, rows/order/totals remain credible, and targeted sorts are acceptably responsive by developer judgment.
- Record only `Pass` or `Fail` for manual acceptance in the delivery handoff or PR checklist. Do not record durations, percentages, samples, reports, or a quantified gain.
- Feature review verifies the seven-index limit, retained Name/CA controls, feature-owned stable player-page prefixes, coordinator-only composition of both player-page roots for the new cancellation/removal operation, retention of existing app-owned cross-feature invalidation, neutral shared mutation ownership, injected callback-before-Tauri ordering, active/current-only conditional deletion, exact cancellation-then-removal without broad Search/Planner eviction, preservation of Search suggestions and unrelated Planner queries, route blocking, late-result proof, two-observer ownership, exact key deduplication, relation-driven ordering, warm/cold potential contracts, bounded IPC, one SQLite connection, and absence of client-side full-set sorting or speculative generation/concurrency mechanisms.

## Documentation impact

Feature completion reconciled the implemented architecture in `.wiki/ARCHITECTURE.md`, moved this complete ledger to the completed-features owner, and updated `.wiki/TODO.md` to remove the active pointer and add the completed record. ADR-0025 now records the implemented decision. BACKLOG and planned specs remain unchanged. No performance evidence, command documentation, or performance architecture document was created.

## Exact implementation refs

- Base: `f8b511693cc879c2f64f4e267637e3b8744007a0`
- Planned content refs: `a1bf86feeddfc89fc0c3b0d3328ac9bc971b8ed7`, `5985da86ab02abbb80df7c252f5e418167483a73`, `1ed1f17bae6f0a8c401301a7e82d53062c014adc`, `812aa46f0e0dffc68dd55c3d6de8f588c2eb942a`, `db65160bdd0a6e5a598efdcc2606679db2894454`, `0111f0835a25bb24211e1254635c027a09c83a39`
- Correction ref: `b5a5dc688e59fcd5f9e9a567c4387705ddedff30`
- Documentation reconciliation ref: Pending record

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feature/player-table-sort-performance
base_branch: main
base_ref: f8b511693cc879c2f64f4e267637e3b8744007a0
publication_provider: GitHub
merge_method: squash
required_check_name: check
pr_template: .github/pull_request_template.md
feature_close_out: current
feature_review_blocking: false
feature_review_critical: none
feature_review_high: none
feature_review_medium: none
feature_review_nitpick: none
ci_repair_rounds: 0 of 2
earlier_pr_merge_refs: []
correction_ref: b5a5dc688e59fcd5f9e9a567c4387705ddedff30
close_out_documentation_ref: Pending record
implementation_range: "f8b511693cc879c2f64f4e267637e3b8744007a0..0111f0835a25bb24211e1254635c027a09c83a39"
final_pr_commit_set:
  - a1bf86feeddfc89fc0c3b0d3328ac9bc971b8ed7
  - 5985da86ab02abbb80df7c252f5e418167483a73
  - 1ed1f17bae6f0a8c401301a7e82d53062c014adc
  - 812aa46f0e0dffc68dd55c3d6de8f588c2eb942a
  - db65160bdd0a6e5a598efdcc2606679db2894454
  - 0111f0835a25bb24211e1254635c027a09c83a39
  - b5a5dc688e59fcd5f9e9a567c4387705ddedff30
  - Pending record
```

## Feature close-out

**State:** Current.

The feature-review correction round at `b5a5dc688e59fcd5f9e9a567c4387705ddedff30` fixed stale cached replacement promotion and incomplete potential-sort proof. Correction review: Blocking No, no remaining findings, Test portfolio Pass, Project fit Conforms. Final validation passed: exact focused frontend 233 tests; `./scripts/dev check-rust` 601 tests and 2 ignored; full `./scripts/dev check`; smoke 49 tests; and developer manual representative-save checklist `Pass`. The PR remains unpublished and unmerged, with zero of two CI repair rounds used.
