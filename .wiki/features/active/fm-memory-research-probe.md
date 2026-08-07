# FM Memory Research Probe

## Status

Active

## Intent

Add a reusable, developer-only probe for finding FM26 player data in memory. The probe must capture small, labeled memory samples for explicit player UIDs and compare those samples with FM-exported CSV values. It will support later offset research for Youth Academy career statistics and other numeric analysis without changing production snapshot data until an offset has been independently verified.

## User-visible behavior

- A developer can export a player view from FM, select explicit UIDs from that CSV, and request a bounded memory capture while the same save state remains open.
- The bridge writes a versioned `probe.json` with game, layout, module, candidate-address, root-window, and bounded pointer-target metadata for each matched UID.
- A repository command can correlate numeric CSV columns against one capture and can compare before-and-after CSV/capture pairs.
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
- Correlation results are hypotheses. Production layouts, dump schemas, SQLite, Academy behavior, and Moneyball features change only in later features after independent live validation.
- Raw CSV exports, probe captures, and analysis output stay untracked under `.work/` or outside the repository. They must not enter Git.

## Non-goals

- Discovering or shipping offsets for senior league appearances, goals, assists, international caps, or any other new production field.
- Changing dump schema v5, Rust dump validation, snapshot ingest, SQLite migrations, Academy DTOs, statistics UI, or Moneyball behavior.
- Adding a product-facing probe UI or sending raw memory over Tauri IPC.
- Capturing the whole heap, following arbitrary addresses supplied by the operator, or performing recursive unbounded pointer scans.
- Supporting non-numeric CSV correlation, save-file parsing, non-Windows FM editions, or unsupported FM builds.
- Treating one matching player, one matching value, or one changed byte as enough evidence for an offset.

## Current-state map

- Relevant components: `bridge/Plugin.cs` polls the production request file, owns one background scan thread, and serializes scans through `ScanGate`; `bridge/Scanning/CapADumpPipeline.cs` resolves layouts and coordinates candidate discovery and extraction.
- Data model: `PersonScanner` returns `PersonCandidate(ObjectAddress, Uid, Ca, Pa, ClassOffset)`. The player-block base is `ObjectAddress - ClassOffset`; the person object remains available at `ObjectAddress`.
- Memory boundary: `IMemoryReader` provides controlled scalar and block reads; `WindowsMemoryReader` uses `ReadProcessMemory`, and `Tests/Fakes/FakeMemoryReader` provides deterministic sparse memory.
- Layout boundary: `IFmMemoryLayout` and `Fm263Layout` own supported-build pins. Known anchors include UID, CA, PA, attributes, market value, and reputation.
- Output boundary: `DumpWriter` and `StatusWriter` demonstrate temporary-file plus atomic-replace JSON output. Production dump schema v5 is frozen and validated by Rust.
- Persistence and migrations: none for this feature. Probe artifacts are disposable research files and never enter SQLite.
- Existing behavioral assumptions: full candidate discovery takes about 26 seconds on the reference save and already retains the addresses needed to derive player and person bases. Live FM attach validation remains manual on Windows.
- Architectural seams: capture and raw-memory interpretation stay in the C# bridge; a developer CLI prepares probe requests and analyzes local CSV/JSON files; Rust, React, and snapshot ingest remain outside the feature.
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

The capture records two stable roots per matched candidate: player-block-relative and person-object-relative. It then follows only aligned pointer values found inside captured ranges, with named source paths, a small maximum depth, address deduplication, and a strict per-player byte budget. The request may select UIDs and a bounded preset, but it may not supply arbitrary process addresses or remove the built-in limits.

The .NET developer tool under `bridge/Tools/MemoryProbe/` owns CSV parsing, request creation and waiting, scalar interpretation, cross-player correlation, and before/after comparison. It uses runtime libraries only. `scripts/dev` exposes the tool as `memory-probe` so the repository retains one command surface.

One PR is appropriate. Capture without analysis is not yet a useful research workflow, while the analyzer depends directly on the probe schema. The final documentation commit closes the same developer capability and records the manual live validation contract.

## Uncertainty register

### Known

- Candidate discovery already yields UID, object address, class offset, CA, and PA for each accepted player.
- Known FM 26.3 layout pins provide enough anchors to test whether correlation finds correct relative locations and transforms.
- Academy currently represents senior league appearances, goals, assists, and international caps as unavailable nullable values.
- Production Load Data depends on frozen dump schema v5 and the existing `request.json` / `status.json` files.

### Assumptions

