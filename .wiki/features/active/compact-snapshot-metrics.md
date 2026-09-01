# Compact Snapshot Metrics and Load Progress

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** ffd74f8f3bc4d94484b753ac12a5d35874bc547e34fd9f1362ab6ba1d575dfaf

## Intent

Replace the normalized player and staff role-score row explosion with compact, directly queryable metrics for each effective current snapshot. Keep historical raw facts, preserve atomic snapshot publication, start a fresh database generation, and make Load Data report truthful phase progress and detailed timings without adding concurrency infrastructure.

## User-visible behavior

- Search, Squad, Player Profile, Planner, Staff Search, My Club Staff, Staff Shortlist, and Staff assignment optimization keep their existing role metric behavior, including arbitrary sort and filter across all 68 current and 68 potential player metrics and all 21 current staff metrics.
- Load Data keeps the prior snapshot and visible Search and Squad results usable during scan, validation, preparation, and scoring.
- Load Data shows a compact phase label and native progress bar in the existing top-bar outcome region. Scan is indeterminate. Later phases show determinate counts only when the backend has truthful totals.
- The Load Data button uses the active phase label. The existing success or error outcome remains, and the final command result stays authoritative if a best-effort progress message is missed.
- A successful effective-current replacement atomically exposes the new snapshot and clears or invalidates the established result roots. A failed load preserves the old snapshot and visible data.
- Switching saves while a load runs cannot show progress or an outcome from the captured prior save context.
- The final outcome reports scan, preparation and scoring, database save and finalization, and total timings.
- The new build uses a new database file, such as `app-v2.db`. It never opens or changes the existing `app.db`. After the new database is verified, the user can manually remove the old file to reclaim disk space; reinstall alone might not remove application data.

## Invariants

- The closed player catalog contains 68 safe snake_case role IDs. One `player_role_metrics` row exists for every player in each effective current snapshot and for no historical snapshot. Each row has nullable named current and potential columns plus explicit score and projection model versions.
- The closed staff catalog contains 21 safe snake_case role IDs. One `staff_role_metrics` row exists for every staff member in each effective current snapshot and for no historical snapshot. Each row has nullable named role columns plus an explicit score model version.
- A missing source attribute produces a null metric. Null is not zero, missing, or an absent compact row.
- Current-only projected player attributes remain in the existing nullable JSON and projection-model fields on `players`. Historical snapshots contain neither projected attributes nor player or staff metric rows.
- Raw player and staff facts remain immutable for every retained snapshot. A future individual Player Profile timeline recalculates historical projections and role scores with the then-current model; this feature does not implement that UI.
- Trusted schema and query identifiers come only from closed-catalog lookup followed by safe snake_case validation. WebView input never becomes an SQL identifier.
- The first compact migration owns one immutable checked-in inventory of exactly 68 player current columns, the matching 68 player potential columns, and 21 staff columns. Historical migration DDL never derives from mutable runtime catalogs. Runtime closed-catalog mapping must match that exact schema and model contract.
- Later catalog, formula, projection, or model-version changes add a new migration and model version. They never mutate the first compact migration or its DDL.
- Snapshot publication uses one `Db(Mutex<Connection>)`, one `rusqlite` connection, one SQLite writer, and one final transaction. The previous snapshot remains current until commit.
- A different effective current snapshot clears the old derived rows only in the publication transaction. Deletion promotion rebuilds compact metrics from retained raw facts before commit. A supported player boost atomically replaces that player's raw values, projected attributes, and current and potential compact metrics.
- Club DNA and Moneyball formulas and ownership do not change. Existing scalar indexes remain. No per-role index is added without representative failure and replanning.
- Tauri progress is command-scoped and best effort. Phases are ordered and phase-local. The final result is authoritative. No weighted overall percentage is shown.
- Accessibility uses a text phase label plus native `<progress>`. Progress does not rely on color alone.

## Non-goals

- Historical timeline, comparison, chart, Search, Planner, Academy, Staff, or Moneyball UI.
- Historical projected attributes or historical player or staff role-score matrices.
- Data-preserving upgrade, legacy import, compatibility read, `VACUUM`, automatic old-file deletion, or opening the existing `app.db`.
- Connection pool, WAL conversion, second SQLite driver, Rayon or another dependency, multiple writers, detached jobs, cancellation, job center, modal, toast history, or new global store.
- Club DNA or Moneyball formula or ownership changes.
- New per-role indexes without a representative acceptance failure and replan.
- Performance claims without representative before-and-after measurements.
- The ignored 184k or 500k ingest tests and any query against the live legacy database.

## Current-state map

