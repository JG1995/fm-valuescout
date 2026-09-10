# Production-scale FM graphics

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** b6f45d38f916e96f77fc5eb8ecd66077dc20ffea79e183f04a85cfe4c58d694a

## Intent

Deliver Linear JAY-64. Scale one selected local FM graphics root to representative Sortitoutsi Cut-Out facepacks with a config file around 50 MB and complete roots above 1,000,000 entries. Keep local graphics private, deterministic, bounded, and fail-closed at the calibrated full-root entry cap. Replace image-byte IPC and data URLs with raw image delivery through a narrow Tauri custom protocol.

## User-visible behavior

- Startup, choosing a root, and rescanning enqueue a background rebuild and return promptly. Image presentation never starts a root scan.
- Settings exposes only safe aggregate status and diagnostics. It can report rebuilding or truncation, but never a root path.
- Profile, Search, Squad, and My Club use one lazy, asynchronously decoded image component. Exact positive UIDs and the committed graphics generation remain the image identity. Text and fixed fallbacks remain while images load, fail, are missing, or the index rebuilds.
- A complete root within the calibrated entry cap installs atomically. The first entry beyond that cap fails closed and installs no index.
- Final production limits come only from measured representative-pack evidence collected by the host-aware private-root harness. The plan accepts no guessed production number or unlabeled native-Windows performance claim.

## Invariants

- Preserve root-capability traversal, component-by-component no-follow and junction/reparse protection, source containment, deterministic global lexical config precedence, logo-over-icon resolution, bounded image size, and safe aggregate diagnostics.
- Persist only the selected root in SQLite. The index, scan work, protocol results, and image cache are memory-only. No watcher or persisted generated index exists.
- For each config, stream and stage only syntactically valid compact candidates. Do not validate sources until clean EOF. A malformed XML event or malformed attribute discards every candidate from that config before any source check. A missing or unreadable source increments diagnostics and discards only that record, not its siblings. A per-config parser-work breach discards only that config.
- Entry counts are inclusive: `entries <= max_entries` succeeds; the first entry beyond `max_entries` fails closed and discards the entire index. Define and enforce per-config and root-wide parser byte, record, and attribute bounds. A root-wide parser breach discards the current incomplete config, retains only deterministic complete prior configs in global lexical order, marks the result truncated and diagnosed, and may install that completed-config prefix as existing config/mapping partial behavior does. Representative validation must show no truncation.
- Rust holds neither runtime state, transition gate, nor database mutex during root traversal, config parsing, source validation, or image file I/O. Generation checks prevent stale scan results and stale cache inserts.
- One stdlib worker, owned by runtime worker-control, serially scans one target and keeps one replaceable pending target. Runtime drop or app exit marks it stopping, clears pending work, wakes it, does not wait for an active scan, and prevents that scan from installing after stop. Active filesystem work may finish detached.
- The custom protocol accepts only an exact generation, closed graphics kind, and positive UID. It accepts no filesystem path, source text, root identifier, name, query parameter, or arbitrary request method. It returns raw validated image bytes with an explicit MIME type or a bounded normal failure response.
- Available-image caching is bounded by exact raw bytes and entry count. Missing caching is count-bounded. No cache dependency is added.

## Non-goals

- Persisted index, watcher, parallel traversal or parsing, component interning, path arena, a new cache dependency, pack management, pack download/copy/install, network graphics, Staff imagery, name matching, or unrelated graphics/UI work.
- A partial index after an entry-budget truncation, path IPC, JSON number-array/base64 image transport, a frontend filesystem API, or synchronous root discovery during image resolution.
- A mandatory `HashMap` conversion. Use it only if calibration shows that the existing deterministic structures are the measured bottleneck and a change preserves precedence and proof.

## Current-state map

- `src-tauri/src/features/graphics/index.rs` owns `GraphicsIndex`, capability traversal, `Limits`, bounded config/image reads, root-relative `Identity`, deterministic `BTreeMap` indexes, and parser diagnostics. Current limits are 8 MiB per config, 1,000,000 entries, and 500,000 accepted mappings. `parse_config` reads whole configs, stages `(from, to)` strings, `discover` uses `parent.to_vec()` per entry, `Identity::cmp` joins components, and `add_mapping` reopens source parents from root.
- `src-tauri/src/features/graphics/runtime.rs` owns selected-root generations, scan completion, and three per-kind `Lru` caches. It scans synchronously from commands and lazy `resolve`; available and missing classes each permit 256 UIDs, but available bytes have no bound.
- `src-tauri/src/features/graphics/commands.rs` registers `resolve_graphics`, which returns `ResolveResult::Available { bytes: Vec<u8>, mime }` through serde. `src-tauri/src/lib.rs` registers it and has no graphics protocol.
- `src/features/graphics/api/graphics-query-options.ts` invokes `resolve_graphics`; `types/graphics.ts` exposes number-array bytes; `graphics-ipc-mock.ts` models resolve calls. `search.tsx`, `my-club.tsx`, and `players.$uid.tsx` convert bytes to base64 data URLs and each own graphics rendering logic.
- `src-tauri/tauri.conf.json` allows `data:` images in both CSP modes. Tauri 2.11.3 supports `Builder::register_asynchronous_uri_scheme_protocol`; its documented Windows form is `http://<scheme>.localhost/...` and CSP must permit it. Source: <https://docs.rs/tauri/latest/tauri/struct.Builder.html>.
- Existing Rust tests cover malformed-config atomicity, deterministic precedence, entry-budget discard, no-follow/junction behavior, source replacement, and image bounds. Existing route tests cover exact UID requests, fallback geometry, and visible virtual-row bounds; migration replaces their resolve-IPC evidence with protocol URL evidence.
- Supported commands are `./scripts/dev test`, `./scripts/dev check-rust`, `./scripts/dev check`, `./scripts/dev smoke`, and `./scripts/dev inspect-ui`. `./scripts/dev package-windows` is installer validation, not real-pack measurement.

## Feature architecture

1. `GraphicsIndex` remains the pure capability engine. It scans configs in global lexical order, streams bounded compact candidates, and only after each clean EOF validates each candidate from the already-open config-parent capability. Per-config failures stay local; root-wide parser truncation may retain only complete lexical predecessors.
2. `GraphicsRuntime` owns a dedicated stdlib worker-control object. Commands reserve the committed generation, replace the one pending target, and return. The worker serially scans and installs only an unstopped matching generation. Lookup snapshots a mapping/cache decision, reads after locks release, and inserts only for the matching generation.
3. One asynchronous `graphics` URI protocol uses the existing runtime. It validates an exact Windows protocol URL and returns raw bytes with explicit MIME, `X-Content-Type-Options: nosniff`, and `Cache-Control: no-store`.
4. One shared `GraphicsImage` component forms the closed URL from safe graphics status, a closed kind, and a positive UID. It uses native lazy loading and asynchronous decoding, then removes itself on error so caller-owned fallback slots remain.
5. A local host-aware calibration command runs an ignored Rust test against a private environment root and emits path-free JSON with exact host/OS/filesystem context. A later commit records representative evidence and final values without labeling WSL measurements as native Windows.

