# Snapshot Date Edit (JAY-48)

## Status

Ready for final publication

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** c1b155117bdbdcd847e87f2707f0a026238649f501e85680f39b9fd5a8a86a90

## Intent

Linear JAY-48, “Change date of snapshot”: let the user correct a stored snapshot's in-game date from the existing Settings snapshot history, so a misdated snapshot orders correctly and the right snapshot becomes current. This is a small management extension to the completed snapshot-history contract, not a new history model.

## User-visible behavior

- Settings snapshot history gains a separate date edit action per snapshot row, alongside the existing Rename and Delete actions.
- The date editor accepts exactly one valid, non-null in-game date in canonical `YYYY-MM-DD` form (real calendar date, including leap-year rules).
- Empty, null, malformed, or calendrically impossible values fail with a clear dialog-local error and leave persisted state unchanged.
- On success the snapshot's `game_date` is updated; history reorders immediately using the existing shared order (dated first, greatest `game_date`, then `loaded_at_utc`, then snapshot ID); the winner becomes current.
- The frontend awaits `onBeforeContextChange` before every date-edit IPC call because backend winner state is authoritative; this can clear transient player-result context even when the winner does not change.
- When the returned `previousCurrentSnapshotId` and `currentSnapshotId` wire fields differ, current-snapshot views refresh through the existing route-owned invalidation (Search, Player, Moneyball, Club DNA, managed club, Planner, Academy, Staff).
- When the returned IDs are equal and the edited snapshot ID equals `currentSnapshotId`, the panel reconciles the existing `snapshotKeys.current()` cache locally with `queryClient.setQueryData`: it preserves the full cached `SnapshotSummary` and replaces only its `gameDate` from the returned updated metadata. It does not refetch or invalidate sibling roots.
- When the returned IDs are equal and the edited snapshot is non-current, only the history query refreshes; sibling query roots are not invalidated or refetched.
- The edited date persists across restarts (SQLite).
- Rename and delete behavior stay unchanged.

## Invariants

- Exactly one current snapshot per save with snapshots present; the winner is always the shared `SNAPSHOT_ORDER_BY` selector (`game_date IS NULL, game_date DESC, loaded_at_utc DESC, id DESC`).
- Date edit is token-bound (`snapshot_id` + `context_token`); stale or mismatched identity fails closed without writing.
- Validation happens before any write; a rejected value leaves the prior committed row (including its current marker) intact.
- The edited column is `game_date` only: `custom_name`, `game_date_source`, `loaded_at_utc`, and all snapshot-owned data stay unchanged.
- No `Set active` / manual current selector is added; current follows only from the shared date order.
- No edited-date provenance is recorded and no date-source change is made.
- No Academy class is created, removed, or recomputed merely because the date was edited; existing classes remain save-scoped.
- No Academy classes, ingest-derived historical data, role scores, Planner, Youth, Moneyball, or CSV behavior change merely because the date was edited, except the established current-promotion compact reconciliation (`player_role_metrics` / `staff_role_metrics`) that `select_current_snapshot` already performs.
- Failed date-edit operations leave the prior committed state intact.
- A date edit that can change effective current uses the established current-selection mutation exclusion (boost/context gate, same as `delete_snapshot`), so it cannot race boosts or Load Data publication.

## Non-goals

- Set active / manual current selector, edited-date provenance, or `game_date_source` changes.
- Recompute, backfill, or change Academy classes or any ingest-derived historical data merely because the date was edited.
- Snapshot merge, copy, export, restore, undo, deduplication, retention policy, or bulk date editing.
- Bridge protocol, dump schema, memory scanning, player-write, CSV parsing, or Moneyball ownership changes.
- A new route or a history selector in Search, Player Profile, Planner, or Academy.
- Historical player browsing, development graphs, or any other historical analysis.

## Current-state map