- Relevant components: `src-tauri/src/features/snapshot/{commands,load_data,ingest,service}.rs` compose scan, ingest, current selection, and timings. `src/features/memory-read/{api,hooks,types,components}` and `src/app/components/app-top-bar.tsx` own Load Data UI composition.
- Data model: `src-tauri/src/db/migrations.rs` creates normalized `player_role_scores`, `player_potential_role_scores`, and `staff_role_scores`. Migration v34 adds current-only projected JSON fields to `players`. `src-tauri/src/db/mod.rs::APP_DB_FILE` is `app.db`; `Db` wraps one `Mutex<Connection>`.
- Player writers: `snapshot/ingest.rs::insert_role_scores` writes 68 current rows for every retained snapshot. `player_metrics/potential_scores.rs` writes 68 current-only potential rows and projected JSON. `snapshot/service.rs::select_current_snapshot` owns selection and promotion. `player/service.rs::reconcile_verified_boost` and `replace_role_scores` own one-player atomic reconciliation.
- Player readers: `player_metrics/resolver.rs`, `search/{filter,query}.rs`, `planner/{squad,depth,optimizer,role_reference}.rs`, and `player/query.rs` query normalized relations. Their inline and sibling test modules seed, corrupt, and assert those relations.
- Staff writers and readers: `snapshot/ingest.rs::insert_staff` writes only calculable normalized staff rows. `staff/{metrics,filter,query,service,assignment_optimizer_query}.rs` read the relation, with tests in those modules and `assignment_optimizer_query_tests.rs`.
- Existing progress analogue: `player/commands.rs::SquadPlayerBoostProgressDto` and staff batch progress use `tauri::ipc::Channel`; frontend Squad and Staff APIs construct `Channel`, and Squad renders native `<progress>`.
- Existing UI behavior: `use-load-data.ts` clears Search and Squad result roots before Tauri and invalidates broad roots on settlement. `AppTopBar` captures save ID and context token for outcome suppression. `LoadDataOutcome` owns the persistent polite live region and success/error banner.
- Measured evidence: a 250,000-player load performs about 34 million current and potential player score inserts; a later load deletes about 17 million potential rows. A live inventory contained about 134.8 million current and 55.7 million potential player-role rows in an approximately 31 GiB database. Core projection and scoring took about 6.2 seconds for 250,000 × 136 metrics, so persistence is dominant.
- Directional prototype: 250,000 wide player rows with 136 nullable scores occupied about 79 MiB, inserted in about 2.5 seconds, and sorted an unindexed role column in about 56–66 ms. This is planning evidence, not an acceptance promise.
- Architectural seams: ADR-0015 keeps Rust-owned `rusqlite` and one connection. ADR-0028 supersedes ADR-0026 and the normalized scope of ADR-0027, and amends ADR-0025 without changing its UI ownership or selective scalar indexes.
- Project validation commands: `./scripts/dev test`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke`. Windows native validation uses `pnpm tauri dev` with representative FM data.
- Primary risks: wide-schema identifier drift, nullable value loss, current/historical ownership mistakes, partial publication, missed consumer cutovers, long final mutex hold, stale progress after save switching, and unsupported performance claims.

## Feature architecture

`scoring::catalog` and `staff::scoring` remain the closed runtime model catalogs. The first compact migration checks in one immutable static inventory of exactly 68 player current columns, 68 matching player potential columns, and 21 staff columns; it never generates historical DDL from those mutable runtime catalogs. A narrow compact-metric helper validates each trusted safe snake_case role ID and maps it to the checked-in schema/model contract for trusted dynamic queries and writes. Contract tests compare the full runtime mapping with that exact migration inventory. Later catalog or formula changes require a new migration and model version rather than edits to old DDL. Static checked-in migration SQL is preferred where it preserves this immutability.

Snapshot preparation produces validated raw rows and compact metric values before final persistence when practical. The publication service acquires the existing database mutex only for context revalidation and one transaction that inserts raw rows, writes compact rows for the effective current snapshot, clears derived state from a displaced current snapshot, updates `is_current`, and commits. A non-winning retained snapshot stores raw facts only.

Player and staff readers keep their current typed metric IDs and DTOs. The resolver maps validated role IDs to compact columns. Search and Squad use the named columns directly for projection, filter, and `ORDER BY`; Profile, Planner, and Staff consumers decode only catalog-owned columns. Read validation checks the one compact row, expected model versions, and requested values rather than counting normalized role rows.

Load Data accepts a command-scoped typed Tauri channel. Backend phases are `scan`, `preparing`, `scoring`, `saving`, and `finalizing`; each event carries the captured save context, an ordered phase, and optional truthful completed and total counts. The frontend keeps this progress in the mutation/component owner, not Zustand. The AppTopBar ignores messages and outcomes whose save ID or context token no longer matches the captured operation.

## Uncertainty register

### Known

- The player and staff catalogs are closed at 68 and 21 roles.
- Current source and tests query all three normalized score tables across the named consumer modules.
- The existing single connection and mutex are architectural constraints, not temporary implementation details.
- The old `app.db` must remain untouched.

### Assumptions

- Safe snake_case validation can accept every current trusted role ID without renaming the public metric IDs.
- The local wide-row prototype is directionally representative enough to justify implementation, but only Windows product validation can accept performance.
- The current Tauri Channel pattern supports command-scoped best-effort Load Data progress without a capability change.

### Decisions

- Use two PRs. PR 1 isolates the risky, independently mergeable persistence and query foundation. It changes the database generation, schema, lifecycle, every backend consumer, and deletion of legacy paths. Merging and measuring this foundation before UI orchestration limits the review surface and gives PR 2 a stable compact publication seam.
- Use a wide row with named nullable columns, not normalized rows or a blob.
- Keep projected attribute JSON on `players`.
- Use a fresh database filename and manual legacy cleanup.
- Keep one writer and one final transaction. Replan instead of adding concurrency if compaction does not meet acceptance.

### Unknowns

- Exact production phase timings and final `app-v2.db` size on a fresh representative Windows load.
- Whether any unindexed role sort exceeds acceptable interactive latency in the production query shape.
- Exact final write-lock duration after preparation moves outside the mutex.

### Risks

- A catalog/schema mismatch could make a trusted role unreadable or map it to the wrong column.
- A mixed old/new consumer seam could pass focused tests while one Planner or Staff path still references a removed relation.
- Promotion and boost rollback defects could expose partial derived state.
- Broad settlement invalidation could clear valid old data on failure; delayed progress could describe the wrong save.
- The fresh filename can leave both databases on disk until manual cleanup, increasing temporary disk use.

## Walking skeleton

PR 1 creates the fresh compact schema, writes one current player row, reads one validated current and potential role through Search, and proves historical snapshots have no derived rows before expanding the cutover to all consumers and staff. PR 2 then prepares outside the mutex, streams one command-scoped phase sequence, preserves old rows during work, and renders the sequence in the existing top-bar banner region.

## Delivery plan

### PR 1 — Compact active snapshot metrics

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/compact-snapshot-metrics

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** GitHub required strict status `check`

**Feature close-out:** Not required

**CI repair rounds:** 0

**Provisional PR title:** `perf(scoring): compact active snapshot metrics`

**Purpose:** Deliver and measure the risky persistence/query foundation as one independently mergeable change. It starts a fresh database generation, makes compact metrics authoritative for all backend behavior, and removes normalized score persistence before progress and UI orchestration depend on it.

**Depends on:** Clean `main` at `ea9b4ea646b6212ba1bc96045dbc9d9efda5b16c`; accepted plan review and Delivery fingerprint.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(scoring): record compact metrics feature plan`

**Work:** Commit the independently reviewed planning artifacts before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, GitHub changes, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted review result and commit only the exact reviewed planning paths.

**Files and responsibilities:**

- `.wiki/features/active/compact-snapshot-metrics.md` — feature intent, architecture, delivery authority, and commit packets.
- `.wiki/TODO.md` — Active feature pointer while preserving the gender item under Next.
- `.wiki/BACKLOG.md` — narrow historical player work to an individual Profile timeline with current-model recomputation.
- `.wiki/decisions/0025-selective-index-driven-player-sorts.md` — narrow compact-representation amendment pointer.
- `.wiki/decisions/0026-eager-current-potential-scoring.md` — preserve history and mark the normalized design superseded.
- `.wiki/decisions/0027-scoped-potential-read-validation.md` — mark normalized-row validation superseded while retaining scoped-read principle.
- `.wiki/decisions/0028-compact-current-snapshot-metrics.md` — accepted compact-metric and fresh-database decision.
- `.wiki/decisions/README.md` — decision index state.

**Behavior and data flow:**

- Record one reviewed ledger as the active source of delivery truth. No planned feature spec exists or is removed.

**Ordered implementation steps:**

1. Verify branch and base without changing Git state beyond the separately authorized activation.
2. Confirm the worktree contains only the eight reviewed planning paths.
3. Run `ledger_state.py`, `delivery_state.py`, and `git diff --check` for the exact eight planning paths, then inspect the complete planning diff.
4. Stage only these paths for independent checkpoint review.

**Tests and proof:**

- Not applicable — planning documents only. `ledger_state.py` proves schema and state consistency, `delivery_state.py` proves the reviewed recorded fingerprint and delivery authority, and `git diff --check` proves whitespace integrity for the exact eight-path planning diff.

**Patterns to verify:**

- `.wiki/features/active/README.md`, the ADR format in `.wiki/decisions/README.md`, and TODO/BACKLOG ownership rules.

**Constraints and non-goals:**

- Do not alter scope, packet order, implementation, tests, configuration, branches, or unreviewed documentation.

**Dependencies and sequencing:**