## Uncertainty register

### Known

- JAY-63 persists only the selected root and already uses generations to reject stale scan completion. This feature retains that ownership model.
- `quick_xml` currently parses after a bounded whole-file read. The replacement must preserve config atomicity while separating syntactic staging from source validation.
- Tauri documents asynchronous protocol responders and the Windows `http://<scheme>.localhost` origin.
- The developer supplied a representative graphics root. The supervisor confirmed that the current host can access it through the mounted Windows filesystem.
- The current WSL environment has no `powershell.exe` bridge.

### Assumptions

- The capability crates can stream through `BufReader` while preserving the no-follow handle and metadata checks. Existing Linux and Windows containment proof must confirm that before replacement.

### Decisions

- Use one PR. Scanner, worker, protocol, consumer migration, and calibration share the same scale and image-delivery regression surface. An additional PR would expose an unused protocol or retain synchronous production behavior on trunk.
- ADR-0029 records the consequential raw-image protocol decision. Directory-index omission is accepted because this task may edit only the ledger, the JAY-64 Active entry in `.wiki/TODO.md`, and ADR-0029.
- Retain `BTreeMap`; do not add dependencies, persistence, a watcher, parallelism, interning, or a speculative `HashMap`.
- Use provisional, visibly non-production limits only until calibration records measured values. They must support generated proofs above the superseded 8 MiB and 1,000,000 thresholds.
- Commit 11 may select and record final numeric limits without a further per-constant approval. It uses approximately twice the observed structural workload for entries, config bytes, total parser bytes/records/attributes, mappings, and source work when that remains reasonable; it preserves depth 32 and the 8 MiB per-image limit unless the representative pack disproves them; and it derives available-cache count and bytes from observed image sizes and peak working set.
- The developer waives native Windows Tauri custom-protocol/CSP manual proof. This is an accepted validation gap, not a protocol, CSP, or filesystem-security change. Unit parser/response tests, compilation, existing automated Linux/Windows Rust checks, frontend tests, and `./scripts/dev smoke` still run; smoke is Chromium-stub evidence and does not prove native Windows custom-protocol registration or CSP.

### Unknowns

- Representative structural totals, phase timings, peak working set, host/OS/filesystem context, image-size distribution, and final entry/config/mapping/parser/cache limits.

### Risks

- Streaming code can commit before EOF, erase valid siblings after one missing source, or turn a local breach into a root failure. Packets 2–4 isolate and prove those semantics.
- Worker shutdown can install after stopping or block app exit. Packet 5 owns that lifecycle and controlled interleavings.
- Protocol validation or CSP can broaden filesystem access. Packets 8–9 use a closed request grammar and delete the former transport only after all consumers migrate.
- Supporting the representative pack with reasonable headroom can require excessive scan time or memory, weaken a security invariant, cause partial entry-truncated installation, or require a structural change outside this plan. Packet 11 stops only for those conditions.

## Walking skeleton

Record the reviewed plan and ADR; parse one generated config larger than 8 MiB without committing malformed input; scan more than 1,000,000 entries with the inclusive cap and fail-closed overflow; validate a clean config's sibling sources through its parent capability; enqueue one persisted root through the owned worker; serve one portrait through the closed protocol; then migrate every consumer and calibrate final values.

## Delivery plan

### PR 1 — Scale local graphics delivery

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/production-scale-fm-graphics`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub protected `main` requires strict status `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `feat(graphics): scale local graphics delivery`

**Purpose:** Deliver bounded indexing, non-blocking runtime ownership, raw-image delivery, and measured final limits as one mergeable JAY-64 outcome.

**Depends on:** JAY-63 local graphics, merged at `11479c695d017b05c9e452d9f6be821342638f3e`.

#### Commit 1 — Record the approved production-scale plan

**Status:** Completed

**Provisional commit:** `docs(graphics): record production-scale plan`

**Work:** Commit the independently reviewed JAY-64 ledger, `.wiki/TODO.md` Active entry, and ADR-0029 before implementation.

**Atomicity:** This is the smallest sensible planning outcome: the ledger, its committed-work pointer in `.wiki/TODO.md`, and the accepted ADR establish one delivery decision. A further split cannot yield two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because either delivery intent, its committed-work pointer, or its durable protocol rationale would be absent.

**Out of scope:** Implementation, tests, executable configuration, BACKLOG, planned-spec deletion (none exists), delivery authorization generation, and Git mutation in this planning task.

**Implementation packet:** Commit only the three reviewed planning paths after renewed authorization and branch verification. No planned-spec or BACKLOG change exists.

**Files and responsibilities:**

- `.wiki/features/active/production-scale-fm-graphics.md` — intent, packets, limits policy, risks, and calibration record.
- `.wiki/TODO.md` — JAY-64 Active entry that records the committed-work pointer.
- `.wiki/decisions/0029-local-graphics-custom-protocol.md` — accepted raw-image delivery rationale.

**Behavior and data flow:** Planning truth becomes one reviewed active ledger, committed-work pointer, and ADR before executable work. The ADR records a proposed boundary and does not claim implementation.

**Ordered implementation steps:**

1. Verify the authorized branch and reviewed three-path diff.
2. Run the classifier and exact staged inspection.
3. Obtain checkpoint review before committing.

**Patterns to verify:** Active-ledger template, ADR-0014 boundary, and ADR threshold.

**Constraints and non-goals:** Do not alter scope, packets, implementation, configuration, or reviewed authority after review.

**Dependencies and sequencing:** Requires cleared independent review, developer acceptance, renewed delivery authorization, and authorized branch activation.

