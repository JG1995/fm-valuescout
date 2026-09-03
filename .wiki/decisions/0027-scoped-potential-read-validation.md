# 0027 — Scoped potential read validation

## Status

Partially superseded by [ADR-0028](./0028-compact-current-snapshot-metrics.md) and its missing-FM26 amendment.

- **Superseded:** the normalized `player_potential_role_scores` row scope and per-role `(snapshot_id, role_id, projection_model_version, score)` completeness counts retired by the compact `player_role_metrics` wide row (68→79 columns) and by the per-snapshot `snapshots.compact_score_model_version INTEGER NOT NULL DEFAULT 1 CHECK IN (1,2)` availability marker introduced for the 79-role catalog. That marker revisits the rejected "persisted completeness marker" alternative from this ADR — but only as a snapshot-level availability version (`1` = legacy 68, `2` = full 79), not as a general completeness blob — and its SQLite `NOT NULL DEFAULT 1` semantics (existing rows store `1`, omitted inserts store `1`, explicit `NULL` rejected) plus explicit `bind 2` on every fresh ingest are now the authoritative availability signal.
- **Retained (still valid):** the scoped-read principle — ordinary reads validate only the compact model/version and values they consume, while lifecycle writers own complete current-snapshot postconditions — plus the width/index reasoning for scoped identifier validation. These principles are reused for the new provenance-aware `assert_read_models_complete` / `assert_snapshot_complete` (including `IS NULL` checks on the 11 new columns for provenance `1` via `LEGACY_V1_ROLE_IDS` set membership and dual `score_model_version`+`projection_model_version` enforcement for any `potential_role.*` read).

## Context

ADR-0026 introduced complete eager potential-role persistence and a snapshot-wide read assertion. The assertion checks every current player, all 47 projected attributes, and all 68 catalog role rows. Ordinary potential reads repeated that work while they held the application's single SQLite connection mutex.

A representative save can contain approximately 250,000 players. Direct Planner depth reads run from the `/my-club` route loader even when no assignment needs potential data. Search, Squad, role reference, and Player Profile also called the same full assertion before narrower reads. One assertion could therefore block unrelated IPC commands and make navigation appear frozen.

The atomic migration, snapshot-selection, promotion, ingest, and boost writers still own complete potential materialization. Product reads must remain read-only and must reject missing or wrong-version data that affects their result.

## Decision

Keep the snapshot-wide assertion for migration and materialization postconditions and for existing Planner mutation preflights. Do not call it from ordinary product reads.

Validate only the potential data that each read consumes:

- Player Profile validates the requested player's exact projected-attribute map and complete catalog role set.
- Search and Squad collect the validated potential role IDs used by requested fields, filters, or sorting. They compare an indexed exact-version row count with the current snapshot player count for only those roles.
- Planner role reference validates the requested tactic roles for the managed-club players it loaded.
- Direct Planner depth reads validate the exact IP and OOP rows for each assignment that resolves to a current player. An empty depth or an unassigned player does not trigger a potential-data scan.

Keep all checks read-only. Return `Current potential snapshot is incomplete` when required data is missing or has the wrong projection-model version. Do not calculate, repair, or write potential data from a read.

## Alternatives considered

### Keep the snapshot-wide assertion on every potential read

This preserves the strongest fail-fast check. It also repeats work proportional to the full player and role catalog under the single database mutex. Rejected because it caused navigation stalls on large saves.

### Remove read validation

Atomic writers make incomplete current state unlikely, and exact-version joins remain read-only. A missing required row could still become a null value, excluded result, or changed sort order. Rejected because scoped checks retain that protection at much lower cost.

### Add a persisted completeness marker

A snapshot marker could make reads constant-time. It would require a migration and exact invalidation across every potential-data writer. The marker could also drift from its rows if a future writer missed the invalidation path. Rejected because existing indexes support narrow checks without another persisted lifecycle.

## Consequences

### Positive

- Route reads no longer validate every projected attribute and catalog role for every player.
- Missing or wrong-version data still fails before it can affect a result that needs that data.
- The change adds no migration, cache, background task, connection pool, or frontend state.
- Read paths remain calculation-free and write-free.

### Negative

- Corruption outside a read's requested players or roles no longer blocks that unrelated read. It fails when a consumer requests it or when an existing snapshot-wide mutation preflight runs.
- Search and Squad completeness work grows with the number of requested potential roles, although it uses the existing `(snapshot_id, role_id, projection_model_version, score)` index.
- Planner mutation preflights still run the snapshot-wide assertion. Revisit them only if measured mutation latency becomes a user problem.

### Follow-up

- Keep the direct-depth regression test that proves unassigned player potential state does not gate route loading.
- Keep scoped missing-row and wrong-version tests for Profile, Search, Squad, role reference, and assigned Planner depth reads.
- Reconsider a persisted completeness marker only if indexed requested-role checks become measurably slow.

## Related work

- [ADR-0015 — SQLite with Rust-owned migrations and queries](./0015-sqlite-rust-owned.md)
- [ADR-0026 — Eager current-snapshot potential scoring](./0026-eager-current-potential-scoring.md)
- [Ingest-Time Potential Scoring](../features/completed/ingest-potential-scores.md)
- Active amendment: [Complete missing FM26 attribute role definitions](../features/active/missing-fm26-attribute-role-definitions.md) and [ADR-0028 — Compact current-snapshot metrics](./0028-compact-current-snapshot-metrics.md) (amended — scoped principles retained, snapshot-marker rejection partially superseded by `compact_score_model_version`)