- Requires a clear independent plan review, developer acceptance, a non-pending Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/compact-snapshot-metrics.md && python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/compact-snapshot-metrics.md . && git diff --check -- .wiki/features/active/compact-snapshot-metrics.md .wiki/TODO.md .wiki/BACKLOG.md .wiki/decisions/0025-selective-index-driven-player-sorts.md .wiki/decisions/0026-eager-current-potential-scoring.md .wiki/decisions/0027-scoped-potential-read-validation.md .wiki/decisions/0028-compact-current-snapshot-metrics.md .wiki/decisions/README.md`

**Stop conditions:** Stop on an uncleared review, classifier error, unreviewed path, substantive plan change, a missing or mismatched reviewed recorded Delivery fingerprint, a diff-check error, or branch mismatch.

**Review mandate:** Verify the complete accepted planning outcome, exact eight-path scope, schema 2 state, ADR supersession, and absence of implementation changes.

#### Commit 2 — Create the fresh compact metric schema

**Status:** Completed

**Provisional commit:** `feat(db): create fresh compact metric schema`

**Work:** Point the app at a new database file and add constrained compact player and staff metric tables plus trusted catalog-to-column mapping while retaining normalized tables only as a short-lived sequencing seam.

**Size assessment:** Likely above 200 non-test lines because the immutable checked-in 68-current/68-potential/21-staff column inventory, DDL constraints, and mapping validation form one atomic schema boundary. Prefer static checked-in migration SQL over runtime generation because it preserves historical migration immutability.

**Out of scope:** Reader cutovers, normalized-table deletion, load progress, UI, per-role indexes, and old-file conversion.

**Implementation packet:**

- Make a fresh database generation unavoidable and establish one validated identifier owner used by later writers and readers.

**Files and responsibilities:**

- `src-tauri/src/db/mod.rs::APP_DB_FILE` and tests — select `app-v2.db` and prove path resolution/opening never targets `app.db`.
- `src-tauri/src/db/migrations.rs` — add one immutable checked-in compact migration with the exact 68 player current, 68 player potential, and 21 staff column inventory, plus row/model/check/foreign-key constraints and schema inventory tests; do not derive its DDL from runtime catalogs or backfill legacy rows.
- `src-tauri/src/features/player_metrics/{mod.rs,compact.rs}` — validate closed safe snake_case player role IDs and map the runtime catalog to the exact checked-in current/potential schema and model contract.
- `src-tauri/src/features/staff/{metrics.rs,scoring.rs}` — expose the equivalent safe snake_case staff mapping and prove it matches the exact checked-in schema and model contract.
- `README.md` — document the user-visible `app-v2.db` filename, that `app.db` remains untouched, that reinstall might preserve application data, and that the user must verify the new database before manually deleting the old file.

**Behavior and data flow:**

- A fresh open creates `player_role_metrics(snapshot_id, uid, score_model_version, projection_model_version, <68 current>, <68 potential>)` and `staff_role_metrics(snapshot_id, uid, score_model_version, <21 roles>)` from immutable checked-in migration DDL, with one-to-one player/staff cascades and nullable 0–100 metric checks. Runtime catalogs map only to that exact versioned inventory. No role index is added. The old filename remains untouched because the process never resolves it. The README gives cleanup guidance at the same user-visible filename boundary.

**Ordered implementation steps:**

1. Add RED schema tests for the new filename, exact immutable 68-current/68-potential/21-staff migration inventory, one-row identity, model constraints, nullable score checks, cascades, and absence of per-role indexes.
2. Add closed-catalog safe snake_case validation and deterministic player/staff mapping, then prove the complete runtime catalogs match the exact migration schema/model contract.
3. Add the fresh compact schema migration as static checked-in DDL without runtime catalog derivation, data conversion, or backfill.
4. Document `app-v2.db`, untouched `app.db`, possible reinstall retention, and verification before manual old-file deletion in `README.md`.
5. Keep normalized tables temporarily so existing consumers remain buildable until later commits.
6. Run Rust, full, and documentation diff checks.

**Tests and proof:**

- Modify `db/mod.rs` and `db/migrations.rs` tests. RED must fail because `APP_DB_FILE` is still `app.db` and compact tables do not exist. GREEN proves the checked-in migration inventory stays exactly 68 player current, 68 matching player potential, and 21 staff columns; every runtime catalog role maps once to that exact versioned schema/model contract; invalid identifiers are rejected; nullable scores stay null; invalid scores/model versions fail; and the schema has no role indexes. Inspect `README.md` and run its diff check to prove the cleanup instruction names both files, reinstall retention, and verification before manual deletion.

**Patterns to verify:**

- Existing migration transaction tests, `scoring::catalog::all_roles`, `staff::scoring::all_staff_roles`, and ADR-0015's Rust-owned SQLite boundary.

**Constraints and non-goals:**

- Do not open, inspect, migrate, vacuum, import, or delete `app.db`. Do not derive historical migration DDL from mutable runtime catalogs. Do not mutate the first compact migration for later catalog/formula changes; add a new migration and model version. Do not add dependencies, dynamic WebView identifiers, blobs, generated checked-in code, or compatibility reads.

**Dependencies and sequencing:**

- Depends on Commit 1. Later commits must use this mapping owner before normalized tables are removed.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check && git diff --check -- README.md`

**Stop conditions:** Replan if any catalog role is not safe snake_case, the runtime catalog cannot match the exact immutable migration inventory and model contract, SQLite limits reject the accepted width, a current scalar index must change, or a production path would need the old database.

**Review mandate:** Check filename isolation and user cleanup safety, immutable checked-in migration DDL, exact 68/68/21 inventory, runtime catalog/schema/model parity, safe snake_case identifier trust, later-version migration ownership, null/score/model constraints, foreign-key ownership, no role indexes, no conversion path, README technical-writing accuracy, and the documented temporary normalized seam.

#### Commit 3 — Materialize current player metrics atomically

**Status:** Completed

**Provisional commit:** `refactor(scoring): materialize current player metrics`

**Work:** Add one compact player metric writer and make ingest, current selection, deletion promotion, and player boost reconciliation maintain exactly one current-only row per player while normalized writers remain temporarily for unread consumers.

**Size assessment:** Above the soft target is likely justified because one atomic lifecycle invariant spans materialization, demotion, promotion, and boost rollback. Split helper extraction only if every intermediate commit preserves publication and tests.

**Out of scope:** Player reader cutover, staff metrics, final normalized cleanup, preparation outside the mutex, and progress.

**Implementation packet:**

- Consolidate current and potential player derivation behind one writer that projects each player once and writes one wide row with exact null semantics and model versions.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/{compact.rs,potential_scores.rs,mod.rs}` — replace normalized potential writer internals with compact rebuild, replace-one-player, clear, completeness, and requested-role validation.
- `src-tauri/src/features/snapshot/{ingest.rs,service.rs}` — write compact rows only for the effective current snapshot, clear displaced derived state, and rebuild a promoted retained snapshot from raw facts.
- `src-tauri/src/features/player/service.rs::{reconcile_verified_boost,replace_role_scores}` — replace current/potential compact values and projected attributes in the existing player transaction.
- Existing inline tests in these modules — rewrite lifecycle and rollback assertions for compact ownership while retaining normalized assertions only where the temporary reader seam needs them.

**Behavior and data flow:**

- Raw ingest remains transactional. If the stored snapshot wins, the writer reads its raw players, calculates 68 current metrics, projects once, calculates 68 potential metrics, writes projected JSON and one compact row per player, then commits selection. A non-winner has no compact row or projected JSON. Promotion rebuilds before commit. A boost replaces one player's compact row and projected JSON with the raw update.

**Ordered implementation steps:**

1. Add RED lifecycle tests for winner, non-winner, replacement, deletion promotion, exact null columns, model versions, and rollback.
2. Implement compact score preparation and one-row persistence from the closed catalog.
3. Route `select_current_snapshot` reconciliation through compact clear/rebuild semantics.
4. Route supported player boost reconciliation through one-player compact replacement.
5. Keep the normalized writer only as a temporary dual-write/read compatibility seam and mark it for Commit 8 removal.
6. Run Rust and full gates.

**Tests and proof:**

- Modify tests in `player_metrics/potential_scores.rs`, `snapshot/ingest.rs`, `snapshot/service.rs`, and `player/service.rs`. Prove one compact row per current player, no historical derived state, exact null scores, one projection per rebuild, selection and boost rollback, prior-current visibility on failure, and immutable raw historical rows. Retain existing boost recovery behavior.

**Patterns to verify:**

- Current `potential_scores::{rebuild_snapshot,reconcile_current_selection,replace_player}`, `snapshot::service::select_current_snapshot`, and player boost transaction/recovery tests.

**Constraints and non-goals:**

- One writer and one transaction. No historical metrics, read-time calculation, pool, WAL, parallelism, dependency, or changed scoring formula.

**Dependencies and sequencing:**

- Depends on Commit 2 schema/mapping. Must land before any compact reader.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Replan if one-row replacement cannot preserve rollback, promotion lacks retained raw inputs, projected JSON must move tables, or model/catalog mapping cannot prove completeness.

**Review mandate:** Check current-only ownership, exact nulls, projection count, demotion timing, promotion atomicity, boost atomicity/recovery, raw history immutability, and the bounded temporary dual-write seam.

#### Commit 4 — Cut Search and Squad over to compact player metrics

**Status:** Completed

**Provisional commit:** `refactor(search): query compact role metrics`

**Work:** Make the shared resolver, Search filters/sorts/projections, and Squad table query named compact columns directly.

**Size assessment:** Likely above 200 non-test lines because resolver, filter, Search, and Squad must change together to keep arbitrary field behavior buildable. Keep Planner and Profile outside this commit.

**Out of scope:** Profile, Planner, staff, normalized-table deletion, UI behavior, and new indexes.

**Implementation packet:**

- Preserve all validated public metric IDs and nullable ordering while replacing relation subqueries and joins with direct compact-row expressions.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/resolver.rs` — resolve validated `role.*` and `potential_role.*` IDs to trusted compact columns and model-version predicates.
- `src-tauri/src/features/search/{filter.rs,query.rs}` — compile current/potential filters and arbitrary sorting against the one-to-one compact join.
- `src-tauri/src/features/planner/{squad.rs,squad_tests.rs}` — cut the Squad table adapter to the same direct expressions.
- Existing inline Search/resolver tests — replace normalized SQL-shape and corruption fixtures with compact rows.

