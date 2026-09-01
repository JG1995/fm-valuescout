# 0026 — Eager current-snapshot potential scoring

## Status

Superseded by [ADR-0028](./0028-compact-current-snapshot-metrics.md).

Implementation status: The normalized eager representation was implemented by [Ingest-Time Potential Scoring](../features/completed/ingest-potential-scores.md). ADR-0028 preserves current-only eager ownership, projected attributes on `players`, atomic lifecycle rebuilds, and read-only consumers, but replaces normalized potential score rows and legacy-database backfill with one compact current-player metric row in a fresh database generation. This record remains the history of the prior design.

## Context

Potential visible attributes and potential role scores use one position-sensitive CA-to-PA projection. Player Profile and Planner reads calculate them directly. Search and Squad use `player_potential_role_scores` as a sparse, versioned, disposable cache and fill requested roles from read paths.

This split gives the same derived player state several lifecycle owners. A player can be projected again for a profile, an assignment, a role-reference request, an optimizer run, or a newly requested table role. Sparse table rows also require completeness checks, stale-version replacement, and boost invalidation on product reads.

Snapshot history keeps product behavior current-only. The effective current snapshot is already selected transactionally during ingest and deletion promotion. Current role scores are calculated during ingest, and eager Club DNA persistence provides a related derived-state pattern. Potential persistence differs from Club DNA because historical non-current snapshots do not need potential data.

The developer chose to pay the complete derivation cost when a snapshot becomes effective current. Every product read must use persisted projected attributes and potential role scores. Supported boosts must replace one changed current player's derived values atomically. A projection-model change must have an explicit migration or rebuild owner.

## Decision

Keep `player_potential_role_scores`, its normalized `(snapshot_id, uid, role_id)` identity, nullable score, projection-model version, player cascade, and Search/Squad index. Change its lifecycle from a sparse lazy cache to complete derived rows for only each save's effective current snapshot. Persist one row for every current-snapshot player and every role in the current catalog, including null scores.

Add nullable projected-visible-attribute JSON and projection-model-version fields to the snapshot-owned `players` row. Both fields remain absent on non-current snapshots. For a current player, the projected map and every potential-role row use one shared model version.

One Rust eager writer loads each target player once, calls `project_attributes` exactly once, persists that map, and derives every catalog role score from the same map. Migration backfill, snapshot selection, and boost reconciliation call this writer inside their existing SQLite transactions. Product reads never call it.

Add an explicit migration for the new projected fields and current-only backfill. The migration discards the disposable sparse cache, backfills every save's existing effective current snapshot, leaves historical snapshots empty, and advances `PRAGMA user_version` only in the transaction that completes schema and derived data. A failed backfill leaves the prior schema version active.

The shared effective-current selector owns later lifecycle changes. When ingest selects a new winner, it materializes that snapshot and clears potential data from the former current snapshot before commit. A newly stored non-winning snapshot stays empty. When deletion promotes a retained snapshot, the selector materializes the promoted row before commit. Save switching performs no scoring because each save's current snapshot is already materialized.

A successful supported player boost replaces the changed current player's projected map and complete potential role set in the same transaction as source-player and current-role changes. A local failure rolls back SQLite reconciliation and uses the existing Load Data recovery because Football Manager may already have changed.

A projection formula or model-version change requires a new explicit migration or pre-open rebuild that refreshes all effective current snapshots before reads. Incrementing the version constant without that owner is not valid. Missing, incomplete, or stale current derived data is an invariant failure, not a cache miss.

One shared `player_metrics::potential_scores` assertion protects that read contract without calculation or writes. For an already-resolved current snapshot, it checks that every player has non-null, valid projected JSON at the expected model version and that every `(player, current catalog role)` pair has its own row at that version. The SQL binds the trusted `all_roles()` catalog through a `VALUES` CTE and uses a per-role anti-join, rather than comparing only total row counts, so an extra or obsolete role cannot hide a missing catalog role. Profile and Planner consumers that expose potential data call the assertion before affected reads. Search and Squad call it only when validated requested fields, filters, or sorts require potential data.

Planner consumers also treat the assertion as a mutation-response precondition. After pure untrusted-request validation and current-snapshot resolution, `save_planner_teams`, `add_planner_string`, `remove_planner_string`, `clear_planner_depth`, `clear_planner_assignment`, `assign_planner_player`, and `move_planner_player` must assert the current snapshot before Planner setup, a mutation transaction, or any team, string, or assignment write. If no current snapshot exists, they keep their existing no-snapshot behavior and skip the assertion. Corrupt potential state must return the invariant error with teams, strings, and assignments unchanged, including confirmed destructive removal. Optimizer retains the same preflight before Planner setup or its assignment transaction because its response exposes Planner depth potential values.

Direct `get_depth` continues to assert before setup or assignment reads. An internal depth loader may hold only the existing post-resolution depth assembly so an already-preflighted mutation or optimizer can build its successful response without running the same assertion again. This loader is not a public unchecked read path and does not justify a generic validation framework.

