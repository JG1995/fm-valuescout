# Snapshot History and Management

## Status

Validation

## Intent

Retain every successfully loaded Football Manager snapshot inside its app save instead of replacing the previous snapshot. Keep normal product reads on the snapshot with the greatest in-game date, and add bounded management for snapshot names, snapshot deletion, and save deletion without adding historical player browsing or development analysis yet.

## User-visible behavior

- Every successful **Load Data** stores a new snapshot in the save that was active when the scan started. Existing snapshots in that save remain stored.
- The effective latest snapshot is the snapshot with the greatest valid in-game date, not the snapshot loaded most recently. Search, player profiles, Planner, Academy candidate resolution, sanity data, and CSV matching continue to read only that effective latest snapshot.
- Loading an earlier in-game date stores it without changing the effective latest snapshot. A user who intentionally rolls a career back can delete every later-dated snapshot; the greatest remaining date then becomes latest automatically.
- Snapshot lists are ordered newest in-game date first. Equal dates use newest load first as a deterministic tie-break. Undated snapshots appear after dated snapshots, never supersede a dated snapshot, and use newest load first only when a save contains no dated snapshots.
- A successful same-date load is retained separately and becomes latest ahead of older loads from that date. The feature does not deduplicate snapshots.
- The Load Data outcome distinguishes the newly stored snapshot from the effective latest snapshot so an earlier or undated load is never presented as the data now used by the app.
- The Dashboard lists snapshot metadata for the active save, marks the latest snapshot, and lets the user assign or replace a custom snapshot name. A custom name changes organization metadata only; date ordering remains authoritative.
- The user can delete any snapshot after confirming the named target. Deleting the latest snapshot promotes the greatest remaining date in the same transaction. Deleting the final snapshot leaves the save active with no current data.
- The user can delete any save after a confirmation that names the save and the data that will be removed. Deleting an inactive save leaves the current context alone; deleting the active save activates another existing save; deleting the final save atomically creates a blank active `Default save`.
- Snapshot deletion removes that snapshot's players, staff, role scores, bridge provenance, and Moneyball enrichment through foreign-key cascades. It does not remove save-scoped Planner configuration, Academy records, or Youth career enrichment. Save deletion removes all data owned by that save.
- Moneyball CSV imports attach to the effective latest snapshot and remain frozen with that snapshot. Youth Tracker career totals remain save-scoped because Academy treats them as cumulative career enrichment rather than a measurement of one snapshot.
- The v17 upgrade preserves every existing Moneyball row as quarantined, unassigned legacy data because v17 recorded no source snapshot identity. Legacy rows are not discarded, falsely attached, or used by current or historical reads in this feature.
- CSV import still captures and revalidates the active save and effective latest snapshot. It never targets the snapshot loaded most recently merely because that load finished last.
- Failed ingest, migration, rename, or deletion operations leave the prior committed state intact.

## Invariants

- Exactly one app save is active after every committed save-management transaction.
- A save with no snapshots has no current snapshot. A save with one or more snapshots has exactly one `is_current = 1` row, and that row is the winner of the shared date-ordering rule.
- The shared ordering rule is: valid non-null `game_date` before null; greatest `game_date` first; then greatest `loaded_at_utc`; then greatest snapshot ID. Load order has no effect across different valid in-game dates.
- A non-null in-game date must satisfy the canonical bridge `YYYY-MM-DD` contract before it participates in selection. Null remains a supported value.
- Snapshot custom names never influence current selection, list ordering, player identity, or enrichment ownership.
- Players, staff, and role scores remain snapshot-owned. No player row is merged forward, copied between snapshots, or kept searchable after its owning snapshot stops being latest.
- Existing current-only domain queries stay current-only. This feature adds a management metadata query, not a general historical player-data read model or snapshot selector.
- Memory-backed player fields remain authoritative. CSV never creates a player and still matches exact numeric UIDs from the effective latest snapshot.
- Moneyball rows are owned by `(snapshot_id, player_uid)` and cascade with the owning snapshot. Youth career rows remain owned by `(save_id, player_uid)` and cascade only with the save.
- Every existing v17 Moneyball row moves unchanged to a save-scoped legacy quarantine, retains its values and import timestamp, is excluded from snapshot reads and new imports, and cascades with its save. The migration does not infer a source snapshot from current UID membership.
- Ingest inserts the complete snapshot and selects the winner in one transaction. Snapshot deletion, current promotion, save deletion, and fallback-save activation are likewise atomic.
- Save and snapshot row IDs are not sufficient asynchronous identities because SQLite can reuse them after deletion. Management adds immutable internal context tokens; Load Data, CSV import, and destructive mutations capture and revalidate the relevant ID/token pair, while player writes also retain their existing bridge request-ID check.
- An asynchronous operation remains bound to the context it captured. A Load Data scan whose target save was deleted must fail instead of retargeting, and CSV or player-write reconciliation must keep its stale/partial-outcome handling when a snapshot or save disappears.
- Planner assignments and Academy memberships remain save-scoped. When the latest snapshot changes, their player resolution may change, but deleting one snapshot does not delete those records.
- Automatic Academy class creation follows the effective current snapshot. Storing a non-current earlier or undated snapshot creates no class; promoting a snapshot runs the same trusted-date class check, while classes already created from a formerly current snapshot remain save-scoped.
- Player-development boosts remain bound to the bridge scan that created their snapshot. Promoting an older snapshot, or storing an earlier snapshot after the current one, cannot authorize a write against stale live provenance; the existing fail-closed Load Data recovery remains mandatory.
- No automatic retention limit, age-based cleanup, or implicit pruning is introduced.

