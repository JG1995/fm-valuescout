# Club DNA

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** eb82c2be41d53ec22d539a67dfdef25745fe8d8e3e16f694493d09eb4a2d4bc7

## Intent

Let the user define one save-owned Club DNA score from selected Football Manager player attributes and use the fixed score in the existing Search and Squad tables.

## User-visible behavior

- My Club shows a **Define DNA** action next to **Save managed club**. The action is disabled until the active save has a managed club selection.
- The action opens a form Modal with the Player Profile's FM-style attribute groups. The user can select any number of attributes from the closed visible, goalkeeper, hidden, and personality catalogs. Save requires at least one selection.
- The Modal is the only explanation surface. It lists the selected attributes and states that Club DNA scales each selected 1–20 value by 5, gives every selected attribute equal weight, averages the values, and rounds to a whole 0–100 score.
- Each app save owns at most one definition with the fixed label **Club DNA**. A later managed-club change keeps the definition because the definition is not keyed to a club name.
- The user can create, view, edit or replace, and remove the definition. Removing it does not remove the fixed metric from saved table layouts, filters, sort state, or navigation history.
- A player receives Club DNA only when every selected attribute has a value. If one selected value is missing or null, or the active save has no definition, the metric is unavailable and renders `—`.
- On initial creation, the app appends `club_dna` once to the existing app-local Search and Squad layouts. Editing does not restore a column that the user removed. Re-creating a definition after deletion is a new creation and can append a missing column again.
- General Search can display, sort, and filter Club DNA. Squad can display and sort it. Both use the shared score presentation and retain unavailable values as null rather than zero.
- Hidden and personality attributes remain eligible even when profile concealment is on. Concealment is a presentation preference, not authorization.

## Invariants

- Rust and SQLite own definition validation, persistence, score derivation, filtering, and sorting. React never computes a Club DNA table score.
- Definition attribute IDs use the existing closed metric forms: `attr.<PascalCase>`, `hidden.<PascalCase>`, and `personality.<PascalCase>`. Goalkeeper attributes use `attr.*` because they are stored in `players.attributes_json`.
- A definition contains at least one unique supported attribute ID. The user maximum is the complete closed catalog; no lower implementation cap may reject selecting all supported attributes.
- The formula is deterministic: for `n` selected values, calculate `round((sum(value × 5)) / n)` as one integer from 0 through 100. The same definition and player JSON produce the same result in Search and Squad.
- Missing definition, missing key, explicit JSON null, or any non-integer selected value makes the complete score null. Partial averages and zero substitution are forbidden.
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
- Materialized Club DNA scores, a score cache, ingest-time computation, or changes to snapshot player data unless measured read-time behavior crosses the recorded stop condition.
- New table layout ownership, a new table component, or global removal of saved layout, filter, sort, or history state.

## Current-state map

