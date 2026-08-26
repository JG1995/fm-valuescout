# Cumulative Moneyball Imports

## Status

Ready for final publication

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** de0065ba6fae4c9be8ab25b023a0a3962181a5b0a280fcc852f8977df34e6d9e

## Intent

Make each successful Moneyball import add or update matched players in one cumulative cohort for the effective current snapshot instead of replacing that snapshot's complete cohort. Add the same import path to the My Club Squad overview for Football Manager squad exports while preserving Search as an entry point.

## User-visible behavior

- Search keeps **Upload Moneyball CSV**. The My Club Squad overview adds a secondary **Upload Squad CSV** action.
- Both actions accept the existing bounded Moneyball CSV contract and import into the same cohort for the effective current snapshot.
- Each included player that matches the current snapshot replaces all of that player's Moneyball enrichment, including null values. Existing cohort players omitted from the new file remain unchanged, and newly matched players join the cohort.
- CSV rows whose UIDs are not in the effective current snapshot are skipped. An import never creates a player.
- A valid empty or zero-match import succeeds without changing existing Moneyball rows.
- Invalid, failed, or stale imports preserve the complete prior cohort.
- After a successful import that upserts at least one player, complete-cohort percentiles are recomputed and stored for the resulting snapshot cohort in the same transaction.
- The Squad upload trusts the Football Manager export. It uses only the established current-snapshot UID match and does not validate managed-club membership.
- Import copy describes cumulative add-or-update behavior and no longer says or implies that an upload replaces the whole cohort.
- Successful Squad uploads invalidate Search and Player Profile Moneyball queries. Squad gains no Moneyball columns as part of this feature.
- Feedback remains bound to the save and snapshot that opened the import, never displays a local path, remains keyboard accessible, and names the real import phase.

## Invariants

- Snapshot ownership remains `(snapshot_id, player_uid)`, and all product reads remain restricted to the effective current snapshot.
- Matching uses exact numeric player UID equality against the captured effective current snapshot. CSV data never creates players or overrides memory-owned player fields.
- One cumulative Moneyball cohort exists per effective current snapshot. The product does not model named imports, fixed source sets, source provenance, or import history.
- An included matched row is a full per-player replacement. Null imported values clear that player's prior values rather than inheriting them.
- Omitted current-snapshot players retain their prior Moneyball rows byte-for-byte on an empty or zero-match successful import and retain their raw enrichment on a non-empty import.
- A successful non-empty import persists percentiles calculated from the complete post-upsert cohort. Per metric, null and non-finite values are excluded, lower-is-better metrics are inverted, ties use the lower bound, and singleton or constant populations score 50.
- Full CSV Search continues to use persisted complete-cohort percentiles. Player Profile uses persisted rows only for scored-cohort readiness and membership, reads raw statistics, and recalculates metric percentiles and role scores at read time over natural-position peers.
- Context capture and transaction-time save/snapshot token revalidation remain mandatory. Invalid input, stale context, preparation failure, SQL failure, or percentile serialization failure commits no partial change.
- Existing 1 MiB and 1,000-player per-file limits, UTF-8 and regular-file checks, expected-format validation, safe error mapping, and native picker/drop constraints remain unchanged. The cumulative cohort is bounded only by current-snapshot membership and can exceed 1,000 players after repeated disjoint imports.
- React remains a presentation and Query-cache layer. Rust owns parsing, matching, cumulative persistence, percentile calculation, and SQLite transactions.

## Non-goals

- Clear, remove, rename, select, or manage Moneyball source sets.
- Import history, season identity, source-file provenance, duplicate-source detection, or managed-club membership validation.
- Historical Moneyball browsing or changes to snapshot selection.
- New Moneyball columns in Squad, new formulas, new metrics, new role definitions, or changes to Search filtering and natural-position calculations.
- Changes to Youth Tracker import behavior, Staff Shortlist imports, the bridge protocol, file limits, migrations, or database ownership.
- Removal of the Search upload entry point.

## Current-state map

