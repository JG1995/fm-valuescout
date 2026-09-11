# Bridge scan throughput

## Status

Ready for final publication

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 3d5eb4fd0300bda1b0a663958db9f1e161ae2402ce88f0297d5ae2b38a61bd09

## Intent

Deliver the smallest low-effort, high-effect bridge subset of Linear JAY-65 on `main` at `721aa3321e5ce11cf447e7e4468615661681df46`, without changing the bridge protocol, dump schema, or scan architecture.

## User-visible behavior

- Load Data can receive a bridge request within a fixed 250 ms poll interval instead of two seconds.
- Full scans retain the current schema-v9 output, cancellation behavior, atomic dump replacement, and prior-dump preservation.
- Scalar process-memory reads retain their full, short, and failed-read behavior while avoiding one managed byte-array allocation per call.

## Invariants

- `WindowsMemoryReader.TryRead` reports the native byte count, copies bytes read into the caller span, and returns true only for a complete successful read.
- `DumpWriter` keeps per-record cancellation checks, bounded streaming output, valid schema-v9 JSON, temp-file replacement only after complete success, and prior-dump preservation on cancellation or failure.
- The request loop remains cancellation-aware and has one fixed, unconfigured poll interval.
- No speedup is claimed until equivalent before-and-after evidence exists.

## Non-goals

