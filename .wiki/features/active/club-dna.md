# Club DNA

## Status

Validation

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** f3b0d9469bdad80e388af3cdb915383af95005bcc7abc3777059f44aecfee49b

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
- Definition creation and edit calculate scores synchronously for every player in every retained snapshot of that save. The operation can take longer as retained snapshot and player counts grow, and any failure preserves the prior definition and scores.
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
- A lazy batch materializer, pre-query completeness work, background score jobs, a new migration or index, or general role-score sort optimization.
- New table layout ownership, a new table component, or global removal of saved layout, filter, sort, or history state.

## Current-state map

- Relevant components: `src/app/routes/my-club.tsx::MyClubPageContent` composes the My Club header and `ManagedClubSelector`; `src/features/managed-club/components/managed-club-selector.tsx::ManagedClubSelector` owns the selector form and **Save managed club** action; `src/components/ui/modal/modal.tsx::Modal` owns dialog focus and dismissal; `src/features/player-profile/utils/attribute-groups.ts` owns the current FM-style frontend attribute grouping.
- Data model: `players` stores visible and goalkeeper values in `attributes_json`, hidden values in `hidden_attributes_json`, and personality values in `personality_json`. Committed v31 and `src-tauri/src/features/club_dna/` provide one save-keyed definition plus context-bound get/set/remove commands. Committed v32 adds positive definition versions and the nullable, versioned `club_dna_scores` cache.
- Persistence and migrations: committed migration v31 adds the save-owned definition. Committed migration v32 preserves v31, adds `definition_version`, and adds nullable `club_dna_scores` with identity `(snapshot_id, uid, definition_version, score_model_version)` plus the `(snapshot_id, definition_version, score_model_version, score)` lookup/order index. `src-tauri/src/features/player_metrics/club_dna.rs` currently owns pure scoring, bounded lazy materialization, and exact-version writes. Definition changes and supported player boosts currently invalidate score rows.
- Existing behavioral assumptions: most React query keys omit save IDs because app-shell and Settings context changes invalidate feature roots, but save-owned Club DNA cannot rely on invalidation alone. The established saves query exposes each active save's ID and immutable `contextToken`. Search filters and sort live in URL state. Search and Squad column IDs, order, and widths live app-locally in Zustand key `fm-valuescout-player-table-layouts`, version 5. `addColumns` validates IDs and appends only missing columns.
- Architectural seams: `src/utils/player-metrics.ts` and `src/components/ui/player-metric-picker.tsx` own the frontend fixed metric catalog and picker. `src-tauri/src/features/player_metrics/resolver.rs::MetricField` owns the independent Rust catalog and dynamic value/sort expressions. Current role scores are persisted during ingest, resolve through scalar dynamic SQL for display/sort, and filter through a bound `EXISTS` query. Their scalar sort pattern does not guarantee null-last ordering in both directions. `src-tauri/src/features/search/filter.rs` owns trusted Search filter compilation. `src-tauri/src/features/search/query.rs` and `src-tauri/src/features/planner/squad.rs` own the current-snapshot Search and exact managed-club Squad cohorts.
- Shared table adapters: `src/features/search/components/search-results-panel.tsx` and `src/features/squad/components/squad-overview-panel.tsx` request visible dynamic fields and render nullable values. `src/components/player-table/` owns table interaction. Existing dynamic DTO maps already carry nullable integers.
- Command boundary: `src/lib/tauri-client.ts` is the sole frontend invoke wrapper. `src-tauri/src/lib.rs` registers Tauri commands, and `src-tauri/src/features/mod.rs` registers feature modules.
- Project validation commands: `./scripts/dev test <targets>`, `./scripts/dev check-app`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` is unsupported and cannot be evidence.
- Primary risks: cross-language catalog drift, accepting an empty or unknown definition, partial averages, stale or mixed-version score rows, partial definition or ingest writes, leaving a boosted player with a stale score, synchronous definition cost across retained history, repeated per-player definition lookup, stale active-save Query data, re-adding a user-removed column on edit, and removing saved layout state with the definition.

## Feature architecture

Migration v31 remains the save-owned definition foundation. Migration v32 remains immutable and supplies positive definition versions, nullable `club_dna_scores`, exact `(snapshot_id, uid, definition_version, score_model_version)` identity, score-domain checks, the existing lookup/order index, and snapshot/player cascades. No v33 migration, index, or dependency is added. ADR-0024 supersedes ADR-0023's lazy lifecycle while reusing v32 as the persisted score table.

Definition set and edit use one transaction. The service validates the active save context and canonical definition, updates the definition with its new positive version, deletes prior score rows for the save, and invokes the shared pure scorer for every player in every retained snapshot. It persists exactly one nullable row for each snapshot/player at the new definition version and Rust-owned score-model version. Any read, calculation, insert, or commit failure rolls back the complete transaction, including the definition change and score deletion. Synchronous definition cost therefore scales with all retained snapshots and their player counts. Definition removal deletes the definition and all save-owned score rows atomically. Snapshot/player foreign-key cascades stay unchanged.

Snapshot ingest remains one transaction. After the transaction inserts players and current role scores, it reads the save's definition once. If absent, ingest performs no Club DNA scoring or writes. If present, it validates that stored definition, calculates each newly inserted player from the validated dump or the just-stored JSON values through the same pure scorer, and persists one nullable exact-version/model row for every player in the new snapshot. A Club DNA failure rolls back the snapshot, players, role scores, Club DNA rows, and other ingest-owned writes. Every retained snapshot therefore already has scores if it later becomes current; promotion adds no Club DNA work.

Successful player-boost reconciliation keeps stored source data and derived values in the existing SQLite source-data transaction. After it applies the verified player update and current-role-score changes, it reads the snapshot's current save definition. If present, it recalculates and upserts that exact snapshot/player score from the transaction-visible stored player JSON with the same scorer and current definition/model identity. If absent, it performs no Club DNA work. A scoring or persistence failure rolls back the stored player, current role-score, and Club DNA changes together. Football Manager can already have changed before this SQLite transaction starts, so any eager Club DNA reconciliation failure maps to `PlayerBoostError::SnapshotSync` and the existing Load Data recovery path. The plan does not claim that this local rollback reverses the external FM change. The implementation removes Club DNA cache invalidation because no read-time materializer remains.

Search and Squad treat `club_dna` like a fixed nullable persisted score. The request resolves the active snapshot's current definition version and Rust-owned score-model version once and binds that identity into scalar display/sort SQL and filter SQL. Filters require a matching non-null row for every operator, including `neq`. Club DNA deliberately diverges from the current-role and potential-role scalar sort pattern only in its ordering wrapper: `score IS NULL ASC`, then score in the validated requested direction, then UID ascending. This guarantees null-last for both ascending and descending sorts. Missing definition or missing exact-version/model row yields null for display, false for that filter rule, and an all-null UID-stable result for sort without dropping players. The flat filter AST therefore keeps non-Club-DNA matches in mixed OR requests while isolated and mixed AND Club DNA rules cannot match that player. Existing requested-field, 32-rule filter, limit, offset, and page bounds remain unchanged. SQL interpolates only the fixed internal metric ID and validated sort direction; snapshot, identity, values, limits, and offsets remain bound. Current-role and potential-role ordering remain unchanged, and general role-score optimization stays out of scope.

The backend implementation deletes the lazy batch materializer, its cache-miss and completeness instrumentation, the Club DNA invalidation paths, and obsolete tests together. It preserves the pure scorer, v32 migration/schema/version/cascade characterization, nullable score rules, and independent role/potential-role contracts. No query performs Club DNA pre-materialization, completeness checks, cohort enumeration for scoring, or background work.

The unique Club DNA 2,000-player and 184,000-player performance gates are removed. The exact discarded final evidence remains decision context: 2,000-player warm p95 was 4.129 ms for Search filter, 6.527 ms for Search sort, and 11.339 ms for Squad sort; 184,000-player warm p95 was 514.287 ms, 611.003 ms, and 1596.226 ms. Current-role scores already use persisted rows and scalar dynamic SQL for display/sort, with no representative latency gate. Validation now matches that portfolio and focuses on atomic eager lifecycle, nullable scoring, bounded query correctness, and absence of lazy materializer/cache work. General role-score sorting is not optimized in this feature.

React behavior stays unchanged from the accepted product plan: one fixed integer metric uses backend-supplied values, typed definition adapters bind the active save context, create-only layout append remains guarded, and edit/remove do not rewrite layouts or URL state.

## Uncertainty register

### Known

- Linear JAY-32 is the external work item. There is no planned feature spec to promote.
- Commits 1–7 are complete at `ddd4961e6d90ca24faa435955c6ae7eb5a716f0b`, `d2682ee5c50cb99cd0b7f9facf5fd4f9060d5001`, `7cf5e5924af8a9c54852f5037e17ffe4b2c58cc0`, `d78f97f25497409f6c895a8ac5cdeb74ea5301eb`, `df074b9da78bec038960e2be7c851dcb5879dbdd`, `8250dbe9aac7853ed90ba674f83a67da870a8ecb`, and `2c6943f68fdfaf9311977f024a0d59192a4aed58`. Completed packets and refs are immutable. The active Commit 8 indexed lazy-query attempt was discarded cleanly.
- Clean HEAD `2c6943f68fdfaf9311977f024a0d59192a4aed58` contains v31 definitions and v32 definition versions, nullable score rows, exact identity, index, pure scoring, bounded lazy materialization, and invalidation. Search and Squad do not expose Club DNA.
- The discarded final 2,000-player run measured warm p95 of 4.129 ms for Search filter, 6.527 ms for Search sort, and 11.339 ms for Squad sort.
- The discarded final 184,000-player run measured warm p95 of 514.287 ms for Search filter, 611.003 ms for Search sort, and 1596.226 ms for Squad sort. Squad sort breached the accepted one-second representative gate.
- Current ingest persists about 70 current role scores per player inside the snapshot transaction. Current-role dynamic display and sort use stored score rows through a scalar expression; filters use a bound `EXISTS` query with a non-null score guard. That scalar ordering does not guarantee null-last in both directions. No representative role-score latency gate exists.
- Search supports display, sort, and filter. Squad supports display and sort. Both already carry nullable integer dynamic values.

### Assumptions

- Canonical metric-style IDs remain the narrowest persisted definition because they encode the closed attribute and JSON source.
- A route-owned action slot in `ManagedClubSelector` is sufficient to place **Define DNA** beside **Save managed club** without a cross-feature import.
- The existing `ScoreBadge` is the correct 0–100 presentation in both tables.

### Decisions

- Supersede ADR-0023 with ADR-0024. Reuse committed migrations v31/v32, including `club_dna_scores`, exact identity, nullable score domain, index, versions, and cascades. Do not add v33, another index, or a dependency.
- Match the current-role score persisted lifecycle and dynamic read model. Eagerly calculate and persist Club DNA at definition set/edit, snapshot ingest when a definition exists, and successful player-boost reconciliation when a definition exists.
- Definition set/edit deletes prior save-owned score rows and persists one exact current-version/model nullable row for every player in every retained snapshot in the same transaction as the definition mutation. Any failure preserves the old definition and old scores.
- Ingest calculates the new snapshot's rows after players and role scores inside the existing transaction. It uses dump or stored player JSON and the same pure scorer. A missing definition adds no work; a score failure rolls back the whole ingest.
- Successful boost reconciliation recalculates and upserts the exact snapshot/player row in the SQLite source-data transaction instead of invalidating it. A failure rolls back the stored player, current role-score, and Club DNA changes together, maps to `PlayerBoostError::SnapshotSync`, and requires Load Data recovery because FM can already have changed.
- Definition removal deletes the definition and save-owned scores atomically. Snapshot/player cascade ownership remains unchanged. Historical snapshot promotion performs no Club DNA work because retained snapshots already have exact rows.
- Fixed `club_dna` display/sort uses a request-bound scalar persisted-score expression. Filters require an exact bound identity and non-null score. Club DNA sort deliberately wraps the scalar with `score IS NULL ASC`, then the requested score direction, then UID ascending. Missing definition or row yields null; filters are false; both sort directions retain all rows null-last with UID stability. Current-role and potential-role ordering remain unchanged.
- Bind snapshot, definition version, score-model version, filter values, limits, and offsets. Interpolate only the fixed metric identifier and validated sort direction. Preserve requested-field, filter-rule, limit, offset, and page bounds.
- Delete the lazy batch materializer, cache invalidation, completeness instrumentation, and obsolete tests together. Preserve pure scorer, v32 schema/version/cascade tests, nullable scoring, and independent role/potential-role tests.
- Remove the unique 2,000-player and 184,000-player Club DNA performance gates. Retain the final measurements as decision evidence. Use the current-role score validation portfolio and do not optimize general role-score sorting in this feature.
- Keep `club_dna` permanently valid in both catalogs. Let the backend report create versus replace, consolidate frontend catalogs in the existing frontend packet, and keep all explanation inside the Modal.

### Unknowns

- Native Tauri/WebView density and focus behavior for selecting the full catalog cannot be proved in headless Chromium.
- The exact synchronous definition-save duration across a user's retained snapshots is not an acceptance gate. Implementation must expose failures atomically and record that cost scales with retained history.

### Risks

- Definition create/edit can leave a new definition with missing, old-version, or partial historical rows if deletion, scoring, and persistence do not share one transaction. Retained-history and injected-failure tests must prove full rollback to the old definition and scores.
- Definition save is synchronous across all retained snapshots. Implementation must avoid unbounded extra copies or per-player definition queries and must report the scaling cost without introducing background infrastructure.
- Ingest can commit a snapshot without Club DNA rows or calculate from data that differs from stored player JSON. Present-definition, absent-definition, retained-history, nullable, and injected-write-failure tests must prove the exact transaction boundary and scorer inputs.
- Boost reconciliation can leave SQLite source data and scores inconsistent if it invalidates instead of recalculating, reads pre-update JSON, or commits one side after a failure. Current-ability and mentality paths need exact-row, local rollback, `PlayerBoostError::SnapshotSync`, and Load Data recovery proof without claiming that SQLite can reverse an FM change that already occurred.
- A query that omits definition or model identity can expose stale v32 rows. Resolver, filter, Search, and Squad tests must prove exact bound identity for display, filter, and sort.
- A scalar nullable expression can drop players or place nulls incorrectly if query joins become mandatory or if Club DNA copies the current role scalar ordering without a null wrapper. Ascending and descending Search and Squad tests must include present, computed-null, missing, and stale rows, plus an all-null missing-definition case that proves UID order, totals, and pages.
- Dynamic SQL can weaken the trust boundary if request values become identifiers. Only the fixed metric ID and validated sort direction may be interpolated; all identities, filter values, and bounds stay bound.
- Removing lazy behavior can leave orphan helpers, obsolete tests, or hidden materializer calls. The backend packet must delete the materializer, invalidation, completeness instrumentation, and their tests together, then prove no query triggers Club DNA score writes.
- The same app-local layouts serve every save. A save with no definition must return null for display/sort and false for filters without removing the fixed metric or saved state.
- Definition IPC calls, Query keys, route effects, and Rust checks must retain the exact save ID/context-token contract. Automatic append must still distinguish create, edit, delete, and re-create.

## Walking skeleton

Completed Commits 1–7 establish v31/v32 persistence, the pure scorer, and the measured decision history. Active Commit 8 records this architecture replan and superseding ADR before implementation. Commit 9 installs the complete eager writer lifecycle and removes lazy ownership without exposing Club DNA through queries. Commit 10 then exposes persisted Club DNA through read-only resolver, filter, Search, and Squad paths. Commits 11–13 deliver the unchanged frontend metric foundation, definition Modal, and My Club integration.

## Delivery plan

### PR 1 — Add user-defined Club DNA scoring

**Status:** Ready for publication

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

#### Commit 8 — Record the eager score architecture replan

**Status:** Completed

**Provisional commit:** `docs(club-dna): record eager score replan`

**Work:** Commit the independently reviewed ledger and ADR correction after the final indexed lazy-query attempt breached the representative Squad sort gate and the developer chose eager persisted scores.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, migrations, indexes, dependencies, TODO, BACKLOG, current-state architecture, design documentation, and unrelated decisions.

**Implementation packet:**

- Preserve completed Commits 1–7 and their immutable refs. Record the measured final breach, the developer-approved eager lifecycle/query authority, ADR-0023 supersession, ADR-0024, replacement backend packet, renumbered unchanged frontend packets, invalidated prior fingerprint, and `Pending review`. Do not implement or record a replacement Delivery fingerprint in this planning pass.

**Files and responsibilities:**

- `.wiki/features/active/club-dna.md` — record the bounded replan, final measurements, immutable completed refs, eager architecture, trunk-safe replacement Commits 9–10, renumbered frontend packets, discoveries, risks, and validation contract.
- `.wiki/decisions/0023-lazy-club-dna-score-cache.md` — mark the lazy decision Superseded with a concise pointer and reason while preserving its historical context, decision, alternatives, evidence, and consequences.
- `.wiki/decisions/0024-eager-persisted-club-dna-scores.md` — record the approved eager lifecycle, persisted query model, alternatives, transaction boundaries, synchronous scaling cost, and consequences.
- `.wiki/decisions/README.md` — index ADR-0024 and mark ADR-0023 as superseded.
- `.wiki/TODO.md` and `.wiki/BACKLOG.md` — deliberately unchanged because Club DNA remains active and no scope moves between planned and deferred ownership.

**Behavior and data flow:**

- Replace the disproved lazy-query authority with one reviewed planning record. ADR-0023 remains as history but points to accepted ADR-0024. The ledger keeps one PR, the same branch/base/publication fields, immutable completed history, and no implementation authority while the Delivery fingerprint is pending.

**Ordered implementation steps:**

1. Verify branch `feature/club-dna`, clean starting HEAD `2c6943f68fdfaf9311977f024a0d59192a4aed58`, and immutable completed refs 1–7 without changing Git state.
2. Confirm the discarded Commit 8 worktree is absent and only the ledger, ADR-0023, ADR-0024, and decisions README changed.
3. Record the exact final 2,000- and 184,000-player p95 evidence, developer decision, eager lifecycle/query boundary, obsolete lazy removals, trunk-safe eager-writer and read-only-query backend packets, renumbered frontend packets, and invalidated `3fea871899c7615d5449aaddb1ac13d0395b5d8473fa146389c2f734cfb0ca18` fingerprint.
4. Run both classifiers, Markdown diff checks, exact path/status checks, and LSP diagnostics. `ledger_state.py` must report runnable; `delivery_state.py` may be invalid only because the fingerprint is `Pending review`.
5. Submit the exact four-path planning diff for independent plan review. Do not calculate or record the proposed replacement fingerprint.

**Tests and proof:**

- Not applicable — planning documents only. `ledger_state.py` proves schema 2, sole Active planning commit, completed evidence, and packet completeness. `delivery_state.py` proves that delivery remains unauthorized only because review and fingerprint acceptance are pending. `git diff --check`, exact four-path diff/status inspection, and Markdown LSP diagnostics prove format and path scope. No tests, fixtures, mocks, snapshots, helpers, compatibility paths, implementation, or executable configuration change.

**Patterns to verify:**

- `.wiki/features/active/README.md` schema 2 and prior completed Club DNA planning packets for immutable history.
- `.wiki/decisions/README.md` and ADR-0023 for ADR format, historical preservation, supersession, and links.
- Current role-score ingest and dynamic SQL evidence named in ADR-0024 and Commit 9.

**Constraints and non-goals:**

- Preserve completed Commit 7's packet text byte-for-byte and resolve only its separate Completed work Git ref.
- Preserve the exact one PR, branch, base, provider, template, title, merge method, required check, close-out state, and CI repair count.
- Do not edit TODO, BACKLOG, ARCHITECTURE, DESIGN, code, tests, scripts, configuration, migrations, or dependencies. Do not record a replacement fingerprint.

**Dependencies and sequencing:**

- Depends on the developer's explicit eager-persisted-score decision, clean HEAD `2c6943f68fdfaf9311977f024a0d59192a4aed58`, discarded Commit 8 work, and exact final measurements. Commit 9 requires independent review, developer acceptance, and a later recorded classifier fingerprint.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/club-dna.md`; `python3 /home/jonas/projects/PI_SETUP/scripts/delivery_state.py .wiki/features/active/club-dna.md .`; `git diff --check -- .wiki/features/active/club-dna.md .wiki/decisions/0023-lazy-club-dna-score-cache.md .wiki/decisions/0024-eager-persisted-club-dna-scores.md .wiki/decisions/README.md`; `git diff -- .wiki/features/active/club-dna.md .wiki/decisions/0023-lazy-club-dna-score-cache.md .wiki/decisions/0024-eager-persisted-club-dna-scores.md .wiki/decisions/README.md`; LSP diagnostics for all four changed Markdown paths; `git status --short`

