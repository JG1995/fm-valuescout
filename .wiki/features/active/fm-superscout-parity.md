# FM SuperScout Reader Parity

## Status

Active

## Intent

Bring the production FM26 reader to practical direct-data parity with the permitted FMSuperScout memory plugin. Add the player metadata, staff records, human-manager metadata, database-scope semantics, and complete club discovery that the current ValueScout schema does not preserve. Store the new data for later product features without adding UI now.

After the data contract is stable on `main`, adopt the useful FMSuperScout scan safeguards that can prove semantic parity and a practical benefit in this repository. Keep that operational work separate so native-memory concurrency or snapshot retry can be removed without blocking the data expansion.

## User-visible behavior

- **Load Data** continues to create one current snapshot for the active app save, but the stored snapshot can also contain the remaining direct player metadata, complete staff records, the human manager and managed club, the selected player database scope, and an honest date basis.
- Existing Search, Player, Planner, Optimizer, and Youth Academy screens keep their current behavior. This feature adds no staff UI, gender control, or new player filters.
- Production requests default to the men's player database. The file protocol can also request the women's database or both for developer and future product use.
- A later hardening PR may make full scans faster and more resilient. It must not change accepted players, staff, manager, clubs, field values, ordering, or replacement semantics.
- Unsupported FM builds, failed scans, and materially incomplete reads fail clearly and preserve the prior good dump and app snapshot.

## Invariants

- The bridge remains read-only. It must not write FM memory or invoke game mutators.
- Direct-data parity means memory-backed values that the pinned FMSuperScout plugin actually emits or must identify to emit them. It does not mean copying its JSON shape, UI, estimates, labels, or derived models.
- Every accepted layout pin must remain versioned under `Layouts/`. Unsupported or unverified game builds fail closed.
- Process addresses, raw memory, machine-local paths, and personal live artifacts must never enter `dump.json`, SQLite, Git, or user-facing errors.
- Player/staff dual-role people have one deterministic result: a player facet wins over a staff duplicate with the same UID. A human manager may also have one staff record, but top-level manager metadata must identify the same UID.
- Club candidates require both the expected bounded team-vector shape and a plausible club name. Contract-derived clubs remain the safe fallback when global discovery cannot resolve a stronger squad association.
- A request-scoped player cap may stop the object scan early for diagnostics. When it does, `scanTruncated` describes every emitted entity family, not players alone.
- Player database scope uses the closed values `men`, `women`, and `both`. The selected value is recorded in the dump and snapshot. Staff discovery follows the audited upstream behavior and is not silently filtered by player gender.
- The team-schedule date is a next-fixture consensus. It is derived evidence, not an exact world-clock read. The dump and snapshot must record that basis without breaking Youth Academy year handling.
- Dump schema v6, Rust validation, the golden fixture, SQLite migration, and transactional ingest change together. A dump must not validate and then silently discard a promised field.
- Staff records are snapshot-owned and cascade with snapshot replacement. Staff attributes use stable English keys in one validated JSON object until a staff feature needs a different query shape.
- Existing player query DTOs and product screens must not expose or depend on new fields during this feature.
- Scan hardening must merge results deterministically and preserve the serial semantic baseline. Worker scheduling must not affect deduplication, ordering, counts, diagnostics, or field values.
- Worker count, scan buffers, retry count, and snapshot lifetime have fixed bounds. Every native handle and rented buffer has one owner and is released on success, failure, cancellation, and exception.
- A materially incomplete scan or retry must not replace a prior good `dump.json` or current SQLite snapshot.

## Non-goals

- Reading Youth Academy career appearances, goals, assists, international caps, or Moneyball match statistics.
- Carrying any memory-probe code or plan from `feature/fm-memory-research-probe` onto this branch.
- Validating or shipping constants that FMSuperScout declares but does not emit, including condition, morale, player home reputation, the second value slot, squad number, and staff reputation.
- Emitting an asking-price field that duplicates market value, an always-null wage-demand field, localized staff job names, or formatted duplicate position strings.
- Porting FMSuperScout snapshot history, growth, intake radar, estimates, transfer-interest models, meta scores, potential projections, coaching formulas, or app UI.
- Adding staff search, staff profiles, manager UI, gender controls, database-scope controls, or new player filters.
- Persisting a separate club catalog when club discovery is only needed to resolve people and manager metadata.
- Supporting FM editions, platforms, or game builds outside the existing Windows Steam FM26 boundary.
- Promising a performance improvement without comparable measurements.

## Current-state map