**Behavior and data flow:**

- Search and Squad validate metric IDs through the closed catalog, join the current player's compact row once, read named nullable columns, and order/filter in SQLite. Potential requests require the expected projection model; current requests require the expected score model. Null stays null and follows existing ordering semantics.

**Ordered implementation steps:**

1. Add RED query tests that seed only compact rows and exercise arbitrary current/potential display, filter, ascending/descending sort, nulls, totals, and paging.
2. Change resolver SQL generation to trusted compact columns.
3. Change filter compilation and Search/Squad query assembly to one-to-one compact joins.
4. Remove normalized Search/Squad SQL-shape assertions and obsolete requested-role row-count helpers.
5. Run Rust and full gates.

**Tests and proof:**

- Modify `search/filter.rs`, `search/query.rs`, `planner/squad_tests.rs`, and resolver tests. RED must show compact-only fixtures are unreadable before cutover. GREEN proves arbitrary roles, exact nullable semantics, stable UID ties, current/potential version rejection, bounded pages, and no write or materialization on reads.

**Patterns to verify:**

- Existing `MetricField` closed validation, Club DNA bind handling, scalar sort indexes, and ADR-0025 null/tie behavior.

**Constraints and non-goals:**

- Preserve metric IDs, DTOs, filters, totals, paging, Club DNA, Moneyball, scalar indexes, and read-only queries. Add no per-role indexes or frontend calculation.

**Dependencies and sequencing:**

- Depends on Commit 3 complete compact player rows. Normalized writes remain only for uncut Profile/Planner readers.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Replan if direct wide-column sorting fails correctness, any role requires untrusted identifier interpolation, or a per-role index appears necessary before representative validation.

**Review mandate:** Check identifier safety, all 136 player metrics, null-last/current ordering parity, model versions, one-to-one join cardinality, totals/pages, Moneyball and Club DNA isolation, and no hidden writes.

#### Commit 5 — Cut Profile and Planner over to compact player metrics

**Status:** Completed

**Provisional commit:** `refactor(planner): read compact player metrics`

**Work:** Make Player Profile, Planner depth, optimizer, role reference, and candidate reads consume compact current/potential columns while preserving all existing fit and assignment behavior.

**Size assessment:** Above 200 non-test lines is likely atomic because shared Planner fixtures and all tactic-role consumers must move together to avoid a mixed unreadable state.

**Out of scope:** Staff metrics, normalized writer deletion, scoring formula changes, frontend changes, and historical UI.

**Implementation packet:**

- Replace role-row collections with catalog-directed compact projections. Keep Rust fit/combination logic and DTOs unchanged.

**Files and responsibilities:**

- `src-tauri/src/features/player/query.rs` — load Profile role scores from one compact row and preserve projected JSON validation.
- `src-tauri/src/features/planner/{depth,optimizer,role_reference}.rs` — select only tactic/catalog columns from compact rows and keep combination/allocation in Rust.
- `src-tauri/src/features/planner/{depth_tests,optimizer_tests,role_reference_tests,test_support}.rs` — seed and corrupt compact columns instead of normalized relations.

**Behavior and data flow:**

- Profile maps every catalog-backed display role to current/potential values from one player row. Planner validates tactic roles, selects their named columns, applies existing familiarity/foot/weight logic, and returns unchanged results. Missing/wrong-version compact state fails before a mutation that returns depth.

**Ordered implementation steps:**

1. Add RED compact-only Profile and Planner tests for mapped/unmapped roles, assignments, candidates, role reference, and both optimizer bases.
2. Change Profile loading and scoped compact validation.
3. Change Planner query builders and loaders to catalog-selected compact columns.
4. Rewrite shared fixtures and mutation preflight corruption tests.
5. Remove obsolete normalized Planner/Profile helpers while retaining surviving fit and allocation tests.
6. Run Rust and full gates.

**Tests and proof:**

- Modify the named Player and Planner tests. Prove existing Profile role lists, projected attributes, current/potential combined scores, candidate order, role reference, optimizer allocation, mutation rollback, missing values, and wrong model versions. Do not duplicate pure scoring tests.

**Patterns to verify:**

- Existing `combine_role_scores`, `phase_fit_score`, `lane_fit_score`, scoped-read validation from ADR-0027, and Planner mutation preflight order.

**Constraints and non-goals:**

- No changed tactic, fit, optimizer, Profile concealment, DTO, route, or current-only behavior. No historical timeline.

**Dependencies and sequencing:**

- Depends on Commit 3. Completes player reader cutover so Commit 8 can delete normalized player paths.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Replan if a Planner behavior depends on normalized row multiplicity, compact model validation cannot precede writes, or public DTOs must change.

**Review mandate:** Check all Profile/Planner consumers, current and potential parity, mapped/unmapped roles, null propagation, model checks, mutation ordering/rollback, optimizer allocation, and fixture cleanup.

#### Commit 6 — Materialize current staff metrics atomically

**Status:** Active

**Provisional commit:** `refactor(staff): materialize current staff metrics`

**Work:** Add one compact staff metric writer to current snapshot publication and promotion while retaining normalized staff writes briefly for unread consumers.

**Size assessment:** Within or near the soft target; lifecycle tests may be larger but are excluded from the estimate.

**Out of scope:** Staff reader cutover, player work, final normalized cleanup, progress, and staff formula changes.

**Implementation packet:**

- Calculate all 21 staff roles with exact null semantics and persist one current-only row per staff member with the model version.

**Files and responsibilities:**

- `src-tauri/src/features/staff/{metrics.rs,scoring.rs}` — compact row preparation, model version, and trusted column/value order.
- `src-tauri/src/features/snapshot/{ingest.rs,service.rs}` — current-only staff write, demotion clear, and deletion-promotion rebuild from raw staff facts.
- Inline ingest/service tests — one-row, null, history, promotion, and rollback proof.

