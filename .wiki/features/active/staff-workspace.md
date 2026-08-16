# Staff Workspace

## Status

Active

## Intent

Turn the already extracted staff population into a first-class Staff workspace: searchable and configurable staff tables, comparable job-fit scores, a club-family overview, staff-specific profiles, and one tightly bounded staff development action.

## User-visible behavior

- The primary navigation includes **Staff**, opening `/staff` on the **Search** tab by default.
- Search covers every staff member in the effective current snapshot, including staff from the configured club family.
- Search supports the player-search interaction contract where it applies: URL-backed filters and sorting, flat AND/OR rules, bounded results, virtualization, and columns that can be added, removed, resized, and reordered.
- Search and My Staff retain separate configurable column layouts.
- Default columns are Name, Age / DOB, Nation, CA, PA, and all 20 staff job-fit scores.
- Job-fit scores are integer values from 0 through 100. Each is the rounded arithmetic mean of its required 1–20 staff attributes, scaled by five. A score is unavailable when any required attribute is unavailable.
- The workspace tabs are **Search** and **My Staff**. The selected tab is URL-backed and keyboard-operable.
- My Staff lists staff whose club belongs to any Senior, Reserves, or Youth source in the active save's configured Planner club family. It is an overview with sortable and configurable columns, not a second filter workspace.
- My Staff offers a per-row **Boost CA** action. It always requests +10, caps at PA and 200, and is unavailable at the cap.
- Activating a staff row opens `/staff/$uid` for that staff member in the effective current snapshot.
- A Staff Profile follows the player-profile frame with a compact staff summary above **Attributes** and **Role fit** panels. It has no pitch, position suitability, potential projections, or wonderkid action.
- The Staff Profile groups the 24 extracted staff attributes into **Coaching**, **Mental**, and **Knowledge** tabs. Role fit lists all 20 current job-fit scores without a position filter, ordered by score descending with unavailable scores last and catalog order breaking ties.
- The Staff Profile offers only **Boost CA**. The fixed +10 policy, cap, confirmation, feedback, and recovery behavior match the My Staff action.
- The save-scoped **Hide hidden info** preference is shared by player and staff profiles. When concealed, a Staff Profile omits PA and Boost CA because its availability and preview disclose PA; CA, current staff attributes, and current job-fit scores remain visible. Staff Adaptability remains visible because it is a normal current staff attribute in this profile, even though the bridge reads it from the shared person personality block.
- A successful boost updates FM and the effective current snapshot. An uncertain or unreconciled outcome requires Load Data before another player or staff boost.

## Invariants

- Staff Search never silently narrows to the user's club family.
- My Staff membership is derived in Rust from the entire configured club family; React does not submit club names or staff UIDs as a cohort.
- Current staff job-fit is attribute-based only. There is no potential score or staff progression projection.
- Missing required attributes yield no score; they are never treated as zero, and every formula that lists Authority requires it.
- `Authority` is the stable product/schema key for FM26's renamed Level of Discipline attribute.
- The FM26.3 layout reads `Authority` from `NPLO_ATTRS + 0x30` with the same ×5 encoding as adjacent staff attributes.
- Coach has eight columns: six paired outfield specialties, one Fitness score, and one Goalkeeping score using all three goalkeeping attributes.
- Staff scores are calculated during snapshot ingest and persisted per snapshot. Query paths do not parse JSON and recompute the full catalog per row.
- Existing schema-v7 snapshots remain valid historical data, but they cannot produce complete Staff scores. A schema-v8 Load Data run is required before the workspace claims complete scoring.
- Rust owns query validation, club-family membership, boost policy, source binding, persistence, and recovery state. The bridge alone owns process-memory writes.
- The bridge exposes only the closed fixed-increment staff action accepted by [ADR-0020](../../decisions/0020-action-specific-fm26-staff-ca-boost.md).
- Player and staff profiles read and write one active-save hidden-information preference. The preference remains a presentation control, not an authorization boundary, and Staff Search and My Staff keep their requested columns when it is concealed.
- Player Search, Squad, player boosts, and persisted player table layouts retain their current behavior.

## Non-goals

- Global staff quick search, staff hiring, firing, contract editing, or planner assignment.
- Potential staff role scores, attribute-growth simulation, or a general staff progression model.
- Arbitrary CA values, custom increments, other staff memory edits, or club-family batch boosts.
- Inferring My Staff from the human manager's single club when a configured family exists.
- Filtering the My Staff overview in this feature.
- Refactoring player query services into a speculative generic people-query framework.
- Backfilling complete scores into old snapshots without a fresh compatible dump.

## Current-state map

- Relevant components: `src/app/components/app-nav-rail.tsx`; `src/app/routes/search.tsx`; `src/app/routes/planner.tsx`; `src/app/routes/players.$uid.tsx`; `src/features/search/`; `src/features/squad/`; `src/features/player-profile/`; `src/components/player-table/`; `src/components/ui/player-metric-picker.tsx`; `src/stores/use-player-table-store.ts`.
- Data model: snapshot-owned `staff` rows already contain identity, nationality, gender, CA/PA, employment fields, and `staff_attributes_json`; no staff UI, query service, role-score table, or mutation command exists.
- Persistence and migrations: SQLite schema version 23 is current. `player_role_scores` is the closest persisted-score analogue. `planner_club_sources` owns the configured family. `snapshots.player_boost_recovery_required` currently names a recovery condition that staff writes must share. `saves.reveal_hidden_player_information` stores a save-scoped player-profile presentation preference that this feature must generalize without changing its value.
- Existing behavioral assumptions: Search uses URL-backed flat AND/OR filters with a 32-rule limit and bounded pages; configurable player tables persist separate Search and Squad layouts; Squad family reads match `planner_club_sources.club_name` against snapshot clubs. Player profiles read only the effective current snapshot and conceal PA, projected and potential values, hidden/personality values, and development actions when the save preference is off.
- Architectural seams: React invokes typed Tauri commands; Rust validates and queries SQLite; C# produces dump schema v7 and is the only process-memory reader/writer. The bridge currently indexes and mutates players only.
- Project validation commands: `./scripts/dev test`, `./scripts/dev check`, `./scripts/dev bridge-test`, and `CI=1 ./scripts/dev smoke`. Live staff-mutation proof requires the supported Windows FM build.
- Primary risks: the accepted Authority pin may be wrong; player-table behavior may regress while controls are shared; dynamic score filters may be expensive across a large staff population; snapshot-to-live-process writes may become stale; and FM/SQLite reconciliation may fail after a write.

## Feature architecture

Define one Rust-owned staff role catalog with 20 stable IDs and required attribute keys. Snapshot ingest parses each staff attribute map once, calculates strict current scores, and writes them to a new snapshot-owned `staff_role_scores` table. Staff query code resolves an allow-listed metric and filter catalog into parameterized SQL, with correlated indexed score lookups analogous to player Search.

The score catalog is:

| Score | Required attributes |
| --- | --- |
| Assistant Manager | ManManagement, JudgingPlayerPotential, JudgingPlayerAbility |
| Coach — Attacking Technical | Authority, Determination, Motivating, Attacking, Technical |
| Coach — Attacking Tactical | Authority, Determination, Motivating, Attacking, Tactical |
| Coach — Defending Technical | Authority, Determination, Motivating, Defending, Technical |
| Coach — Defending Tactical | Authority, Determination, Motivating, Defending, Tactical |
| Coach — Possession Technical | Authority, Determination, Motivating, Possession, Technical |
| Coach — Possession Tactical | Authority, Determination, Motivating, Possession, Tactical |
| Coach — Fitness | Authority, Determination, Motivating, Fitness |
| Coach — Goalkeeping | Authority, Determination, Motivating, GoalkeepingDistribution, GoalkeepingHandling, GoalkeepingReflexes |
| Set Piece Coach | Authority, Determination, Motivating, SetPieces, TacticalKnowledge |
| Loan Manager | ManManagement, JudgingPlayerPotential, JudgingPlayerAbility |
| Head of Youth Development | WorkingWithYoungsters, JudgingPlayerPotential, JudgingPlayerAbility |
| Scout | Adaptability, JudgingPlayerPotential, JudgingPlayerAbility |
| Director of Football | JudgingPlayerPotential, JudgingPlayerAbility, Negotiating |
| Technical Director | JudgingStaffAbility, Negotiating |
| Recruitment Analyst | DataAnalysis, JudgingPlayerAbility |
| Head Performance Analyst | DataAnalysis, Determination, JudgingPlayerAbility, TacticalKnowledge |
| Performance Analyst | DataAnalysis, TacticalKnowledge |
| Physio | Physiotherapy |
| Sports Scientist | SportsScience |

`Adaptability` is read from the shared person personality field for staff. `Authority` is read from the previously unmapped staff-attribute byte at `NPLO_ATTRS + 0x30` and decoded with the surrounding ×5 rule. Dump schema v8 publishes both stable keys. The app accepts v8 for new loads and retains existing ingested snapshots; it does not fabricate the missing fields for v7 data.

The frontend gets a staff-specific metric/filter registry and query types. Existing virtualized table chrome is minimally generalized so a caller supplies its metric catalog, row identity, cell renderer, optional row activation, and fixed action cells. Player registries and behaviors stay separate.

`/staff?view=search|my-staff` owns the workspace selection. Search owns filters, sort, and pagination in the URL. My Staff owns sort in the URL and derives its rows from an SQL `EXISTS` against all `planner_club_sources` for the active save. Separate `staff-search` and `my-staff` layout IDs own visible columns, widths, and ordering.

`/staff/$uid` reads one staff row and its persisted scores from the effective current snapshot. Rust returns the stable score catalog metadata with the detail DTO, as the player profile does for roles. React groups current attributes for presentation and never recalculates scores. The profile uses the player profile's compact summary and bounded two-panel workspace, but replaces the pitch with one current-score-ranked list and exposes no potential path.

Migration 26 renames `saves.reveal_hidden_player_information` to `reveal_hidden_information` while preserving every existing value. One generic setter owns that save-scoped preference. Both player and staff detail reads return it, and either profile invalidates both detail-query roots after a successful change. Concealment stays at the React render boundary because the complete DTO is not an authorization concern. The shared preference does not impose one cross-domain attribute classification: player Personality remains concealed, while staff Adaptability is a visible current staff attribute and keeps contributing to the visible current Scout score.

Boost CA follows ADR-0020: a staff-specific bridge operation and candidate index, exact-build capability, expected-value validation, verified readback and rollback, a shared player/staff mutation gate, then targeted SQLite reconciliation. Migration 25 renames the snapshot recovery flag to `boost_recovery_required`; compatibility code migrates the existing value without clearing it.

## Uncertainty register

### Known

- FM26 presents Authority as the renamed Level of Discipline concept.
- The current FM26.3 staff map has 22 attributes and omits Authority and Adaptability.
- The shared person layout already maps Adaptability at personality offset `0x70`.
- Offset `0x30` is the only unmapped byte between the contiguous SportsScience and Negotiating staff attributes. Existing dumps do not serialize it.
- Staff CA and PA use staff-specific offsets and cannot safely share the player candidate path.
- The configured Planner club family, rather than the manager's exact club, is the accepted My Staff scope.

### Assumptions

- Staff club-name matching can use the current exact-string Planner family contract; normalization beyond existing ingestion is not introduced here.
- My Staff keeps its per-row action for fast overview work, while the Staff Profile reuses the same confirmation and result behavior in its summary.
- The player profile's compact summary and two bounded analysis panels can host staff-specific content without a new page-shell pattern.
- The current page-size limits and 32-filter ceiling are suitable for Staff Search unless query measurements show otherwise.
- A 20-score default table will require horizontal scrolling but remains useful because role comparison is the page's primary purpose.

### Decisions

- Goalkeeping is one Coach score containing Authority, Determination, Motivating, and all three goalkeeping skills.
- Fitness is one Coach score. The remaining Coach scores are the six outfield skill-style pairs.
- Scout includes Adaptability.
- Search includes the full current staff population; My Staff includes the entire configured family.
- Scores are strict rounded means on the 0–100 scale and are persisted on ingest.
- Search and My Staff use distinct layouts; My Staff does not duplicate the filter editor.
- Staff CA boost is fixed at +10 and capped by PA and 200, with no player age rule.
- One recovery flag and mutation gate cover both player and staff writes.
- Staff profiles use Coaching, Mental, and Knowledge attribute tabs plus one current-score-ranked Role fit list with catalog-order ties. They have no pitch or potential-score controls.
- The hidden-information preference is one save-scoped profile preference shared by players and staff. Staff concealment hides PA and the profile Boost CA action but keeps all current staff attributes and scores, including Adaptability and Scout, and does not alter either staff table. This is a staff-specific visibility classification, not a claim that the shared person-block storage location makes Adaptability hidden on staff profiles.
- Staff Search and My Staff rows open Staff Profiles by click or Enter after the profile route is delivered.
- Keep two PRs. Staff Profile backend contracts belong to the risky data foundation, while its route reuses the same staff types, table activation, concealment, and boost UI already reviewed in the workspace PR. A third PR would split one shared UI surface without an independent merge or risk boundary.
- Implement Authority as the ×5 staff attribute at `NPLO_ATTRS + 0x30`. The developer accepts the risk that later direct evidence may disprove this pin; implementation does not require a prior spike or live comparison.

### Unknowns

- Query-plan measurements may require an additional staff-score index beyond `(snapshot_id, role_id)`.

### Risks

- If `NPLO_ATTRS + 0x30` is not Authority, every Coach and Set Piece Coach score will be plausible but incorrect. Correct the pin and regenerate affected snapshots if later evidence disproves the accepted mapping.
- A bridge schema bump requires the user to update the installed plugin and run Load Data before complete scoring appears.
- Generalizing player table controls could corrupt persisted layouts or row interaction semantics if migration tests are incomplete.
- Generalizing the hidden-information preference could leave player and staff profiles out of sync if migration, query invalidation, or context-key tests are incomplete.
- A live staff mutation can succeed before SQLite reconciliation fails; all later boosts must then fail closed until Load Data.

## Walking skeleton

The thinnest end-to-end slice is: a schema-v8 staff record with Authority from `NPLO_ATTRS + 0x30` and person-level Adaptability → one persisted Assistant Manager or Coach score → a bounded Rust staff query returning that score → `/staff` Search rendering it in the virtualized configurable table. My Staff and Boost CA build on that read path.

## Delivery plan

