# 0028 — Compact current-snapshot metrics

## Status

Accepted

Implementation status: Planned in [Compact Snapshot Metrics and Load Progress](../features/active/compact-snapshot-metrics.md).

## Context

The current database stores one row per player and role for current scores and another row per current player and role for potential scores. Staff scores use the same normalized shape. A representative 250,000-player load therefore writes about 34 million player score rows. A later load can delete about 17 million current-only potential rows. A live database inventory contained about 134.8 million current player-role rows and 55.7 million potential rows in an approximately 31 GiB `app.db`.

The scoring and projection benchmark took about 6.2 seconds for 250,000 players across 68 current and 68 potential metrics, which indicates that persistence dominates this workload. A local directional prototype stored 250,000 wide player rows with 136 nullable scores in about 79 MiB, inserted them in about 2.5 seconds, and sorted an unindexed role column in about 56–66 ms. These figures guide the design but are not production promises.

Product reads need arbitrary SQLite sorting and filtering for all 68 current and 68 potential player role metrics and all 21 current staff role metrics. Historical product views do not need stored projected attributes or historical role-score matrices. The application must keep one SQLite writer, one atomic snapshot publication, and the Rust-owned `rusqlite` boundary from [ADR-0015](./0015-sqlite-rust-owned.md).

## Decision

Create a fresh database generation under the application database filename `app-v2.db`. The new build must not open, migrate, import, vacuum, delete, or provide a compatibility read path for the existing `app.db`. Reinstalling the application might preserve application data, so the implementation commit that makes the filename user-visible must document both files and require verification of `app-v2.db` before manual deletion of `app.db`.

Store current player metrics in one `player_role_metrics` row per player in each effective current snapshot. The row has nullable named current and potential columns for the closed 68-role catalog, plus explicit score-model and projection-model versions. Retain the current-only projected-attribute JSON and projection-model version on `players`. Store current staff metrics in one `staff_role_metrics` row per staff member in each effective current snapshot, with nullable named columns for the closed 21-role catalog and an explicit score-model version.

The first compact migration owns one immutable checked-in column inventory: exactly 68 player current columns, the matching 68 player potential columns, and 21 staff columns. Prefer static checked-in migration SQL because it preserves historical DDL. Never derive that migration's DDL from mutable runtime catalogs. Runtime mapping derives SQL identifiers only from role IDs that pass closed-catalog lookup and safe snake_case validation, and contract tests compare the complete mapping with that exact migration schema and model version. Do not accept an identifier from WebView input. A player role uses distinct named current and potential columns. A later catalog, formula, projection, or score-model change requires a new migration and model version; it never edits the first compact migration or its DDL. Preserve null as the exact result when required source attributes are unavailable.

Keep raw player and staff facts for every retained snapshot. Keep no projected attributes or player or staff role metrics on historical snapshots. A future individual Player Profile timeline can read one player across snapshots and recalculate projection and role metrics with the then-current model. This decision does not add that UI.

Prepare and validate source data and calculate compact metrics outside the database mutex where practical. Use one `Db(Mutex<Connection>)`, one `rusqlite` connection, one SQLite writer, and one final transaction. Keep the prior snapshot visible until commit. When a different snapshot becomes current, clear the former current derived rows in the same transaction. Current-snapshot deletion promotion rebuilds compact metrics from retained raw facts before commit. A supported player boost atomically replaces that current player's raw values, projected attributes, and current and potential compact metric values.

Do not add WAL mode, a connection pool, another driver, Rayon or another dependency, multiple writers, detached jobs, or cancellation. Replan only if representative measurements show unacceptable final-write blocking after compaction.

Use command-scoped best-effort Tauri channel progress for Load Data. The final command result stays authoritative. Report truthful phase-local progress: scan is indeterminate; validation and preparation, scoring, saving, and finalization are determinate only when a real count exists. Do not create a weighted overall percentage. Keep detailed timings for scan, preparation and scoring, database save and finalization, and total.

Retain existing scalar indexes. Add no per-role index unless representative compact-row sorting fails acceptance and the feature is replanned.

## Alternatives considered

### Keep normalized score rows

Normalized rows preserve the current query shape, but they create tens of millions of writes and retained historical current-score rows that no historical product view reads. Rejected because persistence and database growth dominate the measured workload.

### Store an opaque JSON or binary metric blob

A blob would reduce row count but would weaken direct SQLite sorting and filtering and would require catalog-order decoding. Rejected because arbitrary role sorting and filtering is a current product contract.

### Convert the existing database in place

An in-place migration or compatibility path could preserve existing data, but it would operate on an approximately 31 GiB file, require rollback and disk-space policy, and keep legacy complexity. Rejected in favor of a fresh database generation with explicit manual cleanup of the untouched old file.

### Store derived metrics for historical snapshots

This would make promotion cheaper, but it would retain large matrices for a UI that does not exist and would bind historical display to old model output. Rejected because raw historical facts are sufficient for future one-player current-model recomputation.

### Add role indexes, WAL, pooling, parallel scoring, or background jobs

These options can improve specific workloads, but they add storage, concurrency, dependencies, and recovery paths before compaction is measured. Rejected until representative validation proves a remaining problem.

## Consequences

### Positive

- Current player and staff metrics remain directly sortable and filterable in SQLite with one compact row per current entity.
- Historical storage contains raw facts without projected attributes or role-score matrices.
- Snapshot replacement, promotion, and player boost reconciliation retain one atomic publication boundary.
- The old database remains untouched, which removes conversion and rollback risk.
- Load Data can expose truthful phase progress without a global job system.

### Negative

- The schema has 136 nullable player metric columns and 21 nullable staff metric columns whose names are coupled to the closed catalogs.
- The immutable migration inventory and runtime catalogs can drift unless their exact schema/model contract test remains complete.
- Catalog and model changes require a new migration, model version, and rebuild decision; old migration DDL cannot be corrected in place.
- Deletion promotion must recalculate compact metrics from raw retained facts inside its transaction.
- Users must verify the new database and manually remove the old `app.db` if they want to reclaim its disk space. Reinstall alone might not remove it.
- Unindexed sorting must meet representative acceptance; a failure requires replanning rather than an unreviewed index expansion.

### Follow-up

- Measure a fresh representative Windows load of about 250,000 players. Record phase timings, repeated-load behavior, arbitrary current and potential role sort and filter behavior, and the new database size.
- Add the user-facing `app-v2.db` and safe manual-cleanup instruction in the implementation commit that changes the filename. Reconcile `.wiki/ARCHITECTURE.md` and `.wiki/DESIGN.md` only after implementation makes the new architecture and interaction behavior true.
- Keep historical Player Profile timeline work in the backlog until separately planned.

## Related work

- Feature plan: [Compact Snapshot Metrics and Load Progress](../features/active/compact-snapshot-metrics.md)
- Amends: [ADR-0025 — Selective index-driven player table sorts](./0025-selective-index-driven-player-sorts.md)
- Supersedes: [ADR-0026 — Eager current-snapshot potential scoring](./0026-eager-current-potential-scoring.md)
- Supersedes in part: [ADR-0027 — Scoped potential read validation](./0027-scoped-potential-read-validation.md)
- Retains: [ADR-0015 — SQLite with Rust-owned migrations and queries](./0015-sqlite-rust-owned.md)
