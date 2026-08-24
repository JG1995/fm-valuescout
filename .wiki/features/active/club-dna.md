# Club DNA

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 8b255cb0d43d34e6023ffc26c1e194aaa24e765e99ae1ebca1a639c0810fade8

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
- The first request for an uncached Club DNA page or cohort can take longer while Rust materializes scores. The existing table query-loading state stays visible, no partial filtered or sorted result appears, and later warm requests use the cache. Cold duration is measured and reported separately from warm interaction latency.
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
- Data model: `players` stores visible and goalkeeper values in `attributes_json`, hidden values in `hidden_attributes_json`, and personality values in `personality_json`. Committed v31 and `src-tauri/src/features/club_dna/` now provide one save-keyed definition plus context-bound get/set/remove commands. No definition version or Club DNA score cache exists yet.
- Persistence and migrations: committed migration v31 adds `club_dna_definitions(save_id, attribute_ids_json)` with one save-owned row and no definition version. Save-owned tables use `save_id REFERENCES saves(id) ON DELETE CASCADE`. Migration v21 and `src-tauri/src/features/player_metrics/potential_cache.rs` are the closest disposable-cache, bounded-batch, nullable-row, model-version, page/cohort-scope, and player-boost invalidation analogues.
- Existing behavioral assumptions: most React query keys omit save IDs because app-shell and Settings context changes invalidate feature roots, but save-owned Club DNA cannot rely on invalidation alone. The established saves query exposes each active save's ID and immutable `contextToken`. Search filters and sort live in URL state. Search and Squad column IDs, order, and widths live app-locally in Zustand key `fm-valuescout-player-table-layouts`, version 5. `addColumns` validates IDs and appends only missing columns.
- Architectural seams: `src/utils/player-metrics.ts` and `src/components/ui/player-metric-picker.tsx` own the frontend fixed metric catalog and picker. `src-tauri/src/features/player_metrics/resolver.rs::MetricField` owns the independent Rust catalog and dynamic value/sort expressions. `src-tauri/src/features/player_metrics/potential_cache.rs` shows the established lazy materialization boundary. `src-tauri/src/features/search/filter.rs` owns trusted Search filter compilation. `src-tauri/src/features/search/query.rs` and `src-tauri/src/features/planner/squad.rs` own the current-snapshot Search and exact managed-club Squad cohorts.
- Shared table adapters: `src/features/search/components/search-results-panel.tsx` and `src/features/squad/components/squad-overview-panel.tsx` request visible dynamic fields and render nullable values. `src/components/player-table/` owns table interaction. Existing dynamic DTO maps already carry nullable integers.
- Command boundary: `src/lib/tauri-client.ts` is the sole frontend invoke wrapper. `src-tauri/src/lib.rs` registers Tauri commands, and `src-tauri/src/features/mod.rs` registers feature modules.
- Project validation commands: `./scripts/dev test <targets>`, `./scripts/dev check-app`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` is unsupported and cannot be evidence.
- Primary risks: cross-language catalog drift, accepting an empty or unknown definition, partial averages, stale or mixed-version cache rows, incomplete cohort materialization before global filter/sort, long first-use database lock time, stale active-save Query data, re-adding a user-removed column on edit, and removing saved layout state with the definition.

## Feature architecture

Committed migration v31 remains unchanged and continues to own one validated definition per save. Migration v32 adds `definition_version` to that definition and adds a disposable `club_dna_scores` cache. The cache identity is `(snapshot_id, uid, definition_version, score_model_version)`. Each row stores a nullable score constrained by `score IS NULL OR score BETWEEN 0 AND 100`, references the owning player/snapshot with cascade deletion, and has an index for snapshot/version score lookup. Cache rows are derived data and are safe to delete or rebuild.

`player_metrics::club_dna` owns one pure Rust score function and one lazy materializer. Rust reads and validates the active save's definition once per request. It loads player JSON in bounded batches, calculates one nullable score per player, and writes each batch in its own short transaction. A display-only request materializes only the requested page UIDs. Search filter or sort materializes the complete current-snapshot Search cohort before count, filter, or ordering. Squad sort materializes the exact configured managed-club cohort before ordering. After materialization, Search and Squad resolve `club_dna` only through the versioned cache and keep bounded result pages.

The materializer validates the request, snapshot, definition, and cohort before it opens a write transaction. It does no work for an invalid request or a missing definition. The existing synchronous Search or Squad command keeps its current `Db` mutex guard for the request, including materialization, so another database command cannot observe or interleave an incomplete global cohort. Each batch transaction owns only that batch's cache writes and releases the SQLite write lock before the next batch. No transaction or mutex guard crosses an async wait. Partial derived rows after an error are safe and resumable, but a global filter or sort does not run until its complete required cohort is materialized. The unique cache identity and idempotent upsert keep retries safe. Cold first use can therefore delay other database commands and must remain explicit in UX and validation.

Definition edits increment `definition_version` and delete all stale Club DNA rows owned by that save in the same transaction. Definition removal deletes the definition and all save-owned cache rows atomically. Re-creation inserts a new definition with a fresh version lineage and reports `created: true`. Ingest and current-snapshot promotion do not precompute scores. Snapshot/player deletion cascades cache rows. Every successful supported player boost deletes that player's Club DNA rows in the same reconciliation transaction. A formula change increments `score_model_version`; stale model rows behave as misses and are replaced lazily.

React behavior stays as planned. One fixed integer metric uses backend-supplied cached values and shared score presentation. Typed definition adapters carry the expected active save ID and immutable save context token. The context-bearing Query key, route remount, callback guards, and Rust stale-context verification prevent cross-save data or effects. Only a current-context successful create appends `club_dna` to Search and Squad layouts. Edit and remove do not rewrite layouts or URL state.

[ADR-0023](../../decisions/0023-lazy-club-dna-score-cache.md) records the cache decision, invalidation owners, and measured thresholds. No current-state architecture or design document changes during this replan because v32 and cache-backed queries are not implemented yet.

## Uncertainty register

### Known

- Linear JAY-32 is the external work item. There is no planned feature spec to promote.
- Commit 1 recorded the original plan at `ddd4961e6d90ca24faa435955c6ae7eb5a716f0b`. Commit 2 implemented v31 definition persistence at `d2682ee5c50cb99cd0b7f9facf5fd4f9060d5001`.
- The discarded direct-SQL experiment used a complete-catalog 2,000-player Search cohort. Search filter samples were `[1875,1860,1862,1865,1902,1913,1881,1878,1865,1858,1863,1868,1877,1895,1889,1880,1900,1859,1896,1855]` ms with nearest-rank p95 1902 ms. Search sort samples were `[914,918,920,917,923,920,923,922,923,922,920,913,904,912,902,908,907,911,916,922]` ms with nearest-rank p95 923 ms. Both breach the required `<500 ms` guard.
- No representative roughly 180,000-player run was attempted. The failed code experiment was discarded cleanly; only its evidence and contract-level test cases remain planning inputs.
- Search supports display, sort, and filter. Squad supports display and sort. Both already carry nullable integer dynamic values.
- ADR-0019 and `player_metrics::potential_cache` establish lazy page/cohort materialization, nullable cache rows, model versioning, bounded transactions, cascade deletion, and same-transaction player-boost invalidation.

### Assumptions

- Canonical metric-style IDs remain the narrowest persisted definition because they encode the closed attribute and JSON source.
- A per-definition monotonic integer version plus a constant score-model version is sufficient cache identity; no timestamp or content hash is required.
- A route-owned action slot in `ManagedClubSelector` is sufficient to place **Define DNA** beside **Save managed club** without a cross-feature import.
- The existing `ScoreBadge` is the correct 0–100 presentation in both tables.

### Decisions

- Keep committed v31 unchanged. Add definition versioning and the nullable disposable score cache only in migration v32.
- Use cache identity `(snapshot_id, uid, definition_version, score_model_version)` and an index suitable for snapshot/version score filtering and ordering.
- Read and validate the definition once, compute in Rust in bounded batches, and persist null rows so incomplete players are not recalculated on every read.
- Materialize requested page UIDs for display, the complete Search cohort for filter/sort, and the exact managed-club cohort for Squad sort.
- Missing definition and invalid requests perform no cache writes and return the existing safe null/error contract. First use remains an explicit cold operation; warm latency is the interactive gate.
- Definition edit increments the version and clears save-owned rows atomically. Remove clears definition and rows atomically. Re-create starts a new definition. Ingest and promotion remain lazy.
- Supported player boosts invalidate that player's Club DNA rows in the same reconciliation transaction. Formula changes bump `score_model_version`.
- Keep `club_dna` permanently valid in both catalogs. No-definition and post-removal queries return null rather than an unknown-field error.
- Let the backend report create versus replace. Do not let React infer creation from its Query cache.
- Consolidate the frontend attribute catalogs into a shared utility. Keep a separate Rust closed catalog because Rust is the authoritative trust boundary.
- Keep all explanatory copy and selected-attribute detail inside the form Modal. Tables show only the fixed metric score or `—`.

### Unknowns

- Native Tauri/WebView density and focus behavior for selecting the full catalog cannot be proved in headless Chromium.
- Representative roughly 180,000-player cold first-use and warm filter/sort timings remain unavailable. Publication stops until they are measured or the developer explicitly accepts the unavailable-environment gap. A measured warm breach cannot be accepted as a gap.
- Cold materialization duration and perceived first-use responsiveness are not yet measured. The implementation must report cold timing separately instead of hiding it inside warm samples.

### Risks

- A cache lookup that omits either version can return stale scores after definition or formula changes. Migration, resolver, and query tests must prove exact identity use.
- A materializer can expose incomplete global results if filter or sort runs before every required cohort row exists. Search and Squad tests must prove full-cohort completion before count/order while display remains page-only.
- A long transaction can block unrelated SQLite work. Use bounded player batches and one short write transaction per batch; record cold duration and review lock scope.
- Cold materialization holds the existing command-level `Db` mutex while it computes batches, so other database commands wait even though each SQLite write transaction is short. Cold timing and native first-use behavior determine whether a later background-job decision is required.
- A stored malformed or unsupported ID could poison a full cohort. Validate the definition once before any cache work and fail safely without partial new-version writes.
- A missing or out-of-domain selected value can be mistaken for zero, clamped, or omitted from the denominator. Pure-scoring tests must include missing keys, explicit nulls, non-integers, mixed JSON sources, valid 1 and 20 boundaries, and invalid 0 and 21 boundaries in each applicable JSON source.
- Definition edit/remove and player boosts can leave stale rows. Each owning mutation must delete the exact rows in its existing transaction, with rollback tests.
- The same app-local layouts serve every save. A save with no definition must return null without query or materialization errors.
- Invalidation alone cannot bind a late definition response to the requesting save. All definition IPC calls, Query keys, route effects, and Rust checks retain the exact save ID/context-token contract.
- Automatic append can override user customization if edit is mistaken for create. Backend `created`, store tests, and route tests must distinguish create, edit, delete, and re-create.

## Walking skeleton

Migration v31 remains the trusted definition record. Migration v32 adds definition versions and the disposable score cache. Rust materializes one Search page for display and a complete Search cohort for filter/sort, then resolves the cached nullable score through the existing bounded table path. Later packets add Squad proof and the unchanged frontend definition and layout behavior.

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

#### Commit 5 — Resolve cached Club DNA in Search and Squad

**Status:** Active

**Provisional commit:** `feat(club-dna): resolve cached table scores`

**Work:** Route fixed `club_dna` display, Search filter/sort, and Squad sort through the versioned cache with separate page and full-cohort correctness and performance proof.

**Size assessment:** About 180–240 changed non-test implementation lines. The resolver plus Search and Squad seams may exceed the soft target; keep them atomic so the fixed metric cannot ship with inconsistent display/filter/sort semantics.

**Out of scope:**

- Cache schema or definition mutation changes, frontend metric metadata or UI, eager ingest/promotion, Moneyball, Player Profile, background progress/cancellation, and current-state documentation.

**Implementation packet:**

- Validate requests before materialization. Use Commit 4's materializer for page-only display and complete global cohorts. Resolve all score reads, filters, and sorts from exact current-version cache rows, then preserve existing bounded page DTOs and null-last behavior.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/resolver.rs::MetricSource`, `MetricField::parse`, display expression, and sort expression — recognize only `club_dna` and read a nullable score from the exact snapshot/definition/model cache identity.
- `src-tauri/src/features/search/filter.rs::FieldKind`, `resolve_field`, and `compile_rule` — compile trusted integer comparisons against the exact cache identity with explicit null exclusion, including `neq`; expose needed validated Club DNA intent without materializing during parse failure.
- `src-tauri/src/features/search/query.rs::search_players_in_view` — after all request/filter validation, materialize complete current-snapshot Club DNA rows for a Club DNA filter or sort; for display only, query requested page UIDs first and materialize only those UIDs before final select; add correctness, scope, invalid-request, no-definition, cold, warm, and full-catalog tests.
- `src-tauri/src/features/planner/squad.rs::list_squad_players` — for Club DNA sort, materialize the exact managed-club UID cohort before ordering; for display only, materialize requested page UIDs; retain empty/unconfigured safe paths and add separate Squad sort timing proof.