- Byte-balanced or dynamic scan scheduling, diagnostic-counter redesign, region-filter narrowing, repeated extraction-read redesign, snapshot retries, live-index materialization, or status redesign; these remain deferred in Linear JAY-69 (<https://linear.app/jaycount/issue/JAY-69/investigate-remaining-fm-bridge-scan-performance>).
- Typed or streaming Rust dump preparation, private dump copies, score representation or parallelism, initial projection insertion redesign, or Club DNA movement; these remain deferred in Linear JAY-68.
- Database schema, size, or index work; frontend, Rust, or protocol changes; dependencies; benchmark frameworks; configuration; generalized tuning; or instrumentation.
- A manual or native test as a pre-merge gate. The developer-owned post-merge Windows FM smoke and timing comparison is a disclosed follow-up only.

## Current-state map

- Relevant components: `bridge/Memory/WindowsMemoryReader.cs` allocates `new byte[destination.Length]` in `TryRead`; its block path already pins caller-owned byte arrays at the `ReadProcessMemory` `IntPtr` boundary. `bridge/Output/DumpWriter.cs` calls `Utf8JsonWriter.Flush()` after every player and staff record. `bridge/Plugin.cs` sets `RequestPollInterval` to two seconds.
- Existing proofs: `bridge/Tests/MemoryReaderTests.cs` has Windows-only current-process block-read coverage and fake-reader full, short, and failed-read coverage. `bridge/Tests/DumpWriterStreamingTests.cs` covers schema-v9 JSON, atomic replacement, cancellation/prior-dump preservation, a 5,000-player write-chunk assertion, and 184,000/500,000-player output counts.
- Existing boundaries: `bridge/FmDataBridge.csproj` enables unsafe code and exposes internals to `FmDataBridge.Tests`; bridge tests use xUnit. `bridge/README.md` documents the current request poll interval and the bridge output contract.
- Validation: `./scripts/dev bridge-test` runs the C# suite. `./scripts/dev check` is the repository gate. Windows CI runs bridge tests for `bridge/**` changes, and the required strict GitHub status aggregates applicable checks.
- Baseline evidence: `bridge/README.md` records the hardened 247,781-player/134,316-staff scan at 23.618 s total, including 11.188 s extraction, 3.110 s dump writing, and 19,837,038 process-memory reads. The repository has no general benchmark framework.

## Feature architecture

Keep all changes inside the existing C# bridge seams: pin the scalar caller span at the existing native read boundary, flush `Utf8JsonWriter` only after a fixed 64 KiB pending-byte threshold, and replace the request-loop constant. No interface, protocol, scheduler, persistence, or ownership boundary changes.

## Uncertainty register

### Known

- PR #147 is on current `main`; this feature starts from `721aa3321e5ce11cf447e7e4468615661681df46`.
- `Utf8JsonWriter.BytesPending` is available at the existing writer seam.
- The existing Windows-only test attribute skips native `ReadProcessMemory` tests off Windows.

### Assumptions

- A fixed 64 KiB pending-byte threshold preserves the existing streaming bound while substantially reducing writes for the 5,000-player tracking-stream proof.

### Decisions

- Use one short-lived PR with four maximally atomic commits. The three executable outcomes have separate behavior and proofs but share one safe review and merge boundary.
- Keep the dump threshold fixed at 64 KiB. It is a bounded direct edit, not a configurable tuning surface.
- Add only the scalar current-process and warmed-allocation proof that the existing Windows-only test seam supports. The direct poll constant needs no literal-restatement test.
- No ADR is warranted. This work preserves the established bridge, file-protocol, and atomic-output boundaries.

### Unknowns

- Independent plan review must confirm the provisional threshold and packets before delivery-state classification supplies the fingerprint.

### Risks

- A scalar pinning change could alter partial-read copying or native success semantics.
- A batching change could accidentally remove intermediate flushing, weaken cancellation responsiveness, or grow output chunks without bound.
- The poll documentation must change with the constant to avoid a false operational statement.

## Walking skeleton

Record the reviewed plan, remove scalar-read allocation with a native Windows proof, batch dump flushes without losing bounded streaming or replacement safety, then shorten the request poll constant and its documented interval.

## Delivery plan

### PR 1 — Improve bridge scan throughput

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/bridge-scan-throughput

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** GitHub required strict status check

**Feature close-out:** Current

**CI repair rounds:** 0

**Provisional PR title:** `perf(bridge): improve scan throughput`

**Purpose:** Deliver three small, independently revertible bridge costs in one trunk-safe PR.

**Depends on:** Current `main` at `721aa3321e5ce11cf447e7e4468615661681df46`, independent plan review, developer acceptance, a valid Delivery fingerprint, and exact branch activation. Ledger state alone does not create or switch the branch.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(bridge): record scan throughput plan`

**Work:** Commit the independently reviewed schema-2 ledger and TODO activation before implementation.

**Atomicity:** The active ledger and its sole TODO pointer jointly establish the reviewed feature state; either artifact alone leaves a missing source of truth or queue entry, and no executable outcome exists to split further.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, BACKLOG, planned specs, ADRs, Git state, Linear, and GitHub state.

**Implementation packet:**

- Preserve the accepted plan-review result. Commit only these two reviewed planning paths after authorized branch activation.

**Files and responsibilities:**

- `.wiki/features/active/bridge-scan-throughput.md` — approved JAY-65 intent, authority, delivery plan, and packets.
- `.wiki/TODO.md` — one active ledger link with Linear JAY-65.
- `.wiki/BACKLOG.md`, `.wiki/features/planned/`, and `.wiki/decisions/` — no change; no accepted scope promotion, planned spec, or ADR applies.

**Behavior and data flow:**

- Record the reviewed plan and active TODO link before executable changes. No runtime behavior changes.

**Ordered implementation steps:**

1. Verify the fingerprint-authorized branch and `main` base without unapproved Git mutation.
2. Confirm that only the reviewed ledger and TODO paths changed.
3. Run the ledger classifier and exact-path diff check.
4. Stage and inspect only the reviewed planning diff for checkpoint review.

**Tests and proof:**

- Not applicable — independently reviewed planning documents only. The ledger classifier proves schema-2 state and `git diff --check` proves a clean planning diff.

**Patterns to verify:**

- `.wiki/features/active/README.md` and `.wiki/TODO.md` ownership rules.

**Constraints and non-goals:**

- Do not change feature scope, packet order, implementation, tests, configuration, BACKLOG, ADRs, branches, or publication state.

**Dependencies and sequencing:**

- Requires a clear independent plan-review verdict, developer acceptance, Delivery fingerprint, and exact branch activation before commit.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/bridge-scan-throughput.md && git diff --check -- .wiki/features/active/bridge-scan-throughput.md .wiki/TODO.md`

**Stop conditions:** Stop on an uncleared review, classifier error, unreviewed path, substantive plan change, missing accepted fingerprint, diff-check error, or branch mismatch.

**Review mandate:** Verify that the staged diff contains only the reviewed two-path planning outcome, has exactly one active PR and Commit 1 active, includes JAY-65 in TODO, and grants no implementation or Git authority.

#### Commit 2 — Pin scalar memory reads

**Status:** Completed

**Provisional commit:** `perf(memory-read): pin scalar read buffers`

**Work:** Read directly into the caller span in `WindowsMemoryReader.TryRead` instead of allocating a managed byte array for each scalar read.

**Atomicity:** The pinned native call and its Windows scalar-read/allocation proof form one complete allocation-removal outcome; separating either would leave the changed memory boundary without its only platform-specific proof.

**Size assessment:** Within the soft target; one native-call-path edit and focused test additions.

**Out of scope:**

- `IMemoryReader` redesign, block-read behavior, memory writes, array pooling, dependencies, benchmark tooling, layout changes, and extraction changes.

**Implementation packet:**

- Replace the scalar temporary array and copy with the existing `ReadProcessMemory` `IntPtr` overload against a pinned caller `Span<byte>`. Preserve the existing empty-span success, native byte-count reporting, copied-prefix behavior, and `ok && bytesRead == destination.Length` result rule.

**Files and responsibilities:**

- `bridge/Memory/WindowsMemoryReader.cs` — pin the scalar destination at the established safe P/Invoke boundary and remove the per-call byte-array allocation/copy.
- `bridge/Tests/MemoryReaderTests.cs` — add the smallest Windows-only current-process scalar-read and warmed allocation proof; retain the existing full, short, and failed fake-reader behavior coverage.

**Behavior and data flow:**

- Scalar readers pass stack or caller-owned spans to `TryRead`; the method pins that storage for `ReadProcessMemory`, reports the native byte count, and leaves the caller span with the native readable prefix. Block reads remain on `TryReadDirect` and do not change.

**Ordered implementation steps:**

1. Add a Windows-only RED proof that reads a current-process scalar buffer through `TryRead` and verifies the value and byte count.
2. Warm the same path, then prove repeated scalar reads allocate no managed bytes on the current thread.
3. Pin the caller span at the existing `IntPtr` P/Invoke boundary and preserve the current return and partial-copy rules.
4. Run the focused bridge tests, then `./scripts/dev bridge-test` followed by `./scripts/dev check`.

**Tests and proof:**

- Modify `MemoryReaderTests.cs`. Prove a current-process scalar read succeeds with exact bytes and value, and a warmed repeated-read loop has no managed allocation. Existing fake-reader tests deliberately retain the supported full, short, and failed outcomes; the new test must fail if `TryRead` restores `new byte[destination.Length]`.

**Patterns to verify:**

- `WindowsMemoryReader.TryReadDirect` for the existing pinned native read, and `MemoryReaderTests.TryReadBlock_on_windows_fills_from_current_process_without_requiring_span_copy` for Windows-only current-process setup.

**Constraints and non-goals:**

- Preserve the `IMemoryReader.TryRead` contract and do not use raw pointer dereference, pool state, an abstraction, or a benchmark framework.

**Dependencies and sequencing:**

- Depends on Commit 1 only and is independently revertible before the dump and polling changes.

**Validation:** `./scripts/dev bridge-test && ./scripts/dev check`

**Stop conditions:** Stop and replan if the span cannot be pinned safely at the existing P/Invoke boundary, the native call cannot preserve full/short/failed semantics, or the warmed proof is not reliable on the existing Windows test seam.

**Review mandate:** Check that `TryRead` has no managed byte-array allocation or copy, pins only for the native call, preserves empty/full/short/failed behavior and byte counts, does not alter `TryReadDirect`, and adds a meaningful Windows-only allocation regression proof.

#### Commit 3 — Batch dump writer flushes

**Status:** Completed

**Provisional commit:** `perf(dump): batch streaming flushes`

**Work:** Replace per-record `Utf8JsonWriter.Flush()` calls with a fixed 64 KiB `BytesPending` flush threshold.

**Atomicity:** The thresholded flush and rewritten tracking-stream proof jointly establish bounded streaming with substantially fewer writes; a code-only change lacks proof, while a test-only change does not alter output behavior.

**Size assessment:** Within the soft target; one writer-loop condition and one focused streaming-test rewrite.

**Out of scope:**

- Removing intermediate flushing, configurable thresholds, output-schema changes, buffering the complete document, serializer replacement, protocol changes, and changes to cancellation or atomic replacement design.

**Implementation packet:**

- Add one private fixed 64 KiB pending-byte threshold. After serializing each player or staff record, flush only when `Utf8JsonWriter.BytesPending` reaches that threshold; retain the final flush and per-record cancellation checks.

**Files and responsibilities:**

- `bridge/Output/DumpWriter.cs` — replace per-record flushes with the fixed pending-byte threshold while retaining final output completion and replacement behavior.
- `bridge/Tests/DumpWriterStreamingTests.cs` — rewrite the 5,000-player tracking-stream assertion to prove materially fewer writes than records, a bounded maximum write chunk, and the exact player count; retain cancellation and large-count tests.

**Behavior and data flow:**

- Players and staff continue to serialize one record at a time. The writer accumulates only up to the fixed pending-byte threshold before flushing to the stream, checks cancellation before every record, then closes valid schema-v9 JSON and flushes the remainder. `TryWriteReplaceOnSuccess` still moves the completed temp file only after that succeeds.

**Ordered implementation steps:**

1. Rewrite the existing tracking-stream test as a RED proof for substantially fewer writes than 5,000 records while retaining its bounded-chunk and exact-count observations.
2. Add the one fixed 64 KiB threshold and flush when `BytesPending` reaches it in both record loops.
3. Retain the final flush, cancellation checks, cancellation/prior-dump test, and 184,000/500,000 count tests.
4. Run the focused bridge tests, then `./scripts/dev bridge-test` followed by `./scripts/dev check`.

**Tests and proof:**

- Modify `DumpWriterStreamingTests.cs`. The changed 5,000-player test must fail for per-record flushing, pass only with substantially fewer writes, keep the existing maximum write bound, and parse exactly 5,000 players. Keep cancellation and large-count tests because they protect unchanged atomic-replacement and scale contracts.

**Patterns to verify:**

- `DumpWriter.TryWriteReplaceOnSuccess` and the existing cancellation/tracking stream tests.

**Constraints and non-goals:**

- Keep output streaming bounded and schema-v9 valid. Do not remove all intermediate flushes, expose a setting, retain an unbounded in-memory document, or claim a dump-writing speedup.

**Dependencies and sequencing:**

- Depends on Commit 1 only and is independently revertible from the scalar-read and polling outcomes.

**Validation:** `./scripts/dev bridge-test && ./scripts/dev check`

**Stop conditions:** Stop and replan if `BytesPending` cannot bound output as the existing API promises, the revised test cannot distinguish per-record flushing from the thresholded behavior, cancellation loses prior-dump safety, or valid schema-v9 output fails.

**Review mandate:** Check the threshold is one private fixed 64 KiB constant used for both player and staff loops, cancellation remains per record, the final flush and atomic temp replacement remain, no configuration or complete-document buffering appears, and the tracking test proves fewer writes, bounded chunks, and exact count.

#### Commit 4 — Shorten bridge request polling

**Status:** Completed

**Provisional commit:** `perf(bridge): reduce request poll interval`

**Work:** Change the fixed request poll interval from two seconds to 250 ms and update its bridge documentation.

**Atomicity:** The constant and the matching documented operational interval are one truthful polling-latency outcome; separating them would leave the repository with a false contract, and the direct constant has no smaller meaningful proof seam.

**Size assessment:** Within the soft target; one constant and one existing documentation statement.

**Out of scope:**

- A scheduler, file watcher, configuration, abstraction, polling-loop redesign, protocol change, diagnostics, or a test that only restates the literal.

**Implementation packet:**

- Replace `TimeSpan.FromSeconds(2)` in `Plugin.RequestPollInterval` with one fixed `TimeSpan.FromMilliseconds(250)` value. Update the matching request-protocol sentence in `bridge/README.md`; make no other poll-loop change.

**Files and responsibilities:**

- `bridge/Plugin.cs` — set the fixed cancellation-aware request-loop interval to 250 ms.
- `bridge/README.md` — state the same fixed 250 ms request poll interval.

**Behavior and data flow:**

- `PollRequests` still tries bridge work and stale-module refresh, then waits on the cancellation handle. It repeats the same sequence at a maximum 250 ms wait between iterations.

**Ordered implementation steps:**

1. Change only the interval constant.
2. Update the matching bridge operational statement.
3. Inspect the existing poll loop to confirm its cancellation-aware wait and all other behavior remain unchanged.
4. Run `./scripts/dev bridge-test` followed by `./scripts/dev check`.

**Tests and proof:**

- No new test. The change has no existing observable seam beyond the direct constant; existing automated bridge tests and gates protect the unchanged bridge surface. Source inspection proves the constant remains the `PollRequests` wait input.

**Patterns to verify:**

- `Plugin.PollRequests` and the request-protocol polling statement in `bridge/README.md`.

**Constraints and non-goals:**

- Keep a single fixed interval with no configurability or new scheduling mechanism.

**Dependencies and sequencing:**

- Depends on Commit 1 only and is independently revertible from both throughput optimizations.

**Validation:** `./scripts/dev bridge-test && ./scripts/dev check`

**Stop conditions:** Stop and replan if changing the interval exposes an existing timing contract, test seam, or lifecycle interaction that requires a scheduler or protocol decision.

**Review mandate:** Check that only the interval and matching documentation changed, the value is exactly 250 ms, `PollRequests` remains cancellation-aware, no scheduler/watcher/configuration/test was added, and bridge validation remains automated.

## Active work

**PR:** PR 1 — Improve bridge scan throughput

**Active work:** None — documentation close-out

**Commit:** None — documentation close-out

### RED or removal proof

Not applicable — all planned implementation packets passed deterministic validation and independent checkpoint review.

### Expected outcome

The complete implementation range passed final automated validation and the clear feature review with no correction rounds. Documentation reconciliation is complete, and the feature is ready for publication.

### Explicit exclusions

New implementation scope, manual or native pre-merge testing, release work, and deferred JAY-68 or JAY-69 work.

## Discoveries and replanning

- Final validation on implementation HEAD `a6749a5c9ea4dfb9839d32b9d70e0bc32db130fd` passed `./scripts/dev bridge-test` with 219 passed and 4 skipped on Linux, and `./scripts/dev check` with 872 Rust tests and 3 ignored; Windows-only native proof runs in Windows CI.
- Feature review is clear, with Test portfolio Pass, Architecture and Project fit Conform, and 0 correction rounds.
- Commit 3 uses the standard-library `ArrayBufferWriter<byte>` because the stream-backed writer commits internally before `BytesPending` reaches the threshold; the public streaming contract is unchanged.
- No speedup claim is recorded. Developer-owned post-merge Windows FM smoke and equivalent timing remain follow-up evidence only.
- No manual or native pre-merge gate applies.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Improve bridge scan throughput | Commit 1 — Record the approved feature plan | a34a314bc8c666be67820f9282b50974b19c0771 | Recorded the accepted schema-2 JAY-65 ledger and its sole TODO activation link. | `ledger_state.py` runnable; exact-path and staged diff checks clean; pre-commit fast gate passed. | Not applicable | Clear | 0 | None. |
| PR 1 — Improve bridge scan throughput | Commit 2 — Pin scalar memory reads | 000a00b77a40fdedfb0baccfd2300ec77cddd744 | Pinned scalar caller spans at the existing native read boundary and added a Windows current-process value and allocation proof. | `./scripts/dev bridge-test`: 219 passed, 4 skipped on Linux; `./scripts/dev check`: passed; staged diff check and C# diagnostics clean. | Pass | Clear | 0 | Native proof is skipped locally and runs in Windows CI. |
| PR 1 — Improve bridge scan throughput | Commit 3 — Batch dump writer flushes | f23ea8259e590a5b11ada674cd5970e7d19046ce | Buffered `Utf8JsonWriter` output to a fixed 64 KiB threshold, wrote bounded chunks, and changed the tracking proof to reject per-record flushing and full-document buffering. | `./scripts/dev bridge-test`: 219 passed, 4 skipped on Linux; `./scripts/dev check`: passed; staged diff check and C# diagnostics clean. | Pass | Clear | 0 | Used the standard-library `ArrayBufferWriter<byte>` sink because the stream-backed writer commits internally before `BytesPending` reaches the threshold; the public streaming contract is unchanged. |
| PR 1 — Improve bridge scan throughput | Commit 4 — Shorten bridge request polling | a6749a5c9ea4dfb9839d32b9d70e0bc32db130fd | Reduced the fixed request poll interval to 250 ms and updated the bridge request-protocol documentation. | `./scripts/dev bridge-test`: 219 passed, 4 skipped on Linux; `./scripts/dev check`: passed; staged diff check and C# diagnostics clean. | Pass | Clear | 0 | No test added because a test of the private constant would only restate its literal. |

## Final validation

Run automated validation only, in this order:

1. `./scripts/dev bridge-test` — execute the changed C# bridge tests, including the native Windows path in Windows CI.
2. `./scripts/dev check` — execute the repository gate before publication; GitHub must report the required strict status check before squash merge.

Do not require a manual/native pre-merge test. After merge, the developer may run a Windows FM smoke and compare equivalent before/after timing evidence. That follow-up does not establish a speedup claim by itself.

## Documentation impact

Documentation reconciliation is complete. Commit 4 updated `bridge/README.md` with the implemented fixed 250 ms interval. `.wiki/TODO.md` records completion and links the archive destination. No architecture, ADR, debug report, BACKLOG, protocol, schema, or release documentation changes are warranted. The orchestrator will move this ledger to `.wiki/features/completed/bridge-scan-throughput.md`.
