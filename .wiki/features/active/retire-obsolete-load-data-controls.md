# Retire obsolete Load Data controls

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** eac156fe9cfbcafde3d1ed29c669bf25aaf44c191d7ef5a81a875b10725e7f25

## Intent

Bundle Linear JAY-55 and JAY-56 into one cleanup of obsolete Load Data control support. JAY-55 removes the accepted-player cap from the active app IPC, Rust request writer, and C# bridge scan path so every new scan is unlimited. JAY-56 records that snapshot freshness-indicator support was already removed in top-navigation PR #120 (`d50ff0c` within squash `5e5b438`) and retains the timestamp data that still serves snapshot history, overview, and ordering.

## User-visible behavior

- Clicking **Load Data** starts an unconditional unlimited scan. The app sends no `maxAccepted` argument to Tauri, and Tauri writes no `maxAccepted` property to a new full-dump request.
- A new bridge accepts an old app's extra `maxAccepted` JSON property but ignores it and scans fully. A new app that omits the property remains compatible with an old bridge, which already treats omission as unlimited.
- New dumps still write `scanTruncated: false` and `maxAccepted: null`. Existing capped dumps and stored snapshots remain ingestible and readable. Snapshot Overview warns when a retained capped snapshot is current.
- Snapshot Overview and Snapshot History continue to show `loadedAtUtc`; Snapshot History retains timestamp provenance only and does not show cap metadata or an incomplete-snapshot warning. Rust continues to persist `loaded_at_utc` and uses it as the equal-game-date ordering tie-breaker before snapshot ID.
- The product has no snapshot-freshness chip. No new UI behavior is added.

## Invariants

- Preserve dump schema v8 and SQLite schema. Do not add a dump-schema bump or a migration solely for this cleanup.
- Preserve dump parsing, SQLite columns, and nested snapshot DTO fields for `scanTruncated` and `maxAccepted`. Preserve the incomplete-snapshot warning only in Snapshot Overview for a capped retained current snapshot.
- Preserve `generatedAtUtc`, game and bridge version provenance, and all unrelated bridge status and boost fields. They are not freshness-only state.
- Preserve explicit Load Data, context-token capture and revalidation, progress phases, prior-snapshot safety, rollback behavior, and bridge request serialization safety.
- Keep `loadedAtUtc`/`loaded_at_utc` persisted and exposed where Snapshot Overview, History, and deterministic selection need it.
- Keep the request/status protocol at version 1. Optional property omission and unknown-property tolerance provide compatibility; no protocol-version bump is warranted.
- Do not plan a generated `src-tauri/resources/FmDataBridge.dll` update or a release. The tracked file is an intentional placeholder; release packaging builds `bridge/bin/Release/net6.0/FmDataBridge.dll` through `src-tauri/tauri.release.conf.json`.

## Non-goals

- Remove persisted snapshot cap metadata, historical warnings, dump-validation rules, fixtures, or database columns.
- Remove `generatedAtUtc`, game-version, bridge-version, or protocol-version provenance.
- Change snapshot retention, game-date ordering, context/boost gates, bridge scan safety, player database scope, or progress and rollback behavior.
- Add a freshness replacement, background sync, UI controls, a migration, a dump-schema change, generated binary update, release, manual FM attach test, or UI inspection.
- Make unrelated cleanup to historical feature records, BACKLOG, ADRs, tests, or bridge diagnostics.

## Current-state map

- Relevant app input boundary:
  - `src/app/components/app-top-bar.tsx::AppTopBar` calls `useLoadData` with `null`; the prior cap controls were deleted in `efd0983`.
  - `src/features/memory-read/hooks/use-load-data.ts::useLoadData` still types the mutation variable as `number | null` and forwards it to `loadData`.
  - `src/features/memory-read/api/load-data.ts::loadData` serializes `maxAccepted` into Tauri `load_data` IPC.
  - `src-tauri/src/features/snapshot/commands.rs::load_data` accepts `max_accepted` and passes it through `execute_load_data_with` to the scan closure.
- Relevant Rust file-protocol boundary:
  - `src-tauri/src/features/memory_read/service.rs::{BridgeRequest, request_player_dump_with_limit}` serializes `maxAccepted` in `request.json`; `snapshot/load_data.rs` forwards the same optional limit.
  - `BridgeStatus` and `DumpRequestResult` currently carry transient `scan_truncated` and `max_accepted` from ready `status.json`; `LoadDataResult` and `LoadDataResultDto` expose them only as top-level Load Data result metadata.
- Relevant C# bridge boundary:
  - `bridge/Protocol/{BridgeRequest,RequestAcceptance}.cs` parses and validates full-dump caps.
  - `bridge/Plugin.cs::{TryStartBridgeWorkFromRequestOrForceFlag, RunDumpScan, WriteStatus}` forwards the accepted cap and emits transient ready-status cap metadata.
  - `bridge/Scanning/{CapADumpPipeline,PersonScanner,ScanDiagnostics,PersonScanResult}.cs` implements the cap, serial/parallel switch, early-stop diagnostics, and pipeline result fields.
  - `bridge/Models/DumpDocument.cs` and `bridge/Output/DumpWriter.cs` own persisted dump metadata. These fields remain for historical compatibility and must receive `false`/`null` for every new scan.
- Persisted snapshot compatibility metadata:
  - `src-tauri/src/features/memory_read/dump_validation.rs` and `src-tauri/src/features/snapshot/ingest.rs` require and validate dump `scanTruncated` and nullable `maxAccepted`.
  - `snapshots.scan_truncated` and `snapshots.max_accepted`, `SnapshotSummaryDto`, frontend `SnapshotSummary`, and memory-read-local `LoadDataSnapshotSummary` preserve old cap evidence. `LoadDataSnapshotSummary` remains the minimal local response projection for stored/effective Load Data snapshots; it must not import the snapshot feature or create a shared type solely for this cleanup. `SnapshotOverviewPanel` presents the incomplete warning when that retained snapshot is current.
  - `SnapshotHistoryPanel` lists snapshot date, identity, player count, and `loadedAtUtc` provenance only; its metadata has no cap fields and it shows no incomplete warning. `LoadDataOutcome` currently has an obsolete new-load capped-success branch, which Commit 8 removes.
  - The nested `storedSnapshot` and `effectiveSnapshot` fields are persisted snapshot metadata. They are distinct from the removable top-level Load Data result fields.
- Snapshot timestamp and freshness audit:
  - `src-tauri/src/db/migrations.rs` persists `loaded_at_utc`; `src-tauri/src/features/snapshot/service.rs::SNAPSHOT_ORDER_BY` orders equal game dates by `loaded_at_utc DESC, id DESC`.
  - `SnapshotMetadataDto`, `SnapshotSummaryDto`, frontend snapshot types, `SnapshotOverviewPanel`, and `SnapshotHistoryPanel` expose and display the timestamp.
  - The freshness chip and evaluator were deleted in `d50ff0c`; no production freshness evaluator remains. `.wiki/ARCHITECTURE.md` still incorrectly describes the chip, optional cap controls, `useLoadDataPreferences`, `load_data(maxAccepted)`, capped-reader serial behavior, and top-level `LoadDataResult.scanTruncated`/`maxAccepted` fields.