**Behavior and data flow:**

- Invalid field/filter/sort/page requests fail through existing validation before cache work. A valid request with no definition returns bounded rows with null `club_dna` values and writes no cache rows. Display-only Search or Squad first selects the requested page UIDs under existing non-Club-DNA order, materializes only those UIDs, and then returns their cached values. Search Club DNA filter/sort materializes every player in the active current snapshot before count/filter/order. Squad Club DNA sort materializes only the exact current managed-club cohort before order. All reads require the current definition and score-model versions.
- Cold first use may take longer while Rust fills the cohort. The existing Search or Squad query-loading state remains the user-visible first-use surface until the complete dependent result returns; no partial global result appears. Tests and manual evidence record cold duration separately. Warm interaction is measured only after prefill and must meet the thresholds below. The WebView still receives only bounded pages.

**Ordered implementation steps:**

1. Add RED resolver/filter tests for exact fixed-ID acceptance, unsafe/unknown rejection, current-version cache lookup, null guards for every integer operator, deterministic null-last sort, and stale-version exclusion.
2. Add RED Search tests proving display materializes only page UIDs, filter/sort materialize the complete current snapshot before query, missing definition and invalid requests write nothing, and edit/remove/re-create results cannot reuse stale rows.
3. Add RED Squad tests proving display is page-only, sort materializes the exact managed-club cohort, unconfigured/no-definition requests are safe, and non-members never materialize for Squad sort.
4. Wire the resolver and filter compiler only to exact cache rows. Add page/full-cohort materialization after validation and before the dependent select/count/order operation.
5. Reuse complete-catalog 2,000-player fixtures. Clear Club DNA rows, run one cold Search filter, Search sort, and Squad sort first-use measurement separately, and record each cold duration without treating it as warm evidence.
6. Prefill the current-version cache. For each materially distinct Search filter, Search sort, and Squad sort shape, run 3 unmeasured warm-ups and 20 warm measured executions. Sort samples and use nearest-rank p95 at index 18; require each p95 `<500 ms`.
7. On a representative roughly 180,000-player environment, clear rows and record cold first use separately. Prefill current-version rows, then run 3 warm-ups and 20 warm executions for each materially distinct filter/sort shape; require nearest-rank p95 `<= about 200 ms`.
8. Run Rust and full gates. Publication remains stopped if representative evidence is unavailable until the developer explicitly accepts that gap.

