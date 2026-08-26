# Ingest-Time Potential Scoring

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** de707ef4e1eee17435261eb2dab614968e0f6d252365a2db172cb5653d307e67

Ledger state records delivery intent only. It grants no authority to create, switch, or mutate `feature/ingest-potential-scores` or any other branch.

## Intent

Move visible-attribute projection and potential role scoring from product reads to snapshot-derived persistence. Project each relevant current-snapshot player exactly once, derive every catalog potential role score from that one projected map, and make all product reads consume stored values.

## User-visible behavior

- Player Profile, Search, Squad, Planner assignments, Planner role reference, and potential optimization keep the same projected attributes, potential role scores, null behavior, age boundary, ordering, filtering, and score semantics.
- Load Data finishes only after the effective current snapshot has one persisted projected visible-attribute map and one persisted potential-role row, including a nullable score, for every player and every role in the current scoring catalog.
- A newly stored historical snapshot that does not become current retains no projected attributes or potential role scores.
- When a newer effective snapshot replaces the prior current snapshot, the prior snapshot loses its projected attributes and potential role rows in the same transaction that materializes the new current snapshot.
- Deleting the current snapshot materializes the promoted retained snapshot before the deletion transaction commits. Deleting the final snapshot leaves no potential-derived data for that save.
- Switching saves uses the selected save's already-materialized current snapshot. Save switching does not calculate potential data.
- A successful supported player boost atomically replaces that current player's projected attributes and complete potential-role row set together with the changed player fields and current role scores.
- A projection-model change requires an explicit migration or rebuild change before product reads can use the new model. Reads never replace stale or missing rows lazily.

## Invariants

- Rust and SQLite own projection, potential scoring, persistence, lifecycle, and reads. React does not calculate player growth or role scores.
- For each save, only its effective current snapshot may retain potential projected attributes or `player_potential_role_scores` rows.
- Every player in an effective current snapshot has one non-null serialized projected visible-attribute map at the current projection-model version.
- Every player in an effective current snapshot has exactly one `player_potential_role_scores` row for every role in `scoring::catalog::all_roles()`. A row persists even when its score is null.
- One writer call projects each player once and passes the same projected map to every potential-role calculation for that player.
- Potential role score formulas, the role catalog, CA-to-PA projection, natural-position rules, the age-29 identity boundary, rounding, caps, null preservation, and score combination semantics do not change.
- Current role scores remain separate `player_role_scores` rows calculated from current attributes.
- Snapshot ingest, effective-current replacement, deletion promotion, migration backfill, and supported boost reconciliation are atomic at their existing SQLite transaction boundaries.
- Product reads are read-only. Missing, stale-version, or incomplete current potential data is an invariant failure; no read path repairs it.
- Every Planner command that mutates teams, strings, or assignments and returns depth validates the already-resolved current snapshot before any product-owned mutation transaction or write. Corrupt potential state cannot produce an error response after the requested Planner mutation has committed.
- A projection-model version change owns an explicit migration or startup rebuild before the database becomes available. Incrementing a constant alone is invalid.

## Non-goals

- Frontend scoring or projection.
- Changes to role formulas, projection formulas, model calibration, or catalog membership.
- Historical potential browsing, analytics, trends, comparison, or retained historical derived data.
- Background jobs, cancellable materialization, progress UI, or lazy fallback.
- Persisting potential score basis in Planner state.
- General current-role query or sort optimization.
- Release preparation, packaging, publication, or version changes.

## Current-state map

- Relevant components: `src-tauri/src/features/scoring/projection.rs::project_attributes` owns the pure projection and `scoring/catalog.rs::all_roles` plus `scoring/score.rs::score_role` own catalog scoring. Production projection calls exist in `player/query.rs::get_player` and `load_role_scores`, `planner/depth.rs::resolve_assignment`, `planner/optimizer.rs::load_potential_optimizer_candidates`, `planner/role_reference.rs::get_role_reference`, and `player_metrics/potential_cache.rs::score_missing_roles`.
- Data model: completed Commit 2 added nullable `players.potential_attributes_json` and `potential_projection_model_version`. `player_role_scores` stores complete current ingest-time role rows. Migration v34 now backfills complete, versioned, nullable `player_potential_role_scores` rows keyed by `(snapshot_id, uid, role_id)` for each existing current snapshot; draft Commit 3 extends that eager ownership to later selection events.
- Persistence and migrations: `src-tauri/src/db/migrations.rs` is at schema version 34. Migration v21 created `player_potential_role_scores`; completed Commit 2 added the v34 projected columns and an in-transaction Rust backfill hook.
- Supported input shape: player dump `attributes` maps are sparse and open-keyed. `dump_validation::validate_int_or_null_map` accepts omitted and unknown keys when their values are JSON numbers or null. `snapshot::ingest::attributes_map` preserves every supplied key, requires each non-null number to be an integer representable as `u8`, and currently accepts zero. The closed visible catalog is the 47 exact PascalCase keys in `scoring::catalog::DUMP_ATTRIBUTE_KEYS`. The eager writer must validate supplied catalog values as `1..=20`, then normalize that catalog to an exact-key projected map with omitted catalog keys as null. Unknown source keys retain the existing typed source contract but do not enter projected output.
- Snapshot lifecycle: `snapshot/ingest.rs::ingest_dump_json_for_save` inserts players, staff, and current role scores before `snapshot/service.rs::select_current_snapshot` chooses the effective current row. `snapshot/service.rs::delete_snapshot` calls the same selector when deletion promotes a retained snapshot. `set_active_save` only changes save activation.
- Boost lifecycle: `player/service.rs::reconcile_verified_boost` updates CA or mentality in one transaction, `replace_role_scores` rewrites current role rows after mentality changes, Club DNA is recalculated eagerly, and potential rows are currently invalidated through `potential_cache::invalidate_player_cache`.
- Read consumers: Player Profile derives projected attributes and all role scores in `player/query.rs`; Planner depth, role reference, and potential optimizer derive them independently; Search and Squad already query `player_potential_role_scores` but call lazy materializers first.
- Query ownership: `player_metrics/resolver.rs` and `search/filter.rs` generate trusted potential-role SQL with the projection-model version. `search/query.rs` owns snapshot-wide filter/sort materialization and page materialization. `planner/squad.rs` owns managed-club completeness checks and page materialization.
- Tests: Rust module tests are colocated in `migrations.rs`, `snapshot/ingest.rs`, `snapshot/service.rs`, `player/query.rs`, `player/service.rs`, `search/query.rs`, `player_metrics/potential_cache.rs`, and Planner `*_tests.rs` files. Existing lazy-specific Search, Squad, completeness, stale-row replacement, and boost-invalidation assertions become obsolete.
- Project validation commands: `./scripts/dev check-rust` is the stable Rust gate and discovers the focused Rust tests; `./scripts/dev check` is the required commit gate. The stable script surface does not forward a Rust test-name filter.
- Primary risks: a partial current snapshot, double projection per player, stale historical rows, non-atomic promotion or boost repair, a schema upgrade that commits before its backfill, reads that silently repair missing data, changed null/version semantics, and roughly doubling ingest-time role-row writes for the effective current snapshot.

## Feature architecture

Add nullable `potential_attributes_json` and `potential_projection_model_version` columns to `players`. They remain null for non-current snapshots and form the direct snapshot/player-owned counterpart to `attributes_json`. Keep the normalized `player_potential_role_scores` table and its role/score index because Search and Squad already use those rows for bounded display, filter, and sort. Keep `projection_model_version` on every potential-role row.

Replace the lazy cache module with an eager derived-state writer under `src-tauri/src/features/player_metrics/potential_scores.rs`. The writer loads each target player once and parses current visible attributes and positions once. It first preserves the existing open-keyed source parsing contract: every supplied non-null entry, including an unknown key, must be an integer representable as `u8`. Before normalization or projection, it rejects every supplied non-null `DUMP_ATTRIBUTE_KEYS` value outside `1..=20`. It then builds a complete source map from exactly `DUMP_ATTRIBUTE_KEYS`, represents omitted supported visible attributes as null, ignores accepted unknown keys for projection and projected persistence, calls `project_attributes` once, serializes that exact-key projected map once, and scores the complete `all_roles()` catalog from the same map. It writes the player projection and all nullable role rows inside its caller's transaction. A current-snapshot rebuild first clears the target snapshot's old projection fields and rows, then replaces the complete set. A lifecycle helper also clears projection fields and potential rows from every non-current snapshot in the save.

Migration v34 adds the projected columns and converts the existing sparse table to eager current-snapshot ownership without changing its key, score domain, index, or cascade. Extend the migration registry with an explicit transaction hook for v34 so the schema change, deletion of all old lazy rows, backfill of every save's existing current snapshot, and `PRAGMA user_version = 34` commit together. The hook calls the shared eager writer. A crash or calculation failure leaves the database at v33. Future projection-model changes must add a new explicit migration or equivalent pre-open rebuild that invokes the same complete writer; product reads never own stale-row replacement.

`select_current_snapshot` remains the single effective-current lifecycle owner. In the transaction that changes the marker, it clears potential-derived data from snapshots that are no longer current and fully materializes the selected snapshot when the winner changes or requires the v34 backfill path. During ingest this happens after players and current role scores exist and before Academy current-snapshot effects and commit. A non-winning stored snapshot remains clear. During current deletion, the cascade removes the deleted rows and the selector materializes the promoted snapshot before commit. Save switching performs no materialization because every save's current snapshot is maintained independently.

Boost reconciliation calls a one-player eager replacement after CA or mentality/current-role updates and before Club DNA persistence and commit. It replaces the projected JSON, model version, and every potential role row for that player. Any failure follows the existing `SnapshotSync` recovery result and rolls back all SQLite changes in the boost transaction; it does not claim to reverse an FM memory change that already occurred.

Add one read-only invariant assertion to `player_metrics::potential_scores`. Given an already-resolved current snapshot ID, it runs one statement with a trusted, bound `VALUES` CTE built from `scoring::catalog::all_roles()`. The statement rejects a non-current snapshot; any current player whose projected JSON is null or invalid, or whose projected model version is not `PROJECTION_MODEL_VERSION`; and any `(player, catalog role)` pair without a row at that same expected version. The nested per-role `NOT EXISTS` check uses the table primary key and names every catalog role, so an extra or obsolete role row cannot substitute for a missing catalog role. The assertion returns an invariant error and performs no calculation or mutation.

Player Profile invokes the assertion after it establishes that the requested player exists in the active current snapshot and before it reads projected fields or potential rows. Planner depth invokes it once after resolving a current snapshot and before loading assignments; role reference invokes it before loading players; and optimizer invokes it before opening its assignment transaction because every optimizer response is followed by depth data that exposes potential scores. The Planner mutation services used by `save_planner_teams`, `add_planner_string`, `remove_planner_string`, `clear_planner_depth`, `clear_planner_assignment`, `assign_planner_player`, and `move_planner_player` also resolve the optional current snapshot and invoke the assertion, when a snapshot exists, before `ensure_depth`, a mutation transaction, or any other product-owned write. Existing pure request validation remains before database work, including Planner team input normalization, team enum parsing, and unconditional clear confirmation. Database-dependent validation keeps its current relative order after preflight. No-snapshot mutations skip the assertion and preserve their current behavior.

Extract only the post-resolution body of `depth::get_depth` into a small internal depth loader that accepts the resolved optional snapshot ID. Direct `get_depth` resolves the snapshot, asserts it when present, and calls that loader. Each mutation service returns its preflighted optional snapshot ID as an internal tuple value with its existing result; its command passes that ID to the loader after a successful mutation. `optimizer::optimize_depth_with_basis` already owns the resolved ID and does the same after commit. Do not expose this context through IPC or add a generic validation framework or public bypass: only the direct-read wrapper and already-preflighted Planner mutation/optimizer paths may call the loader.