### PR 1 — Staff data foundation

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(staff): add scored staff data services`

**Branch:** `feature/staff-data-foundation` → **Base:** `main`

**Publication:** GitHub via `.github/pull_request_template.md`; merge with squash after strict required `check` passes.

**Feature close-out:** Not required

**CI repair rounds:** 0

**Purpose:** Establish the versioned extraction, persisted scoring, bounded list/detail queries, shared profile preference, and closed mutation contracts before the UI depends on them. This isolates the memory/schema risk and permits live Windows validation at the backend boundary.

**Depends on:** Current schema-v7 staff extraction, snapshot ingest, Planner club-family persistence, and action-specific player boost infrastructure.

#### Commit 1 — Extract complete staff scoring attributes

**Status:** Completed

**Provisional commit:** `feat(memory-read): extract staff scoring attributes`

**Work:** Extract Authority from the accepted FM26.3 `NPLO_ATTRS + 0x30` pin and Adaptability from the person block, publish dump schema v8, and validate the new contract through bridge and Rust fixtures.

**Out of scope:**

- Staff score formulas, SQLite score persistence, query APIs, UI, or writes.
- Runtime offset discovery, alternate Authority candidates, or dynamic field selection.

**Implementation packet:**

- Treat “Authority” as the stable FM26 name and document that it supersedes Level of Discipline terminology.
- Read Adaptability through the existing person personality layout, not the staff attribute block.
- Add Authority to `StaffAttributeEntries` at offset `0x30` from `NPLO_ATTRS` and decode it with the existing ×5 staff-attribute rule. This is an accepted implementation decision and does not require a spike or live proof first.
- Publish both values inside the existing staff `attributes` object and bump bridge/Rust dump validation to schema v8. Existing ingested snapshots stay readable from SQLite, while stale dump files receive the existing update-and-rescan rejection.

**Files and responsibilities:**

- `bridge/Layouts/IFmMemoryLayout.cs`, `bridge/Layouts/Fm263Layout.cs` — expose Authority at staff offset `0x30` and reuse the personality entry for Adaptability.
- `bridge/Extraction/StaffReader.cs`, `bridge/Extraction/StaffAttributeReader.cs` — compose staff-block attributes with the person-level Adaptability value.
- `bridge/Models/StaffRecord.cs`, `bridge/Output/DumpWriter.cs`, `bridge/Protocol/BridgeProtocol.cs` — preserve stable keys and publish schema v8.
- `bridge/Tests/StaffExtractionTests.cs`, relevant attribute/dump tests — prove decoded values, invalid reads, and serialized keys.
- `src-tauri/src/features/memory_read/dump_validation.rs`, `fixtures/golden_dump_v8.json` — accept and validate v8 with both keys and reject malformed values.
- `bridge/DUMP_SCHEMA.md`, `.wiki/ARCHITECTURE.md` only if the implemented schema contract becomes current in this commit — document v8 and the fresh-load requirement.

**Behavior and data flow:** A successful scan reads the staff block plus the person's Adaptability field, validates 1–20 values, writes the stable keys into the dump, and passes Rust validation. Unreadable values remain null. An old or malformed dump is rejected without replacing the current snapshot.

**Ordered implementation steps:**

1. Add RED bridge tests that place a ×5 Authority byte at staff offset `0x30`, plus tests for staff Adaptability, null handling, and v8 serialization.
2. Add a RED Rust v8 fixture and validation proof for both stable keys.
3. Make the smallest layout, reader, writer, and validator changes that turn the proofs GREEN.
4. Update the dump contract and run focused bridge and memory-read tests, then the commit gate.

**Tests and proof:** Expected RED is a staff extraction/serialization test missing both keys and Rust rejecting schema v8. GREEN proves that the fixture byte at staff offset `0x30` becomes Authority, Adaptability comes from the person block, nulls remain null, malformed schemas are rejected, and bad output does not replace good prior output.

**Patterns to verify:** Existing `Fm263Layout.PersonalityEntries`, player personality extraction, `StaffAttributeReader`, schema-v7 golden fixture, and previous schema-bump tests.

**Constraints and non-goals:** Preserve ×5 decoding and nullable semantics. Use only the accepted `0x30` Authority pin; do not add alternate offsets, broaden layout discovery, or alter player personality output.

**Dependencies and sequencing:** No Authority discovery step precedes production code. The installed bridge must be updated before a v8 Load Data run.

**Validation:** `./scripts/dev bridge-test`; focused Rust memory-read tests through `./scripts/dev test` where applicable; `./scripts/dev check`. A later comparison with FM is useful evidence but does not gate implementation or publication of this extraction decision.

**Stop conditions:** The fixture does not decode `NPLO_ATTRS + 0x30` through the normal staff attribute path; extraction systematically produces unreadable or out-of-range values because of an addressing defect; schema-v8 ingestion replaces a good snapshot on failure; or implementation requires changes to player fields. Lack of independent offset proof is not a stop condition.

**Review mandate:** Verify that the accepted `0x30` pin is applied from the staff-attribute base; person-versus-staff base addressing; ×5 and null handling; schema compatibility behavior; golden fixture coverage; and documentation accuracy.

#### Commit 2 — Persist staff job-fit scores

**Status:** Completed

**Provisional commit:** `feat(scoring): persist staff job scores`

**Work:** Add the 20-entry staff score catalog, strict scoring function, migration 24, and transactional ingest persistence.

**Out of scope:** Query APIs, frontend presentation, potential scores, weights, or score backfill for old snapshots.

**Implementation packet:**

- Add `staff_role_scores(snapshot_id, uid, role_id, score)` with a composite primary key, foreign key to staff with cascade delete, score nullability/range enforcement, and index `(snapshot_id, role_id)`.
- Keep role IDs stable and presentation-independent. Calculate `round(mean(attributes) * 5)` only when every required input is present and 1–20.
- Insert all calculable scores in the same ingest transaction as staff; a failed score insert fails that snapshot ingest without replacing the prior current snapshot.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — migration 24 and migration tests.
- `src-tauri/src/features/staff_scoring.rs` or a focused `features/staff/scoring.rs` — catalog and pure calculation.
- `src-tauri/src/features/snapshot/ingest.rs` — parse once and persist per staff row.
- Focused scoring and ingest tests — catalog count/formulas, rounding, missing/invalid values, replacement, and cascade behavior.

**Behavior and data flow:** Valid v8 staff attributes enter snapshot ingest, generate zero or more score rows, and commit atomically with the snapshot. Missing Authority suppresses every dependent score rather than changing its denominator.

**Ordered implementation steps:**

1. Add RED pure tests for representative single, paired, and multi-attribute formulas plus the 20-ID catalog.
2. Add RED migration/ingest tests for persistence and null omission.
3. Implement migration, catalog, and transactional insertion.
4. Measure a representative large-fixture ingest and run focused tests plus the commit gate.

**Tests and proof:** Cover all formula membership, Coach Goalkeeping's six inputs, Fitness's four inputs, exact rounding, one-attribute ×5 behavior, missing values, migration upgrade, cascade delete, and snapshot replacement safety.

**Patterns to verify:** `player_role_scores`, role-scoring ingest, migration test helpers, and snapshot transaction/replacement tests.

**Constraints and non-goals:** One current score per role; no `phase`; no potential cache; no JSON-time query calculation; no formula configurability.

**Dependencies and sequencing:** Requires commit 1's stable v8 keys. Migration 24 is reserved here.

**Validation:** Focused Rust scoring, migration, and ingest tests; `./scripts/dev check`.

**Stop conditions:** Formula ambiguity; catalog does not contain exactly 20 stable IDs; representative ingest regression is material; or persistence requires changing player role-score contracts.

**Review mandate:** Verify every attribute set against this ledger; strict missing-value behavior; integer rounding; migration constraints/index; transactionality; and ingest cost.

#### Commit 3 — Query scored staff pages

**Status:** Completed

**Provisional commit:** `feat(staff): query scored staff pages`

**Work:** Add bounded Staff Search and My Staff Rust/Tauri read APIs with allow-listed metrics, filtering, sorting, pagination, and club-family membership.

**Out of scope:** React UI, staff profile lookup, quick search, boosts, and generic people-query abstractions.

**Implementation packet:**

- Define staff summary/page/filter/sort DTOs and a registry covering identity, CA/PA, employment fields, extracted attributes, and 20 score metrics.
- Mirror Search's flat AND/OR parameterized filter grammar and limits without sharing player-only SQL fragments.
- Search selects all staff in the effective current snapshot. My Staff adds an `EXISTS` predicate matching staff club to every `planner_club_sources.club_name` for the active save.
- Return explicit no-current-snapshot and no-club-family states rather than interpreting them as an empty successful staff population.

**Files and responsibilities:**

- `src-tauri/src/features/staff/{mod.rs,commands.rs,filter.rs,query.rs}` — staff contract, validation, SQL, and tests.
- `src-tauri/src/lib.rs` or command registration owner — expose list/search commands.
- Existing Planner service/query helpers only if a narrow shared family predicate avoids duplicated semantics.

**Behavior and data flow:** React-supplied metric IDs/operators/values are validated against the staff catalog, compiled to bound SQL, restricted to the effective current snapshot, and returned in bounded pages. Requested score columns use indexed correlated lookups. My Staff applies the configured-family predicate server-side.

**Ordered implementation steps:**

1. Add RED service tests for full-population Search and multi-club-family My Staff.
2. Add RED filter/sort tests including score predicates, injection-shaped values, nulls, limits, and pagination.
3. Implement the smallest staff-specific query surface and register commands.
4. Inspect `EXPLAIN QUERY PLAN` on representative data, add only measured indexes, and run focused tests plus the gate.

**Tests and proof:** Prove all-family inclusion, non-family exclusion, Search inclusion of family staff, current-only semantics, requested dynamic metrics, 32-filter enforcement, stable tie-break sorting, empty/no-setup states, and bounded page sizes.

**Patterns to verify:** `features/search/{filter,query,commands}.rs`, `features/planner/squad.rs`, `features/player_metrics/resolver.rs`, and snapshot current-winner helpers.

**Constraints and non-goals:** Parameterized SQL only; no arbitrary column interpolation; no React-owned club scope; no query-time score formula; no default exclusion of club staff.

**Dependencies and sequencing:** Requires score table/catalog from commit 2.

**Validation:** Focused Rust staff query/filter tests; `./scripts/dev check`.

**Stop conditions:** Unbounded SQL, query plan scans score rows per candidate without usable indexes, conflict with current snapshot semantics, or need for a general query-engine refactor.

**Review mandate:** Inspect SQL allow-listing and binding; snapshot/save joins; family scope; null and pagination semantics; dynamic metric selection; and query plans.

#### Commit 4 — Add the closed staff CA bridge operation

**Status:** Completed

**Provisional commit:** `feat(memory-read): support staff CA boosts`

**Work:** Add the staff mutation candidate index, protocol operation, exact-build capability, verified fixed +10 write/readback, and rollback reporting.

**Out of scope:** Tauri command, SQLite reconciliation, UI, arbitrary increments, or batch mutations.

**Implementation packet:**

- Create a staff-specific candidate from successful live scans; do not infer staff from the player mutation index.
- Extend protocol v1 through optional action-specific fields/capability while preserving read compatibility.
- Accept only the named operation, staff UID, source request ID, and expected CA/PA. Compute and enforce the +10 target inside the trusted path.
- Advertise support only for an exact build after controlled live proof.

**Files and responsibilities:**

- Bridge mutation index/service and scan pipeline owners — retain staff candidates and execute typed writes.
- `bridge/Protocol/` and status/request models — closed operation and capability/result shape.
- Bridge unit tests — identity/source/value rejection, cap, write/readback, rollback, serialization, and operation separation.

**Behavior and data flow:** A serialized request resolves one live staff candidate from the producing scan, revalidates UID/CA/PA, targets `min(CA + 10, PA, 200)`, writes and reads back, and reports either verified success, proved no-write failure, or recovery-required uncertainty.

**Ordered implementation steps:**

1. Add RED tests for fixed increment, cap, non-staff rejection, stale source, mismatched expected values, readback, and rollback.
2. Implement the separate staff candidate and closed operation.
3. Keep scans and all mutation types under one serialized bridge path.
4. Run bridge tests and complete controlled exact-build live proof before capability support is enabled.

**Tests and proof:** Unit fakes prove all branches. Windows proof uses a disposable/accepted save, checks one +10 result and one PA-capped result in FM, then refreshes Load Data to confirm readback.

**Patterns to verify:** `PlayerMutationIndex`, `PlayerValueMutationService`, `PlayerBoostOperationService`, bridge request/status serialization, and exact-build capability gating.

**Constraints and non-goals:** No addresses leave C#; no player fallback; no arbitrary target/increment; PSS remains read-only; one request affects one staff member.

**Dependencies and sequencing:** Requires the proved staff layout from commit 1 and ADR-0020.

**Validation:** `./scripts/dev bridge-test`; `./scripts/dev check`; controlled Windows live proof.

**Stop conditions:** Staff identity cannot be revalidated; write target differs from fixed/capped policy; rollback is uncertain; exact-build proof fails; or protocol changes expose a general write primitive.

**Review mandate:** Verify candidate separation; source binding; duplicate person/player-staff handling; target calculation; typed offset use; rollback classification; serialization; and capability gating.

#### Commit 5 — Reconcile verified staff boosts

**Status:** Completed

**Provisional commit:** `feat(staff): reconcile verified staff boosts`

**Work:** Add the Rust staff boost command, shared mutation/recovery gate, targeted staff persistence, and cache-invalidation contract.

**Out of scope:** React controls, batch action, role-score recomputation, or any other staff field.

**Implementation packet:**

- Tauri accepts only `staff_uid`; Rust derives expected values and rejects capped/stale/non-current targets before writing a request.
- Migration 25 renames `player_boost_recovery_required` to `boost_recovery_required` while preserving existing true values. Player and staff commands use the same application gate.
- On verified success, update only matching current `staff.ca`; do not touch attribute-derived staff scores. On reconciliation failure, mark recovery-required and return the truthful partial outcome.

**Files and responsibilities:**

- `src-tauri/src/features/staff/commands.rs` and service tests — prepare, invoke, reconcile, and classify outcomes.
- `src-tauri/src/features/player/boost_gate.rs` and player command call sites — generalize naming without changing player policy.
- `src-tauri/src/db/migrations.rs` — migration 25 and preservation tests.
- `src-tauri/src/features/memory_read/` protocol models — typed staff request/result support.
- `.wiki/CONCEPT.md` and `.wiki/ARCHITECTURE.md` — record the implemented third exception and shared recovery boundary.

**Behavior and data flow:** Rust locks the shared gate, captures immutable current context, sends one source-bound bridge request, verifies the result, updates current staff CA transactionally, and returns the new value. Any uncertainty blocks future boosts until Load Data.

**Ordered implementation steps:**

1. Add RED Rust tests for +10/cap, non-current context, concurrent player/staff exclusion, verified success, and every reconciliation failure class.
2. Add RED migration tests preserving an existing player recovery requirement.
3. Implement the command, shared naming, protocol mapping, and targeted transaction.
4. Run player and staff boost suites together, then the commit gate.

**Tests and proof:** Prove no bridge call when capped or stale; exact request fields; success changes only the current matching staff row; historical rows and scores remain unchanged; player behavior is unchanged; recovery blocks both action families.

**Patterns to verify:** `features/player/{commands,service,boost_gate}.rs`, memory-read file protocol client, migration 20 recovery tests, and TanStack invalidation contracts documented in feature APIs.

**Constraints and non-goals:** No role-score recompute; no WebView target/increment trust; no cleared recovery flag except successful Load Data; no parallel write path.

**Dependencies and sequencing:** Requires commits 3 and 4. Migration 25 follows the score-table migration.

**Validation:** Focused Rust staff/player boost and migration tests; `./scripts/dev bridge-test`; `./scripts/dev check`.

**Stop conditions:** Existing recovery state cannot migrate losslessly; commands can overlap; a successful FM write cannot be represented truthfully after local failure; or player boost behavior changes beyond naming/shared serialization.

**Review mandate:** Inspect IPC trust boundary; current-context binding; shared gate; migration preservation; transaction targeting; recovery classifications; and player regressions.

#### Commit 6 — Query staff profiles with shared concealment

**Status:** Pending

**Provisional commit:** `feat(staff): query staff profiles`

**Work:** Add the current-snapshot Staff Profile read contract and make the existing save-scoped hidden-information preference apply to both player and staff profiles.

**Out of scope:** Staff React routes or components, table row navigation, potential staff values, profile boost controls, global staff quick search, or score recalculation.

**Implementation packet:**

- Add `get_staff(uid)` for one staff member in the active save's effective current snapshot. Return identity, employment and contract fields, current attributes, all 20 catalog-labelled job-fit scores, and the shared hidden-information preference.
- Migration 26 renames `saves.reveal_hidden_player_information` to `reveal_hidden_information` and preserves every existing `0|1` value. Rename the Tauri setter to the generic `set_hidden_information_revealed`; both profile families use the same active-save transaction.
- Keep complete staff and player DTOs at the command boundary. The preference controls React presentation only and must not become an SQL redaction or authorization rule.
- Preserve player-profile behavior exactly while changing names and invalidation contracts required by the shared preference.

**Files and responsibilities:**

- `src-tauri/src/db/migrations.rs` — migration 26, constrained-column preservation, upgrade, and rollback tests.
- `src-tauri/src/features/staff/{query.rs,commands.rs}` — Staff Profile detail model, catalog-ordered score loading, current-snapshot lookup, and command DTO.
- `src-tauri/src/features/player/{query.rs,commands.rs,service.rs}` — use the generic persisted preference and setter without changing player concealment behavior.
- `src-tauri/src/lib.rs` — register `get_staff` and the renamed setter.
- `src/features/player-profile/api/set-hidden-information-revealed.ts`, `src/app/routes/players.$uid.tsx`, and focused mocks/tests — switch the existing player profile to the generic command in the same trunk-safe commit without changing its UI behavior.
- Focused Rust profile and migration tests — prove active-save/current-snapshot isolation, not-found behavior, null preservation, score order, shared preference state, and unchanged player reads.
- `.wiki/ARCHITECTURE.md` only when the migration and commands become current — record the generic save preference and Staff Profile read boundary.

**Behavior and data flow:** A UID-only Tauri read resolves the active save and effective current snapshot, loads one staff row plus its persisted scores, and returns the current save preference. A setter call from either profile changes that one save; subsequent player and staff reads observe the same value. Missing snapshots or UIDs return the same explicit empty contract used by player profiles.

**Ordered implementation steps:**

1. Add RED migration tests that preserve revealed and concealed values while replacing the player-specific column name.
2. Add RED staff detail tests for current-snapshot lookup, complete nullable attributes, catalog-ordered scores, missing rows, and the shared preference.
3. Add a composed RED test that toggles the preference and observes the new state through both `get_player` and `get_staff` without changing either DTO's data values.
4. Implement the migration, generic setter, player Rust and React call-site rename, and staff detail command; then run focused profile/migration tests and the commit gate.

**Tests and proof:** GREEN proves lossless v23-to-v26 migration through planned migrations 24 and 25, unchanged player-profile semantics, one active-save preference visible from both detail queries, current-only staff lookup, stable 20-score order with nulls, and no backend redaction.

**Patterns to verify:** `features/player/{query,commands,service}.rs`, player profile migration-v23 tests, staff page query models from commit 3, `all_roles()` metadata merging in the player detail query, and effective-current snapshot helpers.

**Constraints and non-goals:** One persisted preference, not parallel player/staff toggles. Do not conceal Search, Squad, Staff Search, or My Staff metrics. Do not add staff potential projection or reuse player role metadata for staff scores.

**Dependencies and sequencing:** Requires commits 2, 3, and 5. Migration 26 follows the score-table migration 24 and shared recovery migration 25.

**Validation:** Focused Rust migration, player-profile, and staff-profile query tests; `./scripts/dev test src/app/routes/players.$uid.test.tsx`; `./scripts/dev check`.

**Stop conditions:** Existing preference values cannot migrate losslessly; player reads or setter semantics change; staff detail requires query-time score calculation; one profile can observe a different save preference from the other; or detail lookup bypasses effective-current snapshot selection.

**Review mandate:** Verify migration preservation; shared save scope; trunk-safe command renaming; current-snapshot isolation; complete/null-safe DTO mapping; stable score metadata; no backend redaction; and player-profile regression coverage.

### PR 2 — Staff workspace UI

**Status:** Awaiting prior PR merge

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(staff): add staff scouting workspace`