**Stop conditions:** Stop on a branch/HEAD mismatch, missing completed ref, any changed path outside the four approved planning paths, any Commit 7 packet change, any classifier error beyond pending fingerprint authorization, an uncleared review, missing later developer acceptance, a changed PR authority field, or any implementation request before reacceptance.

**Review mandate:** Verify (1) exact final measurements and measured breach, (2) developer-approved eager lifecycle/query decision and synchronous retained-history consequence, (3) ADR-0023 historical preservation plus correct supersession, (4) ADR-0024 and the ledger agree on transactions, query ordering, partial boost outcomes, removals, and non-goals, (5) immutable completed refs and byte-for-byte Commit 7 packet, (6) exactly Commit 8 is Active and trunk-safe Commits 9–10 are execution-ready, (7) frontend packets change only by renumbering and dependency references to Commits 11–13, and (8) only the four approved planning paths changed while the fingerprint remains pending.

#### Commit 9 — Install the eager Club DNA writer lifecycle

**Status:** Completed

**Provisional commit:** `feat(club-dna): persist eager scores`

**Work:** Replace the Club DNA lazy materializer with the complete eager definition, ingest, boost, removal, and promotion-safe writer lifecycle without exposing Club DNA through resolver, filter, Search, or Squad reads.

**Size assessment:** Likely 220–300 changed non-test implementation lines. This may exceed the soft target because definition mutation, ingest, boost reconciliation, promotion proof, scorer ownership, and removal of every lazy writer/invalidation path form one persisted consistency boundary. Query exposure remains a separate trunk-safe Commit 10.