Potential optimizer then loads persisted tactic-role rows and keeps its existing familiarity, foot, age, rank, matching, assignment, and rollback logic. Search and Squad retain their existing scalar, `EXISTS`, and `LEFT JOIN` display/filter/sort SQL, but invoke the same snapshot-wide assertion once before count, sort, page, or dynamic-field queries only when validated requested fields, filters, or sorts need potential data. Empty-snapshot, unknown-player, unconfigured-Squad, and non-potential Search/Squad paths keep their current results without validation. The final cleanup deletes lazy batching, stale replacement, invalidation, and cohort-completeness helpers and their obsolete tests, while retaining the read-only invariant assertion.

## Uncertainty register

### Known

- The developer approved current-snapshot-only potential persistence and the ingest, promotion, demotion, save-switch, boost, and model-change lifecycle in this ledger.
- `player_potential_role_scores` already has the required normalized identity, nullable score, model version, cascade, and Search/Squad index.
- `players` is snapshot-owned and already stores the current attribute map and every projection input.
- `project_attributes` returns one visible-attribute map and all potential role consumers use the same role catalog and scorer.
- Supported dump/player attribute maps can omit visible keys and can contain unknown keys. Dump validation accepts number-or-null values without a closed player-key check. Ingest preserves supplied entries and its typed map conversion rejects nonintegers and values outside `u8`, but accepts zero. `DUMP_ATTRIBUTE_KEYS` is the closed 47-key catalog for visible projection and persisted projected output. Missing and explicit null supported attributes are both unavailable to current queries.
- Snapshot selection and deletion promotion already share `select_current_snapshot` inside transactions.
- No planned feature spec exists. BACKLOG scope does not change.
- `.wiki/features/completed/player-table-sort-performance.md` has an unrelated developer-owned worktree modification and is excluded from this feature and every packet.

### Assumptions

- Two nullable columns on `players` are the smallest persisted representation because the projection is one player-owned map and no query needs a second row identity or independent projection lifecycle.
- Existing v21 potential rows can be discarded during v34 because ADR-0019 defines them as disposable and the approved migration requires complete recalculation for current snapshots.
- The current role/score index remains sufficient for eager rows because Search and Squad retain the same SQL access shapes.
- The synchronous cost is acceptable as an explicit Load Data and database-upgrade cost; no separate performance acceptance threshold was approved.

### Decisions

- Supersede ADR-0019 with ADR-0026. Convert the existing table from a lazy disposable cache to complete derived rows for only each save's current snapshot.
- Store projected visible attributes directly on the owning `players` row with the projection-model version. Do not add a second projected-attribute table or a score-vector JSON blob.
- Add migration v34 with an in-transaction Rust backfill hook. Do not commit a schema-only migration and repair it later at read time or after database open.
- Keep one projection-model version shared by the projected JSON and potential-role rows. Missing or wrong-version current data is an error, not a cache miss.
- Centralize complete and one-player writes in `player_metrics::potential_scores`; lifecycle callers own when those writes occur.
- Keep one PR. The migration, lifecycle, boost, and consumer conversions share one persisted invariant and no independent PR provides a useful, lower-risk product seam.

### Unknowns

- Exact representative v34 upgrade and Load Data duration on the largest user database is not available. Validation records functional and transactional evidence, not an unmeasured speed claim.
- Focused independent replan review accepted the changed packets. `delivery_state.py` computed and the ledger records the replacement fingerprint.

### Risks

- A custom migration hook can violate migration atomicity if it runs outside the migration transaction or advances `user_version` before backfill completes.
- A writer can accidentally call `project_attributes` once per role instead of once per player.
- A writer can pass a sparse source map through unchanged, producing an incomplete projected map that violates the persisted assertion even though the dump is supported.
- A writer can normalize and project before validating the supplied source domain. For a growth-eligible player with `CA < PA`, supplied `Acceleration: 0` can project to a valid value such as `1` and let invalid source data satisfy the persisted assertion.
- Ingest can materialize the inserted snapshot before selection and retain potential data on a non-winning historical row.
- Demotion or promotion can leave both snapshots materialized or expose a promoted snapshot before its rows are complete.
- Boost reconciliation can mix updated current values with stale projected rows or partially replace the role catalog.
- A read conversion can change null behavior, tactic scoring, tie breaks, filtering, or ordering while removing computation.
- A Planner mutation can commit team, string, or assignment changes and then fail while building its depth response unless the shared invariant assertion runs before the mutation boundary.
- Destructive team or string changes can delete assignments before post-response validation detects corrupt potential state.
- A scalar subquery, `EXISTS`, or `LEFT JOIN` can silently turn a missing or wrong-version row into null, exclusion, or changed order unless the shared invariant assertion runs first.
- A count-only completeness check can let an extra or obsolete role row substitute for a missing current catalog role.
- Deleting lazy helpers can remove shared version/query contracts or the read-only invariant assertion that eager reads still need.
- Complete potential role rows add synchronous writes and storage to Load Data for the effective current snapshot.

## Walking skeleton

Migration v34 upgrades a database with two saves and retained history. Each save's current snapshot receives one projected map and the complete nullable role catalog for every player, every non-current snapshot has no potential data, and a failed backfill leaves the schema at v33. The next lifecycle commit applies the same invariant to new selection and deletion promotion before any read consumer changes.

## Delivery plan

### PR 1 — Precompute current-snapshot potential scoring

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/ingest-potential-scores`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub required strict status `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `perf(scoring): precompute potential role scores`

**Purpose:** Replace every read-time projection path with one atomic current-snapshot derived-state lifecycle. One PR keeps the migration, lifecycle, boost consistency, and consumer cutover reviewable as one persisted contract.

**Depends on:** Synchronized `main` at planning HEAD `3c58707e`; implemented snapshot history, potential role scores, configurable player tables, player boosts, Planner best-role reference, and Club DNA eager persistence. No planned spec or earlier PR.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(scoring): record ingest potential score plan`

**Work:** Commit the independently reviewed ledger, TODO activation, superseded ADR-0019, accepted ADR-0026, and ADR index update on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, BACKLOG, current-state architecture, planned-spec removal, release work, and unrelated documentation.
- The developer-owned modification to `.wiki/features/completed/player-table-sort-performance.md`.

**Implementation packet:**

- Preserve the accepted independent plan-review outcome. Commit only the five reviewed planning paths after branch and base verification.

**Files and responsibilities:**

- `.wiki/features/active/ingest-potential-scores.md` — approved intent, architecture, delivery sequence, packets, risks, and validation contract.
- `.wiki/TODO.md` — replace `Active: None` with the active feature link.
- `.wiki/decisions/0019-lazy-potential-role-score-cache.md` — mark the former lazy lifecycle Superseded and link ADR-0026.
- `.wiki/decisions/0026-eager-current-potential-scoring.md` — record the approved eager current-snapshot persistence decision.
- `.wiki/decisions/README.md` — identify ADR-0019 as superseded and index ADR-0026.

**Behavior and data flow:**

- Record one active source of feature truth, one superseding persistence decision, and the exact one-PR commit sequence before code changes. BACKLOG and planned feature specs do not change.

**Ordered implementation steps:**

1. Verify `feature/ingest-potential-scores` is based on synchronized `main` without changing Git state or plan scope.
2. Confirm the reviewed diff contains only the five listed planning paths and excludes `.wiki/features/completed/player-table-sort-performance.md`.
3. Run the ledger classifier and repository planning checks required by the delivery workflow.
4. Stage and inspect only the five planning paths for independent checkpoint review.

**Tests and proof:**

- Not applicable — independently reviewed planning documents only. The ledger classifier proves schema consistency. No tests, fixtures, mocks, snapshots, helpers, or executable assets change.

**Patterns to verify:**

- `.wiki/features/active/README.md` schema 2, `.wiki/TODO.md` feature-level ownership, ADR format in `.wiki/decisions/README.md`, and ADR-0024's superseding eager-persistence rationale.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, BACKLOG, current-state architecture, scope, packet order, delivery metadata, or the unrelated dirty completed-feature file.

**Dependencies and sequencing:**

- Requires a clear independent plan-review verdict, developer acceptance, a recorded Delivery fingerprint, and exact branch activation through `/skill:workflow-deliver-feature`.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/ingest-potential-scores.md`; after review, `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/ingest-potential-scores.md /home/jonas/projects/fm-valuescout`; rerun both after recording the fingerprint.

**Stop conditions:** Stop on an uncleared review, classifier error, pending fingerprint, unreviewed path, substantive post-review change, wrong branch/base, or inclusion of `.wiki/features/completed/player-table-sort-performance.md`.

**Review mandate:** Verify the five-path planning diff matches approved intent, uses one PR, marks only this commit Active, grants no branch authority, supersedes ADR-0019 correctly, leaves BACKLOG and the dirty completed-feature file untouched, and contains execution-ready packets.

#### Commit 2 — Add eager current-potential persistence

**Status:** Completed

**Provisional commit:** `feat(scoring): persist current potential projections`

**Work:** Add the projected-attribute columns, a shared eager writer, and atomic v34 upgrade/backfill for every save's existing current snapshot while preserving the existing potential-role table identity and query index.

**Size assessment:** Estimated 150–200 changed non-test implementation lines. Within the soft target; migration characterization may be larger but tests do not count.

**Out of scope:**

- New-ingest selection lifecycle, deletion promotion, save switching, boost reconciliation, read-consumer conversion, lazy-helper deletion, formula changes, and frontend work.

**Implementation packet:**

