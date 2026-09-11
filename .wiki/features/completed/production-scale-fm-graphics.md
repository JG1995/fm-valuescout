# Production-scale FM graphics

## Status

Ready for final publication

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** 63f607c51a453fcbd4b20790bf1c775989a489395a518feb5d4b5f469d046070

**Current delivery run:** Accepted.

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

- `src-tauri/src/features/graphics/index.rs` owns `GraphicsIndex`, capability traversal, `Limits`, bounded config/image reads, root-relative `Identity`, deterministic `BTreeMap` indexes, and parser diagnostics. Current provisional limits are depth 32, 10,000 configs, 1,000,000 entries, 64 MiB per config, 256 MiB root parser bytes, 2,000,000 records per config, 10,000,000 root records, 8,000,000 attributes per config, 40,000,000 root attributes, 500,000 mappings, and 8 MiB per image. Discovery has the inclusive entry cap. Parsing streams clean-EOF candidate groups. `add_mapping` opens each candidate source parent relative to the already-open config parent, then does a no-follow file metadata check for each record.
- `src-tauri/src/features/graphics/runtime.rs` owns selected-root generations, one serial stopping worker, scan completion, and three per-kind `Lru` caches. Available entries have a 256-entry and 32 MiB bound; missing entries have a 256-entry bound. Lookup reads images outside runtime, transition, and database locks.
- `src-tauri/src/features/graphics/commands.rs` owns the closed `http://graphics.localhost/<generation>/<kind>/<uid>` protocol grammar and bounded raw-image response. `src-tauri/src/lib.rs` registers the asynchronous protocol; resolve IPC is removed.
- `src/features/graphics/components` supplies one lazy, asynchronously decoded protocol image component to Profile, Search, Squad, and My Club. Bundled nationality flags still require `data:` CSP; graphics images do not use it.
- Existing Rust tests cover malformed-config atomicity, deterministic precedence, entry-budget discard, no-follow/junction behavior, source replacement, source sibling containment, image bounds, cache accounting, worker lifecycle, and protocol grammar. `graphics_calibration_harness` currently reports total scan time as each phase time, derives its discovery count from config diagnostics instead of entries, and sends a protocol request for every mapped image to calculate size distribution.
- Supported commands are `./scripts/dev test`, `./scripts/dev check-rust`, `./scripts/dev check`, `./scripts/dev smoke`, `./scripts/dev inspect-ui`, and private-root `./scripts/dev graphics-calibration`. `./scripts/dev package-windows` is installer validation, not representative-pack measurement.

## Feature architecture

1. `GraphicsIndex` remains the pure capability engine. It scans configs in global lexical order, streams bounded compact candidates, and only after each clean EOF groups candidates by source directory under the already-open config-parent capability. It opens and enumerates each such directory once with no-follow regular-file checks, marks valid ordered probes, then folds records in their original order. Per-config failures stay local; root-wide parser truncation may retain only complete lexical predecessors.
2. `GraphicsRuntime` owns a dedicated stdlib worker-control object. Commands reserve the committed generation, replace the one pending target, and return. The worker serially scans and installs only an unstopped matching generation. Lookup snapshots a mapping/cache decision, reads after locks release, and inserts only for the matching generation.
3. One asynchronous `graphics` URI protocol uses the existing runtime. It validates an exact Windows protocol URL and returns raw bytes with explicit MIME, `X-Content-Type-Options: nosniff`, and `Cache-Control: no-store`.
4. One shared `GraphicsImage` component forms the closed URL from safe graphics status, a closed kind, and a positive UID. It uses native lazy loading and asynchronous decoding, then removes itself on error so caller-owned fallback slots remain.
5. A local host-aware calibration command runs an ignored Rust test against a private environment root and emits path-free JSON with exact host/OS/filesystem context. It measures discovery, parsing, and source validation at their actual boundaries; takes image-size metadata during validation through a bounded sample; and makes exactly one cold and one warm protocol request. A later commit records representative evidence and final values without labeling WSL measurements as native Windows.

## Uncertainty register

### Known