**Branch:** `feature/staff-workspace` → **Base:** `main`

**Publication:** GitHub via `.github/pull_request_template.md`; merge with squash after strict required `check` passes.

**Feature close-out:** Not run

**CI repair rounds:** 0

**Purpose:** Deliver the user-facing route, configurable Search and My Staff tables, Staff Profiles, and fixed staff actions after the backend contracts are merged and stable.

**Depends on:** PR 1 merged into `main`.

#### Commit 1 — Share configurable table controls

**Status:** Pending

**Provisional commit:** `refactor(tables): share configurable table controls`

**Work:** Generalize only the existing virtual table, header, metric picker, and layout primitives required for staff callers while preserving player behavior and stored layouts.

**Out of scope:** Staff route or data, player query changes, visual redesign, or a universal data-grid framework.

**Implementation packet:**

- Rename or parameterize player-specific presentation primitives only where staff requires it.
- Accept a caller-owned metric catalog/cell renderer, generic row key, optional row activation, and optional fixed action cells.
- Preserve Search row navigation and keyboard semantics. Callers without an activation callback must remain non-interactive; the later Staff Profile commit supplies activation to both staff tables.
- Extend the layout store with versioned `staff-search` and `my-staff` IDs without resetting Search/Squad layouts.

**Files and responsibilities:**