**Out of scope:**

- Resolver, filter, Search, or Squad Club DNA exposure; migration v33; v32 schema/index changes; dependencies; background work; frontend code; Moneyball; Player Profile; current-role or potential-role behavior; general role-score sort optimization; representative performance gates; and current-state documentation.

**Implementation packet:**

- Reuse v32 `club_dna_scores` as eager derived persistence. Refactor the existing pure scorer into transaction-safe score helpers; make definition set/edit, present-definition ingest, and present-definition boost reconciliation persist exact nullable rows; preserve atomic definition removal and cascade ownership; delete lazy materialization, invalidation, and completeness work; and prove promotion only exposes already-persisted exact rows. Leave all resolver, filter, Search, and Squad exposure for Commit 10.

**Files and responsibilities:**

- `src-tauri/src/features/club_dna/service.rs` — make set/edit validate context and definition, update version, delete prior save rows, calculate every retained snapshot/player, and persist exact nullable rows in one transaction; keep create/edit result; keep removal atomic; add retained-history and rollback tests.
- `src-tauri/src/features/player_metrics/club_dna.rs` — preserve `SCORE_MODEL_VERSION` and the pure scorer; replace `materialize_player_scores`, cache-miss loading, bounded batching, and invalidation with transaction-oriented definition lookup, snapshot/player scoring, and exact-row persistence/upsert helpers shared by definition, ingest, and boost owners; delete obsolete lazy tests.
- `src-tauri/src/features/snapshot/ingest.rs` — after player and role-score insertion, calculate/persist the new snapshot when the save has a definition; use validated dump or transaction-visible stored JSON; absent definition does no work; replace the existing no-prefill test with present/absent, nullable, retained-history, and rollback proof.
- `src-tauri/src/features/snapshot/service.rs` — replace the lazy-era promotion assertion with exact seeded current and retained-to-be-promoted score rows; delete the current snapshot and prove its rows cascade while the promoted snapshot's exact score values and definition/model identity remain unchanged and no backfill occurs.
- `src-tauri/src/features/player/service.rs` — after the verified source update and current-role-score reconciliation, recompute/upsert the exact snapshot/player Club DNA row in the same SQLite transaction when a definition exists; remove invalidation calls; map eager Club DNA failures through the existing `PlayerBoostError::SnapshotSync` path; and rewrite invalidation/rollback tests around local transaction rollback plus Load Data recovery.
- `src-tauri/src/db/migrations.rs` — production migration registry remains unchanged; retain v32 identity, domain, index, upgrade, and cascade tests without adding v33.

