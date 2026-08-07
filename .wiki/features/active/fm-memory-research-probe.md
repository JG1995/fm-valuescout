# FM Memory Research Probe

## Status

Active

## Intent

Add a reusable, developer-only probe for finding FM26 player data in memory. The probe must capture small, labeled memory samples for explicit player UIDs and compare those samples with FM-exported CSV values. It must accept the real Youth Academy and Moneyball export shapes through bounded full-export requests. It will support later offset research without changing production snapshot data until an offset has been independently verified.

## User-visible behavior

- A developer can export a player view from FM and request a bounded memory capture for every explicit UID in that CSV while the same save state remains open.
- A developer can keep the full current sample exports unchanged. The fixed 128-player ceiling accommodates the largest current export without removing the hard safety bound.
- The bridge writes a versioned `probe.json` with game, layout, module, candidate-address, root-window, and bounded pointer-target metadata for each matched UID.
- A repository command can correlate declared integer or decimal CSV values against one capture and can compare before-and-after CSV/capture pairs. It reports excluded cells and requires explicit normalization for compound or unit-bearing values.
- Analysis reports candidate relative paths, offsets, encodings, match coverage, and ambiguity. It never labels a candidate as a verified production offset.
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
- The comparison tool must reject or clearly report malformed CSV, duplicate or missing UIDs, incompatible capture metadata, unreadable ranges, unsupported numeric values, and unmatched rows.
- UID handling, missing-value handling, units, compound values, and decimal rounding must be explicit. The tool must not guess a field's meaning from its header or silently coerce display text into scalar truth.
- Correlation results are hypotheses. Production layouts, dump schemas, SQLite, Academy behavior, and Moneyball features change only in later features after independent live validation.
- Raw CSV exports, probe captures, and analysis output stay untracked under `.work/` or outside the repository. They must not enter Git.

## Non-goals

- Shipping or integrating offsets for all-senior appearances, all-senior goals, international caps, or any other new production field. The probe may report candidate paths from live evidence, but a later feature owns production verification and integration.
- Changing dump schema v5, Rust dump validation, snapshot ingest, SQLite migrations, Academy DTOs, statistics UI, or Moneyball behavior.
- Adding snapshot history or reconstructing statistics from earlier points in a season.
- Adding a product-facing probe UI or sending raw memory over Tauri IPC.
- Capturing the whole heap, following arbitrary addresses supplied by the operator, or performing recursive unbounded pointer scans.
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
- Primary risks: unsafe capture breadth, interference with production scans, misleading false correlations, stale or mismatched CSV/captures, and insufficient pointer reach for indirect history structures.

## Feature architecture

```text
FM player-view CSV
  -> ./scripts/dev memory-probe capture
  -> probe-request.json with explicit UIDs
  -> BepInEx bridge: supported layout + existing candidate discovery
  -> fixed player/person windows + budgeted pointer targets
  -> probe-status.json + atomic probe.json
  -> ./scripts/dev memory-probe correlate | diff
  -> ranked candidate paths and ambiguity report
```

The probe protocol has its own schema version and file names under the existing bridge data directory. It reuses the bridge's layout resolution, candidate scanner, safe memory reader, cancellation path, and scan gate, but it does not enter the production status/dump protocol.

The capture records two stable roots per matched candidate: player-block-relative and person-object-relative. It collects roots for the full requested cohort, ranks safe pointer source paths by cross-player availability, and reserves independent quotas for the two roots before it captures any target. It may repeat this planning step once across the selected first-hop ranges. Every target retains its root-relative source path, and fixed depth, count, window, address-region, and byte ceilings prevent graph expansion. The request may select UIDs and a bounded preset, but it may not supply arbitrary process addresses or remove the built-in limits.

The .NET developer tool under `bridge/Tools/MemoryProbe/` owns CSV parsing, declared value normalization, request creation and waiting, scalar interpretation, cross-player correlation, and optional before/after comparison. It uses runtime libraries only. `scripts/dev` exposes the tool as `memory-probe` so the repository retains one command surface.

One PR is appropriate. Capture without analysis is not yet a useful research workflow, while the analyzer depends directly on the probe schema. The final documentation commit closes the same developer capability and records the manual live validation contract.

## Uncertainty register

### Known