- `src/components/player-table/` or a minimally renamed `src/components/data-table/` — generic virtual table/header primitives.
- `src/components/ui/player-metric-picker.tsx` — caller-supplied labels/catalog.
- `src/stores/use-player-table-store.ts` and tests, or a narrowly generalized store — compatible layout persistence.
- Existing Search/Squad component tests — regression proof.

**Behavior and data flow:** Existing player callers pass their unchanged metric catalog and activation callback. New staff layout slots can be initialized independently. No backend behavior changes.

**Ordered implementation steps:**

1. Add characterization tests for current Search/Squad layouts, row activation, metric menus, reorder, resize, and persistence.
2. Refactor the smallest surface needed for a second row/metric type.
3. Add staff layout-ID tests without rendering the Staff page.
4. Run player table/search/squad suites and the commit gate.

**Tests and proof:** The RED proof is a compile/test-only second catalog caller that current player-bound types cannot express. GREEN retains all player assertions and independently persists the two staff layouts.

**Patterns to verify:** `virtualized-player-table.tsx`, `player-table-header.tsx`, `player-metric-picker.tsx`, `player-metrics.ts`, and layout-store migrations.

**Constraints and non-goals:** Do not unify Rust queries, rewrite styling, or add extension points beyond the Staff use case. Preserve storage keys or migrate them deterministically.

