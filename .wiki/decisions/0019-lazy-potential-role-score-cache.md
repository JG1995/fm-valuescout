# 0019 — Lazy persistent potential role-score cache

## Status

Accepted

## Context

Potential attributes and role scores are currently derived at read time from the existing CA-to-PA visible-attribute projection. That is appropriate for one profile or a bounded Planner candidate set, and the resulting IPC response can live in TanStack Query's in-memory cache.

Configurable Search and Squad tables introduce a different access pattern. A potential role can be displayed for many virtual pages and can control a global filter or sort across roughly 183,000 current players. Recalculating the same projection every time the user revisits a table, reopens the app, or selects several potential-role columns would make the table repeatedly pay a minute-scale cold cost. TanStack Query cannot provide correct global filtering or sorting and does not survive an app restart.

The cache must remain derived and disposable. Current role scores are still authoritative ingest-time values, snapshots normally remain immutable, the projection formula can change, and supported player boosts can update the current snapshot after ingest.

## Decision

Add an SQLite `player_potential_role_scores` cache through migration v21. Each row stores `snapshot_id`, `uid`, validated `role_id`, nullable `score`, and `projection_model_version`, with primary key `(snapshot_id, uid, role_id)` and a composite foreign key to `players(snapshot_id, uid)` using `ON DELETE CASCADE`. Add the role/score index required by global table filtering and ordering.

Populate the cache only when a Search filter, selected Search or Squad column, or table sort requires a potential role. Cache only requested roles. When one request needs several missing roles, load and project each affected player once, then score all missing requested roles from that projected attribute map. Store a row even when the score is unknown so missing required attributes are not recalculated on every read. Persist large cohorts in bounded transactions; partial derived rows after an error are safe and resumable, but a global filter or sort does not run until its required cohort is complete.

Use two materialization scopes:

- A potential role used only as a display column materializes the requested virtual page's players.
- A potential role used for filtering or sorting materializes the complete relevant cohort before SQLite counts, filters, or orders it. Search uses the active save's effective current snapshot; Squad uses that snapshot's configured club-family cohort.

Every read requires the current projection-model version. A stale-version row behaves as a cache miss and is replaced lazily. Snapshot or player deletion removes cache rows through the foreign key. Every successful supported player-boost reconciliation deletes all potential cache rows for that snapshot player in the same SQLite transaction, because CA or Determination may have changed and deleting an unaffected player's derived rows is harmless.

The table cache is not used as authoritative player data and does not change profile or Planner calculations in this feature. It is safe to delete and rebuild.

## Alternatives considered

### Keep potential values read-time only

Project every player on every table request and rely on TanStack Query for repeated pages. This preserves the former boundary but repeats work after cache eviction or app restart and cannot avoid the full cold calculation each time a potential field controls global filtering or sorting. Rejected for the new table access pattern.

### Calculate every potential role during Load Data

Persist all 68 potential roles beside current scores for every player. This makes table reads simple, but it increases ingest time and storage for values many users will never inspect. It also turns a table optimization into a mandatory Load Data cost. Rejected in favor of lazy requested-role population.

### Calculate all 68 roles on the first potential request

Project each player once and fill the complete role catalog immediately. This makes every later potential role warm, but one displayed role would create about 12 million score rows for the inspected current snapshot. Rejected in favor of sparse role rows while still sharing the projection when several requested roles arrive together.

### Cache only projected attributes

Persist a projected attribute JSON object per player and calculate role scores from it on each query. This reduces repeated projection but leaves global sort/filter score calculation in the hot path, duplicates another large derived representation, and complicates versioning two cache layers. Rejected until measurements show projection reuse across newly added roles is the dominant remaining cost.

### Store one score vector per player

Persist a catalog-ordered JSON array containing all role scores. This reduces row count but couples stored array indexes to catalog order, calculates unused roles, and makes partial requested-role population and independent invalidation less clear. Rejected in favor of normalized rows keyed by stable role ID.

### Use an in-memory Rust cache

Keep calculated scores only for the current process. This avoids a migration but loses work on restart, duplicates SQLite ownership, and still needs lifecycle rules for snapshots and boosts. Rejected.

## Consequences

### Positive

- Returning to the same table, reopening the app, or reusing a potential role does not repeat unchanged projection work.
- Several selected potential-role columns share one projection pass per affected player.
- Display-only columns remain page-lazy, while global filters and sorts remain correct before the first page is returned.
- Current and potential values keep distinct sources and cannot overwrite one another.
- Cache invalidation is explicit for snapshot deletion, model changes, and supported player boosts.
- Rust and SQLite retain ownership of high-volume computation and global query semantics.

### Negative

- Migration v21 adds another potentially large table and index to an already large application database.
- Selecting a new potential role later can require another projection pass for the relevant cohort because the cache is sparse by role.
- A cold global potential filter or sort can hold the shared database connection while materialization runs.
- Projection-model changes leave stale rows until those role/player combinations are requested again or their snapshot is deleted.
- Boost reconciliation gains one more derived-state invalidation responsibility.

### Follow-up

- Measure cold and warm Search and Squad behavior on a representative Windows snapshot before publication.
- If cold materialization makes the WebView unresponsive or exceeds the accepted minute-scale first-use delay, replace the synchronous materializer with a cancellable background job and explicit progress rather than weakening global query correctness.
- If repeated addition of new roles dominates measured use, reconsider a versioned projected-attribute cache or complete per-player score vector with a new ADR.
- Reconcile the implemented cache and invalidation contract into `.wiki/ARCHITECTURE.md` when the feature completes.

## Related work

- Active feature plan: [Configurable Player Tables](../features/active/configurable-player-tables.md)
- Existing projection contract: [Potential role scores](../features/completed/potential-role-scores.md)
- Existing current-score contract: [Role scoring engine](../features/completed/role-scoring-engine.md)
- Commits: `b258df8`
- Supersedes: the blanket non-persistence decision for potential values only for this disposable table cache; potential values remain derived and non-authoritative