**Behavior and data flow:**

- Set/edit starts after canonical definition and active-context validation. One transaction updates the definition with its new version, removes prior save rows, loads every player across retained snapshots, scores with the pure scorer, and inserts one nullable row per exact snapshot/player/version/model. A failure restores the prior definition and rows. Runtime and writes scale with retained snapshot/player count.
- Ingest uses its existing transaction and sequence: snapshot, players, staff, current role scores, then Club DNA when a definition exists. It reads the definition once and scores every new snapshot player from the dump or stored JSON through the shared scorer. Absence adds no scoring or writes. Any failure rolls back all ingest-owned rows.
- Boost reconciliation updates the stored player and current role scores, then reads the definition and recomputes/upserts that one transaction-visible player score when present. Failure rolls back those SQLite changes together and maps to `PlayerBoostError::SnapshotSync` so the existing command path requires Load Data recovery. Football Manager can already have changed before reconciliation, so this packet makes no false external rollback claim. Removal deletes definition/scores atomically; snapshot/player cascades remain unchanged.
- Promotion performs no score work. The promotion test seeds exact rows for both the current snapshot and the retained snapshot that will be promoted. Deleting current must cascade only its rows; promotion must preserve the retained exact score values and definition/model identity byte-for-byte and execute no backfill.

**Ordered implementation steps:**

1. Add RED definition transaction tests for create/edit across multiple retained snapshots, exact row count and identity, nullable players, version replacement, another save's isolation, synchronous full-history coverage, and injected scoring/write/commit rollback that preserves the old definition and old scores.
2. Add RED ingest tests that replace `ingest_does_not_prefill_club_dna_scores_when_a_definition_exists`: absent definition adds no rows; present definition writes one exact nullable row per new player; a second retained snapshot keeps both; invalid stored definition or injected score write rolls back the complete new snapshot while prior history remains.
3. Add RED boost tests for CA and mentality paths that prove same-SQLite-transaction exact-row recomputation/upsert from updated stored JSON, absent-definition no-work, local rollback of stored player/current role/Club DNA changes, `PlayerBoostError::SnapshotSync` mapping, and the existing Load Data recovery latch.
4. Replace the lazy promotion assertion with RED proof that seeds exact score rows for current and retained-to-be-promoted snapshots, deletes current, observes current-row cascade, and preserves the promoted row values and identity without a Club DNA write or backfill.
5. Refactor the pure scorer and exact persistence helpers without changing formula/domain semantics. Remove materializer batching, cache-miss loading, invalidation, completeness instrumentation, and obsolete tests/helpers in the same change.
6. Implement eager definition, ingest, and boost transaction calls in that order. Keep each definition read once per owning transaction and reuse transaction-visible player data.
7. Keep resolver/filter/Search/Squad Club DNA exposure absent, run the focused RED tests to GREEN, refactor only while focused proof stays green, then run Rust and full gates.

**Tests and proof:**

- RED must fail because current set/edit deletes rows without rebuilding them, ingest explicitly proves no prefill, boosts invalidate, and promotion only asserts that a lazy cache remains empty. The absence of resolver/filter/query exposure is retained and is not a RED condition for this commit.
- Definition proof: create/edit across at least two retained snapshots writes exactly one row for every player at the current definition/model identity, including null scores; edit removes stale rows; injected failures leave the previous definition/version/rows byte-for-byte equivalent; another save is untouched.
- Ingest proof: absent definition writes zero Club DNA rows; present definition writes every new player from authoritative transaction data after role scores; retained historical rows remain; one invalid selected value stores null; injected failure leaves no new snapshot/player/role/Club-DNA rows and preserves prior snapshots.
- Boost proof: both supported source-data paths update or retain the exact row from post-update JSON; absent definition performs no Club DNA statement; injected scoring/upsert failure rolls back stored player fields, current role scores, and score rows in SQLite; the command returns `PlayerBoostError::SnapshotSync` and latches Load Data recovery because FM may already have changed.
- Promotion proof: seed exact score rows for the current snapshot and the retained-to-be-promoted snapshot, including distinct values under the same exact definition/model identity. Delete current; prove its rows cascade, the promoted rows and identity remain unchanged, and no Club DNA insert/update/backfill occurs.
- Removal proof: delete obsolete `materialization_is_page_scoped_nullable_and_reuses_matching_versions`, `materialization_replaces_stale_versions_and_leaves_invalid_requests_empty`, `materialization_commits_bounded_batches_and_resumes_idempotently`, invalidation-specific service/player tests, the lazy-era empty-promotion assertion, and any completeness/performance helpers if present. Retain pure scorer tests, v32 migration/schema/version/index/cascade tests, role-score tests, potential-role tests, ingest fixtures, and boost recovery helpers because they protect current supported contracts.

**Patterns to verify:**

