# Player Development Boosts

## Status

Active

## Intent

Add two guarded actions to an individual player profile that change a running FM26 save through the existing BepInEx bridge: **Boost CA** and **Wonderkid Mentality**. Keep the capability narrow, bind every write to the active snapshot's successful bridge scan, verify the live value before and after each write, and reconcile the verified result into the same SQLite snapshot.

This feature is an accepted narrow exception to the current read-only product boundary. It does not create a general-purpose player editor.

## User-visible behavior

- The player profile Overview tab shows a compact **Development boosts** panel with **Boost CA** and **Wonderkid Mentality** actions.
- **Boost CA** uses the player's age in the active snapshot:
  - age 21 or younger: add 5 CA;
  - age 22 or older: add 10 CA;
  - cap the result at the lower of PA and 200;
  - disable the action when age or PA is unknown or CA has reached the lower of PA and 200.
- The CA confirmation names the current and target values and explains that FM redistributes attributes over the following in-game days, sometimes up to one month.
- Boost CA may be used again after a successful reconciliation until CA reaches PA or 200. PA is never changed.
- **Wonderkid Mentality** examines Ambition, Professionalism, and Determination from the active snapshot. Each known value of 10 or lower receives an independent, uniformly selected integer from 11 through 20. Values above 10 and unknown values remain unchanged.
- The mentality confirmation lists the eligible attributes. The action is disabled when none of the three known values is eligible.
- Each action prevents duplicate submission and reports a specific pending state, the verified values on success, or an eligibility, bridge, live-value, or snapshot-sync failure.
- A successful operation updates the exact current snapshot row and any affected current role scores. Search, profile, Planner, Academy, and sanity-list queries then refresh from SQLite.
- A snapshot created before this feature, a snapshot from another bridge scan, a restarted plugin without a fresh scan, or a replaced active snapshot cannot authorize a write. The user receives a **Load Data** instruction instead.

## Invariants

- The WebView can invoke only the two high-level actions with a player UID. It cannot submit a memory address, field name, arbitrary target value, age, boost amount, or random result.
- Rust derives eligibility and the CA increment from the active snapshot. The bridge accepts only the corresponding action-specific request contract.
- The bridge writes only CA, Ambition, Professionalism, and Determination. It never changes PA or another byte.
- CA remains in `1..=200` and never exceeds the player's live PA. Ambition, Professionalism, and Determination remain in `1..=20`.
- CA is a two-byte player-block value. Ambition and Professionalism are raw person-relative bytes. Determination is a player attribute byte encoded as the requested value multiplied by five.
- A successful full dump publishes a plugin-private UID-to-player-location index bound to that dump request ID. Process addresses never enter JSON, SQLite, IPC DTOs, diagnostics, or normal logs.
- A write request must name the source dump request ID persisted with the active snapshot. The bridge rejects an absent, stale, or mismatched index.
- Before a write, the bridge resolves the approved FM layout, targets the live FM process, re-reads the UID and expected current values, including PA for a CA boost, and rejects any mismatch. A PSS snapshot may supply an address hint but is never a write target.
- Scans and writes use one operation gate. The plugin never scans and mutates concurrently.
- The bridge reads every target back before reporting success. A multi-value mentality write validates all targets first and makes a best-effort verified rollback if a later write fails; it never claims atomicity that process memory cannot provide.
- Rust does not hold the SQLite mutex while it waits for the bridge. After success, it updates only the captured snapshot if that same snapshot is still current and still carries the matching source request ID.
- The verified live FM result is authoritative. If FM changes successfully but SQLite reconciliation fails, the error states that FM changed and requires Load Data; it does not pretend that the game write was undone.
- Determination changes cause all persisted current role scores for that player to be recomputed with the existing scoring catalog. CA changes affect existing read-time potential projections without creating another scoring model.
- Dump schema v6 remains unchanged. The request/status protocol remains version 1 and gains additive operation-specific fields that old consumers ignore. Updated status advertises player-boost support so an old plugin fails clearly before a write request.
- Only an explicitly write-validated FM build may advertise or execute boosts. Unknown or read-only layouts fail closed.

## Non-goals

- Numeric inputs, sliders, direct value selection, or a general player editor.
- Arbitrary memory addresses, arbitrary fields, an exported write API, or a developer console.
- Editing PA, visible attributes other than Determination, hidden attributes, other personality values, staff, clubs, finances, contracts, transfers, or save files.
- Undo history, saved presets, configurable increments, configurable thresholds, seeded randomness, or rerolling values already above 10.
- Simulating, accelerating, or predicting FM's later CA-driven attribute redistribution.
- Automatic background scans or a full Load Data cycle after every boost.
- Support for FM editions or builds without an explicitly validated write layout.

## Current-state map