- JAY-63 persists only the selected root and already uses generations to reject stale scan completion. This feature retains that ownership model.
- `quick_xml` streams clean-EOF candidate groups. Current source validation still opens a nested source parent and calls `open_file` plus metadata for each mapped record.
- The representative mounted-Windows root has safe aggregates of 1,584,013 entries, 480 configs, 109,610,653 total config bytes, a 45,808,191-byte largest config, and 1,320,006 XML records. The 1,000,000-entry production cap truncates before parsing.
- A temporary test-only envelope of 4,000,000 entries and 2,000,000 mappings did not complete within 1,200 seconds. Its current harness phase counts and phase times are not valid evidence because they infer counts and repeat total scan time.
- The current WSL environment can access the mounted Windows filesystem without `powershell.exe`. It is not native-Windows performance evidence.

### Assumptions

- A sorted temporary vector of candidate references can group source directories and preserve a separate original-order fold without a `HashMap`. The RED proof must demonstrate the capability and ordering contracts before calibration relies on it.
- Metadata from the validation pass can provide a bounded, path-free image-size sample without opening every mapped image through the protocol. The calibration packet must prove its report does not read every mapped image.

### Decisions

- Keep one PR and all existing branch, base, provider, template, required-check, and merge fields unchanged. Scanner, worker, protocol, consumer migration, and calibration share one scale and image-delivery regression surface.
- Retain `BTreeMap`; add no dependency, persistence, watcher, parallel traversal/parsing, interning, path arena, or source-validation `HashMap`. The existing LRU `HashMap`s remain outside this decision.
- Commit 11 replaces per-record source-parent reopening with grouped source-directory validation. It does not select final limits. It must preserve clean-EOF atomicity, local missing-source containment, probe order, original-record precedence, root-relative locators, and no-follow/junction protection at validation and resolution.
- Commit 12 first makes calibration measurements truthful, then uses one completed representative run under a test-only non-production 4,000,000-entry / 2,000,000-mapping envelope to select final limits. It uses approximately twice observed structural workload where reasonable; retains depth 32 and the 8 MiB image limit unless evidence disproves them; and derives available-cache bounds from validation metadata and peak working set. It must not report a final value from the failed run.
- The developer waives native Windows Tauri custom-protocol/CSP manual proof. This remains an accepted validation gap. Unit parser/response tests, compilation, automated Linux/Windows Rust checks, frontend tests, and `./scripts/dev smoke` still run; smoke is Chromium-stub evidence and does not prove native Windows protocol registration or CSP.
- This replan invalidates delivery fingerprint `b6f45d...`. The current delivery run stops until an independent plan review clears the changed packets, the supervisor recomputes a fingerprint, and the developer accepts it.

### Unknowns

- Whether grouped validation completes the representative root within the prior 1,200-second calibration window; truthful per-phase time/counts; peak working set; validation-metadata image-size distribution; and final entry/config/mapping/parser/cache limits.

### Risks

- Grouping validation can accidentally validate before EOF, enumerate one source directory more than once, change extension probe order, or fold mappings in grouping order instead of original record order. Commit 11 owns direct RED proof for these paths.
- Directory-entry metadata can be stale after validation. Resolution must retain the capability-based no-follow open, regular-file metadata check, and image-signature validation, so replacement or a junction cannot become image authority.
- A truthful calibration can still expose a bottleneck outside source validation or a resource requirement beyond the accepted architecture. Commit 12 stops and replans instead of changing persistence, dependencies, indexing structures, or security bounds.
- Worker shutdown and protocol/CSP risks remain as recorded in completed packets; this replan does not reopen those contracts.

## Walking skeleton

Record the reviewed plan and ADR; parse one generated config larger than 8 MiB without committing malformed input; scan more than 1,000,000 entries with the inclusive cap and fail-closed overflow; batch a clean config's sibling-source validation through its parent capability while folding in record order; enqueue one persisted root through the owned worker; serve one portrait through the closed protocol; then collect truthful calibration evidence and set final values.

## Delivery plan