- `snapshot::ingest::{ingest_dump_json_for_save,insert_role_scores}` for the existing one-transaction ingest sequence and role-score analogue.
- `player::service::{reconcile_verified_boost,reconcile_mentality,replace_role_scores}` for source-data transaction visibility and rollback.
- `player::commands::{execute_player_boost_with,request_and_reconcile_player_boost}` and `PlayerBoostError::SnapshotSync` for post-FM partial outcomes and Load Data recovery.
- `snapshot::service::delete_snapshot` and its promotion/cascade tests for no-backfill promotion behavior.
- Migration v32 tests and `club_dna::service::{set_club_dna,remove_club_dna}` for exact identity, save ownership, versioning, and existing transaction boundaries.

**Constraints and non-goals:**

- Do not change v31/v32, add v33, add an index/dependency, or weaken the nullable 0–100 domain and exact identity.
- Do not retain query-time materialization, invalidation, completeness checks, a performance gate, or background infrastructure. Do not optimize general role-score sort SQL.
- Keep Rust/SQLite authoritative. React supplies no score, SQL, JSON path, definition version, or model version.
- Preserve resolver, filter, Search, Squad, requested-field, Moneyball, current-role, and potential-role contracts without exposing `club_dna` in this commit.

**Dependencies and sequencing:**

- Depends on completed v31/v32/pure scorer work through Commit 4, immutable planning history through Commit 7, reviewed Commit 8, ADR-0024, and a developer-accepted replacement Delivery fingerprint. Commit 10 may expose the fixed metric only after the complete eager writer lifecycle, local rollback/recovery mapping, lazy removal, and promotion proof pass.

**Validation:** `./scripts/dev check-rust`; `./scripts/dev check`

**Stop conditions:** Stop if v32 cannot represent eager rows without a migration; definition set/edit cannot atomically preserve the old definition and scores on any failure; ingest cannot calculate inside its existing transaction after role scores; authoritative score inputs differ between definition, ingest, and boost paths; boost recomputation cannot share SQLite source-data rollback or map failure to `PlayerBoostError::SnapshotSync` and Load Data recovery; promotion changes retained exact rows or needs backfill; lazy removal would break potential-role behavior; or implementation needs query exposure, background work, another index/dependency, a new performance contract, or general role-sort optimization.

**Review mandate:** Verify (1) definition create/edit covers every retained snapshot atomically and records synchronous scaling cost, (2) ingest absent/present and retained-history behavior uses authoritative data after role scores with whole-ingest rollback, (3) CA and mentality boost paths recompute exact rows inside the SQLite source transaction, roll back local stored player/current role/Club DNA changes together, and map failure to SnapshotSync/Load Data without claiming FM rollback, (4) nullable formula/domain and exact version/model identity remain intact, (5) removal deletes all lazy materializer/invalidation/completeness code and obsolete tests without harming pure scorer/schema tests, (6) promotion seeds both snapshots, cascades current rows, preserves promoted exact values/identity, and performs no backfill, (7) resolver/filter/Search/Squad exposure remains absent while no unique performance gate remains, and (8) no v33, dependency, background work, frontend scope, current/potential-role change, or general role-sort optimization appears.

#### Commit 10 — Expose persisted Club DNA queries

**Status:** Completed

**Provisional commit:** `feat(club-dna): expose persisted scores`

**Work:** Expose fixed `club_dna` through resolver, Search filtering, Search display/sort, and Squad display/sort as read-only exact-identity queries after Commit 9 installs every eager writer.

**Size assessment:** About 120–190 changed non-test implementation lines. Within the soft target; focused Search and Squad tests are excluded from the count.

**Out of scope:**

- Definition, ingest, boost, removal, promotion, scorer, migration, or score-writer changes; lazy materialization; frontend code; Moneyball; Player Profile; current-role or potential-role ordering changes; general role-score optimization; indexes; dependencies; and performance gates.

**Implementation packet:**

- Add one fixed nullable persisted metric. Resolve the active snapshot's current definition/model identity once per validated request, bind it into scalar display and filter expressions, and use a Club DNA-specific null-last sort wrapper. Keep every query read-only and preserve all existing request and page bounds.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/resolver.rs` — add only fixed `club_dna` as a nullable integer metric whose scalar reads one exact persisted row using caller-bound snapshot, definition version, and score-model version; do not change current-role or potential-role expressions or ordering.
- `src-tauri/src/features/search/filter.rs` — accept fixed `club_dna` and compile every integer operator through an exact bound persisted-row `EXISTS` expression with `score IS NOT NULL`; preserve flat AND/OR behavior and the 32-rule bound.
- `src-tauri/src/features/search/query.rs` — resolve current definition/model identity once per validated request; bind it through display, filter, count, page, and select paths; use `score IS NULL ASC`, then the validated score direction, then `players.uid ASC`; preserve missing-definition/missing-row semantics, totals, offsets, limits, and zero score-table mutations.
- `src-tauri/src/features/planner/squad.rs` and `src-tauri/src/features/planner/squad_tests.rs` — bind the same exact identity for display and sorting over the exact managed-club cohort; use `score IS NULL ASC`, then the validated score direction, then `p.uid ASC`; prove exact membership, bounded pages, and no score-table writes.

**Behavior and data flow:**

- After request validation, Search or Squad reads the active snapshot and its current save-owned definition identity once. A fixed scalar expression selects only the score row whose snapshot, UID, definition version, and Rust-owned model version all match. A computed-null exact row, missing exact row, stale-only row, or missing definition yields null for display and cannot match any Club DNA filter, including `neq`.
- Club DNA sort deliberately diverges from current-role and potential-role ordering only in the wrapper. Both ascending and descending use `score IS NULL ASC`, then score in the validated requested direction, then UID ascending. Present values therefore sort by direction, all unavailable forms stay last, and an all-null missing-definition cohort sorts by UID. Current-role and potential-role ordering remain byte-for-byte unchanged.
- Search preserves its flat filter AST: a missing or null Club DNA row cannot satisfy an isolated or AND rule, while another rule can still satisfy a mixed OR request. Search and Squad preserve count/page/select bounds and never call a materializer or execute an insert, update, delete, or upsert.

**Ordered implementation steps:**

1. Add RED resolver/filter tests for fixed-ID acceptance, unsafe or unknown rejection, nullable integer decoding, exact bound snapshot/definition/model identity, every integer operator including `neq`, and unchanged current-role and potential-role expressions.
2. Add RED Search tests for display, filter, filter-plus-sort, totals, offsets, and limits. For both ascending and descending sorts, seed present, computed-null exact, missing exact, and stale-only rows. Add a missing-definition all-null case that requires UID order.
3. Add the equivalent RED Squad ascending and descending tests over the exact managed-club cohort, including a nonmember, present, computed-null exact, missing exact, stale-only, and missing-definition all-null UID order.
4. Record the complete `club_dna_scores` row set before each read-only integration request and assert it is unchanged afterward; ensure no test helper invokes materialization or eager writers during the request.
5. Implement the fixed resolver and filter expressions with bound values. Add request-scoped definition/model identity without accepting versions from the WebView.
6. Implement the Club DNA-specific Search and Squad sort wrapper. Do not alter the general metric wrapper or current-role/potential-role behavior.
7. Run focused tests to GREEN, refactor only while they stay green, then run Rust and full gates.

**Tests and proof:**