- Relevant components: `src/app/routes/my-club.tsx::MyClubPageContent` composes the My Club header and `ManagedClubSelector`; `src/features/managed-club/components/managed-club-selector.tsx::ManagedClubSelector` owns the selector form and **Save managed club** action; `src/components/ui/modal/modal.tsx::Modal` owns dialog focus and dismissal; `src/features/player-profile/utils/attribute-groups.ts` owns the current FM-style frontend attribute grouping.
- Data model: `players` stores visible and goalkeeper values in `attributes_json`, hidden values in `hidden_attributes_json`, and personality values in `personality_json`. `managed_club_settings` has one save-keyed row but Club DNA has no current table or command.
- Persistence and migrations: `src-tauri/src/db/migrations.rs` ends at v30. Save-owned tables use `save_id REFERENCES saves(id) ON DELETE CASCADE`. `src-tauri/src/features/managed_club/service.rs` is the closest one-row-per-save CRUD and cascade analogue.
- Existing behavioral assumptions: most React query keys omit save IDs because app-shell and Settings context changes invalidate feature roots, but save-owned Club DNA cannot rely on invalidation alone. The established saves query exposes each active save's ID and immutable `contextToken`. Search filters and sort live in URL state. Search and Squad column IDs, order, and widths live app-locally in Zustand key `fm-valuescout-player-table-layouts`, version 5. `addColumns` validates IDs and appends only missing columns.
- Architectural seams: `src/utils/player-metrics.ts` and `src/components/ui/player-metric-picker.tsx` own the frontend fixed metric catalog and picker. `src-tauri/src/features/player_metrics/resolver.rs::MetricField` owns the independent Rust catalog, dynamic display expressions, and dynamic sort expressions. `src-tauri/src/features/search/filter.rs` owns trusted Search filter compilation. `src-tauri/src/features/search/query.rs` and `src-tauri/src/features/planner/squad.rs` own the current-snapshot Search and managed-club Squad query paths.
- Shared table adapters: `src/features/search/components/search-results-panel.tsx` and `src/features/squad/components/squad-overview-panel.tsx` request visible dynamic fields and render nullable values. `src/components/player-table/` owns table interaction. Existing dynamic DTO maps already carry nullable integers.
- Command boundary: `src/lib/tauri-client.ts` is the sole frontend invoke wrapper. `src-tauri/src/lib.rs` registers Tauri commands, and `src-tauri/src/features/mod.rs` registers feature modules.
- Project validation commands: `./scripts/dev test <targets>`, `./scripts/dev check-app`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev mutate` is unsupported and cannot be evidence.
- Primary risks: cross-language catalog drift, accepting an empty or unknown definition, partial averages, stale active-save Query data, re-adding a user-removed column on edit, removing saved state with the definition, and a correlated JSON aggregate that is too slow for a representative Search filter or sort.

## Feature architecture

Add a save-owned `club_dna_definitions` table with one row per save and one JSON array of canonical selected attribute IDs. A Rust `club_dna` feature owns get, upsert, remove, and closed-catalog validation. Upsert returns whether it inserted a new row so React can distinguish creation from edit without inferring from stale UI state. The row cascades with `saves`; no snapshot or managed-club foreign key exists.

The fixed `club_dna` metric extends the established player-metric boundary. `player_metrics::club_dna` owns the fixed score SQL expression. `MetricField` recognizes only that exact ID and delegates display and sort SQL to that metric-private owner. Search filter compilation delegates to the same expression and adds the existing integer comparison and null guard. The expression resolves the player's save through `snapshots`, reads that save's definition, resolves each canonical ID against the appropriate player JSON column, requires one non-null integer for every selected ID, scales by 5, averages, and rounds once. No definition produces an empty aggregate and therefore null. Search and Squad keep their current page, DTO, filter, and sort flows.

React adds one fixed integer metric with score presentation. It consolidates the already duplicated frontend attribute lists into a shared utility used by Player Profile grouping, player metric metadata, and the Club DNA form. The Club DNA feature owns typed IPC adapters, context-bearing Query data, local Modal draft state, create/edit/remove mutations, validation feedback, selected-attribute summary, and formula copy. Every get, set, and remove adapter sends the expected active save ID and immutable save context token. Rust verifies that the pair still identifies the active save before it reads or writes. `clubDnaKeys.definition(saveId, contextToken)` isolates cached definitions by save incarnation rather than relying on invalidation. The My Club route derives that context from `savesQueryOptions`, keys the Club DNA feature by the pair, closes and discards its draft when the pair changes, and keeps the trigger and submission disabled while managed-club or Club DNA state is refreshing, mismatched, or errored. Mutation callbacks compare their captured context with the current route context before they show feedback, invalidate current data, or append layouts. Rust remains the authoritative stale-context rejection seam. Only a current-context successful response with `created: true` calls `addColumns("search", ["club_dna"])` and `addColumns("squad", ["club_dna"])`; edit and remove never alter layouts.

This extends established save-owned SQLite and dynamic metric boundaries, so no ADR is warranted. ADR-0019 concerns minute-scale CA-to-PA projection for roughly 183,000 players and does not require caching for existing direct JSON metrics. The direct Club DNA expression remains provisional until the recorded performance proof passes. Crossing that threshold stops delivery for a cache/materialization decision and possible ADR rather than silently adding a cache.

## Uncertainty register

### Known

- Linear JAY-32 is the external work item. There is no planned feature spec to promote and no current Club DNA implementation.
- The approved definition is one fixed-label save-owned record, independent of managed-club name and current snapshot.
- All visible, goalkeeper, hidden, and personality catalog attributes are selectable with equal weight and no user maximum.
- Search supports display, sort, and filter. Squad supports display and sort. Both already carry nullable integer dynamic values.
- Existing raw attribute filters use direct `json_extract`. Their recorded upgrade trigger is about 200 ms p95 on a full roughly 180,000-player snapshot, with a 2,000-player Rust guard below 500 ms. Club DNA must measure Search filter and Search sort as separate query shapes, plus Squad sort if its SQL shape materially differs.

### Assumptions

- Canonical metric-style IDs are the narrowest persisted representation because they encode both the closed attribute and its JSON source without another schema column.
- SQLite `round` on positive values and the existing whole-score presentation match the required whole-number rounding. The Rust tests must characterize tie cases before UI integration.
- A route-owned action slot in `ManagedClubSelector` is sufficient to place **Define DNA** beside **Save managed club** without a cross-feature import.
- The existing `ScoreBadge` is the correct 0–100 presentation in both tables.

### Decisions

- Store only selected canonical attribute IDs. Do not persist name, club name, formula version, weights, or derived scores.
- Use direct read-time SQL derivation through the shared Rust metric boundary. Do not add a score cache or ingest work without measured evidence.
- Keep `club_dna` permanently valid in both catalogs. No-definition and post-removal queries return null rather than an unknown-field error.
- Let the backend report create versus replace. Do not let React infer creation from its Query cache.
- Consolidate the frontend attribute catalogs into a shared utility. Keep a separate Rust closed catalog because Rust is the authoritative trust boundary.
- Keep all explanatory copy and selected-attribute detail inside the form Modal. Tables show only the fixed metric score or `—`.

### Unknowns

- Native Tauri/WebView density and focus behavior for selecting the full catalog cannot be proved in headless Chromium.
- Representative full-cohort Club DNA p95 is not yet measured. Publication requires either the recorded representative measurements or an explicit developer decision that accepts the unavailable-environment gap; a measured threshold breach cannot be accepted as a gap.

### Risks

- A formula assembled independently in filter, display, and sort paths can drift. One Rust score-expression owner and cross-path tests must prevent it.
- A stored malformed or unsupported ID could turn every score null or make JSON extraction unsafe. Rust must validate all writes and migration constraints must preserve a valid JSON-array shape.
- A missing selected value can be mistaken for zero or omitted from the denominator. Tests must include missing keys, explicit nulls, mixed JSON sources, and a valid zero-free 1–20 case.
- The same app-local layouts serve every save. Creation on one save can expose the fixed metric on another save with no definition; the backend must return null without errors.
- Definition mutations can leave cached Search or Squad pages stale. Mutation success must invalidate both query roots.
- Invalidation alone cannot bind a late definition response to the save that requested it. Every get/set/remove request must carry the expected save ID and immutable context token, Rust must reject a pair that is no longer active, Query keys must include the pair, and current-context UI effects must ignore late prior-context results. App top-bar and Settings context paths must still invalidate the Club DNA root and their tests must prove that wiring.
- Automatic append can override user customization if edit is mistaken for create. The backend-created flag and store tests must distinguish create, edit, delete, and re-create.
- Correlated JSON aggregation can exceed the established table-interaction budget. The deterministic 2,000-player guard must use the complete selected catalog and complete representative JSON for every player, then run 3 warm-ups and 20 measured executions per relevant query shape and assert nearest-rank p95 below 500 ms. Representative roughly 180,000-player Search filter and sort measurements use the same run counts and require each nearest-rank p95 at or below about 200 ms. A threshold breach stops for replan/cache decision; an unavailable representative environment stops publication for an explicit developer gap decision.

## Walking skeleton

Migration v31 stores one validated definition for the active save; the fixed Rust `club_dna` metric derives one nullable score through the existing Search query; the frontend form creates the definition and appends the fixed metric to Search and Squad. Later packets complete the full filter/sort, edit/remove, active-save, layout-retention, and browser proofs without changing that path.

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

#### Commit 3 — Resolve Club DNA in Search and Squad

**Status:** Active

**Provisional commit:** `feat(club-dna): derive table metric in Rust`

**Work:** Add the fixed `club_dna` metric to Rust display, Search filter/sort, and Squad sort paths with strict null behavior and direct read-time derivation.

**Size assessment:** About 180–230 changed non-test implementation lines. The shared expression plus three established query seams may slightly exceed the soft target; keep them atomic because separate commits would temporarily give the same fixed metric inconsistent display, filter, or sort behavior.

**Out of scope:**

- React metric metadata, form UI, app-local layout changes, Player Profile, Moneyball, caching, generated columns, expression indexes, and ingest-time scores.

**Implementation packet:**

- Let one Club DNA SQL-expression owner map validated stored IDs to the three player JSON columns, require complete values, and produce a nullable integer. Reuse it from resolver and Search filter compilation so display, filter, and sort cannot drift.

**Files and responsibilities:**

- `src-tauri/src/features/player_metrics/club_dna.rs` — own the fixed score SQL expression and formula contract relative to a supplied player alias; keep the expression independent of WebView input.
- `src-tauri/src/features/player_metrics/mod.rs` — register the metric-private Club DNA expression module.
- `src-tauri/src/features/player_metrics/resolver.rs::MetricSource`, `MetricField::parse`, `sql_expression`, and `sql_sort_expression` — recognize only `club_dna` as a nullable integer dynamic metric and delegate to the shared expression.
- `src-tauri/src/features/search/filter.rs::FieldKind`, `resolve_field`, and `compile_rule` — compile integer Club DNA comparisons with the shared expression and an explicit null guard, including `neq`.
- `src-tauri/src/features/search/query.rs` — add integration tests for display, deterministic rounding, filter, sort, active-save definition changes, missing definition, mixed sources, and strict null behavior; production flow should remain the existing dynamic path unless evidence requires a narrow alias/context adjustment.
- `src-tauri/src/features/planner/squad.rs` — add Squad display/sort/no-definition integration coverage through the existing dynamic path and exact managed-club cohort.

**Behavior and data flow:**

- Search or Squad validates `club_dna` as a fixed metric. The SQL expression resolves the player's save from `players.snapshot_id`, reads that save's one definition, iterates its canonical IDs, selects each value from `attributes_json`, `hidden_attributes_json`, or `personality_json`, and returns null unless every selected item is an integer. For a complete row it averages `value × 5` and rounds once to an integer. Search uses the expression for requested display, integer filters, and sort. Squad uses it for display and sort. A missing definition or removed row leaves the metric valid and returns null, so persisted layouts and URL state do not cause unknown-field or SQL errors.

**Ordered implementation steps:**

1. Add RED resolver and integration tests for exact-ID acceptance, unsafe/unknown rejection, 1 and 20 boundaries, a half-step rounding case, equal weighting across all three JSON sources, missing key, explicit null, missing definition, and active-save isolation.
2. Implement one trusted expression owner with the minimum alias/context support needed by both `players` and `p` query aliases.
3. Extend `MetricField` and prove display and both sort directions remain deterministic when all or some values are null.
4. Extend Search filter compilation with the same expression and explicit null exclusion for all integer operators.
5. Prove Search display/filter/sort against the current snapshot and Squad display/sort against the exact managed-club cohort.
6. Add deterministic 2,000-player timing guards. Select the complete supported visible/goalkeeper, hidden, and personality catalog, populate every player with complete representative JSON for all selected keys, and measure Search filter and Search sort separately. Measure Squad sort separately if inspection shows a materially different SQL shape. For each relevant shape, run 3 unmeasured warm-ups followed by 20 measured executions, sort the 20 durations ascending, take the 19th duration (`sorted[ceil(0.95 × 20) - 1]`, zero-based index 18) as nearest-rank p95, and assert that p95 is below 500 ms in the Rust test environment.
7. On a representative roughly 180,000-player snapshot, run 3 warm-ups and 20 measured executions for Search filter and Search sort separately, record nearest-rank p95 for each, and require each p95 at or below about 200 ms. Include Squad sort if its SQL shape materially differs. Do not add a cache in this packet.
8. Run Rust validation and the full gate.

**Tests and proof:**

- RED: `MetricField::parse("club_dna")` fails before implementation; Search and Squad tests either reject the field or cannot return a score. A partial-average mutant must fail because a player with one missing selected value must return null and fail even `neq` filters.
- GREEN: assert `(10, 20)` produces `75`, a rounding tie uses the characterized whole-score rule, mixed visible/hidden/personality values produce one equal-weight result, and missing/null inputs produce `None`. Assert a no-definition display/sort request returns rows with null dynamic values and no query error. Assert filter/sort changes immediately after edit/remove without materialized invalidation work.
- Performance proof: the deterministic 2,000-player fixtures must give every player complete representative visible/goalkeeper, hidden, and personality JSON for the complete selected-attribute catalog. Search filter and Search sort each receive 3 warm-ups and 20 measured runs; Squad sort receives the same proof if its SQL shape materially differs. Compute nearest-rank p95 from the sorted 20 samples and assert every relevant p95 is below 500 ms in the Rust test environment. For a representative roughly 180,000-player snapshot, use 3 warm-ups and 20 measured runs for filter and sort separately, record nearest-rank p95, and require each relevant p95 at or below about 200 ms. If that environment or evidence is unavailable, publication stops for an explicit developer decision that accepts the recorded validation gap. Any measured threshold breach requires replan/cache decision and cannot be accepted as a gap.
- Add/modify: resolver, filter, Search query, and Squad query Rust tests, full-catalog timing fixtures, and nearest-rank timing helpers local to the tests. Deliberately retain golden dump fixtures and existing raw attribute, role, potential-role, current-snapshot, and managed-club tests because they protect independent contracts. Delete no fixtures, mocks, snapshots, helpers, caches, or compatibility paths.

**Patterns to verify:**

- `MetricSource::JsonInteger`, `MetricField::sql_expression`, and `compile_json_integer_rule` for trusted direct JSON metrics and null guards.
- `src-tauri/src/features/search/query.rs::attribute_filter_on_two_thousand_players_stays_interactive` for the current timing threshold.
- Current role and potential role integration tests for nullable dynamic DTO behavior, without copying their cache boundary.
- `src-tauri/src/features/planner/squad.rs::list_squad_players` for fixed managed-club membership and bounded pages.

**Constraints and non-goals:**

- Never accept a formula, JSON path, column name, or SQL fragment from React.
- Never average available values only and never coerce null to zero.
- Do not materialize, cache, index, or write scores during ingest.
- Keep the metric out of Moneyball mode and Player Profile.
- Preserve the 256 requested-field and 32 filter-rule bounds.

**Dependencies and sequencing:**

- Depends on Commit 2's table, validation, and service module. It must land before the frontend can expose `club_dna`.

**Validation:** `./scripts/dev check-rust` then `./scripts/dev check`

**Stop conditions:** Stop if direct SQL cannot share one formula across display/filter/sort, if SQLite/Rust rounding semantics cannot meet the approved deterministic rule, if no-definition requests error, if any deterministic 2,000-player relevant-shape nearest-rank p95 is 500 ms or more, if any representative roughly 180,000-player relevant-shape p95 exceeds about 200 ms, or if correctness requires a cache/materialization boundary. A measured breach requires replanning and may cross the ADR threshold; it cannot become an accepted gap. If the representative environment or evidence is unavailable, stop publication for an explicit developer decision on the recorded gap.

**Review mandate:** Verify (1) one formula owner across all query paths, (2) complete-value null semantics including `neq`, (3) exact equal weighting and one rounding step, (4) active-save/snapshot resolution, (5) fixed-ID SQL safety, (6) Search filter/sort and Squad sort correctness with no definition, (7) deterministic full-catalog fixtures plus 3-warm-up/20-sample separate-shape nearest-rank p95 proof, and (8) unavailable representative evidence and measured breaches follow their distinct stop paths without hidden caching.

#### Commit 4 — Add the frontend Club DNA domain and fixed metric

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

- Frontend callers receive an explicit context from the established saves query and can fetch that context's nullable definition or invoke set/remove with the same save ID and immutable token. The definition Query key contains both values, so save A data cannot satisfy save B or a replacement save incarnation. Adapters return results to their caller but do not infer currentness; Commit 5/6 UI compares the captured context before applying results, while Rust rejects a context that is no longer active. The shared catalog exposes canonical IDs and FM grouping but no score function. The fixed `club_dna` metadata flows through the existing picker, filter registry, sort validation, requested-field adapter, nullable dynamic DTO, and table cell. Search and Squad render a backend-supplied integer with `ScoreBadge`; null stays `—`.

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

- Depends on Commit 3 so every exposed metric request is already accepted by Rust. The later Modal and route packets consume these adapters and shared groups.

**Validation:** `./scripts/dev test src/features/player-profile/utils/attribute-groups.test.ts src/utils/player-metrics.test.ts src/features/club-dna/api/club-dna-api.test.ts src/features/search/utils/dynamic-columns.test.ts src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx src/app/routes/my-club-squad.test.tsx`; `./scripts/dev check-app`; `./scripts/dev check`

**Stop conditions:** Stop if catalog consolidation changes a current Player Profile group, if a supported frontend ID has no exact Rust counterpart, if Query/API adapters cannot bind get/set/remove to `{ saveId, contextToken }`, if the fixed metric requires a new table component or store owner, if store retention requires resetting user layouts, or if React would need to compute a score.

**Review mandate:** Verify (1) exact catalog parity and no cross-feature import, (2) Player Profile grouping stays unchanged, (3) fixed label/ID and integer filter metadata, (4) no default layout insertion and persisted layout retention, (5) ScoreBadge/null presentation in both tables, (6) context-bearing definition key isolates save IDs and tokens, (7) all typed invoke requests carry the exact expected context, and (8) no frontend score, stale-context authority, or validation duplicates Rust.

#### Commit 5 — Build the Club DNA definition Modal

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

- Depends on Commit 4's typed API and shared catalog. Commit 6 provides route placement and cross-query/layout effects.

**Validation:** `./scripts/dev test src/features/club-dna/components/club-dna-definition.test.tsx`; `./scripts/dev check-app`; `./scripts/dev check`

**Stop conditions:** Stop if the shared Modal cannot support keyboard access, full-catalog scrolling, the one-Modal edit ↔ confirmation transition, pending dismissal guards, or reliable focus return; if a context change cannot close/discard the draft and suppress late prior-context results; if product behavior requires an explanation outside the Modal; or if component state cannot distinguish backend create from edit.

**Review mandate:** Verify (1) full approved catalog and no maximum, (2) minimum-one/current-context guards plus Rust stale-context authority, (3) selected summary and exact formula copy only in Modal, (4) context change closes/discards and late get/set/remove results cannot update current UI, (5) the Planner-style single Modal preserves draft across confirmation and implements exact Cancel/Escape/pending/error transitions, (6) keyboard/focus/accessibility behavior including successful-remove focus return, (7) no score computation or profile surface, and (8) deferred mocks test observable stale-result suppression without duplicating Rust mutation authority.

#### Commit 6 — Integrate Club DNA with My Club and layouts

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

- Depends on Commits 2–5. It is the final implementation packet and moves the feature to Validation only after its full checkpoint clears.

**Validation:** `./scripts/dev test src/features/club-dna/components/club-dna-definition.test.tsx src/app/routes/my-club-squad.test.tsx src/app/app-top-bar.test.tsx src/app/routes/settings.test.tsx src/stores/use-player-table-store.test.ts src/app/routes/search.test.tsx`; `./scripts/dev test`; `./scripts/dev check`; `./scripts/dev smoke`

**Stop conditions:** Stop if placement requires a cross-feature import, if My Club cannot derive and key the feature by the saves query's active ID/token, if create-versus-edit cannot be based on the backend result, if a save change can leave a draft open or allow prior-context data/feedback/layout append, if managed-club or Club DNA refresh/mismatch/error cannot disable interaction, if remove requires deleting catalog/layout/URL state, if Rust cannot remain the stale-context rejection authority, if smoke needs score computation in the stub, or if native-only behavior becomes required for acceptance without an available native environment.

**Review mandate:** Verify (1) exact placement and disabled state covers managed-club plus context refresh/mismatch/error, (2) route-only composition derives the active ID/token and key/remounts by both, (3) deferred save-switch tests prove no stale render, feedback, invalidation, or create-only layout append during open/set/remove paths, (4) Rust remains authoritative for stale get/set/remove rejection, (5) edit/remove/re-create and saved layout/filter/sort/history retention remain exact, (6) AppTopBar and Settings production invalidations and named tests include Club DNA, (7) proportionate route/component/smoke tests use context-bearing mocks without computing scores, and (8) no Moneyball/Profile/cache or frontend score scope creep.

## Active work

**PR:** PR 1 — Add user-defined Club DNA scoring

**Commit:** Commit 3 — Resolve Club DNA in Search and Squad

### RED or removal proof

Add resolver and query tests that fail while `club_dna` is unknown. The tests must prove one formula across display, filter, and sort; strict null behavior; deterministic rounding; active-save isolation; and full-catalog performance.

### Expected outcome

Rust derives one nullable Club DNA score through the existing Search and Squad metric paths, with trusted SQL, equal weighting, one rounding step, and measured interactive query behavior.

### Explicit exclusions

Frontend metric metadata, form UI, app-local layout changes, Player Profile, Moneyball, caching, generated columns, expression indexes, and ingest-time scores.

## Discoveries and replanning

- Source evidence confirms direct raw JSON metrics already display, filter, and sort through `MetricField` and Search filter compilation. ADR-0019 applies to expensive CA-to-PA projection, not every derived table expression. Club DNA therefore starts as a direct read-time metric with an explicit performance stop condition.
- The frontend attribute catalogs are duplicated between Player Profile grouping and player metric metadata. The plan consolidates them into a shared utility so the Club DNA form cannot silently omit a currently supported attribute; Rust retains its independent authoritative catalog.
- No planned spec, accepted BACKLOG non-goal, or ADR needs a planning-path change. The first commit contains exactly this ledger and `.wiki/TODO.md`.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Add user-defined Club DNA scoring | Commit 1 — Record the approved feature plan | ddd4961e6d90ca24faa435955c6ae7eb5a716f0b | Recorded the reviewed schema 2 ledger and TODO activation. | `ledger_state.py`: runnable; `git diff --cached --check`: passed. | Not applicable | Clear | 0 | None. |
| PR 1 — Add user-defined Club DNA scoring | Commit 2 — Persist one save-owned Club DNA definition | Pending record | Added migration v31 and context-bound Rust CRUD for one validated definition per save. | RED failed because v31 was absent; `./scripts/dev check-rust` passed 561 tests with 2 ignored; `./scripts/dev check` passed. | Pass | Clear | 0 | None. |

## Final validation

- `./scripts/dev test` — all frontend component, route, store, catalog, adapter, and IPC-mock tests pass and discover the new Club DNA tests.
- `./scripts/dev check` — Biome, TypeScript, full-tree secretlint, Rust format, Clippy, and all Rust tests pass, including migration, service, formula, Search, filter, Squad, active-save, null, and performance guards.
- `./scripts/dev smoke` — Chromium proves the My Club create flow, Modal explanation/selection, and fixed table-column integration through the browser IPC stub.
- Inspect the exact feature diff with `git diff --check <accepted-base>...HEAD` and the delivery workflow's exact recorded commit set.
- For deterministic 2,000-player Rust guards, select the complete supported attribute catalog and populate complete representative visible/goalkeeper, hidden, and personality JSON for every player. Run 3 warm-ups plus 20 measured executions for Search filter and Search sort separately, and for Squad sort if its SQL shape materially differs. Compute nearest-rank p95 from each 20-sample set and require every relevant p95 below 500 ms.
- On a representative roughly 180,000-player snapshot, run 3 warm-ups plus 20 measured executions for Search filter and Search sort separately, and Squad sort if materially different. Record nearest-rank p95 and require each relevant value at or below about 200 ms. If the representative environment or evidence is unavailable, publication stops for an explicit developer decision that accepts the recorded gap; this is not an automatic pass. A measured breach always requires replan/cache decision and cannot be accepted as a gap.
- Manually verify the native Modal at 1280×800 and 1600×900: full-catalog scrolling, keyboard selection, edit ↔ remove-confirmation transitions in one Modal, confirmation Cancel/Escape return, edit Cancel/Escape discard, pending-removal dismissal blocking, remove-error return path, successful-removal focus return, and no layout shift. Chromium does not replace this check.
- `./scripts/dev bridge-test` is outside the affected bridge path. `./scripts/dev mutate` remains unsupported. Neither may be reported as passed.

## Documentation impact

Complete during reconciliation. Expected current-state owners after implementation are `.wiki/ARCHITECTURE.md` for v31 save-owned Club DNA persistence and the dynamic metric path, `.wiki/DESIGN.md` for the implemented My Club action and definition Modal, `.wiki/TODO.md` for completion state, and this ledger moved to `.wiki/features/completed/club-dna.md`. Create no ADR unless measured evidence forces a consequential cache/materialization decision.