**Tests and proof:**

- RED: `MetricField::parse("club_dna")` fails; no query can resolve cached values; a page-only request cannot distinguish page from cohort work. A stale-version or partial-cohort mutant must fail.
- GREEN correctness: `(10, 20)` resolves as `75`; mixed sources and tie rounding use Commit 4's pure scorer; missing/null inputs stay null; `neq` excludes null; exact current versions control every display/filter/sort; display writes only page rows; Search filter/sort and Squad sort write every and only required cohort row before returning ordered results.
- Safe-path proof: unsupported fields/operators, malformed requests, absent snapshots, absent definitions, and unconfigured Squad return their existing safe error/empty/null outcomes without Club DNA cache writes. Do not materialize before filter and requested-field validation succeeds.
- Performance proof: use the complete supported catalog and complete representative JSON for all 2,000 players. Clear the cache and record cold duration for each distinct first-use shape separately. Prefill, then run 3 warm-ups plus 20 warm samples for Search filter, Search sort, and Squad sort; nearest-rank p95 is sorted index 18 and must be `<500 ms`. Representative roughly 180,000-player cold first use is recorded separately; warm 3+20 filter/sort p95 must be `<= about 200 ms`. Missing representative access stops publication for explicit developer gap acceptance. Any measured warm breach requires replan and cannot be accepted as a gap.
- Add/modify: resolver, filter, Search query, Squad query, page/cohort scope fixtures, complete-catalog timing helpers, and cold/warm reports in test output or captured delivery evidence. Deliberately retain raw attribute, role, potential-role, current-snapshot, managed-club, pagination, null-ordering, and request-bound tests because they protect independent behavior. Delete no fixtures, mocks, snapshots, helpers, or compatibility paths.