- Establish the persisted invariant without changing product reads. The migration must be restart-safe and atomic: schema, old-cache clearing, all-save current backfill, and `user_version` advance commit together.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs::{Migration, apply, all}` — add v34 projected columns and the explicit transaction hook; characterize fresh and v33 upgrades, rollback, current-only scope, complete nullable rows, and model version.
- `src-tauri/src/features/player_metrics/mod.rs` — expose the eager potential-score owner while the old cache module remains temporarily for unchanged consumers.
- `src-tauri/src/features/player_metrics/potential_scores.rs` — own `PROJECTION_MODEL_VERSION`, complete snapshot rebuild, one-player replacement, non-current clearing primitives, the one-projection-per-player writer, and the shared read-only current-snapshot invariant assertion.
- `src-tauri/src/features/player_metrics/potential_cache.rs` and `resolver.rs` — temporarily consume the shared model constant without changing lazy behavior; retain all lazy tests until later packets retire their contracts.

**Behavior and data flow:**

- v34 adds nullable `players.potential_attributes_json` and `players.potential_projection_model_version`. Its hook deletes disposable v21 rows, clears projected fields, finds every `snapshots.is_current = 1` row across saves, loads each player once, projects once, serializes the map, and writes all catalog role rows from that map. It advances `user_version` only after the transaction succeeds. Historical rows remain null and scoreless.

**Ordered implementation steps:**

1. Add focused v33 upgrade tests whose RED state lacks projected columns, retains sparse/stale rows, and cannot satisfy complete current-only backfill.
2. Introduce the shared eager writer with exact current catalog iteration and nullable-row persistence.
3. Add the read-only invariant assertion. Build one trusted, bound `VALUES` CTE from `all_roles()` and reject a non-current snapshot, a null, invalid, or wrong-version projected map, or any exact catalog role missing at the expected version. Use nested per-role `NOT EXISTS`, not a total-row count that extra roles can satisfy.
4. Extend migrations with the smallest custom hook seam and v34 schema change.
5. Run v34 SQL and the eager backfill inside the same migration transaction before updating `user_version`.
6. Add failure injection or constraint-trigger proof that rolls back schema version and derived data.
7. Keep existing reads and lazy materializers compiling against the shared version constant.
8. Refactor only after the migration, writer, and assertion proofs stay green.

**Tests and proof:**

- Add/modify migration tests in `migrations.rs`: upgrading a populated v33 database with two saves, one current and one retained non-current snapshot each, removes stale/sparse rows, fills one projected map and `all_roles().len()` nullable rows per current player, leaves historical snapshots empty, and records the exact model version.
- Add a rollback test where one current player's invalid projection source or an injected write constraint makes v34 fail; assert `user_version` remains 33 and no partial v34 state is committed.
- Add writer tests in `potential_scores.rs` with a projection-call test seam or equivalent observable source fixture that proves all role scores match one serialized projected map and nullable scores still get rows.
- Add assertion tests in `potential_scores.rs`: a complete current snapshot passes; deleting one catalog row fails even when an extra non-catalog row keeps the total count unchanged; changing one role row to the wrong version fails; and a null or wrong-version projected map fails. Install write-denying triggers after corrupting each fixture, assert the helper returns an invariant error, and compare projected fields plus role rows before and after the call to prove no mutation.
- Retain projection formula unit tests, v21 schema/cascade tests, Search/Squad lazy tests, Planner tests, and frontend tests unchanged because their supported contracts have not changed in this commit.

**Patterns to verify:**

- `migrations.rs::apply` transaction and monotonic registry tests; `snapshot/ingest.rs::insert_role_scores` complete nullable catalog writes; `club_dna::persist_snapshot_scores` as the closest eager writer analogue. Deliberately diverge from ADR-0024 by backfilling only effective current snapshots.

**Constraints and non-goals:**

- Reuse the v21 table, primary key, score constraint, cascade, and index. Do not add a projected-attribute table, score-vector JSON, background repair, read-time fallback, dependency, formula change, or historical backfill.
- A future model constant change without an explicit migration/rebuild must fail review.

**Dependencies and sequencing:**

- Requires Commit 1. Later lifecycle and read packets depend on the writer and v34 schema. Existing reads remain safe because migration backfills their current snapshot and old lazy code can see complete exact-version rows.

**Validation:** First run `./scripts/dev check-rust` and confirm the new focused migration/writer tests fail for the intended absent v34 contract; after GREEN rerun `./scripts/dev check-rust`; then run `./scripts/dev check` as the commit gate.

**Stop conditions:** Stop and replan if migration hooks cannot share the existing transaction, if v34 must commit before backfill, if `project_attributes` cannot be reused without changing semantics, if current snapshots cannot be identified for every save, or if the table key/index cannot represent complete nullable catalog rows.

**Review mandate:** Verify migration atomicity and restart behavior; exact current-only scope across multiple saves; one projection per player; complete nullable catalog rows; projected JSON/version pairing; the assertion's trusted bound catalog CTE and per-role anti-join; material missing/stale/map no-write proof; deletion of stale lazy rows; and no hidden formula or schema-vector change.

#### Commit 3 — Maintain potential data across snapshot selection

**Status:** Completed

**Provisional commit:** `feat(snapshot): materialize selected potential scores`

**Work:** Make ingest selection, demotion, deletion promotion, and save switching preserve the current-only materialization invariant; validate supplied visible source values before sparse normalization and projection; and adapt tests whose setup now begins with eager rows.

**Size assessment:** Estimated 120–200 changed non-test implementation lines. Within the soft target. The eager source-domain check is a small addition to the same writer boundary. The broader test diff does not count toward the soft target and remains atomic because eager lifecycle activation itself invalidates the old zero-row fixture baseline; Commit 3 must leave the branch green without weakening the still-supported lazy paths.

**Out of scope:**

- Boost reconciliation behavior, product-read conversion, deletion of lazy Search/Squad paths, formula/catalog changes, and UI progress.

**Implementation packet:**

- Keep `select_current_snapshot` as the single effective-current owner. Materialize only after the winner is known, clear every loser in the save, and commit marker and derived state together.
- Before normalization or the writer's single projection call, parse the open source map with the existing integer/`u8` rules and validate every supplied non-null `DUMP_ATTRIBUTE_KEYS` value as `1..=20`. Then normalize exactly `DUMP_ATTRIBUTE_KEYS`, using null for an omitted supported visible attribute. Continue to accept typed unknown keys under the existing source contract, but exclude them from the exact-key projected map. Preserve malformed JSON, noninteger numbers, values outside `u8`, and supplied supported values outside `1..=20` as failures. Retain `rebuild_snapshot` and `replace_player` post-write calls to `assert_current_snapshot_complete`; the new pre-projection check supplements rather than replaces the exact-key persisted postcondition.
- Make the smallest trunk-safe fixture adaptations for the fourteen known failures in `/tmp/commit3-check.log`. Preserve lazy materializer behavior until Commits 9–10 by clearing eager rows only inside tests whose current purpose is lazy fill/replacement. Use `UPDATE` or `DELETE` plus `INSERT` when a test needs custom score/version rows that eager ingest already created. Do not convert boost behavior or read consumers early.

**Files and responsibilities:**

- `src-tauri/src/features/snapshot/service.rs::{select_current_snapshot, delete_snapshot, set_active_save}` — coordinate current marker, losing-snapshot clear, promotion materialization, and no-write save switching.
- `src-tauri/src/features/snapshot/ingest.rs::ingest_dump_json_for_save` — invoke the selection-owned lifecycle only after players and current role scores exist; extend ingest fixtures and generated-count assertions.
- `src-tauri/src/features/player_metrics/potential_scores.rs::{persist_players, reconcile_current_selection}` — parse the open source map, validate every supplied non-null catalog value as `1..=20` before normalization or projection, normalize sparse visible attributes into the exact supported key set, ignore accepted unknown keys in projected output, reconcile current selection, and correct the normalization proof to expect projected rather than source values.
- `src-tauri/src/db/migrations.rs` tests — replace the committed `{}` rollback characterization with a successful sparse-v33 backfill proof; retain malformed JSON and invalid typed/domain rollback coverage.
- `src-tauri/src/features/search/query.rs` tests — adapt eager-row counts, explicit lazy-cache baselines, and custom exact-version score setup without changing Search production behavior.
- `src-tauri/src/features/planner/squad_tests.rs` — make the same bounded eager-row and custom-score fixture adaptations for Squad while retaining its lazy materializer contract until Commit 10.
- `src-tauri/src/features/player/service.rs` tests — replace duplicate potential-row inserts with updates or delete-and-insert setup so the existing invalidation and rollback contracts remain protected until Commit 4.

**Behavior and data flow:**

- Ingest inserts a stored snapshot and current roles, then selection determines the winner. If the new row wins, the transaction clears the former current's projected fields and rows and materializes the new winner. The writer parses all supplied source entries under the existing integer/`u8` contract, rejects any supplied non-null catalog value outside `1..=20` before normalization or projection, expands the winner's sparse visible map to exactly the closed supported key set, maps each omission to null, excludes accepted unknown keys, projects that complete map once, and persists assertion-compatible projected JSON plus the complete role catalog. If the new row loses, it remains clear and the existing current remains unchanged. Current deletion cascades the deleted data, selects the retained winner, and materializes it before Academy promotion work and commit. `set_active_save` changes only `saves.is_active` and relies on the selected save's maintained current rows.
- Eager ingest means Search, Squad, and boost fixtures now begin with complete derived rows. Commit 3 changes only test setup and assertions needed to express their still-supported pre-cutover contracts: lazy-specific tests explicitly remove the eager rows they intend to rematerialize; custom score/version cases replace existing rows instead of inserting duplicate primary keys; ordinary eager baselines compare the correct complete row inventory.

**Ordered implementation steps:**

1. Preserve the draft RED/GREEN lifecycle tests for later-then-earlier, earlier-then-later, current deletion promotion, demotion clearing, final deletion, save switching without writes, and materialization rollback.
2. Route winner reconciliation through `select_current_snapshot` without projecting the inserted snapshot in advance; clear non-current state and rebuild only a changed winner inside the same transaction.
3. In `persist_players`, parse the stored source object into the same open `HashMap<String, Option<u8>>` shape used by current ingest scoring. Before building a normalized map or calling `project_attributes`, inspect every supplied `DUMP_ATTRIBUTE_KEYS` entry and reject a non-null value outside `1..=20`. Keep an unknown key when parsing so malformed, noninteger, and out-of-`u8` unknown values still fail as they do today; accept a null or `u8` unknown value and omit it from projected persistence.
4. Construct the complete source map from exactly `DUMP_ATTRIBUTE_KEYS` only after source validation passes. Map absent and explicit-null supported attributes to `None`, call `project_attributes` once, and require the serialized projected object to contain exactly the 47 catalog keys. Keep the existing `assert_current_snapshot_complete` call after every successful rebuild or one-player replacement.
5. Add a RED/GREEN writer test with a growth-eligible player (`CA < PA`, age below 29) and supplied `Acceleration: 0`. RED must show the current normalize/project-first draft can persist projected `Acceleration: 1`; GREEN must return the source-domain error before the projection call and leave projected fields and potential-role rows unchanged. Add a typed unknown-key case that remains accepted but absent from the exact-key projected object.
6. Replace `rolls_back_v34_when_current_player_projects_to_an_incomplete_map` with `migrates_v33_sparse_current_player_attributes_as_nulls`: a v33 `{}` or sparse-map proof that reaches v34, stores every projected key as null when unavailable, and writes the complete nullable role catalog. Retain `rolls_back_v34_when_current_player_projection_input_is_malformed`; add focused noninteger, out-of-`u8`, and supported-domain supplied-value cases through the same rollback helper, and keep their unchanged v33 evidence. The supported-domain case must use growth-eligible `Acceleration: 0` and prove `user_version` stays 33 before projection can mask it.
7. Extend `failed_winning_potential_materialization_rolls_back_ingest` or add its narrow analogue with a later winning dump whose growth-eligible player supplies `Acceleration: 0`. Prove the ingest fails, the inserted snapshot and its current/potential rows do not survive, and the prior current snapshot plus its derived state remain unchanged.
8. Correct `rebuild_normalizes_omitted_attributes_to_null` to expect the actual projected `Acceleration` value of 11 from the fixture rather than the source value 10, while proving an omitted key such as `Pace` persists as null and the map has exactly `DUMP_ATTRIBUTE_KEYS.len()` entries.
9. Adapt the failing Search and Squad lazy tests. Explicitly delete eager rows before a call only when the test still protects page/full-cohort lazy materialization or stale replacement; otherwise compare the eager complete baseline. Replace duplicate exact-version inserts with `UPDATE` or `DELETE` plus `INSERT` for custom nullable/tie scores.
10. Adapt the three failing boost tests by updating or replacing the eager row used as their custom seed. Preserve the current successful invalidation result and Club DNA/rollback assertions; Commit 4, not this packet, changes boost behavior to eager replacement.
11. Preserve Academy ordering, current roles, Club DNA, selector results, request validation, Search/Squad query semantics, and all current rollback behavior.
12. Run the full Rust gate, inspect any failure beyond the recorded fourteen as a possible scope contradiction, then run the complete commit gate.

**Tests and proof:**

- In `snapshot/ingest.rs`, retain the new winner/loser assertions for projected JSON and players × catalog rows, including earlier and undated losing ingests. Extend `failed_winning_potential_materialization_rolls_back_ingest` or add a narrow test with growth-eligible `CA < PA` and supplied `Acceleration: 0`; the source-domain error must roll back the winning snapshot before projected `1` can mask it, while the prior current snapshot and all of its source and derived rows remain byte-for-byte unchanged.
- In `snapshot/service.rs`, retain promotion materialization, non-current no-rewrite under write-denying triggers, final deletion, `promoted_snapshot_materialization_failure_rolls_back_current_deletion`, and `switching_between_materialized_saves_performs_no_potential_writes`.
- In `potential_scores.rs`, fix `rebuild_normalizes_omitted_attributes_to_null` so it detects both regressions: passing a sparse map through unchanged and confusing source values with CA-to-PA projected values. Add a direct growth-eligible `Acceleration: 0` RED/GREEN proof that rejects before projection and leaves derived state unchanged. Add an open-key proof that a supplied unknown key with a null or integer `u8` value remains accepted source input but is absent from the persisted exact 47-key projection; malformed, noninteger, and out-of-`u8` unknown values remain failures. Retain the successful rebuild and replacement proofs that run `assert_current_snapshot_complete` after persistence, including exact projected keys and complete nullable catalog rows.
- In `migrations.rs`, replace only `rolls_back_v34_when_current_player_projects_to_an_incomplete_map` with `migrates_v33_sparse_current_player_attributes_as_nulls`; `{}` is a supported sparse map and must migrate successfully. Retain `rolls_back_v34_when_current_player_projection_input_is_malformed`. Add focused rollback tests for a fractional or otherwise noninteger number, a number outside `u8`, and a supplied supported value outside `1..=20`. The domain test must use growth-eligible `Acceleration: 0` so the old draft would project it to `1`; GREEN must reject the source first. Every failure keeps `user_version = 33`, leaves the v34 columns absent, preserves source JSON, and retains the old sparse row.
- In `search/query.rs`, adapt `current_role_sort_materializes_requested_potential_page_fields`, `invalid_filter_rules_do_not_materialize_potential_cache_rows`, `potential_display_materializes_only_requested_page_players`, `potential_role_filters_materialize_each_requested_role_and_replace_stale_rows`, `potential_sort_materializes_the_full_search_cohort_before_ordering`, and `potential_sort_orders_nullable_ties_and_materializes_only_the_distinct_visible_page_role`. Keep each current lazy contract observable with an explicit empty/stale setup where required; do not assert that normal ingest starts empty.
- In `planner/squad_tests.rs`, adapt `current_role_sort_materializes_requested_potential_page_fields`, `potential_display_is_page_scoped_and_potential_sort_materializes_the_squad_cohort`, and `potential_sort_orders_nullable_ties_and_materializes_only_the_distinct_visible_page_role` by the same rule.
- In `player/service.rs`, adapt `successful_player_boost_recomputes_its_club_dna_score_and_invalidates_potential_roles`, `current_ability_reconciliation_rolls_back_exact_derived_rows_when_club_dna_upsert_fails`, and `determination_reconciliation_rolls_back_all_rows_when_club_dna_upsert_fails` so custom seeds replace eager rows. Preserve invalidation, exact rollback snapshots, and Club DNA behavior until Commit 4.
- No production fixtures, frontend mocks, snapshots, or helpers are deleted. Add a narrow test-only row-clear or row-replacement helper only if repeated setup would otherwise obscure the contract.

**Patterns to verify:**

- `dump_validation::validate_int_or_null_map` for the open number-or-null player map; `snapshot::ingest::attributes_map` for integer/`u8` parsing of every supplied key; `DUMP_ATTRIBUTE_KEYS` and `bridge/Layouts/Fm263Layout.cs::AttributeEntries` for the exact 47-key visible catalog; `project_attributes` for missing/null equivalence, growth semantics, and the masking path; shared `SNAPSHOT_ORDER_BY`; the current marker transaction; `delete_snapshot` Academy promotion; ADR-0024 ingest atomicity; Search/Squad lazy materializer tests; and boost invalidation/rollback tests.

**Constraints and non-goals:**

- Do not require dump producers to emit every visible key or reject a typed unknown key that the open player-attribute source contract currently accepts. Do not persist unknown keys in the exact projected map, change missing/null query semantics, bypass validation of supplied catalog values, project more than once per player, or weaken the complete persisted-map assertion.
- Do not materialize every retained snapshot, select by load order, calculate during save switch, alter Academy or Club DNA behavior, change bridge provenance, convert boost behavior before Commit 4, remove lazy Search/Squad behavior before Commits 9–10, or add recovery/background work.
- Test adaptation must preserve a supported contract or isolate a still-supported lazy seam. Do not delete, skip, or weaken a test merely because eager rows changed its initial database state.

**Dependencies and sequencing:**

- Requires completed Commit 2's v34 schema and eager writer. Commit 3 remains atomic and must restore a green trunk-safe state before Commit 4 because it activates eager rows for every normal ingest fixture. Commit 4 ordering remains safe: current boost invalidation stays intact after the seed adaptations, then Commit 4 replaces it atomically. Commits 9 and 10 later retire the explicitly isolated lazy Search/Squad behavior rather than repeating this compatibility repair.

**Validation:** Run `./scripts/dev check-rust` and require all Rust tests, including the fourteen formerly failing cases in `/tmp/commit3-check.log`, to pass; then run `./scripts/dev check` as the commit gate. Inspect `git diff --check` for the exact Commit 3 worktree diff before checkpoint review.

**Stop conditions:** Stop and replan if source-domain validation cannot run before normalization and projection, if the writer must close the currently open player-attribute source key set, if sparse normalization changes supplied valid values or formulas, if a supplied malformed/noninteger/out-of-`u8` value or supported non-null value outside `1..=20` becomes accepted, if projected output contains anything other than the exact 47 `DUMP_ATTRIBUTE_KEYS`, if selection cannot keep marker and derived writes in one transaction, if promotion must commit before scoring, if save switching exposes an unmaterialized current snapshot, if lifecycle changes alter the accepted date selector or Academy semantics, if a Search/Squad test cannot remain meaningful without early consumer conversion, or if a boost test requires eager recomputation before Commit 4.

**Review mandate:** Verify the writer parses the open source map and validates every supplied non-null catalog value as integer `1..=20` before normalization or projection; a growth-eligible supplied `Acceleration: 0` cannot become persisted `1`; omitted catalog keys become null; typed unknown keys retain the supported source contract but cannot enter the exact 47-key projection; `{}` migration succeeds while malformed, noninteger, out-of-`u8`, and supported-domain failures roll back; winning-ingest failure preserves the prior current state; successful rebuild and replacement still run the exact-key post-write assertion; projected-value expectations are correct; winner, loser, promotion, and save-switch lifecycles remain atomic; only current rows survive; compatibility adaptations do not weaken assertions; and date ordering, Academy, Club DNA, deferred boost/lazy ownership, and full gates remain unchanged.

#### Commit 4 — Recompute boosted player potential atomically

**Status:** Completed

**Provisional commit:** `feat(player): refresh boosted potential scores`

**Work:** Replace boost-time potential-cache invalidation with atomic one-player projection and complete role-score replacement.

**Size assessment:** Estimated 70–130 changed non-test implementation lines. Within the soft target.

**Out of scope:**

- Bridge protocol or formulas, batch boost orchestration, snapshot-wide rebuilds, read conversions, and frontend invalidation changes.

**Implementation packet:**

- After verified CA or mentality changes and current-role updates, call the eager one-player writer inside the existing reconciliation transaction. Preserve the existing partial external-FM outcome and Load Data recovery error.

**Files and responsibilities:**

- `src-tauri/src/features/player/service.rs::{reconcile_verified_boost, reconcile_mentality, replace_role_scores, database_sync_error}` — order source updates, current scores, potential replacement, Club DNA, and commit; remove lazy invalidation.
- `src-tauri/src/features/player_metrics/potential_scores.rs` — replace one current player's projected JSON and every catalog role row from one projection.
- Existing tests in `player/service.rs` — starting from Commit 3's eager-row-safe seeds, replace the still-supported invalidation outcome with exact recomputation and rollback assertions for both supported boost operations. Do not repeat Commit 3's duplicate-insert fixture repair.

**Behavior and data flow:**

- The bridge result is verified first. The SQLite transaction updates CA or mentality attributes, rewrites current role scores when required, reloads transaction-visible projection inputs, projects once, replaces projected JSON/version and all potential rows, updates Club DNA when defined, and commits. Any local failure rolls back all SQLite changes and maps to the established `SnapshotSync` Load Data recovery.

**Ordered implementation steps:**

1. Turn Commit 3's green, eager-row-safe invalidation characterization into RED assertions for changed projected JSON and complete changed role rows; the RED reason must be delete-only invalidation, not a duplicate seed failure.
2. Add a mentality-path RED proof where Determination affects current and potential scores.
3. Replace `potential_cache::invalidate_player_cache` with the one-player eager writer after source/current-score updates.
4. Prove catalog replacement removes stale/extra rows and inserts nullable rows.
5. Inject potential-write failures for CA and mentality and assert source fields, current roles, potential projection, potential roles, and Club DNA all retain pre-transaction values.
6. Preserve bridge verification and recovery messages.

**Tests and proof:**

- Modify Commit 3's seed-adapted `successful_player_boost_recomputes_its_club_dna_score_and_invalidates_potential_roles` into an eager recomputation test and add exact row-count/model assertions. This packet changes the behavioral expectation; it does not redo the eager baseline setup.
- Extend the already seed-adapted current-ability and Determination rollback/reconciliation tests to compare stored projection and role scores against post-update source values.
- Extend rollback tests with write-denying triggers on projected fields or potential rows. A plausible wrong implementation that deletes rows, uses pre-update values, omits nullable roles, or commits partial data must fail.
- Retain bridge error, stale context, request-ID, current-role, Club DNA, and squad-wide orchestration tests. No frontend mocks or snapshots change.

**Patterns to verify:**

- Existing `replace_role_scores` complete replacement; Club DNA eager boost reconciliation; `database_sync_error` truthful external-FM recovery; player boost context/token checks.

**Constraints and non-goals:**

- Do not claim the SQLite rollback reverses FM memory, add a bridge action, change eligibility or increments, or leave delete-only invalidation.

**Dependencies and sequencing:**

- Requires Commits 2–3 and Commit 3's green eager-row fixture baseline. Later read packets rely on boosts keeping stored potential values current.

**Validation:** Run `./scripts/dev check-rust` for focused boost RED/GREEN and rollback proofs, then `./scripts/dev check` as the commit gate.

**Stop conditions:** Stop and replan if projection inputs cannot be read transaction-visibly after the boost, if one-player replacement cannot share the reconciliation transaction, or if failures cannot preserve the existing `SnapshotSync` recovery contract.

**Review mandate:** Verify post-update inputs; one projection per player; complete role replacement; CA and mentality paths; rollback across source/current/potential/Club DNA rows; stale-context rejection; and truthful external-FM recovery.

#### Commit 5 — Read persisted profile potential values

**Status:** Completed

**Provisional commit:** `refactor(player): read stored potential values`

**Work:** Convert Player Profile to stored projected attributes and exact-version potential role rows without changing its DTO or visible semantics.

**Size assessment:** Estimated 50–100 changed non-test implementation lines. Within the soft target.

**Out of scope:**

- Planner, Search, Squad, scoring formulas, frontend profile changes, and lazy-module deletion.

**Implementation packet:**

- Remove `project_attributes` and `score_role` from the production profile query. Parse the stored projected JSON and map persisted potential rows through the existing Moneyball role catalog mapping.

**Files and responsibilities:**

- `src-tauri/src/features/player/query.rs::{get_player, map_player_row, load_role_scores}` — preserve the current unknown-player lookup, then invoke the shared snapshot assertion before selecting exact-version projected JSON or potential rows; load current and potential rows while preserving catalog mapping and nullable DTO fields.
- `src-tauri/src/features/player_metrics/potential_scores.rs::assert_current_snapshot_complete` — provide the shared model-version and read-only invariant seam; do not add a profile-specific validator.
- Profile query tests in `player/query.rs` — prove persisted values win and reads do not repair missing/stale data.

**Behavior and data flow:**

- `get_player` resolves the active current snapshot and preserves `None` for an unknown UID without requiring potential data. For a known player, it calls `assert_current_snapshot_complete` before reading projected fields or potential rows, maps the stored projected JSON to `potential_attributes`, and loads exact-version potential score rows in `load_role_scores`. Duplicate Moneyball definitions still map from the same attribute-role row. No SQL write occurs.

**Ordered implementation steps:**

1. Add a RED test that makes valid current source attributes differ from stored projection/score rows and expects the stored values.
2. Add write-denying triggers and missing/wrong-version fixtures to prove the read returns the shared invariant error and never rebuilds.
3. Preserve the base lookup that distinguishes an unknown UID, then invoke `assert_current_snapshot_complete` once for a known current player before selecting projected fields or potential rows.
4. Select and parse the persisted projected JSON after the assertion.
5. Replace in-memory potential scoring with exact-version row loading.
6. Preserve catalog order, duplicate mapping, null behavior, concealment payload, and age-29 output.
7. Remove production projection/scoring imports only after focused proofs pass.

**Tests and proof:**

- Rewrite `returns_role_potential_from_projected_visible_attributes` to assert ingest-persisted projection and scores.
- Retain and adapt age-29 identity, duplicate Moneyball mapping, unknown UID/save, null attribute, visibility, and current-role tests.
- Add a persisted-versus-source divergence test that fails if `project_attributes` or `score_role` returns to the read path.
- Add separate known-player tests that delete one role row, change one role row to the wrong version, null the projected map, and change the projected-map version. Each call must return the shared invariant error under write-denying triggers and leave projected fields and all role rows byte-for-byte unchanged. Retain the unknown-UID `None` contract. No frontend tests change because the DTO is unchanged.

**Patterns to verify:**

- Existing `load_role_scores` Moneyball catalog mapping; current-role persisted query; player detail concealment ownership; resolver exact model-version query.

**Constraints and non-goals:**

- Do not change the DTO, role order, labels, score semantics, concealment, or render behavior. Do not fall back to source projection.

**Dependencies and sequencing:**

- Requires Commits 2–4 so ingest, migration, promotion, and boosts maintain the stored values.

**Validation:** Run `./scripts/dev check-rust` for focused profile RED/GREEN proofs, then `./scripts/dev check` as the commit gate.

**Stop conditions:** Stop and replan if the profile DTO requires a formula change, if duplicate catalog mapping cannot use persisted rows, or if supported current snapshots can legitimately lack exact-version derived state.

**Review mandate:** Verify the assertion runs once for a known current player before projected reads; unknown-UID behavior; missing/wrong-version role and projected-map failures with zero mutation; no product-read projection; exact-version identity; DTO compatibility; duplicate role mapping; null and age-29 semantics; concealment; and a regression test that detects source recalculation.

#### Commit 6 — Read persisted Planner assignment potential

**Status:** Completed

**Provisional commit:** `refactor(planner): read stored assignment potential`

**Work:** Convert resolved Planner assignment cells to combine the two stored potential role rows for their lane, and preflight every Planner mutation that returns depth before it changes teams, strings, or assignments.

**Size assessment:** Estimated 90–160 changed non-test implementation lines. Within the soft target; the added mutation preflights and the small depth-loader extraction remain one atomic truthful-response boundary with assignment read conversion.

**Out of scope:**

- Role reference, optimizer, slot candidates, Search, Squad, formulas, and tactic changes.

**Implementation packet:**

- Replace projection inputs and per-read scoring in `resolve_assignment` with two exact-version potential-role joins while retaining current joins and `combine_role_scores`.
- Make the shared assertion a precondition of every Planner team, string, and assignment mutation whose command returns depth. A corrupt current snapshot must fail before `ensure_depth`, a mutation transaction, or any product-owned write, not after a committed mutation while formatting the response.

**Files and responsibilities:**

- `src-tauri/src/features/planner/depth.rs::{get_depth, add_string, remove_string, clear_all, clear_assignment, assign_player, move_player, resolve_assignment}` — resolve the optional current snapshot and assert it before `ensure_depth`, any mutation transaction, or any write; extract the current post-resolution depth assembly into one internal loader for direct reads and already-preflighted mutation responses; join lane IP/OOP current and potential rows; remove current attribute/CA/PA/age parsing from this read; and preserve assignment state.
- `src-tauri/src/features/planner/teams.rs::{save_team_settings, normalize_inputs}` — preserve pure input normalization before database work, then resolve and assert the optional current snapshot before `unchecked_transaction`, `ensure_team_rows`, destructive assignment/string deletion, or team writes.
- `src-tauri/src/features/planner/commands.rs::{save_planner_teams, add_planner_string, remove_planner_string, clear_planner_depth, clear_planner_assignment, assign_planner_player, move_planner_player}` — keep command-level parsing before the database lock, receive the preflighted optional snapshot ID from the mutation service without exposing it through IPC, and pass it to the internal depth loader rather than the asserting direct-read wrapper.
- `src-tauri/src/features/planner/optimizer.rs::optimize_depth_with_basis` — retain its pre-transaction snapshot assertion and return depth through the same already-preflighted loader after commit.
- `src-tauri/src/features/planner/depth_tests.rs` and `teams_tests.rs` — adapt assignment scoring fixtures, add persisted-versus-source proof, and add mutation preflight, rollback-state, no-derived-write, and validation-order coverage.
- Planner test support only where needed to refresh eager rows after tests deliberately mutate player projection inputs or to snapshot Planner and potential-derived tables without duplicating production scoring.

**Behavior and data flow:**

- `get_depth` preserves its no-snapshot result and setup behavior, but when a current snapshot exists it validates the complete snapshot before `ensure_depth` can create missing Planner rows or `load_assignments` can read any selected potential row. Each assignment query then resolves the player and managed-club state, reads current lane scores and exact-version potential lane scores, and combines each pair with the saved IP weight. Unresolved and outside-pool behavior remains unchanged.
- Each depth-returning mutation preserves current request-validation and no-snapshot semantics. After its existing pure untrusted-input checks, the service resolves the optional current snapshot and, when present, calls `assert_current_snapshot_complete` before Planner setup or mutation begins. Only after that preflight may save teams replace configuration or delete removed assignments/strings, string commands insert/delete/reorder strings, clear commands delete assignments, and assign/move commands insert, delete, or replace assignments. The service returns the preflighted optional snapshot ID with its existing internal result, and the command passes it to the depth loader after success. The assertion and response load perform no projection, repair, or derived write.

**Ordered implementation steps:**

1. Add a RED assignment read test whose stored potential rows differ from what current source JSON would project.
2. Add RED mutation tests that corrupt current potential state, attempt one confirmed destructive team removal and one assignment move or clear, and observe the invariant error with unchanged `planner_teams`, `planner_strings`, `planner_assignments`, projected fields, and potential-role rows.
3. Split `get_depth` only at its existing snapshot-resolution boundary: direct reads resolve and assert the optional current snapshot, while one internal loader assembles depth from an already-resolved snapshot ID.
4. In `save_team_settings`, keep `normalize_inputs` before database state work, then resolve and assert the optional current snapshot before opening the transaction or calling `ensure_team_rows`.
5. In `add_string`, `remove_string`, `clear_all`, `clear_assignment`, `assign_player`, and `move_player`, preserve current pure request checks, then resolve and assert the optional current snapshot before `ensure_depth`, `unchecked_transaction`, or any write. Keep the no-snapshot path unchanged by skipping only the assertion when resolution returns `None`.
6. Return the preflighted `Option<i64>` snapshot ID from each mutation service as an internal tuple value with its existing result. Route `save_planner_teams`, `add_planner_string`, `remove_planner_string`, `clear_planner_depth`, `clear_planner_assignment`, `assign_planner_player`, and `move_planner_player` through the depth loader with that ID after their service mutation succeeds. Keep command-level enum parsing before the database lock and keep the ID out of IPC DTOs.
7. Keep `optimize_depth_with_basis` preflight before `ensure_depth` and its assignment transaction, and use the already-preflighted loader for its post-commit response.
8. Extend assignment SQL joins for the two selected potential roles and current model version.
9. Remove projection-source columns, JSON parsing, catalog lookups, and scoring from `resolve_assignment`.
10. Preserve `combine_role_scores`, assignment states, lane weighting, nullable-score behavior, and existing mutation error semantics after a valid assertion.
11. Update mutation-heavy test helpers to explicitly rebuild derived state when their setup models a supported post-write state.

**Tests and proof:**

- Adapt `assignment_state_uses_managed_club_membership_not_team_level` and potential combined-score tests to seed/rebuild persisted potential rows.
- Add a no-recalculation test that expects stored role values despite divergent source values.
- Add depth-read tests for one deleted role row and one wrong-version role row. Under write-denying triggers, each must fail before assignment potential rows are read and leave the database unchanged. Commit 2's shared tests remain the direct projected-map failure proof; add one depth projected-map case only if the call-order assertion is not otherwise observable.
- In `teams_tests.rs`, start from a populated removable team, corrupt one current potential row or projected-map version, snapshot all Planner team/string/assignment rows and all potential-derived fields/rows, then call confirmed `save_team_settings`. It must return the shared invariant error before the destructive transaction, leave every snapshot byte-for-byte unchanged, and trip no write-denying trigger on projected fields or `player_potential_role_scores`.
- In `depth_tests.rs`, start from an existing assignment with a valid destination, corrupt current potential state, snapshot the same Planner and derived tables, then call `move_player` or `clear_assignment`. It must return the invariant error and leave teams, strings, assignments, projected fields, and potential-role rows unchanged. This test must fail if preflight remains only in the post-mutation `get_depth` response.
- Add focused validation-order coverage: invalid team inputs still return their existing normalization error before invariant/database work, an unconfirmed clear still returns its confirmation error first, command-level invalid team or optimizer basis parsing remains before the database lock, and no-snapshot mutation behavior remains unchanged. Retain existing successful mutation, confirmation, save-isolation, and optimizer rollback tests.
- Retain unresolved replacement, outside-pool, current score, assignment mutation, team, tactic, fixtures, mocks, snapshots, and helpers unless the removed read-time setup makes an asset unused. Add only a narrow table-snapshot helper if both corruption tests can share it without duplicating production logic.

**Patterns to verify:**

- Current `player_role_scores` lane joins; `combine_role_scores`; resolver exact-version identity; Planner test-support score setters; `save_team_settings` pure normalization before its transaction; existing command-level `PlannerTeam::parse` and `ScoreBasis::parse`; optimizer's pre-transaction mutation guard.

**Constraints and non-goals:**

- Do not change lane weight, current scores, player state resolution, slot candidate behavior, tactic validation, confirmation rules, save isolation, or no-snapshot results. Do not repair missing potential rows.
- The assertion must run before `ensure_depth` or any mutation transaction/write on every listed mutation service. Preserve pure untrusted-request validation before database work. Pass only the resolved `Option<i64>` through internal return tuples; do not add a generic preflight framework, assertion token type, or public unchecked depth API. The one internal loader exists only to avoid a duplicate successful-response assertion.

**Dependencies and sequencing:**

- Requires Commits 2–4. Independent from later Planner consumers but ordered before them to keep each conversion reviewable.

**Validation:** Run `./scripts/dev check-rust` for focused depth RED/GREEN proofs, then `./scripts/dev check` as the commit gate.

**Stop conditions:** Stop and replan if assignment resolution needs potential scores outside the fixed lane pair, if exact-version joins change missing semantics, if any listed Planner mutation cannot preflight before `ensure_depth` and its first transaction/write, if preserving request-validation or no-snapshot semantics conflicts with preflight, if the post-mutation response requires a second assertion to be truthful, or if test setup reveals a supported writer that does not rebuild potential state.

**Review mandate:** Verify direct `get_depth` still asserts before setup/read; all seven listed mutation commands reach a service preflight before any team/string/assignment mutation; confirmed destructive team removal and assignment mutation cannot commit under corrupt potential state; teams, strings, assignments, and derived state remain unchanged with no derived writes; pure validation and no-snapshot ordering remain intact; optimizer keeps its pre-transaction guard; only already-preflighted paths bypass a duplicate response assertion; missing/wrong-version rows fail instead of becoming null; and assignment score, state, weighting, and fixture value remain correct.

#### Commit 7 — Read persisted Planner role reference scores

**Status:** Completed

**Provisional commit:** `refactor(planner): read stored role reference potential`

**Work:** Convert Planner best-role reference to stored potential rows for the selected tactic phase.

**Size assessment:** Estimated 60–110 changed non-test implementation lines. Within the soft target.

**Out of scope:**

- Assignment cells, optimizer, tactic definitions, foot/familiarity formulas, Search, and Squad.

**Implementation packet:**

- Load exact-version potential role scores beside current role scores for the bounded tactic-role set. Keep the existing phase fit, lane choice, ties, assignment, and sorting logic.

**Files and responsibilities:**

- `src-tauri/src/features/planner/role_reference.rs::{get_role_reference, load_players}` — call the shared assertion after current snapshot resolution and before player loading; include required stored potential-role rows and remove projection/scoring from the player loop.
- `src-tauri/src/features/planner/role_reference_tests.rs` and narrow test support — refresh persisted setup and prove stored values drive Current/Potential results without read writes.

**Behavior and data flow:**

- The service resolves the current snapshot and managed club, validates tactic phase roles, calls the shared snapshot-wide assertion before `load_players`, loads managed-club current players and both score bases for those roles, applies unchanged familiarity/foot fit, selects each player's best lane for the requested basis, and returns both scores for that lane. It never loads projection inputs or writes.

**Ordered implementation steps:**

1. Add a RED selected-basis test with persisted potential rows that intentionally differ from source projection.
2. Invoke `assert_current_snapshot_complete` once before `load_players` for both Current and Potential basis requests because both response shapes expose `potential_score`.
3. Extend `load_players` to collect exact-version potential rows for current snapshot players.
4. Replace `score_role(projected_attributes, role)` with the stored score lookup.
5. Remove projection inputs/imports and preserve phase validation and fit rules.
6. Update setup helpers to rebuild only when a test intentionally changes supported projection sources.

**Tests and proof:**

- Adapt `selected_basis_changes_the_best_lane_and_keeps_both_scores_for_that_lane` and related potential tests to persisted rows.
- Add a no-recalculation/no-write test with divergent source and stored values.
- Add missing-row and wrong-version role-reference cases under write-denying triggers. Both Current and Potential basis requests must return an invariant error rather than omit a player, choose another lane, or return null, and database state must remain unchanged.
- Retain current-phase, familiarity, preferred-foot, no-eligible, tie-order, managed-club, and tactic validation tests. Remove no fixtures or snapshots unless a helper becomes unused.

**Patterns to verify:**

- `load_players` current-role aggregation; `phase_fit_score`; current tactic role validation; exact-version potential query expressions.

**Constraints and non-goals:**

- Do not change phase choice, lane selection, score ties, position familiarity, foot preferences, response ordering, or managed-club scope.

**Dependencies and sequencing:**

- Requires Commits 2–4. Follows assignment conversion to isolate review surfaces.

**Validation:** Run `./scripts/dev check-rust` for focused role-reference RED/GREEN proofs, then `./scripts/dev check` as the commit gate.

**Stop conditions:** Stop and replan if current tactic roles can lack complete rows under supported lifecycle, if persisted scores cannot preserve both-basis DTOs, or if conversion requires changing fit semantics.

**Review mandate:** Verify the assertion runs before player potential loading for both bases; missing/wrong-version data fails instead of changing eligibility or lane choice; exact tactic-role loading; Current/Potential behavior; phase, familiarity, foot, tie, and ordering preservation; no projection or writes; and a test that detects recalculation.

#### Commit 8 — Read persisted potential optimizer scores

**Status:** Completed

**Provisional commit:** `refactor(planner): read stored optimizer potential`

**Work:** Convert potential optimizer candidates to persisted role rows while keeping the shared allocation algorithm unchanged.

**Size assessment:** Estimated 70–130 changed non-test implementation lines. Within the soft target.

**Out of scope:**

- Current optimizer inputs, allocation/matching changes, tactic persistence, role reference, Search, Squad, and performance redesign.

**Implementation packet:**

- Load each eligible candidate's exact-version potential scores for the tactic role set and feed them through unchanged lane combination and fit logic. Remove projected attribute work from the optimizer invocation.

**Files and responsibilities:**

- `src-tauri/src/features/planner/optimizer.rs::{optimize_depth_with_basis, load_potential_optimizer_candidates}` — resolve and assert the current snapshot before `ensure_depth` or the assignment transaction can write for both bases, then replace CA/PA/age/attributes projection with persisted role-score loading while retaining age eligibility, position and foot inputs, lane scoring, and candidate filtering.
- `src-tauri/src/features/planner/optimizer_tests.rs` and Planner test support — adapt potential setup and add stored-versus-source and no-write proofs.

**Behavior and data flow:**

- Optimization resolves the current snapshot and validates its complete potential state before `ensure_depth` can create Planner rows or the optimizer transaction can delete or insert assignments. This applies to both bases because the returned `get_depth` result always exposes assignment potential scores. Potential optimization then filters the managed-club cohort by team age eligibility, loads positions, foot, and exact-version role rows needed by tactic lanes, combines and adjusts scores as before, and passes candidates to the unchanged exact matcher and assignment transaction.

**Ordered implementation steps:**

1. Add a RED optimizer test where persisted potential scores select a different player than source projection would.
2. Resolve the current snapshot before `ensure_depth` and invoke `assert_current_snapshot_complete` before any Planner setup or `unchecked_transaction` for both score bases. Preserve the existing no-snapshot error contract.
3. Query the tactic's unique potential role set with exact model identity for eligible candidates.
4. Build lane scores from stored IP/OOP values and existing `combine_role_scores` and `lane_fit_score`.
5. Remove projection-source selection, JSON attribute parsing, catalog scoring, and projection imports.
6. Preserve matching, manual reservations, provenance, team order, rollback, and tie breaks.
7. Update test setup through the eager writer rather than duplicating formulas.

**Tests and proof:**

- Adapt `optimizer_switches_between_current_and_projected_candidate_scores` to current versus persisted potential scores.
- Add a write-denying trigger around potential optimization and a source-divergence assertion that fails if projection returns.
- Add missing-row and wrong-version tests for both Current and Potential optimization. Each must return an invariant error before optimizer-owned assignments are deleted or inserted; compare assignments and derived rows before and after to prove zero mutation.
- Retain matcher optimality, zero-score, UID tie, team availability/age, familiarity, preferred-foot, manual assignment, rollback, provenance, and current-basis tests.
- No frontend tests, fixtures, mocks, or snapshots change because the command contract is unchanged.

**Patterns to verify:**

- `load_current_optimizer_candidates` persisted-score shape; `lane_fit_score`; `combine_role_scores`; exact matching and transaction boundaries.

**Constraints and non-goals:**

- Do not unify current and potential SQL at the cost of clarity, change allocation, add full-cohort IPC data, or calculate missing rows.

**Dependencies and sequencing:**

- Requires Commits 2–4. Read-only conversion is independent of Commits 6–7 but follows them for focused review.

**Validation:** Run `./scripts/dev check-rust` for focused optimizer RED/GREEN proofs, then `./scripts/dev check` as the commit gate.

**Stop conditions:** Stop and replan if the persisted catalog cannot supply all tactic roles, if candidate semantics require a formula change, or if optimizer reads expose a supported lifecycle gap.

**Review mandate:** Verify the shared assertion runs before the assignment transaction for both bases; corruption cannot commit assignment changes; exact-version role loading; unchanged age, fit, foot, rank, matching, tie, manual, provenance, and rollback behavior; no potential calculation/repair; and tests that distinguish persisted scores from source projection.

#### Commit 9 — Make Search potential queries read-only

**Status:** Active

**Provisional commit:** `refactor(search): remove potential materialization`

**Work:** Remove snapshot/page potential materialization from Search while preserving persisted display, filter, sort, total, pagination, null, and trust-boundary behavior.

**Size assessment:** Estimated 70–140 changed non-test implementation lines, mostly deletion. Within the soft target.

**Out of scope:**

- Squad materialization, shared lazy-module deletion, query sort optimization, frontend Search behavior, and metric catalog changes.

**Implementation packet:**

- Validate request/filter inputs as before. If a validated requested field, filter rule, or sort uses potential data, call the shared snapshot-wide read-only assertion once, then execute the existing exact-version SQL directly. Reads must not enumerate or write the cohort before count/page queries.

**Files and responsibilities:**

- `src-tauri/src/features/search/query.rs::search_players_in_view` — reuse validated potential field/filter/sort detection to call the shared assertion once, then delete full-snapshot and page materializer calls and role-list preparation that exists only for writes; retain potential sort joins and dynamic field reads.
- `src-tauri/src/features/search/filter.rs::compile_potential_role_score_rule` — consume the eager model-version owner; preserve bound role/value parameters and null semantics.
- `src-tauri/src/features/player_metrics/resolver.rs` — retain exact-version potential expressions under the eager owner.
- Search tests in `search/query.rs` and filter tests — replace lazy/cache assertions with eager read-only correctness and stale/missing invariant behavior.

**Behavior and data flow:**

- Search validates requested fields before snapshot lookup, preserves the current empty-snapshot result, then compiles filters and uses the already-validated sort identity. It determines whether any validated metric needs potential data only after those current ordering rules. Only then, and only for such a request, it calls `assert_current_snapshot_complete` once before count, sort, page-UID, or dynamic-field queries. Requested potential columns keep scalar rows, filters keep bound `EXISTS`, and sorts keep the existing `LEFT JOIN`, direction, and UID tie. Non-potential requests do not pay for or depend on the assertion. No query writes or repairs.

**Ordered implementation steps:**

1. Install write-denying triggers in RED Search filter, sort, and display tests; delete one catalog row or change its version to expose silent exclusion, null, or ordering from the raw SQL shapes.
2. Reuse the validated dynamic fields, `potential_role_ids_from_ast`, and `potential_role_sort_identity` to compute one `potential_requested` condition. Invoke `assert_current_snapshot_complete` once after request/filter validation and current snapshot resolution but before any affected count or page query.
3. Remove `materialize_snapshot_roles`, `materialize_player_roles`, and role collection used only to choose write scope.
4. Point version references to `potential_scores`.
5. Remove Commit 3's explicit empty/stale lazy-fixture setup and rewrite those seed-adapted tests as complete eager-row query tests plus missing/wrong-version invariant-failure tests. Do not repeat the earlier fixture compatibility repair.
6. Preserve the current validation and empty-snapshot ordering, totals, page bounds, OR/AND behavior, null scores, scalar/`EXISTS`/`LEFT JOIN` SQL, and binding.
7. Remove only Search-owned obsolete helpers/imports.

**Tests and proof:**

- Rewrite the Commit 3-adapted `potential_role_filter_materializes_the_full_snapshot_and_reuses_cached_rows`, stale replacement, page-scoped display, current-role-sort display, potential-sort, and invalid-filter/no-materialization tests. Remove their deliberate empty/stale lazy baselines and assert normal complete eager rows are unchanged before and after reads.
- Add separate potential display, filter, and sort cases for a deleted catalog row and a wrong-version row. Each must return the shared invariant error before its scalar, `EXISTS`, or `LEFT JOIN` query can yield null, exclusion, a changed total, or changed order. Write-denying triggers plus before/after projected fields and role rows prove no repair or mutation.
- Add a non-potential Search case with the same corrupt fixture that preserves current scalar behavior and does not invoke the potential assertion. Retain invalid-request proof so validation still fails before snapshot assertion work.
- Retain potential display/filter/sort correctness, nullable scores, multi-role AND/OR, totals, pagination, UID ties, SQL trust-boundary, current-role, Club DNA, Moneyball, and general Search tests.
- Delete only assertions and helpers whose sole supported contract was lazy Search cache filling. No frontend fixtures, mocks, or snapshots change.

**Patterns to verify:**

- Current-role and Club DNA persisted read paths; existing potential SQL joins/subqueries; request validation before DB work.

**Constraints and non-goals:**

- Do not change filter operators, sort/null behavior, metric IDs, role IDs, page bounds, moneyball behavior, or add fallback writes.

**Dependencies and sequencing:**

- Requires Commits 2–4 and Commit 3's green compatibility adaptations. Shared lazy code remains temporarily for Squad, so do not delete the module yet. This packet owns retiring Search's lazy assertions, not repairing their eager-ingest setup.

**Validation:** Run `./scripts/dev check-rust` for focused Search RED/GREEN proofs, then `./scripts/dev check` as the commit gate.

**Stop conditions:** Stop and replan if Search correctness currently depends on partial role rows under a supported writer, if removing materialization changes totals/order/null semantics, or if request-controlled SQL identifiers appear.

**Review mandate:** Verify the assertion condition covers validated potential display, filter, and sort and no other requests; it runs once before affected count/page SQL; missing/wrong-version data cannot become null, exclusion, or changed order; all Search paths are read-only; exact model identity; filter binding; scalar/`EXISTS`/`LEFT JOIN` parity; null-score, total, page, and tie behavior; invalid-request ordering; and removal of only obsolete lazy tests.

#### Commit 10 — Make Squad potential queries read-only

**Status:** Pending

**Provisional commit:** `refactor(squad): remove potential materialization`

**Work:** Remove managed-club completeness and page materialization from Squad while preserving persisted display and sort behavior.

**Size assessment:** Estimated 60–120 changed non-test implementation lines, mostly deletion. Within the soft target.

**Out of scope:**

- Search, shared lazy-module deletion, frontend Squad behavior, managed-club ownership, and sort redesign.

**Implementation packet:**

- After request validation and current/configured context resolution, call the shared snapshot-wide assertion once when a validated requested field or sort needs potential data. Then execute the existing bounded Squad query against complete current-snapshot rows. Do not enumerate managed-club UIDs or write potential rows before display or sort.

**Files and responsibilities:**

- `src-tauri/src/features/planner/squad.rs::{list_squad_players, list_squad_player_uids}` — detect validated potential requested fields or sort, call the shared assertion once, remove potential cohort-completeness/materialization calls, and delete UID enumeration only if no supported caller remains; preserve the potential sort join and dynamic reads.
- `src-tauri/src/features/planner/squad_tests.rs` — replace page/cohort lazy assertions with eager read-only display/sort and no-repair proof.
- `src-tauri/src/features/player_metrics/resolver.rs` only if the final Squad conversion exposes an eager-version import cleanup.

**Behavior and data flow:**

- Squad validates requested fields, resolves the save's current snapshot and configured managed club, and preserves its empty results when either context is absent. If a validated dynamic field or sort uses potential data, it calls `assert_current_snapshot_complete` once before cohort count, sort, page-UID, or dynamic-field queries. Display keeps scalar reads and sort keeps its exact-version `LEFT JOIN`. Non-potential requests do not invoke the assertion. No cohort completeness count or write occurs.

**Ordered implementation steps:**

1. Add RED write-denying Squad display and sort tests with a deleted or wrong-version row that would otherwise become null or change ordering.
2. Derive one `potential_requested` condition from validated dynamic fields and `potential_role_sort_identity`; call `assert_current_snapshot_complete` once after current/configured context resolution and before count/page SQL.
3. Remove `squad_role_rows_are_complete`, player UID enumeration used only by materialization, and `materialize_player_roles` calls.
4. Retain exact-version sort join, dynamic scalar select, cohort predicate, bounds, and UID tie.
5. Remove Commit 3's explicit lazy-fixture clearing and rewrite the seed-adapted page/cohort tests as eager no-write correctness and invariant-failure tests. Do not repeat the earlier eager-row compatibility repair.
6. Remove Squad-owned helpers/imports made obsolete.

**Tests and proof:**

- Rewrite the Commit 3-adapted `potential_display_is_page_scoped_and_potential_sort_materializes_the_squad_cohort`, current-role-sort potential display, and nullable/tie potential-sort tests. Remove their deliberate lazy baselines, assert complete eager rows exist before the call, preserve the requested page/sort result, and prove row counts and content do not change.
- Add separate display and sort cases for a deleted catalog row and a wrong-version row. Each must return the shared invariant error before scalar or `LEFT JOIN` SQL can yield null or changed order; write-denying triggers and before/after state prove no mutation.
- Add a non-potential Squad case with the same corrupt fixture that preserves current behavior without invoking the assertion. Retain empty results for no current snapshot or no managed-club configuration.
- Retain managed-club scope, unconfigured state, current/potential sort, nullable value, UID tie, bounds, current role, Club DNA, dynamic metric, and page tests.
- Delete only lazy Squad completeness/materialization assertions and helpers with no surviving contract.

**Patterns to verify:**

- Squad current-role persisted sort; Club DNA eager read; current managed-club membership SQL; Search's completed eager conversion.

**Constraints and non-goals:**

- Do not change managed-club membership, page limits, sort direction, null behavior, metric identity, or introduce full-cohort Rust processing.

**Dependencies and sequencing:**

- Requires Commits 2–4 and Commit 3's green compatibility adaptations, and follows Commit 9. This packet owns retiring Squad's lazy assertions, not repairing their eager-ingest setup. After this packet, no product read may call the lazy materializer.

**Validation:** Run `./scripts/dev check-rust` for focused Squad RED/GREEN proofs, then `./scripts/dev check` as the commit gate.

**Stop conditions:** Stop and replan if a supported Squad path can observe incomplete current rows, if direct persisted reads alter cohort/order/null semantics, or if UID enumeration has another supported caller.

**Review mandate:** Verify the assertion condition covers validated potential display and sort only and runs once before affected cohort/page SQL; missing/wrong-version data cannot become null or changed order; zero Squad writes/cohort-completeness work; exact model identity; managed-club context, display, sort, nullable-score, page, and tie behavior; obsolete-helper deletion; and parity with Search's eager contract.

#### Commit 11 — Delete the lazy potential cache paths

**Status:** Pending

**Provisional commit:** `refactor(scoring): delete lazy potential cache`

**Work:** Remove the retired lazy cache module, cohort-completeness paths, stale-row replacement, cache invalidation compatibility, and obsolete test assets after every consumer uses eager persistence and the shared read-only invariant assertion.

**Size assessment:** Net production deletion expected; changed lines may exceed 200 because the coherent removal deletes the roughly 580-line lazy module. Keep it atomic so no dead compatibility layer survives.

**Out of scope:**

- Formula changes, query redesign, migration changes beyond fixes required by compilation, frontend work, historical potential data, and new abstractions.

**Implementation packet:**

- Delete only code and tests whose contract is lazy/disposable read-time materialization. Retain eager writer/version ownership, v21/v34 schema/cascade tests, projection formula tests, and all persisted query behavior.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/potential_cache.rs` — delete the module and its batch, cached-role lookup, stale replacement, persistence, invalidation, and completeness tests.
- `src-tauri/src/features/player_metrics/mod.rs` — remove the lazy module export.
- `src-tauri/src/features/player_metrics/potential_scores.rs` — remain the sole potential derived-state writer/version owner and retain `assert_current_snapshot_complete` as the one read-only consumer guard; absorb only still-needed shared types or constants.
- `src-tauri/src/features/player/service.rs`, `search/query.rs`, `search/filter.rs`, `planner/squad.rs`, and `resolver.rs` — confirm no lazy imports or compatibility calls remain.
- Related tests — remove orphan lazy-only helpers and retain current supported lifecycle/query tests.