**Tests and proof:** Not applicable. The classifier, exact staged diff, whitespace check, and checkpoint review prove schema and scope.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/production-scale-fm-graphics.md`; `git diff --cached --stat`; `git diff --cached`; `git diff --cached --check`.

**Stop conditions:** Stop on an uncleared review, classifier error, unreviewed path, branch/base mismatch, substantive post-review change, or an attempt to deliver before renewed authorization.

**Review mandate:** Confirm one active PR and commit, planning-only scope, the exact three paths (the ledger, `.wiki/TODO.md` Active entry, and ADR-0029), no directory-index edit, no delivery authorization, and no implementation claim.

#### Commit 2 — Stream bounded config candidates

**Status:** Completed

**Provisional commit:** `feat(graphics): stream bounded config candidates`

**Work:** Replace whole-config parsing with bounded streaming syntactic candidate staging and define per-config and root-wide parser byte, record, and attribute limits.

**Atomicity:** This is the smallest sensible parser-contract outcome: streaming, all six parser bounds, per-config rejection, and complete-prefix root truncation need one parser state and proof. A further split cannot produce two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because any half would leave an unbounded parser dimension or ambiguous root-prefix result.

**Out of scope:** Discovery identity allocation, entry-cap behavior, source validation, runtime scheduling, cache policy, protocol/CSP/frontend work, and final values.

**Implementation packet:**

- Open each config through its capability parent and wrap it in a bounded buffered reader. Stage compact, syntactically valid `from`/`to` candidates only; do not source-check or index them while events stream.
- Count bytes, records, and attributes both per config and root-wide. Per-config byte/record/attribute breach discards that config and records a safe diagnostic. Malformed XML or attribute decoding also discards it before source validation.
- Process configs in global lexical order. On a root-wide parser breach, discard the current incomplete config, mark `truncated` and diagnose it, and retain only already-complete lexical predecessor configs for later source validation and installation. Representative calibration must reject a truncated result.
- Keep provisional limits large enough to generate a config above the prior 8 MiB ceiling. Do not call any provisional value production-ready.

**Files and responsibilities:**

- `src-tauri/src/features/graphics/index.rs` — streaming parser, compact candidate representation, parser counters, ordered complete-config prefix, and Rust tests.
- `src-tauri/src/features/graphics/runtime.rs`, `src/features/graphics/types/graphics.ts`, `src/features/graphics/api/graphics-ipc-mock.ts`, and `src/features/graphics/components/graphics-settings-section.tsx` — only safe aggregate diagnostics when the DTO changes.

**Behavior and data flow:** A config becomes a complete candidate collection only at clean EOF. Invalid syntax or per-config work erases that collection. Root-wide exhaustion ends parsing at the current collection and permits only earlier complete collections in lexical order.

**Ordered implementation steps:**

1. Add failing generated parser-bound and malformed-after-valid-record proofs.
2. Stream compact candidates and account for all parser work.
3. Implement local discard and root-wide complete-prefix truncation.
4. Propagate only safe aggregate diagnostics.

**Patterns to verify:** `read_bounded`, `quick_xml::Reader`, current `parse_config`, and `GraphicsSummaryDto` serialization.

**Constraints and non-goals:** Do not validate a source before EOF, commit a partial config, allocate whole config bytes, or set final values.

**Dependencies and sequencing:** Follows Commit 1. Commit 4 consumes only clean complete candidate groups.

**Tests and proof:** Add generated temporary input, not a fixture. Prove a config above 8 MiB streams; malformed XML/attribute after valid records leaves no complete candidate; each per-config bound discards only its config; each root-wide bound discards its incomplete config and retains only earlier lexical complete configs; and diagnostics remain path-free. Existing source-containment tests remain for Packet 4.

**Validation:** Focused graphics index Rust tests; `./scripts/dev check-rust`; `./scripts/dev check`; required Windows `rust-windows` Check job.

**Stop conditions:** Replan if the reader cannot enforce a bound before unbounded allocation, a malformed config can escape its staged collection, root-wide truncation cannot preserve deterministic complete predecessors, or a generated >8 MiB proof requires a committed large file.

**Review mandate:** Trace every byte, record, and attribute counter at both scopes; verify no source access occurs before clean EOF; verify local versus root-wide discard semantics and safe diagnostics; reject final numeric claims.

#### Commit 3 — Reduce discovery allocation and enforce entry cap

**Status:** Completed

**Provisional commit:** `feat(graphics): bound graphics discovery`

**Work:** Remove discovery hot-path parent cloning/comparator joins and enforce the inclusive root entry cap with a generated proof above 1,000,000 entries.

**Atomicity:** This is the smallest sensible discovery outcome: mutable component traversal and the inclusive cap govern the same entry walk and need one traversal proof. A further split cannot produce two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because allocation changes alone do not establish the new capped discovery contract, and cap changes alone retain the identified hot path.

**Out of scope:** Config parser/source semantics, worker lifecycle, cache policy, protocol/frontend work, and final limits.

**Implementation packet:**

- Append and pop a mutable root-relative identity during traversal; compare components lexically without `Identity::join()` and do not call `parent.to_vec()` for each entry.
- Count an entry before accepting it. `entries <= max_entries` completes; the first greater entry records `entry_limit`, marks truncation, stops discovery, and discards the entire candidate index rather than installing any config prefix.
- Preserve depth/config-count/no-follow behavior, deterministic global config ordering, and provisional generated test limits above 1,000,000 entries.

**Files and responsibilities:**

- `src-tauri/src/features/graphics/index.rs` — traversal identity, inclusive entry accounting, fail-closed candidate discard, and tests.

**Behavior and data flow:** Capability traversal collects sorted configs only while its global entry budget admits each entry. Overflow returns no installable candidate; non-overflow feeds Packet 2's parser.

**Ordered implementation steps:**

1. Add equality and first-beyond generated entry proofs.
2. Replace clone/join identity handling in the traversal.
3. Count before admission and discard overflow candidates.
4. Re-run ordering and containment proofs.

**Patterns to verify:** `discover`, `consume_entries`, `Identity::cmp`, `retain_config`, and current temporary-tree tests.

**Constraints and non-goals:** Do not install a discovered prefix, change config parser/source behavior, or add interning or `HashMap`.

**Dependencies and sequencing:** Follows Commit 1 and remains independent of Commit 2's parser internals; Commit 4 uses its globally ordered config list.

**Tests and proof:** Add a generated temporary discovery stream above the old one-million threshold. Prove equality is accepted, the first additional entry fails closed with zero installed mappings, and deterministic precedence/no-follow behavior remains.

**Validation:** Focused graphics index Rust tests; `./scripts/dev check-rust`; `./scripts/dev check`; required Windows `rust-windows` Check job.

**Stop conditions:** Replan if overflow can preserve any index, the comparison changes lexical precedence, or the proof needs a committed million-entry fixture.

**Review mandate:** Verify inclusive arithmetic at the exact boundary, candidate discard on the first excess entry, no `parent.to_vec()` or comparator `join`, and unchanged containment/ordering contracts.

#### Commit 4 — Validate config sources by parent capability

**Status:** Completed

**Provisional commit:** `feat(graphics): validate staged config sources`

**Work:** Reuse the already-open config-parent capability to validate each clean-EOF candidate source, preserving per-record source-failure containment and atomic config syntax semantics.

**Atomicity:** This is the smallest sensible source-validation outcome: parent-capability reuse and sibling containment are one validation loop with one observable result. A further split cannot produce two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because source reuse without containment would change the trust boundary incompletely, while containment without the parent capability would retain root reopen behavior.

**Out of scope:** New parser bounds, entry accounting, runtime work, cache policy, protocol/frontend work, and final limits.

**Implementation packet:**

- After and only after clean EOF, validate staged candidates in record order through the config's already-open parent capability. Keep only root-relative identities for later resolution.
- A missing, unreadable, unsafe, or unsupported source increments safe source diagnostics and discards only that record. Continue with its siblings and later configs. Do not reopen the config parent from root.
- Keep malformed XML/attribute and per-config parser breaches as complete-config rejection before this loop. Preserve source containment, extension probe order, duplicate precedence, logo-over-icon behavior, image bounds, and the root-wide parser complete-prefix result.

**Files and responsibilities:**

- `src-tauri/src/features/graphics/index.rs` — parent-capability source validation, staged commit fold, diagnostics, and containment tests.

**Behavior and data flow:** Global lexical config order supplies clean candidate groups. Each group validates source records after EOF; accepted records join the deterministic index while missing siblings do not affect other valid records.

**Ordered implementation steps:**

1. Add sibling-missing and no-source-before-EOF failing tests.
2. Pass clean candidate groups with their open config-parent capability.
3. Validate records, retain accepted root-relative identities, and continue after local failures.
4. Re-run containment and deterministic-resolution tests.

**Patterns to verify:** `open_file`, `open_parent`, `source_candidates`, `parse_target`, and existing Linux/Windows security tests.

**Constraints and non-goals:** Do not reopen the parent from root, discard siblings, weaken no-follow checks, or revise parser bounds.

**Dependencies and sequencing:** Requires Commit 2's clean candidate groups and Commit 3's globally ordered discovery.

**Tests and proof:** Prove a missing source increments diagnostics while a sibling installs; malformed XML/attributes perform no source checks and install nothing from that config; a per-config breach installs nothing from that config; extension probes and duplicate precedence remain deterministic; Linux replacement/no-follow and Windows junction/reparse proof still reject escapes.

**Validation:** Focused graphics index Rust tests; `./scripts/dev check-rust`; `./scripts/dev check`; required Windows `rust-windows` Check job.

**Stop conditions:** Replan if validation needs a root reopen, a missing source discards a sibling, malformed input reaches source validation, or existing Linux/Windows containment proof changes its supported result.

**Review mandate:** Verify source work starts only after clean EOF; trace one missing record beside a valid sibling; inspect parent-capability ownership and every extension probe; confirm malformed and per-config failure containment.

#### Commit 5 — Rebuild graphics through one stopping worker

**Status:** Completed

**Provisional commit:** `feat(graphics): rebuild graphics in background`

**Work:** Move startup, choose, and rescan to one runtime-owned serial stdlib worker with replaceable pending work and non-blocking stopping lifecycle.

**Atomicity:** This is the smallest sensible worker-control outcome: ownership, serial coalescing, stopping, and generation-safe installation jointly define one lifecycle. A further split cannot produce two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because a worker without its stop protocol leaks install authority, and stop without its queue owner has no coherent behavior.

**Out of scope:** Image I/O/cache accounting, protocol/CSP/frontend work, watchers, parallel scanning/parsing, persistence changes, and final limits.

**Implementation packet:**

- Keep `GraphicsRuntime::new` passive. After `app.manage`, start the runtime worker only after the startup database lock scope drops.
- Runtime worker-control owns one stdlib thread, wake primitive, `stopping` flag, and one replaceable pending committed target. The worker scans serially. Startup, choose, and rescan replace pending work and return; a blocked A followed by B lets A finish detached, suppresses A installation when stale, then scans B once.
- Clear and runtime drop/app exit mark stopping, clear pending work, wake the worker, and return without blocking on active filesystem work. Completion checks stopping and committed generation so an active detached scan cannot install after stop.
- `resolve` never enqueues or performs traversal. Preserve failed-persistence and current generation semantics. Add safe rebuilding status only if Settings needs it.

**Files and responsibilities:**

- `src-tauri/src/features/graphics/runtime.rs` — worker-control lifecycle, queue, stopping state, generation checks, and controlled interleaving tests.
- `src-tauri/src/features/graphics/commands.rs` — enqueue after durable transition and return safe status.
- `src-tauri/src/lib.rs` — post-manage startup activation after the DB lock drops.
- Frontend graphics status type/mock/Settings files only if rebuilding status changes.

**Behavior and data flow:** SQLite commits the root, then commands submit the committed target. The one worker takes at most one target at a time. Stopping revokes pending and active installation authority without waiting for filesystem completion.

**Ordered implementation steps:**

1. Add controlled startup, A→B, clear, and stopping interleavings.
2. Introduce runtime-owned worker-control and passive construction.
3. Enqueue after durable transitions and activate after `app.manage`.
4. Add stopping revocation without joining active filesystem work.

**Patterns to verify:** `persist_transition_with`, `begin_rescan_with`, `complete_scan`, injected scanner tests, and `lib.rs` setup scope.

**Constraints and non-goals:** Do not spawn per-command workers, scan in `resolve`, add a watcher, run parallel scans, or block exit.

**Dependencies and sequencing:** Requires scale-safe indexing from Commits 2–4. Commit 6 consumes installed-index-only lookup.

**Tests and proof:** Prove passive construction, startup after DB-lock release, A→B coalescing with one active scan, prompt command return, clear suppressing queued work, failed persistence retaining A, `resolve` not scanning, and focused drop/exit stopping behavior that clears pending, wakes the worker, returns while active work is blocked, and rejects that active result.

**Validation:** Focused graphics runtime tests; `./scripts/dev check-rust`; `./scripts/dev check`; required Windows `rust-windows` Check job.

**Stop conditions:** Replan if one serial stdlib worker cannot own stopping without a framework, exit must wait for active filesystem work, A and B can scan concurrently, or a post-stop result can install.

**Review mandate:** Trace DB-lock release through startup, queue replacement, clear, drop/exit, and every generation/stopping installation check; reject command-spawned workers, a watcher, parallel work, or blocking shutdown.

#### Commit 6 — Read graphics outside runtime locks

**Status:** Completed

**Provisional commit:** `feat(graphics): read graphics outside locks`

**Work:** Snapshot a generation-safe locator under brief state access, perform image I/O unlocked, and retain the existing count-bounded caches and resolve IPC.

**Atomicity:** This is the smallest sensible lookup-seam outcome: locator snapshot, unlocked read, and generation-safe insertion are one correctness path. A further split cannot produce two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because unlocked I/O without a stale insertion guard is unsafe, while a guard without an unlocked read retains the blocking defect.

**Out of scope:** Available-cache byte accounting, protocol/CSP/frontend migration, worker scheduling, and final limits.

**Implementation packet:**

- Under brief runtime state, validate installed matching generation and return a cached result or root-relative locator. Release runtime state, transition gate, and DB mutex before capability open/read/validation.
- Reacquire state only to cache a successful or missing result if that generation remains committed. Keep the current per-kind available and missing UID count bounds in this commit.
- Retain `resolve_graphics` and its existing DTO temporarily for Packet 8.

**Files and responsibilities:**

- `src-tauri/src/features/graphics/index.rs` and `runtime.rs` — locator/read seam, generation-safe insertion, and tests.
- `src-tauri/src/features/graphics/commands.rs` — only ownership adjustments needed by lookup.

**Behavior and data flow:** State supplies an immutable locator; capability I/O happens outside locks; a matching generation is required before cache mutation or response use.

**Ordered implementation steps:**

1. Add blocked-read and stale-insert RED proofs.
2. Expose a root-relative locator from the installed index.
3. Release locks for capability read and recheck generation before insert.
4. Retain the existing count caches and resolve DTO.

**Patterns to verify:** `GraphicsIndex::resolve`, `read_image`, `Lru`, and stale-completion runtime tests.

**Constraints and non-goals:** Do not scan in lookup, retain locks during I/O, add byte policy yet, or remove resolve IPC.

**Dependencies and sequencing:** Requires Commit 5's installed-index-only worker behavior. Commit 7 extends this cache seam.

**Tests and proof:** Prove a blocked image read does not block a root transition, stale bytes neither serve nor enter B's cache, and existing count-bounded available/missing behavior survives. Revise only obsolete lazy-scan expectations.

**Validation:** Focused graphics runtime/index tests; `./scripts/dev check-rust`; `./scripts/dev check`; required Windows `rust-windows` Check job.

**Stop conditions:** Replan if the locator cannot remain root-relative, I/O requires a graphics/transition/DB lock, or stale completion can cache under a newer generation.

**Review mandate:** Inspect lock scope across every image open/read, stale cache insertion, and retained count-cache behavior; confirm lookup never scans.

#### Commit 7 — Account for graphics cache bytes

**Status:** Active

**Provisional commit:** `feat(graphics): bound graphics cache bytes`

**Work:** Add exact per-kind available-cache byte and count accounting, while retaining the bounded missing-cache count.

**Atomicity:** This is the smallest sensible cache-cap outcome: exact insert, replacement, eviction, clear, and generation replacement accounting form one cache invariant. A further split cannot produce two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because a byte cap without every mutation path is false accounting, and changing missing policy separately has no independent behavior.

**Out of scope:** New cache dependency, worker changes, protocol/CSP/frontend migration, persistence, and final calibrated values.

**Implementation packet:**

- Extend each available LRU with provisional raw-byte capacity and exact total accounting. Update total bytes on insert, replacement, LRU eviction, clear, and generation replacement; enforce both available count and bytes.
- Keep missing results count-bounded and do not cache bytes for missing results. Retain resolve IPC temporarily.

**Files and responsibilities:**

- `src-tauri/src/features/graphics/runtime.rs` — available-cache accounting and tests.

**Behavior and data flow:** A successful image enters an LRU only when both limits can be enforced exactly. Missing values use the existing count-only cache; generation replacement clears both classes.

**Ordered implementation steps:**

1. Add RED accounting tests for each cache mutation.
2. Track raw bytes with available entries.
3. Enforce count and byte eviction together.
4. Re-run clear/replacement and missing-cache tests.

**Patterns to verify:** Current `Lru::get`, `Lru::put`, `clear`, and `commit_locked` cache replacement.

**Constraints and non-goals:** Do not add a cache dependency, persist cache state, change missing entries into byte-bearing values, or set final values.

**Dependencies and sequencing:** Requires Commit 6's generation-safe unlocked lookup. Commit 8 reuses it for protocol results.

**Tests and proof:** Prove exact totals after insert/replacement/touch/eviction/clear/generation replacement, available count and byte caps, and the missing count bound. Values remain provisional until Packet 11.

**Validation:** Focused graphics runtime tests; `./scripts/dev check-rust`; `./scripts/dev check`; required Windows `rust-windows` Check job.

**Stop conditions:** Replan if exact accounting cannot cover an eviction path or needs a dependency.

**Review mandate:** Recompute byte totals through every mutation, verify count and byte limits per kind, and confirm missing results never consume the available-byte budget.

#### Commit 8 — Register closed graphics protocol

**Status:** Pending

**Provisional commit:** `feat(graphics): register closed image protocol`

**Work:** Register and test the asynchronous closed graphics protocol and narrow CSP while retaining resolve IPC for the next migration commit.

**Atomicity:** This is the smallest sensible protocol-seam outcome: handler registration, closed parsing, raw response security headers, and CSP must land together to prove one usable native boundary. A further split cannot produce two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because registration without CSP is not usable and CSP without a closed handler broadens policy without behavior.

**Out of scope:** Consumer migration, shared component, deletion of resolve IPC/query/type/mock/data URLs, new graphics kinds, and final limits.

**Implementation packet:**

- Register `graphics` through `Builder::register_asynchronous_uri_scheme_protocol` before `.run`, retrieving only the managed `GraphicsRuntime`.
- Accept only `GET http://graphics.localhost/<generation>/<kind>/<uid>` with exactly three canonical non-empty path segments, closed kind, positive decimal UID, no query or fragment, and matching committed generation. Reject wrong host/method/segments/kind/UID/stale/missing/unreadable cases with bounded non-success responses and no unauthorized file access.
- Return validated raw PNG, JPEG, or WebP bytes with `Content-Type`, `X-Content-Type-Options: nosniff`, and `Cache-Control: no-store`. Preserve the existing resolve IPC until Packet 9.
- Permit exactly `http://graphics.localhost` in production and development `img-src`. Retain `asset:` and remove `data:` only after the Packet 9 consumer audit proves it unused.

