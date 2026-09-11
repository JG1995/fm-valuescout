# Load Data Processing Performance

## Status

Active

**Ledger schema:** 2

## Delivery authorization

**Delivery fingerprint:** c0267b64328b79a666d49378e2989b78ec70d837353bb8a2227c8e37c7e96f61

## Intent

Remove three proven redundant costs from the established Load Data path without changing bridge success/failure handling, snapshot selection, derived-state ownership, or atomic publication.

## User-visible behavior

- Load Data keeps its existing result, error, progress, and retained-snapshot behavior.
- A bridge terminal result still reports whether `dump.json` exists. The captured dump remains authoritatively parsed and validated before publication.
- Repeated loads retain raw historical player rows and clear derived player state only when that state exists.
- A winning snapshot still stores projected attributes and compact metrics for every player in its one final transaction.

## Invariants

- `snapshot::load_data::prepare_raw_for_publish` remains the authoritative captured-dump read, parse, and validation path through `ingest::prepare_dump_json_raw`.
- Load Data keeps one `Db(Mutex<Connection>)`, one final transaction, context revalidation before writes, rollback on failure, and prior-current visibility until commit.
- The greatest valid game date, then load timestamp and ID, selects the current snapshot. Non-winning retained snapshots remain raw-only.
- The displaced current snapshot still has projected attributes, projection version, and compact metrics cleared in the same transaction. Existing NULL projected-state rows are not rewritten.
- Winning-snapshot projected attribute updates use the existing projection model version and update the same `(snapshot_id, uid)` rows before compact-row completeness checks.
- Progress and timing phase boundaries, Club DNA scoring, staff persistence, Academy writes, bridge protocol, schema, and public IPC contracts do not change.

## Non-goals

- JAY-68 work: detailed sub-phase instrumentation, replacing the full projected-JSON completeness scan, typed prepare/scoring representation, fixed-index scoring, skipping scoring for non-winning snapshots, and moving Club DNA scoring outside the transaction.
- A benchmark framework, permanent timing instrumentation, a performance claim, a migration, schema change, dependency, concurrency change, or UI change.
- Manual or native smoke testing as a delivery or pre-merge gate. The developer may smoke-test after merge.

## Current-state map

- Relevant components: `memory_read::service::wait_for_request_terminal` determines terminal status and currently reparses a complete bridge dump only to warn. `snapshot::load_data::capture_completed_dump` captures the request-bound file, and `prepare_raw_for_publish` reads, parses, and validates that private copy through `ingest::prepare_dump_json_raw`.
- Publication seam: `snapshot::ingest::publish_prepared_snapshot_canonical` owns the transaction, current selection, non-current derived cleanup, winning-player projection updates, compact rows, completeness assertions, staff rows, Academy update, and commit.
- Persistence: `players.potential_attributes_json` and `potential_projection_model_version` are current-only. `player_role_metrics` is current-only compact state. Historical snapshots retain raw player rows.
- Existing proofs: inline `memory_read::service` terminal-wait tests cover ready, failed, timeout, and request-ID matching. Inline `snapshot::ingest` tests cover current/history ownership, selection, rollback, one transaction, and winning derived-state persistence. `snapshot::commands` proves preparation runs without the database mutex and preserves phase ordering.
- Project validation commands: `./scripts/dev check-rust` runs Rust format, Clippy, and all Rust tests. `./scripts/dev check` adds the repository frontend and secret gates. `./scripts/dev smoke` is a Playwright WebView suite with IPC stubs and has no changed route or frontend interaction to prove here.
- Primary risks: moving validation could weaken the captured-dump boundary; a cleanup predicate could leave stale derived state or touch the current winner; a reused statement could alter parameter binding, error propagation, or transaction rollback.

## Feature architecture

Keep terminal polling limited to request-matched terminal status and dump-file presence. Keep captured-dump validation at the existing private-copy preparation seam. In canonical publication, limit the non-current projected-state cleanup to rows with at least one non-NULL derived column. Prepare the existing winning-player projected-attribute SQL once from the transaction and execute it for each prepared player with the same values and error mapping. These are local changes; no new abstraction or timing surface is needed.

## Uncertainty register

### Known