- Relevant components: `bridge/Plugin.cs` polls the file protocol and serializes full-dump scans and player boost operations; `bridge/Scanning/CapADumpPipeline.cs` and `PersonScanResult.cs` discover player UIDs plus person and player-block addresses; `src-tauri/src/features/memory_read/` owns the Rust protocol client; `src-tauri/src/features/player/` owns the profile query and snapshot-bound boost reconciliation; `src/features/player-profile/` and `src/app/routes/players.$uid.tsx` own profile presentation and route composition.
- Data model: `players` stores CA, PA, age, visible attributes JSON, and personality JSON for one current snapshot. `player_role_scores` stores current role scores. The profile DTO already exposes CA, PA, age, Determination, Ambition, and Professionalism.
- Persistence and migrations: migration v16 stores the nullable bridge request ID that produced each newly ingested snapshot. Existing rows remain readable but ineligible for writes until a fresh Load Data.
- Existing behavioral assumptions: player age is computed during bridge extraction against the resolved in-game date and stored in the snapshot. Load Data captures the active app save before scanning and replaces that save's snapshot transactionally.
- Architectural seams: the C# bridge owns process memory; Rust owns trust-boundary rules, SQLite, and bounded IPC; React owns presentation and Query mutation state. Route files may compose cross-feature cache invalidation, while feature code must not import another feature.
- Memory layout: `Fm263Layout` reads CA at player block `+0x264`, Ambition at person `+0x71`, Professionalism at person `+0x74`, and Determination at player block `+0x15F+0x33`. Exact build `26.3.2` supports typed byte and unsigned-16-bit writes for those fields only.
- Bridge protocol: protocol v1 accepts `full-dump` plus the two closed boost operations. The Rust client serializes same-directory request files, requires the advertised capability, sends snapshot-derived expectations, and treats timeouts or unverified rollback as requiring Load Data. Status has additive capability and typed verified-result fields. A successful live scan retains candidates in a process-private index; snapshot-backed scans and plugin restarts do not.
- Scan source: healthy scans use live memory. One guarded PSS VA-clone retry may provide scan data after incomplete live reads, so every mutation must reopen and validate live memory.
- Query invalidation analogue: `AppTopBar` invalidates snapshot, search, player, Planner, and Academy roots after Load Data or save switching.
- Test seams: C# uses `FakeMemoryReader`, request/status serialization tests, and pipeline tests; Rust uses temporary migrated SQLite databases and file-protocol fixtures; the profile route uses `mockIPC` and the Playwright IPC stub.
- Project validation commands: `./scripts/dev bridge-test`, `./scripts/dev test`, `./scripts/dev check`, `./scripts/dev smoke`, and Windows-only `./scripts/dev bridge-install` plus a real FM session.
- Primary risks: stale or wrong player addresses, layout drift, duplicate requests, partial process-memory writes, FM-success/SQLite-failure divergence, active-save or snapshot replacement during a request, and accidental repeated user actions.
- Advisory index status: Repowise was stale at `4ad07c4` during planning, so this map comes from direct source, tests, documents, configuration, and Git evidence.

## Feature architecture

### Bridge ownership

- Add a small write abstraction beside `IMemoryReader`, with typed byte and unsigned-16-bit operations, a Windows implementation, and a fake implementation for deterministic tests.
- Add an internal player-value mutation service that owns address arithmetic, value encoding, precondition reads, exact writes, readback, and best-effort rollback. It receives an already resolved player location and never accepts an address from the file protocol.
- Retain a private player-location index only after a successful full dump. Bind it to the dump request ID and replace it only after another successful dump. A failed dump preserves the prior index alongside the prior app snapshot semantics.
- Extend protocol v1 with two closed operations: Boost CA and Wonderkid Mentality. Keep full-dump request behavior and dump schema v6 unchanged. Add optional typed status fields for operation, boost capability, and verified mutation result.
- Serialize dumps and boosts through the existing plugin worker boundary. Recheck the running game version and a write-support flag before every mutation.
- Use the .NET standard random source behind an injectable seam for independent inclusive `11..20` mentality targets. No new dependency is required.

### Rust and SQLite ownership

- Migration v16 adds one nullable source request ID to `snapshots`. Load Data writes the successful bridge request ID in the same ingest transaction. Existing snapshots receive null and remain valid for reads.
- Add two high-level Tauri commands that accept only a player UID. A service captures the active save, current snapshot, source request ID, and expected player values under a brief lock; computes or validates the requested action; releases the lock; then calls the bridge.
- On a verified bridge success, open one SQLite transaction against the captured current snapshot. Update CA and/or the two JSON maps, recompute all current role scores when Determination changed, and reject a snapshot that was replaced or lost its provenance before commit.
- Return a typed result containing the snapshot ID and exact previous and applied values. Errors identify eligibility, bridge, live-value, or snapshot-sync phases. A sync-phase error after a verified game write explicitly requires Load Data.
- Keep the dump and targeted write paths separate. The targeted transaction updates only values that the bridge just verified; normal game progression still enters ValueScout only through explicit Load Data.

### React ownership