- Relevant components:
  - Rust: `src-tauri/src/features/snapshot/service.rs` (`SNAPSHOT_ORDER_BY`, `rename_snapshot`, `delete_snapshot`, `select_current_snapshot` with compact player/staff reconciliation), `src-tauri/src/features/snapshot/commands.rs` (management commands, boost-gate exclusion), `src-tauri/src/features/snapshot/query.rs` (ordered metadata via the shared order), `src-tauri/src/lib.rs` (command registration seam).
  - Validation: `src-tauri/src/features/memory_read/dump_validation.rs` — private `canonical_game_date` already enforces actual calendar-valid `YYYY-MM-DD` with leap-year rules.
  - Frontend: `src/features/snapshot/components/snapshot-history-panel.tsx` (Rename/Delete actions, focused accessible Modal patterns, `onBeforeContextChange` / `onCurrentContextChanged` callbacks), `src/features/snapshot/api/` (`rename-snapshot.ts`, `delete-snapshot.ts` analogues), `src/app/routes/settings.tsx` (route-owned invalidation for Search, Player, Moneyball, Club DNA, managed club, Planner, Academy, Staff).
  - Tests/mocks: `src/testing/snapshot-ipc-mock.ts` (ordered history, rename/delete mocks), `src/features/snapshot/components/snapshot-panels.test.tsx` (order/rename/delete/current proofs), `e2e/tauri-ipc-stub.ts` + `e2e/smoke.spec.ts` (management/order/current smoke).
- Data model: schema with `snapshots(game_date NULLABLE, game_date_source, loaded_at_utc, custom_name, context_token, is_current)` plus the partial unique current index; no migration is expected (the column already exists and stays nullable).
- Persistence and migrations: none planned; the edit is an `UPDATE` of one existing column inside the existing transaction that also runs `select_current_snapshot`.
- Existing behavioral assumptions: retained snapshots, greatest valid in-game date is current, dated rows always precede undated rows, equal dates break ties by newest load then ID, rename never reorders, current deletion promotes atomically with trusted-date Academy handling, all normal domain reads stay current-only.
- Architectural seams: Rust owns validation, transactions, selection, and destructive/current-changing policy; React owns presentation and confirmation state; the Settings route owns cross-feature query invalidation; snapshot feature code must not import sibling-feature query keys.
- Project validation commands: `./scripts/dev test <targets>`, `./scripts/dev check`, `./scripts/dev smoke`. `./scripts/dev bridge-test` is out of scope (no bridge change). `./scripts/dev mutate` is unsupported and must not be reported as passed.
- Primary risks: accepting a malformed date that corrupts the date comparator; introducing a second drifting date contract; changing current without the boost exclusion or without full current-only invalidation; accidentally creating Academy classes or touching ingest-derived data on edit; stale-modal retarget after a save switch.

## Feature architecture

- Rust service owns one new token-bound `update_snapshot_game_date`-style operation: validate the raw required input string exactly against the single canonical date contract with no trimming, reject empty/invalid values with a clear error before any write, `UPDATE` only `snapshots.game_date` for the matching `(id, context_token)` row, then run the existing `select_current_snapshot` in the same transaction and return the updated metadata plus internal `previous_current_snapshot_id` and `current_snapshot_id` surfaced on the wire as `previousCurrentSnapshotId` and `currentSnapshotId` via `#[serde(rename_all = "camelCase")]`. It must not call `academy_service::ensure_class_for_game_date` and must not write any other column.
- Canonical validation has exactly one owner: make the existing `memory_read::dump_validation::canonical_game_date` `pub(crate)` and reuse it from the snapshot service. No copied validator and no new module.
- The Tauri command is a thin management command beside `rename_snapshot` / `delete_snapshot`, registered in `src-tauri/src/lib.rs`, carrying `(snapshot_id, context_token, game_date)`. Because the edit can change effective current, it takes the same boost/context exclusion that guards current-selection mutations (as `delete_snapshot` does via `boost_gate`).
- React adds an `Edit date` action per history row and a focused form Modal following the existing Rename Modal pattern: bounded `YYYY-MM-DD` text input, dialog-local validation and errors, duplicate-submission protection, stable `(id, token)` target identity across save switches, focus restoration, history invalidation on success, and `onBeforeContextChange` awaited before every date-edit IPC call because backend winner state is authoritative (this can clear transient player-result context); when the returned IDs differ, it invalidates `snapshotKeys.current()` and sibling query roots and calls `onCurrentContextChanged`; when the IDs are equal and the edited snapshot ID equals `currentSnapshotId`, it patches the existing `snapshotKeys.current()` cache locally with `queryClient.setQueryData` by preserving the full cached `SnapshotSummary` and replacing only its `gameDate` from the returned updated metadata, without refetching or invalidating sibling roots; when the IDs are equal and the edited snapshot is non-current, it refreshes history only.
- The Settings route reuses its existing `invalidateCurrentContext` unchanged; no new invalidation roots and no sibling-feature imports inside snapshot feature code.