## Non-goals

- Browsing a historical player's profile, selecting an older snapshot as current, or searching across old snapshots.
- Development graphs, attribute deltas, squad comparisons, trend calculations, or any other historical analysis.
- Season identities, import-wave history, amendment rules, or a Moneyball analytics screen. Moneyball data is only versioned by snapshot here.
- Displaying, assigning, merging, or otherwise interpreting quarantined legacy Moneyball rows whose original snapshot no longer exists.
- Versioning Youth Tracker career totals per snapshot or changing Academy's cumulative career-statistics model.
- Snapshot merge, copy, export, restore, undo, deduplication, or automatic retention policy.
- Renaming saves beyond the existing behavior, importing a save from another database, or deleting data outside the explicitly selected snapshot or save.
- Bridge protocol, dump schema, memory-scanning, or player-write changes.
- A new route or a history selector in Search, Player Profile, Planner, or Academy.

## Current-state map

- **Relevant components:** Rust snapshot ingest, reads, save services, and IPC live in `src-tauri/src/features/snapshot/`; CSV persistence lives in `src-tauri/src/features/csv_import/service.rs`; current-only Search, Player, Planner, and Academy reads live in their corresponding Rust feature modules; Tauri command registration lives in `src-tauri/src/lib.rs`.
- **Frontend surfaces:** Dashboard save and snapshot panels live in `src/features/snapshot/`; Load Data result handling lives in `src/features/memory-read/`; Dashboard cross-feature composition lives in `src/app/routes/index.tsx`; browser IPC stubs and product smoke live under `src/testing/` and `e2e/`.
- **Data model:** schema v19 has `saves`, one partially unique current `snapshots` row per save, immutable save/snapshot context tokens, optional snapshot custom names, snapshot-owned players/staff/role scores/Moneyball rows, save-owned Planner/Academy/Youth data, and a save-owned legacy Moneyball quarantine.
- **Persistence and migrations:** `src-tauri/src/db/migrations.rs` owns ordered SQLite migrations and upgrade tests. `snapshots.game_date`, `game_date_source`, `loaded_at_utc`, custom names, immutable save/snapshot context tokens, and snapshot-owned Moneyball rows all exist.
- **Existing behavioral assumptions:** successful ingest retains complete snapshots and selects one current row by the shared date comparator. All normal domain queries join the active save to `snapshots.is_current = 1`. CSV import captures and revalidates that current snapshot before writing: Moneyball is snapshot-scoped, while Youth enrichment remains save-scoped.
- **Architectural seams:** Rust owns selection, transactions, validation, and destructive policy. React owns presentation and confirmation state. Route and app-shell composition own cross-feature query invalidation; snapshot feature code must not import query keys from sibling features.
- **Project validation commands:** `./scripts/dev test`, `./scripts/dev check`, and `./scripts/dev smoke`. `./scripts/dev bridge-test` is not part of the planned feature gate because the bridge is unchanged.
- **Primary risks:** silent loss during the Moneyball table rebuild, an incorrect date comparator making stale players searchable, SQLite row-ID reuse retargeting in-flight work, destructive cascades exceeding the named target, stale caches after current promotion or save deletion, and bridge write provenance becoming misleading after a historical load.

## Feature architecture

### Snapshot retention and latest selection

Snapshot ingest remains the only creator of snapshot-owned data. It will stop deleting the prior current snapshot and will apply one shared selector after a complete new snapshot has been inserted. The selector treats canonical non-null game dates as authoritative, places null dates behind every dated row, and uses load timestamp plus ID only to make equal-date or all-undated results deterministic.

The `is_current` marker remains the compatibility seam for every existing product query. No Search, Player, Planner, or Academy query receives a history parameter. The ingest result returns both the stored snapshot outcome and enough effective-current metadata for Load Data to explain when the stored row did not become latest.

The most recently completed bridge scan may belong to a non-current historical row after an earlier load. Snapshot date selection must not be conflated with live bridge provenance. Player writes continue to fail closed unless the bridge request ID matches the snapshot that authorized the action; recovery is to clean up later snapshots as desired and run Load Data again.

### CSV enrichment ownership