- **Relevant components:** `src-tauri/src/features/csv_import/service.rs::prepare_moneyball_import`, `persist_csv_import`, and `insert_moneyball_stats` prepare percentiles from incoming matched rows, delete every Moneyball row for the captured snapshot, then insert only those prepared rows.
- **Percentile owner:** `src-tauri/src/features/moneyball/percentile.rs::calculate_percentiles` owns null/non-finite exclusion, lower-bound ties, lower-is-better inversion, and singleton/constant neutrality.
- **Data model:** migration v18 in `src-tauri/src/db/migrations.rs` defines `player_moneyball_stats` with primary key `(snapshot_id, player_uid)` plus cascading foreign keys to both the snapshot and exact snapshot player. Migration v30 adds nullable checked `percentiles_json`.
- **Persistence and migrations:** the existing primary key already supports one row per snapshot/player and SQLite conflict upsert. The statistics and percentile JSON columns already store complete per-player raw and derived state. No new identity, ownership, column, index, or compatibility state is required, so this feature plans no migration.
- **Existing behavioral assumptions:** Rust tests `successful_moneyball_reimport_replaces_the_captured_snapshot_cohort` and `moneyball_imports_follow_the_effective_current_snapshot_and_clear_only_its_cohort` assert destructive cohort replacement, including zero-match clearing. Rollback, stale-context, invalid-file, bounds, exact-match, and cascade tests protect surrounding contracts.
- **Consumers:** `src-tauri/src/features/moneyball/query.rs` uses persisted rows for scored-cohort readiness and membership, reads their raw statistics, and recalculates Player Profile metric percentiles and role scores over natural-position peers at read time. `src-tauri/src/features/search/query.rs` reads persisted percentiles for Full CSV and recomputes filtered-cohort percentiles at read time. Their query contracts do not need production changes.
- **Frontend import seam:** `src/features/csv-import/components/squad-csv-import-modal.tsx`, `src/features/csv-import/utils/use-csv-import.ts`, and `src/features/csv-import/api/import-csv.ts` already provide expected-format binding, native browse/drop, context generations, path-redacted feedback, and one shared Rust command.
- **Current UI ownership:** `src/app/routes/search.tsx` owns the Search Moneyball action and invalidates `searchKeys.all` and `moneyballKeys.all`. `src/features/csv-import/components/squad-csv-import-actions.tsx` currently exposes only Youth Academy upload and is composed by `src/app/routes/my-club.tsx`, which already owns cross-feature query invalidation.
- **Copy and tests:** `src/features/csv-import/components/csv-import-outcome.tsx` says matching rows replace earlier enrichment; Search conditionally labels existing cohorts as replacement. Focused coverage lives in `service.rs`, `squad-csv-import-modal.test.tsx`, `src/app/routes/search.test.tsx`, `src/app/routes/my-club-squad.test.tsx`, and `e2e/smoke.spec.ts`.
- **Project validation commands:** `./scripts/dev test [target...]`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke`.
- **Primary risks:** omitted-player data loss, stale percentiles after a partial upsert, null values accidentally merging instead of replacing, zero-match clearing, cross-snapshot writes, Squad membership filtering, misleading replacement copy, and stale Search/Profile caches after Squad upload.

## Feature architecture

Rust keeps one import pipeline for both entry points. Parsing and exact UID filtering remain outside the database lock. Moneyball preparation retains each matched player's complete normalized context and canonical statistics but no longer treats the incoming matched set as the percentile cohort.

Inside the existing revalidated write transaction, a non-empty Moneyball import upserts every included row by `(snapshot_id, player_uid)`. The conflict update replaces all format-owned raw columns, including nullable values, for that player. It does not delete omitted rows. The service then reads the complete resulting snapshot cohort, derives numeric statistics through the existing Moneyball metric contract, calls `calculate_percentiles`, serializes every complete percentile map, and updates every cohort row before commit. Any read, serialization, or update failure rolls back both raw upserts and percentile changes. An empty matched set returns the existing summary without issuing cohort writes or percentile updates.

The existing schema is sufficient. The composite primary key identifies the upsert target; the composite player foreign key rejects rows outside the snapshot; snapshot cascades preserve ownership; and `percentiles_json` already stores the derived full-cohort result. A migration would add no enforceable state and would create unnecessary upgrade risk.

React reuses `SquadCsvImportModal` with `format="moneyball"` from `SquadCsvImportActions`. My Club supplies a Moneyball-success callback that invalidates `searchKeys.all` and `moneyballKeys.all`; it does not invalidate or add Squad data because Squad has no Moneyball projection. Search uses the same modal and import command and keeps its existing invalidations. Shared and Search-specific copy describe cumulative upsert behavior without introducing source-set terms.

The route remains the cross-feature composition owner. The CSV import feature does not import Search or Moneyball query keys, and no frontend code validates managed-club membership or changes the Rust request.

## Uncertainty register

### Known

- Current production code deletes the captured snapshot cohort before inserting the incoming matched rows.
- The schema already enforces one Moneyball row for each `(snapshot_id, player_uid)` and exact snapshot-player ownership.
- `calculate_percentiles` already implements every approved percentile rule.
- Full CSV Search consumes the persisted complete-cohort percentile map. Player Profile uses persisted rows for scored-cohort readiness and membership, then recalculates metric percentiles and role scores from raw statistics over natural-position peers at read time.
- The 1,000-player limit applies to each file, not to the cumulative cohort. Repeated disjoint imports can grow the cohort up to current-snapshot membership.
- Both Search and My Club can compose the existing expected-format modal without adding a Tauri command or capability.

### Assumptions

- Updating `imported_at_utc` for each included matched row is part of full per-player replacement; omitted rows keep their prior timestamp. The timestamp is not exposed as a product comparison basis.
- Existing query tests are sufficient to retain consumer semantics when the query production files do not change. New persistence integration tests will prove the stored Full CSV Search basis changes correctly and that Profile retains its read-time natural-position calculation contract.

### Decisions

- Use one unnamed cumulative cohort per effective current snapshot rather than source sets.
- Upsert by the existing composite primary key and recompute the complete post-upsert cohort synchronously in the same atomic SQLite transaction. This approved contract accepts recomputation across every current-snapshot cohort member even when repeated imports grow the cohort beyond one file's 1,000-player limit.
- Adding a total cohort limit, a background job, or non-atomic recomputation requires a developer decision and replanning.
- Treat a valid empty or zero-match import as a successful no-op. Do not use it as a clear operation.
- Trust Squad exports and apply only the existing current-snapshot UID match. Managed-club membership is not an import boundary.
- Keep one PR. Persistence and UI remain separate atomic commits inside it; no migration or independently mergeable foundation justifies a second PR.
- Do not create an ADR. The approved behavior extends the existing snapshot-owned table, Rust transaction boundary, and route-owned invalidation seams without selecting a new durable architecture.

### Unknowns

- Whether native Windows Tauri browse/drop and restart persistence can be exercised during feature validation. If unavailable, close-out must preserve this as a validation gap.
- Whether a representative real squad export includes enough overlap and omission to manually observe cumulative behavior. Automated Rust fixtures remain the completion proof if no suitable export is available.

### Risks

- Calculating percentiles before the upsert would score only the incoming file and silently leave omitted players on an incompatible basis.
- An `ON CONFLICT` clause that omits nullable columns would retain stale values for included players instead of fully replacing their enrichment.
- Recomputing only changed players would leave unchanged cohort rows with obsolete percentiles after the population changes.
- A zero-match path could still reach the old delete or a recompute path and mutate timestamps or legacy null percentile state.
- Failure after raw upsert but before all percentile updates could commit a mixed cohort unless one transaction owns the whole operation.
- UI reuse could accidentally filter Squad imports by managed-club membership or create a second backend path.
- Search replacement labels or shared helper copy could continue to imply whole-cohort replacement.
- Squad success could leave Full CSV Search results or Player Profile readiness, membership, raw-statistic, and recalculated-score results stale if either query root is omitted from invalidation.
- Synchronous full-cohort recomputation can extend the import transaction as the current snapshot cohort grows; representative timing is reported evidence, not a pass/fail threshold.

## Walking skeleton

Import players A and B into the current snapshot, then import changed A and new C while omitting B. The transaction fully replaces A, preserves B, adds C, and persists recalculated percentiles for A, B, and C for Full CSV Search. Player Profile uses the resulting persisted membership and raw statistics to recalculate natural-position metric percentiles and role scores at read time. The same Rust path is then invoked from the My Club Squad action.

## Delivery plan

### PR 1 — Make Moneyball imports cumulative

**Status:** Ready for publication

**PR ref:** https://github.com/JG1995/fm-valuescout/pull/92

**Merge ref:** Not merged

**Branch:** `feature/cumulative-moneyball-imports`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub strict required status `check`

**Feature close-out:** Current

**CI repair rounds:** 0

**Provisional PR title:** `feat(import): make Moneyball imports cumulative`

**Purpose:** Deliver one coherent current-snapshot cumulative import contract and expose it through both Search and My Club without a migration or a second persistence path.

**Depends on:** Linear JAY-33 approved intent and the completed CSV enrichment, snapshot history, Moneyball views, Moneyball role scores, and My Club Squad workspace foundations already on `main`.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(import): plan cumulative Moneyball imports`

