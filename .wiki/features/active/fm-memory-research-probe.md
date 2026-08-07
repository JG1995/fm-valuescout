# FM Memory Research and Reader Expansion

## Status

Active

## Intent

Add a reusable, developer-only probe for finding FM26 player data in memory. The probe must capture small, labeled memory samples for explicit player UIDs and compare those samples with FM-exported CSV values. It must accept the real Youth Academy and Moneyball export shapes through bounded full-export requests. When scalar root and pointer-window correlation fails, it must support bounded discovery and analysis of player-linked record structures.

After the research workflow is complete, expand the production reader with practically all direct data that the permitted FMSuperScout source already extracts but the original MVP did not port. Persist that data for later product features without adding speculative UI. Treat upstream fields that are only declared or abandoned as research candidates until live evidence confirms their meaning. Finish with bounded scan hardening derived from the same upstream source without coupling those process-memory changes to the data-contract expansion.

## User-visible behavior

- A developer can export a player view from FM and request a bounded memory capture for every explicit UID in that CSV while the same save state remains open.
- A developer can keep the full current sample exports unchanged. The fixed 128-player ceiling accommodates the largest current export without removing the hard safety bound.
- The bridge writes a versioned `probe.json` with game, layout, module, candidate-address, root-window, and bounded pointer-target metadata for each matched UID.
- A structure-aware capture preset can inventory bounded memory contexts that contain exact references to the requested players, then capture only a versioned record shape justified by that inventory.
- A repository command can correlate declared integer or decimal CSV values against one capture and can compare before-and-after CSV/capture pairs. It reports excluded cells and requires explicit normalization for compound or unit-bearing values.
- Analysis can compare direct structured values and declared record aggregates. It reports candidate relative paths, offsets, encodings, non-zero match coverage, and ambiguity. It never labels a candidate as a verified production offset.
- During PR 1, research requests and outputs remain separate from Load Data. The app has no new UI, and schema-v5 `dump.json` remains unchanged.
- After the probe PR merges, Load Data produces a new versioned dump that can include remaining proven player fields, save and manager metadata, complete staff records, and the scan's database scope.
- Snapshot ingest stores the expanded player, save, and staff data transactionally for later features. Existing Search, Player, Planner, Academy, and Moneyball screens do not expose new controls or fields in this feature.
- Production scan requests can select `men`, `women`, or `both`; the current app keeps its existing men's-database default until a later product feature exposes that choice.

## Invariants

- The bridge remains read-only. Probe code must not write FM memory or invoke game mutators.
- Every capture targets an explicit, validated UID set and enforces fixed limits for UID count, root-window size, pointer depth, pointer-target size, and total bytes per player. Whole-process dumps and unbounded pointer traversal are forbidden.
- Every requested UID must resolve exactly once after scanner deduplication. A missing requested UID fails the capture instead of producing an incomplete result.
- The probe accepts only a supported FM build and a resolved versioned layout. It fails closed when either is unavailable.
- `probe-request.json`, `probe-status.json`, and `probe.json` form a separate developer protocol. Probe activity must not modify `request.json`, `status.json`, `diagnostics.txt`, or a prior valid `dump.json`.
- The bridge uses its existing scan gate for full dumps and probes. A pending production full-dump request takes priority when both request types are present.
- A successful `probe.json` is written through a temporary file and atomically replaced. A failed probe must not replace the prior successful capture; its terminal state belongs in `probe-status.json`.
- Each captured byte range records its address basis, relative path or offset, requested length, readable spans or masks, and encoded bytes. Unread bytes must never appear as real zero values. Pointer targets retain the source pointer path so analysis can report a reproducible location.
- Pointer traversal may start only from fully readable aligned pointer cells, and a target must fall inside a pre-enumerated acceptable committed memory region before capture.
- Reverse-reference discovery may search only the same pre-enumerated committed regions for the requested players' exact UID, person-object address, or player-block address. It must use one bounded pass, deterministic grouping, fixed context counts, and fixed byte ceilings.
- A structured record capture may follow only a compiled-in, versioned layout recipe learned from the bounded inventory. Requests must not supply process addresses, record offsets, strides, counts, or byte ceilings.
- The comparison tool must reject or clearly report malformed CSV, duplicate or missing UIDs, incompatible capture metadata, unreadable ranges, unsupported numeric values, and unmatched rows.
- UID handling, missing-value handling, units, compound values, and decimal rounding must be explicit. The tool must not guess a field's meaning from its header or silently coerce display text into scalar truth.
- Correlation results are hypotheses. Production layouts, dump schemas, SQLite, Academy behavior, and Moneyball features change only in later features after independent live validation.
- Zero-valued rows may support a candidate but cannot establish it. Every accepted career-stat candidate must reproduce every eligible row, including every non-zero value, across the available synchronized cohorts without an equally strong conflict.
- Raw CSV exports, probe captures, and analysis output stay untracked under `.work/` or outside the repository. They must not enter Git.
- Production expansion may port fields that the pinned upstream plugin emits directly or fields that this probe independently validates. A constant in upstream `Fields.cs` is not enough evidence by itself.
- Dump schema, Rust validation, and SQLite ingest must advance together at their planned boundary. A new dump must never validate and then silently discard a newly promised field.
- Staff records remain a separate snapshot-owned entity. Player/staff dual-role deduplication must be explicit, deterministic, and compatible with cascade replacement of the current snapshot.
- The schedule-derived date is a next-fixture consensus, not an exact world clock. The expanded contract must record that basis and must not label it as direct memory truth.
- Scanner hardening must preserve the same accepted player, staff, manager, and club results. Parallelism, adaptive buffers, or a frozen-memory retry must not change extraction semantics or weaken prior-dump preservation.

## Non-goals

- Shipping or integrating offsets for all-senior appearances, all-senior goals, international caps, or any other new production field. The probe may report candidate paths from live evidence, but a later feature owns production verification and integration.
- Changing dump schema v5, Rust dump validation, snapshot ingest, or SQLite during PR 1. PR 2 owns one explicit production-contract and persistence transition after the research tooling merges.
- Adding snapshot history or reconstructing statistics from earlier points in a season.
- Adding a product-facing probe UI or sending raw memory over Tauri IPC.
- Capturing the whole heap, following arbitrary addresses supplied by the operator, or performing recursive unbounded pointer scans.
- Adding a generic reverse-pointer graph, accepting operator-defined structure recipes in the live bridge, or retaining every reference hit found in the heap.
- Supporting non-numeric CSV correlation, save-file parsing, non-Windows FM editions, or unsupported FM builds.
- Treating every exported display field as a distinct stored scalar. Later integration may derive rates, percentages, and aggregates from verified base values when that preserves the required semantics.
- Requiring a controlled before/after sample. Diff remains available when a useful state transition can be captured, but cross-sectional replication is the normal evidence path.
- Treating one matching player, one matching value, or one changed byte as enough evidence for an offset.
- Adding staff search, staff profiles, gender filters, manager UI, condition or morale UI, or other product use of the newly stored fields.
- Copying FMSuperScout's app-side estimates, meta scores, potential projections, snapshot history, localized job labels, or derived coaching-star calculations into the memory reader.
- Emitting a separate asking-price field that merely duplicates market value, emitting its always-null wage-demand placeholder, or treating the unused `player+0x238` slot as verified without independent evidence.
- Promising condition, morale, home reputation, squad number, staff reputation, or any other dormant upstream pin when live validation cannot establish its encoding and semantics.

## Current-state map