- RED: `club_dna` is currently rejected by resolver/filter/query paths. The ordering tests also fail if implementation copies the current role scalar order, drops unavailable players, treats stale rows as current, or applies UID before null classification.
- Identity/filter proof: exact current rows display and filter; computed-null, missing, stale-definition, and stale-model rows display null and fail every Club DNA filter including `neq`; unsafe IDs and client-supplied identity remain rejected; mixed AND/OR semantics and the 32-rule bound remain unchanged.
- Ordering proof: separate ascending and descending Search tests and separate ascending and descending Squad tests each include present scores, computed-null exact rows, missing exact rows, and stale-only rows. Every case keeps unavailable players last and uses UID ascending within equal-score and unavailable ties. A missing-definition case for each surface makes the complete cohort null and proves UID order.
- Read-only proof: snapshot the exact `club_dna_scores` rows before display, filter, filter-plus-sort, ascending sort, and descending sort requests and prove byte-for-byte equality afterward. No query invokes materialization, completeness checks, cohort scoring, or any writer helper.
- Add/modify: resolver tests, Search filter/unit and query integration tests, and Squad integration tests. Deliberately retain current-role and potential-role display/filter/sort tests, bounds tests, managed-club membership fixtures, and v32 identity tests because they protect unchanged adjacent contracts. Delete no writer, migration, scorer, boost, ingest, promotion, frontend, fixture, mock, snapshot, or compatibility asset in this commit.

**Patterns to verify:**

- `MetricSource::CurrentRole` and `compile_role_score_rule` for the persisted scalar and non-null `EXISTS` read model, but not as evidence of null-last ordering.
- `staff/query.rs` for the explicit `expression IS NULL ASC`, directional expression, and UID tie-break shape.
- `search_players_in_view` and `list_squad_players` for bounded count/page/select flow and trusted sort-direction interpolation.
- Commit 9's eager writer helpers only as persistence context; query code must not call them.

**Constraints and non-goals:**

- Bind snapshot, definition version, score-model version, filter values, limits, and offsets. Interpolate only the fixed metric identifier and validated sort direction.
- Keep current-role and potential-role ordering unchanged. Do not generalize the Club DNA null wrapper or optimize role sorting.
- Do not change v31/v32, writer lifecycle, formula, score domain, or exact row identity. Do not add query-time writes, materialization, completeness work, an index, dependency, or performance gate.
- Preserve requested-field, filter-rule, limit, offset, page, Moneyball, and managed-club membership contracts.

**Dependencies and sequencing:**

- Depends on Commit 9's complete eager writer lifecycle and removal of lazy ownership. Commit 11 may register the fixed frontend metric only after this read-only backend exposure clears review.

**Validation:** `./scripts/dev check-rust`; `./scripts/dev check`

**Stop conditions:** Stop if requests cannot resolve one exact definition/model identity without client-supplied versions; either query needs a write/materializer/completeness path; the Club DNA null-last wrapper requires changing current-role or potential-role ordering; ascending or descending cannot retain present, computed-null, missing, and stale rows with deterministic UID ties; missing definition cannot produce all-null UID order; bounds or managed-club membership change; or implementation needs v33, an index/dependency, background work, or a performance contract.

**Review mandate:** Verify (1) exact snapshot/definition/model identity is bound and stale rows are ignored, (2) every Club DNA filter including `neq` requires a non-null exact row and preserves flat AND/OR semantics, (3) ascending and descending Search tests cover present, computed-null, missing, and stale rows plus all-null missing-definition UID order, (4) Squad has the equivalent directional and cohort proof, (5) the wrapper is exactly null flag, score direction, then UID, (6) current-role and potential-role ordering remain unchanged and general role optimization is absent, (7) every read leaves score rows unchanged and invokes no lazy/eager writer, and (8) page, filter, trust, Moneyball, and membership bounds remain intact.

#### Commit 11 — Add the frontend Club DNA domain and fixed metric

**Status:** Completed

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

- Frontend callers receive an explicit context from the established saves query and can fetch that context's nullable definition or invoke set/remove with the same save ID and immutable token. The definition Query key contains both values, so save A data cannot satisfy save B or a replacement save incarnation. Adapters return results to their caller but do not infer currentness; Commits 12–13 UI compares the captured context before applying results, while Rust rejects a context that is no longer active. The shared catalog exposes canonical IDs and FM grouping but no score function. The fixed `club_dna` metadata flows through the existing picker, filter registry, sort validation, requested-field adapter, nullable dynamic DTO, and table cell. Search and Squad render a backend-supplied integer with `ScoreBadge`; null stays `—`.

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

- Depends on Commit 10 so every exposed metric request is already backed by the complete eager writer lifecycle and read-only persisted query behavior in Rust. The later Modal and route packets consume these adapters and shared groups.

**Validation:** `./scripts/dev test src/utils/player-attributes.test.ts src/features/player-profile/utils/attribute-groups.test.ts src/utils/player-metrics.test.ts src/features/club-dna/api/club-dna-api.test.ts src/features/search/utils/dynamic-columns.test.ts src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx`; `./scripts/dev check-app`; `./scripts/dev check`

**Stop conditions:** Stop if catalog consolidation changes a current Player Profile group, if a supported frontend ID has no exact Rust counterpart, if Query/API adapters cannot bind get/set/remove to `{ saveId, contextToken }`, if the fixed metric requires a new table component or store owner, if store retention requires resetting user layouts, or if React would need to compute a score.

**Review mandate:** Verify (1) exact catalog parity and no cross-feature import, (2) Player Profile grouping stays unchanged, (3) fixed label/ID and integer filter metadata, (4) no default layout insertion and persisted layout retention, (5) ScoreBadge/null presentation in both tables, (6) context-bearing definition key isolates save IDs and tokens, (7) all typed invoke requests carry the exact expected context, and (8) no frontend score, stale-context authority, or validation duplicates Rust.

#### Commit 12 — Build the Club DNA definition Modal

**Status:** Completed

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

- Depends on Commit 11's typed API and shared catalog. Commit 13 provides route placement and cross-query/layout effects.

**Validation:** `./scripts/dev test src/features/club-dna/components/club-dna-definition.test.tsx`; `./scripts/dev check-app`; `./scripts/dev check`

**Stop conditions:** Stop if the shared Modal cannot support keyboard access, full-catalog scrolling, the one-Modal edit ↔ confirmation transition, pending dismissal guards, or reliable focus return; if a context change cannot close/discard the draft and suppress late prior-context results; if product behavior requires an explanation outside the Modal; or if component state cannot distinguish backend create from edit.

**Review mandate:** Verify (1) full approved catalog and no maximum, (2) minimum-one/current-context guards plus Rust stale-context authority, (3) selected summary and exact formula copy only in Modal, (4) context change closes/discards and late get/set/remove results cannot update current UI, (5) the Planner-style single Modal preserves draft across confirmation and implements exact Cancel/Escape/pending/error transitions, (6) keyboard/focus/accessibility behavior including successful-remove focus return, (7) no score computation or profile surface, and (8) deferred mocks test observable stale-result suppression without duplicating Rust mutation authority.

#### Commit 13 — Integrate Club DNA with My Club and layouts

**Status:** Completed

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

- Depends on Commits 2–12. It is the final implementation packet and moves the feature to Validation only after its full checkpoint clears.

**Validation:** `./scripts/dev test src/features/club-dna/components/club-dna-definition.test.tsx src/app/routes/my-club-squad.test.tsx src/app/app-top-bar.test.tsx src/app/routes/settings.test.tsx src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx`; `./scripts/dev test`; `./scripts/dev check`; `./scripts/dev smoke`