- Candidate discovery already yields UID, object address, class offset, CA, and PA for each accepted player.
- Known FM 26.3 layout pins provide enough anchors to test whether correlation finds correct relative locations and transforms.
- Academy currently represents senior appearances, goals, assists, and international caps as unavailable nullable values. The later integration will omit assists because FM does not export the required career value.
- Production Load Data depends on frozen dump schema v5 and the existing `request.json` / `status.json` files.
- All five sample exports are valid UTF-8, semicolon-delimited CSV files with unique UIDs and no malformed rows. The paired Youth Academy and Moneyball exports contain the same UID sets within each save state.
- `AT Apps`, `AT Gls`, and `Int Apps` mean totals across all senior matches, including league, cup, and continental matches. These are the three Academy research targets.
- Moneyball values are season-to-date statistics at the moment of capture. FM does not expose earlier values through the export, so each load is an honest point-in-time snapshot rather than a complete season history.
- Compound `Appearances` values represent starts and substitute appearances. Later persistence will store those counts separately and derive total appearances.
- The revised Youth Academy export provides strong variation across 103 players. The Moneyball exports mix integers, rounded decimals, missing values, unit-bearing values, localized currency text, and compound appearances.

### Assumptions

- Fixed player/person root windows plus a small, budgeted pointer traversal will expose enough nearby structure to research numeric player fields.
- FM player-view exports include a stable numeric UID column and can include varied known and target numeric columns.
- The existing full candidate walk is fast enough for an occasional developer probe; targeted scanner optimization is not required for the first version.

### Decisions

- Use a separate developer-only probe protocol instead of extending the product dump or Rust protocol.
- Capture raw bounded ranges and pointer provenance instead of adding speculative field offsets to `IFmMemoryLayout`.
- Provide single-capture correlation and before/after diff in one .NET CLI with no new package dependency.
- Keep research artifacts disposable and untracked; preserve only validated conclusions in later feature ledgers or layout code.
- Treat the sample exports as temporary evidence and record only their durable field-shape and coverage conclusions in this ledger and the runbook.
- Raise the fixed request ceiling from 16 to 128 players. The initial capture used a 1,408-byte per-player bound and a 180,224-byte full-request bound. Commits 5 and 6 may replace these limits only with new fixed ceilings that remain below 512 KiB of raw memory for a 128-player request.
- Do not require a controlled before/after sample or disjoint UID sets. Candidate discovery may use one synchronized cohort with varied values. Before later production integration, require fresh synchronized evidence that recovers the known anchors and repeats the same path and encoding with meaningful value variation. A different player set is preferred when FM can provide one, but it is not mandatory. Keep diff as optional supporting evidence.
- Select pointer paths across the requested cohort instead of giving each player the first valid pointers found in player-root order. Rank paths by cross-player availability with deterministic tie-breaking, reserve separate first-hop quotas for player and person roots, and record the fixed capture policy in the probe output.
- Permit one fixed second hop after cohort-level path selection. Allow at most eight player-root first-hop targets, eight person-root first-hop targets, and eight second-hop targets per player. Keep every target window at 128 bytes. Together with the existing roots, the revised maximum is 3,968 bytes per player and 507,904 raw bytes for 128 players.
- Do not research Academy assists. A later Academy feature will remove that unsupported outcome.
- Preserve Moneyball statistics as season-to-date point-in-time values. The existing snapshot-history backlog item may later retain earlier captures inside the app, but this probe does not reconstruct history that FM cannot export.
- Normalize compound appearances into separate starts and substitute-appearance values. A later integration derives total appearances instead of storing the formatted display string.
- Keep production field selection, derived-stat formulas, schema changes, and persistence work in later feature plans after the probe identifies and independently validates the required source values.
- Do not create an ADR. The feature stays within ADR-0016's established C# memory-reader boundary and does not change a product-facing or persisted contract.

### Unknowns

- Whether target career statistics are inline scalar totals, counters inside another object, or derived from history collections.
- Whether the planned depth-two, 3,968-byte capture exposes the Academy career-statistics structure.
- Whether rounded decimal statistics are stored directly as floating-point values, fixed-point totals, or derived from other counters.
- Which FM CSV header names and delimiters appear across other locales; the tool must use explicit UID/field mappings and robust delimiter handling rather than one hard-coded English export.

### Risks