## Alternatives considered

### Keep ADR-0019's lazy sparse cache

This avoids mandatory ingest work for roles the user never views. It also preserves repeated projection across profile and Planner reads, keeps write and completeness work in Search and Squad reads, and leaves boost reconciliation as invalidation rather than replacement. Rejected because the accepted goal is one projection per current player and read-only product queries.

### Persist only projected attributes

This would remove repeated growth projection, but Search and Squad would still need role scoring in filter and sort paths or another lazy score layer. Rejected because complete role scores are required to remove domain calculation from every product read.

### Persist potential data for every retained snapshot

This would make deletion promotion a marker-only operation, like eager Club DNA. It adds storage and ingest or migration work for snapshots that no supported product read can select. Rejected because the product has no historical potential analytics and the developer approved current-snapshot-only ownership.

### Store a catalog-ordered score vector on each player

A JSON vector would reduce row count, but it would couple stored indexes to catalog order and weaken existing indexed Search/Squad filtering and sorting. Rejected because the normalized table already supplies stable role identity and query support.

### Add a separate projected-attribute table

A one-to-one table could model derived ownership explicitly. It would add a second identity, join, cascade, and lifecycle for a single player-owned map. Rejected in favor of nullable fields on the existing snapshot/player row.

### Rebuild at first read after a model change

This would preserve lazy stale-row replacement and make query success depend on hidden write work. Rejected because model changes need explicit, atomic upgrade ownership before product reads.

## Consequences

### Positive

- Each relevant current player is projected once per materialization or supported boost reconciliation.
- Every potential role score for that player comes from the same projected map.
- Player Profile, Planner, Search, and Squad become read-only consumers of one persisted model.
- Search and Squad lose cache misses, page/full-cohort materializers, write-oriented cohort-completeness checks, and stale-row replacement. One shared read-only snapshot assertion fails before scalar, `EXISTS`, or `LEFT JOIN` potential queries can mask corrupt state.
- Snapshot selection, deletion promotion, and boosts have explicit atomic derived-state ownership.
- Planner commands cannot commit team, string, or assignment mutations and then report a potential-state invariant error while building their depth response.
- Historical snapshots carry no unsupported potential data.
- Model changes cannot silently mix stale and current rows through lazy replacement.

### Negative

- Load Data performs one complete projection and a full catalog of potential-role writes for every player when the inserted snapshot becomes current.
- Database upgrade performs the same synchronous work for every save's existing current snapshot.
- Deletion of a current snapshot can take longer because promotion materializes the retained winner before commit.
- The current snapshot stores a projected JSON map plus roughly one additional role row for every existing current-role row.
- The migration framework gains a transaction hook for Rust-owned backfill rather than remaining SQL-only.
- Potential reads add one snapshot-wide invariant query when they need potential data. Depth-returning Planner mutations add the same query before mutation when a current snapshot exists. This is deliberate: missing or corrupt current derived data fails before it can become null, exclusion, changed order, a read-time repair, or a committed mutation followed by an error response.

### Follow-up

- Prove v34 upgrade and rollback with multiple saves, current and retained snapshots, stale sparse rows, nullable scores, and exact model versions.
- Prove ingest winner/loser, demotion clearing, deletion promotion, final deletion, and save-switch no-write behavior.
- Prove CA and mentality boost reconciliation replaces projected JSON and every potential role atomically.
- Convert each current `project_attributes` production caller to exact-version persisted reads guarded by the shared read-only assertion, then delete lazy materialization, cohort-completeness, invalidation, and stale-replacement paths.
- Preflight every Planner team, string, and assignment mutation that returns depth before Planner setup or its first mutation transaction/write. Preserve pure request-validation order and no-snapshot behavior, and keep optimizer preflight before its assignment transaction.
- Prove corrupt current potential state blocks at least one confirmed destructive team/string mutation and one assignment mutation, returns the invariant error, leaves teams, strings, assignments, projected fields, and potential-role rows unchanged, and performs no derived write.
- Prove deleted and wrong-version role rows plus missing and wrong-version projected maps return invariant errors under write-denying triggers and leave all derived and product-owned data unchanged.
- Record representative upgrade and Load Data duration when a suitable large user database is available. Do not claim a speed improvement without measurements.

## Related work

- Completed feature: [Ingest-Time Potential Scoring](../features/completed/ingest-potential-scores.md)
- Supersedes: [ADR-0019 — Lazy persistent potential role-score cache](./0019-lazy-potential-role-score-cache.md)
- Eager analogue: [ADR-0024 — Eager persisted Club DNA scores](./0024-eager-persisted-club-dna-scores.md)
- Existing projection contract: [Potential Role Scores](../features/completed/potential-role-scores.md)
- Existing snapshot lifecycle: [Snapshot History and Management](../features/completed/snapshot-history.md)
