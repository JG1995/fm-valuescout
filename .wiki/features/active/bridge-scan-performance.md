# Bridge Scan Performance

## Status

Active

## Intent

Make **Load Data** fast enough to ingest a complete FM26 player database. Replace the bridge's syscall-heavy scalar heap walk with safe block scanning, then remove the temporary 500-player production cap after live validation proves the full path stays within budget.

## User-visible behavior

- A 500-player diagnostic load completes in less than 10 seconds on the current reference machine.
- A complete reference save of about 184,000 players completes the bridge dump in less than 60 seconds and finishes Load Data in less than 90 seconds.
- Production Load Data no longer truncates the player database at 500 players.
- Failed scans and failed ingests preserve the prior good snapshot.
- Unsupported FM builds still fail closed.

The live-save budgets are provisional until Commit 1 records phase timings on the reference machine. A synthetic 500,000-player downstream run must complete without out-of-memory failure; it is a scale check, not a claim that a comparable live save exists.

## Invariants

- All FM memory access remains read-only and uses safe `ReadProcessMemory` failure handling.
- Scanning and dump I/O stay off the Unity main thread.
- UID deduplication, dynamic class-offset checks, CA/PA bounds, identity validation, and versioned layout pins remain authoritative.
- The bridge replaces `dump.json` only after a non-empty successful scan.
- Dump schema v5 and the C# file-protocol → Rust validation → transactional SQLite boundary remain stable unless measurements prove a contract change is necessary.
- A full scan sets `scanTruncated: false` and `maxAccepted: null`.

## Non-goals

- Unsafe pointer dereferences inside FM.
- A new global player-container pointer or patch-specific object-root traversal without separate reverse-engineering evidence.
- Save-file parsing, background synchronization, or incremental snapshots.
- UI redesign or role scoring.
- Parallel scanning by default. Add bounded workers only if the single-thread block scanner misses the live budget.
- Progress UI or a request-configurable player cap unless measured scan duration still requires them.

## Current-state map

- Relevant components: `bridge/Scanning/PersonScanner.cs`, `CapADumpPipeline.cs`, `bridge/Memory/WindowsMemoryReader.cs`, extraction readers, `DumpWriter.cs`, and Rust snapshot ingest.
- Data model: dump schema v5 stores rich player objects; production scans currently stop at `PersonScanner.DefaultMaxAccepted = 500`.
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
- Keep the 500-player cap until the optimized scanner passes correctness checks and a full live reference run.
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

**Status:** Active

**Provisional PR title:** `perf(bridge): replace scalar heap reads with block scanning`

**Purpose:** Remove the confirmed heap-scan bottleneck behind minute-long capped loads while preserving current output and the 500-player safety cap.

**Depends on:** FM26 memory read and snapshot ingest. Manual measurements require a freshly built and installed bridge DLL.

#### Commit 1 — Add scan phase performance diagnostics

**Status:** Completed — hash pending checkpoint commit

**Work:** Record elapsed time for region enumeration, candidate discovery, player extraction, club indexing, dump writing, and total bridge work. Record process-memory call and requested-byte counts without changing dump schema v5.

**Out of scope for this commit:**
- Scanner algorithm changes.
- Cap, timeout, JSON, SQLite, or UI changes.
- New profiling or benchmark dependencies.

**Validation:** Add focused C# assertions that diagnostics contain every named phase with non-negative values, including region enumeration, plus process-memory call and requested-byte counts. Run `dotnet test`, `dotnet build`, and one fresh 500-player Windows/FM baseline.

**Provisional commit:** `feat(bridge): add scan phase performance diagnostics`

#### Commit 2 — Add reusable block memory reads

**Status:** Pending

**Work:** Add a direct caller-owned byte-array block-read path that avoids the current intermediate allocation and copy. Add bounded subdivision for failed blocks while preserving short-read and invalid-address behavior in the Windows implementation and fake reader.

**Out of scope for this commit:**
- Replacing the scanner loop.
- Parallel reads.
- Extraction batching.

**Validation:** RED tests cover full, partial, failed, and region-edge block reads. Run bridge tests and build.

**Provisional commit:** `perf(bridge): add reusable block memory reads`

#### Commit 3 — Scan heap regions from reusable blocks

**Status:** Pending

**Work:** Replace per-word process-memory calls with bounded region blocks, local aligned-word inspection, in-buffer UID reads, and cached module metadata or vtable-to-class-offset results. Stop immediately when a diagnostic cap is reached and preserve truncation semantics.

**Out of scope for this commit:**
- Player field extraction batching.
- Parallel region workers.
- Full production scans.

**Validation:** Characterization tests prove the same accepted UIDs, CA/PA values, deduplication, cancellation, and fail-safe behavior. A counting reader proves read calls scale with blocks, not 8-byte slots. Fresh Windows/FM validation must put the capped 500-player path below 10 seconds. The uncapped reference-save budget belongs to PR 2.

**Provisional commit:** `perf(bridge): scan heap regions from reusable blocks`