- `wait_for_request_terminal` currently calls `validate_dump_at_bridge_directory` after `dump_present` and ignores its error after logging a warning.
- The later captured-copy preparation path calls `prepare_dump_json_raw`, which delegates to `parse_and_validate_dump` before raw normalization.
- Canonical publication currently clears all non-current players regardless of whether both projected-state fields are already NULL and prepares the winning-player UPDATE through `tx.execute` for each player.
- Existing winner/history, rollback, and timing/progress tests provide the correct regression seams.

### Assumptions

- Preparing the same static projected-attribute UPDATE once inside the existing transaction is supported by the repository's `rusqlite` version and preserves its error behavior.

### Decisions

- Use one short-lived PR. The three outcomes are independent, trunk-safe commits in the same established Rust publication path; separate PRs would add no mergeable seam.
- Keep all delivery proof automated: each implementation commit runs the Rust gate and full repository gate. Automated smoke is not required because no browser route, frontend interaction, or IPC-stub behavior changes.
- Do not add an ADR. The work preserves ADR-0028's current-only state and one-transaction architecture; no consequential alternative is chosen.

### Unknowns

- No unresolved product or architecture decision blocks the first commit. The change must not claim a speedup unless equivalent measurements later exist.

### Risks

- A test that only checks successful loading could miss an invalid captured dump being accepted, stale historical state being retained, or a changed prepared-statement binding/rollback path.
- A broad cleanup predicate could rewrite historical NULL rows, while a narrow predicate that misses either column could retain invalid derived state.

## Walking skeleton

Record the reviewed plan, then remove the terminal reparse while proving captured-copy validation still rejects invalid data; next, skip no-op historical cleanup writes; finally, reuse the winning-player UPDATE statement while preserving the existing publication transaction and proofs.

## Delivery plan

### PR 1 — Streamline Load Data publication

**Status:** Active

**PR ref:** Not published

**Merge ref:** Not merged

**Branch:** feature/load-data-processing-performance

**Base branch:** main

**Publication provider:** GitHub

**PR template:** .github/pull_request_template.md

**Merge method:** squash

**Required checks:** GitHub required strict status check

**Feature close-out:** Not run

**CI repair rounds:** 0

**Provisional PR title:** `perf(load-data): streamline snapshot publication`

**Purpose:** Deliver three isolated Load Data cost reductions while preserving the current bridge, retained-history, and atomic-publication contracts in one reviewable PR.

**Depends on:** Clean `main` at `ef4af09b36c4a602995dd4d66fbb86d3e5b11b5c`; independent plan review, developer acceptance, and the accepted Delivery fingerprint.

#### Commit 1 — Record the approved feature plan

**Status:** Active

**Provisional commit:** `docs(load-data): record processing performance plan`

**Work:** Commit the independently reviewed ledger and TODO entry on the authorized feature branch before implementation.

**Atomicity:** The ledger and active TODO link are one planning-state outcome; either alone leaves the approved feature without its required source of truth or queue entry.

**Size assessment:** No implementation code; planning-only commit.

**Out of scope:**

- Implementation, tests, configuration, BACKLOG, planned specs, ADRs, and GitHub state.

**Implementation packet:**

- Preserve the accepted plan-review outcome and commit only the two reviewed planning paths after branch verification.

**Files and responsibilities:**

- `.wiki/features/active/load-data-processing-performance.md` — approved JAY-66 intent, delivery authority, and packets.
- `.wiki/TODO.md` — active feature link and Linear JAY-66 reference.

**Behavior and data flow:**

- Record one reviewed active ledger and one TODO pointer before implementation; no executable behavior changes.

**Ordered implementation steps:**

1. Verify the authorized branch and `main` base without changing Git state beyond the separately authorized activation.
2. Confirm the worktree contains only the reviewed ledger and TODO paths.
3. Run the ledger classifier and `git diff --check` on those paths.
4. Stage and inspect only the reviewed planning diff for independent checkpoint review.

**Tests and proof:**

- Not applicable — planning documents only. The ledger classifier proves schema-2 state and `git diff --check` proves the exact documentation diff is structurally clean.

**Patterns to verify:**

- `.wiki/features/active/README.md` and `.wiki/TODO.md` ownership rules.

**Constraints and non-goals:**

- Do not change plan scope, packet order, implementation, tests, configuration, BACKLOG, planned specs, ADRs, branches, or publication state.