**Behavior and data flow:**

- Only migration/backfill, snapshot selection, and boost reconciliation write potential-derived data. Product consumers call the read-only snapshot invariant assertion before affected potential queries, then query exact-version stored rows. No cache-miss, cohort-completeness, partial-cohort, or read-triggered write path remains.

**Ordered implementation steps:**

1. Establish removal proof with repository search showing all supported callers were converted in Commits 4–10.
2. Delete the lazy module export and file.
3. Remove orphan imports, helpers, test fixtures, and compatibility comments.
4. Search every Profile, Planner, Search, and Squad potential consumer and confirm it invokes `assert_current_snapshot_complete` at the packet-defined boundary; confirm non-potential Search/Squad paths remain conditional.
5. Search for `project_attributes` and confirm production calls remain only in the eager writer plus the projection implementation itself.
6. Search for potential-row INSERT/UPDATE/DELETE and confirm writers are limited to migration/eager lifecycle and test setup.
7. Run all Rust and commit gates.

**Tests and proof:**

- Remove `potential_cache.rs` completeness tests and Search/Squad/boost assertions that existed only to prove lazy fill, stale replacement, or invalidation; those contracts were rewritten in prior packets before deletion.
- Retain v21 table schema/cascade and v34 current-only migration tests, eager writer and invariant-assertion tests, snapshot lifecycle tests, boost atomicity tests, Profile/Planner/Search/Squad persisted-read and corruption-failure tests, projection formula tests, and current-role tests.
- Add no absence test solely for file deletion. The retained missing-row, wrong-version, projected-map, and no-write product-consumer tests plus source search are stronger observable protection against reintroducing lazy behavior.