**Behavior and data flow:**

- A winning snapshot calculates all staff roles and writes one row even when every metric is null. Historical snapshots retain raw staff only. Promotion calculates from retained `staff_attributes_json` before commit.

**Ordered implementation steps:**

1. Add RED tests for one row per current staff member, 21 named nullable values, no historical rows, and rollback.
2. Implement compact calculation and persistence in catalog order.
3. Integrate it with the same selection reconciliation as player metrics.
4. Keep normalized staff writes only until Commit 7 completes reader cutover.
5. Run Rust and full gates.

**Tests and proof:**

- Modify `snapshot/ingest.rs` and `snapshot/service.rs` tests. Replace the old “only calculable rows” expectation with one compact row whose unavailable roles are null. Prove winner/non-winner, replacement, promotion, final deletion, model version, and failed write rollback.

**Patterns to verify:**

- `all_staff_roles`, `score_staff_role`, player compact lifecycle from Commit 3, and existing staff score rollback tests.

**Constraints and non-goals:**

- No staff score formula or boost change. Staff CA boost does not rewrite role metrics because its source attributes do not change.

**Dependencies and sequencing:**

- Depends on Commit 2 schema and the lifecycle seam established by Commit 3.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Replan if any staff role cannot map to a safe column, staff formula ownership changes, or promotion cannot rebuild from retained facts.

**Review mandate:** Check exact 21-role coverage, null preservation, current-only ownership, promotion/rollback, model version, and the temporary dual-write boundary.

#### Commit 7 — Cut staff consumers over to compact metrics

**Status:** Pending

**Provisional commit:** `refactor(staff): query compact role metrics`

**Work:** Move all staff table, Profile, shortlist, filter, service, and assignment optimization reads to compact named columns.

**Size assessment:** Likely above 200 non-test lines because every staff consumer and shared query fixture must move atomically before relation removal.

**Out of scope:** Formula changes, UI changes, new indexes, player consumers, and normalized deletion.

**Implementation packet:**

- Keep public `role.*` IDs and result DTOs while reading one compact row with model validation.

**Files and responsibilities:**

- `src-tauri/src/features/staff/{metrics,filter,query,service,assignment_optimizer_query}.rs` — direct compact expressions, Profile catalog mapping, and optimizer candidate loading.
- `src-tauri/src/features/staff/{assignment_optimizer_query_tests,assignment_optimizer_tests}.rs` plus inline tests — compact fixtures and behavior proof.

**Behavior and data flow:**

- Staff metric IDs validate against the closed catalog and map to named columns. Search/Shortlist filters and sorts stay in SQLite. Profile emits all 21 catalog entries in order with nulls. Assignment optimization reads the required columns without multiplying candidate rows.

**Ordered implementation steps:**

1. Add RED compact-only tests for Profile, Search, My Staff, Shortlist, filters/sorts, and assignment candidates.
2. Change staff metric expressions and filter compilation.
3. Change Profile/service and assignment query decoding.
4. Rewrite fixtures and remove normalized SQL-shape assertions.
5. Run Rust and full gates.

**Tests and proof:**

- Modify the named tests. Prove exact nullable score semantics, all 21 Profile rows, arbitrary score filter/sort, shortlist joins, managed-club scope, assignment pool/order, no row multiplication, and wrong model rejection.

**Patterns to verify:**

- Existing staff `MetricField` trust boundary, bounded pages, `all_staff_roles` display order, and assignment optimizer role catalog.

**Constraints and non-goals:**

- Preserve DTOs, formulas, scopes, shortlist semantics, concealment, and CA boost behavior. Add no per-role index.

**Dependencies and sequencing:**

- Depends on Commit 6. Completes staff reader cutover for Commit 8 cleanup.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Replan if one-to-one compact joins change cohort counts, a staff consumer requires dynamic unvalidated identifiers, or role indexes appear necessary before manual measurement.

**Review mandate:** Check every staff consumer, 21-role completeness, nulls/model versions, filter binding, sort/tie behavior, shortlist and managed-club scope, optimizer cardinality, and test fixture value.

#### Commit 8 — Remove normalized score persistence

**Status:** Pending

**Provisional commit:** `refactor(scoring): remove normalized score rows`

**Work:** Delete normalized player/current/potential/staff writers, tables, indexes, helpers, and obsolete tests so the final PR 1 state has no normalized score-row persistence or compatibility path.

**Size assessment:** Likely a net deletion. Migration/schema inventory changes are atomic with removal even if touched lines exceed 200.

**Out of scope:** Progress/UI, legacy database conversion, unrelated migration cleanup, and historical timeline.

**Implementation packet:**

- Retire the normalized contract only after all writers/readers are compact. Prove surviving behavior and schema absence.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — final fresh-schema migration removes `player_role_scores`, `player_potential_role_scores`, `staff_role_scores`, and their indexes; update inventory tests.
- `src-tauri/src/features/snapshot/ingest.rs` — remove per-role normalized inserts and ponytail.
- `src-tauri/src/features/player_metrics/potential_scores.rs` and `player/service.rs` — remove obsolete row materialization/count/delete helpers.
- All affected Search/Player/Planner/Staff tests — remove obsolete normalized fixtures, triggers, and SQL-shape assertions; retain compact behavior proofs.

**Behavior and data flow:**

- Fresh `app-v2.db` ends with compact tables only. Runtime has no normalized write, read, dual-write, migration backfill, or compatibility helper. Historical raw facts and current compact rows remain authoritative.

**Ordered implementation steps:**

1. Establish contract-removal proof by listing remaining normalized symbol/table references and the compact tests that replace their supported assertions.
2. Delete normalized writers and helper APIs.
3. Drop normalized tables/indexes in the fresh schema path and remove obsolete migration hook behavior.
4. Remove or rewrite obsolete tests/fixtures/triggers in the same change.
5. Search source and tests for all three table names; allow only historical prose in ADR/completed records.
6. Run Rust and full gates.

**Tests and proof:**

- Contract-removal proof retires normalized persistence, not role behavior. Schema tests must prove the three tables and indexes are absent. Existing compact Search/Squad/Profile/Planner/Staff and lifecycle tests prove surviving behavior. Add an absence assertion because table reintroduction is observable and would recreate the storage regression.

**Patterns to verify:**

- Testing skill contract-removal procedure, migration inventory tests, and complete compact proofs from Commits 3–7.

**Constraints and non-goals:**

- No compatibility path or preservation of obsolete tests solely to keep old tables. Do not rewrite unrelated historical migrations or completed records.

**Dependencies and sequencing:**

- Depends on Commits 3–7. This is the final PR 1 implementation packet.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Stop if any production consumer still references a normalized table, any compact proof is missing, schema absence breaks a supported path, or the fresh database filename is not isolated.

**Review mandate:** Check zero runtime normalized references, schema/index absence, no stale helpers/tests, compact behavior parity, migration idempotence, old-file isolation, and no unrelated migration rewrite.

**PR 1 merge and manual acceptance:** Before publication, run a fresh representative Windows load of approximately 250,000 players without opening or querying `app.db`. Record actual scoring and final save timings, repeated-load behavior with retained history, arbitrary current and potential role sort/filter checks, staff role behavior, and `app-v2.db` size. Compare results honestly with the recorded baseline and prototype without claiming an unmeasured gain. Replan if repeated loads worsen with retained history, direct role sorting is not usable, or a per-role index or concurrency change is proposed. Record mutex blocking as a PR 2 baseline; judge the final lock boundary after PR 2 moves practical preparation outside it.