- Documentation and validation:
  - `bridge/README.md` still documents capped live requests and scans.
  - `bridge/DUMP_SCHEMA.md` must continue to document historical fields, but must say that new dumps always emit `scanTruncated: false` and `maxAccepted: null`.
  - Focused frontend tests use `./scripts/dev test`; the .NET 6.0.428 bridge suite is available as `./scripts/dev bridge-test` and currently passes 224 tests with 3 skips. The project gates are `./scripts/dev check`, `./scripts/dev check-rust`, and `./scripts/dev smoke`.
- Primary risks:
  - Removing the input property without removing all downstream request plumbing would leave a hidden cap contract or break old/new bridge compatibility.
  - Removing top-level transient status/result fields without preserving nested snapshot fields would erase historical warning evidence.
  - Treating `loadedAtUtc` as freshness-only would break overview/history display or equal-date ordering.
  - Removing cap tests indiscriminately could weaken full-scan, historical metadata, context, progress, or rollback coverage.

## Feature architecture

React supplies only the captured save context and progress channel to `load_data`. Rust writes an unconditional full-dump request, waits for a terminal status, captures the dump, and ingests its persisted metadata. The C# bridge accepts a full-dump request without a cap contract, always scans its complete candidate regions, and writes fixed new-dump metadata (`false` and `null`). Commit 7 retires bridge-only transient producers; Commit 8 retires Rust/React consumers because each side remains backward-compatible while deployed separately. Snapshot ingest retains nested persisted values for old data in `SnapshotSummaryDto`, frontend `SnapshotSummary`, and memory-read-local `LoadDataSnapshotSummary`; Snapshot Overview alone presents the retained-current incomplete warning.

The documentation audit is deliberately separate from implementation. It corrects the already-retired freshness claim first. Each later documentation change ships with the behavioral outcome that makes it true.

## Uncertainty register

### Known

- `main` and `origin/main` are synchronized at `b99755ff23f801fbff45ec55bde6e05fb581de11`.
- No active ledger, planned spec, or warranted ADR exists. BACKLOG does not change.
- The prior top-navigation cleanup already deleted the freshness chip, thresholds, cap controls, and `useLoadDataPreferences`; only stale documentation remains for JAY-56.
- New-app/old-bridge compatibility already exists because an omitted `maxAccepted` deserializes as `null` in the old bridge.
- System.Text.Json ignores an unmodeled old-app `maxAccepted` property under the current default configuration, so the new bridge can accept and ignore that property without a protocol version change.
- The tracked resource DLL is a placeholder. `src-tauri/tauri.release.conf.json` packages the release-built bridge DLL from source.

### Assumptions

- One short-lived GitHub PR remains the smallest review boundary. JAY-55 and JAY-56 overlap in the same Load Data contracts, test assets, and documentation; splitting them would create two reviews of one cleanup seam with no independent user value.
- Recent repository practice and the approved delivery boundary use `.github/pull_request_template.md`, squash merge, and required strict GitHub Actions status `check`.
- Automated frontend, Rust, and bridge proofs are sufficient for this no-new-UI cleanup. A manual FM attach test and `inspect-ui` would not prove a stronger supported behavior.

### Decisions

- The developer-approved compatibility direction is retained: a new app omits the property for an old bridge, and a new bridge ignores an old app's extra property and scans fully.
- `loadedAtUtc` remains a snapshot timestamp, not freshness-indicator-only state. Its display and deterministic ordering are supported behavior.
- Dump and database schemas remain unchanged. Historical capped artifacts remain readable; new scans write compatible fixed metadata.
- Test philosophy is developer-approved: delete or rewrite cap-specific tests and helpers with their removed contract; retain positive full-scan, historical capped dump/snapshot validation and presentation, loaded-at display/order, and unrelated context/progress/rollback proofs. Do not add source-absence padding. One positive old-app/new-bridge boundary proof is permitted only when it proves acceptance and unlimited processing.
- No ADR is warranted. This removes obsolete behavior inside established UI, Tauri, Rust, and bridge boundaries; no durable structural alternative remains.

### Unknowns

- None that block planning. Delivery must stop and replan if an apparently cap-only test or type proves to protect another supported contract, if the old-property compatibility test cannot establish unlimited processing at an observable boundary, or if a removal requires a schema or protocol-version change.

### Risks

- The ingress, request, scanner, and result changes span three languages. Each commit must remain buildable and remove its companion tests in the same diff.
- The bridge status shares boost support and result fields with the retired cap metadata. Commit 7 must preserve those fields and error sanitization while removing only optional producers; Commit 8 must tolerate old optional status properties while removing host consumers.
- Historical cap fixtures are intentionally retained. Any change that makes their dump validation, SQLite persistence, or Snapshot Overview retained-current warning fail is a blocker, not cleanup.
- `.wiki/ARCHITECTURE.md`, `bridge/README.md`, and `bridge/DUMP_SCHEMA.md` describe different layers and must be reconciled only with the true outcome they document.

## Walking skeleton

The planning commit establishes the ledger. The first implementation change corrects the already-stale freshness documentation. The next two commits remove the app input and Rust request property, so a new app writes an unlimited request. The bridge then stops accepting the cap as behavior and deletes scanner mechanics while keeping compile-required fixed transient fields. Commit 7 removes only optional C# transient producers; Commit 8 removes compatible Rust/React consumers and unreachable new-load UI. Nested historical snapshot metadata remains visible through Snapshot Overview. Every step is green and reversible in the sole PR.

## Delivery plan

### PR 1 — refactor(load-data): retire obsolete control support

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/retire-obsolete-load-data-controls`

**Base branch:** `main`

**Base ref:** `b99755ff23f801fbff45ec55bde6e05fb581de11`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub required strict status `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `refactor(load-data): retire obsolete control support`

**Purpose:** Deliver the combined JAY-55/JAY-56 cleanup as one reviewable and trunk-safe PR. Documentation, app IPC, file protocol, bridge behavior, and transient reporting are ordered atomic outcomes, not separate PR seams.

**Depends on:** None. Branch activation must start from the recorded synchronized `main` base. Ledger state does not create or switch the branch.

#### Commit 1 — Record the approved cleanup plan

**Status:** Completed

**Provisional commit:** `docs(load-data): record approved cleanup plan`

**Work:** Commit the independently reviewed planning artifacts before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, BACKLOG, ADRs, completed records, and unrelated documentation.

**Implementation packet:**

**Files and responsibilities:**

- `.wiki/features/active/retire-obsolete-load-data-controls.md` — approved JAY-55/JAY-56 intent, invariants, delivery authority, packets, and validation contract.
- `.wiki/TODO.md` — move this feature from no active work into Active with its ledger link.
- `.wiki/BACKLOG.md` — no change; no deferred work changes ownership.
- `.wiki/features/planned/retire-obsolete-load-data-controls.md` — no change; no planned spec exists.
- `.wiki/decisions/` — no change; no ADR is warranted.

**Behavior and data flow:**

- Record one reviewed source of planned feature truth before any code or documentation implementation change.

**Ordered implementation steps:**

1. Verify the feature branch and recorded base without changing Git state.
2. Confirm only the reviewed ledger and TODO paths are in the planning diff; leave the pre-existing completed-ledger formatting change untouched.
3. Run the ledger classifier and inspect the exact planning diff.
4. Stage only the two reviewed planning paths for independent checkpoint review.