**Work:** Commit the independently reviewed ledger and TODO activation before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, BACKLOG, ADRs, completed records, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after exact branch and base verification.

**Files and responsibilities:**

- `.wiki/features/active/cumulative-moneyball-imports.md` — approved intent, architecture, delivery authority, implementation packets, risks, and validation contract.
- `.wiki/TODO.md` — point Active work to this ledger.
- `.wiki/BACKLOG.md` — deliberately unchanged because no accepted deferred scope changes.
- `.wiki/features/planned/cumulative-moneyball-imports.md` — not present, so no promoted spec removal.
- `.wiki/decisions/` — deliberately unchanged because no ADR meets the repository threshold.

**Behavior and data flow:**

- Record one reviewed active source of feature truth and the exact delivery sequence. This commit changes no executable behavior.

**Ordered implementation steps:**

1. Verify `main` is the recorded base and activate only `feature/cumulative-moneyball-imports` under a valid accepted Delivery fingerprint.
2. Confirm the planning work contains exactly this ledger and `.wiki/TODO.md` and excludes the developer-owned completed-record modification.
3. Run the ledger classifier and confirm one Active PR and one Active commit.
4. Stage exactly this initially untracked ledger and `.wiki/TODO.md`.
5. Run `git diff --cached --check`, verify the complete staged-name list equals only `.wiki/TODO.md` and this ledger, and inspect `git diff --cached -- .wiki/features/active/cumulative-moneyball-imports.md .wiki/TODO.md`.
6. Inspect `git status --short` and confirm `.wiki/features/completed/player-table-sort-performance.md` remains unstaged (` M`), while only the ledger and TODO are staged.

**Tests and proof:**

- Not applicable — independently reviewed planning documents only. `ledger_state.py` proves schema and lifecycle structure. The cached diff checks and `git status --short` prove staged path scope and preserve the developer-owned completed record as unstaged.
- No tests, fixtures, mocks, snapshots, helpers, or production compatibility paths change.

**Patterns to verify:**

- `.wiki/features/active/README.md` schema 2 and `.wiki/TODO.md` feature-level ownership.