## Uncertainty register

### Known

- `SNAPSHOT_ORDER_BY` is shared by ingest selection, metadata listing, and deletion promotion; date edit must reuse it, not duplicate it.
- `canonical_game_date` is private in `dump_validation.rs` and already enforces real calendar validity.
- `select_current_snapshot` atomically reconciles compact player and staff metrics; deletion promotion additionally calls `ensure_class_for_game_date`, which date edit must skip.
- `delete_snapshot` acquires the boost gate; `set_active_save` acquires the context gate; Load Data holds the load gate. A date edit changes effective current and therefore needs the current-selection mutation exclusion.
- No planned feature spec exists; no active ledger exists; BACKLOG needs no change; no ADR is warranted.

### Assumptions

- Linear's suggested branch `jaycount/jay-48-change-date-of-snapshot` is advisory; repository evidence (`feat/*` branches, squash merges onto trunk `main`) supports the short branch `feat/snapshot-date-edit`.
- One PR is sufficient; no independently mergeable, trunk-safe seam exists that justifies a second PR.
- No SQLite migration is needed because `game_date` already exists as a nullable canonical string.

### Decisions

- Canonical `YYYY-MM-DD` with real calendar validity (including leap years) is the only accepted input; null/empty/clear-date is rejected, not stored. Consequence: there is no “undate this snapshot” action in this feature; an undated snapshot can only arise from ingest, never from editing.
- Date edit reuses `select_current_snapshot` (with its compact reconciliation) but explicitly skips `ensure_class_for_game_date`. Consequence: promoting a snapshot by date edit never creates an Academy class, unlike deletion promotion.
- The new command takes the established boost/context exclusion for current-selection mutations. Consequence: date edits serialize against boosts and Load Data publication instead of racing them.

### Unknowns

- None remaining that change product behavior, invariants, or architecture. Validation ownership is resolved: make `canonical_game_date` `pub(crate)` and reuse it from the snapshot service; no copied validator and no new module.

### Risks

- Lexical ordering is correct only for canonical dates; any accepted malformed string silently selects the wrong current snapshot. Mitigated by single-owner validation before write plus invalid/empty RED cases.
- A second date contract drifts from the bridge contract. Mitigated by reusing, not copying, `canonical_game_date`.
- Stale caches after a winner change. Mitigated by reusing the exact delete-current invalidation path.
- Stale modal retarget after a save switch or row-ID reuse. Mitigated by binding the dialog to the immutable `(id, token)` identity.

## Walking skeleton

Rename nothing: open Settings history with two dated snapshots, edit the older row's date to a greater valid date, and observe the edited row become `Current` with Search-backed views refreshed — proving validation, persistence, shared ordering, current promotion, and invalidation through one thin path before hardening edge cases.

## Delivery plan

### PR 1 — Snapshot date edit

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feat/snapshot-date-edit`

**Base branch:** `main`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict `check`

**Feature close-out:** Current

**CI repair rounds:** 0

**Provisional PR title:** `feat(snapshot): edit snapshot in-game date`

**Purpose:** Deliver one reviewable, revertible management extension: a validated, token-bound, persisted snapshot date edit that reuses the shared date selector and existing current-only invalidation, without touching Academy or ingest-derived data.

**Depends on:** Completed snapshot history and management on `main`; no other pending feature.

#### Commit 1 — Record the approved feature plan

**Status:** Completed

**Provisional commit:** `docs(snapshot): record approved date-edit plan`

**Work:** Commit the independently reviewed planning artifacts on the feature branch before implementation.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, executable configuration, generated files, and unrelated documentation.

**Implementation packet:**

- Preserve the accepted plan-review outcome. Commit only the reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/snapshot-date-edit.md` — approved feature intent, delivery plan, and packets.
- `.wiki/TODO.md` — JAY-48 as the sole Active feature; existing Next item unchanged.

**Behavior and data flow:**

- Move planning truth into one reviewed active ledger and record the exact delivery sequence before implementation. No planned spec exists to remove; BACKLOG and ADRs are unchanged.

**Ordered implementation steps:**

