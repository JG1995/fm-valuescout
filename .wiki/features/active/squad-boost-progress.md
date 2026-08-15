# Squad Boost Progress and Feedback

## Status

Validation

## Intent

Make both squad-wide development actions visibly advance through their frozen player cohort and finish with concise feedback that does not move the Squad overview actions. This feature implements [JAY-12](https://linear.app/jaycount/issue/JAY-12/add-progress-tracking-to-squad-boost-actions) and its child [JAY-13](https://linear.app/jaycount/issue/JAY-13/keep-squad-boost-outcome-feedback-layout-stable) in one PR.

## User-visible behavior

- After the user confirms **Boost all CA** or **Make all Wonderkids**, the existing Modal first shows that the squad is being prepared, then shows determinate `processed / total` progress.
- Progress advances after every player reaches an updated, skipped, or failed outcome.
- The Modal remains open and neither squad action can be submitted again while the command is pending.
- Normal completion reports one concise summary for the latest action, for example `42 processed — 35 updated, 6 skipped, 1 failed.`
- Recovery-required completion reports only the observed outcomes, states that processing stopped, and requires Load Data before another boost.
- Final feedback uses one reserved Squad overview status region. It does not add separate result blocks below the two action buttons or move the header actions when feedback appears.
- A new current snapshot clears the visible progress and outcome through the existing snapshot-bound component and mutation state.

## Invariants

- Rust remains the source of truth for the frozen squad cohort, total player count, processing order, and outcome counters.
- `processed` always equals `updated + skipped + failed`; it never includes the player whose ambiguous result triggered a recovery-required stop.
- The first progress payload is `0 / total` after the cohort is captured. Every later payload follows a terminal player outcome and counters never decrease.
- Existing sequential execution, shared player-boost gate, per-player commit, active-context verification, and recovery latch remain unchanged.
- A frontend progress listener or channel-delivery failure is observational only. It must not interrupt, retry, reclassify, or otherwise change an FM write or reconciliation result.
- Skipped, failed, and recovery-stopped players are never presented as updated.
- Query invalidation remains route-owned and runs only after the command resolves successfully with its final result.
- Pending locks, confirmation behavior, error retention, focus restoration, and snapshot replacement remain accessible and context-safe.

## Non-goals

- Load Data or memory-scan progress.
- Changes to the C# bridge, file protocol, dump schema, SQLite schema, or migrations.
- Parallel, batched, resumable, cancelable, or retrying player writes.
- Progress persistence, global Tauri events, background notifications, or progress history.
- Changes to profile-level Boost CA or Wonderkid Mentality actions.
- Generic changes to the shared `Panel` or `Modal` primitives.

## Current-state map

- Relevant components:
  - `src-tauri/src/features/player/commands.rs` captures the current squad cohort, processes UIDs sequentially, and returns final updated, skipped, failed, and recovery fields.
  - `src/features/squad/api/boost-squad-current-ability.ts` and `boost-squad-wonderkid-mentality.ts` invoke the two commands without arguments.
  - `src/app/routes/planner.tsx` owns both TanStack Query mutations, their snapshot binding, the shared pending/recovery gate, and cross-feature invalidation.
  - `src/features/squad/components/squad-player-boost.tsx` owns both confirmation Modals and currently renders a separate result block below each action.
  - `src/features/squad/components/squad-overview-panel.tsx` places the action group in the `Panel` header and has no dedicated feedback slot.
  - `src/testing/squad-ipc-mock.ts`, `src/testing/setup.ts`, `src/app/routes/planner.test.tsx`, `e2e/tauri-ipc-stub.ts`, and `e2e/smoke.spec.ts` cover the browser-side command and interaction seams.
- Data model: `SquadPlayerBoostResultDto` / `SquadPlayerBoostResult` is the final summary contract. There is no progress payload today.
- Persistence and migrations: no new persistence is needed. Existing snapshot player rows, potential-score invalidation, and the v20 recovery latch remain authoritative.
- Existing behavioral assumptions: `capture_squad_player_boost_cohort` freezes the ordered UID set before the loop; preparation skips are safe terminal outcomes; proven live-value failures count as failed and continue; ambiguous bridge, context, or reconciliation failures stop and return a recovery-required partial result.
- Architectural seams: Tauri 2.11 already supports per-invocation typed IPC channels. A command-scoped `tauri::ipc::Channel<T>` maps directly to the existing frontend `Channel<T>` without a global event name or listener lifecycle.
- Project validation commands: `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke` cover frontend behavior, Rust behavior, static gates, and browser layout/interaction.
- Primary risks: progress accounting can drift from final counters; notification failure can accidentally become a mutation failure; late events can cross a snapshot or invocation boundary; frequent live-region updates can become noisy; and final feedback can still alter layout if space is not reserved.

## Feature architecture

Rust adds a serializable squad progress DTO and reports immutable counter snapshots from the existing sequential loop. The two public Tauri commands receive a command-scoped progress channel, while the private orchestration remains testable through a small callback seam. Both action-specific wrappers use the same reporter and loop; no progress logic enters the bridge or persistence layers.

Each frontend API adapter creates a typed Tauri `Channel`, assigns the invocation-specific callback, and passes it as `onProgress`. The route continues to own mutation and invalidation behavior. Each action component owns only its active Modal progress state, is keyed to the current snapshot as today, and ignores progress outside its current invocation.

The Squad route selects the latest action for final feedback. `SquadOverviewPanel` gains a Squad-specific status slot rendered in reserved content space below the panel header. The shared feedback presentation derives `processed` from the final counters, uses recovery-specific copy when needed, and supplies the focus fallback when a recovery result leaves both action triggers disabled.

No ADR is required: this adds observation to the already accepted ADR-0018 sequential mutation boundary without changing persistence, the bridge protocol, concurrency, recovery, or a public external API.

## Uncertainty register

### Known

- The cohort and its total are available before the first player iteration.
- Both squad actions already share the same sequential helper, mutation gate, context checks, and final result DTO.
- Tauri 2 supports typed per-invocation channels for streaming command payloads, and the installed frontend and Rust versions are already on the required API generation.
- Existing unit and smoke tests cover confirmation, pending locks, final counts, recovery, invalidation, and focus behavior.

### Assumptions

- Channel messages from one command invocation are delivered in send order before or alongside that invocation's final resolution.
- The supported desktop layouts have room for a reserved two-line feedback region without reducing the squad table below its usable minimum.
- Per-player bridge latency is slow enough that polite progress announcements are useful rather than excessively rapid.

### Decisions

- JAY-12 is squad-only. Load Data progress is deferred because it would require a separate memory-reader/bridge execution design.
- JAY-12 and JAY-13 ship in one PR because they modify the same action, mutation, test, and feedback surfaces.
- Use a command-scoped Tauri channel, not polling or global events.
- Emit an initial `0 / total` payload and a payload after every updated, skipped, or failed outcome.
- Treat progress delivery as best-effort telemetry; final command results remain authoritative.
- Keep progress in the confirmation Modal and final outcome in one shared, reserved Squad overview region.
- Retain the current detailed error inside an open Modal. Use compact standard recovery copy in the shared region instead of appending the potentially long recovery detail to the toolbar area.

### Unknowns

- The exact native WebView announcement cadence and visual density need confirmation through the existing smoke surface and, if necessary, one manual desktop check during implementation. This does not block the first commit.

### Risks

- Incrementing before an outcome is terminal would overstate progress after a recovery stop.
- Returning a channel send error would incorrectly turn feedback failure into an FM mutation failure.
- Reusing progress state after a snapshot replacement would misrepresent a different current cohort.
- An unbounded recovery or error message could defeat the reserved layout.

## Walking skeleton

One confirmed squad action captures its frozen cohort in Rust, sends `0 / total`, sends one counter snapshot after each terminal player outcome, renders those updates in the still-open confirmation Modal, returns the existing final result, invalidates current-only readers, closes the Modal, and presents the final counters. Commit 1 proves this path while retaining the current per-action final result location; Commit 2 moves that same final result into the stable shared region.

## Delivery plan

### PR 1 — Add squad boost progress feedback

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Provisional PR title:** `feat(squad): add boost progress feedback`

**Branch:** `feature/squad-boost-progress`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required-check rule:** `strict_check` (the aggregated GitHub Actions job is `Check / check`)

**Feature close-out:** Not run

**CI repair rounds:** 0

**Purpose:** Deliver JAY-12 and JAY-13 as one coherent squad action improvement without changing the established FM write boundary.

**Merge to trunk when:** both actions stream truthful progress, final feedback is layout-stable and accessible, all existing recovery/context contracts remain green, feature review and documentation reconciliation clear, and the required check passes.

**Depends on:** merged Squad Workspace behavior and ADR-0018; the current `main` branch implementation of the guarded squad boost commands.

#### Commit 1 — Stream squad boost progress

**Status:** Completed

**Git ref:** `b656428` (`feat(squad): stream boost progress`)

**Provisional commit:** `feat(squad): stream boost progress`

**Work:** Add a typed per-invocation progress path from the frozen Rust cohort to the active confirmation Modal for both squad actions.

**Out of scope:**

- Moving or rewriting final result feedback.
- Changing the cohort, mutation, recovery, persistence, or bridge contracts.
- Load Data progress.

**Implementation packet:**

- Add one shared squad progress contract with `processed`, `total`, `updated`, `skipped`, and `failed` counters.
- Preserve the existing private boost orchestration and its test helpers by introducing a reporter seam that can use a no-op in unaffected tests and a Tauri channel in public commands.
- Keep channel delivery best-effort and separate from the command's `Result` path.
- Let each action component own only the progress for its currently open, snapshot-keyed Modal; keep route-owned mutation and invalidation responsibilities unchanged.

**Files and responsibilities:**

- `src-tauri/src/features/player/commands.rs` — define `SquadPlayerBoostProgressDto`; accept an `on_progress` channel in both public squad commands; report initial and terminal-outcome snapshots from `execute_squad_player_boost_with`; add focused progress accounting tests without weakening existing recovery tests.
- `src/features/squad/types/squad-player-boost.ts` — add the camel-case frontend progress type matching the Rust DTO.
- `src/features/squad/api/boost-squad-current-ability.ts` — create the typed `Channel`, attach the supplied callback, and invoke `boost_squad_current_ability` with `onProgress`.
- `src/features/squad/api/boost-squad-wonderkid-mentality.ts` — mirror the same command-scoped channel contract for Wonderkid Mentality.
- `src/app/routes/planner.tsx` — pass each action's invocation-specific progress callback through its existing mutation while retaining snapshot ID variables and query invalidation.
- `src/features/squad/components/squad-player-boost.tsx` — reset progress when confirmation opens; show preparing state until the first payload; render native determinate progress and visible `processed of total processed` copy while pending; retain the current final outcome location for this commit.
- `src/testing/squad-ipc-mock.ts` and `src/testing/setup.ts` — retain the passed channel in mock arguments and provide deterministic helpers that send initial, intermediate, and final progress before resolving the command.
- `src/app/routes/planner.test.tsx` — prove visible progress for both actions, pending locks, terminal counter semantics, zero-player handling, and snapshot replacement cleanup while preserving existing final result and invalidation assertions.
- `e2e/tauri-ipc-stub.ts` and `e2e/smoke.spec.ts` — teach the browser stub to deliver channel messages and prove a confirmed action visibly advances before completion.
- `.wiki/ARCHITECTURE.md` — record the implemented command-scoped progress channel on the squad boost path once the code makes it current state.
- `.wiki/DESIGN.md` — record determinate Modal progress for squad development once implemented, without describing Commit 2 feedback placement early.

**Behavior and data flow:**

- Confirmation starts the existing route mutation and supplies a callback to its action-specific API adapter.
- The adapter creates one `Channel<SquadPlayerBoostProgress>`, assigns `onmessage`, and passes it as `onProgress` to the existing command.
- After the shared gate is acquired and `capture_squad_player_boost_cohort` returns, Rust emits `processed: 0` and `total: player_uids.len()`.
- A skippable preparation outcome increments `skipped`; a verified reconciliation increments `updated`; a proven no-write live-value outcome increments `failed`. Each branch then increments `processed` through the invariant-derived counter sum and emits one new snapshot before the next UID.
- An ambiguous preparation, context, bridge, or reconciliation error returns the existing recovery-required partial result without counting the current UID. It emits no false terminal progress event.
- Failure to send a payload is logged at most as diagnostic information and processing continues unchanged.
- The Modal shows an indeterminate preparing state before the initial event, determinate native progress afterward, and stays non-dismissible while pending. A `0 / 0` cohort reports zero players without constructing an invalid progress maximum.
- The existing final result remains authoritative, closes the Modal on success, and drives the current result block for this commit.

**Ordered implementation steps:**

1. Add RED Rust assertions for the initial payload, ordered updated/skipped/failed payloads, and recovery-stop accounting.
2. Add RED route tests that expect both API adapters to receive and render mocked progress while the command remains pending.
3. Introduce the shared DTO and callback reporter, then connect both Tauri commands with best-effort channel sends until the Rust proofs are GREEN.
4. Connect typed frontend channels through the existing mutations and render accessible Modal progress until the route proofs are GREEN.
5. Extend the browser IPC stub and smoke assertion for an observable intermediate progress state.
6. Update current-state architecture and design text, format only touched paths, and run commit validation in order.

**Tests and proof:**

- Rust RED: collect reporter payloads during a mixed cohort and expect `0 / total` followed by exact monotonic counter snapshots. The old helper has no reporter and cannot satisfy this assertion.
- Rust boundaries: a recovery-triggering UID is absent from `processed`; skipped preparation counts; a failed channel reporter does not alter bridge call order or the final result; an empty cohort emits `0 / 0` and returns zero counters.
- React RED: the pending Modal must show the mocked initial count and at least one later count before resolution; the existing code only shows a loading button.
- React boundaries: cover both actions, pending cross-action locking, no stale progress after current snapshot replacement, and final invalidation exactly once per affected query root.
- Browser proof: pause the stubbed command after an intermediate channel payload, assert the visible value, resolve it, and assert the Modal closes.

**Patterns to verify:**

- Keep the frozen cohort, shared gate, and recovery paths in `execute_squad_player_boost_with` as the source of truth.
- Match the existing camel-case DTO convention and `invokeCommand` wrapper.
- Use Tauri's existing `Channel` API rather than a global event listener.
- Preserve the Modal's pending dismissal guard and the snapshot-based component keys in `planner.tsx`.

**Constraints and non-goals:**

- No dependency, protocol, persistence, schema, or migration changes.
- No progress estimate by elapsed time and no client-derived total from the virtualized Squad table.
- Do not debounce or drop terminal progress payloads; correctness is more important than animation.
- Do not expose the long bridge or recovery diagnostic as progress copy.

**Dependencies and sequencing:**

- Starts from current Squad Workspace and player boost recovery contracts.
- Commit 2 depends on this commit's typed progress and Modal behavior but may change only the final feedback placement and copy.

**Validation:**

1. `./scripts/dev test src/app/routes/planner.test.tsx` — focused route and IPC behavior passes.
2. `./scripts/dev check` — frontend, TypeScript, Rust formatting/lint/tests, and secrets gate passes.
3. `./scripts/dev smoke` — browser progress interaction passes.

**Stop conditions:**

- Tauri's installed channel API cannot carry the typed payload as one command argument without changing the bridge/file protocol or adding a global listener.
- Progress send failure cannot be isolated from the mutation result.
- Accurate progress would require changing the frozen cohort or counting before the existing source-of-truth query.
- Snapshot replacement can leave visible progress from the prior component instance.

**Review mandate:**

- Verify `processed == updated + skipped + failed` for every emitted payload.
- Verify no recovery-triggering ambiguous outcome is counted as terminal.
- Verify notification failure cannot stop, retry, or reclassify an FM write.
- Verify both commands use a per-invocation channel with no leaked global listener.
- Verify the shared gate, bridge-call order, per-player commit, and context checks are unchanged.
- Verify progress is accessible and reset at confirmation, completion, error, and snapshot replacement boundaries.

#### Commit 2 — Stabilize squad boost feedback

**Status:** Completed

**Git ref:** Pending checkpoint hash

**Provisional commit:** `feat(squad): stabilize boost feedback`

**Work:** Replace the two per-action final result blocks with one concise, accessible, reserved status region for the latest squad boost action.

**Out of scope:**

- Progress transport or counting changes.
- Generic `Panel`, `Modal`, or notification-system changes.
- Recovery behavior, retries, or additional mutation actions.

**Implementation packet:**

- Keep pending progress inside the active confirmation Modal from Commit 1.
- Let the route identify the latest opened squad boost action and derive only that mutation's current-context result or error.
- Add a Squad-specific feedback slot to `SquadOverviewPanel`, reserve enough supported-desktop height for compact normal or recovery feedback, and use it as the fallback focus target.
- Remove the outcome blocks beneath individual action buttons and centralize final copy in one presentation path.

**Files and responsibilities:**

- `src/app/routes/planner.tsx` — track the latest opened squad boost action, derive its current-snapshot result/error, reset the relevant mutation on confirmation, create the shared focus target, and pass one feedback node into the overview panel.
- `src/features/squad/components/squad-player-boost.tsx` — remove per-button final outcome rendering; export or retain one shared feedback presenter; accept the shared fallback-focus callback; preserve in-Modal errors and Commit 1 progress.
- `src/features/squad/components/squad-overview-panel.tsx` — add a `status`/feedback prop and render a reserved, Squad-specific region below the fixed header in both populated and empty-cohort states without changing shared `Panel`.
- `src/app/routes/planner.test.tsx` — prove one latest-action outcome, exact normal/recovery copy, no per-button status blocks, recovery focus fallback, errors, and context replacement.
- `e2e/smoke.spec.ts` — compare the squad header action positions before submission and after final feedback; verify concise copy and one shared status region at supported desktop size.
- `.wiki/DESIGN.md` — replace the current general result wording with the implemented Modal progress and shared reserved feedback behavior.

**Behavior and data flow:**

- Opening either confirmation marks that action as latest and clears its prior mutation state, matching today's reset timing.
- Pending progress remains local to that Modal. Command success closes it; a rejected command keeps the error inside it until the user closes or retries.
- Once the Modal closes, the route selects only the latest action's current-snapshot result or error and renders it in the reserved overview feedback region.
- Normal copy derives processed from the three final counters. Recovery copy uses the same observed sum, states that processing stopped, and directs the user to Load Data without appending the unbounded recovery detail.
- The shared region owns `aria-live`, keyboard focus fallback, and a stable minimum block size. The action group contains buttons only, so feedback cannot resize it.
- If a new snapshot becomes current, existing variable-to-snapshot guards hide prior feedback and the keyed action components discard Modal progress.

**Ordered implementation steps:**

1. Add RED route assertions for one shared latest-action status, exact compact summaries, and recovery focus when both triggers are disabled.
2. Add a RED browser assertion that the action buttons retain their pre-action position and dimensions after final feedback appears.
3. Introduce route-owned latest-action selection and the Squad overview feedback slot; move existing outcome presentation without changing mutation behavior.
4. Tighten normal, recovery, and error copy and focus behavior until the focused tests are GREEN.
5. Update the smoke stub/assertions and current-state design text, then format touched paths.
6. Run affected tests, the full check, and smoke in the recorded order.

**Tests and proof:**

- React RED: after one action completes, exactly one named status region reports `3 processed — 2 updated, 1 skipped, 0 failed.` and neither action wrapper owns a result block.
- Latest-action boundary: completing CA, then opening/canceling or completing Wonderkid, never displays two accumulated outcomes.
- Recovery boundary: a partial result reports only the sum of final counters, states `Stopped before all players were processed`, requires Load Data, disables both actions, and receives focus after the Modal closes.
- Error boundary: a rejected command stays in the open Modal; after dismissal, the shared region can report the latest action error without exposing unrelated state.
- Context boundary: replacing the current snapshot removes old feedback and prevents a late prior-context result from becoming visible.
- Browser layout proof: the bounding boxes of both squad boost triggers are unchanged before the Modal opens and after the final shared feedback renders at the supported desktop viewport.

**Patterns to verify:**

- Keep `planner.tsx` as the composition root for cross-feature invalidation and current-context guards.
- Follow existing `Modal.fallbackFocusTo` behavior for a trigger that becomes disabled after recovery.
- Use existing typography, success, warning, and error tokens; no new primitive or token is needed.
- Keep `SquadOverviewPanel` responsible only for Squad overview composition, not mutation state.

**Constraints and non-goals:**

- No toast, global notification store, animation, or generic status primitive.
- Do not display stale results from the non-latest action.
- Do not duplicate the recovery message in both Modal and shared feedback.
- Preserve all current query invalidation and recovery latch behavior.

**Dependencies and sequencing:**

- Requires Commit 1's progress callback and Modal state contract.
- Completes the sole PR and triggers feature-level validation, review, and documentation reconciliation.

**Validation:**

1. `./scripts/dev test src/app/routes/planner.test.tsx` — focused status, focus, recovery, context, and mutation behavior passes.
2. `./scripts/dev check` — complete commit gate passes.
3. `./scripts/dev smoke` — progress, stable-layout, and final-feedback browser flows pass.

**Stop conditions:**

- A reserved Squad-specific region cannot keep the supported desktop header stable without changing the shared `Panel` contract.
- Moving focus to shared feedback breaks Modal restoration or leaves focus on a disabled trigger.
- Latest-action selection can surface data or errors from a non-current snapshot.
- Concise recovery copy cannot remain truthful without changing the final Rust result contract.

**Review mandate:**

- Verify only the latest action contributes final feedback and context guards still apply.
- Verify final processed counts are derived from authoritative result counters.
- Verify the action header does not move or resize at supported desktop widths.
- Verify normal, recovery, and error semantics use the correct accessible live-region behavior.
- Verify recovery closes the Modal, focuses a valid shared target, disables both actions, and requires Load Data.
- Verify no generic UI primitive or unrelated Squad behavior changed.

## Active work

**PR:** PR 1 — Add squad boost progress feedback

**Commit:** Implementation complete; publication boundary reached

### RED proof

Commit 2's focused Planner route assertions cover one shared latest-action status, exact normal and recovery copy, stable action placement after feedback, focus fallback when recovery disables both actions, Modal-only errors before dismissal, and current-snapshot replacement cleanup. The browser smoke proof compares action bounds before and after final feedback.

### Expected outcome

Both squad commands retain the Commit 1 progress contract while final feedback lives in one reserved Squad overview region that reports only the latest current-context action without moving the action header.

### Explicit exclusions

Do not change progress transport or counting, modify Load Data, touch the bridge or database, or change shared `Panel`/`Modal` primitives.

## Discoveries and replanning

- 2026-08-15: JAY-8 remains open as a separate additional player-profile attributes redesign. PR #39 is current-state context, not completion evidence, and JAY-8 is not part of this squad boost feature.
- 2026-08-15: Load Data progress was removed from JAY-12 because the current request crosses the separate memory-reader/bridge execution boundary. The feature is limited to the already sequential squad boost loop.
- 2026-08-15: JAY-13 became a child of JAY-12 and remains in the same PR because both issues share the same action, mutation, feedback, and validation surfaces.
- 2026-08-15: Official Tauri 2 documentation confirms a command-scoped typed Channel is the native progressive-payload mechanism; no dependency or global event architecture is needed.
- 2026-08-15: The first checkpoint was moved to `feature/squad-boost-progress` from `origin/main` to match the delivery plan; the unrelated `.wiki/TODO.md` edit remains outside the feature commits.
- 2026-08-15: Commit 1 review required the frontend IPC fixtures to keep streamed progress and final result counters on the same two-player cohort; both pending action mocks now assert the aligned terminal payload.
- 2026-08-15: Commit 2 review accepted the shared feedback slot, latest-action/context guards, Modal error boundary, recovery focus fallback, stable-layout browser proof, and current validation evidence.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 | Commit 1 — Stream squad boost progress | `b656428` | Typed Rust channel progress, frontend Modal progress, route/mocked IPC coverage, and current-state docs | Sol Medium re-review: Accept; focused route suite 71/71; repository check and smoke pass | Delivery branch corrected before commit; no scope deviation |
| PR 1 | Commit 2 — Stabilize squad boost feedback | Pending checkpoint hash | Shared latest-action feedback slot, compact authoritative summaries, Modal error boundary, recovery focus fallback, snapshot cleanup, and stable-layout smoke proof | Fresh Sol Medium review: Accept; focused route suite 74/74; repository check and smoke pass | None |

## Final validation

1. `./scripts/dev test src/app/routes/planner.test.tsx` — 74 focused Planner tests pass, including both squad actions, progress, final feedback, recovery, focus, errors, and snapshot replacement.
2. `./scripts/dev check` — Biome verify, TypeScript, secretlint, Rust format/lint/tests, and other configured gate checks pass; 396 Rust tests passed and 2 remained ignored.
3. `./scripts/dev smoke` — all 35 browser tests pass, including intermediate progress, completion, one shared outcome region, and stable supported-desktop action layout.
4. Confirm no files under `bridge/`, no migrations, no version owners, and no Load Data execution files changed.
5. Run the required fresh feature-complete review, resolve delegated findings, and complete documentation reconciliation before marking the PR ready for publication.

`./scripts/dev bridge-test` is not required for this feature because the C# bridge is explicitly unchanged. `./scripts/dev mutate` remains unsupported and must not be reported as passed.

## Documentation impact

- During Commit 1, update `.wiki/ARCHITECTURE.md` with the implemented per-invocation squad progress channel and `.wiki/DESIGN.md` with determinate confirmation-Modal progress.
- During Commit 2, update `.wiki/DESIGN.md` with the shared reserved outcome region and final copy contract.
- During feature reconciliation, verify ADR-0018 still describes the unchanged mutation boundary, condense this ledger, move it to `.wiki/features/completed/`, and update `.wiki/TODO.md` from Active to Completed.