**Files and responsibilities:**

- `src-tauri/src/lib.rs` — asynchronous protocol registration using existing runtime.
- `src-tauri/src/features/graphics/{runtime.rs,commands.rs,mod.rs}` — protocol-safe matching lookup while retaining resolve command.
- `src-tauri/tauri.conf.json` — narrow production and development image CSP.
- Rust protocol tests — closed grammar and bounded response proof.

**Behavior and data flow:** A closed URL reaches the existing runtime, which supplies generation-safe raw-image lookup. The handler returns bytes or a bounded error and does not create state or path authority.

**Ordered implementation steps:**

1. Add protocol parser/response RED tests.
2. Register the asynchronous handler against managed runtime state.
3. Add narrow CSP origins and response headers.
4. Run parser/response and configuration checks; record the developer-approved native Windows protocol/CSP validation gap.

**Patterns to verify:** `tauri::Builder`, managed state access in `lib.rs`, `GraphicsKindDto`, and existing CSP configuration.

**Constraints and non-goals:** Do not accept a path/query/fragment, create another runtime/cache, remove resolve IPC, or broaden CSP.

**Dependencies and sequencing:** Requires Commit 7's byte-bounded lookup. Commit 9 is the only consumer/deletion migration.

**Tests and proof:** Rust parser/response tests cover accepted raw MIME response and invalid method/host/segments/kind/UID/generation/query/path/missing/unreadable cases without file access. Retain resolve IPC tests until Packet 9. Record the developer-approved native Windows protocol/CSP gap; `./scripts/dev smoke` remains Chromium-stub UI evidence only.