1. Verify the active branch (`feat/snapshot-date-edit`) and base (`main`) without changing Git state.
2. Confirm the worktree contains only the two reviewed planning paths.
3. Run the ledger classifier and any repository documentation check.
4. Stage and inspect the exact planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — this commit changes planning documents only. The ledger classifier and documentation checks prove structural consistency.

**Patterns to verify:**

- The active-ledger template, current TODO ownership rules, and the rule that the planning-artifact commit lists exactly its two planning paths with no planned-spec/BACKLOG/ADR changes.

**Constraints and non-goals:**

- Do not alter implementation, tests, executable configuration, plan scope, packet order, or reviewed decisions. Do not touch the unrelated pre-existing modification to `.wiki/features/completed/top-navigation.md`.

**Dependencies and sequencing:**

- Requires an accepted plan-review verdict, developer acceptance, a valid Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/snapshot-date-edit.md` plus the repository documentation check when one exists.

**Stop conditions:** Stop on an uncleared review, a classifier error, an unreviewed path, a substantive post-review plan change, or a branch mismatch.

**Review mandate:** Verify that the staged diff contains the complete reviewed planning outcome and no implementation or unrelated files.

#### Commit 2 — Update snapshot game date in Rust

**Status:** Completed

**Provisional commit:** `feat(snapshot): update snapshot game date`

**Work:** Add the validated, token-bound, transactional snapshot date-update service and Tauri command that reuses the shared selector and current-selection exclusion without touching Academy or other columns.

**Size assessment:** Small focused backend change (one service function, one command, registration, focused tests); within the soft target.

**Out of scope:**

- Frontend controls, IPC mocks, and smoke coverage (Commit 3).
- Migrations, `game_date_source` changes, provenance, Academy behavior, Moneyball/CSV/bridge changes, and historical player reads.

**Implementation packet:**

**Files and responsibilities:**

- `src-tauri/src/features/memory_read/dump_validation.rs` — make the existing `canonical_game_date` `pub(crate)` and reuse it from the snapshot service so exactly one canonical date contract exists; no behavior change to dump validation, no copied validator, no new module.
- `src-tauri/src/features/snapshot/service.rs` — add the token-bound date-update function: validate the raw required non-null `YYYY-MM-DD` string exactly with no trimming (reject empty, whitespace-wrapped, and calendrically impossible values with a clear error before any write), `UPDATE` only `snapshots.game_date` for `(id, context_token)`, then run the existing `select_current_snapshot` in the same transaction; the typed service result explicitly carries the updated row as a named `SnapshotMetadata` field (for example `snapshot`) alongside internal `previous_current_snapshot_id` / `current_snapshot_id`; explicitly do not call `academy_service::ensure_class_for_game_date` and do not write any other column.
- `src-tauri/src/features/snapshot/commands.rs` — add the thin `update_snapshot_game_date`-style command `(snapshot_id, context_token, game_date)` with the established boost/context exclusion used for current-selection mutations; the typed DTO result explicitly carries the updated row as a named `SnapshotMetadataDto` field (for example `snapshot`) alongside the `#[serde(rename_all = "camelCase")]` wire fields `previousCurrentSnapshotId` and `currentSnapshotId`, so the frontend cache patch has the updated `gameDate` without a refetch.
- `src-tauri/src/lib.rs` — register the new command beside `rename_snapshot` / `delete_snapshot`.
- Focused Rust tests beside the touched modules proving the contract below.

**Behavior and data flow:**

- Invoke `update_snapshot_game_date(snapshot_id, context_token, game_date)` → validate identity and the raw required canonical date string exactly with no trimming (fail closed, state unchanged) → single transaction: update one column, run shared `SNAPSHOT_ORDER_BY` selection, reconcile compact metrics via the selector's existing path → return the updated row in the named `SnapshotMetadata` result field plus internal `previous_current_snapshot_id` and `current_snapshot_id` surfaced on the wire as the named DTO metadata field and `previousCurrentSnapshotId` / `currentSnapshotId`. Undated-vs-dated, equal-date (`loaded_at_utc`, then id), and winner-change vs. winner-unchanged paths all follow the existing selector.

**Ordered implementation steps:**