- Relevant components: `bridge/Plugin.cs` polls the production request file, owns one background scan thread, and serializes scans through `ScanGate`; `bridge/Scanning/CapADumpPipeline.cs` resolves layouts and coordinates candidate discovery and extraction.
- Data model: `PersonScanner` returns `PersonCandidate(ObjectAddress, Uid, Ca, Pa, ClassOffset)`. The player-block base is `ObjectAddress - ClassOffset`; the person object remains available at `ObjectAddress`.
- Memory boundary: `IMemoryReader` provides controlled scalar and block reads; `WindowsMemoryReader` uses `ReadProcessMemory`, and `Tests/Fakes/FakeMemoryReader` provides deterministic sparse memory.
- Layout boundary: `IFmMemoryLayout` and `Fm263Layout` own supported-build pins. Known anchors include UID, CA, PA, attributes, market value, and reputation.
- Output boundary: `DumpWriter` and `StatusWriter` demonstrate temporary-file plus atomic-replace JSON output. Production dump schema v5 is frozen and validated by Rust.
- Persistence and migrations: PR 1 has none; probe artifacts are disposable research files and never enter SQLite. PR 2 owns the next schema migration and transactional storage for the expanded production dump.
- Production persistence after PR 1: schema v5 stores players only. It has no staff table and no columns for nation UID, gender, raw team type, club reputation, manager identity, managed-club reputation, database scope, or date basis.
- Existing behavioral assumptions: full candidate discovery takes about 26 seconds on the reference save and already retains the addresses needed to derive player and person bases. Live FM attach validation remains manual on Windows.
- Architectural seams: capture and raw-memory interpretation stay in the C# bridge; a developer CLI prepares bounded full-export requests and analyzes local CSV/JSON files; Rust, React, and snapshot ingest remain outside the feature.
- Project validation commands: `./scripts/dev bridge-test` runs fake-memory C# tests; `./scripts/dev check` remains the repository commit gate; `./scripts/dev bridge-install` installs a live-test DLL on Windows/WSL.
- Primary risks: unsafe capture breadth, interference with production scans, misleading zero-driven correlations, stale or mismatched CSV/captures, and incorrect grouping or aggregation of indirect history records.
- Upstream reference: [FMSuperScout commit `4ec3c657`](https://github.com/mavarobli/FMSuperScout/commit/4ec3c657e3b993edf4e5b87d5ea42c4a3700cac6) was audited on 2026-08-07. Its current direct reader, field pins, repin guide, and historical discovery notes are useful provenance; its generated dump and app models are not a contract for this repository.

## Feature architecture

```text
FM player-view CSV
  -> ./scripts/dev memory-probe capture
  -> probe-request.json with explicit UIDs
  -> BepInEx bridge: supported layout + existing candidate discovery
  -> fixed player/person windows + budgeted pointer targets
  -> optional bounded reference inventory + versioned record recipe
  -> probe-status.json + atomic probe.json
  -> ./scripts/dev memory-probe correlate | diff
  -> direct or aggregate candidate paths + ambiguity report

Pinned and independently validated field set
  -> extended player, staff, manager, and club readers
  -> versioned production dump schema
  -> Rust validation + transactional SQLite ingest
  -> stored future-facing data; existing product UI unchanged

Stable expanded reader
  -> deterministic bounded parallel scan
  -> read-quality measurement
  -> optional frozen-memory retry on materially incomplete live reads
```

The probe protocol has its own schema version and file names under the existing bridge data directory. It reuses the bridge's layout resolution, candidate scanner, safe memory reader, cancellation path, and scan gate, but it does not enter the production status/dump protocol.

The capture records two stable roots per matched candidate: player-block-relative and person-object-relative. It collects roots for the full requested cohort, ranks safe pointer source paths by cross-player availability, and reserves independent quotas for the two roots before it captures any target. It may repeat this planning step once across the selected first-hop ranges. Every target retains its root-relative source path, and fixed depth, count, window, address-region, and byte ceilings prevent graph expansion. The request may select UIDs and a bounded preset, but it may not supply arbitrary process addresses or remove the built-in limits.

The .NET developer tool under `bridge/Tools/MemoryProbe/` owns CSV parsing, declared value normalization, request creation and waiting, scalar and structured-record interpretation, cross-player correlation, and optional before/after comparison. It uses runtime libraries only. `scripts/dev` exposes the tool as `memory-probe` so the repository retains one command surface.

The generic depth-two capture remains available for ordinary scalar research. A separate compiled-in history preset first inventories exact UID and player-root references in the same safe memory regions. It groups bounded contexts by stable structural provenance instead of treating every hit as interchangeable. Live evidence from that inventory must justify one FM-versioned record recipe before the bridge follows a container or captures record rows. The analyzer may test direct fields and explicit aggregates from those rows, but it must keep raw values, filters, and aggregation rules visible.

Three PRs are appropriate. PR 1 keeps capture, analysis, and structure-aware career research in one isolated developer workflow. PR 2 changes the production dump and persistence contracts together after the research boundary is stable. PR 3 adopts upstream scan hardening only after the expanded reader has a fixed semantic baseline, so field regressions and process-memory regressions remain separable.

## Uncertainty register

### Known

- Candidate discovery already yields UID, object address, class offset, CA, and PA for each accepted player.
- Known FM 26.3 layout pins provide enough anchors to test whether correlation finds correct relative locations and transforms.
- Academy currently represents senior appearances, goals, assists, and international caps as unavailable nullable values. The later integration will omit assists because FM does not export the required career value.
- Production Load Data depends on frozen dump schema v5 and the existing `request.json` / `status.json` files.
- The three current player-search exports are valid UTF-8, semicolon-delimited CSV files with unique UIDs and no malformed rows. Their synchronized captures contain 101, 120, and 23 players from one FM 26.3.2 state.
- `AT Apps` and `AT Gls` are all-senior club totals across league, cup, continental, and other senior matches. `Int Apps` is the senior international appearance total. These are the three Academy research targets.
- Moneyball values are season-to-date statistics at the moment of capture. FM does not expose earlier values through the export, so each load is an honest point-in-time snapshot rather than a complete season history.
- Compound `Appearances` values represent starts and substitute appearances. Later persistence will store those counts separately and derive total appearances.
- The revised Youth Academy export provides strong variation across 103 players. The Moneyball exports mix integers, rounded decimals, missing values, unit-bearing values, localized currency text, and compound appearances.
- The three current player-search exports contain 244 rows for 235 distinct UIDs. Nine UIDs repeat without a conflicting value. They provide all seven current career targets: `AT Apps`, `AT Gls`, `AT League Apps`, `AT League Goals`, `Int Apps`, `Int Gls`, and `International Assists`.
- The pinned FMSuperScout reader emits direct player nation UID, gender, club reputation, raw team type, manager name, managed club, managed-club reputation, and a separate staff collection that includes identity, CA, PA, contract, club, division, job ID, and 22 staff attributes.
- The same upstream source walks staff, human-manager, and plausible club objects during its full scan. Its second squad walk resolves indirect squad entries by searching each entry for a known person address, which supports PR 1's bounded reverse-reference design.
- FMSuperScout's app-side history, asking-price and transfer-interest estimates, meta scores, potential projections, and coaching calculations are derived after extraction. They are not additional memory fields.
- Upstream declares but does not productize player condition, morale, home reputation, squad number, staff reputation, and a second value slot. Their presence in `Fields.cs` is only a lead.
- Upstream documents that the shared team-schedule value is a next-fixture date and can differ from FM's world date during breaks. The current bridge calls the same schedule consensus `memory` and therefore overstates its precision.

### Assumptions

- Exact UID and player-root references will expose a bounded, repeatable route to the record structure that supplies the exported career totals.
- FM player-view exports include a stable numeric UID column and can include varied known and target numeric columns.
- The existing full candidate walk is fast enough for an occasional developer probe; targeted scanner optimization is not required for the first version.

### Decisions

- Use a separate developer-only probe protocol instead of extending the product dump or Rust protocol.
- Capture raw bounded ranges and pointer provenance instead of adding speculative field offsets to `IFmMemoryLayout`.
- Provide single-capture correlation and before/after diff in one .NET CLI with no new package dependency.
- Keep research artifacts disposable and untracked; preserve only validated conclusions in later feature ledgers or layout code.
- Treat the sample exports as temporary evidence and record only their durable field-shape and coverage conclusions in this ledger and the runbook.
- Raise the fixed request ceiling from 16 to 128 players. The initial capture used a 1,408-byte per-player bound and a 180,224-byte full-request bound. Commits 5 and 6 replace these limits with fixed generic-preset ceilings that remain below 512 KiB of raw memory for a 128-player request. The structure-aware preset has separate compiled-in ceilings because it captures a different artifact shape.
- Do not require a controlled before/after sample or disjoint UID sets. Candidate discovery may use one synchronized cohort with varied values. Before later production integration, require fresh synchronized evidence that recovers the known anchors and repeats the same path and encoding with meaningful value variation. A different player set is preferred when FM can provide one, but it is not mandatory. Keep diff as optional supporting evidence.
- Select pointer paths across the requested cohort instead of giving each player the first valid pointers found in player-root order. Rank paths by cross-player availability with deterministic tie-breaking, reserve separate first-hop quotas for player and person roots, and record the fixed capture policy in the probe output.
- Permit one fixed second hop after cohort-level path selection. Allow at most eight player-root first-hop targets, eight person-root first-hop targets, and eight second-hop targets per player. Keep every target window at 128 bytes. Together with the existing roots, the revised maximum is 3,968 bytes per player and 507,904 raw bytes for 128 players.
- Do not research Academy assists. A later Academy feature will remove that unsupported outcome.
- Preserve Moneyball statistics as season-to-date point-in-time values. The existing snapshot-history backlog item may later retain earlier captures inside the app, but this probe does not reconstruct history that FM cannot export.
- Normalize compound appearances into separate starts and substitute-appearance values. A later integration derives total appearances instead of storing the formatted display string.
- Keep career-stat production selection and integration in a later feature plan after the probe identifies and independently validates the required source values. PR 2 may integrate only the separate upstream-parity field set defined here.
- Preserve the generic depth-two preset as a completed scalar-research path. Add structure-aware work as a separate preset; do not increase its pointer depth or path quotas.
- Use the three synchronized player-search cohorts as the required cross-sectional evidence for all seven career targets. Acceptance requires complete eligible-row and non-zero-row coverage across 235 distinct players; the nine repeated UIDs are a consistency check, not additional independent players.
- Treat `International Assists` as an international-career target only. It does not replace the unavailable all-senior Academy assists value.
- Port every direct field emitted by the pinned upstream reader when this repository can preserve its meaning: player nation UID, gender, club reputation, raw team type; manager and managed-club metadata; complete staff records; database scope; and full-club squad discovery.
- Store future-facing staff attributes as one validated JSON object per staff record. Do not add speculative per-attribute SQL columns, search indexes, DTOs, or UI until a staff feature needs them.
- Add an optional `databaseScope` request value with `men`, `women`, and `both`; keep `men` as the app default. Record the selected scope in the dump and snapshot so mixed data is never ambiguous.
- Give the expanded production contract one schema-v6 transition. C# output, Rust validation, the golden fixture, SQLite migrations, and ingest must agree before the PR can publish.
- Replace the false exact-date claim with an explicit date basis. A next-fixture consensus remains useful but is `derived`, not direct memory truth; existing Academy year behavior must be regression-tested at year boundaries.
- Test the dormant upstream pins before schema v6 is locked. Include a field only when varied live truth establishes its encoding and meaning; record a negative result and omit the field otherwise.
- Keep scan parallelism and snapshot retry in PR 3. PR 2 must first establish a deterministic expanded-reader baseline against which scan hardening can prove semantic parity.
- Do not create a new ADR. The expanded extraction and schema version remain inside ADR-0016's accepted C# bridge, Rust validation, and SQLite ingest boundaries; this ledger records the field-selection and delivery decisions.

### Unknowns

- Whether the player-linked structure contains direct totals, per-season or per-competition records, or both.
- Whether one stable reverse-reference signature reaches both club and international records or the two domains need separate FM-versioned recipes.
- The exact container layout, record stride, category fields, and aggregation rules. Commit 8 must obtain live structural evidence before Commit 9 pins them.
- Whether rounded decimal statistics are stored directly as floating-point values, fixed-point totals, or derived from other counters.
- Which FM CSV header names and delimiters appear across other locales; the tool must use explicit UID/field mappings and robust delimiter handling rather than one hard-coded English export.
- Which dormant pins can be validated from exported or plainly visible FM truth, and whether the second value slot has any stable meaning worth storing.
- Whether player/staff dual-role persons expose one object, two linked facets, or duplicate candidates on every supported save shape. PR 2 must preserve the upstream behavior only after fake and live evidence make the deduplication rule explicit.
- Whether a mixed or women's database exposes additional player class offsets beyond the current FM 26.3 pins. The request enum must not imply coverage that the live scan cannot demonstrate.
- Whether a PSS VA-clone retry remains reliable under the reference save's memory pressure. PR 3 may keep measured unread-region failure without snapshot support if the clone cannot be bounded and released safely.

### Risks

- Common small integers can produce many false matches. The analyzer must rank by multi-player coverage and show ambiguity rather than choose the first hit.
- A CSV export and probe taken from different save moments can create false negatives or false deltas. Request and result metadata plus the runbook must make synchronization explicit.
- Pointer-like values in raw bytes can fan out quickly. Hard depth, count, byte, and UID budgets must remain non-configurable ceilings.
- Adding probe polling to `Plugin.cs` can disturb shutdown or production request handling. Tests and review must cover priority, mutual exclusion, cancellation, and file isolation.
- Stats may require a structure-specific reader. If bounded generic capture cannot expose a known indirection in live validation, replan the probe instead of expanding into unbounded traversal.
- Searching for any matching UID or small integer across many contexts can amplify false positives. Reference inventory must group hits by stable structural provenance before value correlation.
- A record sum can match a displayed total while using the wrong seasons or competition types. Reports must show record membership, filters, and raw contributions for every aggregate candidate.
- Sparse event columns can satisfy simple value variation while still producing weak evidence. Require fresh synchronized evidence and inspect related neighboring fields before accepting a binary or rare-event candidate.
- FM display values can be rounded or derived. A direct byte match may be absent even when the underlying source values are inside the capture, so the analyzer must keep exact, rounded, scaled, and derived evidence distinct.
- Expanding the dump can increase file size and ingest time substantially because staff may add tens of thousands of records. PR 2 must measure bridge write, validation, transaction, and database-size impact before publication.
- A schema bump can make a stale v5 dump unreadable. Validation must return an explicit refresh-required error and must not partially ingest or replace the current snapshot.
- A broad club-object heuristic can introduce false club candidates. Full-club discovery must require the same structural checks and plausible-name rules across fake and live evidence, while contract-seeded clubs remain the safe fallback.
- Parallel region scans can change first-hit order and deduplication if merge rules depend on scheduling. PR 3 must sort and merge by stable addresses and UIDs before extraction.
- A frozen-memory clone can consume substantial commit memory and must always be released. Retry must be conditional on measured read quality and must preserve the prior good dump when both attempts remain incomplete.

## Sample export assessment

The earlier five untracked samples established the export shapes. The three current player-search exports under `.work/memory-probe/` now provide synchronized, multi-club evidence for the career targets from one FM 26.3.2 state.

- The original Youth Academy and Moneyball files contain the same 75 UIDs from one club. They remain useful as one paired data set, but the Moneyball file has 41 players with zero minutes and weak rare-event coverage.
- The revised Youth Academy and Moneyball files contain the same 103 UIDs and no overlap with the original pair. They can provide another player set only if the matching save state is available; the evidence contract does not require that operationally unavailable state.
- The revised Youth Academy file has strong target evidence: `AT Apps` has 73 distinct values and 91 non-zero rows, `AT Gls` has 26 distinct values and 59 non-zero rows, and `Int Apps` has 33 distinct values and 69 non-zero rows.
- The player-search Moneyball file contains 72 different UIDs across 51 clubs and 18 divisions. It has 69 players with minutes, 69 ratings, four non-zero penalty-save rows, and broad variation across the other match statistics.
- Across the revised and player-search Moneyball files, only `Penalties Saved`, `Red cards`, and `Mistakes Leading to Goals` remain notably sparse. Fresh synchronized agreement and neighboring-field structure are mandatory for these fields.
- Academy assists are intentionally absent because FM cannot export the required all-senior-career value. They are no longer a missing research input.
- The analyzer accepts declared compound, rounded-decimal, unavailable, and unit-bearing shapes. Currency ranges, localized wages, dates, and text enums are not direct scalar truth and remain outside automatic correlation.
- Known values in the exports remain useful anchors, but display values are not always identical to the existing product contract. For example, a localized transfer-value range is not the same field as the bridge's scalar `marketValueGbp`.
- The three current exports contain 101, 120, and 23 rows. Together they contain 244 rows, 235 distinct UIDs, and nine repeated UIDs with identical numeric values.
- Across the 235 distinct players, the non-zero evidence counts are 224 `AT Apps`, 190 `AT Gls`, 219 `AT League Apps`, 184 `AT League Goals`, 161 `Int Apps`, 103 `Int Gls`, and 55 `International Assists`. The corresponding distinct-value counts are 175, 70, 175, 66, 79, 27, and 6.
- `AT Apps` is never below `AT League Apps`, and `AT Gls` is never below `AT League Goals`. These relations are useful validation constraints but do not identify a memory location by themselves.
- `International Assists` is sparse and ranges only from 0 to 6. Its 55 non-zero rows must match; agreement on its 180 zero rows is not positive evidence.

## Walking skeleton

The first vertical slice uses fake memory to accept one research request containing one UID, capture bounded player/person ranges, and atomically write `probe.json` without touching production files. The production-expansion slice then uses fake memory containing one player, one staff member, one human manager, and one club to write schema v6, validate it in Rust, and ingest all promised records into one replacement snapshot. Existing UI queries must return the same player results.

## Delivery plan

### PR 1 — Add reusable FM memory research tooling

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(memory-probe): add reusable FM memory research tooling`

**Build-feature-loop profile:** Terra Max — the structure inventory and record reader add bounded full-region reference discovery and live container traversal.

**Purpose:** Deliver one complete developer workflow for bounded full-export capture, scalar and structured-record correlation, optional before/after comparison, and repeatable cross-sectional validation. The capture presets and analyzer share one isolated research protocol and therefore belong in one review and merge boundary.

**Depends on:** Completed FM26 memory read and bridge scan performance foundations. Youth Academy supplies the first downstream research need but is not changed by this PR.

#### Commit 1 — Capture bounded player memory by UID

**Status:** Completed

**Provisional commit:** `feat(memory-probe): capture bounded player memory`

**Work:** Add the isolated research request/status/output protocol, integrate it with the plugin's existing poller and scan gate, locate explicit UIDs through existing candidate discovery, capture fixed player/person windows and budgeted pointer targets, and write a versioned atomic result with fake-memory coverage.

**Out of scope:**

- CSV parsing or candidate correlation.
- Live FM offset discovery.
- Production request, status, diagnostics, or dump schema changes.
- Rust, React, SQLite, or Academy changes.

**Implementation packet:**

- Owners and files: `bridge/Plugin.cs`; focused additions under `bridge/Protocol/`, `bridge/Models/`, `bridge/Output/`, and `bridge/Research/`; bridge tests under `bridge/Tests/`.
- Existing patterns to verify: `RequestAcceptance` TTL and consume behavior, `Plugin.ScanGate` and cancellation, `LayoutRegistry` fail-closed resolution, `PersonScanner` candidate records, `IMemoryReader` safe reads, `DumpWriter`/`StatusWriter` atomic output, and `FakeMemoryReader` fixtures.
- Constraints and invariants: separate file names and schema version; explicit UID validation; fixed capture ceilings; aligned bounded pointer traversal; production full-dump priority; prior successful probe preservation on failure; no FM memory writes.
- Dependencies and ordering: reuse current candidate discovery without first optimizing it. Add the smallest isolated orchestration seam needed for fake-memory testing before wiring it into the plugin poll loop.

**Implementation profile:** Terra xhigh — the output is defined, but in-process lifecycle, cancellation, memory bounds, pointer traversal, and coexistence with the production scan gate require material local design judgment.

**Review profile:** Sol High — review must independently verify process-memory safety, file-protocol isolation, lifecycle behavior, and fail-closed bounds that pure happy-path tests could miss.

**Validation:** Start with a failing fake-memory request-to-output test through the new capture seam. Then run `./scripts/dev bridge-test` and `./scripts/dev check`. Evidence must show UID filtering, root-address derivation, pointer-budget enforcement, atomic success output, failure preservation, and unchanged production protocol files.

**Stop conditions:** Replan if safe capture requires arbitrary address input, unbounded or structure-specific traversal, a change to dump schema v5, Rust orchestration, or concurrent scans. Stop for developer input if production versus research request priority cannot preserve current Load Data behavior.

**Review mandate:**

- Verify every memory read is bounded and failures cannot crash or write to FM.
- Verify UID count, depth, target count, range size, and total-byte ceilings are enforced below the plugin boundary.
- Verify unsupported layouts, missing UIDs, partial reads, cancellation, and output failures produce a terminal research status without replacing a prior successful probe.
- Verify production requests retain priority and no two scans can run together.
- Verify probe files cannot alter or masquerade as schema-v5 dump/status/diagnostic files.
- Verify request identifiers and file handling cannot escape the bridge directory.

#### Commit 2 — Correlate CSV truth with probe captures

**Status:** Completed

**Provisional commit:** `feat(memory-probe): correlate CSV values with captures`

**Work:** Add a dependency-free .NET developer CLI and `./scripts/dev memory-probe` wrapper that can create and await probe requests, correlate mapped numeric CSV columns with one capture, and compare synchronized before/after CSV and capture pairs. Report ranked candidate paths, scalar encodings/transforms, coverage, deltas, unmatched data, and ambiguity.

**Out of scope:**

- Declaring any candidate a verified FM layout offset.
- Domain-specific parsing of appearance history or competition records.
- Product UI, IPC, persisted analysis results, charts, or general-purpose data science tooling.
- A new CSV or command-line package.

**Implementation packet:**

- Owners and files: new console project under `bridge/Tools/MemoryProbe/`; `bridge/FmDataBridge.sln` and project exclusions/references as required; deterministic fixtures and tests under `bridge/Tests/`; `scripts/dev` command dispatch.
- Existing patterns to verify: `scripts/dev` argument forwarding and `ensure_dotnet`; bridge camel-case JSON conventions; exact UID semantics; known layout encodings for UID, CA, PA, ×5 attributes, and 32-bit market value.
- Constraints and invariants: standard-library CSV/JSON handling; explicit column mappings; comma, semicolon, and tab exports with quoted fields; no silent row drops; compatible request/capture identity; deterministic ranking and output; hypotheses labeled as candidates.
- Dependencies and ordering: consume only the schema delivered by commit 1. Keep capture preparation, correlation, and diff as modes of one focused tool instead of separate executables.

**Implementation profile:** Terra xhigh — the command is bounded, but robust CSV normalization, multi-encoding correlation, deterministic ambiguity ranking, and synchronized diff validation have no existing repository analogue.

**Review profile:** Sol High — incorrect normalization or ranking could direct later memory work to a false offset, so review must challenge the evidence model and error reporting as well as CLI behavior.

**Validation:** Start with failing fixtures where duplicate small integers make a one-player match ambiguous and only multi-player coverage identifies the known relative path. Add a before/after fixture with one controlled scalar delta. Then run `./scripts/dev memory-probe --help`, `./scripts/dev bridge-test`, and `./scripts/dev check`.

**Stop conditions:** Replan if correct CSV parsing requires a new dependency, if analysis needs FM-specific history semantics, if the CLI must read production `dump.json` as raw truth, or if deterministic candidate provenance cannot survive pointer-path comparison.

**Review mandate:**

- Verify CSV quoting, delimiter detection or selection, header mapping, numeric parsing, UID uniqueness, and actionable error messages.
- Verify correlation checks all requested players and does not elevate one-player coincidences.
- Verify supported scalar widths, signedness, endianness, and FM ×5 transforms are explicit and tested.
- Verify before/after mode pairs the same UIDs and compatible capture metadata before reporting deltas.
- Verify unreadable bytes, duplicate candidate paths, stale result IDs, unmatched rows, and ties remain visible.
- Verify output distinguishes a candidate, an ambiguous candidate set, and no evidence; it must never say an offset is verified.

#### Commit 3 — Adapt real FM exports for bounded research

**Status:** Completed

**Provisional commit:** `feat(memory-probe): adapt real FM research exports`

**Work:** Raise the fixed request ceiling from 16 to 128 players so each current export can be captured unchanged. Extend the analyzer with declared normalization for the sample field shapes that represent numeric truth: missing cells, compound starts/substitute appearances, unit-bearing decimal values, rounded decimals, and justified floating-point or fixed-scale candidate encodings. Report eligible and excluded UIDs per field and keep unsupported display text explicit.

**Out of scope:**

- Automatic player selection or automatic inference from English header names.
- Treating currency ranges, localized wages, dates, text enums, or derived display metrics as exact scalar truth.
- Increasing per-player byte, pointer-count, or pointer-depth ceilings, or accepting more than 128 UIDs.
- Discovering, validating, or integrating Academy or Moneyball production offsets.

**Implementation packet:**

- Owners and files: `bridge/Protocol/ProbeRequestAcceptance.cs`; `bridge/Tools/MemoryProbe/MemoryProbeCli.cs`, `CsvPlayerTable.cs`, and `ProbeAnalysis.cs`; focused request and CLI tests under `bridge/Tests/`; this ledger for any field-shape deviation.
- Existing patterns to verify: exact UID semantics, fixed request and per-player bounds, deterministic report ordering, candidate encoding provenance, `evidenceSufficient`, and before/after compatibility checks.
- Constraints and invariants: 128 UIDs remain a hard ceiling; the unchanged per-player limit makes the maximum raw captured memory 180,224 bytes; every requested UID must still resolve exactly once; missing values reduce the per-field eligible set instead of dropping a CSV row; compound and unit-bearing values require a named transform; rounded decimal matching must use the declared display precision and must not be reported as exact evidence; unsupported text fails with an actionable message.
- Dependencies and ordering: extend the commit-1 request bound and the commit-2 analyzer without changing schema v1 or any per-player capture bound. Use sanitized fixtures that reproduce the observed shapes instead of committing the raw exports.

**Implementation profile:** Terra xhigh — the changes are localized, but decimal intervals, typed normalization, per-field evidence denominators, and the larger bounded request affect the credibility and safety of every later correlation.

**Review profile:** Sol High — review must challenge false-positive paths, rounding behavior, sparse evidence, total capture bounds, and large-request failure behavior before the tool is used to direct memory-layout work.

**Validation:** Start with failing request tests that accept 103 unique UIDs and reject 129, while preserving the 180,224-byte maximum raw-capture calculation. Add fixtures for `32 (5)`, a rounded decimal, a unit-bearing decimal, and `-`; prove that explicit transforms produce typed values, excluded UIDs remain visible, and an all-zero or one-player field cannot become a candidate. Then run `./scripts/dev memory-probe --help`, `./scripts/dev bridge-test`, and `./scripts/dev check`.

**Stop conditions:** Replan if a value shape needs an open-ended expression language, if reliable decimal comparison requires guessing an undocumented FM formula, if the capture schema or hard memory bounds must change, or if a sample field requires structure-specific history semantics rather than scalar correlation.

**Review mandate:**

- Verify 103-player exports work unchanged, 129 UIDs fail before scanning, duplicate and missing UIDs still fail, and the total raw-capture bound remains explicit.
- Verify missing or unsupported values cannot silently become zero or remove a player from unrelated fields.
- Verify compound appearances expose starts and substitute appearances as separate declared values.
- Verify rounded decimal, floating-point, fixed-scale, and unit conversions remain distinguishable in candidate provenance.
- Verify evidence sufficiency uses eligible selected players and still requires varied multi-player truth.
- Verify display-only values remain rejected unless the operator supplies a supported explicit transform.

#### Commit 4 — Document and validate the research workflow

**Status:** Completed

**Provisional commit:** `docs(memory-probe): document the research workflow`

**Work:** Add the durable developer runbook, document the separate probe files and command surface in the bridge README, and update current architecture after the code exists. Define the manual FM view, synchronization, varied-player correlation, independent-capture replication, optional before/after capture, artifact handling, and evidence required before a later feature may pin a production field.

**Out of scope:**

- Recording speculative Academy or Moneyball offsets.
- Committing raw CSV, raw memory, screenshots, or machine-specific paths.
- Expanding product documentation or presenting the probe as an end-user feature.

**Implementation packet:**

- Owners and files: new focused runbook under `bridge/`; `bridge/README.md`; `.wiki/ARCHITECTURE.md`; this ledger for validation evidence and discoveries.
- Existing patterns to verify: bridge Windows prerequisites and install steps, manual force-scan runbook, `.wiki/INDEX.md` ownership, `.work/` disposal rule, and completed memory-read validation language.
- Constraints and invariants: distinguish FM-exported labeled truth from production data flow; define exact statistic semantics before research; require synchronized captures, varied values, agreement across independent UID sets, and known-anchor recovery; keep raw artifacts untracked.
- Dependencies and ordering: document only commands and files delivered by commits 1 through 3. Run live validation with a freshly built and installed DLL before claiming the workflow works in FM.

**Implementation profile:** Luna Max — once the tool behavior is settled, the remaining work is a bounded runbook and evidence reconciliation task with strong repository analogues.

**Review profile:** Sol Medium — review should check that instructions match the implemented commands and preserve the product/research boundary without requiring new architectural judgment.

**Validation:** Run `./scripts/dev memory-probe --help`, `./scripts/dev bridge-test`, and `./scripts/dev check`. On Windows with FM 26.3 loaded, run the final manual validation below after `./scripts/dev bridge-install`; keep raw evidence under `.work/` and record only the concise result in this ledger.

**Stop conditions:** Return to the owning implementation commit if the documented command or schema differs from reality. Replan if the live probe cannot locate known UIDs, cannot recover known pinned fields, disturbs Load Data, or requires wider-than-approved capture bounds.

**Review mandate:**

- Verify every command, path, filename, prerequisite, and expected terminal state against implementation.
- Verify the manual CSV checklist states field semantics and synchronization requirements clearly.
- Verify known-offset recovery and agreement across independent synchronized captures are mandatory before feature completion. Describe before/after evidence as optional.
- Verify raw memory and CSV artifacts are directed to `.work/` or external local storage and excluded from Git.
- Verify architecture describes only implemented behavior and still names schema v5 as the product dump contract.
- Verify the runbook does not imply that correlation alone validates a production offset.

#### Commit 5 — Select stable first-hop pointer paths across the cohort

**Status:** Completed

**Provisional commit:** `feat(memory-probe): select stable pointer paths`

**Work:** Replace the per-player first-valid-pointer loop with a cohort-level capture plan. Capture both roots for every requested UID, rank valid first-hop source paths by cross-player availability with deterministic tie-breaking, reserve separate fixed quotas for player-root and person-root paths, and capture the selected paths wherever they resolve safely. Add capture-policy metadata and the selected source paths to `probe.json` schema v2 so later reports can distinguish capture strategies. Keep request protocol v1 because the request shape does not change.

**Out of scope:**

- Following pointers found inside first-hop targets.
- Adding Academy or Moneyball field semantics to the capture service.
- Supplying source offsets, target addresses, target counts, or byte limits through the request.
- Changing production dump schema v5, layouts, persistence, or app behavior.

**Implementation packet:**

- Owners and files: `bridge/Research/ProbeCaptureService.cs`; probe document models and protocol version under `bridge/Models/` and `bridge/Protocol/` as required; `bridge/Tests/ProbeCaptureTests.cs`; analyzer compatibility tests under `bridge/Tests/MemoryProbeCliTests.cs` if the result schema changes.
- Existing patterns to verify: root capture and fail-closed reads, `FindReadablePointers`, acceptable-region checks, target-address deduplication, relative-path provenance, atomic output replacement, cancellation, and deterministic JSON ordering.
- Constraints and invariants: root windows remain 0x280 and 0x100 bytes; target windows remain 128 bytes; select at most eight player-root and eight person-root paths; rank by eligible-player count before relative offset; make root quotas independent so player-root candidates cannot starve person-root candidates; retain missing per-player paths as visible coverage rather than substituting bytes; cap this commit at 2,944 bytes per player and 376,832 raw bytes for 128 UIDs; do not accept arbitrary operator addresses or offsets. Single-capture correlation continues to accept schema-v1 artifacts such as `academy-a`; incompatible cross-capture comparisons fail explicitly.
- Dependencies and ordering: restructure capture into root collection, cohort plan selection, then first-hop capture. Keep the analyzer generic over captured ranges. Do not add the second hop until the first-hop plan is deterministic and bounded.

**Implementation profile:** Terra xhigh — the change reorganizes live process-memory reads into phases and changes the research schema, while preserving cancellation, failure, and atomic-output guarantees.

**Review profile:** Sol High — review must independently verify deterministic cohort selection, fair root coverage, hard bounds, schema compatibility, and every existing process-memory safety boundary.

**Validation:** Start with a fake-memory test in which the old first-four policy selects only player-root targets and misses a later path shared by the cohort. Confirm the test fails for the current implementation. Then prove that the new planner selects the stable player path and at least one person-root path for every eligible player, reports unavailable paths without false bytes, produces identical path ordering for identical input, and enforces 2,944 bytes per player and 376,832 bytes for 128 UIDs. Prove that single-capture correlation still reads schema v1 and reads the new schema-v2 policy. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Replan if cohort planning needs unbounded pointer enumeration, operator-supplied offsets, process-wide graph state, or more than the fixed first-hop quotas. Stop for developer input if a schema change would make existing captures unusable without a clear analyzer error.

**Review mandate:**

- Verify source paths are ranked across the full requested cohort, not selected independently per player.
- Verify deterministic tie-breaking cannot depend on dictionary, region, or candidate scan order.
- Verify player-root and person-root quotas are enforced independently and total requested bytes cannot exceed the fixed ceiling.
- Verify invalid, duplicate, out-of-region, unreadable, or absent targets remain excluded and visible without corrupting another player's capture.
- Verify cancellation and any capture failure preserve the prior successful probe and all production protocol files.
- Verify older result schemas either remain readable or fail with an explicit compatibility message.

#### Commit 6 — Capture a bounded second pointer hop

**Status:** Completed

**Provisional commit:** `feat(memory-probe): capture bounded second-hop targets`

**Work:** Extend the cohort planner across the selected first-hop ranges and capture one additional pointer hop. Rank second-hop source paths by cross-player availability, retain full root-to-target provenance, and enforce a maximum of eight second-hop targets per player. Record the final depth, quotas, target-window size, and byte ceilings in the probe output.

**Out of scope:**

- A third pointer hop, recursive traversal, variable target windows, or operator-configurable capture budgets.
- Inferring collections, record counts, competition types, or statistic meanings inside captured bytes.
- Choosing or integrating production Academy or Moneyball offsets.
- Optimizing the existing full candidate scan.

**Implementation packet:**

- Owners and files: `bridge/Research/ProbeCaptureService.cs`; probe models and protocol metadata; focused fake-memory tests in `bridge/Tests/ProbeCaptureTests.cs`; correlation fixtures only where needed to prove that depth-two paths retain stable provenance.
- Existing patterns to verify: commit 5's cohort plan, aligned readable pointer discovery, acceptable committed regions, target-address deduplication, deterministic relative paths, analyzer range enumeration, cancellation, and prior-output preservation.
- Constraints and invariants: pointer depth is exactly two at most; select at most eight second-hop paths across the chosen first-hop ranges; each target remains 128 bytes; cycles and duplicate target addresses do not consume extra reads; the complete maximum is 3,968 bytes per player and 507,904 raw bytes for 128 UIDs; all ceilings remain compiled-in and fail closed below the plugin boundary.
- Dependencies and ordering: build only on the deterministic first-hop plan from commit 5. Keep the capture generic and let correlation decide whether any bytes match the declared CSV truth.

**Implementation profile:** Terra xhigh — bounded graph traversal is small in breadth but safety-critical because cycles, duplicate targets, partial availability, and provenance interact across 128 players.

**Review profile:** Sol High — review must challenge every path by which traversal could exceed its depth, count, address-region, or byte ceilings and must verify that reported paths remain reproducible.

**Validation:** Start with a fake-memory graph where the labeled value exists only behind a pointer inside a first-hop target. Confirm the current depth-one capture produces no candidate. Then prove the revised capture finds the depth-two range, rejects a third hop, deduplicates cycles and alias targets, preserves deterministic path ordering, and enforces 3,968 bytes per player and 507,904 bytes for 128 UIDs. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Do not widen the generic traversal again if the Academy target values remain absent. Record that result and replan a structure-specific history reader. Replan immediately if the byte ceiling, acceptable-region boundary, or production protocol isolation cannot be enforced deterministically.

**Review mandate:**

- Verify no path can reach pointer depth three or exceed eight second-hop targets.
- Verify cycles, alias pointers, zero values, unaligned cells, and out-of-region targets cannot expand the capture or hide missing evidence.
- Verify the per-player and full-request byte calculations include both roots and every possible target.
- Verify every second-hop range retains its complete root-relative source path and correct pointer depth.
- Verify correlation can compare depth-two paths without treating an absent path as zero.
- Verify production request priority, scan exclusion, cancellation, and atomic output behavior remain unchanged.

#### Commit 7 — Revise and repeat the live evidence workflow

**Status:** Completed

**Provisional commit:** `docs(memory-probe): revise live evidence workflow`

**Work:** Update the runbook and bridge protocol summary for the new capture policy and limits. Replace the mandatory disjoint-UID rule with fresh synchronized evidence that has meaningful value variation, while keeping a different cohort as preferred evidence when it is available. Reinstall the bridge, recapture the same 103-player Academy export with a new request ID, rerun correlation, and record concise results and any remaining replan condition in this ledger.

**Out of scope:**

- Committing the Academy CSV, probe JSON, correlation JSON, raw addresses, or machine-specific paths.
- Declaring a weak or ambiguous match to be a verified offset.
- Implementing discovered fields in the production bridge, schema v5, SQLite, Youth Academy, or Moneyball.
- Requiring a controlled before/after state or another Academy squad export that FM cannot provide.

**Implementation packet:**

- Owners and files: `bridge/MEMORY_PROBE.md`; `bridge/README.md`; `.wiki/ARCHITECTURE.md` only if implemented capture behavior is described there; this ledger for concise live evidence and plan changes.
- Existing patterns to verify: absolute `.work/memory-probe/` paths, one-use request IDs, supported FM 26.3 layout checks, exact UID-set checks, known-anchor correlation, explicit field transforms, research/product separation, and untracked raw evidence.
- Constraints and invariants: reuse the unchanged Academy CSV and full 103-player UID set; use a new request ID after installing and restarting the bridge; treat this recapture as candidate discovery under the new capture policy, not as independent proof by itself; require UID, CA, PA, and applicable attribute anchors to recover before evaluating target paths; report weak, ambiguous, or absent evidence honestly.
- Dependencies and ordering: run only after commits 5 and 6 pass automated validation. The developer performs the in-game capture. The documentation commit records only commands that were verified against the final implementation and concise results from the untracked artifacts.

**Implementation profile:** Luna Max — the code behavior is settled; the remaining work is exact procedural writing and evidence reconciliation from a manual FM run.

**Review profile:** Sol Medium — review should verify that the instructions match the final schema and that the evidence language does not overstate what one cohort proves.

**Validation:** Run `./scripts/dev memory-probe --help`, `./scripts/dev bridge-test`, `./scripts/dev check`, and `./scripts/dev bridge-install`. With the matching FM state open, capture the unchanged 103-player Academy CSV under a new request ID and correlate UID, CA, PA, Determination, `AT Apps`, `AT Gls`, and `Int Apps`. Confirm policy metadata, exact UIDs, hard byte totals, root coverage, and depth-two provenance before interpreting candidates.

**Stop conditions:** If the three Academy targets still have no strong candidate in any bounded pointer-target range, do not add depth or breadth. Record the negative result and replan a structure-specific reader for career-history records. Stop for developer input if the loaded FM state no longer matches the existing CSV.

**Review mandate:**

- Verify the runbook no longer requires disjoint UIDs or a controlled state transition.
- Verify it distinguishes candidate discovery, fresh verification evidence, and later production integration.
- Verify all documented ceilings and schema metadata match commits 5 and 6.
- Verify the Academy recapture uses the same synchronized 103-player CSV with a new request ID and keeps all raw artifacts untracked.
- Verify known anchors recover before any Academy target is considered.
- Verify a negative live result triggers structure-specific replanning instead of a wider generic memory scan.

#### Commit 8 — Inventory bounded player-linked structures

**Status:** Active

**Provisional commit:** `feat(memory-probe): inventory player-linked structures`

**Work:** Add a separate history-research capture preset that scans the existing acceptable memory regions once for exact references to each requested UID, person-object address, and player-block address. Group hits across the cohort by stable object or record provenance, rank those groups deterministically, and capture one fixed context for each selected player/signature pair. Record the anchor kind, anchor offset, structural signature, cohort coverage, total hit count, and fixed limits in a new versioned probe schema.

**Out of scope:**

- Following a candidate container, choosing record offsets, or aggregating career values.
- Increasing the generic preset's pointer depth, path quotas, target size, or byte ceilings.
- Accepting a process address, offset, stride, count, signature, or byte limit from the request.
- Adding FM UI bindings, invoking game mutators, or introducing a checked-in dependency on machine-local interop assemblies.

**Implementation packet:**

- Owners and files: focused additions under `bridge/Research/`; `bridge/Research/ProbeCaptureService.cs` only for preset routing and shared roots; probe request/document models and protocol versions under `bridge/Models/` and `bridge/Protocol/`; `bridge/Tools/MemoryProbe/` for preset selection and schema loading; focused fake-memory tests under `bridge/Tests/`.
- Existing patterns to verify: `RegionEnumerator.GetCandidateRegions`, `PersonScanner`'s block scan and cancellation checks, module-relative vtable validation, dynamic class-offset lookup, cohort path ranking, `ProbeWriter` atomic replacement, request priority, and `FakeMemoryReader` sparse-region fixtures.
- Constraints and invariants: search only the three exact anchor kinds for requested players; scan each acceptable region at most once; exclude the known root occurrences; never serialize unselected heap bytes; select at most eight structural signatures per anchor kind; capture at most one 128-byte context per selected signature and player; retain roots for known-anchor checks; enforce 4,480 bytes per player and 573,440 raw bytes for 128 UIDs; treat multiple hits for one player/signature as visible multiplicity, not interchangeable evidence; do not promote a hit that lacks stable object or repeated-record provenance.
- Dependencies and ordering: preserve the generic schema-v2 preset and its analyzer compatibility. The history preset may use a new request protocol and result schema because its policy and output differ. Capture and inspect all three current cohorts before Commit 9 fixes a record recipe.

**Implementation profile:** Terra Max — this adds a second full-region scan with cross-player reference grouping, normalized provenance, strict memory limits, and a new research schema inside the live FM process.

**Review profile:** Sol High — review must challenge scan cost, false structural grouping, address safety, deterministic selection, protocol compatibility, and every path that could retain more heap data than the fixed policy permits.

**Validation:** Start with fake memory where a history object contains a requested player's person pointer and career counters, but the player and person roots contain no pointer back to that object. Confirm the generic preset cannot capture it. Then prove the history preset finds the reference, normalizes the same object/field signature across several players, keeps multiple-hit counts visible, excludes unrelated UID bytes, produces deterministic ordering under permuted region and player input, and enforces the 4,480-byte player and 573,440-byte request ceilings. Prove malformed presets, cancellation, unsafe regions, missing UIDs, and output failure preserve the prior successful probe and all production files. Run `./scripts/dev bridge-test` and `./scripts/dev check`. After installation, capture the 101-, 120-, and 23-player cohorts and record only the stable signature inventory in this ledger.

**Stop conditions:** Replan instead of widening the inventory if no object-backed or repeated-record signature appears across meaningful non-zero players. Stop if a safe inventory requires retaining every heap hit, an operator-supplied address, more than one region pass, a generic reverse-pointer graph, or a machine-local interop build dependency. Reassess Commit 9 if club and international evidence resolve through different structures.

**Review mandate:**

- Verify the scan compares only exact requested anchors and does not become a general value search or heap dump.
- Verify every context is fully inside an acceptable committed region and every count and byte ceiling is enforced below plugin request handling.
- Verify structural signatures use stable provenance rather than absolute addresses, scan order, or first-hit order.
- Verify multiple hits, absent signatures, and excluded hits remain visible without synthesizing zero-filled contexts.
- Verify schema and preset compatibility errors are explicit in capture, correlation, and diff modes.
- Verify scan-gate priority, cancellation, shutdown, atomic replacement, and production-file isolation remain unchanged.

#### Commit 9 — Capture bounded career-history records

**Status:** Pending

**Provisional commit:** `feat(memory-probe): capture bounded career records`

**Work:** Use Commit 8's live structural evidence to add the smallest FM 26.3 research recipe that reaches the identified club and international career containers. Validate each observed descriptor, enumerate its records, and serialize bounded record slices with stable collection and record provenance. Keep the recipe versioned with the supported layout and expose it only through the history preset.

**Out of scope:**

- Guessing a container shape that Commit 8 did not reproduce across the live cohorts.
- Adding a general object browser, general collection parser, arbitrary recipe language, or operator-configurable record reads.
- Assigning statistic meanings, calculating totals, or changing production extraction and schema v5.
- Loading or invoking FM UI panels to make history data appear.

**Implementation packet:**

- Owners and files: a focused versioned recipe under `bridge/Layouts/` or `bridge/Research/`; structured-record additions to `bridge/Models/ProbeDocument.cs`; capture orchestration under `bridge/Research/`; schema validation under `bridge/Tools/MemoryProbe/ProbeAnalysis.cs`; fake-memory fixtures under `bridge/Tests/`.
- Existing patterns to verify: layout fail-closed resolution, checked address arithmetic, acceptable-region membership, fully readable ranges, pointer/alias deduplication, selected-path provenance, atomic output, and analyzer schema validation.
- Constraints and invariants: the recipe records every pointer hop, descriptor field, count field, record stride, and selected record slice; all arithmetic is checked before reading; counts must be plausible and internally consistent; any cap that would truncate a player's required records fails the capture instead of producing an aggregateable partial set; allow at most two career collections, 128 records per collection, and 64 captured bytes per record; enforce 18,432 raw bytes per player and 2,359,296 bytes per 128-UID request; keep all bounds compiled in and layout-versioned.
- Dependencies and ordering: Commit 8 must identify the descriptor shape and its stable provenance before this packet is implemented. Update the exact recipe and ceilings in this ledger if live evidence requires a narrower shape; replan if it requires a materially broader one.

**Implementation profile:** Terra Max — the implementation converts observed raw structure into safe collection traversal where one bad count, stride, or pointer could expand process-memory reads or produce silently incomplete totals.

**Review profile:** Sol High — review must independently verify the evidence-to-recipe link, overflow and truncation behavior, record provenance, layout versioning, and hard bounds.

**Validation:** Start with a fake descriptor and record array that reproduces the exact shape observed in Commit 8. Confirm the inventory-only preset cannot expose all row values. Then prove the recipe captures every required record in deterministic order, rejects negative or excessive counts, invalid begin/end or data/count relations, multiplication and addition overflow, partial records, out-of-region arrays, aliases, and unsupported layouts. Prove a player at the record cap succeeds and one above it fails without replacing prior output. Run `./scripts/dev bridge-test` and `./scripts/dev check`, then recapture all three cohorts and confirm the same recipe resolves for every requested UID.

**Stop conditions:** Replan if the cohorts do not share a stable descriptor and record shape, if records appear only after opening an FM UI panel, if complete rows exceed the fixed request ceiling, or if club and international structures cannot be represented as at most two explicit recipes. Do not proceed to aggregation with truncated or structurally ambiguous records.

**Review mandate:**

- Verify every descriptor field and record slice traces to Commit 8 live evidence and a supported FM layout.
- Verify corrupt or excessive counts fail before allocation, address arithmetic, or output replacement.
- Verify record ordering and paths do not depend on absolute addresses or dictionary enumeration.
- Verify no missing, partial, duplicate, or aliased row can enter a complete collection silently.
- Verify the recorded policy lets the analyzer distinguish complete, empty, absent, and rejected collections.
- Verify the history recipe does not alter the generic preset or any product extraction path.

#### Commit 10 — Correlate structured career totals

**Status:** Pending

**Provisional commit:** `feat(memory-probe): correlate career record totals`

**Work:** Extend correlation for structured history captures. Compare declared CSV fields with direct values in collection headers or record slices and with explicit aggregates justified by the observed record semantics. Report each candidate's collection path, scalar offset and encoding, record-selection rule, aggregation rule, raw contributions, total coverage, non-zero coverage, exclusions, and conflicts. Evaluate all seven current career targets together so neighboring fields and the all-senior versus league subset relations can challenge false matches.

**Out of scope:**

- Searching arbitrary record subsets until a sum matches, inferring competition meanings from English CSV headers, or hiding an unexplained FM formula behind a candidate.
- Treating an aggregate with missing or truncated records as evidence.
- Converting a research candidate into a production layout field, dump value, database column, Academy metric, or Moneyball statistic.
- Reconstructing prior-season snapshots or the unavailable all-senior assists total.

**Implementation packet:**

- Owners and files: structured capture validation and candidate enumeration in `bridge/Tools/MemoryProbe/ProbeAnalysis.cs`; CLI/report changes in `bridge/Tools/MemoryProbe/MemoryProbeCli.cs`; focused CSV and structured-record fixtures under `bridge/Tests/MemoryProbeCliTests.cs` or a dedicated test file.
- Existing patterns to verify: declared field mappings and transforms, eligible/excluded UID accounting, deterministic candidate ranking, exact versus rounded evidence, duplicate-path reporting, schema compatibility, and before/after separation.
- Constraints and invariants: analyze only complete collections; keep direct and aggregate evidence distinct; allow only recipe-declared record filters and aggregations; require the same candidate recipe across cohorts; require complete eligible-row coverage and report non-zero coverage separately; deduplicate the nine repeated UIDs when calculating combined support; enforce `AT Apps >= AT League Apps` and `AT Gls >= AT League Goals` as consistency checks; never map `International Assists` to Academy assists.
- Dependencies and ordering: consume only the structured schema and exact record semantics delivered by Commit 9. If Commit 9 exposes direct summary counters, retain record aggregation only where it proves another required field or future reuse.

**Implementation profile:** Terra xhigh — the work is offline and bounded, but candidate aggregation and sparse-value evidence can create convincing false matches unless provenance and denominators remain exact.

**Review profile:** Sol High — review must challenge zero-driven candidates, accidental subset fitting, duplicate-player inflation, incomplete collections, semantic mixing, and deterministic ranking across three captures.

**Validation:** Start with a fixture where a zero-heavy wrong path outranks the real field by total matches but fails non-zero coverage. Add direct-summary and multi-record fixtures for all-senior totals, league subsets, and international totals. Prove raw contributions sum to each reported aggregate, repeated UIDs count once in combined evidence, missing records invalidate aggregation, relationship violations remain visible, and ties remain ambiguous. Run the exact seven-field correlations for the three live captures, then run `./scripts/dev memory-probe --help`, `./scripts/dev bridge-test`, and `./scripts/dev check`.

**Stop conditions:** Stop for developer input if FM's exported meaning cannot be reproduced from visible record fields without an undocumented filter or formula. Replan if exact aggregation needs a general expression language, if different cohorts require different recipes, or if any target matches only zero-valued players.

**Review mandate:**

- Verify candidate acceptance requires every eligible row and every eligible non-zero row, not a high total dominated by zeros.
- Verify combined coverage uses 235 distinct UIDs and reports the nine overlaps separately.
- Verify every aggregate exposes its selected records, predicate, scalar encoding, and arithmetic.
- Verify club totals, league subsets, and international totals cannot be mixed across structures or fields.
- Verify sparse `International Assists` evidence cannot pass through zero agreement or a one-value coincidence.
- Verify reports still label every result as candidate, ambiguous, or no evidence rather than verified production data.

#### Commit 11 — Validate and document the career-history recipes

**Status:** Pending

**Provisional commit:** `docs(memory-probe): validate career history recipes`

**Work:** Reinstall the final bridge, capture the unchanged 101-, 120-, and 23-player CSV cohorts with new request IDs, and correlate UID, CA, PA, Determination, and all seven career targets. Record the exact versioned structure recipe or direct location, encoding, aggregation semantics, cohort coverage, non-zero coverage, and ambiguity for each target. Update the runbook, bridge protocol summary, and current architecture to describe only the implemented structure-aware workflow and its final fixed limits.

**Out of scope:**

- Committing raw CSVs, captures, correlation reports, absolute addresses, interop assemblies, or machine-specific paths.
- Calling a candidate a production offset or integrating it into schema v5, Rust, SQLite, Academy, or Moneyball.
- Relaxing the evidence contract because one field is sparse or because the three files share one save state.
- Adding another generic capture expansion when a target remains unresolved.

**Implementation packet:**

- Owners and files: `bridge/MEMORY_PROBE.md`; `bridge/README.md`; `.wiki/ARCHITECTURE.md`; this ledger for concise evidence, final recipes, and any unresolved target.
- Existing patterns to verify: new request IDs, exact UID-set checks, known anchors, capture-policy metadata, untracked artifacts, same-state synchronization, versioned layouts, and the candidate-versus-production boundary.
- Constraints and invariants: use all 244 rows and deduplicate to 235 players for combined evidence; require every eligible row and every eligible non-zero row to reproduce; confirm the nine repeated UIDs agree; require the same relative recipe and encoding in all cohorts where the field varies; preserve raw contribution evidence for derived totals; document unresolved fields honestly.
- Dependencies and ordering: run only after Commits 8 through 10 pass automated validation and their live evidence has fixed the recipe. If any of the seven targets fails the contract, keep the feature Active and replan that target instead of marking final validation complete.

**Implementation profile:** Luna Max — the remaining implementation is evidence reconciliation and exact operational writing once the structure and analyzer are settled.

**Review profile:** Sol High — final documentation determines whether later production work can trust the discovered locations and derivations, so review must verify every claim against the three untracked reports and implemented recipe.

**Validation:** Run `./scripts/dev memory-probe --help`, `./scripts/dev bridge-test`, `./scripts/dev check`, and `./scripts/dev bridge-install`. With the matching FM state open, create three new history-preset captures and reports. Confirm all requested UIDs, supported layout metadata, structural signatures, descriptor and record limits, complete collections, known anchors, exact non-zero coverage, repeated-UID consistency, and no equally strong conflicting candidate. Run a fresh-context Sol High review over the documentation and concise evidence before marking the commit complete.

**Stop conditions:** Do not mark PR 1 ready for publication unless every target is either supported by the full evidence contract or explicitly replanned as unresolved. Do not widen generic traversal. Stop for developer input if the source CSV state is no longer available or if a target's exported semantics remain uncertain after structured inspection.

**Review mandate:**

- Verify each of the seven target claims against all applicable non-zero rows in all three reports.
- Verify the documented recipe, limits, commands, schema versions, and report fields match implementation.
- Verify repeated UIDs agree but do not inflate the 235-player combined denominator.
- Verify direct and derived fields remain clearly distinguished and every aggregate states its exact inputs.
- Verify no absolute address or raw artifact enters Git and no weak candidate is presented as verified.
- Verify unresolved evidence leaves the feature Active with a concrete replan condition.

### PR 2 — Expand production memory and snapshot data

**Status:** Awaiting prior PR merge

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(memory-read): store remaining proven FM data`

**Build-feature-loop profile:** Terra Max — this PR expands native object discovery, adds a second entity family, and changes the C# dump, Rust validation, and SQLite snapshot contracts together.

**Purpose:** Port and persist practically all direct data emitted by the pinned FMSuperScout reader that the original MVP omitted. Use the completed probe to test dormant pins before the schema is locked. Keep every new field available for later features without adding product UI or speculative query layers.

**Depends on:** PR 1 merged. Use its generic probe and evidence rules for dormant player pins, but do not integrate PR 1's career-stat candidates in this PR.

#### Commit 1 — Discover staff, managers, and the full club graph

**Status:** Pending

**Provisional commit:** `feat(memory-read): discover staff managers and full clubs`

**Work:** Extend the typed full-region scan result beyond players. Classify the pinned pure-staff and human-manager objects, retain player/staff facets without duplicate output, collect plausible club objects already encountered by the object scan, and return the address maps required for complete squad and manager resolution. Preserve the existing player candidate contract for probe callers.

**Out of scope:**

- Reading staff attributes or remaining player fields.
- Changing dump schema v5, Rust, SQLite, or product UI.
- Parallelizing the scan, adding a memory snapshot, or changing acceptable memory regions.
- Treating any unrecognized class-offset histogram peak as a supported entity.

**Implementation packet:**

- Owners and files: `bridge/Scanning/PersonScanner.cs`; a small typed scan-result model under `bridge/Scanning/`; `bridge/Layouts/IFmMemoryLayout.cs` and `Fm263Layout.cs`; club structural checks under `bridge/Extraction/`; focused fake-memory tests.
- Existing patterns to verify: dynamic class-offset caching, module-vtable validation, in-buffer UID reads, candidate sanity checks, deterministic UID ordering, `ContractClubReader`, `SquadClubIndex`, cancellation, and scan diagnostics.
- Constraints and invariants: use only the pinned FM 26.3 player, player/staff, pure-staff, and human-manager class offsets; validate staff CA/PA before acceptance; require a plausible team-vector shape and club name before retaining a club object; deduplicate entity and club addresses deterministically; keep diagnostic caps explicit and mark the whole scan truncated when early termination can make staff or club results incomplete; production unlimited scans must enumerate the full accepted region set.
- Dependencies and ordering: land the typed discovery seam before any reader or schema assumes staff, manager, or global-club availability. Probe capture must continue to receive the exact player candidate list it uses today.

**Implementation profile:** Terra Max — one region pass will now classify several native object types and preserve cross-object address maps without changing existing player or probe semantics.

**Review profile:** Sol High — review must challenge false club classification, player/staff duplication, capped-scan completeness, cancellation, address provenance, and every compatibility path used by the probe.

**Validation:** Start with fake memory that contains one player, one player/staff object, one pure-staff object, one human manager, one valid club, and near-miss objects. Confirm the current player-only result loses the new entities. Then prove exact classification, deterministic deduplication, rejection of implausible clubs and abilities, cancellation, cap metadata, unchanged player ordering, and unchanged generic/history probe roots. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Replan if the pinned staff or manager classes do not reproduce on the live FM 26.3.2 save, if reliable club discovery requires an unbounded name scan, or if one scan result cannot serve production and probe callers without weakening their current contracts.

**Review mandate:**

- Verify only supported class offsets create typed candidates.
- Verify every accepted address lies inside an acceptable committed region and all base-address arithmetic is checked.
- Verify club candidates require both structural and name evidence and cannot be created from arbitrary vectors.
- Verify player/staff facets and duplicate UIDs have one documented, deterministic outcome.
- Verify capped, cancelled, and failed scans expose incomplete results and cannot replace a complete production dump.
- Verify PR 1 probe behavior and byte ceilings remain unchanged.

#### Commit 2 — Read the remaining proven upstream fields

**Status:** Pending

**Provisional commit:** `feat(memory-read): read remaining proven FM fields`

**Work:** Add typed readers for the direct fields emitted by the pinned upstream plugin. Players gain nation UID, gender, club reputation, and raw team type alongside the existing derived team level. Save metadata gains manager UID and name, managed club and reputation, database scope, and an honest date basis. Staff records gain UID, identity, DOB and age, nation and nation UID, gender, CA, PA, 22 staff attributes, stable job ID, wage, contract expiry, club, and division. Use full-club discovery for squad resolution while retaining contract-seeded fallback.

**Out of scope:**

- Emitting schema v6 or storing any new field.
- Localized staff job names, coaching-star formulas, staff search, or staff profiles.
- Condition, morale, home reputation, squad number, staff reputation, or the second value slot before Commit 3 validates them.
- Asking-price, wage-demand, transfer-interest, meta-score, or potential-projection fields derived by the upstream app.

**Implementation packet:**

- Owners and files: focused readers under `bridge/Extraction/`; production models under `bridge/Models/`; request parsing under `bridge/Protocol/` for the bounded `databaseScope` enum; `CapADumpPipeline.cs` for orchestration; layout pins under `bridge/Layouts/`; bridge tests.
- Existing patterns to verify: `PlayerIdentityReader`, `PlayerAttributeReader`, `PlayerContractReader`, `ContractClubReader`, `SquadClubIndex`, `GameDateResolver`, nullable scalar reads, contiguous attribute batches, and deterministic squad selection.
- Constraints and invariants: default `databaseScope` to `men`; represent gender and scope with closed enums; retain a numeric nation UID separately from the localized nation name; keep raw team type alongside `TeamLevelMap`; use numeric staff job ID as the stable contract; keep unread or impossible values null; never emit process addresses; label schedule consensus as `derived` with basis `next-fixture-consensus`; do not call it the world date.
- Dependencies and ordering: use Commit 1's typed candidates and full club set. Build and test readers behind the existing v5 writer so the production contract changes only once in Commit 4.

**Implementation profile:** Terra Max — the field offsets are known, but staff batching, manager selection, gender scope, full-club squad precedence, date honesty, and null semantics span several native object chains.

**Review profile:** Sol High — review must verify every field basis and encoding against pinned source provenance, ensure localized strings are not mistaken for stable IDs, and challenge manager, loan, and team selection ambiguity.

**Validation:** Add fake-memory tests for every player, staff, manager, club, competition, contract, and date field. Prove male, female, and mixed scope behavior; player/staff deduplication; raw team type plus derived level; nation name plus UID; manager selection; full-club squad precedence; null handling; and next-fixture date labeling. Run `./scripts/dev bridge-test` and `./scripts/dev check`.

**Stop conditions:** Stop if a field emitted by the pinned upstream reader cannot be reproduced with the documented object path, if women's records require an unpinned class, or if manager and managed-club selection remain ambiguous across the live save. Omit rather than guess any field whose stable identity or unit cannot be shown.

**Review mandate:**

- Verify each field traces to a pinned object basis, offset, width, transform, and null rule.
- Verify all 22 staff attributes use the correct stored-times-five transform and stable keys.
- Verify database scope never mislabels an incomplete men's, women's, or mixed scan.
- Verify full-club results cannot override a stronger contract or squad association with a weaker heuristic.
- Verify schedule-derived dates no longer claim direct world-clock precision.
- Verify the current v5 writer and existing player UI behavior remain unchanged in this commit.

#### Commit 3 — Validate the dormant upstream pins

**Status:** Pending

**Provisional commit:** `feat(memory-probe): validate dormant FM field pins`

**Work:** Use the completed probe and small synchronized FM samples to test the upstream constants that its current product does not emit: player condition, morale, home reputation, the second value slot, contract squad number, and staff home, current, and world reputation. Record the exact encoding, transform, null sentinel, coverage, and contradictions for each candidate. Add only confirmed fields to the typed readers that will feed schema v6.

**Out of scope:**

- Treating an upstream constant, one known player, or plausible range as verification.
- Blocking the proven field set when FM cannot export or display trustworthy truth for a dormant candidate.
- Searching for injuries, happiness, tactical familiarity, promises, release clauses, or other fields without an upstream pin and explicit evidence input.
- Expanding probe breadth, pointer depth, or byte ceilings.

**Implementation packet:**

- Owners and files: explicit mappings or transforms under `bridge/Tools/MemoryProbe/`; focused candidate readers under `bridge/Extraction/` only after validation; bridge tests; this ledger for concise live conclusions. All CSVs and reports remain under `.work/memory-probe/`.
- Existing patterns to verify: player-root coverage through `0x280`, contract pointer provenance, explicit field normalization, eligible and non-zero coverage, unread masks, supported-layout checks, and candidate-versus-production labels.
- Constraints and invariants: test `PLAO_CONDITION 0x258`, `PLAO_HOME_REP 0x25E`, `PLAO_MORALE 0x26C`, `PLAO_TRANSFER_VALUE 0x238`, `CON_SQUAD_NUMBER 0x5D`, `NPLO_HOME_REP 0xD4`, `NPLO_CUR_REP 0xD6`, and `NPLO_WORLD_REP 0xD8`; require varied synchronized truth wherever FM exposes it; record unavailable truth as unresolved; omit unresolved or contradicted candidates from schema v6; never rename the second value slot until its semantics are established.
- Dependencies and ordering: run after Commit 2 supplies staff candidates and before Commit 4 freezes schema v6. Reuse PR 1 rather than adding another generic probe mechanism.

**Implementation profile:** Terra xhigh — the reads are bounded and mostly direct, but field semantics, transforms, sentinel handling, and evidence sufficiency require careful live interpretation.

**Review profile:** Sol High — review must reject plausible-but-unproven fields, zero-dominated correlations, mislabeled value semantics, and any production reader that exceeds the recorded evidence.

**Validation:** Capture or inspect synchronized values for a varied player and staff sample where FM exposes ground truth. Confirm known anchors first. For each candidate, report eligible rows, non-zero rows, exact matches, conflicts, encoding, and transform. Add fake-memory boundary and null tests only for accepted fields. Run `./scripts/dev memory-probe --help`, `./scripts/dev bridge-test`, and `./scripts/dev check`.

**Stop conditions:** If FM supplies no trustworthy truth for a candidate, record it as unresolved and omit it; this does not block the proven upstream-emitted fields. Replan only if validating a candidate reveals that an already promised field uses a materially different structure or meaning.

**Review mandate:**

- Verify every accepted dormant field has varied live truth and no unexplained conflict.
- Verify unresolved and rejected pins remain absent from production models and schema plans.
- Verify the second value slot is not called asking price, transfer value, or guide value without direct evidence.
- Verify staff reputation uses the staff block basis rather than player or team reputation offsets.
- Verify all raw artifacts and absolute addresses remain untracked.
- Verify no probe safety limit changes.

#### Commit 4 — Publish and persist dump schema v6

**Status:** Pending

**Provisional commit:** `feat(snapshot): ingest expanded FM dump data`

**Work:** Make one atomic production-contract transition. Update the bridge models and writer to emit schema v6 with the proven player, staff, manager, scope, club, and date-basis fields. Update Rust validation, the golden fixture, and explicit stale-schema errors. Add the next SQLite migration, then ingest every promised field transactionally into expanded snapshots and players plus a new snapshot-owned staff table.

**Out of scope:**

- Career or Moneyball statistic integration.
- Staff, manager, gender, condition, morale, or database-scope UI.
- Staff query commands, search indexes beyond snapshot/name lookup, role scoring, or derived coaching ratings.
- Backfilling old snapshots with invented values or silently accepting v5 as v6.

**Implementation packet:**

- Owners and files: `bridge/Models/DumpDocument.cs`, `bridge/Output/DumpWriter.cs`, `bridge/Protocol/BridgeProtocol.cs`, `bridge/DUMP_SCHEMA.md`, bridge serialization tests; `src-tauri/src/features/memory_read/dump_validation.rs` and fixtures; `src-tauri/src/db/migrations.rs`; `src-tauri/src/features/snapshot/ingest.rs` and focused migration/ingest tests.
- Existing patterns to verify: exact schema-version validation, required-versus-nullable field checks, compact streaming output, replace-only-on-success, transactional snapshot insertion, current-snapshot replacement, cascade deletion, JSON attribute storage, and migration replay tests.
- Constraints and invariants: `playerCount` and `staffCount` must equal their arrays; `scanTruncated` applies to the completeness of every emitted entity family; v6 records database scope and date basis; stale v5 returns a clear refresh-required error; snapshots store manager UID/name, managed club/reputation, staff count, database scope, and date basis; players store new scalar fields without changing existing query DTOs; staff uses `(snapshot_id, uid)` identity plus `staff_attributes_json`; every accepted dormant field follows Commit 3's exact null and transform rules; one transaction inserts snapshot, players, staff, role scores, and Academy class effects or rolls back all of them.
- Dependencies and ordering: Commits 1 through 3 lock the field set and readers. Do not bump the production schema until bridge output, Rust validation, migration, and ingest are all present in this commit.

**Implementation profile:** Terra Max — this is a cross-language, persisted contract change with large-record performance, migration, rollback, and stale-dump consequences.

**Review profile:** Sol High — review must compare every schema field across C# output, Rust validation, SQLite storage, tests, and documentation, with special attention to silent data loss and partial replacement.

**Validation:** Start with a schema-v6 golden dump containing one player, one staff member, manager metadata, mixed nullable values, and all accepted dormant fields. Confirm current validation rejects it before the change. Then prove exact bridge serialization, Rust type validation, migration from every prior schema, complete ingest, staff and player cascade replacement, stale-v5 error text, duplicate UID rejection per entity family, transaction rollback, and unchanged existing player queries. Measure validation, insert time, database growth, and dump size on a representative large fixture. Run `./scripts/dev bridge-test`, `./scripts/dev test`, and `./scripts/dev check`.

**Stop conditions:** Replan if the expanded live dump cannot be written and ingested within practical resource limits, if SQLite storage requires a premature staff-domain API, or if any promised field would validate but not persist. Do not split the production version transition across publishable commits.

**Review mandate:**

- Verify one authoritative schema-v6 field list matches bridge, Rust, fixture, migration, ingest, and documentation.
- Verify every new field is persisted or explicitly excluded before validation succeeds.
- Verify old v5 artifacts fail with a useful rescan instruction and cannot replace the current snapshot.
- Verify staff and player duplicate rules, foreign keys, cascades, and transaction rollback.
- Verify no existing player query, score, Planner, Academy, or Search behavior changes accidentally.
- Verify large arrays stream and ingest without avoidable per-record parsing or index overhead.
- Verify no process address, raw probe evidence, or machine-local source enters the dump or repository.

#### Commit 5 — Validate and document the expanded reader

**Status:** Pending

**Provisional commit:** `docs(memory-read): validate expanded FM data`

**Work:** Install the schema-v6 bridge and validate a live full scan against known players, staff, manager, clubs, loans, database scope, and every accepted dormant field. Confirm the dump validates and persists without changing existing UI results. Update the bridge runbook, dump contract, architecture, and this ledger with concise field provenance, limitations, counts, timings, and unresolved pins.

**Out of scope:**

- Adding UI for the stored data.
- Claiming women's or mixed-database coverage without a live representative scan.
- Retaining raw dumps, personal names, memory addresses, or diagnostic artifacts in Git.
- Starting PR 3 scan optimization before schema-v6 semantic parity is recorded.

**Implementation packet:**

- Owners and files: `bridge/README.md`; `bridge/DUMP_SCHEMA.md`; `.wiki/ARCHITECTURE.md`; this ledger; focused corrections only when live validation exposes a bounded contract defect.
- Existing patterns to verify: `./scripts/dev bridge-install`, version fail-closed behavior, status/dump replacement, Rust validation, Load Data ingest, current-snapshot replacement, diagnostics timing, and untracked `.work/` evidence.
- Constraints and invariants: verify all emitted fields against source truth where practical; record staff/player counts and duplicate handling; confirm player and staff array lengths; confirm manager and managed-club selection; confirm scope and date basis; compare existing Search and Academy results before and after; record dormant pins as accepted, rejected, or unresolved; do not call next-fixture consensus the exact game date.
- Dependencies and ordering: run after schema-v6 automated validation passes. This commit makes PR 2 ready for publication but does not start staff product work.

**Implementation profile:** Luna Max — the code path is fixed; the work is live evidence reconciliation and precise documentation across bridge and persistence contracts.

**Review profile:** Sol High — documentation will become the source for later staff and player features, so every field, omission, limitation, and performance claim must match code and live evidence.

**Validation:** Run `./scripts/dev bridge-test`, `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev bridge-install`. With FM 26.3.2 loaded, create a fresh unlimited schema-v6 dump, validate and ingest it, inspect representative player/staff/manager/club values, confirm replacement semantics, and record dump size plus phase timings. Run a fresh-context Sol High review over the exact PR 2 diff and concise live evidence.

**Stop conditions:** Do not publish PR 2 if any required v6 field is systematically wrong, silently lost, or ambiguous, if staff completeness cannot be established, or if the expanded dump causes impractical resource use. Omit non-required dormant candidates instead of weakening evidence.

**Review mandate:**

- Verify live field values and nulls against representative FM truth.
- Verify staff, manager, club, scope, and date-basis documentation matches the final v6 schema.
- Verify existing UI and Academy behavior remains unchanged.
- Verify timings and sizes are measured and labeled with the tested save.
- Verify dormant-field conclusions do not overstate weak or unavailable evidence.
- Verify no private live artifact enters Git.

### PR 3 — Harden production memory scans

**Status:** Awaiting prior PR merge

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `perf(memory-read): harden full FM scans`

**Build-feature-loop profile:** Terra Max — bounded parallel scanning and optional process-snapshot retry affect memory pressure, cancellation, deterministic deduplication, and prior-dump safety inside FM.

**Purpose:** Adopt the useful scan-engine improvements from the pinned upstream reader after schema-v6 field parity provides a stable baseline. Improve throughput and resilience without changing which data the bridge accepts, emits, or stores.

**Depends on:** PR 2 merged with recorded schema-v6 live counts, field samples, timings, and dump size.

#### Commit 1 — Parallelize deterministic region scanning

**Status:** Pending

**Provisional commit:** `perf(memory-read): parallelize deterministic region scans`

**Work:** Scan independent acceptable regions through bounded worker-local buffers and collections, then merge candidates, staff, managers, clubs, histograms, and diagnostics deterministically. Reuse large buffers between scans and reduce the worker count under measured physical-memory pressure. Preserve cancellation and every semantic result from PR 2.

**Out of scope:**

- Frozen-memory snapshots or retry policy.
- New entities, fields, layouts, schema versions, or persistence changes.
- Configurable worker counts or benchmark-only production switches.

**Implementation packet:**

- Owners and files: `bridge/Scanning/PersonScanner.cs` and typed scan-result helpers; `bridge/Memory/` for a narrow memory-status query if required; scan diagnostics; fake-memory and determinism tests.
- Existing patterns to verify: fixed scan block size, boundary overlap, dynamic-offset cache, region filtering, cancellation token, UID/address sorting, bridge scan gate, and upstream's capped worker-local collection pattern.
- Constraints and invariants: use at most eight workers; never allocate more than one fixed scan buffer per worker; choose a smaller fixed worker count when available physical memory crosses a documented threshold; merge by stable address and UID rather than task completion order; retain exact duplicate rules and histograms; make cancellation stop all workers; preserve one scan at a time through `ScanGate`.
- Dependencies and ordering: use PR 2's semantic fixture and live baseline as the equality oracle. Do not introduce retry until deterministic parallel output is proven independently.

**Implementation profile:** Terra Max — concurrency enters the native-memory hot path and can otherwise create non-deterministic candidates, excessive buffers, or late cancellation.

**Review profile:** Sol High — review must challenge task lifecycle, buffer ownership, deterministic merge, memory ceilings, cancellation races, exception propagation, and scan-gate isolation.

**Validation:** Make a deterministic fake reader block selected regions so the serial baseline is observably slow, then prove bounded concurrency. Compare complete serial and parallel results under permuted region order, duplicate objects, boundary overlap, unread gaps, cancellation, and worker exceptions. Run `./scripts/dev bridge-test` and `./scripts/dev check`, then measure one live schema-v6 scan against PR 2 without claiming improvement unless the before/after numbers support it.

**Stop conditions:** Replan if deterministic equality requires global locks in the scan hot loop, if memory usage exceeds the fixed worker-buffer budget, or if live throughput does not justify the added concurrency. Preserve the serial scanner if the measured tradeoff is unfavorable.

**Review mandate:**

- Verify worker count and buffer memory have hard upper bounds.
- Verify no candidate or diagnostic result depends on scheduling or first completion.
- Verify all worker failures and cancellation terminate cleanly without partial output replacement.
- Verify region overlap still catches objects at block boundaries exactly once after deduplication.
- Verify serial and parallel semantic results are identical.
- Verify no schema or extraction behavior changes.

#### Commit 2 — Retry materially incomplete scans from a frozen snapshot

**Status:** Pending

**Provisional commit:** `feat(memory-read): retry incomplete scans from snapshots`

**Work:** Measure unread bytes and region failures during the live scan. When a compiled-in read-quality threshold shows that the result is materially incomplete, release the live attempt and retry once through a PSS VA-clone memory source. Always release the clone, report the source and quality metrics, and preserve the prior good dump when the retry also remains incomplete.

**Out of scope:**

- Taking a snapshot for every scan, exposing snapshot controls, or retrying more than once.
- Using snapshot APIs on unsupported platforms or making them a build-time dependency outside Windows P/Invoke.
- Accepting partial schema-v6 dumps as complete.
- Changing extraction fields, ordering, or persistence.

**Implementation packet:**

- Owners and files: a focused snapshot-handle abstraction under `bridge/Memory/`; `WindowsMemoryReader` or a sibling reader for the clone; scan-quality diagnostics; pipeline retry orchestration; fake abstractions and Windows-guarded tests.
- Existing patterns to verify: `ReadProcessMemory`, `SafeHandle`-style disposal, `DumpWriter.TryWriteReplaceOnSuccess`, failure diagnostics, scan gate, cancellation, and FMSuperScout's live-first/snapshot-on-poor-read strategy.
- Constraints and invariants: define one compiled-in unread threshold from live evidence; retry at most once; never retain both attempt outputs; snapshot creation failure falls back to an explicit failed/incomplete result rather than a partial dump; dispose every native handle on success, failure, cancellation, and exception; check available commit memory before capture; record `live` or `snapshot-retry` plus unread metrics without exposing addresses.
- Dependencies and ordering: run after Commit 1 fixes deterministic parallel semantics. Keep snapshot use behind a narrow reader boundary so fake tests require no Windows process snapshot.

**Implementation profile:** Terra Max — native handle lifetime, copy-on-write memory pressure, retry state, cancellation, and prior-dump preservation create high-consequence failure paths inside the game process.

**Review profile:** Sol High — review must independently trace every handle and output lifecycle, threshold path, retry failure, cancellation edge, and interaction with production/probe priority.

**Validation:** Start with a fake live reader that returns unread coverage above the threshold and a complete snapshot reader; prove one retry and successful replacement. Add below-threshold, snapshot-creation failure, retry-incomplete, cancellation, exception, low-memory, and double-disposal tests. Confirm no incomplete attempt reaches `dump.json`. Run `./scripts/dev bridge-test` and `./scripts/dev check`, then force or observe a safe live retry on Windows before publication.

**Stop conditions:** Remove the snapshot retry from the plan if PSS VA clone cannot be bounded, reliably released, or exercised safely on the supported Windows build. Keep read-quality measurement and prior-dump preservation even if snapshot support is removed.

**Review mandate:**

- Verify every native handle is owned once and released on every exit path.
- Verify retry triggers only from measured incompleteness and runs at most once.
- Verify snapshot failure or low memory cannot crash FM or replace a prior dump with partial data.
- Verify cancellation and scan-gate priority remain correct across both attempts.
- Verify diagnostics state the source and quality without leaking addresses.
- Verify semantic output matches the live-reader baseline when both reads are complete.

#### Commit 3 — Validate scan parity and operating limits

**Status:** Pending

**Provisional commit:** `docs(memory-read): validate hardened scan behavior`

**Work:** Compare hardened schema-v6 scans with PR 2's semantic baseline, record measured worker, buffer, unread, retry, timing, and memory limits, and update bridge operations and architecture documentation. Keep only changes that improve or materially harden the supported workflow.

**Out of scope:**

- New data fields, schema changes, UI, staff features, or tuning controls.
- Performance claims without comparable measurements.
- Keeping temporary dumps, memory telemetry, or machine-specific diagnostics in Git.

**Implementation packet:**

- Owners and files: `bridge/README.md`; `.wiki/ARCHITECTURE.md`; this ledger; focused corrections only for parity or lifecycle defects found during live validation.
- Existing patterns to verify: bridge install, unlimited full dump, status and diagnostics, semantic count/sample comparison, process-memory measurements, and prior-dump preservation.
- Constraints and invariants: compare player/staff counts and representative field values against PR 2; explain any legitimate save-state drift; report worker count, allocated scan-buffer bound, unread fraction, source, retry count, elapsed phases, and process memory; keep raw evidence under `.work/`; remove any hardening layer whose measured cost exceeds its practical benefit.
- Dependencies and ordering: run after both scan changes pass automated tests and after FM has restarted with the final DLL.

**Implementation profile:** Luna Max — the main work is controlled live comparison and concise operational documentation after high-risk code has stabilized.

**Review profile:** Sol High — final review must verify semantic parity, measured claims, native-resource safety, and documentation accuracy across the full scan lifecycle.

**Validation:** Run `./scripts/dev bridge-test`, `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev bridge-install`. Produce comparable live full scans, confirm schema-v6 validation and ingest, compare entity counts and representative fields, exercise prior-dump preservation, and record measured timing and memory evidence. Run the feature-complete review selected below.

**Stop conditions:** Do not publish a hardening change that causes unexplained field or count drift, increases failure risk, or lacks safe Windows evidence. Revert that bounded hardening layer while retaining the stable expanded reader.

**Review mandate:**

- Verify final player, staff, manager, club, and field parity against PR 2.
- Verify every timing and memory statement comes from comparable measured runs.
- Verify worker, buffer, retry, and unread thresholds match code.
- Verify snapshot and cancellation lifecycles remain safe under failure.
- Verify documentation distinguishes implemented behavior from removed or rejected hardening.
- Verify no raw live artifact or private data enters Git.

## Active work

**PR:** PR 1 — Add reusable FM memory research tooling

**Commit:** Commit 8 — Inventory bounded player-linked structures

### RED proof

Create fake memory where a separate history object contains a requested player's exact person pointer and labeled counters, but neither player root points back to that object. The current generic depth-two preset must miss the history object. This proves that another generic outgoing hop cannot solve the observed reverse-link case.

### Expected outcome

The history preset finds exact UID and player-root references in one bounded region pass, groups them by stable structural provenance across the cohort, and captures only the selected 128-byte contexts. Its result records hit multiplicity, missing players, selection coverage, and the 4,480-byte player and 573,440-byte request ceilings without changing the generic preset.

### Explicit exclusions

- No container, record, or statistic offset is guessed before live signature evidence exists.
- No raw CSV, memory, screenshot, or machine-specific artifact is committed.
- No production schema-v5, Rust, SQLite, Academy, Moneyball, product-facing UI, interop dependency, or generic traversal expansion is added.

## Discoveries and replanning

Record material deviations, blockers, and decisions that change remaining work. State what was planned, what changed, and why.

- Planning selected a separate research protocol so the reusable tool cannot accidentally become part of the frozen product dump contract.
- Repowise showed `Plugin.cs`, `PersonScanner.cs`, and `CapADumpPipeline.cs` as bridge hotspots; its index was behind live `HEAD`, so file relationships and all planning facts were verified against current source.
- Commit 1 fixed the player root at 0x280 bytes and the person root at 0x100 bytes (1,408 bytes maximum per player). The player window covers the known CA, PA, Determination, and market-value anchors; unread root or target bytes now fail before output replacement.
- Commit 2 requires varied values across multiple players before analysis reports a candidate. Explicit request IDs are single-use while matching probe status or capture artifacts remain, because completion timestamps cannot distinguish an older in-flight scan from a same-ID retry.
- The first real exports contain 75 matching unique UIDs, while the original probe cap was 16. The revised pair contains 103 matching UIDs. Commit 3 raises the fixed cap to 128 so each export can remain intact without adding selection machinery.
- The Moneyball export introduced missing, compound, unit-bearing, and rounded decimal values that commit 2 cannot parse. Commit 3 adds only explicit numeric normalization and evidence reporting; display-only and semantically derived fields stay outside automatic correlation.
- The developer confirmed that FM cannot export the required Academy assists total, so it is no longer a research target. The developer also confirmed that `AT Apps`, `AT Gls`, and `Int Apps` cover all senior matches. The revised Academy file supplies strong variation for all three targets.
- The developer confirmed that Moneyball values are season-to-date at the point of recording and that starts and substitute appearances must be stored separately. Snapshot history may later preserve multiple points in the season, but it does not change the probe contract.
- A controlled before/after state is not expected to be practical, and FM cannot export a second Academy squad from the same state. The validation contract therefore accepts fresh synchronized evidence from the same varied cohort. A disjoint UID set remains useful when naturally available, but it is not mandatory. Diff remains optional.
- On 2026-08-07, `./scripts/dev bridge-install` built and installed the current DLL to the configured FM26 BepInEx plugins directory. FM was not running, so no live capture was attempted; the runbook's in-game evidence checklist remains required before feature completion.
- On 2026-08-07, the first live probe command exposed a wrapper working-directory defect: `scripts/dev memory-probe` changed to `bridge/` before passing relative paths to the CLI. The correction runs the command from the repository root and makes the runbook work directory absolute. It does not change the bridge protocol or capture semantics.
- On 2026-08-07, `academy-a` captured all 103 requested Academy players on FM 26.3.2 with no unread ranges. Correlation recovered UID at two known person-object representations, CA at two known player-block representations, PA uniquely at `player-block+0x266`, and Determination ×5 at `player-block+0x192` across 85 of 103 eligible players.
- The same report did not expose a credible path for `AT Apps`, `AT Gls`, or `Int Apps`. Their best root-window coincidences covered only 16, 45, and 35 of 103 eligible players, and none of the three fields produced a candidate in any captured pointer-target range.
- Inspection of the capture policy explained the gap. Each player received the two roots and the first four valid targets encountered while scanning the player root, so the quota was full before the person root was considered. No person-root target was captured, and traversal stopped at depth one even though captured first-hop ranges contain further stable pointer cells. This satisfies the existing replan condition for insufficient pointer reach.
- Commits 5 and 6 replace per-player first-four selection with bounded cohort-level path planning: eight first-hop paths per root and eight second-hop paths, all with 128-byte targets. The resulting fixed maximum is 3,968 bytes per player and 507,904 raw bytes for 128 UIDs. If the revised capture still lacks the targets, the next plan must use a structure-specific history reader rather than a wider generic graph walk.
- On 2026-08-07, `academy-depth2` recaptured the unchanged 103-player Academy cohort on FM 26.3.2. The bridge reported `ready`, captured every requested UID with no unread ranges, selected 16 first-hop and eight second-hop source paths, and captured 398 depth-two ranges. The capture requested and read 334,464 bytes in total, and no player exceeded 3,456 bytes. Both figures stay below the respective 507,904-byte request and 3,968-byte player ceilings. Correlation recovered full-cohort UID, CA, and PA anchors and the known Determination ×5 path for 85 players.
- The new target evidence remains insufficient. `AT Apps`, `AT Gls`, and `Int Apps` were all ambiguous, with best coverage of 18, 45, and 36 of 103 players. `AT Gls` included one depth-two path, but five paths tied at the same coverage. Compared with `academy-a`, the best coverages changed only from 16/45/35 to 18/45/36. Do not widen generic traversal. Commits 8 through 11 replace that approach with bounded structure-specific discovery and validation.
- On 2026-08-07, three synchronized player-search exports supplied 101, 120, and 23 rows from the same FM 26.3.2 state. Together they contain 244 rows for 235 distinct UIDs, with nine repeated UIDs and no conflicting numeric values. All target and anchor cells are present and numeric.
- The combined non-zero evidence is broad for six targets and usable but sparse for the seventh: 224 `AT Apps`, 190 `AT Gls`, 219 `AT League Apps`, 184 `AT League Goals`, 161 `Int Apps`, 103 `Int Gls`, and 55 `International Assists`. `AT Apps` is never below `AT League Apps`, and `AT Gls` is never below `AT League Goals`.
- The matching generic captures resolved every requested UID with no unread ranges. They requested and read 339,456, 427,648, and 73,216 bytes; no player exceeded 3,712 bytes. UID, CA, and PA recovered for every player, and the known Determination ×5 path recovered for 82/101, 104/120, and 18/23 players.
- All seven career targets remained ambiguous. Best generic coverage across the three cohorts was `AT Apps` 13/5/3, `AT Gls` 31/17/5, `AT League Apps` 12/5/3, `AT League Goals` 36/21/5, `Int Apps` 49/28/4, `Int Gls` 79/53/8, and `International Assists` 92/87/13. The high sparse-field figures track zero-valued rows rather than non-zero values.
- Cross-cohort inspection found no common top candidate with meaningful non-zero support: the best common paths matched at most one non-zero `AT Apps` row and zero non-zero rows for the other six targets. This is a strong negative result for the generic capture, not proof that FM lacks the data.
- Read-only inspection of the local generated interop metadata found UI binding reference types named `CareerStatsReference`, `CareerStatsFullReference`, and `PlayerHistoryReference`, but did not reveal an obvious typed raw career-record model in the game-plugin assembly. Machine-local interop DLLs remain research context only and must not become a checked-in or required build dependency.
- On 2026-08-07, the current FMSuperScout source was audited at commit `4ec3c657e3b993edf4e5b87d5ea42c4a3700cac6`. The original ValueScout work was a targeted MVP port, not a complete reader audit. Current upstream has no career, appearance, goal, assist, or season-stat extraction path, so it does not replace Commits 8 through 11.
- The upstream audit identified proven direct data not present in schema v5: player nation UID, gender, club reputation, and raw team type; manager and managed-club metadata; staff identity, CA/PA, contract, club, division, job ID, and 22 attributes; selectable men's, women's, or mixed scans; and global club-object discovery. PR 2 will read and persist these fields without adding UI.
- Upstream also declares player condition, morale, home reputation, squad number, staff reputation, and a second value slot without relying on them in its emitted product data. PR 2 must validate these dormant pins through the probe or omit them from schema v6. Upstream's duplicated asking-price output, null wage demand, estimates, scores, projections, and app-side history are not memory-reader parity targets.
- The audit confirmed that both readers use team next-fixture consensus as a date proxy. Upstream records that it can differ from the world date during breaks. Schema v6 will record an explicit date basis and stop labeling this value as direct memory truth.
- Upstream scan hardening now includes bounded worker-local buffers, adaptive worker count, unread-fraction measurement, and a live-first PSS VA-clone retry. PR 3 isolates these operational changes from PR 2's semantic expansion and may remove any layer that does not prove a practical benefit safely.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Capture bounded player memory by UID | 393f75f | Isolated C# research protocol, scan-gate integration, bounded capture, atomic output, and fake-memory coverage | Sol High accepted after one correction round; no Critical, High, or Medium findings remain | Root window set to 0x280 bytes during review to include planned known anchors; no scope change. |
| PR 1 | Correlate CSV truth with probe captures | ccd6102 | Dependency-free .NET CLI and repository command for bounded request capture, mapped numeric CSV correlation, and synchronized before/after comparison | Sol High accepted after two correction rounds; no Critical, High, or Medium findings remain | Explicit IDs are single-use while matching artifacts remain; candidates require varied multi-player evidence. |
| PR 1 | Adapt real FM exports for bounded research | 73dd764 | Raised the fixed cap to 128; added declared real-export normalization, field-local eligibility, provenance, and focused coverage | Sol High accepted after one correction round; no Critical, High, or Medium findings remain | Uniform cross-player before/after shifts no longer qualify as varied evidence; no scope change. |
| PR 1 | Document and validate the research workflow | f546485 | Added the bridge-local runbook, probe protocol summary, architecture boundary, and honest live-validation handoff | Sol Medium accepted; no Critical, High, Medium, or Nitpick findings | Live FM capture was deferred because FM was not running; the DLL install and automated checks passed. |
| PR 1 | Resolve repository-relative probe paths | 206b775 | Kept `memory-probe` execution at the repository root and made the documented work directory absolute | Review skipped by explicit developer request for the trivial correction | The first live command exposed the wrapper defect; protocol and capture semantics did not change. |
| PR 1 | Select stable first-hop pointer paths across the cohort | 010125e | Added cohort-level player- and person-root first-hop selection, schema-v2 capture-policy provenance, and legacy single-capture compatibility | Sol High accepted; no Critical, High, Medium, or Nitpick findings | No plan deviation. |
| PR 1 | Capture a bounded second pointer hop | b8cb4bf | Added deterministic cohort-ranked depth-two capture, compiled-in second-hop quotas and ceilings, complete provenance, and alias/cycle/missing-evidence coverage | Sol High accepted; no Critical, High, Medium, or Nitpick findings | No plan deviation. |
| PR 1 | Revise and repeat the live evidence workflow | 32ee188 | Updated the runbook, protocol summary, architecture, and same-cohort live evidence contract for schema v2 and bounded depth-two capture | Sol Medium accepted; no actionable findings | The completed live run produced a negative target result and triggered the planned structure-specific replan. |

## Final validation

**Feature review profile:** Sol Max — final review must connect bounded research, native object discovery, cross-language schema v6, migration and ingest completeness, staff/player identity rules, deterministic concurrency, optional native snapshot lifetime, and live operational evidence across three PRs.

Automated evidence:

- `./scripts/dev memory-probe --help`
- `./scripts/dev bridge-test`
- `./scripts/dev test`
- `./scripts/dev check`
- Fresh-context Sol Max feature review over the exact recorded implementation commits and final current-state documentation.

Manual Windows/FM evidence:

1. Build and install the current DLL with `./scripts/dev bridge-install`, restart FM26, and load the state represented by the three current player-search exports.
2. Use new request IDs to capture the unchanged 101-, 120-, and 23-player cohorts with the history preset. Do not advance or change the save between export and capture.
3. Confirm the matching request IDs, supported game/layout metadata, exact requested UID sets, successful research status, versioned structural policy, and the final compiled-in context, collection, record, player, and request ceilings.
4. Confirm the inventory groups exact UID, person-object, and player-block references by stable structural provenance without retaining unselected heap hits. Confirm the record recipe resolves complete club and international collections for every requested player.
5. Run correlation and confirm it recovers UID, CA, PA, and the applicable Determination ×5 anchor before interpreting career fields.
6. Correlate `AT Apps`, `AT Gls`, `AT League Apps`, `AT League Goals`, `Int Apps`, `Int Gls`, and `International Assists`. For each field, record direct or aggregate provenance, eligible and excluded players, total and non-zero coverage, conflicts, and ambiguity.
7. Combine evidence across 235 distinct UIDs. Confirm every eligible row and every eligible non-zero row reproduce through the same FM 26.3 recipe and encoding, and confirm the nine repeated UIDs agree without inflating the denominator.
8. For every derived value, inspect the selected records, category filter, scalar encoding, raw contributions, and arithmetic. Confirm `AT Apps >= AT League Apps` and `AT Gls >= AT League Goals` for the result as well as the CSV truth.
9. Confirm a malformed preset, unsupported layout, missing UID, invalid descriptor, excessive record count, and 129-player request fail only the research status and leave the prior `dump.json`, production `status.json`, and prior successful `probe.json` unchanged.
10. Keep all CSV, probe, analysis, metadata-inspection, and diagnostic artifacts under `.work/memory-probe/` or outside the repository. Record only concise pass/fail evidence and any plan-changing discovery here.
11. Install the PR 2 bridge and create an unlimited schema-v6 dump. Confirm player and staff counts, manager and managed-club metadata, database scope, date basis, full-club squad resolution, and every promised direct field against representative FM truth.
12. Validate and ingest schema v6. Confirm every promised field reaches its snapshot, player, or staff storage location; current-snapshot replacement cascades old staff; stale v5 fails with a clear rescan instruction; and existing Search, Player, Planner, and Academy results remain unchanged.
13. Record every dormant pin as accepted, rejected, or unresolved. Confirm only accepted pins appear in schema v6 and no duplicate asking-price, null placeholder, app estimate, or derived upstream history field is present.
14. Measure schema-v6 dump size, bridge write time, Rust validation time, transaction time, database growth, and peak scan-buffer budget on the representative save.
15. Install the PR 3 bridge and compare complete semantic results against PR 2. Confirm deterministic player, staff, manager, club, and representative field parity under the final scan implementation.
16. Confirm worker count and buffer memory remain within compiled-in limits, cancellation stops all workers, unread metrics are reported, and an incomplete live attempt cannot replace the prior good dump.
17. Exercise the frozen-memory retry safely when retained. Confirm it triggers at most once from measured read quality, releases every native handle, reports its source, and preserves the prior dump when retry fails. If safe live evidence cannot support it, remove the retry and document the retained read-quality behavior.

## Documentation impact

- `bridge/MEMORY_PROBE.md` owns the developer-only capture, correlation, replication, and evidence procedure.
- `bridge/DUMP_SCHEMA.md` will own schema v6, including expanded snapshot and player metadata, staff records, null rules, scope, date basis, and completeness checks.
- `bridge/README.md` documents the separate probe files and `memory-probe` command surface, then the expanded reader and final scan operating limits.
- `.wiki/ARCHITECTURE.md` records the implemented research path, schema-v6 bridge-to-SQLite flow, staff storage boundary, and final scanner lifecycle only as each PR makes them true.
- The completed FM26 memory-read record will be reconciled at feature completion so its v5 intentional gaps do not remain presented as current behavior.
- This ledger records the generic-capture negative result, synchronized career evidence, structure-specific replan, pinned upstream parity audit, dormant-pin outcomes, production expansion, and scan-hardening evidence. No new ADR is warranted unless implementation crosses ADR-0016's existing boundary; no debug report is warranted without a reusable confirmed failure pattern.