**Patterns to verify:**

- `search_players_in_view` potential-role full-snapshot materialization followed by page UID materialization.
- `list_squad_players` exact managed-club potential-role sort and page-display scopes.
- `MetricSource::PotentialRole` and `compile_potential_role_rule` for versioned cache lookup and null guards.
- `attribute_filter_on_two_thousand_players_stays_interactive` for test-environment timing style, with the stricter cold-versus-warm contract in this packet.

**Constraints and non-goals:**

- Never derive a full cohort in SQL or React. Never return the full cohort to the WebView.
- Never filter or sort a partial cohort. Never treat a cold duration as a warm sample or hide first-use cost.
- Do not materialize on invalid requests or missing definitions. Preserve 256 requested-field and 32 filter-rule bounds.
- Keep `club_dna` out of Moneyball mode and Player Profile.

**Dependencies and sequencing:**

- Depends on Commit 4's v32 identity, pure scorer, materializer, and invalidation. Commit 6 may expose the fixed metric only after this packet passes correctness and 2,000-player warm gates.

**Validation:** `./scripts/dev check-rust` then `./scripts/dev check`

**Stop conditions:** Stop if any global query can run against an incomplete cohort, display requires full-cohort work, invalid/no-definition paths write rows, exact version joins cannot support all operators and null-last sort, each batch cannot close its SQLite write transaction before the next batch's calculation, any 2,000-player warm nearest-rank p95 is `>=500 ms`, or any representative warm p95 exceeds about 200 ms. Missing representative evidence stops publication for explicit developer acceptance; a measured warm breach requires replan. Stop and make first-use progress/cancellation a developer decision if measured cold behavior makes the app unusable rather than merely delayed.