- Common small integers can produce many false matches. The analyzer must rank by multi-player coverage and show ambiguity rather than choose the first hit.
- A CSV export and probe taken from different save moments can create false negatives or false deltas. Request and result metadata plus the runbook must make synchronization explicit.
- Pointer-like values in raw bytes can fan out quickly. Hard depth, count, byte, and UID budgets must remain non-configurable ceilings.
- Adding probe polling to `Plugin.cs` can disturb shutdown or production request handling. Tests and review must cover priority, mutual exclusion, cancellation, and file isolation.
- Stats may require a structure-specific reader. If bounded generic capture cannot expose a known indirection in live validation, replan the probe instead of expanding into unbounded traversal.
- Sparse event columns can satisfy simple value variation while still producing weak evidence. Require fresh synchronized evidence and inspect related neighboring fields before accepting a binary or rare-event candidate.
- FM display values can be rounded or derived. A direct byte match may be absent even when the underlying source values are inside the capture, so the analyzer must keep exact, rounded, scaled, and derived evidence distinct.

## Sample export assessment

The five untracked samples under `.work/` are suitable as the field inventory and as source material for full-export correlation. The revised 103-player Academy export has one synchronized `academy-a` capture. Each other sample still needs a synchronized capture from the same loaded save state before its values can be correlated.

- The original Youth Academy and Moneyball files contain the same 75 UIDs from one club. They remain useful as one paired data set, but the Moneyball file has 41 players with zero minutes and weak rare-event coverage.
- The revised Youth Academy and Moneyball files contain the same 103 UIDs and no overlap with the original pair. They can provide another player set only if the matching save state is available; the evidence contract does not require that operationally unavailable state.
- The revised Youth Academy file has strong target evidence: `AT Apps` has 73 distinct values and 91 non-zero rows, `AT Gls` has 26 distinct values and 59 non-zero rows, and `Int Apps` has 33 distinct values and 69 non-zero rows.
- The player-search Moneyball file contains 72 different UIDs across 51 clubs and 18 divisions. It has 69 players with minutes, 69 ratings, four non-zero penalty-save rows, and broad variation across the other match statistics.
- Across the revised and player-search Moneyball files, only `Penalties Saved`, `Red cards`, and `Mistakes Leading to Goals` remain notably sparse. Fresh synchronized agreement and neighboring-field structure are mandatory for these fields.
- Academy assists are intentionally absent because FM cannot export the required all-senior-career value. They are no longer a missing research input.
- The current analyzer cannot consume several Moneyball shapes: compound appearances such as `32 (5)`, rounded decimals such as xG and rating, `-` for unavailable ratings, and unit-bearing values such as `205.1km`. Currency ranges, localized wages, dates, and text enums are not direct scalar truth and remain outside automatic correlation.
- Known values in the exports remain useful anchors, but display values are not always identical to the existing product contract. For example, a localized transfer-value range is not the same field as the bridge's scalar `marketValueGbp`.

## Walking skeleton

With fake memory, accept one research request containing one UID, reuse candidate discovery to locate it, capture bounded player/person root ranges plus one first-hop target, and atomically write a schema-v1 `probe.json` without touching any production protocol file. This proves the safe in-process path before CSV analysis is added.

## Delivery plan