**Validation:** Focused Rust protocol/graphics tests; `./scripts/dev check-rust`; `./scripts/dev check`; required Windows `rust-windows` Check job; `./scripts/dev smoke` as Chromium-stub evidence, not native Windows protocol/CSP proof.

**Stop conditions:** Replan if managed runtime access requires a second owner, CSP needs a broad source, the handler accepts identity beyond generation/kind/UID, or the documented origin form cannot remain within the closed grammar.

**Review mandate:** Inspect every grammar rejection and header, narrow CSP source, existing-runtime ownership, no-store retention, temporary resolve IPC boundary, and the recorded native Windows validation gap.

#### Commit 9 — Migrate graphics consumers to one component

**Status:** Pending

**Provisional commit:** `feat(graphics): migrate graphics to protocol`

**Work:** Migrate all consumers to one shared image component and atomically delete obsolete resolve IPC/query/type/mock/base64 assets.

**Atomicity:** This is the smallest sensible transport-removal outcome: all consumers, the shared component, and obsolete contract deletion must move together. A further split cannot produce two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because it would leave two transports, an unused public protocol, or a compatibility asset with no supported consumer.

**Out of scope:** Root/index/persistence changes, styling redesign, Staff imagery, new kinds, network URLs, and final limits.

**Implementation packet:**

