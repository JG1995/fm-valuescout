# Club DNA

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 3fea871899c7615d5449aaddb1ac13d0395b5d8473fa146389c2f734cfb0ca18

## Intent

Let the user define one save-owned Club DNA score from selected Football Manager player attributes and use the fixed score in the existing Search and Squad tables.

## User-visible behavior

- My Club shows a **Define DNA** action next to **Save managed club**. The action is disabled until the active save has a managed club selection.
- The action opens a form Modal with the Player Profile's FM-style attribute groups. The user can select any number of attributes from the closed visible, goalkeeper, hidden, and personality catalogs. Save requires at least one selection.
- The Modal is the only explanation surface. It lists the selected attributes and states that Club DNA scales each selected 1–20 value by 5, gives every selected attribute equal weight, averages the values, and rounds to a whole 0–100 score.
- Each app save owns at most one definition with the fixed label **Club DNA**. A later managed-club change keeps the definition because the definition is not keyed to a club name.
- The user can create, view, edit or replace, and remove the definition. Removing it does not remove the fixed metric from saved table layouts, filters, sort state, or navigation history.
- A player receives Club DNA only when every selected attribute is an integer from 1 through 20. If one selected value is missing, null, non-integer, or outside 1–20, or the active save has no definition, the metric is unavailable and renders `—`.
- On initial creation, the app appends `club_dna` once to the existing app-local Search and Squad layouts. Editing does not restore a column that the user removed. Re-creating a definition after deletion is a new creation and can append a missing column again.
- General Search can display, sort, and filter Club DNA. Squad can display and sort it. Both use the shared score presentation and retain unavailable values as null rather than zero.
- The first request for an uncached Club DNA page or cohort can take longer while Rust materializes scores. The existing table query-loading state stays visible, no partial filtered or sorted result appears, and later warm requests use the cache. Cold duration is measured and reported separately from warm interaction latency. At the representative 184,000-player scale, each warm Search filter, Search sort, or Squad sort may approach one second.
- Hidden and personality attributes remain eligible even when profile concealment is on. Concealment is a presentation preference, not authorization.

## Invariants

- Rust and SQLite own definition validation, persistence, score derivation, filtering, and sorting. React never computes a Club DNA table score.
- Definition attribute IDs use the existing closed metric forms: `attr.<PascalCase>`, `hidden.<PascalCase>`, and `personality.<PascalCase>`. Goalkeeper attributes use `attr.*` because they are stored in `players.attributes_json`.
- A definition contains at least one unique supported attribute ID. The user maximum is the complete closed catalog; no lower implementation cap may reject selecting all supported attributes.
- The formula is deterministic: for `n` selected values, calculate `round((sum(value × 5)) / n)` as one integer from 0 through 100. The same definition and player JSON produce the same result in Search and Squad.
- Missing definition, missing key, explicit JSON null, or any selected value that is non-integer or outside integer 1–20 makes the complete score null. Partial averages, clamping, and zero substitution are forbidden.
- The definition belongs to `saves`, survives snapshot replacement and restart, changes with the active save, and cascades when its save is deleted. Snapshot deletion does not delete it.
- The fixed `club_dna` metric remains in the frontend and Rust catalogs independently of definition existence. Definition removal does not rewrite app-local layouts or URL-backed Search filter/sort state.
- Authoritative attribute-ID validation occurs in Rust at the command/service boundary. SQL is built only from the fixed metric and validated stored IDs; the WebView cannot supply SQL or arbitrary JSON paths.
- Search and Squad continue to return bounded pages. The WebView does not receive a full cohort to calculate or order.
- No cross-feature frontend import is added. Shared attribute catalog data lives in a shared utility, and `src/app/routes/my-club.tsx` composes the managed-club and Club DNA features.

## Non-goals

- Custom definition names, multiple definitions per save, per-attribute weights, cloning, import, export, or sharing.
- Moneyball integration or a Moneyball Search metric.
- Club-name ownership, automatic definition changes after a managed-club change, or a definition per club.
- Player Profile display, per-player contribution disclosure, or any explanation outside the definition Modal.
- A new hidden-information concealment or authorization rule.
- Eager ingest or promotion-time score computation, an in-memory-only cache, or a request-local full-cohort calculation that is discarded after each query.
- New table layout ownership, a new table component, or global removal of saved layout, filter, sort, or history state.

## Current-state map

- Relevant components: `src/app/routes/my-club.tsx::MyClubPageContent` composes the My Club header and `ManagedClubSelector`; `src/features/managed-club/components/managed-club-selector.tsx::ManagedClubSelector` owns the selector form and **Save managed club** action; `src/components/ui/modal/modal.tsx::Modal` owns dialog focus and dismissal; `src/features/player-profile/utils/attribute-groups.ts` owns the current FM-style frontend attribute grouping.
- Data model: `players` stores visible and goalkeeper values in `attributes_json`, hidden values in `hidden_attributes_json`, and personality values in `personality_json`. Committed v31 and `src-tauri/src/features/club_dna/` provide one save-keyed definition plus context-bound get/set/remove commands. Committed v32 adds positive definition versions and the nullable, versioned `club_dna_scores` cache.
- Persistence and migrations: committed migration v31 adds the save-owned definition. Committed migration v32 preserves v31, adds `definition_version`, and adds the disposable cache with identity `(snapshot_id, uid, definition_version, score_model_version)` plus the `(snapshot_id, definition_version, score_model_version, score)` lookup/order index. `src-tauri/src/features/player_metrics/club_dna.rs` owns pure scoring, bounded lazy materialization, and exact-version cache writes. Definition changes and supported player boosts own atomic invalidation.
- Existing behavioral assumptions: most React query keys omit save IDs because app-shell and Settings context changes invalidate feature roots, but save-owned Club DNA cannot rely on invalidation alone. The established saves query exposes each active save's ID and immutable `contextToken`. Search filters and sort live in URL state. Search and Squad column IDs, order, and widths live app-locally in Zustand key `fm-valuescout-player-table-layouts`, version 5. `addColumns` validates IDs and appends only missing columns.
- Architectural seams: `src/utils/player-metrics.ts` and `src/components/ui/player-metric-picker.tsx` own the frontend fixed metric catalog and picker. `src-tauri/src/features/player_metrics/resolver.rs::MetricField` owns the independent Rust catalog and dynamic value/sort expressions. `src-tauri/src/features/player_metrics/potential_cache.rs` shows the established lazy materialization boundary. `src-tauri/src/features/search/filter.rs` owns trusted Search filter compilation. `src-tauri/src/features/search/query.rs` and `src-tauri/src/features/planner/squad.rs` own the current-snapshot Search and exact managed-club Squad cohorts.
- Shared table adapters: `src/features/search/components/search-results-panel.tsx` and `src/features/squad/components/squad-overview-panel.tsx` request visible dynamic fields and render nullable values. `src/components/player-table/` owns table interaction. Existing dynamic DTO maps already carry nullable integers.
- Command boundary: `src/lib/tauri-client.ts` is the sole frontend invoke wrapper. `src-tauri/src/lib.rs` registers Tauri commands, and `src-tauri/src/features/mod.rs` registers feature modules.
- Project validation commands: `./scripts/dev test <targets>`, `./scripts/dev check-app`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` is unsupported and cannot be evidence.
- Primary risks: cross-language catalog drift, accepting an empty or unknown definition, partial averages, stale or mixed-version cache rows, query shapes that bypass the v32 score index, repeated per-player definition/cache lookup, incomplete cohort materialization before global filter/sort, long first-use database lock time, stale active-save Query data, re-adding a user-removed column on edit, and removing saved layout state with the definition.

## Feature architecture

Committed migrations v31 and v32, ADR-0023, the cache identity, the v32 score index, lazy bounded materialization, invalidation owners, score-model versioning, request scopes, and UX remain unchanged. The accepted representative warm threshold changes from `<=200 ms` to `<=1,000 ms` for each Search filter, Search sort, and Squad sort shape. Cache rows remain nullable derived data. Complete materialization creates one exact-version cache row for every member of a global operation, including unavailable scores.

Each Search or Squad request fetches and validates one trusted Club DNA definition/model context before it plans Club DNA SQL. The request binds the snapshot, definition version, and score-model version once and reuses that context through materialization and the dependent statements. Missing definition is an explicit no-write context. The request performs no materialization and no Club DNA cache join. Display projects `club_dna` as null. A Club DNA sort stays player-first, returns every player with an all-null Club DNA value, and uses the existing UID tie-break for stable order. The flat Search filter compiler emits a SQL-false predicate for each Club DNA rule, including `neq`, instead of replacing the complete filter. An isolated Club DNA rule and any mixed AND filter that contains one therefore return no rows, while a mixed OR filter can still return rows that match a non-Club-DNA rule. A missing-definition request never enters a cache-first plan from an empty cache.

Display-only Search and Squad remain player-first. The query selects the bounded page UIDs under the existing non-Club-DNA order, materializes only those UIDs, and the final select `LEFT JOIN`s one exact-version cache alias to return nullable display values. A display query does not perform a correlated definition lookup and does not turn a missing cache row into zero.

Global Club DNA operations become cache-first after complete materialization. Before UID enumeration or any materializer call, Search compares the exact-version snapshot cache-row count with the snapshot player count. Squad compares the exact-version cache-row count joined to the exact managed-club predicate with that same managed-club cohort count; cache rows for players outside the managed club never contribute. Nullable cache rows count toward completeness. The cache primary key and foreign keys prevent duplicate or orphan rows from making either comparison pass falsely. A matching count is authoritative for the request under the existing command-level mutex: the request skips UID allocation, materializer calls, and every batched cache-miss probe. A mismatched count uses the existing resumable materializer. The request reuses a cohort count when the same validated predicate and snapshot/version context make that reuse safe.

Cohort transitions do not all create a mismatch. A Search player addition or Squad club entry creates a mismatch only when the entrant lacks an exact current-version cache row; an entrant already materialized by Search increases both Squad counts and can preserve equality. Player removal cascades the player and cache rows, so both Search counts decrease together. A Squad club exit removes the player from both cohort predicates, so both Squad counts decrease together without deleting an otherwise valid cache row. A managed-club switch to an already complete cohort can also preserve equality. Every equality case must safely skip UID enumeration and materialization; only an actual count mismatch resumes materialization.

After completeness is established, Search filter and sort query `club_dna_scores` first and bind the exact snapshot and versions. Search filter uses the v32 `(snapshot_id, definition_version, score_model_version, score)` index for exact-version count and candidate selection, then joins those score-filtered candidates to players; its required final CA/UID ordering and bounded page selection may use a temporary sort. Search Club DNA sort and Squad Club DNA sort use the same index for exact score selection and score order. They may use only the documented temporary sort needed for null-last ordering and the UID tie-break that v32 does not cover. Because complete materialization stores nullable rows, these global operations may use an `INNER JOIN` to the exact-version cache without dropping unavailable players; filters explicitly exclude null. No global shape may use a correlated definition lookup or an unbounded player-first scan.

SQL interpolates only fixed internal aliases and validated sort directions. Snapshot, definition version, score-model version, filter values, page bounds, cohort counts, and other data remain bound values. The resolver and filter compiler consume the trusted request context and exact cache alias; they do not emit repeated scalar subqueries or correlated definition lookups for filter, sort, or display. This correction adds no durable completeness marker, migration, or index. The developer rejected a migration v33 with directional, null-ordering, and player-join indexes because the measured v32/count-fast-path queries already meet the accepted one-second representative interaction contract, while more indexes would add storage, write cost, query variants, and maintenance complexity.

Definition edit/remove/re-create, supported player-boost invalidation, ingest/promotion laziness, command-level mutex ownership, and bounded cache write transactions remain as implemented in Commit 4. React behavior also stays unchanged: one fixed integer metric uses backend-supplied values, typed definition adapters bind the active save context, create-only layout append remains guarded, and edit/remove do not rewrite layouts or URL state.

[ADR-0023](../../decisions/0023-lazy-club-dna-score-cache.md) remains the accepted cache decision. This bounded replan records its changed representative threshold, rejected v33 index alternative, measured cold and warm evidence, and current implementation status. It needs no migration, index, current-state architecture, design, TODO, or BACKLOG change.

## Uncertainty register

### Known

- Linear JAY-32 is the external work item. There is no planned feature spec to promote.
- Commits 1–6 are complete at `ddd4961e6d90ca24faa435955c6ae7eb5a716f0b`, `d2682ee5c50cb99cd0b7f9facf5fd4f9060d5001`, `7cf5e5924af8a9c54852f5037e17ffe4b2c58cc0`, `d78f97f25497409f6c895a8ac5cdeb74ea5301eb`, `df074b9da78bec038960e2be7c851dcb5879dbdd`, and `8250dbe9aac7853ed90ba674f83a67da870a8ecb`. The failed Commit 7 six-path worktree was discarded cleanly.
- Committed v32 provides definition versions, nullable cache rows, exact cache identity, the score lookup/order index, bounded lazy materialization, invalidation, and the accepted exact-count completeness architecture. Search and Squad do not expose Club DNA at clean HEAD `8250dbe9aac7853ed90ba674f83a67da870a8ecb`.
- The discarded exact-count/indexed 2,000-player run reported all shapes. Search filter cold first use was 22.367 ms with warm p95 3.575 ms. Search sort cold first use was 25.969 ms with warm p95 5.841 ms. Squad sort cold first use was 25.543 ms with warm p95 7.280 ms. All three warm shapes meet the unchanged `<500 ms` threshold.
- The discarded exact-count/indexed 184,000-player run reported all shapes. Search filter cold first use was 2496.594 ms with warm p95 373.344 ms. Search sort cold first use was 2644.854 ms with warm p95 509.068 ms. Squad sort cold first use was 2904.600 ms with warm p95 856.517 ms. All three warm shapes meet the developer-chosen `<=1,000 ms` threshold.
- The remaining representative cost comes from sorting and joining large exact queries after the exact-count completeness fast path has removed warm full-cohort UID enumeration and batched cache-miss probes. The measured cold durations are delayed first-use evidence and are not warm acceptance measurements.
- Search supports display, sort, and filter. Squad supports display and sort. Both already carry nullable integer dynamic values.

### Assumptions

- Canonical metric-style IDs remain the narrowest persisted definition because they encode the closed attribute and JSON source.
- A route-owned action slot in `ManagedClubSelector` is sufficient to place **Define DNA** beside **Save managed club** without a cross-feature import.
- The existing `ScoreBadge` is the correct 0–100 presentation in both tables.

### Decisions

- Keep ADR-0023, committed migrations v31/v32, the v32 index, cache identity, lazy scopes, invalidation, versions, UX, frontend behavior, exact-count completeness architecture, and one-PR delivery authority. Change only the representative 184,000-player warm threshold from `<=200 ms` to `<=1,000 ms` for each Search filter, Search sort, and Squad sort.
- Reject migration v33 directional, null-ordering, and player-join indexes. The v32/count-fast-path design meets the accepted shared threshold, while v33 would add persistent storage, write amplification, multiple query-specific index variants, migration and EXPLAIN obligations, and maintenance complexity for no required product gain.
- Fetch and validate one trusted Club DNA definition/model context per Search or Squad request. Reuse its bound snapshot, definition version, and score-model version through materialization and all dependent statements.
- Keep display-only queries player-first and page-scoped. Materialize the selected page, then `LEFT JOIN` one exact-version cache alias in the final select.
- Make global Search Club DNA filter/sort and Squad sort cache-first after complete materialization. Search filter uses the v32 score index for exact-version count and candidate selection, then may use the required temporary sort for final CA/UID order. Search and Squad Club DNA sorts use the score index for exact score selection and order; only their documented null-last and UID tie-break work may use a temporary sort. Complete nullable-row materialization permits an `INNER JOIN` without dropping unavailable scores.
- Before UID enumeration or a materializer call, treat exact-version cache-row count equality with the authoritative cohort count as completeness. Search counts exact-version snapshot rows against snapshot players. Squad counts exact-version cache rows joined to the exact managed-club predicate against that same cohort. Nullable rows count; outside-club rows do not. Primary-key and foreign-key constraints prevent duplicate or orphan false matches.
- On count equality, skip UID allocation, the materializer, and every per-batch miss probe. On mismatch, run the existing resumable materializer. A Search addition or Squad entry without an exact current-version row mismatches; an entrant already materialized by Search can preserve Squad equality. Search removal cascades both counts. A Squad exit drops the player from both predicates. A managed-club switch to an already complete cohort can preserve equality. Reuse the cohort count only when the statement uses the same validated snapshot and cohort predicate.
- Remove correlated definition lookups from Club DNA display, filter, and sort SQL. Interpolate only fixed aliases and validated directions; bind versions, model values, filter values, page values, and cohort values.
- Missing definition performs no materialization, cache join, or writes. Display projects null, Club DNA sort stays player-first with every player retained and UID-stable all-null order, and each Club DNA rule compiles to SQL false inside the existing flat AST. Isolated and mixed AND Club DNA filters return empty; mixed OR retains non-Club-DNA matches. Never start a cache-first global plan from an empty cache.
- Do not add a durable completeness marker, migration, index, new ADR, background job, progress/cancellation design, or frontend behavior change for this correction.
- Keep `club_dna` permanently valid in both catalogs. Let the backend report create versus replace, consolidate frontend catalogs in the existing frontend packet, and keep all explanation inside the Modal.

### Unknowns

- Native Tauri/WebView density and focus behavior for selecting the full catalog cannot be proved in headless Chromium.
- No performance unknown blocks Commit 8: the discarded exact-count/indexed six-path implementation supplied complete 2,000- and 184,000-player evidence, and replacement work must reproduce the same shapes from clean HEAD.

### Risks

- A cache-first query that omits either version can return stale scores after definition or formula changes. Resolver, filter, Search, Squad, and EXPLAIN tests must prove exact bound identity.
- A global query can return an incomplete or wrong total if it runs before materialization completes or drives from players instead of the exact-version score index. Search and Squad tests must prove completeness-check-or-materialize-before-count/order/page behavior.
- A completeness count can false-pass if it includes stale versions, counts cache rows outside the exact Squad cohort, fails to count nullable rows, or reuses a cohort count across different predicates. Separate tests must prove that additions and club entries without exact current-version rows mismatch and materialize; player removals cascade both Search counts; club exits leave both Squad predicates equal; entrants already materialized by Search can leave Squad complete; and a switch to an already complete club can skip safely. Exact-version, nullable, partial, boost, definition, outside-club, primary-key, and foreign-key tests must also protect this shortcut.
- Warm requests can remain representative-scale if they allocate every UID or invoke even empty materializer batches after completeness is known. Instrumented integration tests must prove zero UID enumeration, materializer calls, and batch probes on the complete-cache path.
- Reusing a player-correlated scalar expression, even with cached scores, repeats snapshot/definition/cache lookup at representative scale. Review must reject every correlated definition lookup and require one request context plus one cache alias per statement.
- `INNER JOIN` is correct only after a present definition and complete materialization include nullable rows. Missing definition must branch before any materialization or cache-first join. Partial-cache, stale-version, page-scope, and flat AND/OR tests must catch dropped players, false matches, or loss of non-Club-DNA OR matches.
- Dynamic SQL can weaken the trusted boundary if aliases, directions, or values come from the request. Only fixed aliases and validated directions may be interpolated; all values stay bound.
- A benchmark helper that returns on the first shape failure can hide later breaches in the same size-specific test. Each benchmark test must execute and report Search filter, Search sort, and Squad sort before it asserts that test's size threshold. The normal and ignored commands together cover six combinations; no assertion aggregates results across the two processes. The representative risk is regression above the shared `<=1,000 ms` warm limit, especially Squad sort at measured p95 856.517 ms.
- Cold materialization still holds the existing command-level `Db` mutex while bounded batches run. The measured 184,000-player cold durations of 2496.594–2904.600 ms are delayed first-use evidence, separate from the warm gate. This correction does not add background work or change UX.
- The same app-local layouts serve every save. A save with no definition must return null for display/sort without cache joins or writes. Each Club DNA filter rule must be false in the flat AST, so isolated and mixed AND filters are empty while mixed OR keeps non-Club-DNA matches.
- Invalidation alone cannot bind a late definition response to the requesting save. All definition IPC calls, Query keys, route effects, and Rust checks retain the exact save ID/context-token contract.
- Automatic append can override user customization if edit is mistaken for create. Backend `created`, store tests, and route tests must distinguish create, edit, delete, and re-create.

## Walking skeleton

Committed v31/v32 persistence and materialization remain the foundation. Commit 6 records the bounded completeness correction at clean HEAD `8250dbe9aac7853ed90ba674f83a67da870a8ecb`. Commit 7 records the accepted representative latency decision in this ledger and ADR-0023. Commit 8 then implements one request-scoped trusted definition/model context, page-first display with one exact-version `LEFT JOIN`, cache-first global Search and Squad operations driven by the v32 score index, and the exact-count warm-cache fast path before UID enumeration. Commits 9–11 deliver the unchanged frontend metric, Modal, and My Club integration.

## Delivery plan

### PR 1 — Add user-defined Club DNA scoring

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/club-dna

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** GitHub required strict status `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(club-dna): add user-defined scoring`

**Purpose:** Deliver the complete save-owned definition, trusted score derivation, configurable table metric, and My Club form in one review surface. No independent trunk boundary justifies a second PR because the persistence, metric, and UI parts have no user value alone.

**Depends on:** Synchronized `main` at `b573420893da93d91ddaee66ff9a4038f800b6d9`; Linear JAY-32 product decisions recorded in this ledger. No earlier PR or planned spec.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(club-dna): record approved feature plan`