- Relevant components: `bridge/Scanning/PersonScanner.cs` performs one serial block scan and returns player candidates only; `bridge/Scanning/CapADumpPipeline.cs` extracts players, resolves contract-seeded clubs, and writes schema v5.
- Layout and extraction: `bridge/Layouts/Fm263Layout.cs` already contains the player pins shared with FMSuperScout. Dedicated readers under `bridge/Extraction/` own identity, attributes, contracts, clubs, squads, and schedule-derived dates.
- Data model: `bridge/Models/DumpDocument.cs` contains players only. It has no staff record, manager metadata, player nation UID, gender, club reputation, raw team type, database scope, or date basis.
- Persistence and migrations: dump schema v5 is validated in `src-tauri/src/features/memory_read/dump_validation.rs`; migration v14 is current; `src-tauri/src/features/snapshot/ingest.rs` stores snapshots, players, and role scores transactionally.
- Existing behavioral assumptions: production Load Data is unlimited by default; a positive player cap is diagnostic; successful writes use temporary files and atomic replacement; ingest rollback preserves the prior current snapshot.
- Existing scan performance: the completed bridge performance work recorded about 26 seconds for an unlimited reference scan of about 181,000 players. The current scanner is serial and already subdivides failed large reads to recover readable pages.
- Current scan gaps: read-call totals are diagnostic, but the pipeline does not classify overall readable coverage. It has no deterministic parallel merge, adaptive worker bound, or PSS VA-clone retry.
- Architectural seams: C# owns process memory, layouts, extraction, and dump writing. Rust owns dump validation, SQLite migration and ingest, and app-side request orchestration. React remains out of scope.
- Project validation commands: `./scripts/dev bridge-test`, `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev bridge-install`.
- Upstream provenance: [FMSuperScout commit `0f270d39`](https://github.com/mavarobli/FMSuperScout/commit/0f270d39a9cdc850ddfe653710d4904f13709cb5) was audited on 2026-08-08. Its `plugin/` tree is unchanged from the earlier `4ec3c657` audit. Author permission is recorded in `.wiki/notes/superscout-permission.md`.
- Primary risks: false non-player or club classification, incomplete staff coverage, cross-language schema drift, large-dump resource cost, nondeterministic parallel results, and native snapshot-handle leaks or memory pressure.

### Parity inventory

| Area | Already in schema v5 | Add in this feature | Explicitly excluded |
| --- | --- | --- | --- |
| Player | UID, identity, DOB/age, nation names, CA/PA, height, foot, positions, visible/hidden/personality attributes, contract, transfer flags, market value, current/world reputation, clubs, loan state, division, derived team level | Primary nation UID, gender, selected-team club reputation, raw team type | Duplicate asking price, null wage demand, condition, morale, home reputation, second value slot, squad number |
| Staff | None | UID, identity, DOB/age, nation name and UID, gender, CA/PA, 22 attributes, job ID, wage, expiry, club, division | Localized job labels, staff reputation, coaching formulas, staff UI |
| Manager and save | Game/build metadata and schedule-based date | Manager UID/name, managed club/reputation, player database scope, staff count, explicit date basis | Exact world-clock claim, currency duplicates, manager UI |
| Club discovery | Contract-seeded clubs and squad resolution | Same-pass global club candidates with bounded structural validation | Persisted club catalog or unbounded club/name scan |
| Scan operation | Serial block scan, partial-page recovery, cancellation, atomic output | Read-quality gate, deterministic bounded workers, optional one-shot VA-clone retry | User tuning, unlimited retries, semantic changes |

## Feature architecture

```text
Supported FM26 process
  -> one typed object scan
       -> players
       -> pure staff and human managers
       -> structurally verified clubs
  -> dedicated player, staff, manager, contract, club, and squad readers
  -> schema-v6 dump with explicit counts, scope, and date basis
  -> Rust validation
  -> one SQLite transaction for snapshot, players, staff, and role scores

Stable schema-v6 semantic baseline on main
  -> read-quality accounting
  -> bounded deterministic region workers
  -> optional one-shot PSS VA-clone retry for materially incomplete live reads
  -> identical schema-v6 values and replacement behavior
```

The feature keeps the existing bridge and Rust boundary from ADR-0016. It ports field provenance and bounded algorithms into ValueScout's existing structure instead of vendoring FMSuperScout or adopting its product schema.

## Uncertainty register

### Known

- The current ValueScout schema already stores player identity, DOB and age, nationalities by name, height, foot, positions, visible and hidden attributes, personality, CA/PA, contract and transfer flags, market value, current/world reputation, current and parent club, loan state, division, derived team level, and a schedule-based date.
- The audited FMSuperScout plugin additionally emits player nation UID, gender, club reputation, and raw team type.
- The audited plugin emits a separate staff collection with UID, identity, DOB and age, nation name and UID, gender, CA, PA, wage, contract expiry, club, division, stable job ID, and 22 staff attributes.
- The audited plugin identifies human managers and emits manager name, managed club, and managed-club reputation. The manager UID is available from the same validated object header.
- The audited plugin supports men's, women's, and combined player scans and performs global club discovery before squad resolution.
- Asking price is emitted upstream as a duplicate of its market-value field. Wage demand is always null. FMSuperScout's history, estimates, scores, and projections are computed after extraction.
- FMSuperScout declares condition, morale, home-reputation, squad-number, staff-reputation, and second-value offsets without emitting them as current product data. They are not parity requirements.
- FMSuperScout's scan hardening uses at most eight worker-local 32 MiB buffers, reduces workers under memory pressure, records unread coverage, and retries once from a PSS VA clone when a live scan misses more than ten percent.
- The repository already has fake-memory bridge tests, replace-only-on-success dump tests, exact Rust dump validation, migration replay tests, transactional ingest tests, and large generated player-ingest checks.

### Assumptions

- The pinned pure-staff and human-manager class offsets reproduce on the supported FM 26.3.2 save used for final validation.
- The existing person-header UID, name, DOB, nation, gender, and contract chains apply to accepted staff and manager objects as they do upstream.
- One typed scan result can serve production extraction without weakening player ordering, cap, cancellation, or diagnostics behavior.
- A snapshot-owned `staff` table with one attributes JSON object is sufficient until a product feature needs staff queries.
- Two comparable live scans can establish semantic parity and useful timing/memory evidence without changing the FM save between runs.

### Decisions

- Use two PRs. PR 1 delivers independently useful direct-data parity and persistence. PR 2 changes scan mechanics and native resource lifecycles only after PR 1 supplies a stable semantic baseline.
- Define parity against the direct memory-backed output and required discovery behavior of the pinned upstream plugin, not its app features or JSON names.
- Do not depend on the abandoned memory-probe branch. Omit dormant, un-emitted pins instead of reopening memory research.
- Add one schema-v6 transition after the new readers and field list are fixed. Reject stale schema-v5 dumps with a clear plugin-update and rescan instruction.
- Persist future-facing player scalar fields as nullable columns. Persist staff in a snapshot-owned table keyed by `(snapshot_id, uid)` with stable scalar columns and `staff_attributes_json`.
- Store manager UID/name, managed club/reputation, player database scope, staff count, and date basis on the snapshot.
- Keep `men` as the app-generated request default. Support `women` and `both` in the file protocol without adding a UI in this feature.
- Record schedule consensus as `gameDateSource: derived` with a distinct `next-fixture-consensus` basis. Preserve the existing Academy rule that trusts valid `derived` years.
- Use the PR 1 live dump as PR 2's equality oracle. Remove any hardening layer that cannot preserve results or show a practical benefit.
- Start PR 2 with read-quality accounting. Use the audited ten-percent unread threshold as the fixed retry/failure boundary unless live evidence recorded before that commit disproves it.
- Retry at most once from a PSS VA clone. Snapshot failure, low commit memory, cancellation, or an incomplete retry produces a failed request and preserves prior data.
- No new ADR is required. The work remains inside ADR-0016's accepted C# bridge, Rust validation, and SQLite ingest boundaries.

### Unknowns

- Exact live player, staff, manager, and club counts on the reference save, including how many UIDs expose both player and staff facets.
- Whether women's or mixed player scans need any additional class offset on FM 26.3.2.
- Whether global club discovery adds enough extraction time or false candidates to require a narrower proven structural test.
- The final schema-v6 dump size, Rust validation time, SQLite growth, and transaction time with a complete staff set.
- Whether the ten-percent unread boundary is reached in normal live scans on the reference machine.
- Whether a PSS VA clone can be captured, read, cancelled, and released reliably under the reference save's real memory pressure.

### Risks

- A broad club heuristic can accept arbitrary objects and corrupt squad or manager resolution. Require structural and name evidence, deterministic deduplication, and contract fallback.
- A shared staff facet can duplicate nearly every player. Deduplicate by UID only after the full typed scan has identified the player set.
- Adding staff can make dumps and ingest materially larger. Measure the complete pipeline before publication and keep product queries unchanged.
- A schema bump can strand a stale installed DLL or old dump. Return an explicit update/rescan error and never partially ingest it.
- Changing the date label can alter automatic Academy class creation near a year boundary. Preserve valid derived-year behavior with regression tests.
- Parallel workers can change first-hit selection or diagnostics. Merge by stable object address and UID, then apply one deterministic classification and deduplication policy.
- Large worker buffers and a process snapshot can increase FM's memory pressure. Keep hard ceilings, inspect available physical and commit memory, and release resources promptly.
- A snapshot retry can appear successful while still returning incomplete data. Apply the same read-quality gate and semantic validation to both attempts.

## Walking skeleton

Carry one known player's nation UID and one staff record from a typed memory scan through schema v6 validation into the snapshot transaction. This proves the new entity and cross-language contract before later scan mechanics change.

## Delivery plan

### PR 1 — Add SuperScout direct-data parity

**Status:** Merged

**PR ref:** https://github.com/JG1995/fm-valuescout/pull/34

**Merge ref:** d3f1cad1f6d9f33155f51a8cb74a43b5a77d09d7

**Branch:** feature/fm-superscout-parity

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** strict `check`

**Feature close-out:** Not required

**CI repair rounds:** 0

**Provisional PR title:** `feat(memory-read): add SuperScout reader parity`

**Build-feature-loop profile:** Terra Max — this PR adds native object families and changes the C# dump, Rust validation, SQLite migration, and transactional ingest together.

**Purpose:** Deliver and persist the direct memory data that FMSuperScout proves but ValueScout schema v5 omits. Establish a live schema-v6 baseline without changing product UI or scan concurrency.

**Merge to trunk when:** Schema v6 validates and ingests complete player and staff output, representative live values match FM, resource measurements are practical, and existing product behavior remains unchanged.

**Depends on:** `main` at `2c4d8c4`; ADR-0016; FMSuperScout author permission and the pinned audit above.

#### Commit 1 — Discover non-player people

**Status:** Completed

**Provisional commit:** `feat(memory-read): discover non-player people`

**Work:** Replace the player-only scan return with one typed result that also classifies pinned pure-staff and human-manager objects. Retain the object and block bases needed by later readers, deterministic UID/address ordering, class histograms, and player/staff overlap evidence. Preserve the existing player candidate output semantics for the production pipeline.

**Out of scope:**

- Reading staff fields or emitting staff in the dump.
- Global club discovery, database-scope filtering, schema changes, persistence, or UI.
- Parallel scanning, snapshot retry, or changing accepted memory regions.

**Implementation packet:**

- Owners and files: `bridge/Scanning/PersonScanner.cs`; a focused typed scan-result model under `bridge/Scanning/`; `bridge/Layouts/IFmMemoryLayout.cs`; `bridge/Layouts/Fm263Layout.cs`; `bridge/Scanning/ScanDiagnostics.cs`; fake-memory scanner tests.
- Existing patterns to verify: dynamic class-offset caching, module-vtable checks, in-buffer UID reads, CA/PA sanity, request cap, cancellation, class histograms, stable UID ordering, and `FakeMemoryReader` sparse regions.
- Constraints and invariants: accept only the pinned player, player/staff, pure-staff, and human-manager class offsets; validate staff CA/PA against the staff block; retain checked address arithmetic; keep player facets authoritative over staff duplicates; expose manager candidates without inventing a selection rule; keep capped scans explicitly incomplete.
- Dependencies and ordering: establish this typed seam before any reader, club graph, or schema assumes staff and manager availability.

**Implementation profile:** Terra xhigh — the upstream class pins are known, but the scanner return contract, overlap handling, and compatibility with the existing player pipeline require local design judgment.

**Review profile:** Sol High — review must challenge class false positives, address arithmetic, player/staff duplication, cap and cancellation behavior, and compatibility with every current player-scan caller.

**Validation:** Start with a fake region containing a pure player, player/staff person, pure staff member, human manager, duplicate UID facets, and near misses. Confirm the current player-only result cannot expose the required types, then prove exact classification, deterministic ordering, overlap evidence, cap metadata, cancellation, and unchanged player candidates. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Replan if one typed scan result cannot preserve the current player contract, if staff acceptance needs an unpinned class rule, or if checked address bases cannot be represented without exposing raw pointers beyond the bridge.

**Review mandate:**

- Verify only pinned classes create player, staff, or manager candidates.
- Verify all subtract/add address operations reject underflow and overflow.
- Verify player/staff overlap has one deterministic outcome and cannot lose the player.
- Verify capped, cancelled, and failed scans report incomplete state and cannot look complete.
- Verify player ordering, deduplication, CA/PA checks, and diagnostics remain compatible.

#### Commit 2 — Discover the complete club graph

**Status:** Completed

**Provisional commit:** `feat(memory-read): discover the complete club graph`

**Work:** Retain plausible club objects encountered during the same typed region scan and use them as the primary bounded input to squad and manager resolution. Keep contract-derived club addresses as fallback. Add deterministic club deduplication and diagnostics for accepted and rejected structural candidates.

**Out of scope:**

- A persisted club table or club-facing API.
- Reading staff or new player fields.
- Additional heap passes, unbounded name scans, parallelism, or snapshots.

**Implementation packet:**

- Owners and files: `bridge/Scanning/PersonScanner.cs`; typed scan result and diagnostics; `bridge/Extraction/ClubNameReader.cs`; `bridge/Extraction/SquadClubIndex.cs`; `bridge/Scanning/CapADumpPipeline.cs`; focused club and pipeline tests.
- Existing patterns to verify: candidate region bounds, club team-vector begin/end checks, plausible full/short names, `ContractClubReader`, squad wrapper probing, `SquadPick`, date votes, and deterministic `HashSet`/ordered output boundaries.
- Constraints and invariants: require a bounded aligned team vector with a plausible count and a plausible club name; deduplicate by address before walking; never replace a stronger squad or contract association with a weaker hit; retain the existing team and squad count ceilings; record cap-related incompleteness.
- Dependencies and ordering: use Commit 1's typed result. Complete club discovery before staff, manager, and schema work relies on club coverage.

**Implementation profile:** Terra xhigh — the upstream route is known, but false-club rejection and precedence with the existing contract-seeded index need careful local judgment.

**Review profile:** Sol High — review must trace false-positive paths, vector bounds, name validation, precedence, deterministic deduplication, and scan-cost consequences.

**Validation:** Add fake valid clubs, malformed vectors, implausible names, duplicate addresses, contract-only fallbacks, squad wrappers, multi-club loans, and manager-team candidates. Prove complete club input improves resolution without changing existing stronger results. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Replan if reliable clubs require an unbounded search, if structural checks admit unrelated objects in live validation, or if same-pass discovery makes the serial scan impractically slower.

**Review mandate:**

- Verify club acceptance requires both bounded structure and plausible identity.
- Verify all vector lengths and pointer arithmetic have hard limits.
- Verify duplicate or reordered discovery cannot change squad selection.
- Verify contract-derived clubs remain a safe fallback.
- Verify no club catalog or raw address crosses the dump boundary.

#### Commit 3 — Read remaining player metadata

**Status:** Completed

**Provisional commit:** `feat(memory-read): read remaining player metadata`

**Work:** Read player nation UID, gender, club reputation, and raw team type. Add the closed `men`/`women`/`both` player-database scope to the bridge request and extraction policy, with `men` as the app-generated default. Correct schedule dates to a derived next-fixture basis while preserving the existing age and Academy year outcomes.

**Out of scope:**

- Staff and manager record extraction.
- Schema v6, SQLite, UI controls, dormant offsets, or derived estimates.
- Changing current club, loan, or team-level selection rules beyond carrying the raw team type and reputation from the winning team.

**Implementation packet:**

- Owners and files: `bridge/Layouts/IFmMemoryLayout.cs`; `bridge/Layouts/Fm263Layout.cs`; focused player/save readers and models under `bridge/Extraction/` and `bridge/Models/`; `bridge/Protocol/`; `bridge/Scanning/CapADumpPipeline.cs`; Rust request serialization under `src-tauri/src/features/memory_read/`; bridge and Rust tests.
- Existing patterns to verify: nation pointer and object UID, gender bit, `SquadClubIndex`, `TeamLevelMap`, request validation/defaults, status/dump metadata, `GameDateResolver`, and Academy trusted-date tests.
- Constraints and invariants: keep nation name and UID separate; use a closed gender representation with explicit unknown handling; retain raw team type beside derived team level; take club reputation from the selected team; filter only player acceptance by database scope; label next-fixture consensus as derived with a stable basis; keep fallback date semantics explicit.
- Dependencies and ordering: use Commit 2's selected club/team evidence. Fix field semantics before schema v6 is frozen.

**Implementation profile:** Terra xhigh — offsets are proven, but request compatibility, gender scope, team precedence, date honesty, and Academy year behavior cross bridge and Rust boundaries.

**Review profile:** Sol High — review must verify every field's basis and null rule, scope filtering, request defaults, and the downstream effect of date-source changes.

**Validation:** Add fake-memory tests for nation UID, gender values, club reputation, raw team type, unknown reads, and all database scopes. Add request tests for missing/default and invalid scope values. Prove schedule consensus reports the derived basis and that valid derived years still create the same Academy class at year boundaries. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Omit a field if its pinned route does not reproduce. Replan if women's or mixed scans require an unpinned class, or if correcting the date basis would invalidate existing Academy cohort semantics.

**Review mandate:**

- Verify nation UID is read from the nation object, not inferred from text.
- Verify gender filtering and dump scope cannot disagree.
- Verify raw team type and club reputation come from the selected squad team.
- Verify omitted scope remains the documented men's default.
- Verify next-fixture data is never labeled as an exact memory world date.
- Verify existing team level, loan, age, and Academy behavior remains correct.

#### Commit 4 — Extract non-player records

**Status:** Completed

**Provisional commit:** `feat(memory-read): extract non-player records`

**Work:** Add dedicated readers and internal models for the audited staff field set and human-manager metadata. Use stable English staff-attribute keys and numeric job IDs. Resolve staff clubs and divisions through the existing bounded contract/team/competition chains and select one human manager deterministically.

**Out of scope:**

- Dump or SQLite changes, localized job names, staff UI, coaching calculations, or staff reputation.
- Treating a player/staff dual-role person as a duplicate staff record.
- Guessing a human manager when the pinned class produces no valid candidate.

**Implementation packet:**

- Owners and files: staff/manager readers under `bridge/Extraction/`; internal models under `bridge/Models/`; layout entries under `bridge/Layouts/`; `bridge/Scanning/CapADumpPipeline.cs`; focused extraction and orchestration tests.
- Existing patterns to verify: `NameReader`, `NationReader`, `FmDateDecoder`, `PlayerAge`, `AttributeScale`, `PlayerContractReader`, `ContractClubReader`, `CompetitionNameReader`, null/sentinel handling, and manager club evidence from the complete club graph.
- Constraints and invariants: validate staff CA/PA and all 22 stored-times-five attributes; store job ID as the stable value; keep unread or impossible values null; preserve manager UID/name and managed club/reputation from one deterministic candidate; exclude staff UIDs present in the player set; never emit process addresses.
- Dependencies and ordering: requires Commit 1 typed candidates and Commit 2 club coverage. Complete readers before schema v6 exposes the fields.

**Implementation profile:** Terra Max — a second entity family combines identity, attributes, contracts, club resolution, dual-role deduplication, and manager selection across several native chains.

**Review profile:** Sol High — review must compare every field with pinned provenance and challenge staff duplication, attribute transforms, stable identifiers, null rules, and manager ambiguity.

**Validation:** Add fake-memory coverage for all 22 staff attributes, identity, nation UID, gender, CA/PA, wage, expiry, job ID, club, division, player/staff overlap, multiple managers, missing manager, and invalid reads. Prove deterministic selection and no output address leakage. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Omit any field that cannot reproduce its documented upstream route. Replan if the live save cannot establish staff completeness or one deterministic human-manager selection, or if staff records require a separate unbounded scan.

**Review mandate:**

- Verify all staff fields use the correct person, staff-block, contract, team, and competition bases.
- Verify all 22 attribute offsets, stored-times-five transforms, keys, and null rules.
- Verify job ID remains numeric and language-independent.
- Verify every player/staff overlap keeps the player and avoids a duplicate staff row.
- Verify manager metadata and any staff row agree on UID and identity.
- Verify no dormant reputation or localized job field enters the contract.

#### Commit 5 — Publish and persist dump schema v6

**Status:** Completed

**Provisional commit:** `feat(snapshot): ingest SuperScout parity data`

**Work:** Make one atomic production-contract transition. Emit schema v6 with the fixed player, staff, manager, scope, and date-basis fields. Update Rust validation and the golden fixture. Add migration v15 and ingest every promised field transactionally into snapshots, players, and a new snapshot-owned staff table.

**Out of scope:**

- Career or Moneyball statistics, staff query APIs, UI, snapshot history, or backfilled invented values.
- Accepting schema v5 as v6 or partially ingesting a new entity family.
- Per-attribute staff SQL columns, search indexes, or role-score calculations for staff.

**Implementation packet:**

- Owners and files: `bridge/Models/DumpDocument.cs`; `bridge/Output/DumpWriter.cs`; `bridge/Protocol/BridgeProtocol.cs`; `bridge/DUMP_SCHEMA.md`; bridge serialization tests; `src-tauri/src/features/memory_read/dump_validation.rs` and fixture; `src-tauri/src/db/migrations.rs`; `src-tauri/src/features/snapshot/ingest.rs`; focused migration and ingest tests.
- Existing patterns to verify: exact schema/version validation, required versus nullable fields, streamed compact output, replace-only-on-success, migration replay from every prior version, prepared inserts, role scoring, one snapshot transaction, current-snapshot promotion, cascades, and rollback tests.
- Constraints and invariants: `playerCount` and `staffCount` equal their arrays; `scanTruncated` covers both; stale v5 returns a plugin-update/rescan error; snapshots store manager, scope, staff count, and date basis; players store new nullable scalars without changing current query DTOs; staff uses `(snapshot_id, uid)` and `staff_attributes_json`; one transaction inserts the snapshot, players, staff, role scores, and Academy effects or rolls back all of them.
- Dependencies and ordering: Commits 1 through 4 lock discovery, readers, field semantics, and null rules. Do not split the version transition across publishable commits.

**Implementation profile:** Terra Max — this is a cross-language persisted contract with large-record performance, migration, stale-dump, rollback, and silent-data-loss consequences.

**Review profile:** Sol xhigh — review must trace every field across C# output, Rust validation, SQLite storage, replacement, and rollback, including existing-data migration and large-array behavior.

**Validation:** Start with a schema-v6 fixture containing one player, one staff member, manager metadata, all scope/date metadata, and mixed nulls. Confirm the current validator rejects it for the expected version. Then prove exact bridge serialization, Rust type and count validation, duplicate UID rejection per entity family, migration from versions 1 through 14, complete ingest, cascade replacement, stale-v5 error text, transaction rollback, unchanged player queries, and practical generated large-dump validation/insert behavior. Run `./scripts/dev bridge-test`, `./scripts/dev test`, and `./scripts/dev check`.

**Stop conditions:** Replan if any promised field validates but is not persisted, if migration v15 requires destructive backfill, if staff needs a premature product API, or if generated large-record costs exceed the existing Load Data budget before live validation.

**Review mandate:**

- Verify one authoritative schema-v6 field list matches bridge, Rust, fixture, migration, ingest, and documentation.
- Verify every promised field is persisted or excluded before validation succeeds.
- Verify stale v5 cannot replace the current snapshot and gives a useful rescan instruction.
- Verify player and staff duplicate rules, foreign keys, cascades, and transaction rollback.
- Verify role scoring, Academy effects, and existing query DTOs remain unchanged.
- Verify large arrays do not create avoidable per-record parsing or statement overhead.
- Verify no raw address or machine-local evidence enters persisted data.

#### Commit 6 — Validate SuperScout data parity

**Status:** Completed

**Provisional commit:** `docs(memory-read): validate SuperScout data parity`

**Work:** Install the schema-v6 bridge and perform one complete live scan and ingest on FM 26.3.2. Compare representative player, staff, manager, club, scope, and date values with FM and the pinned upstream semantics. Record counts, duplicate handling, dump size, database growth, and phase timings. Reconcile the bridge schema/runbook, architecture, and this ledger.

**Out of scope:**

- New fields, UI, scan concurrency, snapshot retry, or claims about untested women's/mixed saves.
- Committing raw dumps, names, memory addresses, screenshots, or machine-specific diagnostics.
- Starting PR 2 before this PR is merged and its semantic baseline is recorded.

**Implementation packet:**

- Owners and files: `bridge/README.md`; `bridge/DUMP_SCHEMA.md`; `.wiki/ARCHITECTURE.md`; this ledger; focused corrections only when live evidence exposes a bounded contract defect.
- Existing patterns to verify: `./scripts/dev bridge-install`, request/status flow, unlimited Load Data, dump replacement, Rust validation, transactional ingest, existing screen checks, diagnostics timings, and untracked `.work/` evidence.
- Constraints and invariants: use one unchanged loaded save; record player/staff/manager/club counts and dual-role handling; inspect representative known values; confirm scope and date basis; compare existing Search, Player, Planner, and Academy results before/after; measure dump and database size plus scan/validation/insert times; do not retain private artifacts in Git.
- Dependencies and ordering: run after schema-v6 automated validation. This commit makes PR 1 ready for publication and creates PR 2's semantic baseline.

**Implementation profile:** Luna Max — the implementation is fixed; the work is bounded live evidence, any narrow correction, and accurate documentation.

**Review profile:** Sol High — the recorded contract becomes the oracle for persistence and later hardening, so review must verify every claim against code and concise live evidence.

**Validation:** `./scripts/dev bridge-install` installed the schema-v6 DLL, then one FM restart and an unlimited Windows Load Data cycle completed successfully on FM 26.3.2. The bridge and active SQLite snapshot both recorded 247,781 players and 134,316 staff, with no duplicate UIDs, a manager record, `men` scope, derived `next-fixture-consensus` date basis, `scanTruncated: false`, and `maxAccepted: null`. The dump was 491,761,405 bytes; the app database after ingest was 7,107,915,776 bytes. The bridge total was 38.365 s and the observed bridge-ready-to-snapshot-commit interval was 55.7 s. The developer confirmed the representative field checks and existing product screens worked correctly in the Windows app. Existing automated replacement and rollback tests remain the proof for failure preservation. A fresh-context Sol High review accepted the exact PR 1 diff and sanitized evidence after documentation corrections.

**Stop conditions:** Do not publish if required fields are systematically wrong, staff completeness or dual-role behavior is unexplained, manager selection is ambiguous, stale v5 can replace current data, or the expanded dump is impractical. Remove an unproven field instead of weakening the evidence rule.

**Review mandate:**

- Verify representative live values and nulls against FM and pinned source semantics.
- Verify counts, duplicate rules, scope, and date-basis documentation match schema v6.
- Verify existing product screens and Youth Academy year behavior remain unchanged.
- Verify timings and sizes are measured and labeled with the tested save.
- Verify excluded upstream values remain excluded.
- Verify no private live artifact enters Git.

### PR 2 — Harden FM memory scans

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/fm-superscout-scan-hardening

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** strict `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `perf(memory-read): harden FM memory scans`

**Build-feature-loop profile:** Terra Max — read-quality gating, bounded concurrency, and optional process snapshots affect determinism, cancellation, memory pressure, native handles, and prior-dump safety inside FM.

**Purpose:** Adopt only the audited scan safeguards that preserve PR 1's exact schema-v6 results and prove a practical reliability or speed benefit.

**Merge to trunk when:** Comparable live scans match PR 1 semantically, resource ceilings and native lifecycles are proven, and every retained hardening layer has measured value.

**Depends on:** PR 1 merged with its schema-v6 live counts, representative samples, timings, and size baseline recorded.

#### Commit 1 — Measure scan read quality

**Status:** Completed

**Provisional commit:** `feat(memory-read): measure scan read quality`

**Work:** Track requested, readable, unread, and internally failed region bytes for the full typed scan. Record the read source and quality in diagnostics. Treat more than ten percent unread coverage as materially incomplete and fail without replacing `dump.json`, while preserving deliberate capped-scan behavior.

**Out of scope:**

- Parallel workers, snapshots, retries, schema changes, fields, or configurable thresholds.
- Treating zero-filled gaps as real readable memory.
- Rejecting a deliberately capped scan solely because it is truncated.

**Implementation packet:**

- Owners and files: `bridge/Memory/IMemoryReader.cs` and focused read-result types if needed; `bridge/Memory/BlockReadHelper.cs`; `bridge/Scanning/PersonScanner.cs`; typed scan result/diagnostics; `bridge/Scanning/CapADumpPipeline.cs`; fake reader and output-safety tests.
- Existing patterns to verify: partial block subdivision, cleared unread gaps, `CountingMemoryReader`, scan diagnostics, cancellation, `DumpWriter.TryWriteReplaceOnSuccess`, and failed status handling.
- Constraints and invariants: count only bytes proven readable; keep quality accounting independent from semantic values; use one compiled-in ten-percent boundary; mark region exceptions as unread; fail materially incomplete live results before extraction/write; preserve prior dump and snapshot; keep diagnostic capped scans explicit.
- Dependencies and ordering: use PR 1's complete typed entity scan. Establish the quality signal before concurrency or retry consumes it.

**Implementation profile:** Terra xhigh — the behavior is bounded, but current block-read recovery does not expose a simple contiguous readable span, so accurate accounting needs careful local design.

**Review profile:** Sol High — review must verify byte accounting around partial reads, holes, exceptions, cancellation, cap semantics, and every path that can replace prior output.

**Validation:** Use fake blocks with complete reads, sparse unread pages, short reads, recovered subranges, region exceptions, cancellation, and deliberate caps. Prove exact quality totals, the ten-percent boundary, diagnostics, and prior-dump preservation. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Replan if the existing reader abstraction cannot distinguish readable holes without a breaking contract, or if the quality threshold cannot separate ordinary recoverable gaps from a materially incomplete scan.

**Review mandate:**

- Verify every readable/unread byte is counted once.
- Verify cleared buffer gaps never count as successful zero data.
- Verify the exact threshold boundary and region-error treatment.
- Verify cancellation and deliberate caps remain distinct from unread failure.
- Verify incomplete output cannot replace a prior good dump.

#### Commit 2 — Parallelize deterministic region scanning

**Status:** Active

**Provisional commit:** `perf(memory-read): parallelize deterministic region scans`

**Work:** Scan independent candidate regions with bounded worker-local buffers and typed collections, then merge player, staff, manager, club, histogram, and quality results deterministically. Use at most eight workers, reduce the count under measured memory pressure, and return all buffers promptly.

**Out of scope:**

- PSS snapshots or retry policy.
- New fields, schemas, persistence, UI, or production tuning controls.
- Performance claims before comparable live measurements.

**Implementation packet:**

- Owners and files: `bridge/Scanning/PersonScanner.cs`; typed scan-local/result helpers; a narrow memory-status query under `bridge/Memory/`; diagnostics; fake-memory concurrency, exception, cancellation, and determinism tests.
- Existing patterns to verify: fixed 32 MiB scan blocks, boundary overlap, dynamic-offset cache, accepted regions, player cap, cancellation, stable UID/address ordering, `ArrayPool<byte>`, scan gate, and PR 1's complete semantic fixture.
- Constraints and invariants: use one buffer per worker and no more than eight; lower the fixed count under a documented available-memory boundary; keep all mutable collections worker-local; merge by stable address and UID, never completion order; propagate worker exceptions and cancellation; preserve the one-scan-at-a-time gate and every PR 1 value.
- Dependencies and ordering: requires Commit 1 quality accounting and PR 1's equality oracle. Do not add retry until parallel results are deterministic.

**Implementation profile:** Terra Max — concurrency enters the native-memory hot path and combines large buffers, cancellation, exceptions, caps, deduplication, and several entity families.

**Review profile:** Sol xhigh — review must independently trace worker lifecycle, buffer ownership, deterministic merge, shared caches, cancellation races, memory ceilings, and output replacement.

**Validation:** Use a deterministic blocking fake to prove bounded overlap, then compare complete serial and parallel results across permuted regions, duplicate objects, block boundaries, unread holes, caps, cancellation, and worker exceptions. Assert identical entity values, ordering, diagnostics, and quality totals. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Keep the serial scanner if deterministic equality needs hot-loop global locks, fixed memory exceeds the worker budget, cancellation cannot terminate every worker, or the later live comparison shows no practical benefit.

**Review mandate:**

- Verify worker and buffer counts have hard upper bounds and adapt only from measured memory.
- Verify no semantic or diagnostic result depends on task scheduling.
- Verify dynamic-offset caches and collections are not mutated unsafely.
- Verify exceptions and cancellation stop all workers and preserve prior output.
- Verify boundary overlap, duplicate handling, caps, and quality totals match serial behavior.

#### Commit 3 — Retry incomplete scans from a frozen snapshot

**Status:** Pending

**Provisional commit:** `feat(memory-read): retry incomplete scans from snapshots`

**Work:** When the live attempt crosses the fixed unread boundary, discard that attempt and retry once through a PSS VA-clone memory reader. Check available commit memory before capture, apply the same quality and semantic gates to the retry, report the source and failure reason, and release every native resource on every path.

**Out of scope:**

- Snapshotting every scan, retrying more than once, user controls, or using partial attempts.
- Supporting PSS outside Windows or adding machine-local binaries.
- Changing extraction fields, schema, ordering, threshold, or persistence.

**Implementation packet:**

- Owners and files: a focused disposable snapshot owner and handle-backed reader under `bridge/Memory/`; `bridge/Plugin.cs` and `bridge/Scanning/CapADumpPipeline.cs` for attempt orchestration; diagnostics/status; fake abstractions and Windows-guarded native tests.
- Existing patterns to verify: `WindowsMemoryReader`, `VirtualQueryEx`, `ReadProcessMemory`, `SafeHandle` disposal, plugin scan gate, unload cancellation, replace-only-on-success, upstream live-first retry, and current Windows bridge CI.
- Constraints and invariants: retry only after measured live incompleteness; at most one snapshot attempt; never retain or combine attempt results; fail clearly on low memory or snapshot creation/query failure; own each PSS snapshot and clone handle once; release on success, failure, cancellation, and exception; keep module addresses and layout pins unchanged; never replace output until the retry passes all gates.
- Dependencies and ordering: requires deterministic parallel semantics and quality accounting. Keep snapshot mechanics behind `IMemoryReader` so most tests remain platform-neutral.

**Implementation profile:** Terra Max — native handle ownership, copy-on-write memory pressure, retry state, cancellation, and prior-dump preservation create difficult partial-failure paths inside FM.

**Review profile:** Sol xhigh — review must trace every native handle and output lifecycle, retry trigger, low-memory path, cancellation edge, and interaction with plugin unload and the scan gate.

**Validation:** Prove live-complete/no-retry, live-incomplete/snapshot-complete, snapshot creation failure, low memory, retry-incomplete, cancellation, worker exception, and double-disposal prevention with fakes. Add a Windows-only native test that captures the test process, reads known memory through the clone, and releases it. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Remove this commit from the plan if PSS capture cannot be bounded, safely tested, reliably released, or supported under the reference machine's memory pressure. Keep quality measurement and prior-dump preservation even if snapshot retry is removed.

**Review mandate:**

- Verify every native handle has one owner and closes on every exit path.
- Verify retry triggers only from measured incompleteness and runs once.
- Verify low memory or snapshot failure cannot crash FM or replace data.
- Verify cancellation and plugin unload cannot race a live snapshot.
- Verify retry output passes the same semantic and quality gates as live output.
- Verify diagnostics expose no address or private process detail.

#### Commit 4 — Validate hardened scan behavior

**Status:** Pending

**Provisional commit:** `docs(memory-read): validate hardened scan behavior`

**Work:** Compare the final scan with PR 1's unchanged-save schema-v6 baseline. Record workers, fixed buffer bound, readable coverage, source, retries, phase timing, dump size, and process memory. Keep only hardening layers that preserve every semantic result and improve or materially protect the supported workflow.

**Out of scope:**

- New data, schemas, UI, tuning controls, or unsupported performance claims.
- Manufacturing dangerous low-memory conditions merely to force a live snapshot retry.
- Committing raw dumps, personal values, memory addresses, or machine telemetry.

**Implementation packet:**

- Owners and files: `bridge/README.md`; `.wiki/ARCHITECTURE.md`; this ledger; focused corrections or removals only when live evidence exposes a bounded parity, lifecycle, or cost defect.
- Existing patterns to verify: `./scripts/dev bridge-install`, unlimited Load Data, diagnostics phases, PR 1 semantic counts and samples, status/error handling, prior-dump preservation, and untracked `.work/` evidence.
- Constraints and invariants: use the same unchanged loaded save where practical; compare complete player, staff, manager, club, and representative field results; report workers, buffer ceiling, unread fraction, source, retry count, elapsed phases, and process memory; explain any save-state drift; remove concurrency or snapshot support if cost or risk exceeds measured value.
- Dependencies and ordering: run after automated parity and native-lifecycle tests pass and after FM restarts with the final DLL.

**Implementation profile:** Luna Max — the work is a controlled live comparison, bounded correction or removal, and concise documentation after high-risk mechanics stabilize.

**Review profile:** Sol High — final PR review must verify semantic equality, measured claims, native-resource safety, failure behavior, and documentation accuracy.

**Validation:** Run `./scripts/dev bridge-test`, `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev bridge-install`. After one FM restart, run an unlimited schema-v6 Load Data cycle, compare its complete counts and representative values with PR 1, inspect read-quality and resource diagnostics, exercise prior-dump preservation through a safe failure path, and record comparable timings. Run feature close-out with the exact merged PR 1 ref and PR 2 commit set.

**Stop conditions:** Do not publish any layer that causes unexplained value/count drift, increases failure risk, exceeds fixed resource bounds, or lacks adequate Windows evidence. Remove that layer and retain the stable schema-v6 reader.

**Review mandate:**

- Verify complete semantic parity with PR 1 or a documented save-state explanation.
- Verify every timing and memory claim comes from comparable measured runs.
- Verify worker, buffer, threshold, retry, and source documentation matches code.
- Verify incomplete, failed, and cancelled attempts preserve prior data.
- Verify native snapshot and worker lifecycles remain safe under failure.
- Verify no private live artifact enters Git.

## Active work

**PR:** PR 2 — Harden FM memory scans

**Commit:** Commit 2 — Parallelize deterministic region scanning

### RED proof

Use a deterministic blocking fake to prove the current serial scanner does not overlap independent candidate regions. Then compare serial and bounded-worker results across permuted regions, duplicate objects, boundaries, unread holes, caps, cancellation, and worker exceptions.

### Expected outcome

The bridge scans independent candidate regions with bounded worker-local buffers and merges every typed entity, diagnostic, and quality result by stable address and UID, independent of task scheduling.

### Explicit exclusions

- Process snapshots, retry policy, data/schema/UI changes, tuning controls, and unmeasured performance claims.
- Completion-order-dependent results, global hot-loop locks, unbounded workers or buffers, and altered cap/cancellation/read-quality semantics.

## Discoveries and replanning

- The 2026-08-08 audit used a temporary local clone of FMSuperScout at `0f270d39`. The `plugin/` tree has no diff from the earlier audited `4ec3c657`, so the prior field and scan findings remain current.
- The abandoned research branch is not a dependency. Its unmerged probe cannot validate dormant pins on this clean `main` branch, and direct parity does not require those un-emitted fields.
- The old PR 2 and PR 3 remain separate in this new plan as PR 1 and PR 2. Data parity is independently mergeable; scan hardening has separate concurrency, measurement, and native-resource stop conditions.
- Commit 2 review found that a pointer-sized team vector can still be invalid when both endpoints are misaligned. Discovery and squad walking now require aligned endpoints; the focused regression proves the malformed club cannot displace a valid association.
- Commit 3 retains unread player gender as explicit `unknown`: men keeps it to preserve the existing default path, women requires a known female value, and both keeps every player. Staff remains unfiltered.
- Commit 3 labels schedule consensus as derived `next-fixture-consensus`; the no-vote fallback is derived `birth-cohort-and-system-date`. The basis remains diagnostics-only until schema v6 adds its persisted field.
- Commit 4 retains staff and manager data inside the bridge until schema v6. The complete club graph matches human-manager person pointers at team `+0x80`, prefers a first-team match, and falls back to the manager's bounded contract chain; candidates without a readable name produce no manager metadata.
- Commit 1 adds exact readable block ranges so the scanner's deliberate 16-byte overlap is never double-counted. Readers that cannot identify partial coverage fail closed as internally unread bytes, and block fills clear bytes outside final coverage when a child retry invalidates an earlier partial parent read.
- Commit 4 keeps corrupt staff attribute bytes and non-leap-year day 366 contract expiries null. Player attribute compatibility decoding remains unchanged.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 — Add SuperScout direct-data parity | Commit 1 — Discover non-player people | 0f0dc83 | Typed scan result classifies pinned player, staff, and human-manager candidates while preserving the player-only pipeline. | Sol High: accepted after correction review. | None |
| PR 1 — Add SuperScout direct-data parity | Commit 2 — Discover the complete club graph | fce0d07 | Same-pass, bounded club discovery feeds deterministic squad resolution while contract-derived clubs remain fallback. | Sol High: accepted after alignment correction review. | None |
| PR 1 — Add SuperScout direct-data parity | Commit 3 — Read remaining player metadata | 92823ca | Reads player nation UID and gender, carries selected-team raw type/reputation, applies closed request scope, and labels schedule dates with an explicit derived basis. | Sol High: accepted. | None |
| PR 1 — Add SuperScout direct-data parity | Commit 4 — Extract non-player records | 3e68e09 | Retains validated non-player staff fields and deterministic human-manager metadata inside the bridge without changing schema v5 output. | Sol High: accepted after correction review. | None |
| PR 1 — Add SuperScout direct-data parity | Commit 5 — Publish and persist dump schema v6 | 4ea5a43 | Schema v6 bridge, validation, migration v15, and transactional ingest preserve player, staff, manager, scope, and date-basis data. | Sol xhigh: accepted after C5-01 and C5-02 correction review. | None |
| PR 1 — Add SuperScout direct-data parity | Commit 6 — Validate SuperScout data parity | 8553e9f | A live unlimited FM 26.3.2 Load Data run matched bridge declarations with active SQLite rows and recorded the sanitized baseline. | Sol High: accepted after correction review. | None |
| PR 2 — Harden FM memory scans | Commit 1 — Measure scan read quality | Pending record | Adds exact readable coverage, unread-quality diagnostics, and pre-write failure at more than ten percent unread coverage while preserving caps and prior dumps. | Sol High: accepted after stale-buffer correction review. | None |

## Final validation

**Feature review profile:** Sol xhigh — final review must connect native object discovery, schema-v6 persistence, deterministic concurrency, optional PSS lifetime, and prior-data safety across two PRs.

- Run `./scripts/dev format`, `./scripts/dev bridge-test`, `./scripts/dev test`, and `./scripts/dev check` on the final exact implementation set.
- Confirm GitHub's strict required `check` passes for each PR before squash merge.
- PR 1 manual FM26 validation: one install/restart cycle, one unlimited schema-v6 scan and ingest, representative player/staff/manager/club checks, existing-screen regression checks, and recorded counts, sizes, and timings.
- PR 2 manual FM26 validation: one install/restart cycle, one comparable unlimited scan, complete semantic comparison with PR 1, read-quality/resource inspection, and safe prior-dump preservation evidence.
- Use Windows bridge tests to exercise the actual PSS capture/read/release API. Do not require a naturally occurring live retry when the normal scan is complete.
- Review the exact merged PR 1 ref plus the final PR 2 commit set for end-to-end intent, silent field loss, migration/rollback safety, scan determinism, native cleanup, and documentation accuracy.
- Reconcile `bridge/DUMP_SCHEMA.md`, `bridge/README.md`, `.wiki/ARCHITECTURE.md`, `.wiki/TODO.md`, and the completed feature record. Remove temporary parity artifacts from `.work/`.

## Documentation impact

Complete during feature reconciliation. PR 1 owns the schema/runbook and implemented architecture update. PR 2 owns scan operating limits and measured behavior. No ADR is planned unless implementation crosses ADR-0016's existing boundary.