1. Add the smallest RED service/command tests: invalid and empty values rejected with state unchanged; valid edit persists, reorders, and promotes per the shared order; stale token rejected; non-current edit leaves current alone; promotion path performs no Academy writes.
2. Implement the single validation owner reuse plus the minimal service transaction and thin command/registration change to turn the proof GREEN.
3. Refactor only while the focused proof stays green.
4. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- RED then GREEN Rust tests: reject empty string, whitespace-wrapped valid-looking input (` 2024-02-29 `), malformed shape (`2026-13-01`, `2026-02-30`, non-leap `2023-02-29`), and accept leap-day `2024-02-29`; prove every rejected value (including the whitespace-wrapped input) leaves `game_date`, `custom_name`, `game_date_source`, `loaded_at_utc`, and current markers unchanged; prove editing an older row to a greater date promotes it and editing the current row to a lesser date demotes it per `SNAPSHOT_ORDER_BY`; prove equal-date ties break by `loaded_at_utc` then snapshot ID; prove transactional rollback of date, current-marker, and compact rows when promoted player or staff compact materialization fails; prove stale `(id, token)` fails closed; prove `custom_name`, `game_date_source`, and player/role data untouched; prove no Academy class row is created or changed by the edit (guard with a trigger or row-count assertion); prove the result DTO carries the updated metadata in its named field and serializes to the exact camelCase wire keys `previousCurrentSnapshotId` and `currentSnapshotId`.
- Retain existing selector, rename/delete, and cascade tests; update only tests whose contract this commit intentionally changes (none expected beyond new coverage).

**Patterns to verify:**

- `rename_snapshot` token binding (without copying its trimming behavior — date edit validates the raw string exactly), `delete_snapshot` transactional promotion + boost-gate acquisition, `SNAPSHOT_ORDER_BY` reuse from `query.rs`, command DTO `#[serde(rename_all = "camelCase")]` conversion and `lib.rs` registration order.

**Constraints and non-goals:**

- Validate the raw required string exactly with no trimming before write; single transaction; update exactly one column; reuse the shared selector; skip Academy class creation; keep the boost/context exclusion; no migration; no second date contract; no Set-active, provenance, source, Academy, ingest-derived, CSV, Moneyball, or bridge changes.

**Dependencies and sequencing:**

- Requires Commit 1 (planning artifact) on the feature branch. Blocks Commit 3 (frontend), which drives this command.

**Validation:** Focused Rust tests for the new service/command, then `./scripts/dev check` (`./scripts/dev check-rust` may run first but cannot replace it). No `bridge-test`; no `mutate`.

**Stop conditions:** Stop and replan if a migration proves necessary, if one shared validator cannot serve both dump validation and the edit without behavior change, if the selector cannot be reused transactionally, or if the exclusion/semantics require a new product rule (e.g. clear-date support).

**Review mandate:**

- Verify the validator rejects every non-canonical shape and shares one owner with dump validation.
- Verify exactly one column is written and failures leave committed state intact.
- Verify the transaction reuses `select_current_snapshot` verbatim and never calls `ensure_class_for_game_date`.
- Verify the command carries the current-selection mutation exclusion and is registered.
- Verify stale-token and ID-reuse cases fail closed.
- Verify no Academy, Moneyball, CSV, Planner, Youth, or bridge behavior changed.
- Verify test failures are contract failures (wrong date accepted / wrong winner), not setup failures.

#### Commit 3 — Edit snapshot date from Settings history

**Status:** Completed

**Provisional commit:** `feat(snapshot): edit snapshot date from Settings`

**Work:** Add the Settings history `Edit date` action, validated date Modal, typed IPC API, test mocks/stubs, and component/smoke coverage reusing the existing rename/delete interaction and invalidation patterns.

**Size assessment:** Bounded React/TanStack Query surface (one action, one modal, one API module, mock/stub/test updates); within the soft target.

**Out of scope:**

- Backend validation/selection changes (Commit 2 owns them).
- Set-active controls, provenance display, Academy surfaces, visual redesign of unrelated panels, pagination, or bulk editing.

**Implementation packet:**

**Files and responsibilities:**