- Add one graphics feature component that accepts only a closed `GraphicsKind`, positive UID, and existing slot context. It reads safe status, constructs only the protocol URL, uses `loading="lazy"` and `decoding="async"`, and removes itself after native error.
- Compose it in Profile, Search, Squad, and My Club without changing caller-owned fallback geometry, text identity, exact-UID guards, or virtual-row request bounds.
- Delete `resolve_graphics`, `ResolveResult`, result Query keys/options, number-array result types, resolve mocks/deferred machinery, route-local base64 helpers, and obsolete tests/assets in the same commit. Retain status and mutation APIs.
- Audit CSP after deletion and remove `data:` only when no other image flow needs it.

**Files and responsibilities:**

- `src-tauri/src/features/graphics/{commands.rs,runtime.rs,mod.rs}` and `src-tauri/src/lib.rs` — delete resolve IPC and its registration.
- `src/features/graphics/{components,api,types}` — shared component, retained status APIs, and obsolete asset deletion.
- `src/app/routes/{search.tsx,my-club.tsx,players.$uid.tsx}` — compose the shared component and remove transport code.
- Route/component tests, `src/testing/setup.ts`, `e2e/tauri-ipc-stub.ts`, `e2e/smoke.spec.ts`, and `src-tauri/tauri.conf.json` — protocol URL/fallback proof and deletion/CSP audit.

**Behavior and data flow:** Safe status plus exact identity forms a protocol URL. Native image loading calls Packet 8's handler; error leaves the existing fallback. No image bytes cross invoke or become base64.

**Ordered implementation steps:**

1. Add shared-component URL and fallback RED proofs.
2. Migrate Profile, Search, Squad, and My Club.
3. Replace transport tests and mocks with protocol URL evidence.
4. Delete all obsolete resolve/query/type/mock/base64 assets and audit CSP.

**Patterns to verify:** Existing graphics status query, `PlayerIdentityRail`, Search/Squad identity cells, `ManagedClubLogo`, and route virtualization tests.

**Constraints and non-goals:** Do not retain compatibility transport, change slot geometry/text, add a filesystem API, or add network URLs.

**Dependencies and sequencing:** Requires Commit 8's unit-proven closed protocol and narrow CSP. Packet 11 measures this final image path.

**Tests and proof:** Frontend tests prove exact URLs, lazy/async attributes, error fallback, text/slot preservation, and visible-row bounds. Replace resolve-IPC mocks/assertions and prove obsolete symbols/assets absent with focused search plus TypeScript/Rust compilation. `./scripts/dev smoke` remains Chromium-stub UI evidence and does not prove native Windows protocol registration or CSP.

**Validation:** `./scripts/dev test`; focused Rust graphics tests; `./scripts/dev check-rust`; `./scripts/dev check`; `./scripts/dev smoke`; `./scripts/dev inspect-ui /players/42?section=overview 1280 800`; `./scripts/dev inspect-ui /search 1280 800`; `./scripts/dev inspect-ui /my-club 1280 800`; required Windows `rust-windows` Check job. Record the developer-approved native Windows protocol/CSP validation gap.

**Stop conditions:** Replan if any consumer needs resolve compatibility, a fallback cannot remain caller-owned, a URL can include unclosed input, or CSP must be broader.

**Review mandate:** Confirm all four consumers use one component; raw bytes never cross invoke/serde/base64; old commands, query/type/mock/base64 assets are gone; CSP is narrow; and all fallback/virtualization behavior remains.

#### Commit 10 — Add private graphics calibration harness

**Status:** Pending

**Provisional commit:** `test(graphics): add calibration harness`

**Work:** Add the reusable local host-aware `graphics-calibration` command and ignored test that emit path-free JSON with exact host/OS/filesystem context, including the required future `AGENTS.md` command-surface documentation in this implementation commit.

**Atomicity:** This is the smallest sensible reusable-harness outcome: command dispatch, ignored measurement test, privacy and execution-context contract, and its command documentation describe one runnable tool. A further split cannot produce two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because a command without its test/privacy/context proof is unusable and documentation without the command is false.

**Out of scope:** Running the representative pack, selecting final values, editing this planning task's prohibited `AGENTS.md`, pack assets, root paths, arbitrary optimization, and `HashMap` conversion.

**Implementation packet:**

