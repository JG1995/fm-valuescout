# Bridge Scan Performance

## Status

Active

## Intent

Make **Load Data** fast enough to ingest a complete FM26 player database. Replace the bridge's syscall-heavy scalar heap walk with safe block scanning, enable unlimited production loads after live validation, then expose a UI toggle and optional numeric cap for faster diagnostic loads.

## User-visible behavior

- A 500-player diagnostic load completes in less than 10 seconds on the current reference machine.
- A complete reference save of about 184,000 players completes the bridge dump in less than 60 seconds and finishes Load Data in less than 90 seconds.
- Production Load Data defaults to an unlimited scan (`scanTruncated: false`, `maxAccepted: null`).
- The UI can turn a player cap on or off. When the cap is on, the limit is a configurable positive integer (default 500 when enabling). When the cap is off, the scan is unlimited.
- Failed scans and failed ingests preserve the prior good snapshot.
- Unsupported FM builds still fail closed.

The live-save budgets were confirmed on the reference machine: capped path &lt;10s (PR 1), unlimited bridge dump ~26s for ~181k players (PR 2 Commit 4). A synthetic 500,000-player downstream run must complete without out-of-memory failure; it is a scale check, not a claim that a comparable live save exists.

## Invariants

- All FM memory access remains read-only and uses safe `ReadProcessMemory` failure handling.
- Scanning and dump I/O stay off the Unity main thread.
- UID deduplication, dynamic class-offset checks, CA/PA bounds, identity validation, and versioned layout pins remain authoritative.
- The bridge replaces `dump.json` only after a non-empty successful scan.
- Dump schema v5 and the C# file-protocol → Rust validation → transactional SQLite boundary remain stable unless measurements prove a contract change is necessary.
- A full scan sets `scanTruncated: false` and `maxAccepted: null`.
- Request `maxAccepted: null` means unlimited; a positive integer means stop after that many accepted players.

## Non-goals

- Unsafe pointer dereferences inside FM.
- A new global player-container pointer or patch-specific object-root traversal without separate reverse-engineering evidence.
- Save-file parsing, background synchronization, or incremental snapshots.
- Broad UI redesign or role scoring. PR 2 Commit 5 adds only Load Data cap controls.
- Parallel scanning by default. Add bounded workers only if the single-thread block scanner misses the live budget.
- Progress UI unless the measured full path cannot provide acceptable feedback with the existing loading state.

## Current-state map

- Relevant components: `bridge/Scanning/PersonScanner.cs`, `CapADumpPipeline.cs`, `bridge/Memory/WindowsMemoryReader.cs`, extraction readers, `DumpWriter.cs`, and Rust snapshot ingest.
- Data model: dump schema v5 stores rich player objects; production Load Data requests `maxAccepted: null` (unlimited); `PersonScanner.DefaultMaxAccepted` (500) remains the diagnostic/test constant.
- Persistence and migrations: Rust validates the full JSON, parses it again, and inserts each player inside one SQLite transaction.
- Existing behavioral assumptions: the app waits up to 120 seconds for terminal bridge status; prior live evidence recorded about 4.1 GiB scanned, 7.5 million vtable hits, 184,000 accepted players, and 3 minutes 47 seconds.
- Architectural seams: `IMemoryReader` isolates Win32 reads from fake-memory tests; `CapADumpPipeline` separates discovery, extraction, club resolution, and output; scan and ingest are split in Rust.
- Tests and validation: C# fake-reader tests cover scan correctness and truncation; Rust tests cover validation, rollback, and ingest. No phase timers or representative large-data benchmark exists.
- Primary risks: partial block reads, hidden correctness drift from aggressive filtering, full-dump serialization memory, repeated SQLite statement preparation, and stale bundled plugin DLLs during manual tests.

## Feature architecture (this feature)

Keep the existing C# bridge, disk dump, Rust validation, and SQLite architecture.

The scanner will read each candidate region into reusable bounded buffers and inspect aligned words in local memory. It will read vtable, UID, and other nearby fields from the block when possible. Module metadata or resolved vtable class offsets will be cached for the scan so repeated candidates do not trigger repeated kernel calls.