- Add one sibling **Development boosts** Panel to the Overview tab. Reuse Button, Modal, Panel, existing typography, and semantic states; add no new shared primitive.
- Use one primary **Boost CA** action and one secondary **Wonderkid Mentality** action. Each opens one confirmation, preserves focus, disables both actions while either request is pending, and reports an inline `aria-live` outcome.
- Derive preview text and disabled states from the loaded player DTO only. Rust remains authoritative and repeats every rule.
- The profile route coordinates successful cache reconciliation. It refreshes player, search, Planner, Academy, snapshot sanity, and any other snapshot-derived query roots affected by CA, personality, attributes, or role scores.

## Uncertainty register

### Known

- All four offsets and encodings are already used by the current reader and agree with the permitted FM26.3 SuperScout pins.
- The developer confirms that FM26 redistributes visible attributes after a CA change over several in-game days, sometimes up to one month. This feature only writes CA and does not simulate that redistribution.
- The current bridge runs in FM, already discovers both player address bases, and already serializes scans through one background worker.
- Exact build `26.3.2` has successful live proof for the two bridge operations, including verified readback and FM save/reload persistence.
- The controlled proof confirmed that typed writes use the existing process handle without a page-protection change.

### Assumptions

- Repeated Boost CA actions are intentional when the user confirms each new target and CA remains below 200.
- **Wonderkid Mentality** is the accepted working label; changing that copy later does not change the feature contract.

### Decisions

- Deliver a bridge-only risk boundary before application integration. Do not invest in persistence or UI until the bridge passes one controlled live session.
- Keep two action-specific commands instead of a general field/value command.
- Keep protocol v1 and dump schema v6. Use additive optional request/status fields and an explicit boost capability signal for old-plugin detection.
- Persist the producing dump request ID on the snapshot. Expected values alone are not enough to distinguish another app save or bridge session.
- Reconcile verified values into the targeted snapshot instead of forcing a complete scan after each action or patching only the React cache.
- Generate mentality targets inside the C# bridge through an injectable standard-library random source, after live eligibility checks and before writes.
- Set the CA target to `min(current CA + age-based increment, PA, 200)`. Leave unknown age, unknown PA, and unknown mentality values unchanged and ineligible.
- Use confirmation dialogs because both actions change the running game and can persist into the FM save.
- Record the durable write boundary in [ADR-0017](../../decisions/0017-action-specific-fm26-player-boosts.md).

### Unknowns

- Which builds beyond exact `26.3.2` should advertise write support after their own controlled proof.
- Whether FM refreshes every affected screen immediately or only after navigation or time advancement. Bridge readback, not screen repaint timing, defines write success.
- Whether a failed multi-byte operation can always restore earlier bytes. The result contract must expose an unverified partial state when rollback cannot be proved.

### Risks

- A stale candidate could identify another object. Source request binding, UID/facet checks, expected old values, live range checks, and fail-closed build support reduce this risk.
- FM can patch offsets while major/minor remains `26.3`. Write support must be stricter than read layout resolution and disabled until the exact build is validated.
- An operation can change FM and then fail to update SQLite. The service must retain the game change, reject stale local state, and require Load Data.
- A user can switch app saves or replace the snapshot while a request is in flight. The captured snapshot and provenance check must prevent the result from updating another snapshot.
- A request timeout can hide a late success. Expected old values and action-specific targets must make retry safe and force a refresh after an ambiguous result.
- Two or more writes are not a database transaction. The bridge must validate first, verify every result, attempt rollback on failure, and report partial uncertainty honestly.

## Walking skeleton

PR 1 is the walking skeleton. Its controlled Windows proof used the documented force-scan fallback because PR 1 has no Rust or UI caller. On exact build `26.3.2`, a live zero-retry scan advertised boost support; the developer confirmed verified CA and Wonderkid actions, both age increments, and FM save/reload persistence. The bridge emitted no address. Rust, SQLite, and profile work remain outside this PR.

## Delivery plan

### PR 1 — Add safe player boost bridge

**Status:** Merged

**PR ref:** https://github.com/JG1995/fm-valuescout/pull/37

**Merge ref:** `1f4c57754de3585fe71cfc1830963601a8da296c`

**Provisional PR title:** `feat(memory-write): add safe player boost bridge`

**Branch:** `feature/player-boost-bridge`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict `check`

**Feature close-out:** Not required

**CI repair rounds:** 0

**Build-feature-loop profile:** Terra Max — the PR combines process-memory mutation, stale-address defenses, operation serialization, rollback reporting, and a real FM gate.

**Purpose:** Establish and live-validate the smallest safe C# write path before application persistence or UI depends on it. This is a separate merge boundary because a failed live proof removes PR 2 rather than forcing a cross-layer rollback.

**Merge to trunk when:** all bridge tests pass, the normal gate is green, the action protocol exposes no generic write surface, and one controlled FM26 session proves the four values, readback, the CA-to-PA ceiling, PA preservation, and save/reload persistence on an explicitly approved build.

**Depends on:** the completed FM26 reader and scan-hardening foundation on `main`.

#### Commit 1 — Add verified scalar memory writes

**Status:** Completed

**Provisional commit:** `feat(memory-write): add verified scalar writes`