- Add `graphics-calibration` to `./scripts/dev`. It requires `FM_VALUESCOUT_GRAPHICS_ROOT`, invokes one named ignored Rust test with `--nocapture` on the local host, and emits one path-free JSON report. It must work in the current WSL environment against the mounted Windows filesystem and must not require a PowerShell bridge. Missing or unreadable roots fail without printing the path.
- Add the ignored Rust test. It runs discovery/config/source phases and one cold plus warm protocol image request, then reports only test metadata, exact host/OS/filesystem context, phase counts/timings, index totals, peak-working-set method/result, and image timings. It emits no root path, pack asset, UID sample, or image bytes. It labels WSL results as WSL and never calls them native Windows performance.
- In this implementation commit, update `AGENTS.md` to document `./scripts/dev graphics-calibration`, its private-root contract, and host-context labeling. Do not edit `AGENTS.md` in the current planning diff.

**Files and responsibilities:**

- `scripts/dev` — local host-aware command dispatch and environment handling without echoing the root.
- `src-tauri/src/features/graphics/{index.rs,runtime.rs}` and tests — named ignored harness, path-free JSON report, and execution-context metadata.
- `AGENTS.md` — future stable command-surface documentation, changed only by this implementation commit.

**Behavior and data flow:** A private environment root enters only the local test process. The harness returns aggregate measurements plus their host/OS/filesystem context for later human review; no private identifier enters repository output.

**Ordered implementation steps:**

1. Add the ignored test and path-free/context-label JSON assertion.
2. Add local host-aware command dispatch without echoing the environment root or requiring PowerShell.
3. Reject missing or unreadable roots and prove the current WSL mounted-filesystem path works without labeling results native Windows.
4. Update `AGENTS.md` in this implementation commit only.

**Patterns to verify:** Existing ignored-test convention in `src-tauri/src/features/snapshot/ingest.rs`, `scripts/dev` command dispatch, and graphics status privacy tests.

**Constraints and non-goals:** Do not run a private representative pack in CI, print/store its root, persist raw logs, edit `AGENTS.md` in the planning diff, or choose production values.

**Dependencies and sequencing:** Requires Packet 9's delivered protocol for cold/warm image timings. Packet 11 consumes this harness without a later numeric-constant approval.

**Tests and proof:** Prove the ignored test runs only through the command, rejects missing/unreadable input without echoing it, produces JSON without the supplied path, path separators, UID samples, or bytes, and records host/OS/filesystem context that distinguishes WSL from native Windows. Retain generated scale proofs from Packets 2–3.

**Validation:** Focused ignored-harness test through `FM_VALUESCOUT_GRAPHICS_ROOT=<private-root> ./scripts/dev graphics-calibration` in the current WSL environment with a disposable local root; path-free JSON/privacy and host-context assertion; `./scripts/dev check-rust`; `./scripts/dev check`; required Windows `rust-windows` Check job.

**Stop conditions:** Replan if the stable command cannot keep the root private, cannot run against the local host's mounted Windows filesystem without PowerShell, cannot label its execution context correctly, the ignored test cannot exercise the delivered protocol, or `AGENTS.md` cannot accurately document its stable contract.

**Review mandate:** Verify local host-aware behavior, no path echo/leak in JSON/errors, ignored-test invocation, exact WSL/native-Windows labeling, protocol cold/warm measurement, and that the implementation commit—not this planning task—updates `AGENTS.md`.

#### Commit 11 — Calibrate final graphics limits

**Status:** Pending

**Provisional commit:** `perf(graphics): calibrate production limits`

**Work:** Use representative-pack evidence from the host-aware harness to set final bounded limits, update tests, and record safe calibration evidence in the ledger.

**Atomicity:** This is the smallest sensible evidence-to-constant outcome: representative measurement, chosen values, final-bound proof, and safe ledger record are one auditable decision. A further split cannot produce two coherent, independently reviewable, revertible, trunk-safe outcomes with meaningful proof because constants without evidence violate the accepted basis and evidence without final values leaves production intentionally provisional.

**Out of scope:** New feature behavior, pack assets or paths, implementation refactors, watcher/parallelism/dependencies, and a `HashMap` conversion without a bounded replan.

**Implementation packet:**

- Run `FM_VALUESCOUT_GRAPHICS_ROOT=<private-root> ./scripts/dev graphics-calibration` privately against the developer-supplied representative root. Record only the harness's path-free structural totals, phase timings, peak-working-set method/result, first/warm protocol latency, and exact host/OS/filesystem context. Do not persist the path, raw log, pack asset, UID, or image. Do not label WSL timing or memory data as native Windows performance.
- Set final entry, config, mapping, parser byte/record/attribute, image, available-cache byte/count, and missing-count bounds from that evidence. Where reasonable, use approximately twice the observed entries, config bytes, total parser bytes/records/attributes, mappings, and source work for ordinary-growth headroom. Preserve depth 32 and the 8 MiB per-image bound unless the representative pack disproves either. Derive available-cache count and bytes from observed image sizes and peak working set rather than doubling all memory limits. Replace provisional markers consistently.
- Require the representative complete run to have no truncation. Rerun generated tests against final values, including `<= max_entries` success and first-beyond whole-index failure. The worker may choose and record the final numbers without a further approval.

**Files and responsibilities:**

- `src-tauri/src/features/graphics/{index.rs,runtime.rs}` and tests — measured final constants and final-bound proof.
- `.wiki/features/active/production-scale-fm-graphics.md` — path-free structural evidence, host/OS/filesystem context, chosen values, no-truncation result, and validation evidence.

**Behavior and data flow:** The delivered runtime measures one complete selected root and raw protocol reads. Safe aggregate evidence selects bounded constants with ordinary-growth headroom; later over-cap roots still install no index.

**Ordered implementation steps:**

1. Run the private host-aware harness and record only safe evidence and its execution context.
2. Select constants using the delegated headroom and cache-sizing rules, then update final-bound tests.
3. Verify no truncation, complete-root support, and retained entry-overflow failure.
4. Record values and evidence in this ledger with no private data.

**Patterns to verify:** Harness JSON schema, current `Limits`, cache tests, generated scale tests, and JAY-63's real-pack validation gap.

**Constraints and non-goals:** Do not guess values, commit a pack/path/log/image, weaken cap behavior or a security invariant, optimize speculatively, or change `BTreeMap` without replan.

**Dependencies and sequencing:** Requires Commit 10 and the developer-supplied representative root through the current host's mounted Windows filesystem. No later numeric-constant approval or native Windows protocol/CSP proof is required.

**Tests and proof:** Prove generated >8 MiB and >1,000,000 legacy-threshold cases, final `<=` entry acceptance, first-beyond fail-closed discard, no representative truncation, path-free report/ledger, correct host-context labeling, bounded cache accounting based on the measured image-size and peak-working-set evidence, and measured first/warm methodology. Add no pack fixture. The developer-approved native Windows protocol/CSP gap remains; `./scripts/dev smoke` is Chromium-stub evidence only.