Moneyball values describe the player at the time of an export and therefore move from save ownership to snapshot ownership. Migration v17 did not record which current snapshot accepted an import, and save-scoped rows intentionally survived later snapshot replacement. Current UID membership therefore cannot prove a legacy row belongs to the surviving snapshot. The migration preserves the complete old table as a separate save-owned legacy quarantine and starts the new snapshot/player table empty. The quarantine is not a fallback read model and receives no new imports. Future imports continue to match the active save's effective latest snapshot and upsert only that snapshot's matching UIDs.

Youth Tracker fields remain save-scoped because they are cumulative career totals used by Academy, including for previously verified members who may no longer resolve in the latest snapshot. Both import formats retain the existing bounded parsing, exact UID matching, outside-lock preparation, and transaction-time context revalidation.

### Snapshot and save management

Rust exposes active-save snapshot metadata ordered by the shared comparator, validates bounded custom names, and owns transactional rename and deletion operations. The management migration assigns every existing and future save and snapshot an immutable, non-user-visible context token because their SQLite integer IDs may be reused after deletion. Mutations identify their target by ID and token. Deleting a current snapshot reruns the selector before commit. Deleting a non-current snapshot leaves the current marker unchanged. Snapshot deletion relies on existing snapshot foreign keys plus the new Moneyball ownership and must leave save-scoped Planner, Academy, and Youth data intact.

Save deletion uses the existing save ownership graph. Deleting an inactive save leaves the active save unchanged. Deleting the active save activates the first remaining save in the existing ascending save order; when none remains, it creates the existing `Default save` and its baseline Academy state inside the same transaction.

All destructive services and outside-lock operations target captured ID/token identities rather than a reusable numeric ID alone. An in-flight Load Data operation whose captured save is deleted fails token revalidation and cannot write into a fallback or newly created save that later receives the same row ID. CSV import likewise rejects a deleted-and-recreated save or snapshot even when integer IDs are reused. Player writes preserve the stronger bridge request-ID comparison and their existing explicit partial-outcome boundary when FM changed before SQLite reconciliation.

### Dashboard management surface

The existing Dashboard remains the only management location. A snapshot-history panel lists date, custom name or date fallback, player count, load time, and latest state for the active save. Rename is non-destructive. Snapshot and save deletion use the established accessible destructive Modal pattern, name the exact target and cascade scope, retain recoverable input on failure, and restore focus on close.

Snapshot-feature mutations invalidate snapshot-owned queries locally and call a route-provided context-change callback when current data or the active save may change. The Dashboard route then invalidates Search, Player, Planner, and Academy roots without introducing forbidden sibling-feature imports. CSV import state continues to reset from the active save/current snapshot identity.

## Uncertainty register

### Known

- Snapshot rows already persist a nullable in-game date and a load timestamp, and snapshot-owned player data already cascades from the snapshot.
- The database currently enforces at most one current snapshot per save with a partial unique index.
- Every existing Search, Player, Planner, Academy candidate, and sanity query resolves through the active save's current snapshot.
- Existing successful ingest retains every complete snapshot and selects the shared date-order winner in the same transaction.
- CSV import already captures and revalidates the active save and current snapshot. Youth enrichment remains save-scoped; new Moneyball rows are snapshot-scoped, while all v17 Moneyball rows remain unread in a save-scoped legacy quarantine.
- The bridge dump contract permits `gameDate: null`; a non-null bridge date is documented as `YYYY-MM-DD`.
- The feature branch is at migration v19; the base branch was clean at `b7b81d3e11c08bf660f19b9eef8ecadf0a08632e` with migration v17.

### Assumptions

- Existing databases created through supported app paths have at most one current snapshot per save. No existing Moneyball row has a persisted source-snapshot identity; current UID membership and timestamps are insufficient proof after replace-only ingest.
- Ascending save ID remains the existing user-visible save order and is an acceptable deterministic fallback when the active save is deleted.
- A custom snapshot name can use the existing save-name length and trimming policy unless implementation evidence shows a shared validator would create an unwanted coupling.
- Historical metadata volume is small enough for one ordered active-save snapshot query; no pagination or retention cap is needed for this solo desktop app.
- Browser-stub smoke plus Rust migration/service tests are sufficient automated evidence; real WebView, SQLite file, and live-FM integration remain separately stated risks if a native run is unavailable.

### Decisions