**Tests and proof:**

- Not applicable — independently reviewed planning documents only. The ledger classifier proves schema state and the diff proves path scope.

**Patterns to verify:**

- `.wiki/features/active/README.md` schema 2 template, `.wiki/INDEX.md` document ownership, and the current TODO/BACKLOG lifecycle.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, BACKLOG, ADRs, a planned spec, or the accepted packet order.

**Dependencies and sequencing:**

- Requires a clear independent plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/retire-obsolete-load-data-controls.md`.

**Stop conditions:** Stop on an uncleared review, classifier error, branch/base mismatch, unexpected path, substantive post-review plan change, or unrelated completed-ledger modification in the staged diff.

**Review mandate:** Verify the staged diff contains only the complete reviewed ledger and TODO Active entry, records BACKLOG/planned-spec/ADR as no change, and does not include the pre-existing completed-ledger formatting edit.

#### Commit 2 — Reconcile retired freshness documentation

**Status:** Completed

**Provisional commit:** `docs(snapshot): reconcile retired freshness metadata`

**Work:** Correct the current-state freshness claim that became stale when PR #120 removed the chip. Retain timestamp persistence, display, and ordering as implemented behavior.

**Size assessment:** Documentation only; no implementation code.

**Out of scope:**

- Player-cap documentation, app/bridge code, tests, schema changes, generated artifacts, and any new freshness behavior.

**Implementation packet:**

**Files and responsibilities:**

- `.wiki/ARCHITECTURE.md` — remove the AppTopBar snapshot-freshness claim, state that the active save/snapshot surfaces retain their actual timestamp presentation, and keep `loaded_at_utc` as the equal-game-date tie-breaker where the architecture describes snapshot selection.

**Behavior and data flow:**

- Documentation reflects the existing implementation: Rust persists and orders by `loaded_at_utc`; snapshot DTOs expose it; Overview and History present it; no top-bar freshness evaluator remains.

**Ordered implementation steps:**

1. Compare the stale architecture statements with `d50ff0c`, snapshot DTOs, `SNAPSHOT_ORDER_BY`, and Snapshot Overview/History.
2. Remove only freshness-indicator claims and add only supported timestamp retention and ordering facts.
3. Do not remove the adjacent cap-control claim; Commit 3 owns it once app IPC no longer accepts the parameter.
4. Inspect the prose diff for current-state accuracy.

**Tests and proof:**

- Not applicable — documentation-only correction. Verify the retained facts against `src-tauri/src/features/snapshot/service.rs`, `src-tauri/src/features/snapshot/commands.rs`, and `src/features/snapshot/components/{snapshot-overview-panel,snapshot-history-panel}.tsx`.
- Retain loaded-at display and equal-date ordering tests. Do not add a test that asserts deleted freshness source is absent.

**Patterns to verify:**

- The `d50ff0c` top-bar freshness deletion and the existing `SNAPSHOT_ORDER_BY` plus Overview/History timestamp presentation.

**Constraints and non-goals:**

- Do not describe planned cap removal as implemented. Do not remove `loadedAtUtc`, `generatedAtUtc`, or provenance fields.

**Dependencies and sequencing:**

- Requires Commit 1. It is independently mergeable because PR #120 already removed the freshness behavior.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/retire-obsolete-load-data-controls.md` and the repository Markdown/whitespace check when available.

**Stop conditions:** Stop and replan if source shows a live freshness evaluator or if an architecture statement cannot be supported by current source and tests.

**Review mandate:** Confirm the diff removes only stale freshness documentation, accurately retains timestamp persistence/display/order, leaves cap documentation for its owning later outcome, and changes no executable path.

#### Commit 3 — Remove cap from app IPC

**Status:** Completed

**Provisional commit:** `refactor(load-data): remove cap from app IPC`

**Work:** Remove the accepted-player cap from the React hook/API and public Tauri `load_data` input. The app sends only captured save context and the progress channel; the existing Rust internal request path temporarily continues to write its unlimited form until Commit 4 removes the property entirely.

**Size assessment:** Estimated under 120 changed non-test implementation lines. Within the soft target; frontend and command signature changes form one public IPC contract.

**Out of scope:**

- Rust request serialization, C# bridge behavior, scanner mechanics, transient result fields, persisted snapshot metadata, and freshness documentation already owned by Commit 2.

**Implementation packet:**

**Files and responsibilities:**

- `src/app/components/app-top-bar.tsx::AppTopBar` — invoke the Load Data mutation without a cap value.
- `src/features/memory-read/api/load-data.ts::loadData` — remove the cap argument and omit `maxAccepted` from `invokeCommand` arguments.
- `src/features/memory-read/hooks/use-load-data.ts::useLoadData` — change the mutation variable to no value and forward only captured context plus progress callback.
- `src/features/memory-read/types/load-data.ts` — retain nested snapshot `scanTruncated`/`maxAccepted` fields; do not remove top-level transient result fields yet.
- `src-tauri/src/features/snapshot/commands.rs::load_data` — remove the public `max_accepted` command argument and invoke the existing orchestrator's unlimited path while Commit 4 retains its internal optional parameter.
- `.wiki/ARCHITECTURE.md` — remove the now-false optional cap-control, `useLoadDataPreferences`, and `load_data(maxAccepted)` current-state statements; describe the app IPC without a cap input.
- Tests: `src/features/memory-read/api/load-data.test.ts`, `src/features/memory-read/hooks/use-load-data.test.tsx`, and `src/app/app-top-bar.test.tsx` — delete or rewrite cap-input assertions and retain positive progress, captured-context, invalidation, and successful full-load proofs. Update only shared mock types that no longer compile because the input shape changed.

**Behavior and data flow:**

- The top-bar action calls a void mutation. The hook captures save ID/context token, creates the typed progress channel, and invokes `load_data` with no cap property. Tauri validates the same context and calls its temporarily unlimited internal scan seam. Existing old bridge behavior remains compatible because it receives `null`/omitted cap semantics until Commit 4 stops serializing the property.

**Ordered implementation steps:**

1. Establish the removal proof by identifying the cap argument assertions in the app API, hook, top-bar, and command tests; retain the positive full-load and progress assertions.
2. Remove the React parameter and mutation variable, then remove the command parameter while preserving captured context and the progress channel.
3. Rewrite the affected tests to prove a Load Data IPC request omits `maxAccepted` while still carries `saveId`, `contextToken`, and `onProgress`.
4. Reconcile the matching architecture statements now that the public app IPC shape is true.
5. Run focused frontend tests, affected Rust command tests, and the commit gate.

**Tests and proof:**

- Observable behavior: clicking Load Data sends a context-bound IPC request with a progress channel and no cap property; the established full-load success, progress, context switch, invalidation, and rollback behavior still works.
- Delete the obsolete cap-input test and helper assertions with the removed interface. Do not add source-absence padding; an omitted property at the observable IPC boundary is the supported contract proof.
- Deliberately retain nested snapshot cap fields and their historical warning tests because those values are persisted compatibility data, not this input contract.

**Patterns to verify:**

- `src/lib/tauri-client.ts` remains the sole invoke wrapper.
- `useLoadData` retains its invocation-time save context and stale-progress filtering.
- `SnapshotSummaryDto`, frontend `SnapshotSummary`, and local `LoadDataSnapshotSummary` retain nested persisted snapshot fields; Commit 8 removes only temporary top-level result fields.

