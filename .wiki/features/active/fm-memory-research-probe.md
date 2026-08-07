# FM Memory Research Probe

## Status

Active

## Intent

Add a reusable, developer-only probe for finding FM26 player data in memory. The probe must capture small, labeled memory samples for explicit player UIDs and compare those samples with FM-exported CSV values. It must accept the real Youth Academy and Moneyball export shapes through bounded full-export requests. When scalar root and pointer-window correlation fails, it must support bounded discovery and analysis of player-linked record structures. It will support later offset research without changing production snapshot data until a location or derivation has been independently verified.

## User-visible behavior

- A developer can export a player view from FM and request a bounded memory capture for every explicit UID in that CSV while the same save state remains open.
- A developer can keep the full current sample exports unchanged. The fixed 128-player ceiling accommodates the largest current export without removing the hard safety bound.
- The bridge writes a versioned `probe.json` with game, layout, module, candidate-address, root-window, and bounded pointer-target metadata for each matched UID.
- A structure-aware capture preset can inventory bounded memory contexts that contain exact references to the requested players, then capture only a versioned record shape justified by that inventory.
- A repository command can correlate declared integer or decimal CSV values against one capture and can compare before-and-after CSV/capture pairs. It reports excluded cells and requires explicit normalization for compound or unit-bearing values.
- Analysis can compare direct structured values and declared record aggregates. It reports candidate relative paths, offsets, encodings, non-zero match coverage, and ambiguity. It never labels a candidate as a verified production offset.
- Research requests and outputs remain separate from Load Data. The app has no new UI, and schema-v5 `dump.json` remains unchanged.

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

## Non-goals

- Shipping or integrating offsets for all-senior appearances, all-senior goals, international caps, or any other new production field. The probe may report candidate paths from live evidence, but a later feature owns production verification and integration.
- Changing dump schema v5, Rust dump validation, snapshot ingest, SQLite migrations, Academy DTOs, statistics UI, or Moneyball behavior.
- Adding snapshot history or reconstructing statistics from earlier points in a season.
- Adding a product-facing probe UI or sending raw memory over Tauri IPC.
- Capturing the whole heap, following arbitrary addresses supplied by the operator, or performing recursive unbounded pointer scans.
- Adding a generic reverse-pointer graph, accepting operator-defined structure recipes in the live bridge, or retaining every reference hit found in the heap.
- Supporting non-numeric CSV correlation, save-file parsing, non-Windows FM editions, or unsupported FM builds.
- Treating every exported display field as a distinct stored scalar. Later integration may derive rates, percentages, and aggregates from verified base values when that preserves the required semantics.
- Requiring a controlled before/after sample. Diff remains available when a useful state transition can be captured, but cross-sectional replication is the normal evidence path.
- Treating one matching player, one matching value, or one changed byte as enough evidence for an offset.

## Current-state map