- Greatest in-game date defines latest. Load order is only a tie-break for equal dates or the fallback when every snapshot is undated.
- Dated snapshots always sort ahead of undated snapshots. An undated load cannot displace a dated current snapshot.
- Earlier loads are retained but do not become current. Rolling a save back is expressed by deleting every later-dated snapshot rather than by manually selecting an older row.
- Same-date and otherwise duplicate snapshots are all retained; the newest load for that date wins the tie.
- Snapshot names are organization metadata and never override date ordering.
- Save and snapshot management introduces immutable internal context tokens so deletion cannot make an old asynchronous operation valid again through SQLite row-ID reuse.
- New Moneyball enrichment becomes snapshot-scoped. All v17 Moneyball rows remain quarantined and save-scoped rather than being lost or assigned to an unproved snapshot. Youth Tracker career enrichment stays save-scoped.
- Academy automatic class creation runs only for the effective current snapshot, including a snapshot promoted by deletion. Non-current stored snapshots have no Academy side effect.
- Search and all existing player-facing features always use latest. Historical player browsing and development tracking are deferred.
- Deleting the last save recreates a blank active `Default save` instead of permitting a no-save state.
- No automatic cleanup or retention limit is part of this feature.

### Unknowns

- Whether any real pre-feature database contains a malformed non-null `game_date` despite the bridge contract. Build must use migration/fixture evidence and stop rather than silently reinterpret such a value if one is found.
- Whether the final native desktop flow can be exercised on a machine with FM26 during this feature. If it cannot, final reconciliation must preserve the explicit native-integration gap.

### Risks

- Lexical ordering is correct only for canonical dates; accepting malformed strings could select the wrong latest snapshot.
- Rebuilding Moneyball ownership can lose or misrepresent legacy values unless the complete v17 source table moves to quarantine transactionally and the new snapshot-owned table starts empty.
- SQLite can reuse deleted integer primary keys; stale Load Data, CSV, or UI work could mutate a different logical save or snapshot without token revalidation.
- Deleting a current snapshot or active save can expose stale cached players unless every affected query root is invalidated from the composition layer.
- A snapshot cascade can accidentally remove save-scoped Academy or Planner state if deletion logic bypasses the established ownership graph.
- Loading an earlier snapshot changes the bridge's live scan index while leaving a later-dated database snapshot current. Player boosts must remain fail-closed and give a truthful Load Data recovery path.
- Management controls are destructive and require precise target copy, pending-state protection, focus restoration, and failure-local errors. Stale async work must not retarget newly active data.

## Walking skeleton

Load a later-dated dump and then an earlier-dated dump into one save. Both snapshot rows and their player sets survive, the later date remains `is_current`, the Load Data outcome explains that the earlier snapshot was stored without becoming latest, and Search still returns only players from the later snapshot. This proves the persistence, ordering, current-only read seam, and rollback-cleanup foundation before management and enrichment presentation are added.

## Delivery plan

### PR 1 — Retain and manage snapshot history

**Status:** Ready for publication

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** `feature/snapshot-history`

**Base branch:** `main`

**Base ref:** `b7b81d3e11c08bf660f19b9eef8ecadf0a08632e`

**Publication provider:** GitHub

**PR template:** `.github/pull_request_template.md`

**Merge method:** squash

**Required checks:** strict `check`

**Feature close-out:** Not run

**CI repair rounds:** 0

**Build-feature-loop profile:** Terra Max — the PR crosses SQLite ownership, date selection, destructive lifecycle operations, CSV concurrency, and several current-only consumers.

**Provisional PR title:** `feat(snapshot): retain and manage snapshot history`

**Purpose:** Deliver one reviewable persistence contract in which app saves retain dated snapshots, normal features consistently read the greatest date, Moneyball values remain attached to their snapshot, and users can safely organize and delete stored history.

**Depends on:** Completed snapshot ingest, CSV enrichment persistence, Youth Academy, and action-specific player-boost provenance already present on `main`.

#### Commit 1 — Retain snapshots by in-game date

**Status:** Completed

**Provisional commit:** `feat(snapshot): retain snapshots by in-game date`

**Work:** Replace destructive current-snapshot replacement with atomic retention and the shared date selector; preserve current-only domain reads and current-only Academy class creation; return a truthful stored-versus-latest Load Data result; and cover the live-provenance interaction without widening bridge behavior.

**Out of scope:**

- Moneyball table ownership and CSV write changes.
- Snapshot naming, listing, or deletion commands.
- Save deletion.
- Historical player reads or UI selection.

**Implementation packet:**

- **Owners and files:** `src-tauri/src/features/snapshot/ingest.rs`, `load_data.rs`, `query.rs`, and `commands.rs`; targeted date validation under `src-tauri/src/features/memory_read/` if the current validator does not enforce the documented non-null format; Load Data types/outcome and focused tests under `src/features/memory-read/`; existing snapshot test support where the current IPC result is stubbed.
- **Existing patterns to verify:** transaction-first complete ingest, the partial unique current index, current snapshot DTO mapping, Load Data context capture, current-only joins, Academy trusted-date class creation, the outcome live region, and the player-boost request-ID fail-closed path.
- **Constraints and invariants:** retain every complete ingest; reject rather than order a malformed non-null date; select dated greatest-first with equal-date/load/ID ties; make one undated row current only when no dated row exists; leave all normal queries on `is_current`; create an automatic Academy class only for the selected current row; never claim an earlier stored row became latest.
- **Dependencies and ordering:** this commit establishes the history seam before management. Moneyball remains save-scoped temporarily and must be migrated before the PR can finish or publish.