**Dependencies and sequencing:**

- Requires a clear independent plan-review verdict, developer acceptance, accepted Delivery fingerprint, and exact branch activation.

**Validation:** `python3 /home/jonas/projects/PI_SETUP/scripts/ledger_state.py .wiki/features/active/load-data-processing-performance.md && git diff --check -- .wiki/features/active/load-data-processing-performance.md .wiki/TODO.md`

**Stop conditions:** Stop on an uncleared review, classifier error, unreviewed path, substantive plan change, missing accepted fingerprint, diff-check error, or branch mismatch.

**Review mandate:** Verify the staged diff is only the reviewed two-path planning outcome, has schema-2 Active state with one active PR and commit, preserves JAY-68 as Linear-only deferral, and contains no implementation changes.

#### Commit 2 — Remove terminal dump revalidation

**Status:** Pending

**Provisional commit:** `perf(memory-read): remove terminal dump revalidation`

**Work:** Remove the terminal-wait full-dump validation and warning so only the captured-copy preparation path parses and validates the dump.

**Atomicity:** Removing the redundant call and its now-obsolete helper/tests together leaves one authoritative validation seam; separating them would leave dead API/test protection or change the behavior without proof.

**Size assessment:** Within the soft target; one Rust service and its inline tests.

**Out of scope:**

- Bridge status polling semantics, dump capture, dump-validation rules, errors, logging unrelated to the removed warning, and ingest publication.

**Implementation packet:**

- Keep terminal polling responsible only for request-matched state and `dump.json` presence. Remove `validate_dump_at_bridge_directory`, its now-unreferenced `dump_validation::validate_dump_file` wrapper, and their direct test calls when no remaining caller exists. Retain the ready, failed, timeout, and request-ID behavior tests. Prove invalid content fails only at `load_data::prepare_raw_for_publish` after the dump has been captured.

**Files and responsibilities:**

- `src-tauri/src/features/memory_read/service.rs` — remove terminal full-file validation, warning, obsolete bridge-directory helper, and direct helper test use; retain terminal result behavior tests.
- `src-tauri/src/features/memory_read/dump_validation.rs` — remove `validate_dump_file` and its dedicated tests if the terminal helper was its only production caller; retain shared parse/value validation used by ingest and snapshot date handling.
- `src-tauri/src/features/snapshot/load_data.rs` — add or adjust the captured-copy preparation proof only if the existing test does not directly prove malformed captured JSON returns `LoadDataError::Ingest` before publication.

**Behavior and data flow:**

- `request_player_dump` still waits for a terminal request-matched status and returns `dump_present`. `capture_completed_dump` still binds the private copy to the ready request. `prepare_raw_for_publish` reads the private copy and `prepare_dump_json_raw` validates it before scoring or database publication.

**Ordered implementation steps:**

1. Establish the RED/contract-removal proof that terminal success no longer needs a valid complete dump while captured-copy preparation rejects malformed content.
2. Remove the terminal validator call, warning, and unreferenced helper/API.
3. Rewrite only obsolete direct-validation test assertions; retain terminal-status and captured-preparation behavior tests.
4. Run the automated Rust and repository gates.

**Tests and proof:**

- Modify inline tests in `memory_read/service.rs`; modify `snapshot/load_data.rs` only if needed for an observable captured-copy malformed-dump failure. Prove terminal ready with a present dump returns its normal result without parsing the full file, while `prepare_raw_for_publish` rejects malformed captured JSON before publication. Retain the existing request match, missing/failed status, capture replacement, and prior-current-on-ingest-error coverage because those contracts survive.

**Patterns to verify:**

- `load_data::capture_completed_dump`, `prepare_raw_for_publish`, and `ingest::prepare_dump_json_raw` as the existing request-bound private-copy validation sequence.

**Constraints and non-goals:**

- Do not weaken dump validation, accept an invalid captured dump, move validation to another bridge path, or change the bridge protocol/result DTO.

**Dependencies and sequencing:**

- Depends on Commit 1 only. It is independently mergeable before either publication optimization.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Stop and replan if another production caller requires `validate_dump_at_bridge_directory`, captured-copy preparation no longer validates before publication, or an invalid dump can reach scoring or SQLite.