**Patterns to verify:**

- ADR-0024's deletion of Club DNA lazy ownership; repository contract-removal procedure; compiler dead-code/import checks.

**Constraints and non-goals:**

- Do not remove the potential table, index, model version, nullable-row semantics, projection service, or query contracts. Do not retain a deprecated shim or speculative generic derived-state framework.

**Dependencies and sequencing:**

- Requires Commits 4–10. This is the final implementation packet and cannot begin while any lazy caller remains.

**Validation:** Use `rg -n "potential_cache|materialize_(snapshot|player)_roles|squad_role_rows_are_complete|invalidate_player_cache" src-tauri/src` as removal evidence and `rg -n "project_attributes" src-tauri/src --glob '*.rs'` to confirm the sole production writer; run `./scripts/dev check-rust`; then run `./scripts/dev check` as the commit gate.

**Stop conditions:** Stop and replan if a production caller still needs the module, if deletion removes the shared read-only assertion or a supported exact-version/query contract, if any potential consumer can query before validation, if projected computation remains in a product read, or if tests reveal another supported source mutation that lacks eager reconciliation.

**Review mandate:** Verify complete lazy-caller removal; retained shared assertion and exact consumer coverage; no lazy compatibility shim; correct version/table/query ownership; obsolete test/helper cleanup; sole production projection owner; writer-only potential mutations; material no-write corruption proofs; and full validation discovery.