**Review mandate:** Verify (1) validation precedes materialization, (2) page display versus complete Search/exact Squad cohort scopes, (3) exact version/null semantics across display/filter/sort including `neq`, (4) bounded DTOs and no WebView calculation, (5) safe missing-definition/unconfigured/invalid paths with zero writes, (6) cold timing is separate and user-visible first-use cost is explicit, (7) 2,000 and representative warm methodology/thresholds are exact, and (8) warm breach and missing-environment stop paths remain distinct.

#### Commit 6 — Add the frontend Club DNA domain and fixed metric

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

- Frontend callers receive an explicit context from the established saves query and can fetch that context's nullable definition or invoke set/remove with the same save ID and immutable token. The definition Query key contains both values, so save A data cannot satisfy save B or a replacement save incarnation. Adapters return results to their caller but do not infer currentness; Commits 7–8 UI compares the captured context before applying results, while Rust rejects a context that is no longer active. The shared catalog exposes canonical IDs and FM grouping but no score function. The fixed `club_dna` metadata flows through the existing picker, filter registry, sort validation, requested-field adapter, nullable dynamic DTO, and table cell. Search and Squad render a backend-supplied integer with `ScoreBadge`; null stays `—`.

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

- Depends on Commit 5 so every exposed metric request is already accepted and cache-backed in Rust. The later Modal and route packets consume these adapters and shared groups.