**Implementation profile:** Terra Max — the change rewrites the central ingest lifecycle and must preserve atomicity, date truth, current-only consumers, and write provenance.

**Review profile:** Sol xhigh — a wrong selector or deletion remnant can expose stale players, discard snapshots, or authorize misleading downstream state.

**Validation:** Add focused Rust tests that first fail because the old ingest deletes the prior row and always promotes the newest load; add a focused Load Data outcome test that first fails because the result cannot distinguish stored from latest. Then run `./scripts/dev test src/features/memory-read/components/load-data-outcome.test.tsx src/features/snapshot/components/snapshot-panels.test.tsx`, `./scripts/dev check`, and `./scripts/dev smoke`. GREEN evidence must include later-then-earlier, earlier-then-later, equal-date, dated-versus-undated, all-undated, invalid-date, failed-ingest rollback, removed-player invisibility, current-only Academy class creation, and truthful outcome cases. A composed player-boost test must use equal CA/PA values across later request R2 and subsequently loaded earlier request R1, prove R2 remains database-current and is the prepared/outgoing source request, and prove the existing bridge mismatch contract rejects R2 against R1's live index.

**Stop conditions:** Stop and replan if supported existing data contains noncanonical non-null dates; if one shared comparator cannot serve ingest and later deletion promotion; if current-only consumers depend on newest load rather than `is_current`; or if the boost path can bypass its existing request-ID provenance check.

**Review mandate:**

- Prove old snapshots are retained only after a complete successful ingest and failures preserve the previous current row.
- Check the comparator for different dates, equal dates, null dates, timestamp collisions, and deterministic ID fallback.
- Trace a removed player through Search, Player, Planner, and Academy candidate reads to confirm history does not leak.
- Verify Load Data copy distinguishes the inserted row from the effective latest row, including truncated scans.
- Verify the explicit R2-current/R1-live regression proves the live bridge scan cannot authorize a player boost for a different current snapshot even when player values match.
- Verify a non-current ingest creates no Academy class and a newly selected current ingest creates only its trusted-date class.
- Check that no retention limit, deduplication, or history-selection API slipped into scope.

#### Commit 2 — Version Moneyball data by snapshot

**Status:** Completed

**Provisional commit:** `feat(import): version Moneyball data by snapshot`

**Work:** Add the next SQLite migration to preserve the complete v17 Moneyball table as an unread save-scoped legacy quarantine, create empty snapshot/player-owned Moneyball persistence for future imports, and make those imports upsert only the captured effective latest snapshot while Youth Tracker behavior remains unchanged.

**Out of scope:**

- Season keys, import history, CSV file metadata, or Moneyball presentation.
- Snapshot versioning of Youth career totals.
- Changes to parsing, canonical calculations, row limits, or memory-field precedence.
- Assigning or exposing quarantined legacy Moneyball values whose source snapshot was already deleted by the old replace-only model.

**Implementation packet:**

- **Owners and files:** `src-tauri/src/db/migrations.rs` and `src-tauri/src/features/csv_import/service.rs`, with their existing migration, concurrency, replacement, cascade, and negative tests.
- **Existing patterns to verify:** monotonic `PRAGMA user_version` migrations, table-rebuild tests from an explicitly old schema, exact UID eligibility, parsing outside the database lock, transactional revalidation, and complete per-player Moneyball row replacement.
- **Constraints and invariants:** one current-format Moneyball row per snapshot/player; composite ownership rejects nonmembers; every v17 source row and timestamp survives unchanged in a quarantine that accepts no new imports and has no product read path; the new snapshot-owned table starts empty on upgrade; deleting a snapshot cascades its current-format Moneyball rows; deleting a save cascades both formats; Youth rows remain save-owned and survive snapshot deletion.
- **Dependencies and ordering:** depends on Commit 1 retaining snapshots. The migration must remain safe when applied directly from v17 and when the target save contains only its legacy current snapshot.

**Implementation profile:** Terra Max — the commit changes ownership of persisted user data and must prove upgrade safety, cascade scope, and stale-import behavior.

**Review profile:** Sol xhigh — migration or foreign-key mistakes can silently lose imported values or attach statistics to the wrong historical player state.

**Validation:** Add a migration RED test from a populated v17 database with both current and departed player UIDs, plus service RED tests showing two snapshots can retain independent new-format Moneyball rows while Youth remains save-scoped. Run `./scripts/dev check`. GREEN evidence must include complete legacy value/timestamp and row-count preservation, an empty new-format table immediately after upgrade, no quarantine read or write path, migration rollback on a forced write failure, idempotent migration application, current-only import matching, re-import replacement within one snapshot, historical-row preservation, snapshot cascade, save cascade across both Moneyball tables, Youth survival across snapshot deletion, and stale-context rejection when current changes during parsing.