### PR 2 — Phased Load Data progress

**Status:** Awaiting prior PR merge

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/load-data-progress

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** GitHub required strict status `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(load-data): report phased loading progress`

**Purpose:** Build progress, timing, and retained-UI behavior on the merged and measured compact foundation. Keeping this separate prevents UI orchestration from obscuring persistence correctness and lets the branch start from the exact PR 1 merge on synchronized `main`.

**Depends on:** PR 1 merged; its immutable merge ref recorded and reachable from synchronized `main` before activation.

#### Commit 1 — Prepare snapshot data before publication

**Status:** Pending

**Provisional commit:** `refactor(load-data): prepare snapshots before publication`

**Work:** Split dump validation, raw row preparation, projection, and player/staff scoring from the final database transaction so practical CPU work occurs outside the database mutex.

**Size assessment:** Above 200 non-test lines is justified by the atomic preparation/publication boundary across ingest types and lifecycle validation. Keep one coherent prepared snapshot type rather than duplicate player/staff staging layers.

**Out of scope:** Tauri progress channel, frontend changes, parallelism, cancellation, pool/WAL, and formula changes.

**Implementation packet:**

- Introduce a bounded in-memory prepared snapshot owned by `snapshot::ingest`; final persistence revalidates captured context and publishes it in one transaction.

**Files and responsibilities:**

- `src-tauri/src/features/snapshot/{ingest.rs,load_data.rs,commands.rs}` — parse/validate and build raw/compact values before acquiring `Db`; pass prepared data to final save.
- `src-tauri/src/features/player_metrics/compact.rs` and staff compact helpers — expose pure preparation separate from transactional persistence.
- Existing ingest/load tests — preparation errors, context reuse, rollback, prior-current visibility, and timing boundaries.

**Behavior and data flow:**

- Capture save context under a brief lock, scan without it, parse/validate and calculate compact values without it, then reacquire once. The transaction revalidates the captured save, inserts raw facts, selects the winner, persists only current derived rows, and commits. Failure before or during commit leaves prior data visible.

**Ordered implementation steps:**

1. Add RED lock-boundary and failure tests around pure preparation and final publication.
2. Extract validated prepared raw rows and compact values without changing formulas.
3. Change final ingest to consume prepared values inside one transaction and revalidate context first.
4. Delete duplicate parse/score work inside the transaction.
5. Run Rust and full gates.

**Tests and proof:**

- Modify `snapshot/ingest.rs` and `snapshot/load_data.rs` tests. Prove preparation performs no database writes, context changes reject publication, failures preserve the current snapshot, one final transaction publishes raw and derived state, non-winners stay raw-only, and timing buckets do not overlap misleadingly.

**Patterns to verify:**

- Existing CSV import prepare-then-revalidate pattern, captured `SaveContext`, `load_data_after_scan_with_context`, and compact writer contracts from PR 1.

**Constraints and non-goals:**

- One writer, one connection, one final transaction. No threads, Rayon, dependency, detached task, cancellation, WAL, or pool.

**Dependencies and sequencing:**

- Requires PR 1 merge ref on synchronized main.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Replan if preparation requires holding a rusqlite row/transaction, memory use is unbounded beyond the accepted dump size, context revalidation cannot precede writes, or atomic publication weakens.

**Review mandate:** Check mutex scope, prepared-data bounds, context capture/revalidation, formula parity, one transaction, old-current visibility, rollback, non-winner history, and no concurrency additions.

#### Commit 2 — Stream ordered Load Data progress and timings

**Status:** Pending

**Provisional commit:** `feat(load-data): stream phased progress`

**Work:** Add typed command-scoped best-effort Tauri progress and detailed phase timings to the Load Data command and result.

**Size assessment:** Within or near the soft target; typed DTOs and phase tests may increase test lines.

**Out of scope:** Frontend rendering/cache behavior, weighted percentage, job state, cancellation, and persistence changes.

**Implementation packet:**

- Reuse the Squad boost Channel pattern. Emit ordered phase-local events tied to captured save context, and keep the final result authoritative.

**Files and responsibilities:**

- `src-tauri/src/features/snapshot/commands.rs` — `LoadDataProgressDto`, typed `Channel`, best-effort send adapter, command contract, and serialization tests.
- `src-tauri/src/features/snapshot/{load_data.rs,ingest.rs}` — phase callbacks and timings for scan, validation/preparation, scoring, database save/finalization, and total.
- `src-tauri/src/lib.rs` only if command registration signature proof requires adjustment; no new command.

**Behavior and data flow:**

- Emit scan as indeterminate. Emit determinate preparation, scoring, saving, or finalization counts only when completed and total are truthful. Events carry save ID/context token and monotonic phase identity. A failed send is ignored. Success/error still comes from the command result.

**Ordered implementation steps:**

1. Add RED serialization and service tests for phase order, truthful count shape, context fields, missed delivery, and detailed result timings.
2. Define the closed progress phase enum/DTO and sender callback.
3. Instrument existing boundaries without inventing work weights.
4. Preserve command errors and final result authority when channel sends fail.
5. Run Rust and full gates.

**Tests and proof:**

- Modify command/load/ingest tests. Prove phase order, indeterminate scan, determinate events only with valid totals, best-effort send failure, error phase boundaries, captured context, and nonnegative detailed timing coverage. Do not assert wall-clock speed in automated tests.

**Patterns to verify:**

- `SquadPlayerBoostProgressDto`, staff channel commands, current `LoadDataTimingsDto`, and Tauri camelCase serialization tests.

**Constraints and non-goals:**

- No overall weighted percentage, event bus, global state, retries, cancellation, detached work, or additional dependency.

**Dependencies and sequencing:**

- Depends on PR 2 Commit 1 phase boundaries.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Replan if Tauri Channel cannot be command-scoped with the existing dependency, truthful totals are unavailable for a claimed determinate phase, or progress requires a new runtime/capability.

**Review mandate:** Check typed boundary, phase ordering, count truthfulness, best-effort semantics, context binding, authoritative result, timing definitions, error mapping, and no weighted aggregate.

#### Commit 3 — Preserve visible data until successful replacement

**Status:** Pending

**Provisional commit:** `fix(load-data): retain results until publication`

**Work:** Change the frontend Load Data mutation to keep old Search/Squad data during work and clear/invalidate established roots only after a successful effective-current replacement.

**Size assessment:** Within the soft target.

**Out of scope:** Top-bar visual progress, new global store, backend changes, unrelated cache policy, and inactive historical-load behavior changes beyond truthful invalidation.

**Implementation packet:**

- Remove Load Data from `playerResultContextMutationKey` so the long scan/preparation operation does not trigger the Search and My Club blocking/unmount path. Move result-root clearing from pre-command to success after comparing stored and effective snapshot contexts. Bind progress/result ownership to the save captured at click time. Keep the neutral mutation key for actual save, snapshot, and managed-club context mutations.

**Files and responsibilities:**

- `src/features/memory-read/api/load-data.ts` — construct the typed Channel and pass progress to the command.
- `src/features/memory-read/types/load-data.ts` — progress phases, context, and expanded timings.
- `src/features/memory-read/hooks/use-load-data.ts` — local progress callback, captured context contract, removal of Load Data from `playerResultContextMutationKey`, context-matching success-only clearing/invalidation, and failure preservation.
- `src/app/player-result-context.ts` and AppTopBar/Search/My Club composition tests — keep exact root coordination, prove Load Data does not enter the neutral-key blocking path, and retain that key for actual save, snapshot, and managed-club context mutations.
- Frontend IPC mocks under `src/testing/` — ordered progress, delayed result, failure, and save-switch controls.