**Validation:** `./scripts/dev test src/features/player-profile/utils/attribute-groups.test.ts src/utils/player-metrics.test.ts src/features/club-dna/api/club-dna-api.test.ts src/features/search/utils/dynamic-columns.test.ts src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx`; `./scripts/dev check-app`; `./scripts/dev check`

**Stop conditions:** Stop if catalog consolidation changes a current Player Profile group, if a supported frontend ID has no exact Rust counterpart, if Query/API adapters cannot bind get/set/remove to `{ saveId, contextToken }`, if the fixed metric requires a new table component or store owner, if store retention requires resetting user layouts, or if React would need to compute a score.

**Review mandate:** Verify (1) exact catalog parity and no cross-feature import, (2) Player Profile grouping stays unchanged, (3) fixed label/ID and integer filter metadata, (4) no default layout insertion and persisted layout retention, (5) ScoreBadge/null presentation in both tables, (6) context-bearing definition key isolates save IDs and tokens, (7) all typed invoke requests carry the exact expected context, and (8) no frontend score, stale-context authority, or validation duplicates Rust.

#### Commit 7 — Build the Club DNA definition Modal

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

- Depends on Commit 6's typed API and shared catalog. Commit 8 provides route placement and cross-query/layout effects.

**Validation:** `./scripts/dev test src/features/club-dna/components/club-dna-definition.test.tsx`; `./scripts/dev check-app`; `./scripts/dev check`

**Stop conditions:** Stop if the shared Modal cannot support keyboard access, full-catalog scrolling, the one-Modal edit ↔ confirmation transition, pending dismissal guards, or reliable focus return; if a context change cannot close/discard the draft and suppress late prior-context results; if product behavior requires an explanation outside the Modal; or if component state cannot distinguish backend create from edit.

**Review mandate:** Verify (1) full approved catalog and no maximum, (2) minimum-one/current-context guards plus Rust stale-context authority, (3) selected summary and exact formula copy only in Modal, (4) context change closes/discards and late get/set/remove results cannot update current UI, (5) the Planner-style single Modal preserves draft across confirmation and implements exact Cancel/Escape/pending/error transitions, (6) keyboard/focus/accessibility behavior including successful-remove focus return, (7) no score computation or profile surface, and (8) deferred mocks test observable stale-result suppression without duplicating Rust mutation authority.

#### Commit 8 — Integrate Club DNA with My Club and layouts

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

- Depends on Commits 2–7. It is the final implementation packet and moves the feature to Validation only after its full checkpoint clears.

**Validation:** `./scripts/dev test src/features/club-dna/components/club-dna-definition.test.tsx src/app/routes/my-club-squad.test.tsx src/app/app-top-bar.test.tsx src/app/routes/settings.test.tsx src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx`; `./scripts/dev test`; `./scripts/dev check`; `./scripts/dev smoke`

**Stop conditions:** Stop if placement requires a cross-feature import, if My Club cannot derive and key the feature by the saves query's active ID/token, if create-versus-edit cannot be based on the backend result, if a save change can leave a draft open or allow prior-context data/feedback/layout append, if managed-club or Club DNA refresh/mismatch/error cannot disable interaction, if remove requires deleting catalog/layout/URL state, if Rust cannot remain the stale-context rejection authority, if smoke needs score computation in the stub, or if native-only behavior becomes required for acceptance without an available native environment.