**Work:** Commit the independently reviewed Club DNA ledger and TODO activation on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, BACKLOG, ADRs, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch and base verification.

**Files and responsibilities:**

- `.wiki/features/active/club-dna.md` — approved intent, architecture, delivery sequence, execution packets, risks, and validation contract.
- `.wiki/TODO.md` — replace `Active: None` with the active Club DNA ledger link.

**Behavior and data flow:**

- Record one active source of feature truth and its exact one-PR commit sequence before implementation. There is no planned spec, BACKLOG disposition, or warranted ADR to include.

**Ordered implementation steps:**

1. Verify `feature/club-dna` is based on synchronized `main` at the accepted base without changing plan scope.
2. Confirm the worktree contains only the two independently reviewed planning paths.
3. Run the active-ledger classifier.
4. Stage and inspect only those two paths for the normal independent checkpoint review.

**Tests and proof:**

- Not applicable — independently reviewed planning documents only. `ledger_state.py` proves schema and classifier consistency. No test fixtures, mocks, snapshots, or helpers change.

**Patterns to verify:**

- `.wiki/features/active/README.md` schema 2 template and `.wiki/TODO.md` feature-level ownership.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, BACKLOG, current-state architecture, plan scope, packet order, or delivery authority.

**Dependencies and sequencing:**

- Requires an independent plan-review verdict, developer acceptance, a recorded Delivery fingerprint, and exact branch activation through the delivery workflow.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/club-dna.md`

**Stop conditions:** Stop on an uncleared plan review, classifier error, unreviewed path, substantive post-review plan change, missing fingerprint, base mismatch, or branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome, exactly the ledger and TODO paths, no implementation, and no unapproved documentation.

#### Commit 2 — Persist one save-owned Club DNA definition

**Status:** Completed

**Provisional commit:** `feat(club-dna): persist save-owned definition`

**Work:** Add migration v31 and the authoritative Rust CRUD/IPC boundary for one validated definition per save.

**Size assessment:** About 160–200 changed non-test implementation lines. Within the soft target; tests and migration characterization are excluded from the count.

**Out of scope:**

- Score calculation, player-metric resolution, Search/Squad query changes, React adapters, UI, layout mutation, caches, and current-state documentation.

**Implementation packet:**

- Add one save-keyed definition row and closed-catalog validation. Return enough mutation state for a later React packet to distinguish create from edit without relying on cached frontend state.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — add v31 `club_dna_definitions(save_id PRIMARY KEY REFERENCES saves(id) ON DELETE CASCADE, attribute_ids_json TEXT NOT NULL)` with JSON-array, non-empty-array, and valid-JSON shape constraints; register v31 and update exact version/schema tests.
- `src-tauri/src/features/club_dna/mod.rs` — register the Rust feature module.
- `src-tauri/src/features/club_dna/service.rs` — define canonical ID validation, duplicate rejection, and context-bound get, upsert, and remove operations; require the expected save ID and immutable context token to match the still-active save in the same database operation; expose a typed definition and an upsert result with `created`.
- `src-tauri/src/features/club_dna/commands.rs` — thin `get_club_dna`, `set_club_dna`, and `remove_club_dna` Tauri commands whose camelCase requests require `saveId` and `contextToken`, with safe stale-context errors.
- `src-tauri/src/features/mod.rs` — register `club_dna`.
- `src-tauri/src/lib.rs` — register the three commands.
- `src-tauri/src/features/snapshot/service.rs` tests — include `club_dna_definitions` in the existing save-deletion cascade portfolio if that portfolio enumerates save-owned tables.

**Behavior and data flow:**

- The WebView sends the active save ID and immutable save context token with every definition request. Before get reads or set/remove writes, Rust verifies that both values still identify the row with `is_active = 1`; a save switch, deletion, or ID reuse therefore produces a safe stale-context error and no read or mutation. Set then validates the non-empty attribute list against the exact visible/goalkeeper, hidden, and personality catalogs, rejects unknown or duplicate IDs, serializes the canonical ordered list, and upserts only the verified active save's row. Get returns that row's ordered list or no definition. Remove deletes only that verified row and reports whether it existed. Save deletion cascades the row; snapshot creation, replacement, promotion, and deletion do not touch it.

**Ordered implementation steps:**

1. Add RED migration/service tests for fresh v31 shape, one-row-per-save isolation, required save-context input, stale-context rejection after an active-save switch, all-catalog acceptance, empty/unknown/duplicate rejection, create-versus-replace result, restart persistence, snapshot independence, remove, and save cascade.
2. Add the v31 table and update registry/version assertions without backfilling a definition.
3. Implement the smallest typed service operations. Match `save_id`, immutable `context_token`, and `is_active = 1` before any read or write, and reuse the existing Rust attribute catalogs rather than accepting frontend labels or arbitrary keys.
4. Add thin context-bearing commands and registration.
5. Add the definition table to the existing save-deletion preservation/cascade proof where applicable.
6. Run Rust validation, then the full gate.

**Tests and proof:**

- RED: new tests in `src-tauri/src/db/migrations.rs` and `src-tauri/src/features/club_dna/service.rs` fail because v31 and the service do not exist. The migration test must fail if the row is not save-owned or permits an empty JSON array. Service/command tests must fail if one save can read another save's definition, if a request with the old active save ID/token can read, set, or remove after switching saves, if stale set/remove changes either save's row, or if edit reports `created: true`.
- GREEN: prove all supported IDs, including goalkeeper, hidden, and personality examples, survive an upsert/read/reopen cycle in order; reject empty, unknown, and duplicate input; prove edit replaces the complete list; prove current-context remove is idempotent and reports existence; prove stale-context get/set/remove return safe errors with no mutation; prove snapshot changes retain the row and save deletion cascades it.
- Add/modify: migration and service unit/integration tests. Modify the existing snapshot save-cascade table list only if required. Deliberately retain existing managed-club, save, snapshot, and migration fixtures because they protect adjacent persisted contracts. Delete no fixtures, mocks, snapshots, helpers, or compatibility paths.

**Patterns to verify:**

- `src-tauri/src/features/managed_club/{service.rs,commands.rs}` for one-row-per-save CRUD and active-save commands.
- `src-tauri/src/db/migrations.rs::all`, `latest_version`, fresh-database tests, and save-owned `ON DELETE CASCADE` tables.
- `src-tauri/src/features/player_metrics/resolver.rs::{DUMP_ATTRIBUTE_KEYS,HIDDEN_ATTRIBUTE_KEYS,PERSONALITY_KEYS}` for the authoritative closed keys.

**Constraints and non-goals:**

- Persist canonical IDs only. Do not persist label, weights, club name, snapshot ID, score, or formula metadata.
- Do not create default rows for existing or new saves. Absence is valid.
- Keep validation and active-save context verification authoritative in Rust and errors safe for IPC. Do not rely on React invalidation or callback guards to protect persistence.
- Do not add a dependency, ADR, cache, or frontend behavior.

**Dependencies and sequencing:**

- Depends only on Commit 1. It establishes the trusted definition boundary required by score derivation and UI packets.

**Validation:** `./scripts/dev check-rust` then `./scripts/dev check`

**Stop conditions:** Stop if the existing Rust catalogs do not cover every approved frontend attribute, if a valid definition cannot be represented without another persisted contract, if get/set/remove cannot verify the expected save ID and immutable token against the still-active save before access, if snapshot replacement currently rewrites save-owned settings, if save cascade cannot be proved, or if an architecture decision beyond established save-owned persistence is required.

**Review mandate:** Verify (1) one row per save and cascade ownership, (2) mandatory save ID/token on all three commands and authoritative still-active verification, (3) no snapshot or club-name coupling, (4) all approved catalogs with non-empty/duplicate/unknown rejection, (5) create-versus-edit accuracy, (6) stale get/set/remove cannot read or mutate either save and errors stay safe, (7) no backfill or derived score persistence, and (8) migration, switch, and deletion tests protect realistic wrong-save and data-loss paths.

#### Commit 3 — Record the approved cache replan

**Status:** Completed

**Provisional commit:** `docs(club-dna): record lazy cache replan`

**Work:** Commit the independently reviewed bounded replan and accepted ADR before cache implementation resumes.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, TODO, BACKLOG, current-state architecture, design documentation, and unrelated decisions.

**Implementation packet:**

- Preserve completed Commits 1–2 and record only the approved lazy-cache architecture, failed direct-SQL evidence, replacement packets, invalidated prior fingerprint, and ADR index entry. Before delivery resumes, record the fingerprint that independent review clears and the developer accepts.

**Files and responsibilities:**

- `.wiki/features/active/club-dna.md` — bounded replan, measured stop evidence, delivery packets 3–8, validation, and immutable completed refs.
- `.wiki/decisions/0023-lazy-club-dna-score-cache.md` — accepted cache decision, alternatives, consequences, owners, and thresholds.
- `.wiki/decisions/README.md` — index ADR-0023.
- `.wiki/TODO.md` — deliberately unchanged because its active Club DNA link remains correct.

**Behavior and data flow:**

- Replace the invalid direct-SQL delivery packet with one reviewed planning commit, one cache foundation commit, and one cached query integration commit. Keep the existing frontend product behavior and one-PR authority unchanged. The planning diff remains pre-acceptance evidence while the Delivery fingerprint is pending. After independent review clears the correction and the developer accepts the plan, record the accepted fingerprint before this artifact is delivered or implementation resumes.

**Ordered implementation steps:**

1. Verify branch `feature/club-dna`, clean starting HEAD `d2682ee5c50cb99cd0b7f9facf5fd4f9060d5001`, and completed refs without changing Git state.
2. Confirm that only the ledger, ADR-0023, and ADR index changed and that `.wiki/TODO.md` is unchanged.
3. Before acceptance, run `ledger_state.py`, `delivery_state.py`, and Markdown/diff checks. Treat the pending-fingerprint delivery failure as review evidence, not execution authority.
4. Submit the exact planning diff for independent plan review. After review clears and the developer accepts the plan, record the accepted Delivery fingerprint.
5. Rerun both classifiers and require a valid delivery state. Stage only the three reviewed planning paths, run `git diff --cached --check`, and inspect the complete cached diff for all three paths before checkpoint review.

**Tests and proof:**

- Not applicable — planning documents only. Before acceptance, `ledger_state.py` proves schema and ledger-state consistency; only `delivery_state.py` reports that the pending fingerprint is invalid. After the accepted fingerprint is recorded, both classifiers must pass before delivery resumes. Markdown and cached-diff checks prove formatting and exact three-path scope. No fixtures, mocks, snapshots, helpers, or compatibility paths change.

**Patterns to verify:**

- `.wiki/features/active/README.md` schema 2, ADR-0019's cache rationale and structure, and `.wiki/decisions/README.md` indexing style.

**Constraints and non-goals:**

- Preserve product intent, completed Git refs, one PR, branch/base, provider, title, merge method, checks, and publication boundaries.
- Do not resume delivery with a pending, unreviewed, or unaccepted Delivery fingerprint. Do not change `.wiki/ARCHITECTURE.md`, `.wiki/DESIGN.md`, `.wiki/TODO.md`, implementation, or tests.

**Dependencies and sequencing:**

- Depends on the developer's explicit lazy-cache decision and clean completed Commit 2 HEAD. Independent plan review must clear this material replan before implementation resumes.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/club-dna.md`; `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/club-dna.md .`; `git diff --cached --check`; `git diff --cached -- .wiki/features/active/club-dna.md .wiki/decisions/0023-lazy-club-dna-score-cache.md .wiki/decisions/README.md`; `git status --short`