**Stop conditions:** Stop and request a migration decision if the complete legacy table cannot be preserved unchanged beside the new table; if the quarantine would need a user-visible read, reassignment, or cleanup policy; if another current feature reads Moneyball only by save and cannot be adapted without a new product rule; or if SQLite cannot enforce snapshot/player ownership without weakening existing foreign keys.

**Review mandate:**

- Compare every source row and timestamp with the quarantined destination after a populated v17 upgrade, and prove the new table is empty.
- Verify the old table is not dropped before a failing backfill can roll back safely.
- Prove two snapshots for the same save and UID retain different Moneyball rows.
- Prove quarantined rows cannot appear in current or historical snapshot reads, cannot receive new imports, and cascade only with their save.
- Prove imports always bind to the effective latest snapshot rather than the most recently loaded row.
- Prove Youth career enrichment remains save-scoped and available to unresolved tracked Academy members.
- Check snapshot and save cascades against the intended ownership table.

#### Commit 3 — Add snapshot and save management commands

**Status:** Completed

**Provisional commit:** `feat(snapshot): add history management commands`

**Work:** Add immutable save/snapshot context tokens, optional snapshot naming plus ordered metadata, token-bound rename and delete services, current promotion, and transactional deletion of any save with deterministic fallback activation or blank default creation when the active save is removed.

**Out of scope:**

- Dashboard controls or browser IPC stubs.
- Viewing player data for a non-current snapshot.
- Undo, soft delete, trash, retention, or bulk snapshot deletion.

**Implementation packet:**

- **Owners and files:** `src-tauri/src/db/migrations.rs`; `src-tauri/src/features/snapshot/service.rs`, `query.rs`, `commands.rs`, and `mod.rs`; command registration in `src-tauri/src/lib.rs`; focused Rust tests beside those modules.
- **Existing patterns to verify:** save-name validation, `ensure_default_save`, baseline Academy creation, save ordering, command DTO conversion, Load Data and CSV context capture/revalidation, bridge request-ID provenance, the shared selector from Commit 1, partial unique active/current indexes, SQLite row-ID allocation, and all `ON DELETE CASCADE` relationships.
- **Constraints and invariants:** every migrated and newly inserted save/snapshot has a unique immutable context token without adding a security dependency; metadata and mutation DTOs carry the token only as internal identity; metadata list is limited to one requested/active save and uses the shared order; custom name is bounded and trimmed; rename changes no ownership or order; current deletion promotes atomically; deleting the last snapshot leaves no current; inactive-save deletion preserves active; active deletion chooses the first remaining save; final-save deletion creates exactly one active `Default save` with baseline state.
- **Dependencies and ordering:** depends on the shared selector from Commit 1 and snapshot-owned Moneyball cascade from Commit 2. Commands must return enough context for precise frontend invalidation without exposing historical player data.

**Implementation profile:** Terra Max — destructive transactions cross all persisted save children and must preserve both active-save and current-snapshot uniqueness under every boundary case.

**Review profile:** Sol xhigh — an error can delete beyond the named target, leave the app without an active context, or promote the wrong snapshot.

**Validation:** Add Rust RED tests for the absent metadata/rename/delete services and commands, then run `./scripts/dev check`. GREEN evidence must cover token backfill and uniqueness, ordered metadata with dated/equal/undated rows, name validation and non-ordering, stale-token rejection, non-current deletion, current promotion with trusted-date Academy class creation, final-snapshot deletion, inactive-save deletion, active-save fallback, final-save default recreation, Planner/Academy/Youth preservation on snapshot deletion, full cascade on save deletion, nonexistent IDs, transaction rollback on failure, stale CSV/player reconciliation, deliberate save/snapshot integer-ID reuse, and a Load Data scan whose captured save is deleted and recreated before ingest without retargeting the replacement.

**Stop conditions:** Stop if immutable context tokens cannot be backfilled and enforced without rebuilding unrelated ownership tables; if any existing save-owned table lacks the expected cascade or would be orphaned; if deleting an active save cannot preserve exactly one active replacement transactionally; or if the shared comparator is duplicated rather than reused for promotion.

**Review mandate:**

- Trace every foreign-key cascade for snapshot deletion and save deletion.
- Force SQLite integer-ID reuse and prove token validation rejects every stale operation.
- Verify rename and list operations cannot change current selection.
- Verify current promotion uses the identical comparator as ingest.
- Verify promotion applies the Academy trusted-date class rule without deleting existing save-scoped classes.
- Exercise zero, one, and multiple remaining snapshot/save boundaries.
- Verify fallback save activation and baseline Academy creation commit atomically.
- Verify in-flight Load Data, CSV import, and player-write work cannot retarget after snapshot/save deletion.
- Confirm commands expose metadata only and do not introduce historical player reads.

#### Commit 4 — Manage snapshot history from the Dashboard