## Active work

**PR:** PR 1 — Precompute current-snapshot potential scoring

**Commit:** Commit 9 — Make Search potential queries read-only

### RED or removal proof

Install write-denying triggers and corrupt one eager role row so Search potential display, filter, or sort reaches the old materializer and attempts repair. The focused test must fail before materialization removal.

### Expected outcome

Search validates requests, conditionally asserts complete current potential state when fields, filters, or sort need it, then directly counts, filters, sorts, and pages exact-version persisted rows. Missing/stale state returns the shared invariant error without writes; totals, nulls, pagination, ties, SQL binding, and non-potential paths remain unchanged.

### Explicit exclusions

Squad conversion, shared lazy-module deletion, filter/operator or sort redesign, Moneyball changes, frontend behavior, and `.wiki/features/completed/player-table-sort-performance.md`.

## Discoveries and replanning

- No planned feature spec exists, so there is no promotion or deletion path.
- The repository's stable script surface has no Rust test-name forwarding. Packets use focused Rust test cases discovered by `./scripts/dev check-rust`, followed by `./scripts/dev check` as the commit gate.
- CodeGraph was unavailable for this checkout and pi-lens reports a stale review graph. The plan therefore verifies symbols against current source reads and repository tests; indexed relationship output is advisory only.
- The initial plan completed independent review and recorded fingerprint `e6335f4e8bbff83fa77bd23c71fbb1c3c0ed987989d57d59b98258a01edfc9f7` before the required Commit 1 checkpoint review.
- That checkpoint review found a HIGH execution defect: retained scalar, `EXISTS`, and `LEFT JOIN` query shapes could turn a missing or wrong-version potential row into null, exclusion, or changed order after lazy completeness checks were removed. The correction adds one shared snapshot-wide read-only assertion and places it before every affected Profile, Planner, Search, and Squad read.
- The checkpoint review also found MEDIUM stale plan state: Unknowns, Active work, and Discoveries still described the completed initial review and fingerprint calculation as pending. The correction preserves that historical fact; focused correction review cleared and `delivery_state.py` recorded the replacement fingerprint.
- This is a substantive packet correction without a scope, PR-boundary, or commit-topology change. Commits 2 and 5–11 require new packet fingerprints and a fresh worker run after focused independent correction review; no implementation can resume under the superseded delivery fingerprint.
- Correction round 2 found a second HIGH at the Planner mutation-response boundary: `save_planner_teams`, `add_planner_string`, `remove_planner_string`, `clear_planner_depth`, `clear_planner_assignment`, `assign_planner_player`, and `move_planner_player` currently commit their service mutations before calling `get_depth`. A `get_depth`-only assertion would therefore report corrupt potential state after product-owned Planner data changed, and confirmed team removal can delete assignments first. Commit 6 now preflights each service before setup or mutation, preserves request-validation and no-snapshot ordering, and uses one internal already-preflighted depth loader to avoid a second successful-response assertion. The optimizer's existing planned pre-transaction guard remains unchanged. Focused correction review accepted this boundary correction with no remaining finding.
- Commit 3 disproved Commit 2's complete-source-map assumption. Supported dump `attributes` maps are sparse: validation permits omitted keys, ingest persists only supplied entries, and production fixtures use that shape. The committed `{}` rollback test therefore characterized supported input as invalid. The correction normalizes the closed `DUMP_ATTRIBUTE_KEYS` set inside the eager writer, treats omitted supported attributes as null before the single projection, and preserves malformed/type/domain rejection. Missing and null remain equally unavailable to current queries, while persisted projected maps become complete and assertion-compatible. ADR-0026's durable eager ownership and lifecycle decision does not change.
- Activating eager materialization also disproved the packet assumption that existing Search, Squad, and boost tests would still start with zero or sparse potential rows. The 14 failures in `/tmp/commit3-check.log` comprise the obsolete `{}` rollback, one projected-value expectation (`Acceleration` is 11 after projection, not source 10), Search/Squad zero/page-count assumptions and duplicate custom inserts, and boost custom seeds that collide with eager rows. Commit 3 now owns the minimum trunk-safe test setup and assertion adaptations. It explicitly clears eager rows only where a still-supported lazy materializer must remain protected until Commits 9–10, and uses `UPDATE` or `DELETE` plus `INSERT` for custom score/version setup.
- The snapshot lifecycle draft already places selector ownership and rollback proofs in `snapshot/service.rs`, `snapshot/ingest.rs`, and `player_metrics/potential_scores.rs`; the known failures do not disprove that architecture. Commit 3 remains one coherent atomic commit because enabling eager lifecycle and reconciling its immediate supported-input and test-baseline consequences cannot land separately on a green trunk. Commit 4 remains safely ordered after it: Commit 3 preserves current boost invalidation behavior, then Commit 4 changes that behavior to eager one-player replacement. Commits 9 and 10 own the later removal of Search/Squad lazy behavior, not a second copy of Commit 3's compatibility repair.
- This replan changes Commit 3 and clarifies the test-ownership and dependency packets for Commits 4, 9, and 10. Their prior packet fingerprints and Delivery fingerprint `95f8488c8bbb78735e6afdf9b4ffdedb7ed28bce8964b81734105b8958cd1b94` are invalid. Focused replan review accepted the replacement packets and `delivery_state.py` computed the new authorization.
- Focused replan correction round 1 retained a MEDIUM source-domain finding in Commit 3. `dump_validation` accepts any JSON number, `attributes_map` accepts `u8` zero, and the draft eager writer normalized and projected before checking the FM visible domain. A growth-eligible supplied `Acceleration: 0` could therefore become projected `1` and pass the persisted assertion. The corrected packet validates every supplied non-null `DUMP_ATTRIBUTE_KEYS` value as integer `1..=20` before normalization or projection, retains the open typed contract for unknown source keys, and excludes unknown keys from the exact 47-key projected output. New writer, migration, and winning-ingest RED/GREEN rollback proofs own the correction. Focused correction review accepted the packet with no remaining finding.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Precompute current-snapshot potential scoring | Commit 1 — Record the approved feature plan | 52be1f96b1c06177d4b92fa52ef8f2f8e673c064 | Recorded the reviewed schema 2 ledger, TODO activation, superseded ADR-0019, accepted ADR-0026, and ADR index update. | `ledger_state.py`: runnable; `delivery_state.py`: runnable; `git diff --cached --check`: passed. | Not applicable | Clear | 2 | Checkpoint review added a shared read-only derived-data invariant assertion and pre-mutation Planner guards before the replacement fingerprint was accepted. |
| PR 1 — Precompute current-snapshot potential scoring | Commit 2 — Add eager current-potential persistence | 71c6f8bec15e4c495349c0cf6d948fd03524fa4f | Added migration v34, atomic current-only backfill, one-projection eager writer, exact projected-map and catalog-row invariant assertion, shared model-version ownership, and truthful migration errors. | `./scripts/dev check-rust`: 611 passed, 2 ignored; `./scripts/dev check`: passed; LSP and `git diff --cached --check`: passed. | Pass | Clear | 3 | The atomic migration/writer/assertion implementation exceeded the soft estimate; review corrections strengthened projected-map validation, writer postconditions, rollback context, one-call proof, and nullable SQL-row proof without changing packet scope. |
| PR 1 — Precompute current-snapshot potential scoring | Commit 3 — Maintain potential data across snapshot selection | 1de1e8540fb8c12886c72932655275f7a0f94b50 | Added selector-owned winner materialization and loser clearing, deletion-promotion/final-deletion lifecycle, write-free save switching, sparse source normalization with pre-projection domain validation, and eager-baseline-compatible lazy/boost tests. | `./scripts/dev check-rust`: 619 passed, 2 ignored; `./scripts/dev check`: passed; LSP and `git diff --cached --check`: passed. | Pass | Clear | 1 | Replanned after supported sparse maps and eager test baselines disproved the original packet assumptions; compatibility fixes preserved deferred lazy and boost behavior. |
| PR 1 — Precompute current-snapshot potential scoring | Commit 4 — Recompute boosted player potential atomically | 556a2071f4f55a8e50fa3d6fca2591261e83426a | Replaced boost-time invalidation with post-update one-player projected-map and complete potential-role replacement inside the existing reconciliation transaction. | `./scripts/dev check-rust`: 621 passed, 2 ignored; `./scripts/dev check`: passed; LSP and `git diff --cached --check`: passed. | Pass | Clear | 0 | Updated one invalid Determination-zero fixture to nullable input under Commit 3's enforced source-domain contract; supported missing-value behavior remains covered. |
| PR 1 — Precompute current-snapshot potential scoring | Commit 5 — Read persisted profile potential values | e8db28c8bf6372dc073d114b0d6c579084bb4d21 | Converted known-player profile reads to shared-invariant-guarded projected JSON and exact-version persisted potential rows, removing production read-time projection and scoring. | `./scripts/dev check-rust`: 626 passed, 2 ignored; `./scripts/dev check`: passed; LSP and `git diff --cached --check`: passed. | Pass | Clear | 0 | Materialized one Staff test fixture that inserts a player directly so its profile read satisfies the new invariant; production Staff behavior is unchanged. |
| PR 1 — Precompute current-snapshot potential scoring | Commit 6 — Read persisted Planner assignment potential | 09a591be7d06eab5c8851186537c97b933c62915 | Converted assignment potential to exact-version stored IP/OOP rows and added pre-write potential preflight plus already-validated response loading for every depth-returning Planner mutation. | `./scripts/dev check-rust`: 634 passed, 2 ignored; `./scripts/dev check`: passed; LSP and `git diff --cached --check`: passed. | Pass | Clear | 1 | None. |
| PR 1 — Precompute current-snapshot potential scoring | Commit 7 — Read persisted Planner role reference scores | 51a89ac4e7e1d4e3e378b2e61c05d0b608f6aa8d | Converted Planner role reference to shared-invariant-guarded exact-version persisted potential tactic-role rows while preserving fit, lane, tie, and ordering behavior. | `./scripts/dev check-rust`: 636 passed, 2 ignored; `./scripts/dev check`: passed; LSP and `git diff --cached --check`: passed. | Pass | Clear | 0 | Extended the shared Planner no-write trigger helper to projected player fields for corruption proofs. |
| PR 1 — Precompute current-snapshot potential scoring | Commit 8 — Read persisted potential optimizer scores | Pending record | Converted potential optimizer candidates to exact-version persisted tactic-role rows while retaining all allocation, fit, age, tie, reservation, provenance, and rollback behavior. | `./scripts/dev check-rust`: 638 passed, 2 ignored; `./scripts/dev check`: passed; LSP and `git diff --cached --check`: passed. | Pass | Clear | 0 | None. |