**Stop conditions:** Stop on a branch/HEAD mismatch, changed implementation or tests, changed TODO content, missing completed ref, classifier schema error, a pending or invalid fingerprint at execution time, uncleared independent review, missing developer acceptance, or any substantive architecture or authority change.

**Review mandate:** Verify (1) exact measured samples and nearest-rank p95 values, (2) accepted cache identity/scopes/invalidation, (3) completed refs and one-PR authority remain exact, (4) exactly Commit 3 is Active, (5) packets 4–8 are execution-ready and retain product behavior, (6) the pending fingerprint is only pre-acceptance review evidence and delivery requires the independently reviewed, developer-accepted value, (7) ADR-0023 matches the ledger, and (8) no unapproved path changed.

#### Commit 4 — Add the versioned Club DNA score cache

**Status:** Completed

**Provisional commit:** `feat(club-dna): add lazy score cache`

**Work:** Add migration v32, definition versioning and invalidation, pure scoring, bounded lazy materialization, and supported player-boost invalidation.

**Size assessment:** About 220–300 changed non-test implementation lines. This may exceed the soft target because schema identity, atomic invalidation, pure scoring, materialization, and boost reconciliation form one persisted consistency boundary; query exposure remains in Commit 5.

**Out of scope:**

- Search filter/sort SQL, Squad query resolution, frontend code, eager ingest/promotion work, background jobs, global process locks, current-state documentation, Moneyball, and Player Profile display.

**Implementation packet:**

- Follow ADR-0019's disposable-cache pattern with Club DNA-specific definition versioning. Keep v31 immutable. Add one pure formula owner and materialize requested UIDs in bounded batches while every cache read/write identity includes both versions.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — add v32 without modifying v31; add positive `definition_version` to `club_dna_definitions`; add nullable `club_dna_scores(snapshot_id, uid, definition_version, score_model_version, score)` with `CHECK (score IS NULL OR score BETWEEN 0 AND 100)`; add composite player ownership with `ON DELETE CASCADE`, unique identity `(snapshot_id, uid, definition_version, score_model_version)`, and an index ordered for snapshot/version/score lookup; update fresh and v31-upgrade schema/version/index/cascade/score-domain tests.
- `src-tauri/src/features/club_dna/service.rs` — return definition version internally; atomically increment version and delete all save-owned cache rows on edit; atomically delete rows and definition on remove; preserve `created`; ensure re-create starts a new definition; add rollback and no-cross-save tests.
- `src-tauri/src/features/player_metrics/club_dna.rs` — own `SCORE_MODEL_VERSION`, pure closed-catalog JSON scoring, request validation, cache-miss lookup, bounded player loading, nullable scoring, and idempotent batch persistence under the caller's existing synchronous `Db` mutex guard.
- `src-tauri/src/features/player_metrics/mod.rs` — register the Club DNA metric owner.
- `src-tauri/src/features/player_metrics/potential_cache.rs` — retain existing behavior; reuse its bounded transaction pattern rather than coupling the caches.
- `src-tauri/src/features/player/service.rs::reconcile_verified_boost` — invalidate the reconciled player's Club DNA cache rows in the same transaction as player and potential-cache reconciliation.

**Behavior and data flow:**

- A caller supplies a validated snapshot and bounded UID cohort. The materializer reads the save-owned definition once, validates every canonical ID before writes, and returns without work when no definition exists. It loads missing/stale players in bounded batches and requires every selected JSON value to be an integer from 1 through 20. A missing key, explicit null, non-integer, 0, 21, or any other out-of-domain value makes the whole score null. For a valid complete player it calculates `round((sum(value × 5)) / n)` in pure Rust with equal weighting and one rounding step. It stores a row even when the score is null and commits one batch at a time. Existing matching rows are reused. Stale definition/model rows are misses.
- Definition edit validates the replacement, increments `definition_version`, deletes all cache rows whose snapshots belong to that save, and updates the definition in one transaction. Remove deletes the cache rows and definition in one transaction. Re-create inserts a new definition version lineage and still returns `created: true`. Snapshot/player deletion cascades rows. Successful player boost reconciliation deletes rows for only that snapshot/player before commit. Ingest and promotion write no Club DNA scores.
- Request validation and definition validation finish before the first cache transaction. A batch failure rolls back only that batch; prior derived batches remain safe and later work can resume. The command retains the established `Db` mutex for the synchronous materialization and dependent query, but no SQLite write transaction spans calculation of a different batch and no lock crosses an async wait. Do not add another feature-wide lock.

**Ordered implementation steps:**

1. Add RED v31→v32 migration tests for new definition version, exact cache columns/key/index, `score IS NULL OR score BETWEEN 0 AND 100`, accepted null/0/100 cache rows, rejected out-of-range cache rows, player/snapshot cascade, and preservation of v31 definition data.
2. Add RED pure-scoring tests for valid 1 and 20 boundaries, tie rounding, mixed visible/goalkeeper/hidden/personality sources, missing key, explicit null, non-integer value, and deterministic equal weighting. Prove that selected 0 and 21 values each make the whole score null in `attributes_json`, `hidden_attributes_json`, and `personality_json`; include both visible and goalkeeper examples in the shared `attributes_json` source.
3. Add RED materializer tests for no-definition/no-write, invalid request/no-write, page UID scope, complete nullable rows, matching-version reuse, stale model/definition replacement, bounded multi-batch commits, resumable partial derived state, and idempotent repeat.
4. Implement v32 and the minimum version-aware definition changes without rewriting v31.
5. Implement pure scoring and bounded cache materialization. Validate before writes; compute outside each short batch transaction; use exact version identity in lookup and upsert.
6. Add edit/remove/re-create atomic invalidation and rollback proofs, including another save's retained rows.
7. Extend supported player-boost reconciliation and its existing potential-cache test to prove both caches invalidate in the same successful transaction and both survive reconciliation rollback.
8. Prove ingest and current-snapshot promotion remain lazy, then run Rust and full gates.

**Tests and proof:**

- RED: migration tests fail at version 31; no Club DNA cache owner exists; edit leaves no version/invalidation contract; a seeded Club DNA row survives a supported boost.
- GREEN: upgrading a v31 database preserves the definition at a positive starting version; fresh and upgraded schemas accept only null or 0–100 cache scores; the scorer accepts selected 1 and 20 values but returns null for missing, null, non-integer, 0, 21, or other out-of-domain selected values; exact matching-version rows are reused; null scores are persisted; stale versions are not returned; edits/removes/boosts clear only owned rows atomically; re-create reports creation; snapshot/player cascade works; no definition or invalid request writes zero rows.
- Batch/lock proof: use a fixture larger than one materialization batch. Prove every UID receives exactly one current-version row, injected batch failure does not expose a complete cohort, committed prior batches remain disposable/resumable, and no SQLite transaction spans scoring of the next batch. Review that the synchronous command retains only the established `Db` mutex through materialization and its dependent query, with no async wait or second feature lock; do not infer lock safety from elapsed time alone.
- Add/modify: v32 migration tests, Club DNA service tests, new pure-scoring/materializer tests, and player boost reconciliation tests. Deliberately retain v31 definition tests, potential-cache tests, snapshot ingest/promotion tests, role-score tests, and boost recovery fixtures because they protect adjacent persisted and rollback contracts. Delete only direct-SQL experiment test helpers if any survived; the developer states the experiment was discarded cleanly.

**Patterns to verify:**

- `PLAYER_POTENTIAL_ROLE_SCORES_SQL`, `player_metrics::potential_cache::{materialize_snapshot_roles,materialize_player_roles,persist_scores,invalidate_player_cache}`, and ADR-0019.
- `club_dna::service::{set_club_dna,remove_club_dna}` for current transaction boundaries.
- `player::service::reconcile_verified_boost` for same-transaction derived-cache invalidation.
- Fresh migration and exact registry/index tests in `src-tauri/src/db/migrations.rs`.

**Constraints and non-goals:**

- Do not change committed migration v31. Do not backfill scores or compute during ingest/promotion.
- Cache data is nullable, versioned, derived, disposable, constrained to null or 0–100, and never an authoritative definition or player value.
- Never accept JSON paths, SQL, formula, or versions from React. Keep score/model version constants in Rust.
- Do not add a dependency, background worker, cancellation framework, progress IPC, or lock beyond the established command-level `Db` mutex in this packet.

**Dependencies and sequencing:**

- Depends on completed Commit 2 and reviewed Commit 3/ADR-0023. Commit 5 is the only consumer that may expose cached scores to Search and Squad.

**Validation:** `./scripts/dev check-rust` then `./scripts/dev check`

**Stop conditions:** Stop if v32 cannot preserve existing v31 definitions, exact cache identity needs another durable field, save-owned invalidation cannot be atomic, player boost invalidation cannot share reconciliation rollback, scoring requires untrusted dynamic SQL/JSON paths, a SQLite write transaction must span an unbounded cohort, the established synchronous `Db` mutex cannot protect materialization plus the dependent global query, or correctness requires an unapproved job/lock architecture.

**Review mandate:** Verify (1) v31 is immutable and v32 upgrades safely, (2) exact four-part identity/index, nullable cascade ownership, and the null-or-0–100 schema check, (3) pure formula semantics require every selected value to be integer 1–20 across all three JSON sources, with 1/20 valid and 0/21 null proofs, equal weighting, and one rounding step, (4) short batch transactions under only the established synchronous `Db` mutex with safe resumability and no async wait, (5) atomic edit/remove/re-create ownership, (6) model-version misses, (7) same-transaction player-boost invalidation and rollback, and (8) no ingest/eager/background or query-exposure scope creep.

#### Commit 5 — Record the indexed query-shape correction

**Status:** Completed

**Provisional commit:** `docs(club-dna): record indexed query correction`

**Work:** Commit this independently reviewed planning correction and status-only ADR reconciliation after the representative warm Search filter breach invalidated the prior Commit 5 packet.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- ADR-0023's decision, alternatives, consequences, thresholds, and index; TODO, BACKLOG, ARCHITECTURE, DESIGN, implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve completed Commits 1–4 and replace only the disproved cached query packet, affected risks, evidence, delivery order, and validation contract. Reconcile only ADR-0023's stale implementation-status sentence with committed v31/v32. After independent review clears the exact two-path planning diff, let the orchestrator obtain and record the classifier's exact replacement Delivery fingerprint, rerun both classifiers, and present that exact fingerprint for developer acceptance.

**Files and responsibilities:**

- `.wiki/features/active/club-dna.md` — record the measured representative breach, confirmed correlated-expression cause, bounded indexed query-shape decision, replacement Commit 6 packet, renumbered unchanged frontend packets, completed Commit 4 ref, and pending delivery authorization.
- `.wiki/decisions/0023-lazy-club-dna-score-cache.md` — update only the implementation-status sentence to state that the v31/v32 cache foundation is implemented at `d78f97f25497409f6c895a8ac5cdeb74ea5301eb` and indexed Search/Squad integration remains pending Commit 6.

**Behavior and data flow:**

- Keep all accepted persistence, cache, invalidation, UX, product scope, PR authority, and publication behavior unchanged. Replace the invalidated implementation packet with one request-scoped trusted definition/model context, page-first display SQL, and cache-first global SQL that uses the committed v32 score index. The ledger owns the correction; ADR-0023 receives only a current implementation-status reconciliation.

**Ordered implementation steps:**

1. Verify branch `feature/club-dna`, starting HEAD `d78f97f25497409f6c895a8ac5cdeb74ea5301eb`, and completed refs without changing Git state.
2. Confirm the failed Commit 5 implementation worktree is absent and the planning diff changes only `.wiki/features/active/club-dna.md` plus the implementation-status sentence in `.wiki/decisions/0023-lazy-club-dna-score-cache.md`.
3. Run Markdown and exact two-path diff/status checks, then submit that complete planning diff for independent plan review. The plan review must clear before fingerprint generation.
4. After review clears, run `ledger_state.py`, then `delivery_state.py`. Record the exact classifier-returned Delivery fingerprint without changing any packet or authority input.
5. Rerun both classifiers with that recorded value and require both to pass. Present the reviewed planning diff and that exact fingerprint for developer acceptance. Do not hardcode or reuse the invalidated pre-correction fingerprint.
6. Delivery starts with exactly the reviewed two-path planning diff still uncommitted. Stop if the worktree contains extra paths or if either reviewed path differs from the accepted diff; do not stop merely because this exact reviewed planning diff makes the worktree dirty.
7. During delivery, stage and inspect only those two reviewed paths for the normal independent checkpoint review.

**Tests and proof:**