**Stop conditions:** Stop if placement requires a cross-feature import, if My Club cannot derive and key the feature by the saves query's active ID/token, if create-versus-edit cannot be based on the backend result, if a save change can leave a draft open or allow prior-context data/feedback/layout append, if managed-club or Club DNA refresh/mismatch/error cannot disable interaction, if remove requires deleting catalog/layout/URL state, if Rust cannot remain the stale-context rejection authority, if smoke needs score computation in the stub, or if native-only behavior becomes required for acceptance without an available native environment.

**Review mandate:** Verify (1) exact placement and disabled state covers managed-club plus context refresh/mismatch/error, (2) route-only composition derives the active ID/token and key/remounts by both, (3) deferred save-switch tests prove no stale render, feedback, invalidation, or create-only layout append during open/set/remove paths, (4) Rust remains authoritative for stale get/set/remove rejection, (5) edit/remove/re-create and saved layout/filter/sort/history retention remain exact, (6) AppTopBar and Settings production invalidations and named tests include Club DNA, (7) proportionate route/component/smoke tests use context-bearing mocks without computing scores, and (8) no Moneyball/Profile/cache or frontend score scope creep.

## Active work

**PR:** PR 1 — Add user-defined Club DNA scoring

**Commit:** None — feature validation

### RED or removal proof

Not applicable. All planned implementation packets are completed and independently reviewed. Feature validation now proves the complete supported contract across Rust, frontend, Chromium, documentation, and the exact recorded commit set.

### Expected outcome

All ledger validation passes, a fresh feature reviewer clears the complete implementation and test portfolio, durable documentation is reconciled, and the ledger is ready to move to completed features before publication.

### Explicit exclusions

New behavior, scope expansion, release preparation, unreviewed corrections, and unrelated repository changes.

## Discoveries and replanning

- The original direct read-time SQL plan was disproved by complete-catalog 2,000-player measurements and replaced by ADR-0023's lazy cache. Commits 3–7 preserve the immutable cache, indexed-query, completeness, and threshold planning history.
- Commit 7 is complete at clean HEAD `2c6943f68fdfaf9311977f024a0d59192a4aed58`. The active Commit 8 indexed lazy-query attempt was discarded cleanly.
- The discarded final 2,000-player warm p95 values were Search filter 4.129 ms, Search sort 6.527 ms, and Squad sort 11.339 ms.
- The discarded final 184,000-player warm p95 values were Search filter 514.287 ms, Search sort 611.003 ms, and Squad sort 1596.226 ms. Squad sort breached the accepted one-second representative gate.
- The developer asked why one Club DNA score was not calculated on save like about 70 role scores on ingest, then explicitly chose eager persisted scores: calculate and persist Club DNA through the role-like writer lifecycle, reuse the persisted scalar/`EXISTS` read model, add a Club DNA-only null-last sort wrapper, and remove the unique Club DNA 184,000-player latency gate.
- Repository evidence confirms current role scores are inserted during ingest, resolve through stored-row scalar dynamic SQL for display/sort, filter through a bound non-null `EXISTS` query, and have no representative latency gate. The current scalar sort pattern does not guarantee null-last in both directions, so it is not the Club DNA ordering analogue.
- ADR-0024 supersedes ADR-0023's lazy lifecycle. It reuses v32 and requires atomic definition rescoring across retained snapshots, present-definition ingest scoring, successful boost recomputation, atomic removal, and no promotion work.
- Commit 9 installs the complete eager writer lifecycle, removes lazy materialization/invalidation/completeness ownership, and adds exact no-backfill promotion and SnapshotSync recovery proof without resolver/filter/Search/Squad exposure. Commit 10 then adds exact-identity read-only exposure and a Club DNA-only null-last sort wrapper. Both add no v33, index, dependency, background work, performance gate, current/potential-role ordering change, or general role-sort optimization.
- The architecture and packet change invalidated Delivery fingerprint `3fea871899c7615d5449aaddb1ac13d0395b5d8473fa146389c2f734cfb0ca18`. Independent review cleared the eager replan, the developer accepted it and re-invoked delivery, and both classifiers report runnable under replacement fingerprint `f3b0d9469bdad80e388af3cdb915383af95005bcc7abc3777059f44aecfee49b`.
- The frontend attribute catalogs remain duplicated between Player Profile grouping and player metric metadata. Renumbered Commit 11 still consolidates them into a shared utility; Rust retains its independent authoritative catalog. Commits 11–13 otherwise retain their substantive content.
- `.wiki/TODO.md`, `.wiki/BACKLOG.md`, `.wiki/ARCHITECTURE.md`, and `.wiki/DESIGN.md` remained unchanged during the bounded replan.
- Feature review cleared the complete implementation after two correction rounds covering immutable token proof, explicit UID ties, eager definition-query availability, and same-context refetch-error guards. The developer explicitly accepted the unavailable native Tauri/WebView validation gap after the supported Rust, frontend, full repository, and Chromium gates passed.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Add user-defined Club DNA scoring | Commit 1 — Record the approved feature plan | ddd4961e6d90ca24faa435955c6ae7eb5a716f0b | Recorded the reviewed schema 2 ledger and TODO activation. | `ledger_state.py`: runnable; `git diff --cached --check`: passed. | Not applicable | Clear | 0 | None. |
| PR 1 — Add user-defined Club DNA scoring | Commit 2 — Persist one save-owned Club DNA definition | d2682ee5c50cb99cd0b7f9facf5fd4f9060d5001 | Added migration v31 and context-bound Rust CRUD for one validated definition per save. | RED failed because v31 was absent; `./scripts/dev check-rust` passed 561 tests with 2 ignored; `./scripts/dev check` passed. | Pass | Clear | 0 | None. |
| PR 1 — Add user-defined Club DNA scoring | Commit 3 — Record the approved cache replan | 7cf5e5924af8a9c54852f5037e17ffe4b2c58cc0 | Recorded ADR-0023 and the reviewed lazy-cache packets after measured direct-SQL failure. | Both classifiers were runnable with the accepted fingerprint; staged diff and Markdown checks passed. | Not applicable | Clear | 0 | Replaced the direct-SQL packet after measured 2,000-player threshold breaches. |
| PR 1 — Add user-defined Club DNA scoring | Commit 4 — Add the versioned Club DNA score cache | d78f97f25497409f6c895a8ac5cdeb74ea5301eb | Added v32 definition versioning, pure scoring, bounded nullable materialization, and atomic definition and player-boost invalidation. | RED failed because the cache owner was absent; focused rollback tests passed; `./scripts/dev check-rust` and `./scripts/dev check` passed 569 tests with 2 ignored. | Pass | Clear | 2 | Review corrections removed repeated definition validation, strengthened batch and late invalidation rollback proof, and restored adjacent role-score rollback coverage. |
| PR 1 — Add user-defined Club DNA scoring | Commit 5 — Record the indexed query-shape correction | df074b9da78bec038960e2be7c851dcb5879dbdd | Recorded the representative correlated-query breach, indexed cache-first correction, and ADR implementation status. | Both classifiers were runnable with the accepted fingerprint; staged diff and Markdown checks passed. | Not applicable | Clear | 0 | Replaced the correlated cache-read packet after the 184,000-player warm threshold breach. |
| PR 1 — Add user-defined Club DNA scoring | Commit 6 — Record the warm-cache completeness correction | 8250dbe9aac7853ed90ba674f83a67da870a8ecb | Recorded the representative warm materializer overhead and exact-count completeness fast path. | Both classifiers were runnable with the accepted fingerprint; staged diff and Markdown checks passed. | Not applicable | Clear | 0 | Replaced warm full-cohort UID enumeration and batch probes after all 184,000-player shapes breached. |
| PR 1 — Add user-defined Club DNA scoring | Commit 7 — Record the representative latency decision | 2c6943f68fdfaf9311977f024a0d59192a4aed58 | Recorded the developer-chosen one-second representative threshold and rejected v33 index alternative. | Both classifiers were runnable with the accepted fingerprint; staged diff and Markdown checks passed. | Not applicable | Clear | 0 | Replaced the former 200 ms representative gate after exact v32/count-fast-path measurements. |
| PR 1 — Add user-defined Club DNA scoring | Commit 8 — Record the eager score architecture replan | aa5e097c66692bab1aac4444a4df0b02878b7994 | Recorded ADR-0024, superseded the lazy decision, and split eager writers from persisted reads. | Both classifiers were runnable with the accepted fingerprint; staged diff and Markdown checks passed. | Not applicable | Clear | 1 | Replaced the breached lazy architecture after the developer chose role-like eager persistence; corrected the post-acceptance authority narrative. |
| PR 1 — Add user-defined Club DNA scoring | Commit 9 — Install the eager Club DNA writer lifecycle | b3ccf37f9a1b23b334d235605b91db681d57bb13 | Added bounded eager definition, ingest, and boost score writers; removed lazy ownership; and proved promotion needs no backfill. | `./scripts/dev check-rust` passed 574 tests with 2 ignored; `./scripts/dev check` passed. | Pass | Clear | 2 | Review corrections bounded retained-snapshot memory and added exact CA and mentality rollback proof. |
| PR 1 — Add user-defined Club DNA scoring | Commit 10 — Expose persisted Club DNA queries | cf7cc027c6a7058f10635c9e8ee7cd69cb6ba0c1 | Added exact-identity read-only Club DNA display/filter/sort in Search and display/sort in Squad. | `./scripts/dev check-rust` passed 581 tests with 2 ignored; `./scripts/dev check` passed. | Pass | Clear | 0 | None. |
| PR 1 — Add user-defined Club DNA scoring | Commit 11 — Add the frontend Club DNA domain and fixed metric | b7c947efedacf27dcda27478d2c0b744e4a4210b | Added the canonical frontend attribute catalog, typed context adapters, fixed metric, layout retention, and score rendering. | `./scripts/dev test` passed 577 tests; `./scripts/dev check-app` and `./scripts/dev check` passed. | Pass | Clear | 1 | Review corrections anchored full catalog order and exact save/token Query-key isolation. |
| PR 1 — Add user-defined Club DNA scoring | Commit 12 — Build the Club DNA definition Modal | c78fc85062d547b923572b6fae6edbaaf4e3ae56 | Added the full-catalog accessible definition Modal, edit/remove lifecycle, context-safe pending/error handling, and bounded IPC mocks. | `./scripts/dev test` passed 591 tests; `./scripts/dev check-app` and `./scripts/dev check` passed. | Pass | Clear | 1 | Review corrections strengthened stale success/error isolation, unavailable confirmation guards, and edit `created: false` proof. |
| PR 1 — Add user-defined Club DNA scoring | Commit 13 — Integrate Club DNA with My Club and layouts | 45b4bc60727a605367749051cf35b50a6909c222 | Composed Define DNA beside managed-club controls, guarded save-context side effects, appended both layouts on creation, wired invalidation, and added smoke coverage. | `./scripts/dev test` passed 598 tests; `./scripts/dev check-app`, `./scripts/dev check`, and `./scripts/dev smoke` passed 49 Chromium tests. | Pass | Clear | 2 | Review corrections closed the save-refresh race, proved mutation invalidation, kept the disabled action visible, widened the responsive control group, and strengthened removal retention proof. |