- `src/features/snapshot/api/update-snapshot-date.ts` (new, mirroring `rename-snapshot.ts`) — typed `invokeCommand` wrapper for `(snapshotId, contextToken, gameDate)` returning the updated `SnapshotMetadata` in a named field alongside the camelCase wire fields `previousCurrentSnapshotId` and `currentSnapshotId`.
- `src/features/snapshot/components/snapshot-history-panel.tsx` — per-row `Edit date` action and a focused form Modal modeled on `SnapshotRenameModal`: `YYYY-MM-DD` text input, dialog-local errors, pending protection, stable `(id, token)` target identity, focus restoration; await `onBeforeContextChange` before every date-edit IPC call (backend winner state is authoritative, and the hook can clear transient player-result context); on success invalidate history, then branch on the returned IDs: when they differ, invalidate `snapshotKeys.current()` and call `onCurrentContextChanged` for the route-owned sibling refresh; when they are equal and the edited snapshot ID equals `currentSnapshotId`, reconcile the existing `snapshotKeys.current()` cache locally with `queryClient.setQueryData` by preserving the full cached `SnapshotSummary` and replacing only its `gameDate` from the returned updated metadata, without refetching or invalidating sibling roots; when they are equal and the edited snapshot is non-current, refresh history only.
- `src/testing/snapshot-ipc-mock.ts` — date-edit mock mode with canonical validation (raw string exactly, no trimming), shared-order re-sorting, and current promotion returning the updated metadata alongside the camelCase wire keys `previousCurrentSnapshotId` and `currentSnapshotId`, mirroring the existing rename/delete mocks.
- `src/features/snapshot/components/snapshot-panels.test.tsx` — RED-then-GREEN cases below.
- `e2e/tauri-ipc-stub.ts`, `e2e/smoke.spec.ts` — stub support plus one smoke path: edit an older snapshot to a greater date and observe reorder + promoted current data.
- `src/app/routes/settings.tsx` — no logic change expected; reuse the existing `invalidateCurrentContext` (confirm coverage, do not add new roots or sibling imports).

**Behavior and data flow:**

- Click `Edit date` → dialog opens bound to the row's immutable `(id, token)` → submit `YYYY-MM-DD` → frontend minimal shape check, then await `onBeforeContextChange`, then IPC (backend winner state is authoritative and the hook can clear transient player-result context) → success: invalidate `snapshotKeys.history`; when the returned `previousCurrentSnapshotId` and `currentSnapshotId` differ, also invalidate `snapshotKeys.current()` and call `onCurrentContextChanged()` (route refreshes Search, Player, Moneyball, Club DNA, managed club, Planner, Academy, Staff); when the IDs are equal and the edited snapshot ID equals `currentSnapshotId`, patch the existing `snapshotKeys.current()` cache with `queryClient.setQueryData` by preserving the full cached `SnapshotSummary` and replacing only its `gameDate` from the returned updated metadata, without refetching or invalidating sibling roots; when the IDs are equal and the edited snapshot is non-current, history refresh only; errors stay in the dialog with the input retained and focus restored on close.

**Ordered implementation steps:**

1. Add the smallest RED component tests: missing ordered date-edit action; invalid/empty submit shows a dialog-local error with no IPC write; valid edit reorders and promotes with route callback; non-promoting edit avoids the global callback; stale identity surfaces the backend error in the dialog.
2. Add the minimal API module, panel action + Modal, and mock updates to turn the proof GREEN.
3. Extend the e2e stub and add the single smoke case.
4. Refactor only while the focused proof stays green.
5. Run targeted, affected, and commit-level validation in the recorded order.

**Tests and proof:**

- RED then GREEN `snapshot-panels.test.tsx` cases: renders `Edit date` per row with an accessible name that disambiguates duplicate dates (same `snapshotTargetLabel` convention as Delete); empty and malformed inputs fail locally without invoking IPC and preserve persisted mock state; the mock and component consume the exact camelCase wire keys `previousCurrentSnapshotId` and `currentSnapshotId` plus the named updated-metadata field; a promoting edit re-sorts the list, moves the `Current` marker, and invokes `onCurrentContextChanged`; a non-promoting edit to a non-current row refreshes history only; editing the current row while it remains current updates the Snapshot overview visible date via the local `setQueryData` patch and does not call `onCurrentContextChanged`; every submit awaits `onBeforeContextChange` before IPC; a stale-token backend rejection renders in the dialog with the target retained; pending state blocks duplicate submission; Escape/cancel restores focus to the panel.
- GREEN smoke in `smoke.spec.ts`: with two dated snapshots, edit the older to a greater date, expect date-ordered list and promoted current player count.
- Deliberately retain all existing rename/delete/order/current tests unchanged.