- Not applicable — planning documents only. Before fingerprint recording, `ledger_state.py` proves schema, exactly one Active commit, completed evidence, and packet completeness; `delivery_state.py` may be invalid only because the fingerprint is pending. After clear review, recording the exact returned fingerprint and rerunning both classifiers must produce valid states before acceptance. `git diff --check`, the exact two-path diff, and `git status --short` prove Markdown and path scope. No fixtures, mocks, snapshots, helpers, compatibility paths, ADR decision content, or other documents change.

**Patterns to verify:**

- `.wiki/features/active/README.md` schema 2; the completed cache replan in Commit 3; committed v32 cache ownership in Commit 4; and ADR-0023's unchanged decision, lazy-cache boundary, index, and thresholds.

**Constraints and non-goals:**

- Preserve the exact one PR, branch, base, provider, template, title, merge method, required check, close-out state, and CI repair count.
- Preserve all completed refs and evidence. Do not compute or record a replacement Delivery fingerprint in this correction pass.
- Change only ADR-0023's implementation-status sentence. Do not edit its decision, alternatives, consequences, thresholds, or index. Do not edit TODO, BACKLOG, ARCHITECTURE, DESIGN, code, tests, scripts, configuration, or any other path.

**Dependencies and sequencing:**

- Depends on completed Commit 4 at `d78f97f25497409f6c895a8ac5cdeb74ea5301eb`, the discarded failed Commit 5 worktree, the supplied performance evidence, the confirmed query-shape root cause, and the developer's explicit bounded correction decision. Commit 6 requires this reviewed planning artifact and a newly accepted Delivery fingerprint.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/club-dna.md`; `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/club-dna.md .`; `git diff --check -- .wiki/features/active/club-dna.md .wiki/decisions/0023-lazy-club-dna-score-cache.md`; `git diff -- .wiki/features/active/club-dna.md .wiki/decisions/0023-lazy-club-dna-score-cache.md`; `git status --short`

**Stop conditions:** Stop on a branch/HEAD mismatch, missing completed ref, an extra changed path, either reviewed planning path differing from the accepted diff, any classifier error beyond the pre-acceptance pending fingerprint, an uncleared independent review, missing developer acceptance, or any requested change to ADR-0023's decision/thresholds/index, v32 schema/index, cache scopes, invalidation, versions, UX, product behavior, or delivery authority. The exact reviewed two-path planning diff is expected to remain uncommitted and is not itself a stop condition.

**Review mandate:** Verify (1) exact supplied 2,000- and 184,000-player evidence, (2) confirmed repeated correlated-expression cause, (3) bounded cache-first correction with one request context and no migration/index/ADR-decision change, (4) immutable completed refs including Commit 4, (5) exactly Commit 5 is Active and Commit 6 is execution-ready, (6) Commits 7–9 change only by renumbering and dependencies, (7) plan review precedes fingerprint recording and the exact recorded classifier value precedes developer acceptance, and (8) only the ledger plus ADR-0023's implementation-status sentence change.

#### Commit 6 — Record the warm-cache completeness correction

**Status:** Completed

**Provisional commit:** `docs(club-dna): record warm-cache correction`

**Work:** Commit this independently reviewed ledger-only bounded replan after the indexed Commit 6 attempt proved that warm requests still scan the complete cohort for cache misses.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- ADR-0023's decision, alternatives, consequences, thresholds, and all content except its implementation-status sentence; TODO, BACKLOG, ARCHITECTURE, DESIGN, implementation, tests, executable configuration, generated files, migrations, indexes, and unrelated documentation.

**Implementation packet:**

- Preserve completed Commits 1–5 and their immutable refs. Record the exact all-shape benchmark evidence, confirmed warm completeness root cause, exact-count fast-path decision, replacement Commit 7 packet, renumbered unchanged frontend packets, invalidated fingerprint, and pending-review state. Do not compute or record a replacement Delivery fingerprint in this planning pass.

**Files and responsibilities:**

- `.wiki/features/active/club-dna.md` — record the bounded correction, benchmark and diagnostic evidence, delivery packets 6–10, completed Commit 5 ref, final validation, risks, dependencies, and pending packet fingerprints.
- `.wiki/decisions/0023-lazy-club-dna-score-cache.md` — update only the implementation-status sentence to remove the fragile commit ordinal while preserving the accepted decision and thresholds.

**Behavior and data flow:**

- Keep v32, ADR-0023, indexed query architecture, UX, cache scopes, versions, thresholds, formula, invalidation, product behavior, one-PR boundary, and every publication authority field unchanged. Add only the reviewed plan for an exact-version cohort-count completeness check before global UID enumeration and materializer calls. The ledger remains non-runnable while its Delivery fingerprint is `Pending review`.

**Ordered implementation steps:**

1. Verify branch `feature/club-dna`, clean starting HEAD `df074b9da78bec038960e2be7c851dcb5879dbdd`, and completed refs 1–5 without changing Git state.
2. Confirm the failed Commit 6 worktree was discarded and only `.wiki/features/active/club-dna.md` plus ADR-0023's implementation-status sentence changed.
3. Record the exact indexed 2,000- and 184,000-player all-shape timings, diagnostic medians, confirmed completeness root cause, bounded decision, replacement Commit 7 proof, unchanged frontend packets as Commits 8–10, and invalidated `4917d5fd65279b9390c2fac5fd37448561996367b7e4a41c129a1868a16cc03a` authorization. Reconcile ADR-0023's implementation status without a commit ordinal.
4. Run the ledger classifier, delivery classifier, Markdown check, exact two-path diff check, and status check. The delivery classifier must fail only because review and a replacement fingerprint remain pending.
5. Submit the exact two-path planning diff for independent plan review. Do not generate, compute, or record a replacement Delivery fingerprint during this packet.
6. After a later authorized review-and-acceptance step records the classifier-returned fingerprint, rerun both classifiers before delivery resumes.

**Tests and proof:**

- Not applicable — planning documents only. `ledger_state.py` must prove schema 2, exactly one Active commit, completed evidence, and packet completeness. `delivery_state.py` must classify the replan as non-runnable while the fingerprint is pending. `git diff --check`, the exact two-path diff, and `git status --short` prove Markdown and path scope. No tests, fixtures, mocks, snapshots, helpers, compatibility paths, ADR decision content, or other documents change.

**Patterns to verify:**

- `.wiki/features/active/README.md` schema 2; completed Commit 5 at `df074b9da78bec038960e2be7c851dcb5879dbdd`; ADR-0023's unchanged v32 decision, scopes, versions, thresholds, and index; and the discarded Commit 6 evidence supplied for this bounded replan.

**Constraints and non-goals:**

- Preserve the exact one PR, branch, base, provider, template, title, merge method, required check, feature close-out state, and CI repair count.
- Preserve all completed refs and evidence. Change only ADR-0023's implementation-status sentence outside this ledger. Do not compute or record a replacement Delivery fingerprint.
- Do not change v32, cache identity, indexed read architecture, materialization scopes, invalidation, versions, UX, thresholds, formula, frontend behavior, or product behavior.

**Dependencies and sequencing:**

- Depends on completed Commit 5 at `df074b9da78bec038960e2be7c851dcb5879dbdd`, the discarded failed Commit 6 worktree, the supplied all-shape performance evidence and diagnostic medians, the confirmed warm completeness root cause, and the developer's explicit bounded decision. Commit 7 requires this reviewed and later accepted planning artifact.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/club-dna.md`; `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/club-dna.md .`; `git diff --check -- .wiki/features/active/club-dna.md .wiki/decisions/0023-lazy-club-dna-score-cache.md`; `git diff -- .wiki/features/active/club-dna.md .wiki/decisions/0023-lazy-club-dna-score-cache.md`; `git status --short`

**Stop conditions:** Stop on a branch/HEAD mismatch, missing completed ref, a changed path outside the ledger and ADR-0023, any ADR-0023 change beyond its implementation-status sentence, any classifier error beyond pending review/fingerprint authorization, a changed authority field, a request to edit code/tests or other documentation, or any change to v32, indexed query architecture, UX, scopes, versions, thresholds, or product behavior.

**Review mandate:** Verify (1) exact all-shape 2,000- and 184,000-player evidence, (2) exact diagnostic medians are labeled non-acceptance evidence, (3) confirmed warm UID-enumeration and per-250 miss-probe cause, (4) exact-version Search and exact-predicate Squad completeness semantics, including every equality-preserving cohort transition, nullable rows, and outside-club rows, (5) immutable completed refs 1–5 and clean HEAD, (6) exactly Commit 6 is Active and Commit 7 is execution-ready, (7) Commits 8–10 are unchanged except renumbering and dependencies, and (8) only the ledger and ADR-0023's implementation-status sentence changed while the Delivery fingerprint remains pending review.

#### Commit 7 — Record the representative latency decision

**Status:** Completed

**Provisional commit:** `docs(club-dna): record representative latency decision`

**Work:** Commit the independently reviewed bounded threshold replan and ADR-0023 reconciliation after the discarded exact-count/indexed implementation met the developer-chosen representative warm contract.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, migrations, indexes, TODO, BACKLOG, ARCHITECTURE, DESIGN, new ADRs, and unrelated documentation.

**Implementation packet:**

- Preserve completed Commits 1–6 and resolve Commit 6 to clean HEAD `8250dbe9aac7853ed90ba674f83a67da870a8ecb`. Record the exact six benchmark measurements, remaining large exact-query sort/join cost, developer rejection of v33 indexes, accepted `<=1,000 ms` representative warm threshold, replacement Commit 8 packet, renumbered unchanged frontend packets, and invalidated delivery fingerprint. Do not compute or record a replacement Delivery fingerprint in this planning pass.

**Files and responsibilities:**

- `.wiki/features/active/club-dna.md` — record the accepted product and operational contract, evidence, decisions, risks, delivery packets 7–11, completed Commit 6 ref, final validation, dependencies, and pending authorization.
- `.wiki/decisions/0023-lazy-club-dna-score-cache.md` — record the developer-chosen representative warm threshold, the considered and rejected v33 index alternative, delayed cold first-use evidence, decision consequences, and current implementation status linked to this ledger.

**Behavior and data flow:**

- Keep v32, the exact-count completeness fast path, cache-first indexed query architecture, exact-count correctness, cache identity, scopes, invalidation, versions, formula, UX, frontend behavior, one-PR boundary, and every publication authority field unchanged. Change the 184,000-player warm acceptance limit for each Search filter, Search sort, and Squad sort from `<=200 ms` to `<=1,000 ms`. The ledger classifier remains runnable, while delivery remains unauthorized and non-runnable until review clears and the Delivery fingerprint is recorded.

**Ordered implementation steps:**

1. Verify branch `feature/club-dna`, clean starting HEAD `8250dbe9aac7853ed90ba674f83a67da870a8ecb`, and completed refs 1–6 without changing Git state.
2. Confirm the failed Commit 7 six-path worktree was discarded and only `.wiki/features/active/club-dna.md` and `.wiki/decisions/0023-lazy-club-dna-score-cache.md` changed.
3. Record the exact 2,000- and 184,000-player cold/p95 evidence, the large exact-query sort/join cost, rejected v33 index alternative, accepted shared threshold, replacement Commit 8 proof, renumbered unchanged frontend packets as Commits 9–11, invalidated prior authorization, and `Pending review` fingerprint.
4. Run the ledger classifier, delivery classifier, Markdown check, exact two-path diff check, LSP diagnostics, and status check. Before review, the delivery classifier may fail only because the replacement fingerprint is pending.
5. Submit the exact two-path planning diff for independent plan review. Correct any evidence, schema, packet, or ADR inconsistency and rerun review until the verdict clears.
6. After review clears, run `ledger_state.py`, then `delivery_state.py`. Record only the exact classifier-returned fingerprint in a later authorized acceptance step, rerun both classifiers, and present that exact fingerprint to the developer for acceptance. Do not compute or record it during this bounded replan.

**Tests and proof:**

- Not applicable — planning documents only. `ledger_state.py` must prove schema 2, exactly one Active commit, completed evidence, and packet completeness. `delivery_state.py` must classify the replan as non-runnable while the fingerprint is pending. `git diff --check`, the exact two-path diff, LSP diagnostics, and `git status --short` prove Markdown, links, and path scope. No tests, fixtures, mocks, snapshots, helpers, compatibility paths, or other documents change.

**Patterns to verify:**

- `.wiki/features/active/README.md` schema 2; completed Commit 6 at `8250dbe9aac7853ed90ba674f83a67da870a8ecb`; ADR-0023's accepted lazy cache and v32 index decision; and the discarded Commit 7 measurements supplied for this bounded replan.

**Constraints and non-goals:**

- Preserve the exact one PR, branch, base, provider, template, title, merge method, required check, feature close-out state, and CI repair count.
- Preserve completed refs 1–6 and exact-count/indexed architecture. Change ADR-0023 only where its status, threshold, considered alternative, and consequences must record the accepted product and operational contract.
- Do not add migration v33, another index, a new ADR, code, tests, or another documentation path. Do not compute or record a replacement Delivery fingerprint.

**Dependencies and sequencing:**

- Depends on completed Commit 6 at `8250dbe9aac7853ed90ba674f83a67da870a8ecb`, the discarded failed Commit 7 six-path worktree, the supplied exact-count/indexed evidence, and the developer's explicit threshold and index decisions. Commit 8 requires this reviewed plan and a later developer-accepted replacement Delivery fingerprint.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/club-dna.md`; `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/club-dna.md .`; `git diff --check -- .wiki/features/active/club-dna.md .wiki/decisions/0023-lazy-club-dna-score-cache.md`; `git diff -- .wiki/features/active/club-dna.md .wiki/decisions/0023-lazy-club-dna-score-cache.md`; LSP diagnostics for both changed Markdown paths; `git status --short`

**Stop conditions:** Stop on a branch/HEAD mismatch, missing completed ref, a changed path outside the ledger and ADR-0023, any classifier error beyond pending fingerprint authorization, uncleared review, missing developer acceptance before later fingerprint authorization, a changed PR authority field, or a request to change v32, exact-count completeness, cache-first query architecture, exact-count correctness, cache scopes, versions, invalidation, formula, UX, or frontend behavior. Replan instead of adding v33 indexes if the clean implementation exceeds the accepted threshold.

**Review mandate:** Verify (1) exact supplied 2,000- and 184,000-player evidence, (2) cold results are delayed first-use evidence and warm p95 values use the correct thresholds, (3) the root cause is remaining large exact-query sorting/joins rather than completeness scans, (4) the rejected v33 alternative and storage/complexity rationale match ADR-0023, (5) immutable completed refs 1–6 and clean HEAD are exact, (6) exactly Commit 7 is Active and replacement Commit 8 is execution-ready from clean HEAD, (7) Commits 9–11 are unchanged except renumbering and dependencies, and (8) only the ledger and ADR-0023 changed while the replacement fingerprint remains pending.

#### Commit 8 — Integrate indexed queries with the completeness fast path

**Status:** Active

**Provisional commit:** `perf(club-dna): skip complete cache scans`

**Work:** Integrate fixed `club_dna` display, Search filter/sort, and Squad sort with request-scoped trusted context, indexed cache-first global queries, and an authoritative exact-count fast path that skips warm complete-cache scans.