**Constraints and non-goals:**

- No UI control is removed here; it was already absent. Do not alter current snapshot selection, progress phases, boost/context gates, or nested historical metadata.

**Dependencies and sequencing:**

- Requires Commits 1 and 2. Commit 4 follows to remove the Rust internal property rather than leaving dead request plumbing.

**Validation:** `./scripts/dev test src/features/memory-read/api/load-data.test.ts src/features/memory-read/hooks/use-load-data.test.tsx src/app/app-top-bar.test.tsx`; `./scripts/dev check-rust`; `./scripts/dev check`.

**Stop conditions:** Stop and replan if Tauri requires a protocol-version change, if a changed test proves a supported non-cap contract cannot be retained, or if the app can still emit a cap property through another production caller.

**Review mandate:** Verify no frontend or public Tauri `load_data` input accepts or sends `maxAccepted`; captured context and progress channel remain exact; positive full-load/context/progress/rollback tests remain meaningful; nested persisted snapshot metadata remains untouched; architecture changes describe only completed behavior.

#### Commit 4 — Omit cap from dump requests

**Status:** Completed

**Provisional commit:** `refactor(memory-read): omit cap from dump requests`

**Work:** Remove Rust's full-dump request cap field and internal limit plumbing. A new app now writes a protocol-v1 full-dump request without `maxAccepted`; an old bridge treats that omission as unlimited.

**Size assessment:** Estimated under 150 changed non-test implementation lines. Within the soft target; the request DTO, request writer, scan adapter, and their tests are one file-protocol outcome.

**Out of scope:**

- C# bridge acceptance and scanner removal, transient status/result fields, persisted dump/snapshot metadata, and bridge documentation that remains true until bridge behavior changes.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/memory_read/service.rs::{BridgeRequest, request_player_dump, request_player_dump_with_limit}` — remove `max_accepted`, collapse the full-dump writer to one unconditional request path, and retain request guard, request ID, player database scope, atomic file replacement, wait, and dump validation behavior.
- `src-tauri/src/features/snapshot/load_data.rs::{scan_dump_from_local_app_data, scan_dump_from_bridge}` — remove the optional limit from scan adapters and call the one request function.
- `src-tauri/src/features/snapshot/commands.rs::execute_load_data_with` — remove the internal limit argument and make its scan closure parameterless while preserving phase order, context revalidation, and publish flow.
- Rust tests in `src-tauri/src/features/memory_read/service.rs`, `src-tauri/src/features/snapshot/load_data.rs`, and `src-tauri/src/features/snapshot/commands.rs` — replace capped-request and non-positive-limit tests with one request-shape/full-load proof that omits the property; retain ready/failed/replaced-dump, ingest, progress, context, and rollback tests.
- `bridge/Tests/RequestProtocolTests.cs::Accept_omitted_max_accepted_is_unlimited` — deliberately retain through this commit as the old-bridge compatibility proof. Commit 5 replaces it with the old-app/new-bridge extra-property proof.

**Behavior and data flow:**

- Tauri's public command from Commit 3 runs the parameterless scan adapter. Rust writes a full-dump request with protocol version, request ID, timestamp, operation, and player database scope only. The old bridge deserializes omitted `maxAccepted` as `null` and performs its established unlimited scan.

**Ordered implementation steps:**

1. Identify the request DTO field, optional limit helper, scanner closure argument, and cap-only tests.
2. Remove the field and collapse callers to the unconditional request function without changing request locking, atomic write, terminal-status matching, or dump capture.
3. Rewrite the request test to parse a real new request and prove it omits `maxAccepted` while preserving required protocol-v1 fields.
4. Run focused Rust tests that exercise request writing and scan-to-ingest orchestration, then the commit gate.

**Tests and proof:**

- Observable behavior: a new Rust full-dump request is valid protocol-v1 input without `maxAccepted`, receives a ready response, and ingests a valid dump; the retained old C# bridge acceptance test proves that omission still deserializes as unlimited.
- Delete `request_player_dump_with_limit`, positive-cap serialization, and non-positive-cap rejection tests with the retired Rust contract. Retain the old-bridge omission proof, full-load, request guard, status matching, replacement, failure, progress, context, and rollback coverage.
- Do not add a rejection/absence test. The parsed request shape and successful full-load prove the supported omitted-property contract.

**Patterns to verify:**

- `write_player_dump_request` retains temp-file then rename behavior.
- `execute_load_data_with` retains its lock-free scan and context revalidation shape.

**Constraints and non-goals:**

- Do not remove `scanTruncated` or `maxAccepted` from dump validation, snapshot ingest, or nested snapshot DTOs. C# still emits compatible fields until Commit 6.

**Dependencies and sequencing:**

- Requires Commit 3. Commit 5 makes the new bridge ignore an old client's extra property.

**Validation:** `./scripts/dev bridge-test`; `./scripts/dev check-rust`; `./scripts/dev check`; focused Rust tests exercised through the Rust gate for `memory_read::service`, `snapshot::load_data`, and `snapshot::commands`.

**Stop conditions:** Stop and replan if omitting the property fails with an old bridge, if the request shape cannot remain protocol v1, or if a removed helper is needed by a non-cap operation.

**Review mandate:** Verify new full-dump JSON has no `maxAccepted`; request locking, atomic replacement, status matching, dump capture, context revalidation, and progress timing remain intact; cap-only tests are removed rather than weakened; persisted historical metadata is untouched.

#### Commit 5 — Stop accepting capped dump requests

**Status:** Completed

**Provisional commit:** `refactor(bridge): stop accepting capped dump requests`

**Work:** Remove the C# bridge's active full-dump cap contract. The bridge ignores an old app's unknown `maxAccepted` property, starts an unparameterized full scan, and keeps all non-cap request validation and bridge safety behavior.

**Size assessment:** Estimated under 180 changed non-test implementation lines. Within the soft target; request acceptance, Plugin dispatch, and pipeline entry are one external bridge behavior boundary.

**Out of scope:**

- Deleting internal scanner cap mechanics, transient status/result reporting, persisted dump fields, snapshot persistence, schema changes, and release DLL output.

**Implementation packet:**

**Files and responsibilities:**

- `bridge/Protocol/BridgeRequest.cs` — remove the modeled full-dump cap property so System.Text.Json treats an old app's extra JSON property as unknown and ignores it.
- `bridge/Protocol/RequestAcceptance.cs` — remove full-dump cap validation and copy logic while retaining protocol, freshness, operation, scope, and boost-field validation.
- `bridge/Plugin.cs::{TryStartBridgeWorkFromRequestOrForceFlag, RunDumpScan}` — remove cap forwarding and start the pipeline with its unparameterized full-scan entry. Keep work-gate, force-scan, scope validation, live indexes, error behavior, and other status fields.
- `bridge/Scanning/CapADumpPipeline.cs::Run` — remove its request cap parameter and call the existing scanner's temporary unlimited default. Commit 6 owns deletion of the scanner's internal cap-only mechanics.
- `bridge/Tests/RequestProtocolTests.cs` — delete cap rejection/preservation tests and add the single positive old-app/new-bridge compatibility proof: a full-dump JSON request with an extra positive `maxAccepted` property is accepted, has ordinary full-dump scope, and routes to the unparameterized unlimited entry.
- `bridge/Tests/CapADumpTests.cs` — rewrite the pipeline cap outcome test as a positive unparameterized full-scan proof over more candidates than the old cap fixture; remove only cap-owned setup.
- `bridge/README.md` — remove active capped request/scan instructions and describe new full-dump requests as unconditional unlimited scans. Keep historical dump-warning information for Commit 6's schema documentation.