**Dependencies and sequencing:** PR 1 must define the staff metric IDs before this refactor finalizes its caller contract.

**Validation:** Focused frontend table/store/Search/Squad tests; `./scripts/dev check`; `CI=1 ./scripts/dev smoke` if shared table interaction changes affect browser flows.

**Stop conditions:** Player layouts reset; Search row activation/accessibility changes; a broad data-grid rewrite becomes necessary; or staff requirements cannot fit without coupling catalogs.

**Review mandate:** Check persisted-layout compatibility; generic type safety; optional row interaction semantics; virtualization measurements; keyboard access; and scope discipline.

#### Commit 2 — Add Staff Search

**Status:** Pending

**Provisional commit:** `feat(staff): add staff search workspace`

**Work:** Add the Staff navigation item, `/staff` route, accessible Search/My Staff tab shell, staff filters, configurable Search table, and all default score columns.

**Out of scope:** My Staff data panel, boost action, staff profiles, or global quick search.

**Implementation packet:**

- Default `view` to `search`; normalize invalid workspace/filter/sort state through the route's existing URL patterns.
- Define staff metric/filter metadata separate from player metrics and request only visible/sort/filter metrics.
- Use the default 25-column layout: Name, Age / DOB, Nation, CA, PA, then all 20 scores in catalog order.
- Render `—` for unavailable attributes/scores. If data predates schema v8 or complete scores are absent, explain that Update Bridge/Load Data is required; do not show zero.
- Preserve bounded/infinite paging, loading, empty, error, and replacement behavior.

**Files and responsibilities:**

- `src/app/routes/staff.tsx`, route tests, generated route tree — URL contract and workspace composition.
- `src/app/components/app-nav-rail.tsx`, shell routing tests — Staff navigation.
- `src/features/staff/{api,types,utils,components}/` — DTOs, keys/options, URL parser, registry, filters, tabs, and results panel.
- Shared configurable table caller — render staff values without row activation.

**Behavior and data flow:** URL state selects Search, filters, sort, and visible metrics; TanStack Query invokes the bounded staff command; the virtual table renders current staff and fetches further pages near the end.

**Ordered implementation steps:**

1. Add RED route/nav/tab and URL parser tests.
2. Add RED registry/filter/default-layout and results-state tests.
3. Implement API/types/state, then the smallest route/panel rendering the walking skeleton.
4. Add interaction tests and run frontend validation plus smoke.

**Tests and proof:** Cover default/invalid view, all 20 score headers/formulas IDs, AND/OR filter serialization, metric add/remove/move/resize, no row navigation, loading/empty/error/legacy-data states, bounded next-page fetching, and keyboard tabs/menus.

**Patterns to verify:** Search route/filter/results, Planner workspace tabs, Academy empty/setup states, app nav rail, and configurable player tables.

**Constraints and non-goals:** Search is the primary/default tab. No score color semantics beyond existing numeric table conventions. Horizontal scrolling is acceptable; fixed identity/action regions must remain usable.

**Dependencies and sequencing:** Requires PR 2 commit 1 and PR 1 query contracts.

**Validation:** Focused Staff route/component/util tests; existing Search/Squad table tests; `./scripts/dev check`; `CI=1 ./scripts/dev smoke`; manual 1280×800 and 1600×900 native-window checks including keyboard-only filtering and column menus.

**Stop conditions:** Default columns cannot be reached/accessed at supported viewport; URL state exceeds existing safe parser limits; legacy data is presented as scored zero; or shared table behavior regresses.

**Review mandate:** Verify default columns and labels; URL normalization; query-key completeness; accessibility; virtualization/scroll containment; empty/error/stale states; and no accidental profile semantics.

#### Commit 3 — Add the club-family staff overview

**Status:** Pending

**Provisional commit:** `feat(staff): add club-family staff overview`

**Work:** Populate My Staff with the full configured club-family query and its independent configurable table layout.

**Out of scope:** Filters, boost button, family editing, team grouping, or exact-manager-club fallback.

**Implementation packet:**

- Select `view=my-staff` through the existing accessible tabs and fetch the Rust-owned family result.
- Use the same default staff columns initially but persist changes under `my-staff` only.
- Use the same bounded virtual page-fetch flow as Staff Search so the overview can reach every matching staff member in a configured family larger than one page.
- When no family exists, link to Dashboard Club Setup. Distinguish no setup, configured family with no matching staff, load/error, and populated states.

**Files and responsibilities:**

- `src/features/staff/api/` — My Staff query keys/options.
- `src/features/staff/components/my-staff-panel.tsx` and tests — overview states/table.
- `src/app/routes/staff.tsx` and tests — view composition and URL transitions.

**Behavior and data flow:** The tab changes URL state, a distinct query key invokes the server-scoped family command, and the configurable virtual table requests later bounded pages near the end until it can render the full matching family across Senior, Reserves, and Youth clubs.

**Ordered implementation steps:**

1. Add RED UI tests for multi-club results and no-family recovery.
2. Add a RED bounded-paging test whose second page contains a staff member from another configured family club.
3. Implement the query binding, later-page fetch trigger, and separate layout caller.
4. Prove Search/My Staff layout independence and tab state retention.
5. Run focused tests, gate, smoke, and native-window checks.

**Tests and proof:** Include two or more configured clubs, a family larger than one bounded page with a visible later-page row, duplicate source-name behavior, empty configured results, Dashboard link, independent layouts/sorts, keyboard tab transition, and query invalidation after club-family changes.

**Patterns to verify:** Squad overview query keys/panel, Planner club setup states, and workspace tab panel attributes.

**Constraints and non-goals:** Rust remains authoritative for membership. Do not filter client-side or silently use only the senior/manager club.

**Dependencies and sequencing:** Requires Staff Search shell and PR 1 My Staff command.

**Validation:** Focused My Staff and route tests; `./scripts/dev check`; `CI=1 ./scripts/dev smoke`; manual configured three-club family check.

**Stop conditions:** Any configured family source or later bounded page is omitted; club-family changes leave stale rows; duplicate staff rows appear; or no-setup is indistinguishable from an empty club.

**Review mandate:** Check entire-family and later-page semantics; query keys/invalidation; duplicate handling; state distinctions; layout isolation; and tab accessibility.

#### Commit 4 — Add per-staff CA boost

**Status:** Pending

**Provisional commit:** `feat(staff): add staff CA boost action`

**Work:** Add the fixed per-row Boost CA control to My Staff with confirmation, cap preview, mutation feedback, focus restoration, and query invalidation.

**Out of scope:** Search-tab boosts, batch boosts, custom values, progress algorithms, or other staff mutations.

**Implementation packet:**

- Add a fixed Actions column outside configurable metrics. Disable Boost CA when `CA >= PA`; otherwise preview `CA → min(CA + 10, PA, 200)`.
- The mutation sends only the staff UID. Pending state prevents duplicate activation. Success invalidates Staff Search, My Staff, snapshot sanity, and any later staff detail roots through one staff key prefix; error text preserves whether Load Data is required.
- Confirmation and result UI follow existing player/squad boost language and modal focus behavior, adapted to one staff member.

**Files and responsibilities:**