**Constraints and non-goals:**

- Do not alter implementation, tests, configuration, BACKLOG, ADRs, completed records, Git history, plan scope, packet order, or approved decisions.
- Preserve `.wiki/features/completed/player-table-sort-performance.md` exactly and exclude it from the planning diff.

**Dependencies and sequencing:**

- Requires a clear independent plan review, developer acceptance, the later classifier-generated Delivery fingerprint, and exact branch activation. It precedes every implementation commit.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/cumulative-moneyball-imports.md`; after exact staging, `git diff --cached --check`; `test "$(git diff --cached --name-only)" = "$(printf '%s\n' .wiki/TODO.md .wiki/features/active/cumulative-moneyball-imports.md)"`; `git diff --cached -- .wiki/features/active/cumulative-moneyball-imports.md .wiki/TODO.md`; and `git status --short` with the developer-owned completed record still unstaged.

**Stop conditions:** Stop on an uncleared review, classifier error, unresolved fingerprint placeholder at delivery time, unreviewed path, substantive plan change, wrong branch/base, or any change to the preserved completed record.

**Review mandate:**

- Verify the diff contains the complete reviewed plan and TODO activation only.
- Verify the ledger matches approved JAY-33 behavior and schema 2.
- Verify there is one Active PR and exactly one Active commit.
- Verify no rejected source-set, migration, clear action, or managed-club filtering scope appears.
- Verify the developer-owned completed-record modification is absent from the planning diff.

#### Commit 2 — Upsert cumulative Moneyball cohorts

**Status:** Completed

**Provisional commit:** `feat(import): upsert cumulative Moneyball cohorts`

**Work:** Replace destructive snapshot-cohort replacement with full per-player upserts and atomic full-cohort percentile recomputation, including successful empty no-op behavior.

**Size assessment:** Estimated 100–170 changed non-test implementation lines in one Rust service file. Within the soft target; persistence and derived-state atomicity belong in one revertible commit.

**Out of scope:**

- Frontend actions or copy, query production changes, schema migrations, source-set identity, clear/remove operations, Youth Tracker behavior, and unrelated service refactors.

**Implementation packet:**

- Keep parsing, canonical statistic derivation, exact current-snapshot filtering, and context capture outside the transaction as they are. Change only the Moneyball prepared shape and persistence sequence needed to defer percentile calculation until the complete post-upsert cohort exists.

**Files and responsibilities:**

- `src-tauri/src/features/csv_import/service.rs::prepare_moneyball_import` — retain total count and complete matched per-player normalized/raw statistics without assigning incoming-only cohort percentiles.
- `src-tauri/src/features/csv_import/service.rs::persist_csv_import` — return a no-write success for an empty matched set; otherwise upsert included players, load the complete resulting captured-snapshot cohort, calculate and serialize percentiles for every row, persist every map, and commit once.
- `src-tauri/src/features/csv_import/service.rs::insert_moneyball_stats` — replace insert-only behavior with composite-key upsert that updates every format-owned per-player field, including nullable fields and import time, without touching omitted players.
- New narrow private helpers in `service.rs`, only if needed, own transaction-local cohort loading and percentile-map updates. Do not create a generic repository layer.
- Existing `service.rs` tests — rewrite destructive replacement assertions and add cumulative, null replacement, no-op, percentile, rollback, current-snapshot, and stale-context proofs.
- `src-tauri/src/db/migrations.rs` — deliberately retained unchanged; existing primary key, foreign keys, raw JSON, and `percentiles_json` satisfy the contract.
- `src-tauri/src/features/moneyball/percentile.rs` and its tests — deliberately retained unchanged because the approved formula is already implemented and directly reused.
- `src-tauri/src/features/moneyball/query.rs` and its tests — deliberately retained unchanged because Player Profile still uses persisted rows only for scored-cohort readiness and membership, reads raw statistics, and recalculates metric percentiles and role scores over natural-position peers at read time.
- `src-tauri/src/features/search/query.rs` and its tests — deliberately retained unchanged because Full CSV Search still uses persisted complete-cohort percentiles and filtered Search still recalculates its comparison cohort at read time.

**Behavior and data flow:**

- `import_csv` captures the active save, immutable tokens, effective current snapshot, and snapshot UIDs; parses and prepares the bounded file; then calls `persist_csv_import`.
- For Moneyball, preparation skips non-snapshot UIDs and preserves complete values for matched players. No matching row means a successful summary with zero stored and no SQL mutation.
- For a non-empty match, the revalidated transaction upserts each included row by `(snapshot_id, player_uid)`. Included nulls overwrite old non-null values; omitted rows are not selected for raw updates.
- The transaction reads canonical statistics for every row now in that snapshot cohort, uses `numeric_statistics` and `calculate_percentiles`, and updates all persisted percentile maps. Commit exposes raw and derived state together.
- A parse, stale-context, upsert, cohort decode, percentile serialization, update, or commit failure rolls back to the complete prior cohort.
- Full CSV Search observes the new persisted complete-cohort percentile basis through its unchanged current-only query. Player Profile uses the resulting persisted row membership and raw statistics, then recalculates metric percentiles and role scores over natural-position peers at read time.

**Ordered implementation steps:**

1. RED: rewrite `successful_moneyball_reimport_replaces_the_captured_snapshot_cohort` as a cumulative A+B then changed-A+new-C proof that currently fails because B is deleted and percentiles use only A+C.
2. RED: rewrite the zero-match assertion in `moneyball_imports_follow_the_effective_current_snapshot_and_clear_only_its_cohort` to require an unchanged active cohort and unchanged older snapshot cohort.
3. RED: add an included-player full-replacement case in which incoming nulls clear prior asking price, playing time, and statistic values while an omitted peer retains raw values.
4. RED: strengthen rollback proof so a forced failure during upsert or cohort percentile persistence leaves every prior raw value and percentile unchanged.
5. GREEN: change the prepared Moneyball shape and composite-key writer with the smallest private helpers needed for complete post-upsert cohort recomputation.
6. GREEN: short-circuit empty matched imports after revalidation and summary construction without issuing Moneyball writes.
7. GREEN: inside the same transaction, load all resulting cohort statistics, call the existing percentile engine, serialize and update every row, and commit.
8. REFACTOR: remove delete-based naming and obsolete replacement-only test setup while preserving focused invalid, stale, bounds, exact-match, cascade, and Youth tests.
9. Run Rust and full project gates in the recorded order.

**Tests and proof:**

- Modify `successful_moneyball_reimport_replaces_the_captured_snapshot_cohort` to `successful_moneyball_reimport_upserts_the_captured_snapshot_cohort` or an equally explicit name. Expected RED: omitted B disappears under current code. GREEN must assert changed A, preserved B, new C, exactly three rows, and percentiles recalculated for all three.
- Modify `prepares_moneyball_scores_from_only_matched_snapshot_players` so preparation still proves exact matching but no longer treats incoming preparation as the persisted percentile cohort. Move full-cohort score assertions to the persistence integration test.
- Modify `moneyball_imports_follow_the_effective_current_snapshot_and_clear_only_its_cohort` to prove imports target only the effective current snapshot and a zero-match valid import preserves both current and historical cohorts. Expected RED: current code deletes the effective-current cohort.
- Add or extend a real temporary-SQLite service test for included null replacement and omitted-player preservation. It prevents partial-field merge logic from retaining stale values.
- Modify `rolls_back_moneyball_replacement_when_an_insert_fails` to cover the cumulative upsert/recompute transaction and rename it. It prevents mixed raw/percentile cohorts after a late SQL failure.
- Retain stale save/snapshot token, current-change-after-parse, invalid/overflow, path-redaction, bounded-file, foreign-key/cascade, and Youth tests because each still protects a supported trust, ownership, or data-loss contract.
- Retain percentile unit tests for ties, inversion, null/non-finite exclusion, singleton, and constants. Do not duplicate formula cases in service tests beyond one multi-player recalculation that proves the service uses the engine over the resulting cohort.
- No fixtures, mocks, snapshots, or standalone helpers are added unless the existing small service builders cannot express changed/null/omitted rows. Delete any replacement-only helper made unused; retain the canonical CSV fixtures.

**Patterns to verify:**

- Existing Youth `upsert_youth_career_stats` for direct SQLite conflict-update style, without copying its save ownership.
- Existing `calculate_percentiles`, `numeric_statistics`, and serializers for Moneyball derivation.
- Existing `revalidate_import_context` transaction boundary and database-fingerprint rollback tests.
- Migration v18 composite ownership and v30 percentile JSON constraint.

**Constraints and non-goals:**

- Do not add a migration, dependency, background job, second command, source label, managed-club query, or frontend calculation.
- Do not delete omitted rows or coalesce incoming nulls with stored values.
- Do not recompute on an empty matched import.
- Do not change formulas, full-versus-filtered consumer behavior, snapshot selection, limits, errors, or Youth persistence.

**Dependencies and sequencing:**

- Depends on Commit 1. It establishes the authoritative backend behavior before frontend copy and the Squad entry point claim cumulative semantics.

**Validation:** `./scripts/dev check-rust`, then `./scripts/dev check`.

**Stop conditions:** Stop and replan if the current schema cannot express full per-player upsert without migration; if complete post-upsert cohort calculation cannot remain synchronous and atomic inside the revalidated transaction; if a consumer requires incoming-file percentiles; if implementation would add a total cohort limit, background job, or non-atomic recomputation; or if implementation would change snapshot selection, percentile formulas, or Youth behavior. Any such scale-contract change requires a developer decision.

**Review mandate:**

- Trace omitted, included, new, null, empty, zero-match, stale, invalid, and SQL-failure paths for data loss.
- Verify the conflict update replaces every included format-owned field and no omitted row.
- Verify every row in the resulting cohort receives percentiles from the complete resulting population in the same transaction.
- Verify no-op imports cause no Moneyball row, percentile, or timestamp mutation.
- Verify exact current-snapshot UID matching and composite foreign-key ownership remain intact.
- Verify rollback restores both raw values and percentiles after failures at each write phase.
- Verify no migration or consumer-query change is needed and no incoming-only percentile path survives.
- Audit modified, deleted, and retained tests against plausible regressions and command discovery.

#### Commit 3 — Upload cumulative Moneyball data from Squad

**Status:** Completed

**Provisional commit:** `feat(squad): upload cumulative Moneyball CSV`

**Work:** Add the secondary Squad upload action through the shared Moneyball modal, invalidate Search and Player Profile Moneyball caches after success, and replace whole-cohort replacement copy across the existing upload surfaces.

**Size assessment:** Estimated 80–150 changed non-test implementation lines across existing React composition and CSV-import components. Within the soft target; the action, shared truthful copy, and cross-feature invalidation form one user-visible integration outcome.

**Out of scope:**

- Backend persistence or schema changes, Squad Moneyball columns, managed-club filtering, new modal infrastructure, Search upload removal, source-set controls, broad My Club refactors, and unrelated design changes.

**Implementation packet:**

- Reuse `SquadCsvImportModal` and `useCsvImport`; do not add a new IPC adapter, command, picker listener, or import state store. The action passes `format="moneyball"`, so Rust applies the same expected-format and current-snapshot matching contract as Search.

**Files and responsibilities:**

- `src/features/csv-import/components/squad-csv-import-actions.tsx::SquadCsvImportActions` — render secondary **Upload Squad CSV** and existing **Upload Youth Academy CSV** actions, own separate modal selection state, and invoke the shared modal with Moneyball format and the route callback.
- `src/features/csv-import/components/squad-csv-import-modal.tsx::SquadCsvImportModal` — remove replacement-only title state if no caller needs it; retain accessible Modal, native single-file drop, context close/reset, pending lockout, and path-redacted outcome behavior.
- `src/features/csv-import/components/csv-import-outcome.tsx::CsvImportOutcome` — replace ambiguous replacement copy with cumulative per-player add/update and omitted-row preservation copy while keeping exact-current-snapshot skip language and phase-specific statuses.
- `src/app/routes/my-club.tsx::MyClubPageContent` — pass Moneyball success handling from the composition root and invalidate `searchKeys.all` plus `moneyballKeys.all`; do not invalidate Squad solely for this import.
- `src/app/routes/search.tsx::SearchPageContent` — keep **Upload Moneyball CSV**, remove conditional **Replace Moneyball CSV** and replacement-only modal state, and retain successful `searchKeys.all` and `moneyballKeys.all` invalidation plus context-bound last-import feedback.
- `src/features/moneyball/components/moneyball-profile-panel.tsx` — update only upload-location copy that claims Player Search is the sole import location, if the final UI would otherwise be false; retain query and calculation behavior.
- `src/features/csv-import/components/squad-csv-import-modal.test.tsx` — update copy/title assertions and retain browse/drop, expected-format, pending, path-redaction, stale-context, and keyboard-close coverage.
- `src/app/routes/search.test.tsx` — rewrite the obsolete existing-cohort replacement-label test to require stable Upload wording and retain successful Search/Moneyball invalidation behavior.
- `src/app/routes/my-club-squad.test.tsx` — replace the current assertion that Squad has no Moneyball action with action presence, shared-import invocation, success feedback, and Search/Moneyball cache invalidation proof without a managed-club membership request.
- `e2e/smoke.spec.ts` and its existing IPC stub only as needed — update the upload-ownership smoke to prove both entry points remain, Squad opens the Moneyball-formatted modal, success stays path-redacted, and Youth remains available.
- Existing CSV import API, hook, types, IPC mocks, Tauri capabilities, and Rust command registration — deliberately retained unchanged.

**Behavior and data flow:**

- In My Club Squad, the user activates **Upload Squad CSV**, chooses or drops one file, and the shared hook captures the current save/snapshot generation before calling `importCsv(path, "moneyball")`.
- The same Rust command performs bounded parsing, current-snapshot UID matching, cumulative persistence, and returns total/stored/skipped counts. The frontend neither sends managed-club membership nor filters returned players.
- The modal reports pending, success, skipped, format, and safe-error states in its existing live regions. A context change closes it and suppresses late feedback; local paths never render.
- On success, My Club invalidates Search and Moneyball profile query roots. It does not add Moneyball data to the Squad table.
- Search follows the same path, keeps its existing invalidations, and always labels the action as upload because a later file adds or updates players rather than replacing the cohort.

**Ordered implementation steps:**

1. RED: change `my-club-squad.test.tsx` to expect **Upload Squad CSV**, open its Moneyball modal, complete the mocked import, and observe both Search and Moneyball query refresh evidence. Current code fails because the action is absent.
2. RED: rewrite the Search replacement-label test to require **Upload Moneyball CSV** even when a cohort exists, and add shared outcome copy assertions for cumulative update plus omitted-row preservation.
3. RED: update the smoke ownership scenario so Search and Squad both expose Moneyball entry points while Squad also retains Youth Academy upload.
4. GREEN: extend `SquadCsvImportActions` with separate Moneyball modal state and a typed success callback; compose it in My Club with route-owned invalidations.
5. GREEN: remove replacement-only Search/modal props and copy, preserve context-bound summary handling, and update profile empty-state location copy only where necessary.
6. REFACTOR: keep shared modal/hook behavior single-sourced; remove obsolete `replace`, cohort-presence state, imports, or tests only when no supported behavior uses them.
7. Run focused component/route tests, the frontend/full gate, and smoke in the recorded order.

**Tests and proof:**

- Modify `src/app/routes/my-club-squad.test.tsx` to prove the secondary Squad action exists beside Youth, uses `expectedFormat: "moneyball"`, reports summary without a path, and invalidates Search plus Moneyball profile observers. The plausible regression is a second path that omits expected-format binding or leaves consumers stale.
- Modify `src/app/routes/search.test.tsx` test `labels an existing Moneyball cohort as a replacement` to assert stable upload wording for both empty and populated cohorts. Delete replacement-only presence plumbing assertions if the production plumbing is removed; retain upload success and context-transition tests.
- Modify `squad-csv-import-modal.test.tsx` copy assertions while retaining every picker cancellation, pending lock, duplicate drop, listener cleanup, format mismatch, path-redaction, stale context, and multi-path rejection test. These remain supported and protect native-boundary and accessibility regressions.
- Modify `e2e/smoke.spec.ts` test `Moneyball Search owns its CSV upload while Squad keeps Youth Academy` to reflect shared ownership and prove keyboard-reachable visible actions and modal format. The browser stub is presentation evidence only.
- Retain `src/testing/csv-import-ipc-mock.ts` and `e2e/tauri-ipc-stub.ts` unless a small response adjustment is required; do not add a second mock protocol.
- No snapshots or fixtures are expected. Remove only replacement-specific assertions/helpers made obsolete by stable Upload copy.

**Patterns to verify:**

- Existing Search Moneyball modal callback for `searchKeys.all` and `moneyballKeys.all` invalidation.
- Existing route-owned cross-feature invalidation in My Club and the no-cross-feature-import rule inside feature modules.
- Existing modal focus trap, pending close protection, native listener cleanup, context generation, safe error copy, and live-region outcomes.
- Existing secondary Button and My Club action-group layout.

**Constraints and non-goals:**

- Do not validate managed-club membership in React or Rust and do not send a club ID or player list.
- Do not create a new command, modal, query cache, Tauri capability, Squad column, or source-set concept.
- Do not show local file paths or preserve late feedback after context replacement.
- Keep all controls keyboard operable and all pending/success/error copy phase-truthful.

**Dependencies and sequencing:**

- Depends on Commit 2 so every exposed entry point has cumulative semantics. This is the final implementation commit and moves the feature to Validation after checkpoint completion.

**Validation:** `./scripts/dev test src/features/csv-import/components/squad-csv-import-modal.test.tsx src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx`, then `./scripts/dev check`, then `./scripts/dev smoke`.

**Stop conditions:** Stop and replan if My Club cannot own cross-feature invalidation without a forbidden feature import; if the shared modal cannot support two independent actions without mixing context or feedback; if the backend request would need managed-club identity; if native capability changes are required; or if Squad already exposes Moneyball columns that make the approved no-column boundary ambiguous.

**Review mandate:**

- Verify both entry points call the same expected-format Moneyball IPC path and no club-membership data crosses the boundary.
- Verify Search and Player Profile Moneyball roots invalidate after Squad success while Squad stays unchanged.
- Verify all visible copy describes cumulative add/update behavior and never whole-cohort replacement.
- Verify modal focus, keyboard close rules, live regions, pending labels, and context-change cleanup remain accessible and truthful.
- Verify errors and success summaries cannot expose selected paths.
- Verify separate Youth and Squad modal state cannot cross-trigger or retain stale feedback.
- Verify obsolete replacement-only state and tests are removed without weakening supported upload coverage.
- Verify smoke claims only browser-stub presentation and does not stand in for native Tauri or SQLite proof.

## Active work

**PR:** PR 1 — Make Moneyball imports cumulative

**Active work:** None — documentation close-out

**Commit:** None — documentation close-out

### RED or removal proof

Not applicable — all three planned packets, full feature validation, feature review, and documentation reconciliation are complete. The reviewed close-out is ready for final PR publication.

### Expected outcome

The reviewed close-out is ready for final PR publication.

### Explicit exclusions

- Release preparation and unrelated implementation or documentation.
- The pre-existing developer-owned modification in `.wiki/features/completed/player-table-sort-performance.md`.

## Discoveries and replanning

- Planning confirmed that migration v18's composite primary key and player/snapshot foreign keys plus migration v30's percentile JSON column already express cumulative per-player upsert and complete derived-cohort persistence. No migration is planned.
- Planning confirmed that Full CSV Search reads persisted complete-cohort percentiles. Player Profile uses persisted rows only for scored-cohort readiness and membership, reads raw statistics, and recalculates metric percentiles and role scores over natural-position peers at read time. Consumer production queries stay unchanged.
- Planning confirmed that the 1,000-player cap is per file. The cumulative cohort is bounded only by current-snapshot membership, and the approved synchronous atomic recomputation contract applies to the complete resulting cohort. A total cohort limit, background job, or non-atomic recomputation requires a developer decision and replanning.
- Planning selected one PR because the persistence and UI commits are atomic and ordered but share one feature review surface; no risky foundation or independently publishable seam requires a second PR.
- Feature close-out passed `./scripts/dev test`, `./scripts/dev check-rust` (641 passed, 2 ignored), `./scripts/dev check`, and `./scripts/dev smoke` (49 passed); focused frontend tests passed 187 tests. Independent feature review found no CRITICAL, HIGH, or MEDIUM findings, rated the test portfolio Pass and project fit Conforms, and retained one non-blocking NITPICK for the unused `pendingMoneyballCohort` test-mock mode.
- Native Windows/Tauri picker and WebView drop, packaged IPC, focus restoration at 1280×800 and 1600×900, a representative overlapping Search/Squad export, native pending context replacement, real application SQLite restart, and representative cumulative-cohort timing were unavailable and remain explicit manual validation gaps.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Make Moneyball imports cumulative | Commit 1 — Record the approved feature plan | fd37d2fa2c159f10c4a6507dac861d97e7b76a1f | Recorded the reviewed schema 2 ledger and activated the feature in TODO without changing executable behavior. | `ledger_state.py`: runnable; `delivery_state.py`: runnable; `git diff --cached --check`: passed. | Not applicable | Clear | 0 | None. |
| PR 1 — Make Moneyball imports cumulative | Commit 2 — Upsert cumulative Moneyball cohorts | 3d9d94362555097e0dd0663a2d41d50da41f9013 | Replaced destructive cohort replacement with per-player upserts, write-free empty matches, and atomic complete-cohort percentile recomputation while preserving current-snapshot ownership and rollback. | `./scripts/dev check-rust`: 641 passed, 2 ignored; `./scripts/dev check`: passed; LSP and `git diff --cached --check`: passed. | Pass | Clear | 0 | None. |
| PR 1 — Make Moneyball imports cumulative | Commit 3 — Upload cumulative Moneyball data from Squad | 58035e4de0f53c2f52f5a0169f7c8124893ef9b7 | Added the shared Moneyball upload action to My Club Squad, route-owned Search/Profile invalidation, stable cumulative copy, and removed obsolete whole-cohort replacement UI state. | Focused frontend tests: 187 passed; `./scripts/dev check`: passed with 641 Rust tests passed and 2 ignored; `./scripts/dev smoke`: 49 passed; `git diff --cached --check`: passed. | Pass | Clear | 0 | None. |

## Final validation

Required automated evidence before feature review:

- `./scripts/dev test`
- `./scripts/dev check-rust`
- `./scripts/dev check`
- `./scripts/dev smoke`
- Rust service proof for cumulative A/B then A/C upsert, included null replacement, omitted-player preservation, new-player addition, complete post-upsert percentile recomputation, empty/zero-match no-op, rollback, current-snapshot ownership, and stale/invalid preservation.
- Frontend proof for both upload entry points, stable cumulative copy, expected Moneyball format, context-bound and path-redacted feedback, keyboard operation, and Search/Moneyball query invalidation after Squad success.

Manual/native evidence target:

- On a supported Windows Tauri build, import a representative Moneyball CSV from Search, then a partly overlapping Squad export from My Club. Confirm included players update, omitted players remain in Full CSV Search and the Player Profile scored cohort, new matched players appear, and an outside-snapshot UID is skipped. Confirm Full CSV Search uses the persisted complete-cohort percentiles and Player Profile recalculates metric percentiles and role scores from raw statistics over natural-position peers.
- Exercise browse and native WebView drop with keyboard focus restoration at 1280×800 and 1600×900. Change save or effective snapshot during an open or pending import and confirm the modal closes or suppresses late feedback without exposing the path.
- Restart the app and confirm the cumulative cohort and persisted complete-cohort percentiles survive in Full CSV Search. Confirm Player Profile retains scored-cohort readiness and membership and recalculates its metric percentiles and role scores from persisted raw statistics over natural-position peers.
- Record representative synchronous import timing for the available cumulative cohort size as evidence only. Do not apply an invented threshold.
- If native Windows/Tauri, real SQLite-file restart, or representative export validation is unavailable, record each exact gap. Playwright IPC stubs and temporary-database Rust tests do not prove native picker/drop, packaged WebView IPC, or real-file restart behavior.

`./scripts/dev bridge-test` is not planned because bridge source and protocol are unchanged. Run it and replan if implementation crosses that boundary. `./scripts/dev mutate` remains unsupported and must not be reported as passed.

## Documentation impact

Reconciliation complete: `.wiki/ARCHITECTURE.md`, `.wiki/DESIGN.md`, and `.wiki/TODO.md` now describe the implemented cumulative import behavior and upload ownership. `.wiki/CONCEPT.md` requires no change. No ADR or debug report is warranted: the feature extends the existing snapshot-owned table, transaction boundary, and route-owned invalidation seams. The orchestrator must move this complete ledger to `.wiki/features/completed/cumulative-moneyball-imports.md` after inspection.

Active work is none; documentation close-out is complete. Preserve the accepted Delivery fingerprint and PR status: Ready for publication / PR ref Not published / merge ref Not merged.