**Behavior and data flow:**

- An old app may serialize `maxAccepted`; deserialization ignores it because the new request DTO has no member. `RequestAcceptance` accepts the otherwise valid full-dump request, `Plugin` dispatches `RunDumpScan` without a cap, and the pipeline invokes its full-scan entry. The bridge remains protocol v1 and continues to validate scope and closed boost operations.

**Ordered implementation steps:**

1. Add the positive compatibility proof using an old-shaped JSON request with a cap property and an unparameterized full-scan fixture that returns every candidate.
2. Remove the BridgeRequest member, cap validation, Plugin forwarding, and pipeline input in one buildable change.
3. Delete/rewrite cap-specific acceptance and pipeline tests with their owner; retain scope, TTL, operation, force-scan, full-scan, and failure coverage.
4. Update bridge request documentation to match the active bridge entry behavior.
5. Run bridge tests and the full commit gate.

**Tests and proof:**

- Observable behavior: an otherwise valid old-app request containing `maxAccepted` is accepted and its new-bridge path scans fully, not to the old cap.
- This is the one allowed compatibility boundary proof. It must assert positive acceptance plus the unparameterized full-scan outcome; it must not be a source-absence test.
- Retain positive full-scan tests and all unrelated protocol, scope, lifecycle, output-safety, and rollback tests. Commit 6 owns remaining scanner cap tests.

**Patterns to verify:**

- `RequestAcceptance` keeps its malformed/stale request deletion contract.
- `Plugin` preserves `WorkGate`, background scan dispatch, module/status handling, and live candidate-index behavior.
- `tauri.release.conf.json` continues to package a source-built release DLL; do not touch the placeholder resource DLL.

**Constraints and non-goals:**

- Do not change protocol version, player database scope, boost request validation, dump schema, dump metadata fields, or scanner safety and cancellation.

**Dependencies and sequencing:**

- Requires Commit 4 for new-app request omission. Commit 6 follows to delete now-unreachable scanner cap implementation.

**Validation:** `./scripts/dev bridge-test`; `./scripts/dev check`.

**Stop conditions:** Stop and replan if the serializer is configured to reject unknown properties, if old-property input does not reach the unlimited path, if scope/boost validation regresses, or if the change requires a protocol-version bump.

**Review mandate:** Verify old-app cap JSON is tolerated and processed as unlimited; no C# production request/Plugin/pipeline entry accepts cap behavior; TTL, scope, force-scan, work-gate, status error handling, and boost fields remain intact; no generated DLL changes; README states only true bridge behavior.

#### Commit 6 — Delete scanner cap mechanics

**Status:** Completed

**Provisional commit:** `refactor(bridge): delete scanner cap mechanics`

**Work:** Delete the C# scanner's accepted-player cap, early-stop state, serial-only cap branch, and cap-only diagnostics. New dumps always carry compatible fixed metadata. Keep `CapADumpResult.ScanTruncated` and `MaxAccepted` through this commit with fixed `false`/`null` values because Plugin still compiles against them; Commit 7 retires that transient result contract.

**Size assessment:** Estimated 200 to 260 changed non-test implementation lines. This may exceed the soft target because scanner signatures, diagnostics, pipeline construction, diagnostics formatting, result construction, tests, and schema documentation must remain coherent and compile together.

**Out of scope:**

- Removing `CapADumpResult` transient fields, Plugin ready-status/log consumers, status/report DTOs, Rust/React result DTOs, snapshot ingestion/persistence, database migration, and unrelated scanner optimizations. Commit 7 owns the C# producer removals; Commit 8 owns Rust/React consumers.

**Implementation packet:**

**Files and responsibilities:**

- `bridge/Scanning/PersonScanner.cs` — remove `DefaultMaxAccepted`, optional cap parameters, cap validation, early-stop branches, and the cap-dependent serial/parallel selection. Preserve cancellation, candidate dedupe, scope filtering, club discovery, read-quality behavior, and ordinary parallel scan policy.
- `bridge/Scanning/ScanDiagnostics.cs` and `bridge/Scanning/PersonScanResult.cs` — remove cap-only diagnostics and scanner result state; preserve cancellation and read-quality information.
- `bridge/Scanning/CapADumpPipeline.cs` — remove scanner cap plumbing, write `DumpDocument.ScanTruncated = false` and `MaxAccepted = null` for every new dump, and retain `CapADumpResult.ScanTruncated`/`MaxAccepted` with the same fixed values so existing Plugin status and logging consumers compile until Commit 7.
- `bridge/Output/DumpWriter.cs` — retain persisted JSON fields and output safety, but remove diagnostics-text formatting of the deleted `ScanDiagnostics.MaxAccepted` and `StoppedEarly` fields.
- `bridge/Models/DumpDocument.cs` — retain compatible persisted fields; adjust only comments needed to distinguish fixed new values from readable historical data.
- `bridge/Tests/{CapADumpTests,ParallelScannerTests,DumpWriterStreamingTests}.cs` — delete/rewrite scanner cap, serial-cap, and diagnostics-text assertions; retain positive full-scan, ordinary parallel, cancellation, output, read-quality, and prior-output tests. Add one bounded fake-reader scanner or pipeline proof with 501 valid candidates, the smallest clear count above the former 500-player boundary: every candidate must emit and the persisted new dump must contain `scanTruncated: false` and `maxAccepted: null`.
- `bridge/DUMP_SCHEMA.md` — retain field definitions for historical and persisted compatibility, state that capped historical dumps may contain positive metadata, and state that every new post-removal dump writes `false` and `null`.
- `.wiki/ARCHITECTURE.md` — replace the stale capped-reader serial-worker statement with the implemented normal full-scan worker policy. Do not describe Commit 7/8 host-result changes before they land.

**Behavior and data flow:**

- The pipeline always asks `PersonScanner` for the full candidate set. The scanner uses its normal parallel policy without a cap. Pipeline output fixes both persisted dump values and still-populated transient `CapADumpResult` values to false/null. Plugin therefore remains buildable and continues its existing ready status/log behavior until Commit 7 removes those transient consumers. Rust can ingest old capped dumps and new unlimited dumps through the unchanged schema-v8 parser.

**Ordered implementation steps:**

1. Identify cap-only scanner constants, parameters, early-stop state, serial-only condition, `ScanDiagnostics` fields, `PersonScanResult` state, diagnostics text, and tests.
2. Remove scanner mechanics and cap diagnostics while preserving candidate acceptance, cancellation, read quality, scope filtering, clubs, and ordinary parallel behavior.
3. Remove the obsolete `DumpWriter` diagnostics-text references, set persisted dump fields and retained `CapADumpResult` fields explicitly to false/null, and confirm Plugin still compiles against the latter.
4. Add the bounded 501-valid-candidate fake-reader proof, asserting every candidate emits and persisted new-dump metadata is false/null; then delete cap-specific scanner tests and helpers and run full-scan, parallel, cancellation, output, and prior-output tests.
5. Reconcile schema documentation without changing its version or historical-field contract, and update only the scanner/worker-policy architecture statement now made true.