**Review mandate:** Verify (1) exact placement and disabled state covers managed-club plus context refresh/mismatch/error, (2) route-only composition derives the active ID/token and key/remounts by both, (3) deferred save-switch tests prove no stale render, feedback, invalidation, or create-only layout append during open/set/remove paths, (4) Rust remains authoritative for stale get/set/remove rejection, (5) edit/remove/re-create and saved layout/filter/sort/history retention remain exact, (6) AppTopBar and Settings production invalidations and named tests include Club DNA, (7) proportionate route/component/smoke tests use context-bearing mocks without computing scores, and (8) no Moneyball/Profile/cache or frontend score scope creep.

## Active work

**PR:** PR 1 — Add user-defined Club DNA scoring

**Commit:** Commit 5 — Resolve cached Club DNA in Search and Squad

### RED or removal proof

Add resolver, filter, Search, and Squad tests that fail while `club_dna` is unknown and no query can distinguish page-only from complete-cohort materialization. Add cold and warm complete-catalog timing proofs at the recorded query seams.

### Expected outcome

Search and Squad expose exact-version cached Club DNA values with page-scoped display, complete Search and exact managed-club sort/filter cohorts, strict null behavior, and measured warm interaction latency.

### Explicit exclusions

Cache schema or definition mutation changes, frontend metadata or UI, eager ingest or promotion, Moneyball, Player Profile, background progress or cancellation, and current-state documentation.

## Discoveries and replanning

- The direct read-time SQL plan was disproved by measured complete-catalog 2,000-player behavior. Search filter nearest-rank p95 was 1902 ms and Search sort p95 was 923 ms, both above the `<500 ms` stop condition. No roughly 180,000-player run was attempted. The failed code experiment was discarded cleanly.
- The developer explicitly chose the recommended lazy persistent cache. ADR-0023 now owns that durable decision and follows ADR-0019's disposable versioned cache pattern with Club DNA-specific definition ownership.
- The material replan invalidated Delivery fingerprint `eb82c2be41d53ec22d539a67dfdef25745fe8d8e3e16f694493d09eb4a2d4bc7`. Independent review cleared packets 3–8, the developer accepted the replan and re-invoked delivery, and the ledger records replacement fingerprint `8b255cb0d43d34e6023ffc26c1e194aaa24e765e99ae1ebca1a639c0810fade8`.
- Commit 3 is now the planning-artifact packet. Commit 4 adds v32, scoring, materialization, and invalidation. Commit 5 integrates cached values and proves page/full-cohort correctness and performance. Existing frontend foundation, Modal, and integration behavior moves unchanged to Commits 6–8.
- The frontend attribute catalogs remain duplicated between Player Profile grouping and player metric metadata. Commit 6 still consolidates them into a shared utility; Rust retains its independent authoritative catalog.
- `.wiki/TODO.md` remains factually correct and unchanged. There is no planned spec or BACKLOG disposition. `.wiki/ARCHITECTURE.md` and `.wiki/DESIGN.md` remain unchanged until implementation makes the new state true.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Add user-defined Club DNA scoring | Commit 1 — Record the approved feature plan | ddd4961e6d90ca24faa435955c6ae7eb5a716f0b | Recorded the reviewed schema 2 ledger and TODO activation. | `ledger_state.py`: runnable; `git diff --cached --check`: passed. | Not applicable | Clear | 0 | None. |
| PR 1 — Add user-defined Club DNA scoring | Commit 2 — Persist one save-owned Club DNA definition | d2682ee5c50cb99cd0b7f9facf5fd4f9060d5001 | Added migration v31 and context-bound Rust CRUD for one validated definition per save. | RED failed because v31 was absent; `./scripts/dev check-rust` passed 561 tests with 2 ignored; `./scripts/dev check` passed. | Pass | Clear | 0 | None. |
| PR 1 — Add user-defined Club DNA scoring | Commit 3 — Record the approved cache replan | 7cf5e5924af8a9c54852f5037e17ffe4b2c58cc0 | Recorded ADR-0023 and the reviewed lazy-cache packets after measured direct-SQL failure. | Both classifiers were runnable with the accepted fingerprint; staged diff and Markdown checks passed. | Not applicable | Clear | 0 | Replaced the direct-SQL packet after measured 2,000-player threshold breaches. |
| PR 1 — Add user-defined Club DNA scoring | Commit 4 — Add the versioned Club DNA score cache | Pending record | Added v32 definition versioning, pure scoring, bounded nullable materialization, and atomic definition and player-boost invalidation. | RED failed because the cache owner was absent; focused rollback tests passed; `./scripts/dev check-rust` and `./scripts/dev check` passed 569 tests with 2 ignored. | Pass | Clear | 2 | Review corrections removed repeated definition validation, strengthened batch and late invalidation rollback proof, and restored adjacent role-score rollback coverage. |