**Review mandate:** Check that no terminal-wait parse/validation or warning remains, exactly one captured-copy parse/validation still protects publication, terminal status/result semantics remain unchanged, obsolete helper/tests are removed, and automated proofs cover the surviving boundary.

#### Commit 3 — Skip no-op historical projection cleanup

**Status:** Pending

**Provisional commit:** `perf(snapshot): skip cleared projection cleanup`

**Work:** Restrict canonical non-current projected-state cleanup to player rows whose projected JSON or projection model version is still non-NULL.

**Atomicity:** The SQL predicate and its history/current-selection regression proof jointly preserve the current-only cleanup contract while eliminating no-op historical rewrites; neither has a complete observable outcome alone.

**Size assessment:** Within the soft target; one transaction-local SQL change and inline ingest proof.

**Out of scope:**

- Compact-row cleanup, staff cleanup, snapshot ordering, raw history retention, projection/scoring formulas, and winning-player statement reuse.

**Implementation packet:**

- Change only the `players` cleanup in `publish_prepared_snapshot_canonical` so it selects non-current snapshots in the save and rows where either `potential_attributes_json` or `potential_projection_model_version` is non-NULL. Keep clearing a displaced current snapshot intact, including cases where only one derived field is non-NULL.

**Files and responsibilities:**

- `src-tauri/src/features/snapshot/ingest.rs` — narrow the existing non-current player projected-state UPDATE and add/adjust the inline lifecycle proof.

**Behavior and data flow:**

- After current selection, compact and staff derived state retain their existing cleanup. The player UPDATE clears stale projected state from every non-current row that has any derived value, skips rows already `(NULL, NULL)`, then winning publication materializes the selected new snapshot as before.

**Ordered implementation steps:**

1. Add a focused RED test that seeds retained historical raw-only rows plus stale/displaced projected-state rows.
2. Narrow the cleanup WHERE clause with explicit NULL checks for both columns.
3. Confirm winning/current selection and rollback proofs still pass; do not change later winning-player writes.
4. Run the automated Rust and repository gates.

**Tests and proof:**

- Modify inline `snapshot/ingest.rs` tests. Prove a repeated winning load clears the displaced current snapshot, preserves raw retained history/current selection, and does not update historical rows where both projected fields are already NULL; include a one-field-non-NULL row so the predicate clears partial stale state. Existing transaction rollback and raw-only non-winner tests remain the stronger proof for atomic visibility and history semantics.

**Patterns to verify:**

- `player_compact::clear_non_current_snapshots`, `ingest::prepare_and_publish_preserve_winner_and_raw_history_semantics`, and existing compact rollback tests.

**Constraints and non-goals:**

- Keep one transaction and the existing order: selection, non-current cleanup, winning derived materialization, completeness checks, and commit. Do not change any columns beyond the two projected-state fields.

**Dependencies and sequencing:**

- Depends on Commit 1 only. It is independent of Commit 2 and Commit 4, though it shares the canonical publication file.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Stop and replan if the predicate cannot clear a displaced current snapshot, partial stale state is possible but cannot be represented safely, current selection/history semantics change, or a failure can commit partial cleanup.

**Review mandate:** Check the predicate requires at least one non-NULL projected field, clears both fields for partial/stale rows, excludes already-cleared history, preserves selection/order and compact/staff cleanup, and retains transaction rollback/prior-current visibility proof.

#### Commit 4 — Reuse the winning-player projection statement

**Status:** Pending

**Provisional commit:** `perf(snapshot): reuse projected attribute statement`

**Work:** Prepare the winning-snapshot projected-attribute UPDATE once and reuse it for every prepared player in canonical publication.

**Atomicity:** One prepared SQL statement and the per-player execution loop form the complete same-transaction persistence outcome; splitting preparation from execution would create no independently useful or provable behavior.

**Size assessment:** Within the soft target; one transaction-local statement and inline publication proof.

**Out of scope:**

- Projection/scoring representation or completeness scan changes, batch SQL redesign, fixed-index scoring, non-winning scoring changes, Club DNA timing, and timing instrumentation.

**Implementation packet:**

- Replace per-player `tx.execute` calls in the `effective == snapshot_id` branch with one `tx.prepare` before the loop and `statement.execute` for each prepared player. Preserve the exact SQL, `(snapshot_id, uid, projected JSON, projection version)` bindings, `String` error mapping, statement lifetime before later completeness assertions, and transaction rollback.