**Tests and proof:**

- Observable behavior: a bounded fake-reader scanner or pipeline run with 501 valid candidates emits all 501, writes `scanTruncated: false` and `maxAccepted: null` to the persisted new dump, and returns fixed false/null transient result values for the still-compiled Plugin seam. This positive complete-scan proof detects accidental retention of the former 500-player cap without a live FM or manual test.
- Delete tests whose only contract is cap reach, exact-cap truncation, cap-induced serial ordering, capped diagnostics, or diagnostics text for retired fields. Retain full-scan, ordinary parallel, cancellation, output safety, and historical dump/snapshot validation/presentation tests in their owning layers.
- Do not add source-absence tests. The full-scan, serialized new-dump, and fixed-result outcomes prove the replacement contract.

**Patterns to verify:**

- `PersonScanner.ScanParallel` remains the ordinary full-scan parallel implementation.
- The 501-candidate fixture uses the smallest clear count above the former boundary and remains a bounded fake-reader test, not a live-FM performance or attach probe.
- `DumpWriter` continues temp-file streaming and replace-only-on-success output safety after removing only cap diagnostics formatting.
- `Plugin::RunDumpScan` still compiles against fixed `CapADumpResult` fields; Commit 7 is the sole owner of their deletion.
- `.wiki/ARCHITECTURE.md` scanner wording follows the finished worker policy, not the later host-result cleanup.
- Rust `dump_validation` and snapshot ingest remain unchanged consumers of the persisted JSON fields.

**Constraints and non-goals:**

- Do not remove `scanTruncated` or `maxAccepted` from `DumpDocument`, JSON serialization, schema v8 validation, SQLite, or snapshot DTOs. Do not alter Plugin transient consumers in this commit. Do not change cancellation, memory safety, read-quality retry, or output replacement.

**Dependencies and sequencing:**

- Requires Commit 5, which makes cap mechanics unreachable from production requests. Commit 7 removes retained C# transient producers after this commit fixes their values. Commit 8 then removes compatible Rust/React consumers.

**Validation:** `./scripts/dev bridge-test`; `./scripts/dev check`; focused Rust dump-validation coverage through the Rust gate.

**Stop conditions:** Stop and replan if deleting scanner fields leaves a `DumpWriter` diagnostics formatter or other compile consumer, if Plugin cannot retain fixed transient values until Commit 7, if full scans alter cancellation/read-quality/output safety, or if historical readability requires a schema/migration change.

**Review mandate:** Verify the bounded 501-candidate fake-reader proof emits every candidate and serializes persisted false/null metadata; no C# scanner path can stop for a player cap; no deleted `ScanDiagnostics` field has a diagnostics-text consumer; Plugin remains buildable with fixed `CapADumpResult` fields; full-scan parallel and cancellation semantics remain sound; `.wiki/ARCHITECTURE.md` states the true worker policy only; historical field parsing remains supported; cap-only tests/helpers leave with their contract; no scanner optimization, live-FM test, or generated binary change is smuggled in.

#### Commit 7 — Remove bridge transient cap producers

**Status:** Active

**Provisional commit:** `refactor(bridge): remove transient cap producers`

**Work:** Remove the C# bridge's transient cap producers: `CapADumpResult` fields, Plugin ready-status/log consumers, and `BridgeStatus`/`StatusWriter` output. Leave Rust and React consumers for Commit 8. The old host remains compatible because all removed ready-status properties are optional.

**Size assessment:** Estimated under 180 changed non-test implementation lines. Within the soft target; the C# result, Plugin, status writer, status tests, and bridge documentation are one producer contract.

**Out of scope:**

- Rust/React status or result consumers, Load Data result DTOs, LoadDataOutcome, snapshot UI, persisted dump/snapshot fields, scanner mechanics, protocol version, migration, and architecture result-contract documentation. Commit 8 owns host consumers and that documentation.

**Implementation packet:**

**Files and responsibilities:**

- `bridge/Scanning/CapADumpPipeline.cs::CapADumpResult` — remove its transient `ScanTruncated` and `MaxAccepted` fields now that Commit 6 fixes persisted dump fields directly.
- `bridge/Plugin.cs::{RunDumpScan, WriteStatus}` — remove cap-specific ready-status arguments and log suffix while preserving request ID, players found, errors, module presence, live indexes, boost support/results, and work-gate behavior.
- `bridge/Protocol/BridgeStatus.cs` and `bridge/Output/StatusWriter.cs` — remove only transient cap status fields and their sanitization copies. Preserve all non-cap status and boost fields.
- `bridge/Tests/BridgeStatusSerializationTests.cs` and relevant pipeline/Plugin-facing tests — remove cap producer assertions and retain status round-trip, error sanitization, ready status, full-scan, and boost-status coverage.
- `bridge/README.md` and `bridge/DUMP_SCHEMA.md` — describe status as no longer carrying cap signals while preserving dump and snapshot historical metadata documentation.

**Behavior and data flow:**

- After Commit 6, new dumps already carry false/null persisted metadata. This commit stops C# from emitting parallel ready-status/result metadata. An old Rust host deserializes missing optional fields as `None`, keeps request-ID terminal matching, and continues ingesting the dump; no host source must change in this commit.

**Ordered implementation steps:**

1. Add or adjust the bridge status proof for a ready status without cap properties while retaining request ID and all supported status/boost fields.
2. Remove `CapADumpResult` cap fields, Plugin status/log consumers, and C# status writer fields in one buildable producer change.
3. Delete only producer-owned cap status tests and update bridge documentation.
4. Run bridge tests and the commit gate.

**Tests and proof:**

- Observable behavior: a new bridge ready status has request identity and supported status/boost data but no cap properties; its full dump remains available for the existing host.
- Retain positive full-scan, ready-status, error-sanitization, and boost-status tests. Delete cap-producer assertions with their C# owner.
- Do not add source-absence padding. The serialized ready-status contract is the observable proof.

**Patterns to verify:**

- `StatusWriter.SanitizeError` keeps every supported non-cap field.
- Existing Rust `BridgeStatus` optional fields make missing producer properties compatible until Commit 8 removes host fields.

**Constraints and non-goals:**

- Do not change persisted dump metadata, Rust/React consumers, `LoadDataOutcome`, Snapshot Overview, Snapshot History, scanner behavior, or protocol version.

**Dependencies and sequencing:**

- Requires Commit 6. Commit 8 follows and removes host consumers while accepting old bridge statuses with extra optional cap properties.

**Validation:** `./scripts/dev bridge-test`; `./scripts/dev check`.

**Stop conditions:** Stop and replan if missing C# status properties break the current Rust host, if status sanitization loses a supported field, or if removal needs a protocol-version change.

**Review mandate:** Verify every `CapADumpResult` cap field and Plugin status/log consumer leaves in this commit; ready status remains usable by the old host; non-cap module, request, error, and boost fields survive serialization and sanitization; persisted dump metadata and host/UI code remain untouched.

#### Commit 8 — Remove host transient cap consumers

**Status:** Pending