- `src/features/staff/api/boost-staff-current-ability.ts`, staff keys/options — invoke and invalidation boundary.
- `src/features/staff/components/staff-ca-boost.tsx` and My Staff table integration — action, modal, feedback, and focus.
- Focused component/route tests and smoke mocks — cap, payload, pending/error/success, and cross-view refresh.

**Behavior and data flow:** The user activates one row action, confirms the fixed target, React sends only UID, Rust/bridge perform the closed operation, and success refreshes every staff presentation. Recovery-required outcomes direct the user to Load Data and leave later actions disabled by backend truth.

**Ordered implementation steps:**

1. Add RED interaction tests for fixed preview, cap, UID-only payload, pending lock, feedback, focus, and invalidation.
2. Implement API binding and per-row control.
3. Add recovery/error and cross-tab refresh coverage.
4. Run focused tests, full gate, browser smoke, and supported-build manual proof.

**Tests and proof:** Prove +10, PA cap, cap disablement, no age rule, Actions column not configurable, no Search action, UID-only invoke, double-click suppression, success/error announcements, focus restoration, and refreshed CA in both views.

**Patterns to verify:** Player profile boost panel, Squad boost confirmation/progress feedback, modal primitives, staff query-key root, and table fixed-column behavior.

**Constraints and non-goals:** No optimistic CA patch before verified success; no arbitrary values; no action at or above PA; no role-score invalidation requirement beyond broad staff query refresh.

**Dependencies and sequencing:** Requires My Staff panel and PR 1 boost command.

**Validation:** Focused Staff boost tests; related player/squad boost tests; `./scripts/dev bridge-test`; `./scripts/dev check`; `CI=1 ./scripts/dev smoke`; controlled supported-build Windows boost proof.

**Stop conditions:** Invoke payload contains a target/increment; action can run concurrently; recovery outcome permits another action; cross-view CA disagrees after success; or focus/announcement behavior fails.

**Review mandate:** Inspect trust boundary; fixed/capped preview; action availability; pending concurrency; recovery messaging; invalidation breadth; fixed-column accessibility; and player boost regressions.

#### Commit 5 — Add Staff Profiles

**Status:** Pending

**Provisional commit:** `feat(staff): add staff profiles`

**Work:** Add `/staff/$uid`, staff-specific summary, attribute tabs, current job-fit list, shared concealment control, the sole profile Boost CA action, and row entry from both staff tables.

**Out of scope:** A pitch, position familiarity, potential attributes or scores, Wonderkid Mentality, profile history, comparison charts, global staff quick search, or new memory-write operations.

**Implementation packet:**

- Follow the current player-profile frame: one compact summary above two bounded panels. Staff uses **Attributes** and **Role fit**; Role fit shows all 20 current scores in descending order, keeps unavailable scores last, uses catalog order for ties, and has no pitch or score-basis switch.
- The summary shows name, club/division, Age / DOB, nationality, wage, contract expiry, CA, revealed PA, and the best available job-fit score with catalog-order tie breaking. Missing values render `—` and no raw `jobId` is presented as a job title.
- Group the 24 current staff attributes into URL-backed tabs: **Coaching** (`Attacking`, `Defending`, `Fitness`, `GoalkeepingDistribution`, `GoalkeepingHandling`, `GoalkeepingReflexes`, `Possession`, `SetPieces`, `Tactical`, `Technical`), **Mental** (`Adaptability`, `Authority`, `Determination`, `ManManagement`, `Motivating`, `WorkingWithYoungsters`), and **Knowledge** (`DataAnalysis`, `JudgingPlayerAbility`, `JudgingPlayerPotential`, `JudgingStaffAbility`, `Negotiating`, `Physiotherapy`, `SportsScience`, `TacticalKnowledge`). Default and invalid tabs resolve to Coaching.
- Reuse the commit-4 Staff CA confirmation/action in the summary. It is the only development action, always previews the fixed capped +10 result, and is absent when hidden information is concealed.
- Put the same concealment control last in the summary action row. A successful toggle invalidates both `playerKeys.all` and `staffKeys.all`; either profile therefore reads the same active-save value. Concealment hides PA and the profile action only. It does not hide CA, current attributes, current job-fit scores, Staff Search columns, or My Staff columns. Test Adaptability and Scout explicitly so the staff-specific visible classification cannot drift with the player Personality rules.
- Enable click and Enter row activation in Staff Search and My Staff only when this route exists. Navigation uses route path only and preserves the originating table URL in browser history.

**Files and responsibilities:**

- `src/app/routes/staff.$uid.tsx`, route tests, and generated route tree — validated UID/tab state, loader/query wiring, snapshot/not-found states, shared mutations, invalidation, and bounded profile composition.
- `src/features/staff/api/{get-staff-query-options,staff-keys}.ts` and types — typed detail read and one query-key root shared by lists and details.
- `src/features/staff/components/{staff-overview-panel,staff-attributes-panel,staff-role-fit-panel}.tsx` — summary, concealment/action placement, exact attribute groups, and 20-score presentation.
- `src/features/staff/utils/staff-profile-tab.ts` and grouping tests — canonical URL parsing, labels, order, and attribute membership.
- `src/features/player-profile/components/player-profile-tabs.tsx` or a narrowly extracted shared profile-tab primitive — reuse the existing accessible Arrow/Home/End behavior without changing player tab semantics or IDs.
- Staff Search and My Staff table callers — supply `/staff/$uid` row activation after the route exists.
- `src/app/routes/players.$uid.tsx` and profile tests — invalidate the new staff query root after a successful concealment change so the shared preference cannot remain stale.
- Smoke IPC mocks and `e2e/smoke.spec.ts` — Staff Profile entry, concealment echo, and Boost CA flow.

**Behavior and data flow:** A table row navigates to the UID-only route. The loader resolves current snapshot and `get_staff`; React renders current staff data and persisted scores. The generic preference mutation sends only the explicit revealed state, then refreshes both profile families. The Boost CA action sends only the staff UID through the already delivered closed operation and refreshes every staff view after verified reconciliation.

**Ordered implementation steps:**

1. Add RED route tests for table entry, valid/invalid UID and tab state, no snapshot, not found, and the summary/two-panel skeleton.
2. Add RED component tests for every attribute group, all 20 role scores, descending score order, catalog tie breaking, nulls last, no pitch/potential/wonderkid content, and the sole Boost CA action.
3. Add RED shared-preference tests that toggle from a staff profile, invalidate both roots, conceal PA/action while preserving CA, Adaptability, and the Scout score, and verify the existing player route also invalidates staff details and continues to conceal player Personality.
4. Implement the detail API binding, route, presentation, row activation, and narrow shared tab behavior. Reuse the existing staff boost component and refactor only while focused tests stay green.
5. Run focused player/staff profile and table tests, the commit gate, smoke, and native-window checks.

**Tests and proof:** Prove current-only lookup presentation; route-only navigation by click and Enter from both tables; canonical tab URLs and keyboard operation; exact 24-attribute membership without duplicates; all 20 score rows ordered by current score with stable catalog ties and nulls last; `—` for nulls; PA/action concealment with CA, Adaptability, Scout, and all other current attributes/scores retained while player Personality stays concealed; one preference observed from both profile families; UID-only Boost CA payload; no Wonderkid/pitch/potential UI; pending/error/success/focus states; and profile/table CA agreement after success.

**Patterns to verify:** `src/app/routes/players.$uid.tsx`, all current `src/features/player-profile/` panels and utilities, Search/Squad row activation, `ScoreBadge` hero/card variants, the commit-4 staff boost component, and profile loading/empty-state tests.