**Files and responsibilities:**

- `src-tauri/src/features/snapshot/ingest.rs` — prepare and reuse the existing projected-attribute UPDATE in the winning-player loop; strengthen or retain inline winning/rollback proof as needed.

**Behavior and data flow:**

- The final transaction inserts raw data, selects the winner, performs cleanup, then uses one prepared statement to write each winning player's projected JSON and model version before compact rows and existing completeness checks. Non-winning snapshots remain raw-only. Any statement execution error aborts the same transaction.

**Ordered implementation steps:**

1. Identify the existing winner and rollback test that observes projected fields and prior-current visibility; add the smallest RED proof only if it does not cover multiple prepared players and statement-execution failure.
2. Prepare the static UPDATE once inside the winning branch and execute it for each player with unchanged values.
3. Remove no longer needed per-call execution only; keep compact persistence and assertions unchanged.
4. Run the automated Rust and repository gates.

**Tests and proof:**

- Modify inline `snapshot/ingest.rs` tests only where needed. Prove a winning multi-player snapshot persists each player's projected JSON and projection version, while a forced projected-attribute UPDATE failure rolls back the new snapshot and leaves the prior current visible. Retain existing one-transaction, winner/raw-history, timing-boundary, and compact completeness tests because they cover unchanged surrounding behavior.

**Patterns to verify:**

- `insert_prepared_players_raw` and `player_compact::persist_rows_borrowed`, which already prepare once and execute per row in the same transaction.

**Constraints and non-goals:**

- Preserve the one transaction, rollback, prior-current visibility, phase boundary placement, progress events, and exact model version. Do not add batching, a new helper, or a benchmark claim.

**Dependencies and sequencing:**

- Depends on Commit 1 only. It is independent of Commits 2 and 3 and must remain valid after their earlier merge order in this PR.

**Validation:** `./scripts/dev check-rust && ./scripts/dev check`

**Stop conditions:** Stop and replan if a prepared statement changes parameter ownership/lifetimes, cannot preserve the existing error mapping or rollback, moves outside the transaction, or alters timing/progress boundaries.

**Review mandate:** Check one statement is prepared only in the winning branch and reused for all players, bindings and model version are exact, no per-player `tx.execute` remains for this UPDATE, non-winners stay raw-only, error propagation rolls back, and phase/progress boundaries are unchanged.

## Active work

**PR:** PR 1

**Commit:** Commit 1

### RED or removal proof

Not applicable — independently reviewed planning documents only. The ledger classifier and exact-path diff check provide the planning proof.

### Expected outcome

A reviewed schema-2 ledger and TODO entry record the active JAY-66 feature, one PR, the planning commit, and three pending independent implementation outcomes.

### Explicit exclusions

No implementation, tests, BACKLOG, planned spec, ADR, configuration, branch, commit, or publication mutation.

## Discoveries and replanning

- Repository evidence confirms the terminal revalidation is a warning-only full-file parse after terminal status, while the captured-copy preparation path validates again before publication.
- Repository evidence confirms the two ingest changes are separate transaction-local SQL improvements and existing tests cover current selection, history, rollback, and timing boundaries. Replan if a focused proof exposes a changed persistence or publication contract.

## Completed work

| PR | Commit | Git ref | Implementation | Validation | Test portfolio | Review | Fix rounds | Deviations |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |

## Final validation

Run only automated commands before feature review and merge:

1. `./scripts/dev check-rust` — Rust format, Clippy, and all inline Rust regression tests, including the changed terminal-wait and canonical-publication seams.
2. `./scripts/dev check` — the required repository `check` gate before GitHub publication and strict `check` status before squash merge.

Do not require `./scripts/dev smoke`: this feature changes no frontend route, browser interaction, or IPC-stub contract, and the Playwright smoke suite cannot prove the native bridge or SQLite publication changes. Do not require a manual/native smoke test or timing capture before merge. Do not claim a speedup without equivalent measurements.

## Documentation impact

Complete during reconciliation. No current-state document, ADR, BACKLOG, planned spec, or release documentation change is expected because the supported behavior and architecture remain unchanged.