**Provisional commit:** `refactor(load-data): remove transient cap consumers`

**Work:** Remove Rust and React consumption of transient cap status/result fields and delete the unreachable capped-new-load LoadDataOutcome branch. Retain memory-read-local `LoadDataSnapshotSummary` with nested persisted cap fields, and retain the Snapshot Overview warning for a capped retained current snapshot.

**Size assessment:** Estimated 170 to 240 changed non-test implementation lines. This may exceed the soft target because Rust result DTOs, frontend boundary types/mocks, ordinary result behavior, historical compatibility, and intrinsic architecture documentation must remain coherent together.

**Out of scope:**

- C# producer code, persisted dump/snapshot fields, local LoadDataSnapshotSummary extraction or cross-feature imports, Snapshot History cap UI, snapshot ordering/timestamps, request input, scanner mechanics, protocol version, migration, or new UI behavior.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/memory_read/service.rs::{BridgeStatus, DumpRequestResult, parse_bridge_status, wait_for_request_terminal}` — remove cap status fields and propagation. Add or modify a compatibility proof that a new Rust host accepts an old ready status containing extra cap properties and preserves its request ID for terminal matching.
- `src-tauri/src/features/snapshot/load_data.rs::{LoadDataResult, publish_prepared_with_progress, load_data_after_scan_with_context}` and `src-tauri/src/features/snapshot/commands.rs::{LoadDataResultDto, LoadDataResultDto::from}` — remove only top-level transient result fields. Keep nested persisted `SnapshotSummaryDto.scan_truncated`/`max_accepted` unchanged.
- `src/features/memory-read/types/{bridge-status,load-data}.ts`, `src/testing/snapshot-ipc-mock.ts`, `src/features/memory-read/api/bridge-status-ipc-mock.ts`, `e2e/tauri-ipc-stub.ts`, and relevant Load Data API/hook tests — remove only top-level transient status/result types, `truncatedSuccess` new-load fixtures, cap-only mocks, and stub fields. Retain nested stored/effective snapshot cap fields and retain `LoadDataSnapshotSummary` as the local memory-read response projection; do not import `SnapshotSummary` from the snapshot feature or introduce a shared type.
- `src/features/memory-read/components/{load-data-outcome,load-data-outcome.test}.tsx` — delete the capped-success branch and cap-only tests for the just-stored new snapshot; retain ordinary success, stored-versus-effective history message, progress, error, and timing coverage.
- `src/features/snapshot/components/{snapshot-overview-panel,snapshot-panels.test}.tsx` — retain persisted cap fields and add or modify the focused positive proof that a retained capped current snapshot shows the incomplete warning in Snapshot Overview.
- `src/features/snapshot/components/snapshot-history-panel.tsx` — no change. It retains `loadedAtUtc` provenance only; do not add cap fields or a warning UI.
- `.wiki/ARCHITECTURE.md` — remove stale top-level `LoadDataResult.scanTruncated`/`maxAccepted` current-state claims and state the implemented host result contract without them. Do not rewrite completed records.

**Behavior and data flow:**

- A new Rust host ignores extra cap properties from an old bridge status, retains request-ID terminal matching, and reads all historical cap evidence only from `dump.json`. Rust persists that evidence into nested snapshot summaries. The frontend keeps its local `LoadDataSnapshotSummary` boundary shape for stored/effective snapshots, including nested cap fields, but removes top-level transient fields. A just-stored new snapshot is always unlimited, so LoadDataOutcome has only ordinary outcomes. Snapshot Overview reads a retained current snapshot's persisted fields for its sole cap warning; Snapshot History remains cap-free.

**Ordered implementation steps:**

1. Separate persisted dump/snapshot fields and the local LoadDataSnapshotSummary response projection from transient Rust/React status and top-level result fields.
2. Remove Rust consumer fields and add the old-ready-status compatibility proof that preserves request-ID terminal matching.
3. Remove frontend transient types and capped-new-load mocks, update `e2e/tauri-ipc-stub.ts` to remove only top-level Load Data result cap fields while retaining nested stored/effective snapshot fields, then delete LoadDataOutcome capped-success UI/tests without weakening ordinary result, history message, progress, error, timing, context, or rollback proofs.
4. Prove the retained capped-current Snapshot Overview warning and leave Snapshot History metadata and UI cap-free.
5. Reconcile the top-level host-result architecture statement and run focused tests and project gates.

**Tests and proof:**

- Observable behavior: a retained capped current snapshot shows Snapshot Overview's incomplete warning from persisted nested metadata; a new Load Data success uses the ordinary branch and has no capped-success fixture or top-level cap report.
- Compatibility proof: a new Rust host accepts an old ready `status.json` with extra cap properties and still preserves the matching request ID through terminal-status handling.
- Delete/rewrite transient host status/result tests, capped-new-load mocks, LoadDataOutcome cap tests, and only top-level result cap fields in the browser IPC stub. Retain nested stored/effective snapshot fields, historical capped dump validation, SQLite snapshot persistence, Snapshot Overview warning, Snapshot History loaded-at display, and unrelated stored-versus-effective history, progress, error, timing, context, and rollback tests.
- `./scripts/dev smoke` exercises the updated browser IPC stub and is the positive surviving-contract proof for the browser path.
- Do not add source-absence padding. The old-status compatibility and retained-current Snapshot Overview behavior are positive supported-contract proofs.

**Patterns to verify:**

- `wait_for_request_terminal` keeps request-ID terminal matching and dump-presence validation when old status JSON contains ignored cap properties.
- `LoadDataSnapshotSummary` stays local to memory-read; Biome's no-cross-feature import rule remains satisfied.
- LoadDataOutcome has only ordinary new-load outcomes; SnapshotOverviewPanel is the sole retained cap-warning surface.
- SnapshotHistoryPanel continues loaded-at presentation without cap fields or incomplete-warning UI.

**Constraints and non-goals:**

- Do not remove nested `scanTruncated`/`maxAccepted` from DumpDocument, dump parsing, migrations, database columns, SnapshotSummaryDto, frontend SnapshotSummary, memory-read-local LoadDataSnapshotSummary, or Snapshot Overview. Do not add cap fields or warnings to Snapshot History. Do not extract or import a shared snapshot type. Do not remove loadedAtUtc, provenance, boost, progress, context, or rollback fields.

**Dependencies and sequencing:**

- Requires Commit 7. This is the final implementation commit; after it clears review, the feature status becomes Validation.

**Validation:** `./scripts/dev test src/features/memory-read/components/load-data-outcome.test.tsx src/features/memory-read/hooks/use-load-data.test.tsx src/features/snapshot/components/snapshot-panels.test.tsx`; `./scripts/dev check-rust`; `./scripts/dev check`; `./scripts/dev smoke`.

**Stop conditions:** Stop and replan if an old ready status fails to preserve terminal request-ID matching, a local type requires a forbidden cross-feature import to remain correct, a retained capped current snapshot no longer warns in Snapshot Overview, Snapshot History requires cap UI to satisfy a documented contract, or a schema/migration change becomes necessary.