**Patterns to verify:**

- `SnapshotRenameModal` focus/error/pending behavior, `SnapshotDeletionModal` stable target identity + current-vs-non-current invalidation split, `snapshotKeys.history/current` factories, route-owned sibling invalidation in `settings.tsx`, `snapshotOrder` parity between the IPC mock and `SNAPSHOT_ORDER_BY`.

**Constraints and non-goals:**

- No sibling-feature imports inside snapshot feature code; no new invalidation roots; names never replace visible date metadata; destructive-vs-form Modal variants used correctly (date edit is a form Modal, not destructive); no Set-active, provenance, source, Academy, or ingest-derived display changes.

**Dependencies and sequencing:**

- Requires Commit 2's command on the feature branch. Route invalidation in `settings.tsx` already exists and is reused, not rebuilt.

**Validation:** `./scripts/dev test src/features/snapshot/components/snapshot-panels.test.tsx`, `./scripts/dev smoke`, then `./scripts/dev check`. No `bridge-test`; no `mutate`.

**Stop conditions:** Stop if the UI needs sibling-feature imports to invalidate, if the dialog cannot retain `(id, token)` identity across save switches, if the mock/stub would falsely imply native SQLite proof, or if fitting the action requires a broader Settings redesign decision.

**Review mandate:**

- Verify the action is separate from Rename and every row disambiguates duplicates accessibly.
- Verify invalid/empty input never reaches IPC and leaves state unchanged.
- Verify promoting vs. non-promoting edits invalidate exactly the right roots.
- Verify keyboard flow, initial focus, focus restoration, busy states, and error recovery.
- Verify no historical player view, manual current selector, provenance, or source change slipped in.
- Verify mocks/stubs replicate the shared order and promotion truthfully.

## Active work

**PR:** PR 1 — Snapshot date edit

**Active work:** None — feature validation and close-out

**Commit:** None — feature validation and close-out

### RED or removal proof

Not applicable — all planned implementation packets are complete and independently reviewed. The feature now requires full validation, feature review, and documentation reconciliation.

### Expected outcome

The complete feature passes its recorded validation and review, durable documentation reflects the implemented behavior, and the final PR becomes ready for publication.

### Explicit exclusions

- Release preparation and unrelated implementation or documentation.
- Any expansion beyond the accepted JAY-48 date-edit behavior.

## Discoveries and replanning

- Commit 3 also requires the existing `src/testing/setup.ts` IPC router to forward `update_snapshot_game_date` to the snapshot mock and reset its busy state. The packet named the mock but omitted this seven-line test-harness seam; the change is mechanical, stays inside frontend proof infrastructure, and does not alter scope, architecture, behavior, packet order, or authority.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PR 1 — Snapshot date edit | Commit 1 — Record the approved feature plan | `092110cc53be697319703dc78edd601da7ac2388` | Added the accepted schema-2 JAY-48 ledger and made Snapshot Date Edit the sole TODO Active feature. | `ledger_state.py` runnable; `delivery_state.py` runnable; staged diff check clean; no repository documentation command exists. | Not applicable | Clear | 0 | None |
| PR 1 — Snapshot date edit | Commit 2 — Update snapshot game date in Rust | `bb01feaafba67bb13c5f1a7239e0386735d8f674` | Added the validated token-bound game-date update transaction, shared current selection, compact reconciliation, current-selection exclusion, typed DTO, and command registration without Academy writes. | RED compile proof failed on missing symbols; focused snapshot tests passed 94/94; `./scripts/dev check-rust` passed; `./scripts/dev check` passed with 775 Rust tests, 0 failed, 2 ignored. | Pass | Clear | 0 | None |
| PR 1 — Snapshot date edit | Commit 3 — Edit snapshot date from Settings history | `aab9506908651d3921c1014ed1c7e11ba7a15a4d` | Added the accessible Settings date editor, typed IPC adapter, truthful mock/stub ordering and promotion, precise current-cache reconciliation, and browser smoke path. | RED action test failed on the missing control; component suite passed 35/35; `./scripts/dev smoke` passed 55/55 after one unrelated Squad progress timing retry; `./scripts/dev check` passed with 775 Rust tests, 0 failed, 2 ignored. | Pass | Clear | 1 | Added the required seven-line `src/testing/setup.ts` IPC route/reset seam omitted from the packet; corrected a cross-save current-cache identity defect found in review. |