- Fixed player/person root windows plus a small, budgeted pointer traversal will expose enough nearby structure to research numeric player fields.
- FM player-view exports include a stable numeric UID column and can include varied known and target numeric columns.
- The existing full candidate walk is fast enough for an occasional developer probe; targeted scanner optimization is not required for the first version.

### Decisions

- Use a separate developer-only probe protocol instead of extending the product dump or Rust protocol.
- Capture raw bounded ranges and pointer provenance instead of adding speculative field offsets to `IFmMemoryLayout`.
- Provide single-capture correlation and before/after diff in one .NET CLI with no new package dependency.
- Keep research artifacts disposable and untracked; preserve only validated conclusions in later feature ledgers or layout code.
- Do not create an ADR. The feature stays within ADR-0016's established C# memory-reader boundary and does not change a product-facing or persisted contract.

### Unknowns

- Whether target career statistics are inline scalar totals, counters inside another object, or derived from history collections.
- Which pointer depth and target-window size will expose future fields without needless capture breadth.
- Which FM CSV header names and delimiters appear across locales; the tool must use explicit UID/field mappings and robust delimiter handling rather than one hard-coded English export.
- Whether a controlled in-game stat change updates one scalar immediately or several cached/aggregate locations.

### Risks

- Common small integers can produce many false matches. The analyzer must rank by multi-player coverage and show ambiguity rather than choose the first hit.
- A CSV export and probe taken from different save moments can create false negatives or false deltas. Request and result metadata plus the runbook must make synchronization explicit.
- Pointer-like values in raw bytes can fan out quickly. Hard depth, count, byte, and UID budgets must remain non-configurable ceilings.
- Adding probe polling to `Plugin.cs` can disturb shutdown or production request handling. Tests and review must cover priority, mutual exclusion, cancellation, and file isolation.
- Stats may require a structure-specific reader. If bounded generic capture cannot expose a known indirection in live validation, replan the probe instead of expanding into unbounded traversal.

## Walking skeleton

With fake memory, accept one research request containing one UID, reuse candidate discovery to locate it, capture bounded player/person root ranges plus one first-hop target, and atomically write a schema-v1 `probe.json` without touching any production protocol file. This proves the safe in-process path before CSV analysis is added.

## Delivery plan