**Status:** Completed

**Provisional commit:** `feat(snapshot): manage history from the Dashboard`

**Work:** Add typed frontend APIs and an active-save snapshot management panel, snapshot rename and delete controls, per-save deletion controls, accessible confirmations, route-owned cross-feature invalidation, browser IPC stubs, and focused component/smoke coverage.

**Out of scope:**

- A historical profile/search route, snapshot selector, compare action, charts, or development tracking.
- Bulk actions, undo, pagination, retention settings, or automatic cleanup.
- Visual redesign of unrelated Dashboard, Search, Planner, Profile, or Academy surfaces.

**Implementation packet:**

- **Owners and files:** snapshot API/types/components under `src/features/snapshot/`; `src/app/routes/index.tsx` and `index.test.tsx`; shared test IPC state under `src/testing/`; `e2e/tauri-ipc-stub.ts` and `e2e/smoke.spec.ts`; the existing Modal/Button/TextField primitives without modifying them unless a demonstrated accessibility gap requires it.
- **Existing patterns to verify:** `SaveSwitcher`, `SnapshotPanelsWithErrorBoundary`, current snapshot query options, snapshot query-key factories, Academy destructive Modal focus/error behavior, route-owned sibling-feature invalidation, and CSV result reset on save/snapshot identity changes.
- **Constraints and invariants:** newest date renders first with undated rows last; latest is explicit; names never replace visible date metadata; destructive copy names the target and cascade; duplicate submission is blocked; a dialog retains the target ID/token identity even if active context changes; errors remain with the dialog; focus returns; current/save-changing success invalidates every current-only consumer; non-current rename/delete avoids unnecessary global churn where practical.
- **Dependencies and ordering:** depends on all three backend commits. The Dashboard route supplies cross-feature invalidation; snapshot feature modules remain isolated from Search, Player, Planner, Academy, and CSV internals.

**Implementation profile:** Terra xhigh — the work is a bounded React/TanStack Query surface, but destructive state, stale modal context, and multi-root invalidation require careful integration.

**Review profile:** Sol High — consequences are primarily interaction correctness, accessibility, and cache truth after destructive mutations rather than new persistence policy.

**Validation:** Add component RED tests for the missing ordered list, rename flow, confirmation scope, promotion result, inactive and active save deletion, save fallback, focus restoration, pending protection, stable target identity, and failure-local errors. Run `./scripts/dev test src/features/snapshot/components/snapshot-panels.test.tsx src/app/routes/index.test.tsx`, `./scripts/dev smoke`, and `./scripts/dev check`. GREEN smoke must cover listing two dated snapshots in date order, renaming without reordering, deleting a non-current row, deleting the latest row and observing promoted current data, deleting an inactive save without switching context, and deleting the final save into a blank `Default save`.

**Stop conditions:** Stop if the UI requires sibling-feature imports to invalidate correctly; if a destructive Modal cannot retain target identity across save switches; if the browser stub would falsely imply native SQLite proof; or if management cannot fit the existing Dashboard hierarchy without a broader design decision.

**Review mandate:**

- Verify list order, latest labeling, undated copy, and date visibility independently of custom names.
- Verify every destructive confirmation names the exact snapshot/save and truthful cascade scope.
- Check keyboard flow, initial focus, focus restoration, busy states, and error recovery.
- Trace invalidation after non-current delete, current promotion, active-save deletion, and default recreation.
- Verify CSV state and every current-only screen refresh against the new active/current identity.
- Confirm no historical player view or manual current selector is exposed.

## Active work

**PR:** PR 1 — Retain and manage snapshot history

**Commit:** Commit 4 — Manage snapshot history from the Dashboard

### RED proof

Add focused Dashboard tests for ordered metadata, rename, destructive confirmation, current promotion, inactive and active save deletion, final-save replacement, focus restoration, pending protection, stable target identity, and dialog-local failures.

### Expected outcome

The Dashboard manages date-ordered snapshot metadata and destructive save/snapshot actions while current-only product queries refresh only when their active/current context changes.

### Explicit exclusions

- Do not expose historical player data or a manual current selector.
- Do not add undo, soft deletion, retention, or bulk deletion.

## Discoveries and replanning