**Behavior and data flow:**

- Clicking captures save ID/context token. Load Data does not use `playerResultContextMutationKey`, so Search and My Club do not block or unmount result controllers for the long operation. Old Search and Squad rows remain mounted, readable, sortable, and activatable during scan and preparation. Only a context-matching successful stored snapshot that becomes effective current clears the exact Search/Squad result roots, then invalidates current owners. A stored historical non-winner refreshes history/outcome without falsely replacing current results. Failure preserves caches. Late progress/result from a different active context is ignored. Actual save, snapshot, and managed-club context mutations retain the neutral key and its blocking behavior.

**Ordered implementation steps:**

1. Add RED hook/composition tests for retained mounted and interactive rows, neutral-key exclusion, failure preservation, success replacement, historical non-winner, and save switch.
2. Add Channel creation and typed progress callback in the API.
3. Remove Load Data's `playerResultContextMutationKey` and pre-command clearing while preserving the neutral key on actual save, snapshot, and managed-club context mutations.
4. Add context-bound successful effective-current handling and exact root clearing/invalidation.
5. Keep broad owner refresh limited to the successful effective-current case or metadata owners that truthfully changed.
6. Run focused frontend tests, app gate, and full gate.

**Tests and proof:**

- Add or modify `use-load-data`, AppTopBar, Search, and My Club tests plus IPC mocks. RED must detect current pre-command clearing and Load Data's use of `playerResultContextMutationKey`. GREEN proves old Search and Squad rows stay mounted, readable, sortable, and activatable during scan/preparation; Search and My Club do not block or unmount for Load Data; actual save, snapshot, and managed-club context mutations still use the neutral key; failure preserves rows; exact clearing occurs only after a context-matching successful effective-current replacement; stale outcome/progress is ignored after save switch; and a historical non-winner does not clear current results.

**Patterns to verify:**

- ADR-0025 app-owned `clearPlayerResultContext`, current AppTopBar save context token, CSV import stale-context suppression, and TanStack Query mutation ownership.

**Constraints and non-goals:**

- No duplicated Query data, Zustand job state, response generation protocol, sibling-feature imports inside the memory-read feature, clearing before Tauri, or neutral-key blocking for Load Data. Preserve neutral-key behavior for actual save, snapshot, and managed-club context mutations.

**Dependencies and sequencing:**

- Depends on Commit 2 typed channel/result contract.

**Validation:** `./scripts/dev test src/features/memory-read src/app/components && ./scripts/dev check-app && ./scripts/dev check`

**Stop conditions:** Replan if existing context tokens cannot reject stale work, successful publication cannot distinguish effective replacement from historical storage, or exact root coordination must move across architecture boundaries.

**Review mandate:** Check Load Data neutral-key removal, preserved neutral-key behavior for actual context mutations, mounted/readable/sortable/activatable old rows during scan/preparation, success/failure/historical distinctions, context match before exact clearing, callback ordering, stale context suppression, exact cache roots, late promise/channel behavior, feature import boundaries, and mock realism.

#### Commit 4 — Show accessible phased Load Data progress

**Status:** Pending

**Provisional commit:** `feat(load-data): show top-bar progress`

**Work:** As the final planned implementation commit, render compact accessible phase text and native progress in the existing AppTopBar outcome/banner region, keep the button phase-specific, and expand final timing copy.

**Size assessment:** Within the soft target.

**Out of scope:** Modal, job center, toast history, new global store, new design tokens, animations, or weighted overall percentage.

**Implementation packet:**

- Extend the existing polite live region and banner layout. Use local mutation progress and the established design tokens.

**Files and responsibilities:**

- `src/app/components/app-top-bar.tsx` — phase-specific button labels, captured-context progress composition, and existing outcome placement.
- `src/features/memory-read/components/{load-data-outcome.tsx,load-data-outcome.test.tsx}` — pending phase surface, native determinate/indeterminate `<progress>`, success/error retention, and detailed timing copy.
- A focused AppTopBar test file or existing shell tests — button label, phase ordering, save switch, and accessible names.
- `e2e/smoke.spec.ts` only if the existing Load Data stub workflow needs a progress assertion; do not broaden smoke scope.

**Behavior and data flow:**

- Scan renders an indeterminate native progress bar and “Scanning…” text. Later events render their exact phase and determinate counts only when supplied. The button mirrors the current phase without changing width. The banner keeps a stable polite live region. Success/error replaces pending content and reports detailed timings.

**Ordered implementation steps:**

1. Add RED RTL tests using accessible roles/names for indeterminate and determinate phases, button labels, outcome replacement, and stale suppression.
2. Extend the existing banner component instead of adding a new surface.
3. Render native `<progress>` with omitted `value` for indeterminate work and real `max`/`value` for determinate work.
4. Keep success/error and dismissal behavior; expand timing labels without unsupported claims.
5. Run focused frontend tests, smoke if changed, app gate, and full gate.

**Tests and proof:**

- Modify `load-data-outcome.test.tsx` and add focused AppTopBar coverage. Prove visible text plus progress, semantic progress role/name, truthful determinate values, no invented percentage, phase-specific button labels, stable success/error, detailed timing labels, and stale context suppression. Retain existing truncated/historical outcome tests.

**Patterns to verify:**

- `.wiki/DESIGN.md` Top Bar, Button loading, semantic colors, mutation phase principle, existing `LoadDataOutcome` live region, and `SquadPlayerBoost` native progress.

**Constraints and non-goals:**

- Reuse current tokens/components. No modal, toast, history, job list, decorative motion, custom progress primitive, new store, or overall percentage.

**Dependencies and sequencing:**

- Depends on Commit 3 retained-data/context behavior.

**Validation:** `./scripts/dev test src/features/memory-read src/app/components && ./scripts/dev check-app && ./scripts/dev smoke && ./scripts/dev check`

**Stop conditions:** Stop if progress cannot be expressed with semantic native HTML, button width/layout regresses at 1280×800, save switching can expose stale work, or design changes require new tokens/owners.

**Review mandate:** Check accessibility, indeterminate/determinate semantics, text plus color, phase/button parity, live-region stability, outcome preservation, context suppression, top-bar density, and no new global surface.

## Active work

**PR:** PR 1 — Compact active snapshot metrics

**Commit:** Commit 6 — Materialize current staff metrics atomically

### RED or removal proof

RED tests for one row per current staff member, 21 named nullable values, no historical rows, and rollback fail while staff publication still writes only normalizable normalized rows.

### Expected outcome

A winning snapshot calculates all 21 staff roles and writes one `staff_role_metrics` row per current staff member even when every metric is null; historical snapshots retain raw staff only; deletion promotion rebuilds from retained `staff_attributes_json` before commit; wrong-model writes fail; failed compact staff writes roll back the snapshot; normalized staff writes remain only as the temporary dual-write seam; `./scripts/dev check-rust` and `./scripts/dev check` pass.

### Explicit exclusions

Staff reader cutover, player work, final normalized cleanup, progress, and staff formula changes.

## Discoveries and replanning