Player extraction will batch contiguous field ranges and bounded strings instead of issuing one process-memory call per byte. Dump output will stream compact schema-v5 JSON so full databases do not require a second complete serialized copy in memory. Rust ingest will receive a representative generated-data benchmark; it will change only where that benchmark identifies material overhead.

The first implementation remains single-threaded. Public FMSuperScout evidence shows the same safe approach moving from scalar reads to 32 MiB block reads and module-image caching, with a later bounded parallel commit reporting a 17-second to 8-second heap-scan reduction for about 49,000 players. Its source is useful evidence, not a benchmark for this machine. ADR-0016 records the author's permission and the requirement to reimplement independently:

- [Block scanner and cached module reads](https://github.com/mavarobli/FMSuperScout/blob/7370cfbfee0a33d9c0d54b525492285fdd8a62c7/plugin/MemScan.cs)
- [Bounded parallel scan commit](https://github.com/mavarobli/FMSuperScout/commit/c60451559d36b3895da74558f755ecae4797dfd1)

## Uncertainty register

### Known

- The current scanner performs one allocating `ReadProcessMemory` call per aligned 8-byte heap word.
- A 4.1 GiB walk therefore makes about 500 million base reads before vtable metadata and player extraction reads.
- Each vtable hit currently adds two memory reads to resolve its dynamic class offset.
- Player extraction performs dozens of scalar reads plus byte-by-byte string reads per accepted player.
- FM Genie Scout publishes no current source, scanner design, or reproducible benchmark.

### Assumptions

- Candidate regions returned by `VirtualQuery` can be read in bounded blocks on the reference FM build.
- Block scanning removes enough syscall and allocation overhead to meet the single-thread candidate-discovery budget.
- A 500,000-player generated dump is sufficient to expose downstream serialization and ingest memory problems.

### Decisions

- Preserve safe `ReadProcessMemory`; do not trade game stability for raw pointer speed.
- Optimize syscall count before changing region-selection heuristics.
- Keep the hardcoded 500-player production default until PR 2 Commit 4 wires request-scoped `maxAccepted` and a full live reference run passes.
- After that live proof, production Load Data defaults to unlimited; a UI cap remains available for diagnostic loads (PR 2 Commit 5).
- Start single-threaded. Consider bounded worker-local buffers only after measured evidence.
- Do not add benchmark dependencies. Use diagnostics, deterministic read-count assertions, generated data, and manual Windows timing.

### Unknowns

- Current phase split between candidate discovery, extraction, club indexing, JSON writing, validation, and SQLite ingest.
- Best block size for the reference machine.
- Whether single-thread block scanning meets the complete-dump budget.
- Whether compact streaming output and prepared SQLite inserts are sufficient for a generated 500,000-player dump.

### Risks

- One inaccessible page can fail a large `ReadProcessMemory` call. Failed blocks must be subdivided or skipped with diagnostics.
- Narrower region filters can silently omit valid players. Do not add them without before/after accepted-set evidence.
- Parallel readers can increase FM CPU and memory pressure while the game is running.
- A stale bundled DLL can invalidate manual measurements. Build and install the current bridge before every live benchmark.

## Walking skeleton

Add phase timings, implement one reusable single-thread block-read path, and prove on the current 500-player run that it returns the same schema-valid player set while reducing candidate-discovery time and process-memory call count. Keep the cap and all downstream behavior unchanged in PR 1.

## Delivery plan

### PR 1 — Replace scalar heap reads with block scanning

**Status:** Ready to merge — capped live path validated (`totalMs=6700`)

**Provisional PR title:** `perf(bridge): replace scalar heap reads with block scanning`

**Purpose:** Remove the confirmed heap-scan bottleneck behind minute-long capped loads while preserving current output and the 500-player safety cap.

**Depends on:** FM26 memory read and snapshot ingest. Manual measurements require a freshly built and installed bridge DLL.

#### Commit 1 — Add scan phase performance diagnostics

**Status:** Completed — `b2c8663`

**Work:** Record elapsed time for region enumeration, candidate discovery, player extraction, club indexing, dump writing, and total bridge work. Record process-memory call and requested-byte counts without changing dump schema v5.

**Out of scope for this commit:**
- Scanner algorithm changes.
- Cap, timeout, JSON, SQLite, or UI changes.
- New profiling or benchmark dependencies.

**Validation:** Add focused C# assertions that diagnostics contain every named phase with non-negative values, including region enumeration, plus process-memory call and requested-byte counts. Run `dotnet test`, `dotnet build`, and one fresh 500-player Windows/FM baseline.

**Provisional commit:** `feat(bridge): add scan phase performance diagnostics`

#### Commit 2 — Add reusable block memory reads

**Status:** Completed — `6f299da`

**Work:** Add a direct caller-owned byte-array block-read path that avoids the current intermediate allocation and copy. Add bounded subdivision for failed blocks while preserving short-read and invalid-address behavior in the Windows implementation and fake reader.

**Out of scope for this commit:**
- Replacing the scanner loop.
- Parallel reads.
- Extraction batching.

**Validation:** RED tests cover full, partial, failed, and region-edge block reads. Run bridge tests and build.

**Provisional commit:** `perf(bridge): add reusable block memory reads`

#### Commit 3 — Scan heap regions from reusable blocks

**Status:** Completed — `648b81f`

**Work:** Replace per-word process-memory calls with bounded region blocks, local aligned-word inspection, in-buffer UID reads, and cached module metadata or vtable-to-class-offset results. Stop immediately when a diagnostic cap is reached and preserve truncation semantics.

**Out of scope for this commit:**
- Player field extraction batching.
- Parallel region workers.
- Full production scans.

**Validation:** Characterization tests prove the same accepted UIDs, CA/PA values, deduplication, cancellation, and fail-safe behavior. A counting reader proves read calls scale with blocks, not 8-byte slots. Fresh Windows/FM validation must put the capped 500-player path below 10 seconds. The uncapped reference-save budget belongs to PR 2.

**Provisional commit:** `perf(bridge): scan heap regions from reusable blocks`

**Merge to trunk when:** C# tests/build pass; the capped live path meets budget; diagnostics prove the candidate scan no longer performs scalar heap reads; schema and prior-dump safety remain unchanged.

### PR 2 — Enable complete player snapshots

**Status:** Ready to merge — delivery plan complete; optional manual capped UI check remaining

**Provisional PR title:** `perf(load-data): enable complete player snapshots`

**Purpose:** Scale the post-discovery path, validate a complete live dump, enable unlimited production loads via request-scoped `maxAccepted`, then expose toggleable and configurable Load Data cap controls.

**Depends on:** PR 1 merged to `main`.

#### Commit 1 — Batch contiguous player field reads

**Status:** Completed — `8316c43`

**Work:** Read contiguous attribute, position, personality, and bounded string ranges into reusable buffers, then decode locally. Keep pointer-chain reads safe and nullable.

**Out of scope for this commit:**
- Layout changes or new fields.
- Parallel extraction.
- JSON and SQLite changes.

**Validation:** Existing extraction tests stay green; new counting-reader tests prove field reads are batched without changing decoded players. Compare extraction phase timing on the reference save.

**Provisional commit:** `perf(bridge): batch contiguous player field reads`

#### Commit 2 — Stream compact full dump output

**Status:** Completed — `a686189`

**Work:** Write compact schema-v5 JSON incrementally to the existing temporary file and retain atomic replace-on-success behavior.

**Out of scope for this commit:**
- Schema or protocol version changes.
- Chunked Rust ingestion.
- Snapshot history.

**Validation:** Existing schema/round-trip tests remain green; generated 184,000- and 500,000-player documents complete without a second full serialized copy or out-of-memory failure.

**Provisional commit:** `perf(bridge): stream compact full dump output`

#### Commit 3 — Reduce measured large-dump ingest overhead

**Status:** Completed — `862486a`

**Work:** Add a generated large-dump measurement around validation and transactional ingest. Reuse prepared SQLite work and remove duplicate parse work only where the measurement shows material cost.

**Out of scope for this commit:**
- Database schema changes.
- Snapshot history.
- Role scoring.

**Validation:** Existing rollback and replacement tests stay green. Generated 184,000- and 500,000-player runs record validation, insert, and total ingest timing and complete without unbounded memory growth.

**Provisional commit:** `perf(snapshot): reduce large dump ingest overhead`

#### Commit 4 — Request-scoped scan limit and unlimited production default

**Status:** Completed — `d8b7b04`

**Work:** Add optional `maxAccepted` to `BridgeRequest` / `request.json`. Pass it from Rust `load_data` into the bridge. Treat request `null` as unlimited in `CapADumpPipeline` (stop collapsing omitted/null into `DefaultMaxAccepted`). Default production Load Data to unlimited (`null`) so the reference full save can run. Align the Rust wait timeout with the measured envelope if needed, and update operational documentation with reference-save results. Keep explicit caps in tests and characterization paths.

**Out of scope for this commit:**
- Load Data UI controls.
- Background refresh or incremental scans.
- New progress UI unless the measured full path cannot provide acceptable feedback with the existing loading state.

**Validation:** Request plumbing tests cover positive caps and unlimited (`null`). A fresh full Windows/FM run loads the complete reference save, reports `scanTruncated: false` and `maxAccepted: null`, preserves the prior snapshot on forced failure, completes the bridge phase below 60 seconds and end-to-end Load Data below 90 seconds, then passes `./scripts/dev check` and `./scripts/dev test`.

**Provisional commit:** `feat(load-data): request-scoped scan limit with unlimited default`

#### Commit 5 — UI toggle and configurable player cap

**Status:** Completed — `b67fc3b`

**Work:** Add Load Data controls: a toggle for the player cap and a numeric field used when the cap is on. Toggle off sends `maxAccepted: null` (unlimited). Toggle on sends a positive integer (default 500 when enabling). Wire through the Commit 4 IPC and request path. Persist the preference lightly in the UI store if that keeps diagnostic loads convenient. Default the toggle to off after Commit 4's live unlimited proof.

**Out of scope for this commit:**
- Progress UI beyond the existing loading state.
- Background refresh or incremental scans.
- Changing dump schema v5.

**Validation:** Frontend and IPC tests cover capped and unlimited Load Data requests. Manual check: capped load truncates with the chosen limit; uncapped load reports `scanTruncated: false`. Pass `./scripts/dev check` and `./scripts/dev test`.

**Provisional commit:** `feat(ui): toggleable configurable Load Data player cap`

**Merge to trunk when:** Complete live data passes the reference budget and correctness spot checks; generated 500,000-player output/ingest completes without out-of-memory failure; timeout and docs match measured behavior; Load Data can run unlimited by default and optionally capped from the UI.

## Active work

**PR:** PR 2 — Enable complete player snapshots

**Commit:** _None_ — delivery plan complete

### Next step

Run `/finish-feature` after optional manual UI check (capped Load Data truncates at the chosen limit; uncapped reports `scanTruncated: false`). Then open/merge the PR when ready.

### Explicit exclusions (feature)

- Progress UI beyond the existing loading state.
- Background refresh or incremental scans.
- Changing dump schema v5.

## Discoveries and replanning

- Initial analysis found a stronger cause than the backlog's tentative region-filter direction: the scanner makes an allocating process-memory call for every aligned 8-byte heap word and repeats metadata reads for millions of vtable hits. Optimize read granularity before narrowing regions.
- Public FMSuperScout history provides directly comparable evidence for safe block scanning and cached module metadata. Genie Scout does not publish actionable implementation details.
- If the single-thread block scanner misses the candidate-discovery budget, add one pending PR 1 commit for bounded worker-local scanning after reporting the measured CPU and memory trade-off. Do not add it preemptively.
- If large-dump validation and ingest already meet budget, remove PR 2 Commit 3 rather than refactor Rust without evidence.
- Commit 1: `clubIndexingMs` stops after squad/club indexing only; game-date resolution and DumpPlayer assembly remain in residual `totalMs` so live baselines do not misattribute serialization cost to club work.
- **Pre-block-scan live baseline** (2026-07-30, FM 26.3.2, `maxAccepted=500`, `stoppedEarly=true`, build with Commit 1 diagnostics). Full paste: `.cursor/work/baselines/2026-07-30-capped-500-pre-blockscan-diagnostics.txt`.
  - `regionEnumerationMs=45`
  - `candidateDiscoveryMs=72958` (~73s) — dominates `totalMs=73171`
  - `extractionMs=48`, `clubIndexingMs=89`, `dumpWritingMs=24`
  - `processMemoryCalls=171533882`, `processMemoryRequestedBytes=1355448581` (~1.26 GiB requested)
  - `bytesScanned=1302961856` (~1.21 GiB), `vtableHits=4344871`, `regionCount=2166`
  - Confirms the plan thesis: candidate discovery + scalar heap reads are the bottleneck; post-discovery phases are already sub-second at the 500-player cap. PR 1 success criterion remains capped path `<10s` after block scanning.
- **Post-block-scan live validation** (2026-07-30, FM 26.3.2, `maxAccepted=500`, `stoppedEarly=true`, build with Commit 3 block scanner). Full paste: `.cursor/work/baselines/2026-07-30-capped-500-post-blockscan-diagnostics.txt`.
  - `regionEnumerationMs=40`
  - `candidateDiscoveryMs=6480` (~6.5s) — still dominates `totalMs=6700`
  - `extractionMs=47`, `clubIndexingMs=94`, `dumpWritingMs=22`
  - `processMemoryCalls=1238027` (~139× fewer than pre-scan), `processMemoryRequestedBytes=1546632333` (~1.44 GiB)
  - `bytesScanned=1538692064` (~1.43 GiB), `vtableHits=5297686`, `regionCount=2108`
  - Meets PR 1 capped budget (`totalMs=6700` and discovery well under 10s). No parallel workers needed for the capped path. Sample players decode sanely; extraction/club/dump remain sub-second.
- Commit 2: failed-block subdivision splits must align to `MinBlockReadSize` (page size). When `length/2` rounds below one page, split at `MinBlockReadSize` instead of an unaligned midpoint so mid-gap starts cannot miss a later accessible page.
- Commit 2: `TryReadBlock` `bytesRead` is a success count, not a contiguous prefix after hole recovery. Commit 3 must scan the full requested length (cleared gaps stay zero), not `buffer[0..bytesRead)`.
- Commit 3: heap discovery uses `MemoryConstants.DefaultScanBlockSize` (32 MiB) with 16-byte overlap; UID is read from the block buffer; vtable→class-offset results are cached for the scan (including negative 0). CA/PA remain scalar until PR 2 extraction batching.
- Commit 3: `FakeMemoryReader.TryReadBlock` composes sparse `AddBytes` segments with first-fill-wins so overlapping fixture blobs do not erase earlier person headers (matches scalar `TryRead` first-match).
- **PR 2 replan (2026-07-30):** Replace hard-delete of the production cap with request-scoped `maxAccepted` (Commit 4) plus UI toggle/configurable limit (Commit 5). Unlimited becomes the production default after live full-save validation; capped loads remain available for diagnostics. Progress UI stays a non-goal unless the measured full path needs it.
- **PR 2 Commit 1:** Attribute visible+hidden share one contiguous `TryReadBlock` from `AttrsOffset`; personality and positions each get one span; `FmStringReader.TryReadCString` uses one bounded block (gaps/zeros terminate). Unread bytes remain 0 → same null/skip decode as failed scalar reads. Pointer chains (name/nation/contract) stay scalar.
- **PR 2 Commit 2:** `DumpWriter.WriteCompact` emits unindented schema-v5 via `Utf8JsonWriter`, serializing each player then flushing so write chunks stay bounded (no second full JSON string). Atomic temp→`dump.json` replace unchanged. Generated 184k/500k minimal-player docs complete under the streaming tests.
- **PR 2 Commit 3:** Ingest measurement harness (`IngestTimings` + `ingest_dump_file_for_save_timed`) recorded generated minimal-player runs: 184k `validation_ms=5799` `insert_ms=2529` `total_ms=8329`; 500k `validation_ms=15551` `insert_ms=6797` `total_ms=22349`. Validation (parse+schema walk) dominated; removed the second full `serde_json::from_str` via `parse_and_validate_dump`, and reuse one prepared player `INSERT` statement per transaction. Existing rollback/replace tests stay green.
- **PR 2 Commit 4:** Request `maxAccepted` plumbed end-to-end. C# `BridgeRequest.MaxAccepted` + Plugin pass-through; `CapADumpPipeline` no longer coalesces null/omit to `DefaultMaxAccepted`. Rust `BridgeRequest.max_accepted` + production `request_player_dump` defaults to unlimited (`null`); `request_player_dump_with_limit` keeps explicit caps for tests/Commit 5. Dump wait timeout left at 120s (comfortable vs measured bridge ~26s).
- **Post-unlimited live validation** (2026-07-30, FM 26.3.2, unlimited request, `stoppedEarly=false`, build with Commit 4). Full paste: `.cursor/work/baselines/2026-07-30-unlimited-fullsave-diagnostics.txt`.
  - `candidatesAccepted=181210`, `regionCount=2168`, `bytesScanned≈3.94 GiB`
  - `regionEnumerationMs=48`
  - `candidateDiscoveryMs=17037` (~17s) — still largest phase
  - `extractionMs=5666`, `clubIndexingMs=1597`, `dumpWritingMs=1656`
  - `totalMs=26155` (~26s) — meets PR 2 bridge budget (&lt;60s) with headroom; no parallel workers needed
  - `processMemoryCalls=10260800`, `processMemoryRequestedBytes≈4.11 GiB`
  - Sample players decode sanely (known names/attrs). `clubUnresolved=18382` / `playersLinkedViaSquad=0` remain pre-existing club-link noise, not a scan-cap regression.
  - End-to-end Load Data wall clock not recorded in this paste; bridge alone leaves ample room under the 90s e2e budget if ingest stays near the generated-dump harness (~8s for 184k).
- **PR 2 Commit 5:** `load_data` IPC takes optional `maxAccepted`; AppTopBar Cap players checkbox + limit field (default off / unlimited; enable defaults to 500); preference persisted via `useLoadDataPreferences`. Frontend tests assert unlimited vs capped invoke args; Rust `scan_dump_from_bridge` forwards positive caps through `request_player_dump_with_limit`.

## Completed work

| PR | Commit | Hash | Notes |
| --- | --- | --- | --- |
| 1 | Add scan phase performance diagnostics | `b2c8663` | Phase ms + process-memory call/byte counts in diagnostics.txt |
| 1 | Add reusable block memory reads | `6f299da` | `TryReadBlock` + page-aligned failed-block subdivision |
| 1 | Scan heap regions from reusable blocks | `648b81f` | Block walk + in-buffer UID + vtable offset cache; live capped path `totalMs=6700` (~11× faster than pre-scan) |
| 2 | Batch contiguous player field reads | `8316c43` | Attrs/personality/positions/cstrings via `TryReadBlock` + ArrayPool decode |
| 2 | Stream compact full dump output | `a686189` | Utf8JsonWriter compact stream + per-player flush; 184k/500k scale tests |
| 2 | Reduce measured large-dump ingest overhead | `862486a` | Single parse + prepared INSERT; timed 184k/500k harness |
| 2 | Request-scoped scan limit and unlimited production default | `d8b7b04` | Request maxAccepted; production Load Data unlimited; caps for tests/Commit 5 |
| 2 | UI toggle and configurable player cap | `b67fc3b` | Cap players checkbox + limit; `load_data` maxAccepted; preference persist |

## Final validation

At feature end.

## Documentation impact

At feature end, reconcile `ARCHITECTURE.md`, `bridge/README.md`, `bridge/DUMP_SCHEMA.md` (request `maxAccepted` semantics), completed memory-read and snapshot-ingest records, TODO, and backlog.