## Final validation

- `./scripts/dev check-rust` — Rust format, Clippy, and all Rust tests pass for the complete eager writer lifecycle, removal of lazy behavior, exact no-backfill promotion, SnapshotSync/Load Data boost recovery, and read-only persisted queries.
- `./scripts/dev test` — all frontend component, route, store, catalog, adapter, and IPC-mock tests pass and discover the new Club DNA tests.
- `./scripts/dev check` — Biome, TypeScript, full-tree secretlint, Rust format, Clippy, and all Rust tests pass, including v32 characterization, definition/version eager rescoring, pure scoring, ingest absent/present behavior, retained history, local boost rollback and recovery mapping, promotion cascade/preservation, Search/filter/Squad exact identity, and nullable-score guards.
- `./scripts/dev smoke` — Chromium proves the My Club create flow, Modal explanation/selection, and fixed table-column integration through the browser IPC stub.
- Inspect the exact feature diff with `git diff --check b573420893da93d91ddaee66ff9a4038f800b6d9...HEAD` and the delivery workflow's exact recorded commit set.
- Inspect focused Rust tests and the exact feature diff to confirm definition create/edit eagerly covers every retained snapshot, ingest does no work without a definition and writes every new player with one, and definition/ingest failures remain fully atomic.
- Inspect `src-tauri/src/features/player/service.rs` and command recovery tests to confirm boost reconciliation upserts rather than invalidates; eager Club DNA failure rolls back stored player/current role/Club DNA changes in SQLite; the result maps to `PlayerBoostError::SnapshotSync` and latches Load Data recovery; and no test claims that this reverses an FM change that already occurred.
- Inspect `src-tauri/src/features/snapshot/service.rs` promotion proof to confirm exact rows are seeded for current and retained-to-be-promoted snapshots, deleting current cascades only its rows, promoted exact values and definition/model identity remain unchanged, and no backfill occurs.
- Inspect focused Search and Squad tests to confirm persisted exact-version/model display and every integer filter including `neq`. Require ascending and descending tests on both surfaces with present, computed-null exact, missing exact, and stale-only rows; require all-null missing-definition UID order; and confirm exact managed-club membership, totals, offsets, limits, and no query-time Club DNA writes.
- Inspect the exact backend diff to confirm Commit 9 removes lazy materializer, cache-miss loading, invalidation, completeness instrumentation, and their obsolete tests before Commit 10 adds read-only exposure. Confirm pure scorer and v32 migration/schema/version/index/cascade tests remain and current-role/potential-role ordering is unchanged.
- No Club DNA-specific 2,000-player or 184,000-player performance command remains in final validation. The measured breach is decision context, not a publication gate. General role-score optimization remains out of scope.
- Manually verify the native Modal at 1280×800 and 1600×900: full-catalog scrolling, keyboard selection, edit ↔ remove-confirmation transitions in one Modal, confirmation Cancel/Escape return, edit Cancel/Escape discard, pending-removal dismissal blocking, remove-error return path, successful-removal focus return, and no layout shift. Chromium does not replace this check.
- `./scripts/dev bridge-test` is outside the affected bridge path. `./scripts/dev mutate` remains unsupported. Neither may be reported as passed.

## Documentation impact

During this bounded replan, change `.wiki/features/active/club-dna.md`, supersede ADR-0023 with a concise pointer while preserving its historical text, create ADR-0024, and update `.wiki/decisions/README.md`. Leave `.wiki/TODO.md`, `.wiki/BACKLOG.md`, `.wiki/ARCHITECTURE.md`, and `.wiki/DESIGN.md` unchanged. During feature reconciliation after implementation, update `.wiki/ARCHITECTURE.md` for v31/v32 definition persistence, atomic eager definition/ingest/boost score lifecycle, persisted query behavior, and cascade owners; update `.wiki/DESIGN.md` for the implemented My Club action and definition Modal; update `.wiki/TODO.md` for completion; preserve both ADRs; and move this ledger to `.wiki/features/completed/club-dna.md`.