### PR 1 — Scale local graphics delivery

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/production-scale-fm-graphics`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** GitHub protected `main` requires strict status `check`

**Feature close-out:** Current

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

**Status:** Completed

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

**Status:** Completed

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

**Status:** Completed

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

**Status:** Completed

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

#### Commit 11 — Batch config source validation

**Status:** Completed

**Provisional commit:** `perf(graphics): batch source validation`

**Work:** Replace repeated per-record source-parent opens and metadata reads with one no-follow enumeration for each relative source directory under an already-open config parent, then fold validation results in original record order.

**Atomicity:** Source-directory grouping, validation marking, and original-order folding are one source-validation contract. Splitting them would either retain the measured per-record capability work or leave a batch result that can change probe or duplicate precedence without meaningful proof.

**Size assessment:** Expected within the soft target. The change stays in one index module; its security and ordering tests are coupled to the loop.

**Out of scope:** Final production constants, representative-pack calibration, protocol/cache/runtime/UI changes, persistence, dependencies, parallelism, interning, path arenas, and a source-validation `HashMap`.

**Implementation packet:**

- Keep `parse_config` as clean-EOF syntactic staging. Before any source enumeration, reject malformed XML, malformed attributes, and per-config parser breaches exactly as now.
- Normalize each clean candidate's relative source path and closed target after EOF. Keep its original record ordinal and ordered extension probes. Group only valid source-directory candidates through a sorted temporary vector of references; do not use a `HashMap` and do not change the staged vector's record order.
- For each group, open its relative source directory one time from the already-open config-parent capability with no-follow protection. Enumerate it once, accept only regular non-link files, and mark matching ordered probes as valid. Invalid targets or source paths retain their safe diagnostics; a missing or unreadable record remains local to that record.
- Fold the marks through the original candidate order into the existing deterministic maps. Preserve duplicate precedence, logo-over-icon selection, extension probe order (`png`, `jpeg`, `jpg`, `webp`), root-relative locators, mapping-cap behavior, and complete-config prefix behavior. Resolution continues to open the selected locator with no-follow protection, regular-file metadata validation, bounded read, and signature validation.

**Files and responsibilities:**

- `src-tauri/src/features/graphics/index.rs` — grouped source-directory validation, original-order mapping fold, minimal test-only observation seam, and index tests.
- `.wiki/features/active/production-scale-fm-graphics.md` — mechanical commit completion evidence and activation of Commit 12; keep Commit 11 Git ref `Pending record` until Commit 12 records it.

**Behavior and data flow:** A config reaches clean EOF with its candidate vector intact. A sorted reference view batches only source-directory validation against the open config parent. Validation marks return to the unchanged vector, whose original record order alone decides probes, duplicates, and mapping insertion. A later image request revalidates the selected root-relative file before serving bytes.

**Ordered implementation steps:**

1. Add RED tests that observe one source-directory enumeration for many same-directory records and that fail if grouping changes original-order output.
2. Build the no-follow grouped validation pass after clean EOF, including local invalid/missing handling and ordered probe marking.
3. Fold marked candidates in their original order and remove the per-record source-parent open path.
4. Re-run parser atomicity, source containment, precedence, and resolution security proofs before the Rust gates.

**Tests and proof:** Add focused Rust RED/GREEN proof for one nested source directory with many records, a missing sibling beside a valid one, grouped records whose original order controls the duplicate winner, and fixed extension probing. Retain malformed-after-valid-record proof with zero source validation, per-config breach discard, root-relative locator, Linux replacement/no-follow, and Windows junction/reparse proof. The test-only observation must show one enumeration per relative source directory, not per mapping.

**Patterns to verify:** `parse_config`, `ConfigCandidate`, `source_relative_identity`, `source_candidates`, `open_relative`, `open_file`, `read_image`, and the existing source-containment and lexical-precedence tests.

**Constraints and non-goals:** Do not source-check before clean EOF; reopen a source parent from root; retain per-record directory enumeration; permit symlinks, junctions, absolute or escaping locators; reorder mapping insertion; or relax resolution validation.

**Dependencies and sequencing:** Follows Commit 10. Commit 12 depends on this validation pass for representative scale and metadata sampling. It does not alter the existing provisional production limits.

**Validation:** Focused graphics index tests, including the new RED proof; `./scripts/dev check-rust`; `./scripts/dev check`; and required Windows `rust-windows` Check job.

**Stop conditions:** Replan if one no-follow directory enumeration cannot validate the required regular files, a grouped pass needs a new dependency or `HashMap`, any malformed config reaches validation, original-order fold cannot preserve precedence, or Linux/Windows containment proof changes its supported result.

**Review mandate:** Verify source access begins only after clean EOF; trace a nested source directory from open capability through one enumeration to original-order insertion; recalculate probe and duplicate outcomes; reject per-record parent opens, path authority, altered diagnostics, or any relaxation of resolution's no-follow/signature checks.

#### Commit 12 — Calibrate truthful final graphics limits

**Status:** Completed

**Provisional commit:** `perf(graphics): calibrate production limits`

**Work:** Replace false harness phase fields with boundary-owned measurements, use validation metadata rather than protocol fan-out for image-size distribution, run the representative root without truncation, and set final bounded constants from that evidence.

**Atomicity:** Truthful instrumentation, one complete representative run, final-bound tests, and the path-free ledger record form one auditable evidence-to-constant result. Splitting them would leave either misleading calibration output or production constants without their required basis.

**Size assessment:** Expected within the soft target unless boundary instrumentation requires a small shared measurement type. Do not add an abstraction beyond the scanner and ignored-harness need.

**Out of scope:** New user-visible behavior, private assets/paths/logs, protocol fan-out, persistence, watchers, dependencies, `HashMap` conversion, parallel work, or a native-Windows performance claim.

**Implementation packet:**

- Make the scanner measure discovery, config parsing, and source validation at their real boundaries. Report actual discovery entries, parsed config bytes/records/attributes, source records and source directories, mapping/index totals, and separate elapsed time for each phase. Do not derive a phase count from summary diagnostics or reuse total scan time for a phase.
- Use a test-only, visibly non-production calibration envelope of 4,000,000 entries and 2,000,000 mappings. It exists only for the ignored private harness, exceeds the known representative totals, and must never become an application production limit.
- During grouped validation, retain a bounded path-free sample of successful source metadata lengths with its method and count. Report sample distribution from that metadata, not raw image bytes, UIDs, locators, or a request for every mapping. Select one deterministic installed mapping, make exactly one cold and one warm protocol request for it, and fail the harness if either request cannot supply its measurement.
- Run the private representative root once after the harness becomes truthful. Require no truncation. Record only safe aggregates, execution/filesystem context, measurement method, phase data, metadata sample, peak-working-set method/result, two image latencies, final constants, headroom rationale, and validation results in this ledger.
- Set final entry, config, mapping, parser byte/record/attribute, image, available-cache byte/count, and missing-count bounds from the complete run. Use the accepted approximately-two-times structural headroom where reasonable; keep depth 32 and the 8 MiB image limit unless evidence disproves them. Re-run final boundary tests after replacing all provisional wording.

**Files and responsibilities:**

- `src-tauri/src/features/graphics/index.rs` — boundary-owned scan measurements, bounded validation-metadata sample, test-only calibration envelope, final index limits, and final-bound tests.
- `src-tauri/src/features/graphics/runtime.rs` — truthful path-free harness report, exactly two protocol requests, cache-limit tests, and host/working-set reporting.
- `.wiki/features/active/production-scale-fm-graphics.md` — Commit 11 ref, path-free representative evidence, final values, headroom rationale, no-truncation result, and validation evidence.

**Behavior and data flow:** The ignored harness invokes the same capability scanner under test-only calibration limits. Scanner boundaries produce their own counts and durations. Source validation supplies bounded metadata statistics without image reads. The protocol proves one uncached and one cached raw-image request; the safe report then selects limits that still fail closed on later entry overflow.

**Ordered implementation steps:**

1. Add RED tests for distinct phase accounting, actual discovery-entry count, bounded metadata sampling, and exactly two protocol calls.
2. Add the smallest scanner-owned measurements and harness report fields that make those proofs green.
3. Run the complete private representative calibration once under the test-only envelope and stop unless it completes without truncation within 1,200 seconds.
4. Select final constants from the truthful report, update final-bound/cache tests, and record only safe evidence in this ledger.

**Tests and proof:** Prove phase time/counts come from their owning scanner boundaries; discovery count is filesystem entries rather than configs; source validation uses a bounded metadata sample and never protocol-reads every mapping; the harness makes exactly one cold plus one warm request; output and ledger contain no root path, UID, image, locator, or raw log; and the representative run has no truncation. Retain generated >8 MiB and >1,000,000 legacy-threshold proofs, final `<=` entry acceptance, first-beyond whole-index discard, cache accounting, and the developer-approved native-Windows protocol/CSP gap.

**Patterns to verify:** `GraphicsIndex::scan`, `GraphicsSummary`, grouped validation from Commit 11, `graphics_calibration_harness`, `graphics_protocol_response`, `Lru`, the calibration command, and current generated-bound tests.

**Constraints and non-goals:** Do not report the failed run as calibration evidence; infer phase data; read every mapped image through the protocol; persist a sample or raw logs; expose path, UID, asset, image, or locator; guess constants; or label WSL measurements as native Windows.

**Dependencies and sequencing:** Requires Commit 11 and the same private representative root accessible from the current host. The existing native-Windows protocol/CSP gap remains accepted. No later numeric-constant approval is required after the complete truthful run.

**Validation:** Focused graphics index/runtime/harness tests; one private `FM_VALUESCOUT_GRAPHICS_ROOT=<private-root> ./scripts/dev graphics-calibration` run on the current host; `./scripts/dev check-rust`; `./scripts/dev check`; `./scripts/dev smoke`; required Windows `rust-windows` Check job; and `git diff --check`. Run `./scripts/dev inspect-ui` only if calibration changes component behavior.

**Stop conditions:** Replan if the representative root truncates under the test-only envelope, still does not finish within 1,200 seconds, phase evidence identifies a material bottleneck outside this packet, sample collection requires unbounded storage or image reads, final headroom weakens a security invariant or needs excessive resources, or final support requires persistence, a dependency, parallelism, or a different index structure.

**Review mandate:** Trace each phase count and duration to one scanner boundary; verify exactly two protocol requests and bounded metadata sampling; verify path-free report and ledger fields; trace every final constant to safe complete-run evidence and sizing rule; confirm entry overflow remains fail-closed; and reject inferred metrics, private data, guessed values, or unplanned architecture changes.

## Discoveries and replanning

- Commit 11 hit its explicit stop condition. The 1,000,000-entry production cap truncates before parsing the representative root. A temporary 4,000,000-entry / 2,000,000-mapping run did not complete within 1,200 seconds, so it cannot select final values.
- Current clean-EOF staging and config-parent ownership remain sound, but each record opens its source parent and file then reads metadata. Commit 11 now batches validation by relative source directory and folds marks in original order. This is the narrowest source-side change that can address the evidenced bottleneck without persistence, a dependency, or a `HashMap`.
- Commit 10's calibration harness reports false phase timing and discovery count, and it reads every mapped image through the protocol. Commit 12 replaces those outputs before one representative run: scanner-owned phase metrics, bounded validation-metadata image sampling, and exactly one cold plus one warm protocol request.
- The accepted protocol keeps `Cache-Control: no-store`; browser caching must not bypass the bounded Rust cache. The developer-approved native-Windows protocol/CSP validation gap remains unchanged.
- This material packet split invalidates fingerprint `b6f45d...`. The current delivery run stops for independent plan review, supervisor fingerprint recomputation, and developer acceptance. Completed Commit 10 is preserved at `dfc8a9c4805cec9b3532c41381be6cb6a7a521e7`; no private root, UID, asset, image, or raw log is recorded.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Scale local graphics delivery | Commit 1 — Record the approved production-scale plan | `c4bd51150fe97e75170301c31e234561a3108f60` | Recorded the accepted schema-2 ledger, JAY-64 TODO pointer, and ADR-0029 on the authorized feature branch. | `delivery_state.py` and `ledger_state.py` reported runnable; `git diff --cached --check` passed. | Not applicable | Clear | 0 | None |
| PR 1 — Scale local graphics delivery | Commit 2 — Stream bounded config candidates | `b305ac4ea460aec248129b6fa7de7e0006aab326` | Streamed config XML into clean-EOF candidate groups with per-config and root-wide byte, record, and attribute bounds; local breaches discard one config and root breaches retain only complete lexical predecessors. | Graphics index tests passed; `./scripts/dev check-rust` and `./scripts/dev check` passed with 851 Rust tests and 2 ignored; `git diff --cached --check` passed. | Pass | Accepted findings — provisional parser limits still use the production limit name and must be labeled before close-out. | 2 | The initial implementation did not capture RED before code; correction tests reproduced and fixed the byte-precedence defect. |
| PR 1 — Scale local graphics delivery | Commit 3 — Reduce discovery allocation and enforce entry cap | `fbe18777052fdc444d78a390e0d14efd0947643c` | Reworked discovery around one mutable component path and direct lexical component comparison; made the entry cap inclusive and discarded the whole index on first overflow. | Graphics tests passed; `./scripts/dev check-rust` passed with 853 Rust tests and 2 ignored; `./scripts/dev check` and `git diff --cached --check` passed. | Pass | Clear | 0 | None |
| PR 1 — Scale local graphics delivery | Commit 4 — Validate config sources by parent capability | `350a4be30992eed8a31c66289724ed802b94ff65` | Validated clean-EOF candidates through their open config-parent capability and retained valid siblings when another source was missing or unsafe. | Graphics index tests passed; `./scripts/dev check-rust` and `./scripts/dev check` passed; `git diff --cached --check` and Rust LSP passed. | Pass | Clear | 0 | None |
| PR 1 — Scale local graphics delivery | Commit 5 — Rebuild graphics through one stopping worker | `b2ef48b167a6056442e137b92cc8b122678afe45` | Moved startup, choose, and rescan rebuilds to one runtime-owned serial worker with replaceable pending work, generation-safe installation, and non-blocking stop revocation. | Runtime tests passed; `./scripts/dev check-rust` passed with 856 Rust tests and 2 ignored; `./scripts/dev check`, `git diff --cached --check`, and LSP passed. | Pass | Clear | 1 | Close-out correction `5cc2eb59e5f74498013494d7c507d6b537c9d3ce` made rebuilding accurate during startup and after clear. |
| PR 1 — Scale local graphics delivery | Commit 6 — Read graphics outside runtime locks | `d24d6bb126ca15414fc7e8c547a22f8f6be398eb` | Split lookup into a generation-validated locator snapshot, unlocked image read, and generation-rechecked cache insertion while retaining count-bounded caches and resolve IPC. | Focused graphics tests passed; `./scripts/dev check-rust` passed with 857 Rust tests and 2 ignored; `./scripts/dev check`, `git diff --cached --check`, and LSP passed. | Pass | Clear | 0 | None |
| PR 1 — Scale local graphics delivery | Commit 7 — Account for graphics cache bytes | `966cf8dbb86c174f5629089c79915f1dfddf17ca` | Added exact raw-byte accounting and combined byte/count eviction to every per-kind available cache while retaining count-only missing caches. | Runtime tests passed; `./scripts/dev check-rust` passed with 859 Rust tests; `./scripts/dev check`, `git diff --cached --check`, and LSP passed. | Pass | Clear | 0 | None |
| PR 1 — Scale local graphics delivery | Commit 8 — Register closed graphics protocol | `96b043fccdde188ff2226d2c181219520978fbd7` | Registered one asynchronous closed graphics protocol on the managed runtime, added raw validated-image responses with secure headers, and allowed the exact origin while retaining current data-URL consumers. | Protocol tests passed; `./scripts/dev check-rust` and `./scripts/dev check` passed with 863 Rust tests; `./scripts/dev smoke` passed 62 tests; `git diff --cached --check` and LSP passed. | Pass | Clear | 2 | Close-out correction `5cc2eb59e5f74498013494d7c507d6b537c9d3ce` rejected explicit ports, documented browser-side fragments, and added direct stale-generation protocol proof. Native Windows protocol/CSP proof remains the developer-approved validation gap. |
| PR 1 — Scale local graphics delivery | Commit 9 — Migrate graphics consumers to one component | `7055ed098d9ba16a8a24793576bd6ed1ec21ed0a` | Migrated Profile, Search, Squad, and My Club to one lazy protocol image component; removed resolve IPC, result queries/types/mocks, and base64 conversion while retaining `data:` CSP for bundled nationality flags. | `./scripts/dev test` passed 1,020 tests; `./scripts/dev check-app`, `./scripts/dev check-rust`, `./scripts/dev check`, and `./scripts/dev smoke` passed; three required 1280×800 UI inspections passed; absence search, diff check, and LSP passed. | Pass | Clear | 2 | Native Windows protocol/CSP proof remains the developer-approved gap; Chromium inspection confirmed fallback slots and containment only. |
| PR 1 — Scale local graphics delivery | Commit 10 — Add private graphics calibration harness | `dfc8a9c4805cec9b3532c41381be6cb6a7a521e7` | Added the stable private-root calibration command and ignored host-aware harness with path-free JSON, protocol timings, image-size distribution, and peak-working-set reporting; documented its contract. | Disposable-root calibration emitted one JSON line; missing and unreadable roots failed without echoing the supplied value; `./scripts/dev check-rust`, `./scripts/dev check`, `git diff --cached --check`, and LSP passed. | Pass | Accepted findings — phase fields repeat total scan time and discovery count does not measure filesystem entries; Commit 12 must replace them before selecting limits. | 0 | No representative private pack ran and no final value was selected in this commit. |
| PR 1 — Scale local graphics delivery | Commit 11 — Batch config source validation | `80367d689e515bf7ee61073ac19b26f3dc484dd4` | Grouped clean-EOF candidates by relative source directory, enumerated each capability once with no-follow regular-file checks, and folded valid probes in original record order. | Graphics index tests passed; `./scripts/dev check-rust`, `./scripts/dev check`, `git diff --cached --check`, and LSP passed. | Pass | Clear | 1 | Representative calibration exposed quadratic entry-to-probe matching; bounded correction `409d53a7c59f04b5fe0c944673894471c1f9f268` replaced it with reviewed sorted binary-range matching while preserving the packet contract. |
| PR 1 — Scale local graphics delivery | Commit 12 — Calibrate truthful final graphics limits | `137515592727bed3b7e66f838bfb8a32644fd72c` | Added scanner-owned discovery, parser, and source metrics; bounded deterministic successful-source metadata sampling; a test-only calibration envelope; exactly one cold and warm protocol request; and calibrated production index/cache limits. | Graphics tests passed (49); the representative run completed without truncation under 1,200 seconds; `./scripts/dev check-rust` and `./scripts/dev check` passed with 867 Rust tests and 3 ignored; `./scripts/dev smoke` passed 62 tests; `git diff --cached --check` and Rust LSP passed. Safe representative evidence: Linux x86_64 native-Unix execution on a mounted-Windows filesystem; 1,584,012 entries; 480 configs; 109,610,653 parser bytes; 1,320,006 records; 2,639,052 attributes; 635,489 source records across 5 directories; 635,159 mappings; a 4,096-entry smallest-successful metadata sample (1,511 minimum, 11,736 p50, 12,315 p95, 12,353 maximum bytes); discovery/parser/source times 4,518/8,374/327,544 ms; 1,021,104,128-byte peak working set by `linux-proc-vmHWM`; and 4/0 ms cold/warm requests. | Pass | Clear | 1 | Final bounds are depth 32, 960 configs, 3,200,000 entries, 96 MiB per config, 220 MiB root parser bytes, 2,700,000 records and 5,300,000 attributes at each scope, 1,300,000 mappings, 8 MiB images, and per-kind caches of 512 available entries/64 MiB plus 512 missing entries. Structural limits use approximately twice observed workload where reasonable; depth/image security bounds remain unchanged. Close-out correction `5cc2eb59e5f74498013494d7c507d6b537c9d3ce` optimized bounded sampling while preserving the calibrated result. Native-Windows protocol/CSP proof remains the accepted gap. |

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
10. One host-aware representative-pack calibration with no truncation under the test-only 4,000,000-entry / 2,000,000-mapping envelope; path-free evidence for exact host/OS/filesystem context, scanner-owned discovery/config/source totals and timings, bounded validation-metadata image-size sample, peak working set, exactly one cold and one warm image latency, delegated headroom rationale, and final values.

Inspect captured UI for fixed fallback slots, readable text, row/rail containment, and no stale image after generation replacement. Chromium captures do not prove native Tauri protocol, filesystem, SQLite, or real-pack behavior.

## Documentation impact

Complete during feature reconciliation. The final ledger archives measured production limits and calibration evidence. Update current-state documentation only after implementation proves the final behavior; this planning task changes no current-state document.