## Final validation

- `./scripts/dev test` — all frontend component, route, store, catalog, adapter, and IPC-mock tests pass and discover the new Club DNA tests.
- `./scripts/dev check` — Biome, TypeScript, full-tree secretlint, Rust format, Clippy, and all Rust tests pass, including v32 upgrade, definition/version invalidation, pure scoring, materialization scope, boost rollback, Search, filter, Squad, active-save, null, and performance guards.
- `./scripts/dev smoke` — Chromium proves the My Club create flow, Modal explanation/selection, and fixed table-column integration through the browser IPC stub.
- Inspect the exact feature diff with `git diff --check b573420893da93d91ddaee66ff9a4038f800b6d9...HEAD` and the delivery workflow's exact recorded commit set.
- For the deterministic complete-catalog 2,000-player proof, populate complete representative visible/goalkeeper, hidden, and personality JSON for every player. For each materially distinct Search filter, Search sort, and Squad sort, clear Club DNA rows and record cold first-use materialization duration separately. Prefill the current-version cache, then run 3 unmeasured warm-ups plus 20 warm measured queries. Compute nearest-rank p95 from sorted sample index 18 and require each p95 `<500 ms`.
- On a representative roughly 180,000-player environment, clear Club DNA rows and record cold first-use duration separately. Prefill current-version rows, then run 3 warm-ups plus 20 warm measured executions for each materially distinct Search filter/sort and Squad sort. Require nearest-rank p95 `<= about 200 ms`. If the environment remains unavailable, publication stops for explicit developer gap acceptance. Any measured warm breach requires replan and cannot be accepted as a gap.
- Manually verify first-use behavior in the native app. Record the visible duration and confirm the UI does not falsely appear warm or return partial filter/sort results while materialization is in progress. If cold work makes the app unusable rather than delayed, stop for a progress/cancellation architecture decision.
- Manually verify the native Modal at 1280×800 and 1600×900: full-catalog scrolling, keyboard selection, edit ↔ remove-confirmation transitions in one Modal, confirmation Cancel/Escape return, edit Cancel/Escape discard, pending-removal dismissal blocking, remove-error return path, successful-removal focus return, and no layout shift. Chromium does not replace this check.
- `./scripts/dev bridge-test` is outside the affected bridge path. `./scripts/dev mutate` remains unsupported. Neither may be reported as passed.

## Documentation impact

During this replan, add only ADR-0023 and its index entry; leave `.wiki/ARCHITECTURE.md`, `.wiki/DESIGN.md`, and `.wiki/TODO.md` unchanged. During feature reconciliation after implementation, update `.wiki/ARCHITECTURE.md` for v31/v32 definition persistence, lazy versioned Club DNA materialization, query scopes, and invalidation owners; update `.wiki/DESIGN.md` for the implemented My Club action and definition Modal; update `.wiki/TODO.md` for completion; preserve ADR-0023; and move this ledger to `.wiki/features/completed/club-dna.md`.