**Work:** Add the typed write seam and an internal player-value service that can validate, encode, write, read back, and restore CA, Ambition, Professionalism, and Determination against a fake or live memory image.

**Out of scope:**

- Request files, plugin worker integration, scan provenance, random mentality rules, Rust, SQLite, React, and live FM validation.
- Any field beyond the four approved values or any public arbitrary-address API.

**Implementation packet:**

- Owners and files: `bridge/Memory/`, a focused mutation module under `bridge/`, `bridge/Layouts/IFmMemoryLayout.cs`, and bridge test fakes/tests.
- Existing patterns to verify: `IMemoryReader`, `WindowsMemoryReader`, `MemoryReaderExtensions`, `FakeMemoryReader`, `Fm263Layout`, `PersonCandidate`, and current range validation in `PersonScanner`.
- Constraints and invariants: typed one- or two-byte writes only; checked address arithmetic; CA and personality ranges; CA target no greater than live PA; Determination ×5 encoding; PA and neighboring bytes unchanged; readback required; rollback uncertainty returned, not hidden; no address logging.
- Dependencies and ordering: use the existing current-process handle and layout offsets. Do not wire the service into `Plugin` until its fake-memory contract is green.

**Implementation profile:** Terra xhigh — the outcome is fixed, but exact byte encoding, checked address arithmetic, readback, and partial-write handling require material local judgment.

**Review profile:** Sol xhigh — an offset, width, or rollback error can corrupt live FM state even though deterministic fake-memory coverage is available.

**Validation:**

- RED: add focused bridge tests for all four encodings, a CA target above PA, unchanged PA/neighbors, invalid values, readback mismatch, and partial-write rollback; run `./scripts/dev bridge-test` and confirm the expected missing-write behavior fails.
- GREEN: run `./scripts/dev bridge-test` and require all bridge tests to pass.
- Gate: run `./scripts/dev check` and require the repository gate to pass without unrelated changes.

**Stop conditions:** Stop and replan if the existing candidate bases do not identify the four live fields, if safe writes require a broader page-protection or raw-pointer mechanism, if readback cannot distinguish failure, or if the service must expose arbitrary addresses or fields to remain testable.

**Review mandate:**

- Verify every offset base, byte width, and encoding against the existing read path.
- Prove CA writes cannot touch adjacent PA.
- Prove the service rejects a CA target above live PA before it writes.
- Prove invalid or stale values fail before any write.
- Trace partial failure and rollback reporting without assuming atomic memory writes.
- Confirm production writes remain behind an interface that fake tests can exercise.
- Confirm no process address enters output, exceptions, or normal logs.

#### Commit 2 — Expose player boost operations

**Status:** Completed

**Provisional commit:** `feat(memory-write): expose player boost operations`

**Work:** Bind successful scan candidates to their dump request ID, add the two closed request operations and typed results, serialize them with scans, generate eligible mentality targets, enforce exact-build write support, and complete the controlled live FM proof.

**Out of scope:**

- Rust protocol callers, snapshot provenance persistence, SQLite reconciliation, Tauri commands, profile UI, or additional edit operations.
- Storing or publishing candidate addresses.

**Implementation packet:**

- Owners and files: `bridge/Plugin.cs`, `bridge/Protocol/`, `bridge/Scanning/CapADumpPipeline.cs`, `bridge/Scanning/PersonScanResult.cs`, layout write capability, and request/status/pipeline tests.
- Existing patterns to verify: request TTL and consume rules, `ScanGate`, status atomic writes, successful-result preservation, PSS retry behavior, game-version detection, and request/status serialization tests.
- Constraints and invariants: retain protocol v1 and full-dump behavior; optional capability/result fields only; index replaced only on successful dump; source request and expected CA/PA match; live reader/writer only; no concurrent scan/write; CA increments restricted to 5 or 10 and capped to live PA and 200; independent inclusive `11..20` random values; clear old-plugin and unsupported-build failures.
- Dependencies and ordering: depends on Commit 1. The live proof must use the final code from this commit and one newly installed DLL.

**Implementation profile:** Terra Max — lifecycle, request idempotency, scan provenance, exact-build gating, PSS-derived hints, random multi-field writes, and partial-failure reporting combine in one settled implementation.

**Review profile:** Sol xhigh — concurrency, stale-pointer, retry, and corruption paths cross a file protocol and a live game process with incomplete automated coverage.

**Validation:**

- RED: add request, status, index-lifecycle, stale-provenance, duplicate-request, deterministic-random, unsupported-build, and scan/write exclusion tests; run `./scripts/dev bridge-test` and confirm the intended missing behavior fails.
- GREEN: run `./scripts/dev bridge-test` and require all bridge tests to pass.
- Gate: run `./scripts/dev check` and require the repository gate to pass.
- Windows live gate: run `./scripts/dev bridge-install`, restart FM26 once with a throwaway or backed-up save, run one fresh Load Data, issue both action requests in the same session, and verify a 21-or-younger player with enough PA headroom receives +5 CA, an older player with enough PA headroom receives +10 CA, CA caps at PA and 200 in automated coverage, PA remains unchanged, each eligible mentality value becomes 11–20, values above 10 remain unchanged, status reports verified results without addresses, and the values survive save/reload. Record only sanitized aggregate evidence.