### PR 1 — Add reusable FM memory research tooling

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(memory-probe): add reusable FM memory research tooling`

**Purpose:** Deliver one complete developer workflow for bounded UID capture, CSV correlation, before/after comparison, and repeatable live validation. The capture and analyzer share one research schema and therefore belong in one review and merge boundary.

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

**Status:** Active

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

#### Commit 3 — Document and validate the research workflow

**Status:** Pending

**Provisional commit:** `docs(memory-probe): document the research workflow`

**Work:** Add the durable developer runbook, document the separate probe files and command surface in the bridge README, and update current architecture after the code exists. Define the manual FM view, synchronization, varied-player correlation, controlled before/after capture, artifact handling, and evidence required before a later feature may pin a production field.

**Out of scope:**

- Recording speculative Academy or Moneyball offsets.
- Committing raw CSV, raw memory, screenshots, or machine-specific paths.
- Expanding product documentation or presenting the probe as an end-user feature.

**Implementation packet:**

- Owners and files: new focused runbook under `bridge/`; `bridge/README.md`; `.wiki/ARCHITECTURE.md`; this ledger for validation evidence and discoveries.
- Existing patterns to verify: bridge Windows prerequisites and install steps, manual force-scan runbook, `.wiki/INDEX.md` ownership, `.work/` disposal rule, and completed memory-read validation language.
- Constraints and invariants: distinguish FM-exported labeled truth from production data flow; define exact statistic semantics before research; require synchronized captures, varied values, controlled deltas, and independent revalidation; keep raw artifacts untracked.
- Dependencies and ordering: document only commands and files delivered by commits 1 and 2. Run live validation with a freshly built and installed DLL before claiming the workflow works in FM.

**Implementation profile:** Luna Max — once the tool behavior is settled, the remaining work is a bounded runbook and evidence reconciliation task with strong repository analogues.

**Review profile:** Sol Medium — review should check that instructions match the implemented commands and preserve the product/research boundary without requiring new architectural judgment.

**Validation:** Run `./scripts/dev memory-probe --help`, `./scripts/dev bridge-test`, and `./scripts/dev check`. On Windows with FM 26.3 loaded, run the final manual validation below after `./scripts/dev bridge-install`; keep raw evidence under `.work/` and record only the concise result in this ledger.

**Stop conditions:** Return to the owning implementation commit if the documented command or schema differs from reality. Replan if the live probe cannot locate known UIDs, cannot recover known pinned fields, disturbs Load Data, or requires wider-than-approved capture bounds.

**Review mandate:**

- Verify every command, path, filename, prerequisite, and expected terminal state against implementation.
- Verify the manual CSV checklist states field semantics and synchronization requirements clearly.
- Verify known-offset recovery and controlled-delta checks are mandatory before feature completion.
- Verify raw memory and CSV artifacts are directed to `.work/` or external local storage and excluded from Git.
- Verify architecture describes only implemented behavior and still names schema v5 as the product dump contract.
- Verify the runbook does not imply that correlation alone validates a production offset.

## Active work

**PR:** PR 1 — Add reusable FM memory research tooling

**Commit:** Correlate CSV truth with probe captures

### RED proof

Add deterministic capture and CSV fixtures where duplicate small integers make a one-player match ambiguous, while multi-player coverage identifies the known relative path. The initial RED must fail for missing correlation behavior rather than CSV or test-harness setup. Add a synchronized before/after fixture with one controlled scalar delta.

### Expected outcome

The dependency-free .NET developer CLI can create and await a bounded probe request, correlate explicit numeric CSV mappings with one compatible capture, and compare synchronized before/after CSV and capture pairs. Its output ranks candidate paths and encodings while making ambiguity and unmatched data visible.

### Explicit exclusions

- No candidate is declared a verified FM layout offset.
- No domain-specific appearance-history or competition-record parsing.
- No product UI, IPC, persisted analysis results, charts, or general-purpose data science tooling.
- No new CSV or command-line package.

## Discoveries and replanning

Record material deviations, blockers, and decisions that change remaining work. State what was planned, what changed, and why.

- Planning selected a separate research protocol so the reusable tool cannot accidentally become part of the frozen product dump contract.
- Repowise showed `Plugin.cs`, `PersonScanner.cs`, and `CapADumpPipeline.cs` as bridge hotspots; its index was behind live `HEAD`, so file relationships and all planning facts were verified against current source.
- Commit 1 fixed the player root at 0x280 bytes and the person root at 0x100 bytes (1,408 bytes maximum per player). The player window covers the known CA, PA, Determination, and market-value anchors; unread root or target bytes now fail before output replacement.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Capture bounded player memory by UID | Pending record | Isolated C# research protocol, scan-gate integration, bounded capture, atomic output, and fake-memory coverage | Sol High accepted after one correction round; no Critical, High, or Medium findings remain | Root window set to 0x280 bytes during review to include planned known anchors; no scope change. |
| PR 1 | Correlate CSV truth with probe captures | Pending record | Pending | Pending | None |
| PR 1 | Document and validate the research workflow | Pending record | Pending | Pending | None |

## Final validation

**Feature review profile:** Sol High — the final review must connect in-process memory safety, isolated file lifecycle, CLI evidence quality, and live operational instructions across all three commits.

Automated evidence:

- `./scripts/dev memory-probe --help`
- `./scripts/dev bridge-test`
- `./scripts/dev check`
- Fresh-context Sol High feature review over the exact recorded implementation commits.

Manual Windows/FM evidence:

1. Build and install the current DLL with `./scripts/dev bridge-install`, restart FM26, and load a supported save.
2. Export at least eight varied players in one FM view. Include UID, name, club, CA, PA, Determination, market value, and any research targets, with the target statistic semantics written down.
3. Without advancing or changing the save, run a UID capture from that CSV and confirm the matching request ID, supported game/layout metadata, requested UID set, bounded byte totals, and successful research status.
4. Run correlation and confirm it reports the known UID, CA, PA, Determination ×5, and market-value paths with the correct interpretations and transparent ambiguity.
5. Take synchronized before/after CSV and probe pairs around one controlled in-game numeric change. Confirm diff mode pairs the same UIDs and narrows the changed candidates to the correct relative path in a known fixture and a plausible bounded set in the live capture.
6. Confirm a malformed request, unsupported layout, and missing UID fail only the research status and leave the prior `dump.json`, production `status.json`, and prior successful `probe.json` unchanged.
7. Keep all CSV, probe, analysis, and diagnostic artifacts under `.work/memory-probe/` or outside the repository. Record only concise pass/fail evidence and any plan-changing discovery here.

## Documentation impact

Complete during reconciliation. Expected owners are the bridge research runbook, `bridge/README.md`, `.wiki/ARCHITECTURE.md`, `.wiki/TODO.md`, and the completed feature record created from this ledger.