### PR 1 — Add reusable FM memory research tooling

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(memory-probe): add reusable FM memory research tooling`

**Build-feature-loop profile:** Terra xhigh — this is the highest implementation profile among the PR's non-removed commits.

**Purpose:** Deliver one complete developer workflow for bounded full-export capture, CSV correlation, optional before/after comparison, and repeatable cross-sectional validation. The capture and analyzer share one research schema and therefore belong in one review and merge boundary.

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

**Status:** Active

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

**Status:** Pending

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

**Status:** Pending

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

## Active work

**PR:** PR 1 — Add reusable FM memory research tooling

**Commit:** Commit 5 — Select stable first-hop pointer paths across the cohort

### RED proof

The synchronized `academy-a` capture resolved all 103 requested players and recovered the known roots without unread ranges, but the current first-four traversal chose targets only from the player root and stopped at depth one. Add a fake-memory cohort in which the shared useful path and all person-root paths lose to earlier per-player pointers; confirm that the current capture misses them.

### Expected outcome

A developer can recapture the same 103-player Academy cohort with deterministic player-root and person-root coverage plus one bounded second hop. Correlation can then either expose a strong reproducible target path or provide a clear negative result that justifies a structure-specific reader. The evidence procedure accepts fresh synchronized validation from the same cohort when a disjoint player set is not available.

### Explicit exclusions

- No speculative Academy or Moneyball offset is recorded.
- No raw CSV, memory, screenshot, or machine-specific artifact is committed.
- No product-facing UI, production schema change, or unbounded capture behavior is added.

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

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Capture bounded player memory by UID | 393f75f | Isolated C# research protocol, scan-gate integration, bounded capture, atomic output, and fake-memory coverage | Sol High accepted after one correction round; no Critical, High, or Medium findings remain | Root window set to 0x280 bytes during review to include planned known anchors; no scope change. |
| PR 1 | Correlate CSV truth with probe captures | ccd6102 | Dependency-free .NET CLI and repository command for bounded request capture, mapped numeric CSV correlation, and synchronized before/after comparison | Sol High accepted after two correction rounds; no Critical, High, or Medium findings remain | Explicit IDs are single-use while matching artifacts remain; candidates require varied multi-player evidence. |
| PR 1 | Adapt real FM exports for bounded research | 73dd764 | Raised the fixed cap to 128; added declared real-export normalization, field-local eligibility, provenance, and focused coverage | Sol High accepted after one correction round; no Critical, High, or Medium findings remain | Uniform cross-player before/after shifts no longer qualify as varied evidence; no scope change. |
| PR 1 | Document and validate the research workflow | f546485 | Added the bridge-local runbook, probe protocol summary, architecture boundary, and honest live-validation handoff | Sol Medium accepted; no Critical, High, Medium, or Nitpick findings | Live FM capture was deferred because FM was not running; the DLL install and automated checks passed. |
| PR 1 | Resolve repository-relative probe paths | 206b775 | Kept `memory-probe` execution at the repository root and made the documented work directory absolute | Review skipped by explicit developer request for the trivial correction | The first live command exposed the wrapper defect; protocol and capture semantics did not change. |

## Final validation

**Feature review profile:** Sol High — the final review must connect in-process memory safety, isolated file lifecycle, real-export normalization, deterministic bounded traversal, CLI evidence quality, and live operational instructions across all recorded commits.

Automated evidence:

- `./scripts/dev memory-probe --help`
- `./scripts/dev bridge-test`
- `./scripts/dev check`
- Fresh-context Sol High feature review over the exact recorded implementation commits.

Manual Windows/FM evidence:

1. Build and install the current DLL with `./scripts/dev bridge-install`, restart FM26, and load a supported save.
2. Use the fixed field contracts. Moneyball values are season-to-date at capture time, with starts and substitute appearances treated separately. Academy uses `AT Apps`, `AT Gls`, and `Int Apps` across all senior matches; assists are not a target.
3. Load the state represented by the revised Academy CSV and run a new capture without advancing or changing the save. Reuse the full 103-player UID set and use a new request ID.
4. Confirm the matching request ID, supported game/layout metadata, exact requested UID set, versioned capture-policy metadata, the 3,968-byte per-player ceiling, the 507,904-byte full-request ceiling, and successful research status.
5. Run correlation and confirm it reports the known UID, CA, PA, and Determination ×5 paths with the correct interpretations and transparent ambiguity. Use a scalar market-value anchor only when the synchronized export supplies that exact value; treat the localized transfer-value range only as display context.
6. Run the supported integer and decimal target fields. Record eligible players, exclusions, value diversity, candidate coverage, and whether the result is exact, rounded, scaled, ambiguous, or has no evidence.
7. Treat the revised capture as candidate discovery because the traversal policy changed. Before production integration, obtain fresh synchronized evidence that recovers the known anchors and repeats the same relative path and encoding with meaningful value variation and no equally strong conflict. The same UID cohort is acceptable; use a disjoint cohort only when FM can provide it naturally. Use diff only as optional supporting evidence.
8. For sparse Moneyball fields, inspect whether related statistics form a coherent neighboring structure. Do not accept a rare-event candidate from zero-heavy value agreement alone.
9. Confirm a malformed request, unsupported layout, missing UID, and 129-player request fail only the research status and leave the prior `dump.json`, production `status.json`, and prior successful `probe.json` unchanged.
10. Keep all CSV, probe, analysis, and diagnostic artifacts under `.work/memory-probe/` or outside the repository. Record only concise pass/fail evidence and any plan-changing discovery here.

## Documentation impact

- `bridge/MEMORY_PROBE.md` owns the developer-only capture, correlation, replication, and evidence procedure.
- `bridge/README.md` documents the separate probe files and `memory-probe` command surface alongside the existing bridge protocol.
- `.wiki/ARCHITECTURE.md` records the implemented research path without changing the schema-v5 product dump boundary.
- This ledger records the `academy-a` negative result, the bounded traversal replan, and the remaining live FM evidence requirement. No ADR or debug report is warranted.