**Stop conditions:** Stop and replan if any write targets the wrong player or field, PA changes, an expected-value or source-request mismatch can still write, the exact build cannot be pinned, save/reload loses a value, a rollback failure is hidden, or live success requires repeated offset hunting beyond this one bounded session. `$workflow-spike` remains available if the developer prefers to isolate this runtime gate before completing the commit.

**Review mandate:**

- Trace request acceptance, TTL refresh, duplicate handling, terminal status matching, and retries for both operations.
- Verify the private index lifetime matches prior-dump preservation and plugin restart behavior.
- Prove every mutation reopens and validates live memory even after a PSS-backed scan.
- Verify scan/write mutual exclusion and unload cancellation cannot leave a second operation running.
- Confirm exact-build gating is stricter than major/minor read-layout resolution.
- Inspect random eligibility and inclusive bounds with deterministic tests.
- Confirm status and logs contain results but no UIDs beyond the requested one, addresses, raw memory, or machine paths.
- Treat the manual FM gate as required evidence, not an optional confidence statement.

### PR 2 — Add player development boosts

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(profile): add player development boosts`

**Branch:** `feature/player-development-boosts`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Build-feature-loop profile:** Terra Max — the PR combines a migration, a cross-process command, partial FM/SQLite failure handling, score reconciliation, cache invalidation, and two user-facing mutations.

**Purpose:** Turn the validated bridge capability into two bounded profile actions while preserving snapshot provenance and consistent downstream scores. It waits for PR 1 because application code must not depend on an unproved write contract.

**Merge to trunk when:** every commit and final feature validation clears, the exact PR set passes feature review, current-state documentation is reconciled, and the two real player-profile flows succeed against the merged bridge foundation.

**Depends on:** PR 1 merged to `main`; completed snapshot ingest, scoring, player profile, Planner, and Academy foundations.

#### Commit 1 — Bind snapshots to bridge scans

**Status:** Completed

**Provisional commit:** `feat(snapshot): bind snapshots to bridge scans`

**Work:** Add nullable bridge source-request provenance to snapshots and persist the successful dump request ID inside the same transaction that creates the snapshot.

**Out of scope:**

- Player writes, boost eligibility, role-score updates, Tauri mutation commands, and UI.
- Backfilling a request ID for existing snapshots or changing dump schema v6.

**Implementation packet:**

- Owners and files: `src-tauri/src/db/migrations.rs`, `src-tauri/src/features/snapshot/ingest.rs`, `load_data.rs`, snapshot models/queries as needed, and migration/load tests.
- Existing patterns to verify: migration v15 registration, transactional snapshot insert/replace, captured active-save semantics, `DumpRequestResult.request_id`, and current snapshot preservation on ingest failure.
- Constraints and invariants: migration v16 is additive and nullable; existing databases open unchanged; direct fixture ingest may supply null; Load Data captures the completed dump before it releases the scan result, confirms that `status.json` still names the same ready request, then supplies that request ID before commit; no persisted provenance field is added to snapshot DTOs or UI, and the existing Load Data result keeps its scan request ID.
- Dependencies and ordering: starts only after PR 1 merges. The persisted value must match the request whose dump was ingested, not a later status file.

**Implementation profile:** Terra xhigh — the schema change is small, but provenance must remain correct across active-save capture, ingest rollback, and current-snapshot replacement.

**Review profile:** Sol xhigh — existing-data migration and incorrect provenance could authorize a write against the wrong FM scan.

**Validation:**

- RED: add migration and Load Data tests for new, existing, failed-ingest, replaced-snapshot, and captured-save cases; run `./scripts/dev check` and confirm the expected missing provenance behavior fails.
- GREEN and gate: run `./scripts/dev check` and require migration, Rust, TypeScript, Biome, and secret checks to pass.

**Stop conditions:** Stop and replan if migration version 16 is already claimed, if the request ID cannot be inserted atomically with the snapshot, if a failed ingest can persist provenance, or if the design requires placing the request ID in dump schema v6.

**Review mandate:**

- Verify old databases migrate with null provenance and retain their current snapshot.
- Trace the exact request ID from the successful scan result into the intended captured save.
- Prove scan and ingest failures do not alter prior provenance.
- Prove snapshot replacement removes or supersedes the old binding with the old row.
- Confirm no public DTO or UI begins treating the request ID as user data.

#### Commit 2 — Persist verified player boosts

**Status:** Completed

**Provisional commit:** `feat(player): persist verified player boosts`

**Work:** Add the Rust bridge client and two high-level player commands, enforce snapshot-derived rules, reconcile verified values into the same current snapshot, and recompute affected current role scores.

**Out of scope:**

- React UI, arbitrary target values, full-dump refreshes, history, undo, or another scoring path.
- Changing unrelated player fields or snapshot freshness timestamps.

**Implementation packet:**

- Owners and files: `src-tauri/src/features/memory_read/service.rs`, `src-tauri/src/features/player/commands.rs` plus a focused service module, scoring reuse, `src-tauri/src/lib.rs`, and Rust tests with temporary SQLite and fake bridge responses.
- Existing patterns to verify: `load_data` releases the DB lock during bridge work; player query JSON parsing; ingest-time role scoring; typed `LoadDataError`; save/snapshot capture tests; high-level Planner and Academy mutation commands.
- Constraints and invariants: commands accept UID only; age 21 boundary; CA target capped to PA and 200; null age or PA rejected; CA equal to or above PA rejected; null mentality untouched; source request and expected values sent; same snapshot checked after return; CA/personality/attribute JSON and scores update in one transaction; FM-success/sync-failure remains an explicit partial outcome; no database lock during polling.
- Dependencies and ordering: depends on Commit 1 and the PR 1 protocol. Reuse the existing scoring catalog and formula rather than copying role logic.

**Implementation profile:** Terra Max — cross-process polling, idempotency, save/snapshot races, JSON mutation, role-score reconciliation, and irreversible partial success combine despite a fixed outcome.

**Review profile:** Sol xhigh — retry, concurrency, existing-data, and partial-failure paths can otherwise edit FM while leaving misleading local data.

**Validation:**

- RED: add Rust tests for ages 21 and 22, PA headroom below the age increment, CA equal to or above PA, CA 200, unknown age or PA, repeat behavior, mentality eligibility and null preservation, source mismatch, bridge failure, late success, save switch, snapshot replacement, SQLite rollback, role-score recomputation, and FM-success/sync-failure reporting; run `./scripts/dev check` and confirm the expected missing behavior fails.
- GREEN: run `./scripts/dev check` and require all Rust and repository checks to pass.
- Cross-language contract: run `./scripts/dev bridge-test` and require the bridge request/status fixtures to remain aligned.

**Stop conditions:** Stop and replan if the command needs a WebView-supplied target value, if the DB lock must remain held during bridge polling, if a replaced snapshot can be patched, if role scores cannot be updated transactionally with Determination, or if any failure path claims FM was unchanged without proof.

**Review mandate:**

- Trace UID-only IPC through snapshot-derived action rules and the closed bridge request.
- Verify age and value boundaries, including PA headroom below the age increment, CA equal to or above PA, and the cap at 200.
- Prove request timeout or retry cannot apply the same boost twice silently.
- Inspect the no-lock polling interval and the post-return current-snapshot/provenance checks.
- Verify every changed SQLite representation and all affected role-score rows share one transaction.
- Confirm search, profile, Planner, Academy, and sanity data can all be refreshed from the reconciled row.
- Distinguish bridge failure, stale live values, and FM-success/SQLite-failure in typed results.

#### Commit 3 — Add the CA boost action

**Status:** Completed

**Provisional commit:** `feat(profile): add CA boost action`

**Work:** Add the Development boosts panel, Boost CA preview and confirmation, pending/success/error states, and route-owned cache reconciliation for a successful CA boost.

**Out of scope:**

- Wonderkid Mentality, numeric input, direct CA selection, automatic time simulation, or new shared UI primitives.
- Recomputing age, targets, projections, or scores in React.

**Implementation packet:**

- Owners and files: `src/features/player-profile/api/`, `components/`, and `types/`; `src/app/routes/players.$uid.tsx`; `src/app/routes/players.$uid.test.tsx`; `src/testing/snapshot-ipc-mock.ts`; Playwright stub only if the final flow needs it.
- Existing patterns to verify: `PlayerOverviewPanel`, Button, Modal, Panel, player Query keys, route-level cross-feature composition, `AppTopBar` invalidation map, and existing mutation outcome accessibility in Planner or Academy.
- Constraints and invariants: exact snapshot-derived preview only; unknown age or PA, CA equal to or above PA, and CA 200 disabled with a visible reason; one confirmation per click; both boost actions share one in-flight lock once the second arrives; inline phase-specific outcome; no cross-feature imports from player-profile feature code.
- Dependencies and ordering: depends on Commit 2. The route owns broad invalidation; the panel receives bounded callbacks and mutation state.

**Implementation profile:** Luna Max — the backend contract is settled and the UI follows established profile, Modal, Query, and route-composition patterns.

**Review profile:** Sol High — a profile mutation crosses Query caches and can otherwise show stale CA or enable accidental repeated writes.

**Validation:**

- RED: add profile-route tests for age 21 and 22 previews, PA and 200 caps, unknown age or PA, CA equal to PA, confirmation, duplicate-submit prevention, success values, phase errors, focus restoration, and invalidation; run `./scripts/dev test 'src/app/routes/players.$uid.test.tsx'` and confirm the intended missing UI fails.
- GREEN: run `./scripts/dev test 'src/app/routes/players.$uid.test.tsx'` and require the focused suite to pass.
- Gate: run `./scripts/dev check` and require the repository gate to pass.

**Stop conditions:** Stop and replan if React must calculate authoritative targets, if the feature needs a new global store, if route composition cannot invalidate affected feature roots without violating import zones, or if success can leave the visible profile on an old CA value.

**Review mandate:**

- Verify the visible target matches the 21/22 rule and the PA and 200 ceilings without becoming authoritative.
- Confirm unknown age or PA, CA equal to or above PA, and CA 200 have clear disabled explanations.
- Check confirmation, pending lock, keyboard operation, focus restoration, and `aria-live` feedback.
- Trace the returned verified result into profile refresh and all required cache invalidations.
- Confirm the copy explains FM's delayed attribute redistribution without promising a fixed schedule.
- Verify no numeric editor or arbitrary value surface appears.

#### Commit 4 — Add Wonderkid Mentality

**Status:** Active

**Provisional commit:** `feat(profile): add Wonderkid Mentality action`

**Work:** Add the second action to the existing panel, preview eligible mentality attributes, confirm the random `11..20` rule, and show the verified applied values.

**Out of scope:**

- Rerolling values above 10, revealing or configuring a random seed, direct value selection, personality labels, or additional attributes.
- A separate panel, route, or generalized mutation framework.

**Implementation packet:**

- Owners and files: the Development boosts component/API/types from Commit 3, profile route tests, IPC mock, and `e2e/smoke.spec.ts` plus its stub for the final profile action path.
- Existing patterns to verify: attribute/personality lookup and missing-value display, Button and Modal variants, Academy row-action outcomes, profile query refresh, and the shared route invalidation map from Commit 3.
- Constraints and invariants: only known values at 10 or lower are previewed; null and values above 10 remain untouched; result shows each actual random value; both actions remain disabled during either request; no reroll after reconciliation unless a later Load Data again makes a field eligible.
- Dependencies and ordering: depends on Commit 3 and reuses its panel, mutation state, confirmation structure, and route invalidation.

**Implementation profile:** Luna Max — this extends a settled action panel and backend contract with deterministic presentation rules and established test seams.

**Review profile:** Sol High — the reviewer must verify eligibility honesty, random-result display, cache reconciliation, and accessible multi-field feedback.

**Validation:**

- RED: add profile-route tests for each threshold, mixed eligible/ineligible/null values, no-op disablement, confirmation content, exact verified success values, duplicate-submit prevention, errors, and focus restoration; run `./scripts/dev test 'src/app/routes/players.$uid.test.tsx'` and confirm the intended missing behavior fails.
- GREEN: run `./scripts/dev test 'src/app/routes/players.$uid.test.tsx'` and require the focused suite to pass.
- Product suites: run `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke` and require all applicable suites to pass.

**Stop conditions:** Stop and replan if the UI must generate random values, if unknown values become zero or eligible, if the response cannot identify exact applied values, if a second click can silently reroll the same live values, or if the final panel does not fit the profile at 1280×800.

**Review mandate:**

- Verify independent eligibility for Ambition, Professionalism, and Determination at the inclusive threshold of 10.
- Confirm unknown and already-high values remain unchanged in preview, request, result, and SQLite.
- Verify actual returned values, not speculative targets, appear in success feedback.
- Check both actions share duplicate-submission protection and preserve focus and keyboard access.
- Trace Determination changes through refreshed current role scores and downstream queries.
- Confirm the final panel follows the existing design tokens, one-primary-action rule, and no-new-primitive boundary.

## Active work

**PR:** PR 2

**Commit:** Commit 4

### RED proof

Add profile-route coverage for each inclusive threshold, mixed eligible, ineligible, and null values, no-op disablement, confirmation content, exact verified success values, duplicate-submit prevention, errors, and focus restoration. It must fail because the Wonderkid Mentality action does not exist.

### Expected outcome

The existing panel shows a guarded Wonderkid Mentality action beside Boost CA. React presents only eligibility from the snapshot; the bridge chooses the random values and the UI shows its verified result.

### Explicit exclusions

No rerolls for values above 10, random-seed configuration, direct value selection, a separate action panel or route, full-dump refreshes, history, undo, or another scoring path.

## Discoveries and replanning

- 2026-08-09: The developer confirmed that FM26 itself redistributes attributes after a CA edit over several in-game days, sometimes up to one month. The feature therefore writes CA only and treats redistribution as FM-owned behavior.
- 2026-08-09: `CONCEPT.md` currently excludes in-app FM edits. This feature intentionally creates only the two fixed exceptions described here; feature reconciliation must keep transfers and general editing out of scope.
- 2026-08-09: The current snapshot does not store the dump request ID, so expected field values alone cannot safely bind an app save to the plugin's live player index. PR 2 adds nullable request provenance instead of widening dump schema v6.
- 2026-08-09: A complete Load Data currently costs a full scan and multi-gigabyte ingest on the reference save. The accepted design performs a verified targeted snapshot transaction after each boost and retains explicit Load Data for later FM progression.
- 2026-08-09: Repowise was stale at commit `4ad07c4`; planning used direct source, tests, documents, configuration, and Git evidence.
- 2026-08-09: A successful PSS snapshot retry can produce a valid dump but its candidate addresses are not safe for live writes. The bridge therefore retains its mutation index only from a successful live-reader dump and clears it after a successful snapshot-backed dump or plugin restart.
- 2026-08-09: Exact build `26.3.2` is write-validated; the implementation refuses other `26.3.x` builds until each has its own proof.
- 2026-08-09: Wonderkid requests now carry nullable per-field snapshot expectations. `null` is an immutable unknown field, preventing a later live reread from turning it into an eligible write.
- 2026-08-09: Each manual force scan receives a unique provenance ID, and a request that has already expired cannot have its TTL refreshed while bridge work is busy.
- 2026-08-09: The controlled Windows proof used the documented force-scan fallback on exact build `26.3.2`. A live zero-retry scan advertised boost support; the developer confirmed CA and Wonderkid actions, both age increments, and save/reload persistence. This record retains only sanitized pass/fail evidence.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Add verified scalar memory writes | `4826495` | Added internal typed CA, Ambition, Professionalism, and Determination writes with live preconditions, readback, and verified rollback reporting. | Sol xhigh accepted after one local correction to narrow the writer seam to byte/u16 operations. | None |
| PR 1 | Expose player boost operations | `c695556` | Added the closed boost protocol, private live-scan index, exact-build capability, verified result status, and scan/write serialization. | Sol xhigh accepted after one correction round; controlled Windows proof passed. | Manual proof used the documented force-scan fallback. Rust, SQLite, and profile integration remain PR 2. |
| PR 2 | Bind snapshots to bridge scans | `9f0b598` | Added migration v16, atomically persisted source request IDs, captured each completed dump before ingest, and rejected changed request status. | Sol xhigh accepted after a request/dump correlation and transaction-boundary correction. | Snapshot summaries now load before commit so an error rolls back the new binding. |
| PR 2 | Persist verified player boosts | `bc5678c` | Added the closed Rust bridge client, UID-only commands, snapshot-derived eligibility, verified targeted reconciliation, role-score refresh, and explicit uncertain-result recovery. | Sol xhigh accepted after one ledger lifecycle correction. | React boost controls remain Commit 3 and Commit 4 work. |
| PR 2 | Add CA boost action | Pending record | Added the guarded Overview action, snapshot-only preview, confirmation, verified outcome, and route-owned cache invalidation. | Sol High accepted after one architecture-summary correction. | Wonderkid Mentality remains Commit 4 work. |

## Final validation

**Feature review profile:** Sol xhigh — final review must cross-check process-memory safety, request idempotency, migration/provenance, FM-success/SQLite-failure behavior, score reconciliation, cache invalidation, and the two profile actions.

- Run `./scripts/dev format` before the final deterministic gates and inspect any changes.
- Run `./scripts/dev bridge-test` and require all C# bridge tests to pass.
- Run `./scripts/dev test` and require all frontend tests to pass.
- Run `./scripts/dev check` and require Biome, TypeScript, secretlint, Rust format, clippy, and Rust tests to pass.
- Run `./scripts/dev smoke` and require the player-profile boost flow and existing product smoke paths to pass.
- On Windows, run `./scripts/dev bridge-install`, restart FM26 with a throwaway or backed-up save, and perform one fresh Load Data with the final DLL.
- From the profile, verify +5 at age 21, +10 at age 22, CA clamping at PA and 200, the disabled state when CA has reached PA, repeated confirmed boosts while PA headroom remains, PA preservation, mixed mentality eligibility, independent `11..20` results, values above 10 and nulls unchanged, exact success feedback, and synchronized Search/Profile/Planner/Academy/sanity data.
- Verify an old plugin, missing fresh scan, mismatched app save, stale expected value, concurrent request, and unsupported FM build all fail without a write and give an actionable Load Data or plugin-update message.
- Save and reload FM. Confirm the four values persist. Advance in-game time only to observe FM-owned CA redistribution; do not make feature acceptance depend on a fixed redistribution deadline.
- Retain only sanitized counts, versions, and pass/fail evidence. Do not retain names, UIDs, addresses, dump contents, save files, or machine-local paths.
- Run feature review against the exact recorded PR 1 merge ref and PR 2 commit set before documentation reconciliation.

## Documentation impact

- Planning: this ledger, `.wiki/TODO.md`, and [ADR-0017](../../decisions/0017-action-specific-fm26-player-boosts.md) own the accepted feature intent and write boundary.
- PR 1: `.wiki/CONCEPT.md`, `.wiki/ARCHITECTURE.md`, `bridge/README.md`, and `bridge/DUMP_SCHEMA.md` describe the narrow bridge-only write capability without changing dump schema v6.
- PR 2 and final reconciliation: update `.wiki/DESIGN.md` with the Development boosts profile panel, action hierarchy, confirmation, and outcome states.
- At feature completion: condense this ledger into `.wiki/features/completed/`, update TODO state, and reconcile ADR-0017 follow-up references with the exact delivered refs.