- 2026-08-11: Initial discussion selected greatest in-game date over last load as the authoritative latest rule. Loading an older career state must retain it without displacing later data; deletion is the rollback mechanism.
- 2026-08-11: The bridge legitimately permits a null game date. Accepted fallback: dated rows always win; an all-undated save uses newest load; equal dates use newest load with ID as final deterministic tie-break.
- 2026-08-11: Existing CSV enrichment has two different temporal meanings. Moneyball values are snapshot measurements and will move to snapshot ownership; Youth Tracker values are cumulative Academy career totals and remain save-owned.
- 2026-08-11: Retaining earlier loads separates database latest from the bridge's most recent live scan. The plan preserves the existing request-ID fail-closed player-write boundary and requires truthful recovery rather than redefining current by load order.
- 2026-08-11: The originally backlogged history item included browsing/selecting old snapshots. Product scope now limits this feature to persistence and management metadata; historical player views and development tracking remain deferred.
- 2026-08-11: Plan review found that v17 stored no Moneyball source-snapshot identity and rows can outlive both their original snapshot and player membership. The migration now quarantines every v17 row as unread save-scoped legacy data instead of failing app startup, losing data, or inventing an association from current membership.
- 2026-08-11: Plan review made two current-only interactions explicit: historical loads need a composed player-boost provenance regression, and only a snapshot that becomes current may create an automatic Academy class.
- 2026-08-11: Migration v19 can backfill and preserve immutable context tokens with additive columns, unique indexes, and SQLite triggers, without rebuilding unrelated ownership tables.

## Completed work

| PR | Commit | Git ref | Implementation | Review | Deviations |
| --- | --- | --- | --- | --- | --- |
| PR 1 — Retain and manage snapshot history | Commit 1 — Retain snapshots by in-game date | 1be3dd4 | Retained complete snapshots with shared date selection, truthful Load Data metadata, and current-only compatibility. | Sol xhigh accepted after a focused timestamp-precedence test correction. | None |
| PR 1 — Retain and manage snapshot history | Commit 2 — Version Moneyball data by snapshot | 730c812 | Migrated new Moneyball imports to snapshot/player ownership while quarantining all v17 rows by save. | Sol xhigh accepted. | None |
| PR 1 — Retain and manage snapshot history | Commit 3 — Add snapshot and save management commands | 120c333 | Added immutable target tokens plus rename/delete commands, current promotion, and transactional save fallback/default recreation. | Sol xhigh accepted after rollback coverage for final-save replacement. | None |
| PR 1 — Retain and manage snapshot history | Commit 4 — Manage snapshot history from the Dashboard | Pending record | Added Dashboard history management with accessible rename/delete flows and route-owned current-only refreshes. | Sol High accepted after one correction round. | None |

## Final validation

**Feature review profile:** Sol xhigh — final review must cross-check data retention, two migrations, destructive cascades, current-only reads, CSV concurrency, cache invalidation, and bridge provenance across all commits.

Required automated evidence before feature review:

- `./scripts/dev format`
- `./scripts/dev test`
- `./scripts/dev check`
- `./scripts/dev smoke`
- `git diff --check b7b81d3e11c08bf660f19b9eef8ecadf0a08632e...HEAD`
- Populated v17-to-latest migration proof preserving every existing Moneyball row in an unread legacy quarantine while the new snapshot-owned table starts empty.
- Rust service proof for the complete date/null/tie matrix, ingest rollback, context-token backfill and forced row-ID reuse, current promotion, snapshot/save cascades, final default recreation, and stale Load Data/CSV context.
- Frontend and smoke proof for date ordering, stored-versus-latest Load Data copy, rename, destructive confirmations, current promotion, save fallback, focus recovery, and cross-feature refresh.

Manual native evidence target:

- If a supported Windows FM26 environment is available, load a later date and then an earlier date, restart the app, confirm Search remains on the later date, rename and delete snapshots until the earlier date becomes latest, import Moneyball into the latest row, and delete the save.
- If native execution is unavailable, record that exact gap during `$workflow-finish-feature`; Playwright browser IPC stubs must not be presented as proof of real WebView, SQLite-file, dialog, or live-FM integration.

`./scripts/dev bridge-test` is not planned because no bridge source or protocol changes are in scope. Run it and replan if implementation crosses that boundary. `./scripts/dev mutate` remains unsupported and must not be reported as passed.

## Documentation impact

Complete during feature reconciliation after implementation is true:

- Update `.wiki/ARCHITECTURE.md` with retained snapshots, authoritative date selection, new snapshot-owned Moneyball rows, the unread save-scoped v17 Moneyball quarantine and its save cascade, save-owned Youth enrichment, immutable context tokens, management commands, and current-only automatic Academy class creation on ingest and promotion without deleting existing save-scoped classes.
- Update `.wiki/CONCEPT.md` so current-only product reads and snapshot persistence are no longer described as replace-only, while development analysis remains deferred.
- Update `.wiki/DESIGN.md` with the implemented Dashboard snapshot/save management surface, destructive states, and date/name presentation. Keep profile history and comparison surfaces deferred.
- Reconcile `.wiki/TODO.md`, the historical Moneyball backlog wording, and this ledger; move the condensed record to `.wiki/features/completed/` during `$workflow-finish-feature`.
- Assess whether the completed CSV enrichment record needs a short supersession pointer without rewriting its historical delivered-state account.
- No ADR is planned: the feature extends existing save/snapshot ownership and can record its durable boundary in current-state architecture. Replan if implementation requires a new cross-feature ownership mechanism or a different latest-selection policy.