**Merge to trunk when:** C# tests/build pass; the capped live path meets budget; diagnostics prove the candidate scan no longer performs scalar heap reads; schema and prior-dump safety remain unchanged.

### PR 2 — Enable complete player snapshots

**Status:** Pending

**Provisional PR title:** `perf(load-data): enable complete player snapshots`

**Purpose:** Scale the post-discovery path, validate a complete live dump, and remove the temporary production cap.

**Depends on:** PR 1 merged to `main`.

#### Commit 1 — Batch contiguous player field reads

**Status:** Pending

**Work:** Read contiguous attribute, position, personality, and bounded string ranges into reusable buffers, then decode locally. Keep pointer-chain reads safe and nullable.

**Out of scope for this commit:**
- Layout changes or new fields.
- Parallel extraction.
- JSON and SQLite changes.

**Validation:** Existing extraction tests stay green; new counting-reader tests prove field reads are batched without changing decoded players. Compare extraction phase timing on the reference save.

**Provisional commit:** `perf(bridge): batch contiguous player field reads`

#### Commit 2 — Stream compact full dump output

**Status:** Pending

**Work:** Write compact schema-v5 JSON incrementally to the existing temporary file and retain atomic replace-on-success behavior.

**Out of scope for this commit:**
- Schema or protocol version changes.
- Chunked Rust ingestion.
- Snapshot history.

**Validation:** Existing schema/round-trip tests remain green; generated 184,000- and 500,000-player documents complete without a second full serialized copy or out-of-memory failure.

**Provisional commit:** `perf(bridge): stream compact full dump output`

#### Commit 3 — Reduce measured large-dump ingest overhead

**Status:** Pending

**Work:** Add a generated large-dump measurement around validation and transactional ingest. Reuse prepared SQLite work and remove duplicate parse work only where the measurement shows material cost.

**Out of scope for this commit:**
- Database schema changes.
- Snapshot history.
- Role scoring.

**Validation:** Existing rollback and replacement tests stay green. Generated 184,000- and 500,000-player runs record validation, insert, and total ingest timing and complete without unbounded memory growth.

**Provisional commit:** `perf(snapshot): reduce large dump ingest overhead`

#### Commit 4 — Enable complete production data loads

**Status:** Pending

**Work:** Remove the temporary production cap, retain explicit caps only in tests or diagnostics, align the Rust wait timeout with the measured envelope if needed, and update operational documentation with reference-save results.

**Out of scope for this commit:**
- User-configurable scan limits.
- Background refresh or incremental scans.
- New progress UI unless the measured full path cannot provide acceptable feedback with the existing loading state.

**Validation:** A fresh full Windows/FM run loads the complete reference save, reports `scanTruncated: false` and `maxAccepted: null`, preserves the prior snapshot on forced failure, completes the bridge phase below 60 seconds and end-to-end Load Data below 90 seconds, then passes `./scripts/dev check` and `./scripts/dev test`.

**Provisional commit:** `perf(load-data): enable complete player snapshots`

**Merge to trunk when:** Complete live data passes the reference budget and correctness spot checks; generated 500,000-player output/ingest completes without out-of-memory failure; timeout and docs match measured behavior.

## Active work

**PR:** PR 1 — Replace scalar heap reads with block scanning

**Commit:** Add scan phase performance diagnostics

### RED test (active commit)

Format diagnostics for a completed fake pipeline run and require named duration fields for region enumeration, candidate discovery, extraction, club indexing, dump writing, and total time, plus process-memory call and requested-byte counts. It fails today because the bridge exposes scan-result counters but no phase timings or read-volume evidence, which would leave later optimization decisions unmeasured.

### Expected outcome

`diagnostics.txt` provides a stable phase breakdown and memory-read evidence for capped and full scans without changing dump schema v5 or user behavior.

### Explicit exclusions

- No scanner optimization yet.
- No cap or timeout change.
- No dependency, UI, dump-schema, or SQLite change.

## Discoveries and replanning

- Initial analysis found a stronger cause than the backlog's tentative region-filter direction: the scanner makes an allocating process-memory call for every aligned 8-byte heap word and repeats metadata reads for millions of vtable hits. Optimize read granularity before narrowing regions.
- Public FMSuperScout history provides directly comparable evidence for safe block scanning and cached module metadata. Genie Scout does not publish actionable implementation details.
- If the single-thread block scanner misses the candidate-discovery budget, add one pending PR 1 commit for bounded worker-local scanning after reporting the measured CPU and memory trade-off. Do not add it preemptively.
- If large-dump validation and ingest already meet budget, remove PR 2 Commit 3 rather than refactor Rust without evidence.

## Completed work

| PR | Commit | Hash | Notes |
| --- | --- | --- | --- |
| — | — | — | No implementation completed |

## Final validation

At feature end.

## Documentation impact

At feature end, reconcile `ARCHITECTURE.md`, `bridge/README.md`, `bridge/DUMP_SCHEMA.md`, completed memory-read and snapshot-ingest records, TODO, and backlog.