**Constraints and non-goals:** Match the existing design tokens and minimum 1280×800 shell. Keep the two panels internally scrollable without nested page scrolling. React must display persisted Rust-owned staff scores and must not calculate job-fit formulas. Rust returns the shared preference, while React owns concealment presentation. Do not introduce a generic person-profile domain model or display numeric job IDs as labels.

**Dependencies and sequencing:** Requires PR 1 commit 6 and PR 2 commits 2 through 4. The profile route is the final UI commit and activates existing table rows only after the detail command and route are both available.

**Validation:** Focused Staff Profile, player-profile regression, shared-tab, table-entry, and boost tests; `./scripts/dev test`; `./scripts/dev check`; `CI=1 ./scripts/dev smoke`; manual 1280×800 and 1600×900 native-window checks with revealed/concealed states and keyboard-only entry/tabs/action.

**Stop conditions:** Staff and player profiles observe different save preferences; concealment leaves PA or a PA-derived action state visible; a table row navigates before the route is available; layout introduces page-level nested scrolling; scores are recomputed in React; numeric job IDs are presented as titles; or reuse requires a broad player-profile rewrite.

**Review mandate:** Verify current-snapshot and route semantics; exact attribute/score membership; shared preference with domain-specific Adaptability handling; invalidation; action-only surface; no potential/pitch/wonderkid leakage; keyboard/focus behavior; bounded layout; and player-profile regressions.

## Active work

**PR:** PR 1 — Staff data foundation

**Commit:** Reconcile verified staff boosts

### RED proof

Add Rust tests for UID-only staff boost preparation, fixed +10 and cap handling, stale/current-context rejection, shared player/staff mutation exclusion, verified bridge reconciliation, recovery-required outcomes, and migration preservation. They fail today because Rust has no typed staff bridge request/result or staff boost service, and the recovery column remains player-specific.

### Expected outcome

Rust accepts only a staff UID, derives the current snapshot and expected CA/PA under the shared mutation gate, sends the source-bound closed bridge request, and reconciles only a verified value into the matching current staff row. Migration 25 preserves and generalizes the recovery flag so any uncertain player or staff outcome blocks both action families until Load Data.

### Explicit exclusions

No React control, batch action, staff score recomputation, arbitrary target/increment, profile read contract, or unrelated player policy change belongs in the active commit.

## Discoveries and replanning

- Planning inspection found Adaptability already mapped on the shared person layout at `0x70`; staff extraction should reuse it rather than create a second offset.
- Planning inspection found no current Authority/Level of Discipline entry in the local or audited upstream staff map. The developer accepted the only unmapped contiguous staff slot, `NPLO_ATTRS + 0x30`, as Authority and accepted the risk of correcting it later if direct evidence disproves the mapping. This removes the planned spike and proof stop condition.
- The requested entire configured-family scope reuses Planner persistence and supersedes an exact-manager-club interpretation.
- Staff Profile extends the same feature rather than adding PR 3. PR 1 now owns its detail/preference contract, and PR 2 owns table entry plus the profile UI because those changes share query keys, row activation, concealment, and the existing staff boost component.
- The controlled exact-build proof on FM 26.3.2 verified one fixed `+10` result and one PA-capped result, then independently read both values back through a second full bridge scan. The durable bridge record keeps only aggregate CA/PA evidence, not identities or save contents.
- Repowise's local index matched the current HEAD, but its context query did not return usable output within the bounded inspection; direct repository evidence owns this plan.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Extract complete staff scoring attributes | `009c21f` | Schema v8 publishes nullable Authority from accepted staff offset `0x30` and person-level Adaptability; Rust validates the fixed 24-key staff contract | Sol Medium accepted after one fix round added full bridge serialization proof | Live FM comparison not run; accepted Authority pin remains empirically unverified |
| PR 1 | Persist staff job-fit scores | `94bc921` | Migration 24 adds snapshot-scoped staff score rows; one 20-role Rust catalog calculates and transactionally persists only complete current-ability scores | Sol Medium accepted after one fix round added successful replacement and 2,000-staff ingest proof | Repowise advisory index was stale; direct source and 423-test gate used |
| PR 1 | Query scored staff pages | `98cf8c5` | Bounded current-snapshot Staff Search and unfiltered configured-family My Staff APIs expose allow-listed identity, employment, 24 attribute, and 20 score fields with parameterized filters and stable pagination | Sol Medium accepted after one fix round removed My Staff filters and bounded raw requested-field input | Representative score lookup uses its composite primary key; Repowise advisory index was stale |
| PR 1 | Add the closed staff CA bridge operation | `1932350` | Separate live staff candidates feed one fixed +10, PA/200-capped protocol operation with source/UID/CA/PA revalidation, verified readback, rollback classification, and exact FM 26.3.2 capability | Sol Medium accepted after one fix round updated protocol docs and closed deterministic boundary-test gaps | Controlled live proof passed with +10 and PA-capped results plus independent rescan readback |
| PR 1 | Reconcile verified staff boosts | Pending record | UID-only Rust command derives fixed/capped values, serializes player/staff writes through one gate, reconciles only verified current staff CA, and shares the preserved snapshot recovery latch | Sol Medium accepted after one fix round added command-level recovery classification proof | No role-score recomputation; proven pre-write failures do not latch recovery |
| None | Planning only | Pending record | Ledger and ADR-0020 | Not applicable | None |

## Final validation

1. Run targeted frontend, Rust, and bridge tests recorded in every commit packet.
2. Run `./scripts/dev bridge-test`.
3. Run `./scripts/dev test`.
4. Run `./scripts/dev check`.
5. Run `CI=1 ./scripts/dev smoke` with Staff Search, My Staff, Staff Profile, navigation, table-to-profile entry, shared concealment, layout, and boost flows included.
6. On the exact supported Windows FM build, update/install the bridge, Load Data, verify all 20 score formulas on selected staff, verify the full configured family, and perform accepted +10 and PA-capped staff boosts from both My Staff and a Staff Profile with Load Data readback. Compare representative Authority and Adaptability values with FM when practical; this comparison can disprove the accepted pin but is not a release gate.
7. Toggle hidden information from both a player and staff profile in the same save and verify that both profile families echo the persisted state. Confirm Staff Profile conceals PA and Boost CA only, while Staff Search and My Staff retain their configured columns.
8. Inspect Staff at 1280×800 and 1600×900 in the native Tauri shell, including horizontal scrolling, column menus, reorder/resize, filter editing, table-to-profile navigation/back state, profile attribute tabs and bounded panels, revealed/concealed states, empty/error/setup states, keyboard tabs, modal focus, announcements, and loading stability.
9. Run a fresh feature-complete review, resolve all retained findings, then complete documentation reconciliation before publication.

## Documentation impact

During implementation and final reconciliation:

- Update `bridge/DUMP_SCHEMA.md` for schema v8 and its Authority/Adaptability keys.
- Update `.wiki/ARCHITECTURE.md` for persisted staff scores, Staff list/detail queries, the generic save-scoped hidden-information preference, family membership, and the shared boost recovery/write boundary after those behaviors exist.
- Update `.wiki/CONCEPT.md` to add the accepted staff boost exception, Staff workspace, and Staff Profile capability after implementation.
- Update `.wiki/DESIGN.md` for Staff navigation, Search/My Staff tabs, table defaults, Staff Profile layout and concealment, and action placement after the UI is final.
- Keep ADR-0020 as the durable rationale for the third closed memory-write action.
- Reconcile this ledger into `.wiki/features/completed/` only after both PRs and final validation are complete.