## Final validation

- `./scripts/dev check-rust` — all Rust format, Clippy, migration, persistence, snapshot lifecycle, boost, Player Profile, Planner, Search, and Squad tests pass and discover the focused feature tests.
- `./scripts/dev check` — required complete commit/feature gate passes after lazy assets are removed.
- `rg -n "project_attributes" src-tauri/src --glob '*.rs'` — production projection remains only in the eager derived-state writer; projection module tests may call it directly.
- `rg -n "potential_cache|materialize_(snapshot|player)_roles|squad_role_rows_are_complete|invalidate_player_cache" src-tauri/src` — no retired lazy path remains.
- `rg -n "assert_current_snapshot_complete" src-tauri/src/features/{player,planner,search,player_metrics}` — the shared assertion and every planned consumer call remain; inspect the matches to confirm conditional Search/Squad use and pre-transaction optimizer use.
- Focused Rust corruption tests prove a deleted role row, wrong-version role row, null projected map, and wrong-version projected map return invariant errors under write-denying triggers and leave projected fields, role rows, and optimizer assignments unchanged.
- Focused Planner mutation tests prove corrupt current potential state blocks a confirmed destructive team/string change and an assignment mutation before commit, returns the shared invariant error, leaves `planner_teams`, `planner_strings`, `planner_assignments`, projected fields, and potential-role rows unchanged, and performs no derived write.
- Inspect a v33-to-v34 test database result: every save's effective current snapshot has projected JSON and players × catalog role rows; retained historical snapshots have neither.
- Inspect the exact feature diff and confirm no frontend scoring, formula/catalog changes, BACKLOG edits, release files, or unrelated completed-feature documentation are present.

## Documentation impact

Complete during reconciliation. Expected owners are `.wiki/ARCHITECTURE.md` for implemented eager current-snapshot scoring, the completed feature ledger, `.wiki/TODO.md`, ADR-0019, ADR-0026, and the ADR index. Do not describe the planned architecture as implemented before delivery.