**Review mandate:** Verify Rust/React host consumers, browser IPC stub top-level fields, and capped-new-load UI leave together; the stub retains nested stored/effective snapshot fields; old ready-status compatibility and request matching work; LoadDataSnapshotSummary remains a local memory-read projection with nested persisted fields; Snapshot Overview retains the only cap warning; Snapshot History remains cap-free and retains loaded-at provenance; smoke proves the browser surviving contract; Architecture removes only now-false top-level result claims; ordinary success/history message/progress/error/timing/context/rollback behavior remains intact; no schema, migration, or cross-feature import is introduced.

## Active work

**PR:** PR 1 — refactor(load-data): retire obsolete control support

**Commit:** Commit 7 — Remove bridge transient cap producers

### RED or removal proof

Identify the `CapADumpResult`, Plugin ready-status/log, `BridgeStatus`, and `StatusWriter` cap producers and their producer-owned serialization tests. Preserve request identity, supported status and boost fields, error sanitization, and full-dump availability.

### Expected outcome

A new bridge ready status contains request identity and all supported status and boost data but no transient cap properties. The existing Rust host remains compatible with omitted optional fields.

### Explicit exclusions

Rust/React consumers, persisted dump and snapshot metadata, scanner behavior, LoadDataOutcome, snapshot UI, protocol version, migrations, and architecture result-contract documentation.

## Discoveries and replanning

- Planning discovery: JAY-56 has no remaining production freshness evaluator. PR #120 removed the chip in `d50ff0c`; this plan retains `loadedAtUtc` because current source uses it for Snapshot Overview, History, and equal-date ordering.
- Planning correction round 2: current cap metadata divides into removable bridge producers, removable host consumers, and retained dump/snapshot compatibility fields. Commit 7 removes only optional C# producers; Commit 8 removes compatible Rust/React consumers and unreachable capped-new-load UI. `LoadDataSnapshotSummary` remains a memory-read-local projection with nested persisted cap fields.
- Planning discovery: C# `BridgeRequest` can drop its cap member without a protocol bump because the current serializer default ignores an old app's unknown JSON property. Commit 5 requires a positive compatibility proof.
- The reviewed planning artifacts were committed as `1d648730ff4132b5fbe95801a88a2f33acdb8905`. Record future material deviations, blockers, and changed assumptions here before continuing delivery.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — refactor(load-data): retire obsolete control support | Commit 1 — Record the approved cleanup plan | 1d648730ff4132b5fbe95801a88a2f33acdb8905 | Recorded the reviewed schema 2 ledger and TODO Active entry without changing executable behavior. | `ledger_state.py`: runnable; `delivery_state.py`: runnable with the accepted fingerprint; `git diff --cached --check` and pre-commit `check-fast`: passed. | Not applicable | Clear | 0 | The unrelated completed-ledger formatting edit remained unstaged. |
| PR 1 — refactor(load-data): retire obsolete control support | Commit 2 — Reconcile retired freshness documentation | 0bdbfe8794ea619d167de040809ae7e03fdc8d7c | Removed stale snapshot-freshness chip claims while retaining load-timestamp persistence, presentation, and equal-game-date ordering documentation. | `ledger_state.py`: runnable; `delivery_state.py`: runnable with the accepted fingerprint; `git diff --check` and pre-commit `check-fast`: passed. | Not applicable | Clear | 0 | None. |
| PR 1 — refactor(load-data): retire obsolete control support | Commit 3 — Remove cap from app IPC | dd0d445fa5b258076bf051405b5776f95ecd57df | Removed the cap argument from AppTopBar, the React Load Data API and mutation, and the public Tauri command while preserving context-bound progress and the temporary unlimited Rust scan seam. | Focused frontend tests passed 44 tests; `./scripts/dev check-rust` passed 809 tests with 2 ignored; `./scripts/dev check`, `git diff --check`, and primary LSP diagnostics passed. | Pass | Clear | 0 | The first commit attempt timed out during a passing pre-commit Rust run; the unchanged staged diff passed the rerun and committed normally. |
| PR 1 — refactor(load-data): retire obsolete control support | Commit 4 — Omit cap from dump requests | 3ecc586597d9d92337e3e8bae59d1fe7141d80e9 | Removed the cap field and limit helper from Rust request serialization and collapsed Load Data orchestration to one parameterless full-dump path. | `./scripts/dev bridge-test` passed 224 tests with 3 skipped; `./scripts/dev check-rust` passed 806 tests with 2 ignored; `./scripts/dev check`, `git diff --check`, and primary LSP diagnostics passed. | Pass | Clear | 0 | None. |
| PR 1 — refactor(load-data): retire obsolete control support | Commit 5 — Stop accepting capped dump requests | d5018ddbc88604498bdfb6006ab24ff6151d1022 | Removed active bridge cap request behavior and routed accepted old-shaped requests through a cap-free production dispatch seam to a full scan. | `./scripts/dev bridge-test` passed 217 tests with 3 skipped; `./scripts/dev check` passed with 806 Rust tests and 2 ignored; `git diff --check` and primary C# diagnostics passed. | Pass | Clear | 1 | Initial review found that the compatibility test did not bind accepted input to production dispatch; correction added the narrow shared dispatch seam and passed focused correction review. |
| PR 1 — refactor(load-data): retire obsolete control support | Commit 6 — Delete scanner cap mechanics | c78d283dc3e0ddd96ddba07e686f10400b549a77 | Deleted scanner cap, early-stop, serial-cap, and diagnostics mechanics; fixed new-dump and temporary result metadata; and proved 501 candidates scan completely. | `./scripts/dev bridge-test` passed 216 tests with 3 skipped; `./scripts/dev check` passed with 806 Rust tests and 2 ignored; `git diff --check` and primary C# diagnostics passed. | Pass | Clear | 0 | None. |

## Final validation

Run after all planned implementation commits and before feature review:

1. `./scripts/dev test src/features/memory-read/api/load-data.test.ts src/features/memory-read/hooks/use-load-data.test.tsx src/features/memory-read/components/load-data-outcome.test.tsx src/app/app-top-bar.test.tsx src/features/snapshot/components/snapshot-panels.test.tsx` — focused frontend IPC omission, ordinary new-load outcome, stored-versus-effective history message, progress/context behavior, retained-current Snapshot Overview warning, and loaded-at presentation coverage.
2. `./scripts/dev bridge-test` — C# request compatibility, full scan, dump metadata, status, and output behavior. Expected current baseline before delivery: 224 passed and 3 skipped with .NET 6.0.428.
3. `./scripts/dev check-rust` — focused Rust-owned contract coverage through the Rust gate, including request writing, orchestration, dump validation, snapshot ordering, and persistence.
4. `./scripts/dev check` — full commit gate: Biome, TypeScript, secretlint, Rust format, Clippy, and Rust tests.
5. `./scripts/dev smoke` — automated product smoke after the frontend IPC contract change.

No manual FM attach/app test, `inspect-ui`, generated-binary update, package build, or release evidence is required. A native/manual run would not add material proof for this deletion-only feature; report any unavailable automated command as a validation gap rather than a pass.

## Documentation impact

Complete during reconciliation. Commit 2 corrects the already-retired freshness claim. Commit 3 reconciles app IPC documentation. Commit 5 reconciles active bridge request behavior. Commit 6 reconciles new-dump versus historical-field schema documentation and scanner worker policy. Commit 7 reconciles bridge transient status producers. Commit 8 reconciles the host top-level result contract. Feature close-out must verify that no stale Load Data control claim remains and must not rewrite completed feature records.