## Final validation

- Ledger classifier GREEN on `.wiki/features/active/snapshot-date-edit.md` plus the repository documentation check (planning commit).
- Commit 2: focused Rust tests GREEN, then `./scripts/dev check` (`./scripts/dev check-rust` may run first but cannot replace it).
- Commit 3: `./scripts/dev test src/features/snapshot/components/snapshot-panels.test.tsx`, `./scripts/dev smoke`, then `./scripts/dev check`.
- `git diff --check` clean on the feature branch; staged diff contains only the recorded paths per commit.
- `./scripts/dev bridge-test` not run (no bridge change); `./scripts/dev mutate` unsupported and never reported as passed.
- Native Windows/FM26/WebView/SQLite integration remains an explicit manual gap if unavailable; browser IPC stubs must not be presented as native proof.

## Feature close-out

- Feature validation passed: component suite 35/35, smoke 55/55, and `./scripts/dev check` with 775 Rust tests passed, 0 failed, 2 ignored. One unrelated Squad progress timing assertion required a retry; the JAY-48 smoke path passed throughout.
- Feature review cleared after one correction round with no CRITICAL or HIGH findings. Test portfolio: Pass. Project fit: Conforms.
- Correction evidence: fixed fake year-zero parity and added the no-refetch cache proof.

## Exact implementation refs

**Base:** `a024fb1c04e224489c99ebb043adc09ba89f62a1`

| Ref | Subject | Role |
| --- | --- | --- |
| `092110cc53be697319703dc78edd601da7ac2388` | `docs(snapshot): record approved date-edit plan` | Planning record |
| `bb01feaafba67bb13c5f1a7239e0386735d8f674` | `feat(snapshot): update snapshot game date` | Rust command, transaction, and selector behavior |
| `aab9506908651d3921c1014ed1c7e11ba7a15a4d` | `feat(snapshot): edit snapshot date from Settings` | Settings Modal, IPC, mocks, and smoke path |
| `5d5f2071153bd97e8da30b40aa0fef44db33be0f` | `test(snapshot): align date-edit fakes and cache proof` | Feature-review correction |

The feature implementation range is `a024fb1c04e224489c99ebb043adc09ba89f62a1..5d5f2071153bd97e8da30b40aa0fef44db33be0f`. The documentation close-out ref remains `Pending record` until the close-out commit exists.

## Final publication

```yaml
status: ready_for_publication
pr_status: not_published
merge_status: not_merged
pr_ref: "Not published"
merge_ref: "Not merged"
branch: feat/snapshot-date-edit
base_branch: main
base_ref: a024fb1c04e224489c99ebb043adc09ba89f62a1
publication_provider: GitHub
pr_template: .github/pull_request_template.md
merge_method: squash
required_checks: strict_check
required_check_name: check
pr_count: 1
earlier_prs: none
feature_close_out: current
feature_review_blocking: false
feature_review_critical: none
feature_review_high: none
feature_review_medium: none
feature_review_nitpick: none
project_fit: conforms
feature_review_action: skip
feature_review_correction_rounds: 1
ci_repair_rounds: 0
implementation_range: a024fb1c04e224489c99ebb043adc09ba89f62a1..5d5f2071153bd97e8da30b40aa0fef44db33be0f
implementation_refs:
  - 092110cc53be697319703dc78edd601da7ac2388
  - bb01feaafba67bb13c5f1a7239e0386735d8f674
  - aab9506908651d3921c1014ed1c7e11ba7a15a4d
  - 5d5f2071153bd97e8da30b40aa0fef44db33be0f
close_out_documentation_ref: Pending record
publication_correction_evidence: none
```

## Documentation impact

Reconciliation completed after final validation and cleared feature review. `ARCHITECTURE.md` now records the date-edit command, transactional shared selection and compact reconciliation, cache behavior, and the absence of Academy writes. `DESIGN.md` now records the Edit date action, form Modal validation and focus behavior, and winner-change, same-current, and non-current refresh semantics. `TODO.md` no longer lists JAY-48 as Active. No CONCEPT, BACKLOG, ADR, or debug report change is warranted. No feature-owned `.work` artifacts are known. The active ledger `.wiki/features/active/snapshot-date-edit.md` is approved for removal after this completed record is inspected.