- Direct current source confirms that `snapshot/ingest.rs` writes normalized current player rows for every retained snapshot, while `potential_scores.rs` writes current-only normalized potential rows and projected JSON.
- Direct source confirms that Load Data clears Search/Squad result roots before Tauri and holds the database mutex across ingest. PR 2 must reverse the clear timing and split preparation without changing one-transaction publication.
- CodeGraph was unavailable/stale for this planning run. All current-state claims were verified against direct source and tests.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Compact active snapshot metrics | Commit 1 — Record the approved feature plan | 43ade8c1e63640158b7b42f2b1e3b8c2bd9d852f | Recorded the reviewed schema 2 ledger, active TODO pointer, narrowed backlog item, and ADR supersession on the authorized branch. | `ledger_state.py` runnable, `delivery_state.py` runnable with recorded fingerprint ffd74f8f3bc4d94484b753ac12a5d35874bc547e34fd9f1362ab6ba1d575dfaf, `git diff --check` clean on the eight planning paths, independent blank-slate plan review clear, checkpoint review clear. | Not applicable | Clear | 0 | None |
| PR 1 — Compact active snapshot metrics | Commit 2 — Create the fresh compact metric schema | f15ce7989b9e4354435099292fc100b6c4202831 | Fresh `app-v2.db` filename with never-touches-`app.db` proof, immutable checked-in compact migration v38 with the exact 68/68/21 column inventory plus row/model/check/foreign-key constraints and no per-role indexes, closed-catalog safe snake_case mapping owners, schema/model contract parity tests, README cleanup guidance, normalized tables retained as the temporary seam. | `./scripts/dev check-rust` passed (694 Rust tests, 0 failures, 2 intended-ignored), `./scripts/dev check` passed, `git diff --check -- README.md` clean, independent review clear, catalog parity cross-checked against the 68-player/21-staff catalogs. | Pass | Clear | 0 | None |
| PR 1 — Compact active snapshot metrics | Commit 3 — Materialize current player metrics atomically | fb09740e86ec29c027aba4e842e9cbb5431e2e1c | Compact one-row player writer (`persist_rows`/`clear_snapshot`/`clear_non_current_snapshots`/`assert_snapshot_complete`), winner/non-winner/promotion/boost lifecycle integrated into ingest, selection, deletion promotion, and per-player boost reconciliation with exact nulls and model versions, normalized dual-write seam retained and marked for Commit 8 removal, migration-34 hook kept seam-only. | `./scripts/dev check-rust` passed (702 Rust tests, 0 failures, 2 intended-ignored), `./scripts/dev check` passed, `git diff --check` clean, independent review clear (one NITPICK noted: two boost test asserts hardcode model version literal 1 instead of the constant; no functional finding). | Pass | Clear | 0 | None |
| PR 1 — Compact active snapshot metrics | Commit 4 — Cut Search and Squad over to compact player metrics | 192d0f1f8cc582981f8c7e338ef4a1cbbf9f7add | Search and Squad queries read role metrics directly from the one `player_role_metrics` row via the closed-catalog validated mapping: `player_metrics_join` one-to-one compact join with kind-version predicates, `assert_read_models_complete` scoped read validation, simplified filter clauses with single numeric params, shared generic ORDER BY branch preserving null/tie semantics, obsolete requested-role row-count helper removed. | `./scripts/dev check-rust` passed (698 Rust tests, 0 failures, 2 intended-ignored), `./scripts/dev check` passed, `git diff --check` clean, independent review clear, source search confirms no normalized-relation reference in production search/squad/resolver paths. | Pass | Clear | 0 | None |
| PR 1 — Compact active snapshot metrics | Commit 5 — Cut Profile and Planner over to compact player metrics | Pending record | Player Profile loads all 68 current/potential role scores from one compact row with preserved projected JSON validation; Planner depth, optimizer (both bases), and role reference select catalog-lane columns from compact rows keeping combine/fit/allocation in Rust; mutation preflight now uses `assert_read_models_complete` so corrupt/missing/wrong-version state blocks every mutation before writes; shared fixtures seed/corrupt compact columns; normalized writers remain as the temporary seam. | `./scripts/dev check-rust` passed (698 Rust tests, 0 failures, 2 intended-ignored), `./scripts/dev check` passed, `git diff --check` clean, independent review clear, source search confirms no normalized-relation reference in production player-query or planner reader paths. | Pass | Clear | 0 | None |

## Final validation

Automated, after all 12 planned commits are complete and obsolete normalized tests are removed:

- `./scripts/dev test` — all frontend behavior, IPC mocks, progress, context, and existing UI suites pass and discover the changed tests.
- `./scripts/dev check-rust` — formatting, Clippy, Rust persistence/query/lifecycle/progress tests, and schema tests pass.
- `./scripts/dev smoke` — the product smoke path passes with any Load Data stub changes.
- `./scripts/dev check` — the complete repository gate passes.
- Source search confirms no runtime or test persistence reference to `player_role_scores`, `player_potential_role_scores`, or `staff_role_scores`; historical planning/completed prose may retain their names.
- Do not run or depend on ignored 184k or 500k ingest tests. Do not open or query the live legacy `app.db`.
- After these commands and the manual proof pass, run `/skill:workflow-finish-feature`. It owns feature review, any bounded correction, documentation reconciliation, active-ledger archival, TODO close-out, and the close-out commit. It must verify that Commit 2's README cleanup text still matches the implemented filename and safety behavior, reconcile `.wiki/ARCHITECTURE.md` and `.wiki/DESIGN.md` to implemented state, retain the BACKLOG timeline as unimplemented, and rerun the ledger/delivery classifiers plus the repository documentation gate on the final reconciliation diff.

Manual Windows proof on a fresh representative approximately 250,000-player load:

1. Record machine/build context and actual scan, validation/preparation, scoring, database save/finalization, and total timings. Compare with evidence without claiming an unsupported speedup.
2. Confirm the previous snapshot and visible Search/Squad screens remain usable during scan/preparation and remain unchanged after a forced failure.
3. Confirm successful effective-current replacement changes all current screens atomically and historical non-winning storage does not replace current results.
4. Repeat loads while retaining history and confirm load cost/database growth do not worsen with historical derived matrices.
5. Exercise arbitrary current and potential role display, ascending/descending sort, and filter across several player roles, including nullable values; exercise staff role display/sort/filter and Planner/Profile behavior.
6. Delete the current snapshot and verify promotion rebuilds compact player/staff metrics atomically. Exercise both supported player boosts and verify one-player current/potential/projected replacement.
7. Inspect `app-v2.db` size. Confirm the old approximately 31 GiB `app.db` was never opened or changed. Verify the documented manual deletion instruction only after the new database is accepted.
8. Observe ordered command-scoped progress, indeterminate scan, truthful determinate phases, phase-specific button text, detailed final timings, failure preservation, and save-switch stale suppression.
9. Replan before adding a per-role index, pool, WAL, second driver, multiple writers, parallel dependency, detached job, or cancellation.

## Documentation impact

`README.md` is intrinsic to PR 1 Commit 2 because `app-v2.db` becomes user-visible there. That commit must document that `app.db` remains untouched, reinstall might preserve application data, and the user must verify `app-v2.db` before manually deleting `app.db`.

After PR 2 Commit 4 completes and final automated/manual validation passes, `/skill:workflow-finish-feature` owns documentation reconciliation and close-out rather than a planned content packet. It must verify and correct the README only if implementation or validation changed its truth; reconcile `.wiki/ARCHITECTURE.md` with the implemented compact schema, immutable migration/version contract, preparation/publication boundary, fresh filename, progress channel, and measured timing language; reconcile `.wiki/DESIGN.md` with implemented top-bar progress behavior; preserve the individual Profile timeline in `.wiki/BACKLOG.md` as unimplemented; remove the feature from `.wiki/TODO.md`; archive the complete ledger under `.wiki/features/completed/`; and reconcile ADR links/status only if implementation materially deviated. It must not claim automatic cleanup, historical timeline delivery, unmeasured performance, or release publication. Inspect the complete documentation diff, rerun both classifiers, run the repository documentation gate, and stop if cleanup safety, measurements, implementation truth, or archive classification cannot be verified.