- Relevant components: `bridge/Plugin.cs` polls the production request file, owns one background scan thread, and serializes scans through `ScanGate`; `bridge/Scanning/CapADumpPipeline.cs` resolves layouts and coordinates candidate discovery and extraction.
- Data model: `PersonScanner` returns `PersonCandidate(ObjectAddress, Uid, Ca, Pa, ClassOffset)`. The player-block base is `ObjectAddress - ClassOffset`; the person object remains available at `ObjectAddress`.
- Memory boundary: `IMemoryReader` provides controlled scalar and block reads; `WindowsMemoryReader` uses `ReadProcessMemory`, and `Tests/Fakes/FakeMemoryReader` provides deterministic sparse memory.
- Layout boundary: `IFmMemoryLayout` and `Fm263Layout` own supported-build pins. Known anchors include UID, CA, PA, attributes, market value, and reputation.
- Output boundary: `DumpWriter` and `StatusWriter` demonstrate temporary-file plus atomic-replace JSON output. Production dump schema v5 is frozen and validated by Rust.
- Persistence and migrations: none for this feature. Probe artifacts are disposable research files and never enter SQLite.
- Existing behavioral assumptions: full candidate discovery takes about 26 seconds on the reference save and already retains the addresses needed to derive player and person bases. Live FM attach validation remains manual on Windows.
- Architectural seams: capture and raw-memory interpretation stay in the C# bridge; a developer CLI prepares bounded full-export requests and analyzes local CSV/JSON files; Rust, React, and snapshot ingest remain outside the feature.
- Project validation commands: `./scripts/dev bridge-test` runs fake-memory C# tests; `./scripts/dev check` remains the repository commit gate; `./scripts/dev bridge-install` installs a live-test DLL on Windows/WSL.
- Primary risks: unsafe capture breadth, interference with production scans, misleading zero-driven correlations, stale or mismatched CSV/captures, and incorrect grouping or aggregation of indirect history records.

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
```

The probe protocol has its own schema version and file names under the existing bridge data directory. It reuses the bridge's layout resolution, candidate scanner, safe memory reader, cancellation path, and scan gate, but it does not enter the production status/dump protocol.

The capture records two stable roots per matched candidate: player-block-relative and person-object-relative. It collects roots for the full requested cohort, ranks safe pointer source paths by cross-player availability, and reserves independent quotas for the two roots before it captures any target. It may repeat this planning step once across the selected first-hop ranges. Every target retains its root-relative source path, and fixed depth, count, window, address-region, and byte ceilings prevent graph expansion. The request may select UIDs and a bounded preset, but it may not supply arbitrary process addresses or remove the built-in limits.

The .NET developer tool under `bridge/Tools/MemoryProbe/` owns CSV parsing, declared value normalization, request creation and waiting, scalar and structured-record interpretation, cross-player correlation, and optional before/after comparison. It uses runtime libraries only. `scripts/dev` exposes the tool as `memory-probe` so the repository retains one command surface.

The generic depth-two capture remains available for ordinary scalar research. A separate compiled-in history preset first inventories exact UID and player-root references in the same safe memory regions. It groups bounded contexts by stable structural provenance instead of treating every hit as interchangeable. Live evidence from that inventory must justify one FM-versioned record recipe before the bridge follows a container or captures record rows. The analyzer may test direct fields and explicit aggregates from those rows, but it must keep raw values, filters, and aggregation rules visible.

One PR is appropriate. Capture without analysis is not yet a useful research workflow, while the analyzer depends directly on the probe schema. The final documentation commit closes the same developer capability and records the manual live validation contract.

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
- Keep production field selection, derived-stat formulas, schema changes, and persistence work in later feature plans after the probe identifies and independently validates the required source values.
- Preserve the generic depth-two preset as a completed scalar-research path. Add structure-aware work as a separate preset; do not increase its pointer depth or path quotas.
- Use the three synchronized player-search cohorts as the required cross-sectional evidence for all seven career targets. Acceptance requires complete eligible-row and non-zero-row coverage across 235 distinct players; the nine repeated UIDs are a consistency check, not additional independent players.
- Treat `International Assists` as an international-career target only. It does not replace the unavailable all-senior Academy assists value.
- Do not create an ADR. The feature stays within ADR-0016's established C# memory-reader boundary and does not change a product-facing or persisted contract.

### Unknowns

- Whether the player-linked structure contains direct totals, per-season or per-competition records, or both.
- Whether one stable reverse-reference signature reaches both club and international records or the two domains need separate FM-versioned recipes.
- The exact container layout, record stride, category fields, and aggregation rules. Commit 8 must obtain live structural evidence before Commit 9 pins them.
- Whether rounded decimal statistics are stored directly as floating-point values, fixed-point totals, or derived from other counters.
- Which FM CSV header names and delimiters appear across other locales; the tool must use explicit UID/field mappings and robust delimiter handling rather than one hard-coded English export.

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

With fake memory, accept one research request containing one UID, reuse candidate discovery to locate it, capture bounded player/person root ranges plus one first-hop target, and atomically write a schema-v1 `probe.json` without touching any production protocol file. This proves the safe in-process path before CSV analysis is added.

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

**Stop conditions:** Do not mark the feature ready for closeout unless every target is either supported by the full evidence contract or explicitly replanned as unresolved. Do not widen generic traversal. Stop for developer input if the source CSV state is no longer available or if a target's exported semantics remain uncertain after structured inspection.

**Review mandate:**

- Verify each of the seven target claims against all applicable non-zero rows in all three reports.
- Verify the documented recipe, limits, commands, schema versions, and report fields match implementation.
- Verify repeated UIDs agree but do not inflate the 235-player combined denominator.
- Verify direct and derived fields remain clearly distinguished and every aggregate states its exact inputs.
- Verify no absolute address or raw artifact enters Git and no weak candidate is presented as verified.
- Verify unresolved evidence leaves the feature Active with a concrete replan condition.

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

**Feature review profile:** Sol High — the final review must connect in-process memory safety, isolated file lifecycle, real-export normalization, deterministic bounded traversal and reference discovery, structure recipe validity, aggregate evidence quality, and live operational instructions across all recorded commits.

Automated evidence:

- `./scripts/dev memory-probe --help`
- `./scripts/dev bridge-test`
- `./scripts/dev check`
- Fresh-context Sol High feature review over the exact recorded implementation commits.

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

## Documentation impact

- `bridge/MEMORY_PROBE.md` owns the developer-only capture, correlation, replication, and evidence procedure.
- `bridge/README.md` documents the separate probe files and `memory-probe` command surface alongside the existing bridge protocol.
- `.wiki/ARCHITECTURE.md` records the implemented research path without changing the schema-v5 product dump boundary.
- This ledger records the generic-capture negative result, the three synchronized career cohorts, the structure-specific replan, and the remaining live FM evidence requirement. No ADR or debug report is warranted.