**Validation:** `FM_VALUESCOUT_GRAPHICS_ROOT=<private-root> ./scripts/dev graphics-calibration` on the current host; focused final-bound Rust tests; `./scripts/dev check-rust`; `./scripts/dev check`; `./scripts/dev smoke`; required Windows `rust-windows` Check job; recorded peak-working-set observation and host/OS/filesystem context. Run `./scripts/dev inspect-ui` only if calibration changes component behavior.

**Stop conditions:** Replan only if supporting the representative pack with reasonable headroom would require clearly excessive scan time or memory, weaken a security invariant, permit partial entry-truncated installation, or require a structural change outside this plan.

**Review mandate:** Trace every final constant to recorded measurement and delegated sizing rule; verify no truncation, path-free evidence, host-context labeling, final cap boundaries, and cache values; reject guessed values, mislabeled native-Windows claims, unrelated optimization, or unplanned `HashMap` conversion.

## Active work

**PR:** PR 1 — Scale local graphics delivery

**Commit:** Commit 7 — Account for graphics cache bytes

### RED or removal proof

Add cache-accounting tests for insert, replacement, touch, eviction, clear, generation replacement, and missing-result bounds.

### Expected outcome

Each available-image cache enforces exact raw-byte and entry-count limits through every mutation. Missing caches remain count-bounded and consume no available-byte budget.

### Explicit exclusions

- New cache dependencies, worker changes, protocol/CSP/frontend migration, persistence, and final production values.

## Discoveries and replanning

- Current `parse_config` stages strings but reopens source parents from root. Packets 2 and 4 separate clean-EOF syntax staging from parent-capability source validation to preserve both atomicity and per-record source failure containment.
- Current entry equality fails at `>=`; Packet 3 changes the contract to accept equality and fail only on the first entry beyond the limit.
- Current runtime synchronously scans from commands and lazy resolve. Packet 5 replaces it with owned serial worker-control and non-blocking stopping behavior.
- The accepted protocol keeps `Cache-Control: no-store`; browser caching must not bypass the bounded Rust cache.
- The developer delegated final numeric limit selection. Commit 11 now uses the supplied representative root, measured structural evidence, and the recorded sizing rules; it stops only for the explicit excessive-resource, security-invariant, entry-truncation, or out-of-plan structural conditions.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Scale local graphics delivery | Commit 1 — Record the approved production-scale plan | `c4bd51150fe97e75170301c31e234561a3108f60` | Recorded the accepted schema-2 ledger, JAY-64 TODO pointer, and ADR-0029 on the authorized feature branch. | `delivery_state.py` and `ledger_state.py` reported runnable; `git diff --cached --check` passed. | Not applicable | Clear | 0 | None |
| PR 1 — Scale local graphics delivery | Commit 2 — Stream bounded config candidates | `b305ac4ea460aec248129b6fa7de7e0006aab326` | Streamed config XML into clean-EOF candidate groups with per-config and root-wide byte, record, and attribute bounds; local breaches discard one config and root breaches retain only complete lexical predecessors. | Graphics index tests passed; `./scripts/dev check-rust` and `./scripts/dev check` passed with 851 Rust tests and 2 ignored; `git diff --cached --check` passed. | Pass | Accepted findings — provisional parser limits still use the production limit name and must be labeled before close-out. | 2 | The initial implementation did not capture RED before code; correction tests reproduced and fixed the byte-precedence defect. |
| PR 1 — Scale local graphics delivery | Commit 3 — Reduce discovery allocation and enforce entry cap | `fbe18777052fdc444d78a390e0d14efd0947643c` | Reworked discovery around one mutable component path and direct lexical component comparison; made the entry cap inclusive and discarded the whole index on first overflow. | Graphics tests passed; `./scripts/dev check-rust` passed with 853 Rust tests and 2 ignored; `./scripts/dev check` and `git diff --cached --check` passed. | Pass | Clear | 0 | None |
| PR 1 — Scale local graphics delivery | Commit 4 — Validate config sources by parent capability | `350a4be30992eed8a31c66289724ed802b94ff65` | Validated clean-EOF candidates through their open config-parent capability and retained valid siblings when another source was missing or unsafe. | Graphics index tests passed; `./scripts/dev check-rust` and `./scripts/dev check` passed; `git diff --cached --check` and Rust LSP passed. | Pass | Clear | 0 | None |
| PR 1 — Scale local graphics delivery | Commit 5 — Rebuild graphics through one stopping worker | `b2ef48b167a6056442e137b92cc8b122678afe45` | Moved startup, choose, and rescan rebuilds to one runtime-owned serial worker with replaceable pending work, generation-safe installation, and non-blocking stop revocation. | Runtime tests passed; `./scripts/dev check-rust` passed with 856 Rust tests and 2 ignored; `./scripts/dev check`, `git diff --cached --check`, and LSP passed. | Pass | Accepted findings — the safe `rebuilding` flag is stale after clear and false during startup rebuild; correct it before close-out. | 0 | None |
| PR 1 — Scale local graphics delivery | Commit 6 — Read graphics outside runtime locks | Pending record | Split lookup into a generation-validated locator snapshot, unlocked image read, and generation-rechecked cache insertion while retaining count-bounded caches and resolve IPC. | Focused graphics tests passed; `./scripts/dev check-rust` passed with 857 Rust tests and 2 ignored; `./scripts/dev check`, `git diff --cached --check`, and LSP passed. | Pass | Clear | 0 | None |

## Final validation

Run after all implementation packets and before feature review/publication:

1. `./scripts/dev test`
2. `./scripts/dev check-rust`
3. `./scripts/dev check`
4. `./scripts/dev smoke`
5. `./scripts/dev inspect-ui /players/42?section=overview 1280 800`
6. `./scripts/dev inspect-ui /search 1280 800`
7. `./scripts/dev inspect-ui /my-club 1280 800`
8. Required GitHub Linux and Windows Rust checks, including no-follow/junction/reparse proof.
9. Record the developer-approved gap for native Windows Tauri custom-protocol/CSP registration and CSP proof. Unit parser/response tests, compilation, existing automated Linux/Windows Rust checks, frontend tests, and `./scripts/dev smoke` still run; smoke is Chromium-stub evidence and does not prove native Windows custom-protocol registration or CSP.
10. Host-aware representative-pack calibration with no truncation and path-free evidence for exact host/OS/filesystem context, peak working set, phase totals/timings, mapping/config/entry totals, observed image sizes, first/warm image latency, delegated headroom rationale, and final values.

Inspect captured UI for fixed fallback slots, readable text, row/rail containment, and no stale image after generation replacement. Chromium captures do not prove native Tauri protocol, filesystem, SQLite, or real-pack behavior.

## Documentation impact

Complete during feature reconciliation. The final ledger archives measured production limits and calibration evidence. Update current-state documentation only after implementation proves the final behavior; this planning task changes no current-state document.