**Size assessment:** About 200–300 changed non-test implementation lines. The request context, resolver/filter contract, completeness count, Search count/page/select shapes, and Squad sort shape form one measured consistency and performance boundary; splitting them could leave one supported operation on the breached global scan or an incomplete indexed integration.

**Out of scope:**

- Migration v32 or index changes, a durable completeness marker, ADR-0023, definition mutation or invalidation changes, cache scope changes, frontend metric metadata or UI, eager ingest/promotion, Moneyball, Player Profile, background progress/cancellation, and current-state documentation.

**Implementation packet:**

- Freshly implement the complete corrected indexed-query integration from clean HEAD `8250dbe9aac7853ed90ba674f83a67da870a8ecb`. Replace every Club DNA correlated scalar read with one request-scoped trusted definition/model context and fixed cache aliases. Keep display player-first and page-scoped. Before any global UID enumeration or materializer call, compare exact-version cache-row count with the authoritative operation cohort count. Skip all UID allocation, materializer calls, and batched miss probes on equality; otherwise use the existing resumable materializer. After completeness, drive global operations from exact-version `club_dna_scores` rows. Search filter must use the v32 score index for exact-version count and candidate selection before joining those score-filtered candidates to players and applying the required final CA/UID temporary sort and bounded page selection. Search and Squad Club DNA sorts must use the score index for exact score selection and order; permit only their documented null-last and UID tie-break temporary sort work.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/club_dna.rs` — expose the minimum trusted request context that fetches and validates the snapshot's definition once, carries the Rust-owned score-model version, distinguishes missing definition, lets materialization reuse the same validated definition/version instead of looking it up again, and supports instrumentation that proves whether UID enumeration, materializer entry, or per-batch miss probes occurred.
- `src-tauri/src/features/player_metrics/resolver.rs` — recognize only fixed `club_dna`; resolve display and sort against a caller-supplied fixed cache alias and trusted bound context; remove any scalar snapshot/definition/cache expression; preserve nullable integer decoding and null-last behavior.
- `src-tauri/src/features/search/filter.rs` — represent Club DNA filter intent in the validated AST and compile every integer operator against the fixed exact-version cache alias with explicit `score IS NOT NULL`, including `neq`; keep filter values bound and preserve AND/OR composition and the 32-rule bound.
- `src-tauri/src/features/search/query.rs` — fetch one trusted context after request validation; reuse it for materialization and SQL binds; keep display player-first, materialize only selected page UIDs, and `LEFT JOIN` one exact-version cache alias in the final select; for Club DNA filter or sort, compare the exact-version snapshot cache count with the snapshot player count before UID enumeration, skip all materializer work on equality, otherwise resume existing materialization; make filter count and candidate selection use the v32 score index before joining those score-filtered candidates to players and applying the required final CA/UID temporary sort and bounded page selection; make Club DNA sort score selection and order use that index, with only documented null-last and UID tie-break temporary sort work; add correctness, completeness, stale/missing-definition, page-scope, EXPLAIN, and benchmark-runner proof.
- `src-tauri/src/features/planner/squad.rs` — fetch one trusted context after request and configured-cohort validation; keep display page-first with one exact-version `LEFT JOIN`; for Club DNA sort, obtain the exact managed-club cohort count, compare it with exact-version cache rows joined to that same managed-club predicate before UID enumeration, skip all materializer work on equality, otherwise resume existing exact-cohort materialization, then use the v32 score index for exact score selection and order before joining only cohort members; permit only documented null-last and UID tie-break temporary sort work, and reuse the cohort count only where the exact same validated predicate makes it safe.
- `src-tauri/src/features/planner/squad_tests.rs` — prove exact cohort/nonmember behavior, completeness fast-path and resumability, managed-club and membership changes, missing definition, stale versions, page display, indexed sort planning, and include Squad in each size-specific three-shape benchmark report.

**Behavior and data flow:**

- Invalid field, filter, sort, page, snapshot, or unconfigured Squad requests follow existing validation and safe paths before Club DNA writes. A valid Search or Squad request fetches the trusted definition/model context once. The same context supplies bound snapshot, definition version, and score-model version values to materialization and every dependent SQL statement. No statement performs a correlated definition lookup.
- With a present definition, display-only Search and Squad select the requested page under the existing non-Club-DNA order, materialize only those UIDs with the fetched context, and run one final player select with an exact-version cache `LEFT JOIN`.
- With no definition, the request branches before materialization and before any cache join. Display projects `club_dna` as null. Club DNA sort stays player-first, retains every player, projects all-null Club DNA values, and uses the existing UID tie-break for stable order. Each Club DNA rule compiles to a SQL-false predicate within the existing flat AND/OR AST. An isolated rule and any mixed AND filter that contains one return empty, while a mixed OR filter retains rows matched by its non-Club-DNA rules. Never drive a missing-definition request cache-first from an empty cache.
- With a present definition, a global request proves completeness before any UID enumeration. Search compares `COUNT(*)` for exact snapshot/definition/model cache rows with the snapshot player count. Squad compares the exact managed-club member count with `COUNT(*)` for exact-version cache rows joined to that same club predicate. Nullable rows count as materialized. Rows outside the managed club cannot satisfy Squad completeness. The cache primary key prevents duplicates, and the player/snapshot foreign key prevents orphan rows. Equality skips UID allocation, materializer entry, and every per-250 `NOT EXISTS` miss probe. Inequality uses the existing resumable materializer, including partial-batch recovery and boost invalidation. A new Search player or Squad entrant without an exact current-version row creates inequality and materializes. Search player removal cascades both row counts. A Squad exit drops the player from both cohort predicates. A Squad entrant already materialized by Search and a switch to an already complete club can preserve equality and must skip UID enumeration and materialization safely. Reuse a cohort count only within the same validated snapshot and exact predicate.
- Once complete, Search Club DNA filter/sort uses cache-first SQL that binds exact versions and starts from `club_dna_scores`. Search filter uses the v32 score index for exact-version filtered count and candidate selection, then joins those score-filtered candidates to players and may use the required temporary sort for final CA/UID order and bounded page selection. Search Club DNA sort and Squad Club DNA sort use the v32 score index for exact score selection and score order. They may use only the documented temporary sort needed for null-last ordering and the UID tie-break that v32 does not cover. Because complete materialization stores one row per required player even when `score` is null, exact-version `INNER JOIN` global operations retain unavailable rows for null-last sorting. Filters explicitly exclude null. Reject correlated definition lookups and unbounded player-first global scans.
- SQL interpolates only fixed internal aliases and validated `ASC`/`DESC` directions. It binds snapshot, versions, filter values, limit, offset, save/cohort values, and other request data. Final DTOs remain bounded and React receives no cohort or score inputs.

**Ordered implementation steps:**

1. Add RED request-context tests that count one definition fetch per Search or Squad request, prove materialization reuses that validated context, and reject any correlated definition lookup in generated Club DNA SQL.
2. Add RED completeness tests that instrument UID enumeration, materializer entry, and per-batch miss probes. Require a complete exact-version Search or Squad cache, including nullable rows, to trigger zero calls at all three seams. Prove stale versions do not count; one missing row, a partial batch, player-boost invalidation, and definition-version change resume the current materializer.
3. Add separate RED cohort-transition tests for each count result. A Search addition without an exact current-version row and a Squad entry without one must mismatch and materialize. A Search removal must cascade both counts and remain equal. A Squad exit must leave both cohort predicates equal. A Squad entrant already materialized by Search and a managed-club switch to an already complete cohort must remain equal and skip UID enumeration/materialization safely. Outside-club exact-version rows must not false-pass an incomplete cohort. Prove count reuse occurs only for the same validated snapshot and predicate.
4. Add RED resolver/filter tests for fixed-ID acceptance, unsafe/unknown rejection, exact bound snapshot/definition/model values, fixed cache aliases, all integer operators with null exclusion including `neq`, mixed AND/OR/all-operator composition, and stale-version exclusion. For no definition, prove each Club DNA rule emits SQL false inside the flat AST rather than replacing the full filter. Name every new correctness test for definition context, completeness, nullable/stale/partial/boost behavior, cohort transitions, no-definition flat AND/OR behavior, and zero enumeration with the exact stable prefix `club_dna_completion_fast_path_`.
5. Add RED Search integration tests for count, ordered page selection, and final select across Club DNA display, filter, sort, and combined filter-plus-sort. Prove page-only display, completeness-check-or-materialize before global statements, exact current versions, bounded pages, nullable rows, deterministic ties, stale-version exclusion, and edit/remove/re-create isolation. For no definition, cover every operator in isolation, mixed AND, mixed OR, display, and Club DNA sort; require all-null UID-stable player-first results where applicable and zero materialization/cache writes.
6. Add RED Squad tests in `planner/squad_tests.rs` for page-only display; exact configured-cohort sort; nonmember exclusion from completeness, materialization, and results; null-last unavailable members; absent-definition all-null player-first sort with every member retained, UID-stable order, no cache join, and zero writes; and stale-version exclusion.
7. Implement the minimum trusted request context in `player_metrics/club_dna.rs`. Keep the existing public pure scorer and cache identity; let the current materializer delegate through the context-aware path so existing callers and Commit 4 tests stay valid.
8. Replace correlated resolver/filter expressions with fixed-alias expressions. Add exact-version authoritative count comparisons before global UID allocation. Build player-first display SQL with one exact-version `LEFT JOIN`, and cache-first global Search/Squad SQL with exact-version binds and proven completeness preconditions.
9. Add shape-specific `EXPLAIN QUERY PLAN` assertions. For Search filter, require the committed v32 `(snapshot_id, definition_version, score_model_version, score)` index for exact-version count and candidate selection, then allow the required temporary sort for final CA/UID ordering and bounded page selection after joining those score-filtered candidates to players. For Search Club DNA sort and Squad Club DNA sort, require that index for exact score selection and score order; allow only the documented temporary sort needed for null-last ordering and the UID tie-break that v32 does not cover. Reject every correlated definition lookup and unbounded player-first global scan.
10. Add one deterministic benchmark runner that uses the production `search_players_in_view`/`search_players` and `list_squad_players` paths through final DTO mapping. Build the exact 2,000- and 184,000-player fixtures and exact requests defined in **Tests and proof**. Exclude fixture construction from every timer and run correctness assertions before timing acceptance.
11. In each size-specific benchmark test, clear the cache for each Search filter, Search sort, and Squad sort shape and record cold first use separately. Prefill the shape, run 3 unmeasured warm-ups and 20 measured samples, sort samples, and take nearest-rank p95 at index 18. The test must execute and report all three of its shapes before it asserts that size's threshold.
12. Run `cd src-tauri && cargo test club_dna_completion_fast_path_ -- --nocapture --test-threads=1` and require discovery and execution of all newly prefixed correctness tests before any benchmark. Then run the normal 2,000-player benchmark test, which asserts every warm p95 `<500 ms` after reporting its three shapes. Run the ignored generated 184,000-player benchmark test separately; it asserts every warm p95 `<=1,000 ms` after reporting its three shapes. The two commands together cover six combinations, but no result or assertion crosses the process boundary. Treat the measured 856.517 ms representative Squad sort p95 as the closest threshold margin and a focused regression risk.
13. Run `./scripts/dev check-rust` and `./scripts/dev check` after the focused correctness command and both benchmark commands pass.

**Tests and proof:**

- RED: clean HEAD has no Club DNA resolver/filter query contract or completeness fast path. The discarded failed Commit 7 proof implemented the accepted exact-count/indexed architecture and reported all six required shapes. Its 184,000-player warm p95 values of 373.344 ms, 509.068 ms, and 856.517 ms meet the developer-chosen `<=1,000 ms` threshold. A test that permits player-first correlated SQL, complete-cache UID enumeration, any complete-cache materializer/miss-probe call, or early benchmark exit is insufficient.
- Request-context proof: instrument the trusted definition fetch or use an equivalent focused seam to prove exactly one fetch and validation per Search/Squad request, including filter-plus-sort-plus-display. Inspect generated SQL or query-plan evidence to prove no correlated definition lookup remains.
- Completeness proof: seed complete exact-version Search and Squad caches and assert zero UID enumeration, zero materializer calls, and zero batched `NOT EXISTS` probes. Include nullable cache rows and require them to count as complete. Then remove one row, leave a partial materializer batch, and invalidate one boosted player; each case must enter the existing resumable materializer and restore complete current-version rows. Seed stale definition/model rows and prove they never count toward completeness.
- Cohort-transition proof: use separate tests for each transition instead of assuming every change mismatches. Add a Search player without an exact current-version row and move an unmaterialized player into the managed club; each must mismatch and materialize. Remove a Search player and prove cascading deletion reduces both Search counts and keeps equality. Move a player out of the managed club and prove both Squad cohort predicates exclude it while the exact-version cache row can remain. Move a player with an exact row already materialized by Search into the club and switch to a different already complete club; each must keep equality and safely perform zero UID enumeration/materialization. Seed outside-club exact-version rows and prove they cannot false-pass an incomplete Squad cohort or permit stale cohort-count reuse.
- Search proof: cover count, page UID selection, final select, display-only, filter-only, sort-only, filter-plus-sort, every operator (`gt`, `lt`, `eq`, `neq`), mixed AND and OR rules, null exclusion, stale definition/model rows, deterministic null-last order, and bounded offset/limit. With no definition, prove display and Club DNA sort stay player-first, retain every player with null DTO scores and UID-stable order, perform no cache join or writes, and never start cache-first. Prove each operator in isolation returns empty, mixed AND returns empty, and mixed OR retains non-Club-DNA matches because only the Club DNA rule compiles to SQL false. With a definition, prove display materializes only the returned page and global operations either prove exact-version count equality before UID enumeration or complete current-snapshot materialization before their first dependent count/order/page statement.
- Squad proof: cover page-only display, exact managed-club cohort sort, unavailable members retained null-last, nonmembers excluded from completeness/materialization/results, unconfigured safe empty behavior, stale versions, managed-club change, player entry/exit, and outside-club cache rows. With no definition, require a player-first all-null Club DNA sort that retains every exact-club member in UID-stable order and performs no cache join or writes.
- Deterministic fixture: run once with `N = 2,000` and once with `N = 184,000`. Create only players with `uid` from 1 through `N`, and put all `N` players in the exact managed club `Benchmark FC`. Each player JSON object includes every selected catalog key for that JSON source: visible/goalkeeper, hidden, or personality. Each selected value normally equals `((uid - 1) % 20) + 1`, which produces tied scores from 5 through 100. For every `uid % 100 == 0`, keep all keys present but set one selected personality value to JSON null, which makes exactly 1% of scores null. The timed fixture contains no outside-club player, so General Search and Squad totals remain exactly `N` and Search filter totals remain `980` and `90,160`. Keep nonmember exclusion in separate focused Squad correctness tests outside the timed fixture. Exclude all fixture construction from timing.
- Exact Search filter request: call the production `search_players_in_view`/`search_players` path through final DTO mapping with filter `club_dna gt 50`, the existing default deterministic non-DNA sort (`ca` descending with UID tie-break), requested display field `club_dna`, limit 50, and offset 0. Before timing acceptance, assert total `49 * N / 100` (`980` for 2,000 and `90,160` for 184,000), every returned row satisfies the filter, and every final DTO has the expected Club DNA score.
- Exact Search sort request: call the same production path through final DTO mapping with `club_dna` descending, requested display field `club_dna`, limit 50, and offset 0. Before timing acceptance, assert total `N`, null-last order, score-100 ties followed by UID ascending, and expected final DTO scores.
- Exact Squad sort request: call production `list_squad_players` through final DTO mapping for the complete exact `Benchmark FC` cohort of `N` players, `club_dna` ascending, requested display field `club_dna`, limit 50, and offset 50. Before timing acceptance, assert total `N`, correct tied score then UID order, and expected final DTO scores. Prove nonmember exclusion in separate focused Squad correctness tests, not in this timed fixture.
- Index proof: Search filter `EXPLAIN QUERY PLAN` must use the committed v32 score index for exact-version count and candidate selection; after joining those score-filtered candidates to players, it may use the required temporary sort for final CA/UID ordering and bounded page selection. Search Club DNA sort and Squad Club DNA sort must use that index for exact score selection and score order; permit only the documented temporary sort for null-last ordering and the UID tie-break that v32 does not cover. Reject correlated definition lookups and unbounded player-first global scans in all three shapes.
- Focused correctness proof: every new test for definition context, completeness, nullable/stale/partial/boost behavior, cohort transitions, no-definition flat AND/OR behavior, and zero UID enumeration uses the exact stable prefix `club_dna_completion_fast_path_`. Run `cd src-tauri && cargo test club_dna_completion_fast_path_ -- --nocapture --test-threads=1` before either benchmark and require Cargo to discover and execute those tests.
- Performance proof: run correctness assertions before accepting any timing. Within each size-specific test, clear the cache for each exact shape, measure cold first use separately, prefill, run 3 unmeasured warm-ups and 20 measured samples, and compute nearest-rank p95 from sorted index 18. The 2,000-player test executes and reports its three shapes before asserting every p95 `<500 ms`. The generated 184,000-player test executes and reports its three shapes before asserting every p95 `<=1,000 ms`. The two commands cover six combinations without any cross-process aggregate. The discarded exact-count/indexed run measured representative p95 values of 373.344 ms, 509.068 ms, and 856.517 ms; reproduce rather than hardcode them.
- Add/modify: request-context and materializer instrumentation tests in `player_metrics/club_dna.rs`; resolver/filter tests; Search completeness, correctness, EXPLAIN, fixture, and benchmark helpers in `search/query.rs`; Squad completeness, cohort-transition, correctness, and query-plan coverage in `planner/squad_tests.rs`. Deliberately retain migration/index characterization, pure scoring, materializer batching/rollback, raw attribute, role, potential-role, current-snapshot, managed-club, pagination, null-ordering, and request-bound tests because they protect independent current contracts. Delete only failed Commit 7 worktree helpers if any reappear; none exist at clean starting HEAD. No mocks or snapshots change.

**Patterns to verify:**

- `player_metrics::club_dna::{definition_for_snapshot,materialize_player_scores}` for the current definition fetch and bounded materializer that the request context must reuse without weakening validation.
- `search_players_in_view`, `search_players`, and `list_squad_players` for existing request validation, final DTO mapping, potential-role materialization scopes, bounded count/page/select sequencing, and deterministic UID ties.
- `MetricSource::PotentialRole`, `compile_potential_role_score_rule`, and their tests for fixed-cache aliases, exact-version null guards, and bound values; deliberately diverge from their correlated scalar subquery shape for Club DNA.
- Migration v32 index tests for the exact score index contract; `attribute_filter_on_two_thousand_players_stays_interactive` only for local timing style, not query shape or representative acceptance.

**Constraints and non-goals:**

- Keep ADR-0023, v32 schema/index, cache identity, lazy scopes, invalidation, versions, formula, mutex/transaction boundaries, UX, the accepted `<500 ms` 2,000-player threshold, the accepted `<=1,000 ms` representative threshold, and frontend behavior unchanged. Do not add a durable completeness marker or v33 index.
- Never filter or sort a partial cohort. Never return the full cohort to the WebView. Display remains page-only and player-first; global Club DNA operations become cache-first only after authoritative exact-count equality or complete resumable materialization.
- Completeness must count exact snapshot/definition/model rows. Squad must join those rows to the exact current managed-club predicate before counting. Nullable rows count; outside-club rows do not. Additions and entries without an exact current-version row mismatch; removals cascade both Search counts; exits leave both Squad predicate counts equal; Search-materialized entrants and switches to already complete clubs can remain equal. Skip UID allocation and all materializer/miss-probe calls on every equality case.
- No correlated definition lookup is permitted. Bind every value; interpolate only fixed internal aliases and validated sort directions.
- Missing definition performs no materialization, cache join, or writes. Display and Club DNA sort stay player-first, retain every player with null values, and use UID-stable order. Each Club DNA rule compiles to SQL false inside the flat AST, so isolated and mixed AND cases are empty while mixed OR retains non-Club-DNA matches. Never run cache-first from an empty cache.
- Preserve 256 requested-field and 32 filter-rule bounds. Keep `club_dna` out of Moneyball mode and Player Profile.
- Do not add a migration, index, dependency, background worker, cancellation framework, progress IPC, ADR, or current-state documentation.

**Dependencies and sequencing:**

- Depends on completed Commit 4's v32 cache/index, pure scorer, bounded materializer, and invalidation; completed Commit 5's reviewed indexed architecture at `df074b9da78bec038960e2be7c851dcb5879dbdd`; completed Commit 6's bounded completeness correction at `8250dbe9aac7853ed90ba674f83a67da870a8ecb`; and reviewed Commit 7's representative latency decision. Commit 9 may expose the fixed metric only after Commit 8 passes completeness, correctness, EXPLAIN, 2,000-player, and generated 184,000-player warm gates.

**Validation:** `cd src-tauri && cargo test club_dna_completion_fast_path_ -- --nocapture --test-threads=1`; `cd src-tauri && cargo test club_dna_indexed_query_shapes_on_complete_catalog_2k -- --nocapture --test-threads=1`; `cd src-tauri && cargo test club_dna_indexed_query_shapes_on_generated_184k -- --ignored --nocapture --test-threads=1`; `./scripts/dev check-rust`; `./scripts/dev check`

**Stop conditions:** Stop if one trusted definition/model context cannot serve materialization and every dependent statement; any correlated definition lookup remains; any version/model value must be interpolated; a missing-definition request materializes, joins the cache, writes rows, drops all players from sort/display, or replaces the complete flat filter instead of only its Club DNA rule; display requires full-cohort work; a complete-cache global request still enumerates UIDs, enters the materializer, or runs a batch miss probe; an exact-count comparison mishandles nullable/stale/partial/boost/definition-version rows or any separately specified cohort-transition equality/mismatch case; any incomplete global query runs before resumed materialization completes; complete nullable rows cannot support cache-first `INNER JOIN` semantics; Search count/page/select or Squad exact-cohort behavior cannot be proved; required correctness tests do not use or execute under `club_dna_completion_fast_path_`; Search filter EXPLAIN does not use the v32 score index for exact-version count and candidate selection, Search or Squad Club DNA sort EXPLAIN does not use it for exact score selection and order, any shape uses an unbounded player-first global scan, or temporary sorting exceeds the documented Search-filter CA/UID final order or Club DNA sort null-last/UID tie work; either size-specific runner adds an outside-club player, omits final DTO mapping, times fixture construction, accepts timing before correctness, fails to execute/report its own three shapes before its own assertion, or depends on cross-process aggregation; any 2,000-player warm p95 is `>=500 ms`; or any generated 184,000-player warm p95 is `>1,000 ms`. Replan rather than adding v33 indexes or accepting a measured breach.

**Review mandate:** Verify (1) exactly one trusted request context and no correlated definition lookup, (2) exact bound snapshot/definition/model values and safe fixed alias/direction interpolation, (3) complete exact-version caches including nullable rows trigger zero UID enumeration, materializer calls, and batch probes, while missing/partial/boost/definition-version cases resume, (4) separate cohort-transition tests prove unmaterialized additions/entries mismatch, removals cascade both Search counts, exits leave both Squad predicates equal, Search-materialized entrants and already-complete club switches remain equal, and outside-club rows cannot false-pass, (5) present-definition page display uses one exact-version `LEFT JOIN`, while missing-definition display/sort stays player-first with every player retained, null values, UID-stable order, no cache join, and zero writes, (6) every required new correctness test uses `club_dna_completion_fast_path_` and the focused command runs before benchmarks, (7) each size-specific benchmark contains only its exact `N` club members, executes/reports its own three production-path shapes, and asserts `<500 ms` at 2,000 players and `<=1,000 ms` at 184,000 players with no cross-process aggregate, and (8) EXPLAIN proves Search filter index use for exact-version count/candidate selection and Search/Squad Club DNA sort index use for exact score selection/order, permits only the documented CA/UID or null-last/UID temporary sort work, rejects unbounded player-first global scans, treats the measured 856.517 ms representative Squad p95 as the closest threshold margin, and stops delivery on any warm breach.

#### Commit 9 — Add the frontend Club DNA domain and fixed metric

**Status:** Pending

**Provisional commit:** `feat(club-dna): add frontend metric foundation`

**Work:** Add typed Club DNA IPC/query adapters, consolidate the frontend attribute catalog, register the fixed metric, and render it as a score in Search and Squad.

**Size assessment:** About 170–210 changed non-test implementation lines. The shared catalog extraction and fixed table metric are one coherent frontend contract; minor excess is acceptable if splitting would leave duplicated catalogs or a non-rendering metric.

**Out of scope:**

- My Club action, form Modal, mutation UI, automatic layout append, active-save invalidation wiring, backend logic, Moneyball, and Player Profile score display.

**Implementation packet:**

- Establish one shared frontend attribute catalog used by Player Profile, metric metadata, and the later Club DNA form. Add typed context-bound backend adapters and a save-context-bearing Query key; keep the fixed metric selectable even when the active save has no definition.

**Files and responsibilities:**

- `src/utils/player-attributes.ts` and `src/utils/player-attributes.test.ts` — own and prove canonical frontend visible/goalkeeper, hidden, and personality groups, IDs, labels, and ordering.
- `src/features/player-profile/utils/attribute-groups.ts` and `attribute-groups.test.ts` — consume the shared catalog while preserving every current Player Profile group, goalkeeper composition, label, and ordering contract.
- `src/utils/player-metrics.ts` and `player-metrics.test.ts` — consume the shared attribute keys and add fixed integer metric `club_dna`, label `Club DNA`, category `ability-reputation`, sortable/filterable operators, right alignment, and score-sized width.
- `src/features/club-dna/types/club-dna.ts` — typed `{ saveId, contextToken }` definition context, definition, and mutation result shapes.
- `src/features/club-dna/api/club-dna-keys.ts`, `club-dna-query-options.ts`, `set-club-dna.ts`, and `remove-club-dna.ts` — define `clubDnaKeys.definition(saveId, contextToken)` and require the same expected context in every typed `invokeCommand` call; no raw Tauri import.
- `src/features/club-dna/api/club-dna-api.test.ts` — prove definition-key isolation and exact save-context invoke arguments for get/set/remove.
- `src/features/search/components/search-results-panel.tsx` — render numeric `club_dna` through `ScoreBadge`, preserving `—` for null.
- `src/features/squad/components/squad-overview-panel.tsx` — render the same metric through `ScoreBadge`, preserving `—` for null.
- `src/stores/use-player-table-store.test.ts` — prove `club_dna` is accepted by Search and Squad layouts and retained through hydration; production store version remains unchanged unless deterministic evidence shows a migration is required.
- `src/features/search/utils/dynamic-columns.test.ts`, `src/app/routes/search.test.tsx`, and `src/app/routes/my-club-squad.test.tsx` — update only the focused fixed-metric selection/request/null-score assertions; retain unrelated route and table behavior unchanged.

**Behavior and data flow:**

- Frontend callers receive an explicit context from the established saves query and can fetch that context's nullable definition or invoke set/remove with the same save ID and immutable token. The definition Query key contains both values, so save A data cannot satisfy save B or a replacement save incarnation. Adapters return results to their caller but do not infer currentness; Commits 10–11 UI compares the captured context before applying results, while Rust rejects a context that is no longer active. The shared catalog exposes canonical IDs and FM grouping but no score function. The fixed `club_dna` metadata flows through the existing picker, filter registry, sort validation, requested-field adapter, nullable dynamic DTO, and table cell. Search and Squad render a backend-supplied integer with `ScoreBadge`; null stays `—`.

**Ordered implementation steps:**

1. Add RED catalog tests that require one canonical list to preserve all Player Profile groups and make every approved ID available.
2. Extract the minimum shared constants and update Player Profile and player-metric consumers without changing current layout behavior.
3. Add RED metric/store tests for fixed ID, integer operators, picker category, Search/Squad layout acceptance, hydration retention, and nullable score presentation.
4. Add RED API tests or focused adapter assertions that distinguish two save IDs/tokens in Query keys and require the expected pair in get/set/remove invoke arguments.
5. Add typed context-bound Club DNA Query/mutation adapters.
6. Add the fixed metric and special score rendering in both table adapters; do not add it to defaults.
7. Run focused frontend tests, the frontend gate, then the full gate.

**Tests and proof:**

- RED: `getPlayerMetric("club_dna")` is undefined; store hydration drops it; table adapter tests render a plain or absent dynamic cell. Shared-catalog tests fail until the current profile and metric key sets consume the same canonical source.
- GREEN: prove all supported IDs appear once, goalkeeper attributes remain in the visible JSON source, Player Profile groups are unchanged, `club_dna` uses integer operators and fixed label, Search/Squad accept it without defaulting it into layouts, backend `null` renders `—`, integer values render the shared score badge, save A and save B use distinct definition keys, and all three IPC requests carry the exact expected save ID/token.
- Add/modify: shared catalog tests, `attribute-groups.test.ts`, `player-metrics.test.ts`, `use-player-table-store.test.ts`, focused Club DNA API tests, and focused Search/Squad adapter tests where current suites provide the seam. Deliberately retain all existing Player Profile grouping, dynamic-column, layout migration, picker, and table-interaction tests because the extraction must be behavior-preserving. Delete no fixtures, IPC mocks, snapshots, helpers, or compatibility paths.

**Patterns to verify:**

- `src/features/player-profile/utils/attribute-groups.ts` for current FM layout.
- `src/utils/player-metrics.ts` and `src/features/search/utils/filter-registry.ts` for fixed integer metric propagation.
- Role-score branches in Search and `ScoreBadge` conventions for 0–100 display.
- `src/stores/use-player-table-store.ts::sanitizeLayout` for catalog-based persistence retention.

**Constraints and non-goals:**

- React must not calculate or validate the authoritative score. Rust remains authoritative for stale-context rejection; frontend keying and guards prevent stale rendering and side effects but do not replace that boundary.
- Do not add `club_dna` to default layouts, Moneyball metrics, Player Profile, or current-state docs.
- Do not create a cross-feature import from Club DNA to Player Profile or vice versa.
- Preserve table store key and existing layouts. Do not bump version without a migration need.

**Dependencies and sequencing:**

- Depends on Commit 8 so every exposed metric request is already accepted, completeness-checked, and cache-backed in Rust. The later Modal and route packets consume these adapters and shared groups.

**Validation:** `./scripts/dev test src/utils/player-attributes.test.ts src/features/player-profile/utils/attribute-groups.test.ts src/utils/player-metrics.test.ts src/features/club-dna/api/club-dna-api.test.ts src/features/search/utils/dynamic-columns.test.ts src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx`; `./scripts/dev check-app`; `./scripts/dev check`

**Stop conditions:** Stop if catalog consolidation changes a current Player Profile group, if a supported frontend ID has no exact Rust counterpart, if Query/API adapters cannot bind get/set/remove to `{ saveId, contextToken }`, if the fixed metric requires a new table component or store owner, if store retention requires resetting user layouts, or if React would need to compute a score.

**Review mandate:** Verify (1) exact catalog parity and no cross-feature import, (2) Player Profile grouping stays unchanged, (3) fixed label/ID and integer filter metadata, (4) no default layout insertion and persisted layout retention, (5) ScoreBadge/null presentation in both tables, (6) context-bearing definition key isolates save IDs and tokens, (7) all typed invoke requests carry the exact expected context, and (8) no frontend score, stale-context authority, or validation duplicates Rust.

#### Commit 10 — Build the Club DNA definition Modal

**Status:** Pending

**Provisional commit:** `feat(club-dna): build definition modal`

**Work:** Add the accessible create/edit/remove form Modal with the approved FM-style catalogs, selected summary, equal-weight explanation, and mutation lifecycle.

**Size assessment:** About 180–240 changed non-test implementation lines. The complete accessible form and its destructive removal confirmation are one atomic user interaction; splitting state from presentation would create an incomplete component.

**Out of scope:**

- Placement in My Club, managed-club enablement, automatic table append, app-shell context invalidation, table query invalidation outside the component's mutation contract, Player Profile display, and backend formula changes.

**Implementation packet:**

- Build a feature-owned action and one Modal that use the typed context-bound Query/API layer and shared catalog. Keep draft selections local, reset them from persisted data on each open, discard them when the supplied save context changes, and keep explanation only inside the Modal. Use an edit-form state ↔ destructive remove-confirmation state transition inside the same Modal.

**Files and responsibilities:**

- `src/features/club-dna/components/club-dna-definition.tsx` — accept the current `{ saveId, contextToken }` and route-owned availability state, load only the context-keyed definition, expose a **Define DNA** trigger, present grouped checkboxes, selected summary, formula copy, save/cancel, edit replacement, and a same-Modal remove-confirmation state.
- `src/features/club-dna/components/club-dna-definition.test.tsx` — component-level accessible workflow, context changes, deferred responses, draft, one-Modal confirmation transition, validation, mutation, focus, and error coverage.
- `src/testing/club-dna-ipc-mock.ts` and `src/testing/setup.ts` — bounded context-aware test IPC state for get/set/remove, call capture, independently deferred responses, pending/error controls, and reset; no score computation in the mock.

**Behavior and data flow:**

- Opening reads the definition keyed by the supplied save ID/token and copies it into local draft state. Checkboxes follow the shared Player Profile group order and allow the full catalog. The Modal shows a selected-attribute list and the equal-weight formula. Save stays disabled at zero selections and while the supplied context is refreshing, mismatched, errored, or no longer current; it sends the complete ordered ID list with the captured context. A context change closes the Modal, discards its draft, resets visible feedback, and prevents late prior-context get/set/remove results from changing the current-context component. Set errors stay in the edit form.
- One `Modal` instance switches between edit-form state and destructive remove-confirmation state, following `src/features/planner/components/planner-team-management.tsx::PlannerTeamManagement`. Entering confirmation preserves the edit draft. Cancel or Escape from confirmation returns to edit and restores useful focus without discarding the draft. Cancel, Escape, backdrop dismissal, or close from edit closes the Modal and discards unsaved changes. Pending removal blocks dismissal and duplicate submission. A remove error stays in confirmation with an actionable error and Cancel return path. Successful removal closes the Modal, reports only to the matching context owner, and restores focus to the trigger.

**Ordered implementation steps:**

1. Add RED component tests for disabled/refreshing/mismatched/errored context, initial create state, full-catalog selection, empty-save guard, selected summary/formula, persisted edit state, cancel discard, backend error retention, successful create/edit callbacks, one-Modal destructive remove transition, pending dismissal guards, and focus return.
2. Add deferred-response tests that change from save A context to save B while the edit Modal is open and while set or remove is completing. Require the Modal/draft to close, prior feedback to disappear, and late A results to produce no B callback or UI update.
3. Implement the minimum feature-owned trigger and form using one `Modal`, semantic fieldsets/legends, native checkboxes, context-keyed Query data, local draft state, and context-captured mutations.
4. Implement edit ↔ remove-confirmation as state inside that Modal. Preserve the draft when entering/leaving confirmation; block all dismissal and duplicate remove while pending; keep remove errors in confirmation with a Cancel return path.
5. Keep server validation and stale-context rejection authoritative while providing responsive non-empty and current-context guards in React.
6. Ensure all selected attributes can be reached by keyboard and the full catalog remains inside Modal scroll bounds.
7. Run the focused component test, frontend gate, then full gate.

**Tests and proof:**

- RED: no **Define DNA** control or dialog exists. The new test fails on missing accessible trigger/dialog, no selected summary, and absent set/remove calls.
- GREEN: prove create sends all selected canonical IDs with the supplied context, edit starts from stored IDs and reports `created: false`, zero selection cannot submit, cancel/reopen restores persisted data, and set errors remain actionable in edit. Prove one dialog changes from edit to confirmation while preserving the draft; Cancel/Escape from confirmation returns to edit; Cancel/Escape from edit discards and closes; pending removal blocks Escape, backdrop, Cancel, and duplicate submit; remove errors remain in confirmation with a return path; successful removal closes and restores trigger focus.
- Deferred proof: switch from save A to save B with edit open and during set/remove completion. Assert the A draft closes and is discarded, B never renders A data or feedback, and late A success/error cannot call the current-context success handler. The IPC mock must still route stale requests so Rust command/service tests remain the authoritative proof that no wrong-save mutation occurs; frontend tests prove stale UI suppression. A mock that computes a score is forbidden because Rust tests own that contract.
- Add: one component test and one bounded context-aware IPC mock helper. Modify `src/testing/setup.ts` only to route commands and reset state. Deliberately retain shared `Modal` tests, existing managed-club mocks, and the Planner team-management confirmation tests in `src/app/routes/my-club-squad.test.tsx` because they protect the shared dialog, route boundary, and established one-Modal analogue. Delete no fixtures, snapshots, helpers, or compatibility paths.

**Patterns to verify:**

- `src/components/ui/modal/modal.tsx` for focus trap, dismissal, and return focus.
- `src/features/player-profile/components/player-attributes-panel.tsx` and the shared catalog for FM-style grouping and value labels, without importing the Player Profile feature.
- `src/features/planner/components/planner-team-management.tsx::PlannerTeamManagement` and its route tests in `src/app/routes/my-club-squad.test.tsx` for the established one-Modal edit ↔ destructive-confirmation transition, draft preservation, Escape return, pending guards, errors, and focus behavior.

**Constraints and non-goals:**

- Explanation stays in the Modal. Do not add help text to tables, profile, header, or another page.
- No custom name, weights, maximum, reorder, clone/import/export, or contribution display.
- Do not calculate scores or duplicate Rust validation beyond responsive non-empty and current-context UI guards. Rust remains authoritative for stale-context rejection.
- Do not render a nested or second Modal for removal.
- Do not add new dependencies or custom listbox behavior for native checkboxes.

**Dependencies and sequencing:**

- Depends on Commit 9's typed API and shared catalog. Commit 11 provides route placement and cross-query/layout effects.

**Validation:** `./scripts/dev test src/features/club-dna/components/club-dna-definition.test.tsx`; `./scripts/dev check-app`; `./scripts/dev check`

**Stop conditions:** Stop if the shared Modal cannot support keyboard access, full-catalog scrolling, the one-Modal edit ↔ confirmation transition, pending dismissal guards, or reliable focus return; if a context change cannot close/discard the draft and suppress late prior-context results; if product behavior requires an explanation outside the Modal; or if component state cannot distinguish backend create from edit.

**Review mandate:** Verify (1) full approved catalog and no maximum, (2) minimum-one/current-context guards plus Rust stale-context authority, (3) selected summary and exact formula copy only in Modal, (4) context change closes/discards and late get/set/remove results cannot update current UI, (5) the Planner-style single Modal preserves draft across confirmation and implements exact Cancel/Escape/pending/error transitions, (6) keyboard/focus/accessibility behavior including successful-remove focus return, (7) no score computation or profile surface, and (8) deferred mocks test observable stale-result suppression without duplicating Rust mutation authority.

#### Commit 11 — Integrate Club DNA with My Club and layouts

**Status:** Pending

**Provisional commit:** `feat(club-dna): connect My Club and table layouts`

**Work:** Place **Define DNA** beside **Save managed club**, bind it to managed-club state, apply create-only Search/Squad layout append, and complete context/query invalidation and browser coverage.

**Size assessment:** About 120–180 changed non-test implementation lines. Within the soft target.

**Out of scope:**

- New persistence or formula behavior, store reset/version migration, definition removal from saved layouts or URLs, Player Profile, Moneyball, new workspaces, current-state documentation, and release work.

**Implementation packet:**

- Use route composition as the only cross-feature seam. Add a narrow action slot to `ManagedClubSelector`, then let My Club derive the current active save ID/token from the established saves query, key/remount Club DNA by that context, own managed-club/context availability, guard mutation side effects, and append Search/Squad layouts only for a matching current-context creation.

**Files and responsibilities:**

- `src/features/managed-club/components/managed-club-selector.tsx::ManagedClubSelector` — accept a narrow route-supplied adjacent action slot and render it in the existing control group beside **Save managed club**; do not import Club DNA.
- `src/app/routes/my-club.tsx::MyClubPageContent` — read `savesQueryOptions`, derive the active `{ saveId, contextToken }`, key/remount the Club DNA feature by both values, combine managed-club and Club DNA refresh/mismatch/error state into availability, guard create/edit/remove callbacks by captured-versus-current context, append `club_dna` to Search and Squad only when a matching current-context backend response has `created: true`, and invalidate current Club DNA/Search/Squad query roots after matching mutations.
- `src/app/components/app-top-bar.tsx` and `src/app/routes/settings.tsx` — include `clubDnaKeys.all` in existing active-save/current-context invalidation sets as refresh wiring in addition to, not instead of, context-bearing keys and Rust verification.
- `src/app/app-top-bar.test.tsx` and `src/app/routes/settings.test.tsx` — extend the production invalidation-path tests to prove Club DNA invalidation after save switch/Load Data and save/snapshot current-context changes.
- `src/app/routes/my-club-squad.test.tsx` — prove placement, managed-club/context availability, key/remount behavior, deferred save-switch behavior, create/edit/remove effects, create-only current-context append, edit non-restoration, deletion retention, and re-create append using the established route/mocks.
- `e2e/tauri-ipc-stub.ts` and `e2e/smoke.spec.ts` — add one proportionate Chromium path for opening the Modal, selecting attributes, seeing formula/summary, creating Club DNA, and observing the Search or Squad column; retain headless limitations explicitly.

**Behavior and data flow:**

- My Club reads the established saves query, derives the active save ID/token, and reads managed-club plus context-keyed Club DNA state. The route supplies a Club DNA component keyed by that pair beside the managed-club save button. The trigger and submission remain disabled while the saves/managed-club/Club DNA context is refreshing, mismatched, or errored, and while no managed club is selected. A context-key remount closes and discards any open draft.
- Set/remove capture their opening context. A completion first compares that pair with the route's current active pair. Only a match can show feedback or invalidate the current definition plus Search and Squad data. Only a matching set result with `created: true` calls Zustand `addColumns` once for each layout. A late save A completion after switching to save B cannot update B UI, append layouts, or present stale success/error feedback. Edit only refreshes values. Remove refreshes values but leaves layouts and URL-backed filter/sort state untouched. Re-create returns `created: true` and appends a missing column again. App-top-bar and Settings invalidation accelerate refresh, while the context-bearing Query key prevents cross-save rendering and Rust rejects stale get/set/remove requests authoritatively.

**Ordered implementation steps:**

1. Add RED route tests for exact action placement; no-managed-club and refreshing/mismatched/errored disabled states; active save ID/token derivation; context key/remount; successful current-context create append in both layouts; no duplicate append; edit after user removal; remove retention; and re-create append.
2. Add deferred-response route tests for switching from save A to save B with the Modal open and while set or remove completes. Assert the draft closes, B never displays A definition or feedback, stale completions do not invalidate/update current UI, and no stale `created: true` result appends either layout.
3. Add the narrow managed-club action slot and compose the keyed Club DNA feature in the route without a cross-feature import.
4. Wire matching-context success callbacks to query invalidation and backend-created layout behavior; do not mutate layout on remove and ignore every prior-context completion.
5. Add Club DNA query-root invalidation to existing save switch, save deletion/fallback, and Load Data/current-context paths where those paths already invalidate managed-club and product query roots. Extend `src/app/app-top-bar.test.tsx` and `src/app/routes/settings.test.tsx` at those exact seams.
6. Add one focused smoke flow and context-bearing IPC stub state; do not duplicate Rust formula or stale-context authority assertions in Chromium.
7. Run focused component/route/invalidation tests, full frontend tests, full gate, and smoke.

**Tests and proof:**

- RED: My Club has no **Define DNA** button; creation cannot append columns; active-save invalidation omits Club DNA. The route test must fail if edit re-adds a user-removed column or removal deletes persisted layout state.
- GREEN: prove the action shares the managed-club control group and is disabled when no managed club is selected or any saves/managed-club/Club DNA context is refreshing, mismatched, or errored. Prove save ID/token keying isolates definitions and remounts on context change; current-context creation appends one column to both layouts; repeated callbacks do not duplicate; edit does not restore removed columns; remove preserves layout and sort/filter-compatible metric validity; and re-create appends missing columns.
- Deferred proof: switch saves with the Modal open and during set/remove completion. Prove no wrong-context definition or feedback renders, no late completion appends layouts or updates current UI, and each IPC call retained its opening save ID/token. Pair this with Commit 2's Rust stale-context rejection tests to prove no wrong-save mutation.
- Invalidation proof: `src/app/app-top-bar.test.tsx` asserts `clubDnaKeys.all` invalidation on save switch and Load Data; `src/app/routes/settings.test.tsx` asserts it on save/snapshot current-context changes alongside the existing production paths.
- Browser proof: Chromium opens the form, shows grouped attributes and formula, creates a definition, and exposes the fixed score column. It does not prove native WebView focus, Rust/SQLite calculation, or Rust stale-context rejection.
- Add/modify: `my-club-squad.test.tsx`, `app-top-bar.test.tsx`, `settings.test.tsx`, bounded context-aware Club DNA IPC mocks, `e2e/tauri-ipc-stub.ts`, and one `e2e/smoke.spec.ts` case. Deliberately retain existing managed-club selector, layout independence, dynamic column, active-save invalidation, Modal, and smoke tests because each protects a neighboring contract. Delete no fixtures, snapshots, helpers, history compatibility, or saved-layout paths.

**Patterns to verify:**

- `MyClubPageContent::onManagedClubSaved` and existing route composition invalidations.
- `ManagedClubSelector` fieldset layout and `onSaved` callback.
- `usePlayerTableStore::addColumns` for append-once behavior.
- `savesQueryOptions` and `SaveSummary.contextToken` for the established active save context.
- `AppTopBar` and Settings `invalidateCurrentContext`, with `src/app/app-top-bar.test.tsx` and `src/app/routes/settings.test.tsx`, for active-save/current-context Query roots.
- Existing My Club dynamic-column, deferred-context managed feature, and managed-club route tests plus the Tauri IPC smoke stub.

**Constraints and non-goals:**

- Route composition only; managed-club code must not import Club DNA.
- Do not key the definition to `managedClub.clubName` or remove it when that name changes.
- Do not remove `club_dna` from layouts, filters, sort, local storage, or history on definition removal.
- Do not add the metric to Moneyball layouts or default layouts.
- Keep scores, definition validation, and authoritative stale-context rejection in Rust. Frontend context keys, remounting, and callback guards are required defense against stale rendering and side effects, not persistence authority.

**Dependencies and sequencing:**

- Depends on Commits 2–10. It is the final implementation packet and moves the feature to Validation only after its full checkpoint clears.

**Validation:** `./scripts/dev test src/features/club-dna/components/club-dna-definition.test.tsx src/app/routes/my-club-squad.test.tsx src/app/app-top-bar.test.tsx src/app/routes/settings.test.tsx src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx`; `./scripts/dev test`; `./scripts/dev check`; `./scripts/dev smoke`

**Stop conditions:** Stop if placement requires a cross-feature import, if My Club cannot derive and key the feature by the saves query's active ID/token, if create-versus-edit cannot be based on the backend result, if a save change can leave a draft open or allow prior-context data/feedback/layout append, if managed-club or Club DNA refresh/mismatch/error cannot disable interaction, if remove requires deleting catalog/layout/URL state, if Rust cannot remain the stale-context rejection authority, if smoke needs score computation in the stub, or if native-only behavior becomes required for acceptance without an available native environment.

**Review mandate:** Verify (1) exact placement and disabled state covers managed-club plus context refresh/mismatch/error, (2) route-only composition derives the active ID/token and key/remounts by both, (3) deferred save-switch tests prove no stale render, feedback, invalidation, or create-only layout append during open/set/remove paths, (4) Rust remains authoritative for stale get/set/remove rejection, (5) edit/remove/re-create and saved layout/filter/sort/history retention remain exact, (6) AppTopBar and Settings production invalidations and named tests include Club DNA, (7) proportionate route/component/smoke tests use context-bearing mocks without computing scores, and (8) no Moneyball/Profile/cache or frontend score scope creep.

## Active work

**PR:** PR 1 — Add user-defined Club DNA scoring

**Commit:** Commit 8 — Integrate indexed queries with the completeness fast path

### RED or removal proof

Add focused `club_dna_completion_fast_path_` tests that fail while the fixed metric, request context, exact-count completeness, no-definition filter leaves, and indexed Search/Squad query paths are absent. Reproduce both exact benchmark scales before accepting the implementation.

### Expected outcome

Indexed cache-first queries plus exact-count completeness deliver bounded Club DNA display, filtering, and sorting with full correctness and warm p95 below 500 ms at 2,000 players and at most 1,000 ms at 184,000 players.

### Explicit exclusions

Migrations, v33 indexes, ADR changes, definition/cache lifecycle changes, frontend code, eager ingest or promotion, Moneyball, Player Profile, background work, progress/cancellation, and current-state documentation.

## Discoveries and replanning

- The original direct read-time SQL plan was disproved by complete-catalog 2,000-player measurements and replaced by the accepted lazy cache in ADR-0023. Commits 3–4 recorded and implemented that cache foundation.
- Commit 6 recorded the exact-count completeness correction at `8250dbe9aac7853ed90ba674f83a67da870a8ecb`. The failed Commit 7 six-path worktree was discarded cleanly, so replacement implementation starts fresh from that HEAD.
- The discarded exact-count/indexed 2,000-player run reported every shape: Search filter cold 22.367 ms and warm p95 3.575 ms; Search sort cold 25.969 ms and warm p95 5.841 ms; Squad sort cold 25.543 ms and warm p95 7.280 ms. Every warm shape meets `<500 ms`.
- The discarded exact-count/indexed 184,000-player run also reported every shape: Search filter cold 2496.594 ms and warm p95 373.344 ms; Search sort cold 2644.854 ms and warm p95 509.068 ms; Squad sort cold 2904.600 ms and warm p95 856.517 ms. Every warm shape meets the developer-chosen `<=1,000 ms` threshold.
- Exact-count completeness removed warm full-cohort UID enumeration and batched miss probes. The remaining representative cost is sorting and joining large exact queries. Cold results are delayed first-use evidence and remain separate from warm acceptance.
- The developer rejected migration v33 directional, null-ordering, and player-join indexes. The v32/count-fast-path design meets the accepted shared threshold, while more indexes would add persistent storage, write amplification, query-specific variants, migration and EXPLAIN obligations, and maintenance complexity.
- Commit 8 keeps exact-count completeness and cache-first indexed reads. Count equality skips UID allocation, every materializer call, and every batch miss probe. Count mismatch uses the existing resumable materializer. Search and Squad retain the exact cohort-transition, nullable-row, stale-version, missing-definition, binding, and pagination contracts already planned.
- This threshold and packet change invalidated Delivery fingerprint `8cb75ca05c99424144dd6662ea12953b27b482a00e5b3cd96df3a7acce96c4ea`. Independent review cleared the latency decision, the developer accepted it and re-invoked delivery, and the ledger records replacement fingerprint `3fea871899c7615d5449aaddb1ac13d0395b5d8473fa146389c2f734cfb0ca18`.
- Each size-specific benchmark test executes and reports Search filter, Search sort, and Squad sort before asserting `<500 ms` at 2,000 players or `<=1,000 ms` at 184,000 players. Both retain separate cold measurements, 3 warm-ups, 20 measured samples, and nearest-rank p95. Any warm breach requires replanning; do not add v33 indexes without a new developer decision.
- The frontend attribute catalogs remain duplicated between Player Profile grouping and player metric metadata. Commit 9 still consolidates them into a shared utility; Rust retains its independent authoritative catalog.
- `.wiki/TODO.md`, `.wiki/BACKLOG.md`, `.wiki/ARCHITECTURE.md`, and `.wiki/DESIGN.md` remain unchanged during this bounded replan. ADR-0023 records only the accepted threshold, rejected alternative, delayed cold evidence, consequences, and current implementation status.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Add user-defined Club DNA scoring | Commit 1 — Record the approved feature plan | ddd4961e6d90ca24faa435955c6ae7eb5a716f0b | Recorded the reviewed schema 2 ledger and TODO activation. | `ledger_state.py`: runnable; `git diff --cached --check`: passed. | Not applicable | Clear | 0 | None. |
| PR 1 — Add user-defined Club DNA scoring | Commit 2 — Persist one save-owned Club DNA definition | d2682ee5c50cb99cd0b7f9facf5fd4f9060d5001 | Added migration v31 and context-bound Rust CRUD for one validated definition per save. | RED failed because v31 was absent; `./scripts/dev check-rust` passed 561 tests with 2 ignored; `./scripts/dev check` passed. | Pass | Clear | 0 | None. |
| PR 1 — Add user-defined Club DNA scoring | Commit 3 — Record the approved cache replan | 7cf5e5924af8a9c54852f5037e17ffe4b2c58cc0 | Recorded ADR-0023 and the reviewed lazy-cache packets after measured direct-SQL failure. | Both classifiers were runnable with the accepted fingerprint; staged diff and Markdown checks passed. | Not applicable | Clear | 0 | Replaced the direct-SQL packet after measured 2,000-player threshold breaches. |
| PR 1 — Add user-defined Club DNA scoring | Commit 4 — Add the versioned Club DNA score cache | d78f97f25497409f6c895a8ac5cdeb74ea5301eb | Added v32 definition versioning, pure scoring, bounded nullable materialization, and atomic definition and player-boost invalidation. | RED failed because the cache owner was absent; focused rollback tests passed; `./scripts/dev check-rust` and `./scripts/dev check` passed 569 tests with 2 ignored. | Pass | Clear | 2 | Review corrections removed repeated definition validation, strengthened batch and late invalidation rollback proof, and restored adjacent role-score rollback coverage. |
| PR 1 — Add user-defined Club DNA scoring | Commit 5 — Record the indexed query-shape correction | df074b9da78bec038960e2be7c851dcb5879dbdd | Recorded the representative correlated-query breach, indexed cache-first correction, and ADR implementation status. | Both classifiers were runnable with the accepted fingerprint; staged diff and Markdown checks passed. | Not applicable | Clear | 0 | Replaced the correlated cache-read packet after the 184,000-player warm threshold breach. |
| PR 1 — Add user-defined Club DNA scoring | Commit 6 — Record the warm-cache completeness correction | 8250dbe9aac7853ed90ba674f83a67da870a8ecb | Recorded the representative warm materializer overhead and exact-count completeness fast path. | Both classifiers were runnable with the accepted fingerprint; staged diff and Markdown checks passed. | Not applicable | Clear | 0 | Replaced warm full-cohort UID enumeration and batch probes after all 184,000-player shapes breached. |
| PR 1 — Add user-defined Club DNA scoring | Commit 7 — Record the representative latency decision | Pending record | Recorded the developer-chosen one-second representative threshold and rejected v33 index alternative. | Both classifiers were runnable with the accepted fingerprint; staged diff and Markdown checks passed. | Not applicable | Clear | 0 | Replaced the former 200 ms representative gate after exact v32/count-fast-path measurements. |

## Final validation

- `./scripts/dev test` — all frontend component, route, store, catalog, adapter, and IPC-mock tests pass and discover the new Club DNA tests.
- `./scripts/dev check` — Biome, TypeScript, full-tree secretlint, Rust format, Clippy, and all Rust tests pass, including v32 upgrade, definition/version invalidation, pure scoring, materialization scope, boost rollback, Search, filter, Squad, active-save, null, and performance guards.
- `./scripts/dev smoke` — Chromium proves the My Club create flow, Modal explanation/selection, and fixed table-column integration through the browser IPC stub.
- Inspect the exact feature diff with `git diff --check b573420893da93d91ddaee66ff9a4038f800b6d9...HEAD` and the delivery workflow's exact recorded commit set.
- `cd src-tauri && cargo test club_dna_completion_fast_path_ -- --nocapture --test-threads=1` — run before benchmarks. Require every new correctness test for definition context, completeness, nullable/stale/partial/boost behavior, cohort transitions, no-definition flat AND/OR behavior, and zero UID enumeration to use the exact stable prefix and be discovered by this command. Complete exact-version Search and Squad caches, including nullable rows, must perform zero UID enumeration, materializer calls, and batch miss probes. Missing, partial, boosted, stale, and definition-version cases must take the correct mismatch path. Separate cohort-transition tests must prove unmaterialized additions/entries mismatch and materialize, player removals cascade both Search counts, club exits leave both Squad predicates equal, Search-materialized entrants and already-complete club switches remain equal and skip safely, and outside-club rows do not false-pass. No-definition display/sort must retain all players with null UID-stable results and zero cache work; isolated and mixed AND Club DNA rules must be empty while mixed OR retains non-Club-DNA matches.
- `cd src-tauri && cargo test club_dna_indexed_query_shapes_on_complete_catalog_2k -- --nocapture --test-threads=1` — run the exact Commit 8 deterministic production-path workload at `N = 2,000`: create only UIDs 1..N, put all `N` players in exact club `Benchmark FC`, include all selected keys, use values `((uid - 1) % 20) + 1`, and set one selected personality value to null for each `uid % 100 == 0`. Do not add an outside-club player; separate focused Squad correctness tests own nonmember exclusion. Assert all three final DTO results before timing: General Search total is N; Search `club_dna gt 50` under default CA-descending order with display `club_dna`, limit 50, offset 0 has total 980; Search `club_dna` descending with the same display/page has total N, null-last, score-100 ties then UID ascending; Squad `club_dna` ascending over exact club membership with display `club_dna`, limit 50, offset 50 has total N and correct tied score/UID order. Exclude fixture construction. For each shape, clear and record cold first use separately, prefill, run 3 warm-ups and 20 samples. This test must execute and report all three shapes before asserting every p95 `<500 ms`.
- `cd src-tauri && cargo test club_dna_indexed_query_shapes_on_generated_184k -- --ignored --nocapture --test-threads=1` — run the same exact production functions, final DTO assertions, fixture formula, requests, cache cycle, and 3+20 method with only the `N = 184,000` `Benchmark FC` members and no outside-club player; General Search and Squad totals are N, and the Search filter total is 90,160. This test must execute and report its Search filter, Search sort, and Squad sort shapes before asserting every p95 `<=1,000 ms`. The preceding 2,000-player test separately reports its three shapes before asserting every p95 `<500 ms`. The two processes together cover six combinations; neither process depends on a cross-process aggregate. A measured warm breach requires replan and cannot be accepted as a gap. Treat the prior 243.8 ms Squad sort-page diagnostic median as review focus only, not acceptance evidence.
- Inspect shape-specific `EXPLAIN QUERY PLAN` assertions. Search filter must use the committed v32 `(snapshot_id, definition_version, score_model_version, score)` index for exact-version count and candidate selection; after joining those score-filtered candidates to players, allow its required temporary sort for final CA/UID ordering and bounded page selection. Search Club DNA sort and Squad Club DNA sort must use that index for exact score selection and score order; allow only the documented temporary sort needed for null-last ordering and the UID tie-break that v32 does not cover. Reject correlated definition lookups and unbounded player-first global scans.
- Manually verify first-use behavior in the native app. Record the visible duration and confirm the UI does not falsely appear warm or return partial filter/sort results while materialization is in progress. If cold work makes the app unusable rather than delayed, stop for a progress/cancellation architecture decision.
- Manually verify the native Modal at 1280×800 and 1600×900: full-catalog scrolling, keyboard selection, edit ↔ remove-confirmation transitions in one Modal, confirmation Cancel/Escape return, edit Cancel/Escape discard, pending-removal dismissal blocking, remove-error return path, successful-removal focus return, and no layout shift. Chromium does not replace this check.
- `./scripts/dev bridge-test` is outside the affected bridge path. `./scripts/dev mutate` remains unsupported. Neither may be reported as passed.

## Documentation impact

During this bounded replan, change `.wiki/features/active/club-dna.md` and ADR-0023's threshold, rejected alternative, delayed cold evidence, consequences, and implementation-status sentence. Leave `.wiki/TODO.md`, `.wiki/BACKLOG.md`, `.wiki/ARCHITECTURE.md`, and `.wiki/DESIGN.md` unchanged. During feature reconciliation after implementation, update `.wiki/ARCHITECTURE.md` for v31/v32 definition persistence, lazy versioned Club DNA materialization, indexed query scopes, and invalidation owners; update `.wiki/DESIGN.md` for the implemented My Club action and definition Modal; update `.wiki/TODO.md` for completion; preserve ADR-0023; and move this ledger to `.wiki/features/completed/club-dna.md`.
