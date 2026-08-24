# 0024 — Eager persisted Club DNA scores

## Status

Accepted

Implementation status: Current at feature branch HEAD `7d21fa8e73666979862153246ac441f15fc90dfc`, across Commits 1–13 plus the reviewed correction. Migration v32 and `club_dna_scores` are implemented. No migration v33, new index, or dependency was added.

## Context

Club DNA derives one nullable 0–100 score from a save-owned selection of player attributes. Search must display, filter, and sort that score. Squad must display and sort it. Both interfaces return bounded pages.

ADR-0023 chose lazy persistent materialization because direct JSON calculation breached the 2,000-player interaction threshold. Later indexed and completeness-fast-path attempts reduced repeated materialization work, but the final discarded 184,000-player run still measured warm p95 of 514.287 ms for Search filter, 611.003 ms for Search sort, and 1596.226 ms for Squad sort. The corresponding 2,000-player p95 values were 4.129 ms, 6.527 ms, and 11.339 ms. The representative Squad sort breached the accepted one-second gate.

Current role scores use stored rows with scalar dynamic SQL for display and sort and an `EXISTS` subquery for filters. Those scalar sort patterns do not guarantee null-last ordering in both directions. The repository has no representative role-score latency gate. Club DNA is also one persisted score per player, while ingest already calculates about 70 role scores per player. The developer chose to match the persisted lifecycle and read model instead of retaining a unique lazy materializer and unique representative publication gate. Club DNA deliberately diverges only in its sort wrapper to guarantee null-last ordering.

## Decision

Reuse migration v32 and `club_dna_scores`. Do not add migration v33, another index, or a dependency. The row identity remains `(snapshot_id, uid, definition_version, score_model_version)`, and each row stores a nullable score for one exact definition and score-model version.

Definition set or edit performs one atomic transaction. It validates the active save context and definition, updates the definition and version, deletes prior score rows for that save, then calculates and persists one exact-version/model row for every player in every retained snapshot of that save. A calculation or write failure rolls back the definition and all score changes, so callers continue to see the prior definition and scores. This synchronous cost scales with the number of retained snapshots and players.

Snapshot ingest calculates Club DNA inside the existing ingest transaction after players and role scores when the save has a definition. It uses the validated dump or stored player data and the same pure Rust scorer, then stores one nullable exact-version/model row for every player in the new snapshot. A save without a definition does no Club DNA work. Any Club DNA failure rolls back the complete ingest.

A successful supported player-boost reconciliation recalculates and upserts the exact snapshot/player score inside the same SQLite source-data transaction when a definition exists. It deletes no valid row merely to defer work. A calculation or write failure rolls back the stored player update, current role-score changes, and Club DNA score together. The bridge can already have changed Football Manager before this SQLite transaction starts, so the command maps any eager Club DNA reconciliation failure to `PlayerBoostError::SnapshotSync` and requires Load Data recovery; it does not claim a full rollback of the external FM change. Definition removal deletes the definition and its scores atomically. Existing snapshot and player foreign-key cascades remain responsible for row cleanup.

Search and Squad resolve fixed `club_dna` against persisted rows with the request-bound current definition version and Rust-owned score-model version. As with current role scores, a nullable scalar expression supplies display and filters require a matching non-null persisted score. Club DNA deliberately does not copy the current-role sort ordering because that scalar pattern does not guarantee null-last in both directions. Its sort wrapper orders by `score IS NULL ASC`, then the requested score direction, then UID ascending. A missing definition or exact row produces null; a Club DNA filter is false for that player; a Club DNA sort retains all players in null-last, UID-stable order. SQL interpolates only the fixed metric ID and validated sort direction. Snapshot, definition version, score-model version, filter values, limits, and offsets remain bound. Existing request-field, filter-rule, and page bounds remain unchanged. Current-role and potential-role ordering remain unchanged, and general role-score optimization is out of scope.

Delete the Club DNA lazy batch materializer, cache invalidation paths, completeness instrumentation, and their obsolete tests together. Preserve migration v32 characterization, the pure scorer, schema/version tests, and supported query contracts. Do not add pre-query materialization, a completeness gate, background work, or a Club DNA-specific latency publication gate.

## Alternatives considered

### Continue the indexed lazy design

Retain exact-count completeness and optimize the representative Squad sort further. This keeps definition edits cheaper, but it preserves a unique materialization lifecycle and a feature-specific representative gate for one score. The final measured implementation still breached that gate. Rejected after the measured breach and the developer's decision to match the persisted role-score lifecycle and read model.

### Add migration v33 indexes

Add directional, null-ordering, or cohort-specific indexes to reduce the final lazy query cost. This could improve one or more measured shapes, but it adds storage, write amplification, migration and query-plan obligations, and more Club DNA-specific architecture. Rejected because the chosen lifecycle removes the need for the unique lazy gate and reuses the existing stored-score query pattern.

### Calculate only the current snapshot on definition change

Calculate the current snapshot and defer historical work until promotion. This reduces definition-save cost, but promotion could expose a historical snapshot without scores and would create another lifecycle branch. Rejected because eager calculation across retained snapshots gives every promotable snapshot the exact current definition/model rows immediately.

## Consequences

### Positive

- Club DNA follows the established persisted current-role score lifecycle and read model, with one deliberate null-last sort-wrapper divergence.
- Search and Squad do no Club DNA materialization or completeness work before a read.
- Every retained snapshot has scores immediately after a definition change, so later promotion needs no Club DNA backfill.
- Definition, ingest, and removal remain fully atomic. The SQLite boost reconciliation transaction rolls back stored player, current role-score, and Club DNA changes together and maps failure to Load Data recovery when FM may already have changed.
- Migration v32, its nullable score domain, exact identity, index, and cascade ownership remain reusable.
- The feature no longer carries unique 2,000-player or 184,000-player Club DNA latency publication gates.

### Negative

- Definition creation and edit synchronously rescore every player in every retained snapshot of the save.
- Load Data does extra synchronous work for each player when a definition exists.
- Supported player boosts recalculate one Club DNA score even when the changed source field does not affect the definition.
- Retaining historical snapshots increases definition-save cost and score-table storage.
- Query performance now follows the existing role-score validation portfolio rather than a representative Club DNA performance contract.

### Follow-up

- Prove fully atomic eager definition create/edit across retained snapshots and ingest with and without a definition. Prove nullable scoring and rollback at those transaction boundaries.
- Prove boost reconciliation rolls back stored player, current role-score, and Club DNA changes in SQLite, maps eager Club DNA failure to `PlayerBoostError::SnapshotSync`, and requires Load Data recovery without claiming that FM rolled back.
- Prove read-only display and filtering through the existing bounded Search and Squad paths.
- Prove ascending and descending Search and Squad ordering with present, computed-null, missing, and stale rows. Also prove UID order when every row is null because the definition is missing.
- Remove lazy materializer, invalidation, completeness, and Club DNA performance tests in the eager-writer lifecycle commit before persisted query exposure.
- Keep current-role and potential-role ordering unchanged. Do not optimize general role-score sorting in this feature.

## Related work

- Feature record: [Club DNA](../features/completed/club-dna.md)
- Supersedes: [ADR-0023](./0023-lazy-club-dna-score-cache.md)
- Reuses: migration v32 and `club_dna_scores` from `d78f97f25497409f6c895a8ac5cdeb74ea5301eb`
- Decision context: discarded final 184,000-player p95 of 514.287/611.003/1596.226 ms and 2,000-player p95 of 4.129/6.527/11.339 ms
