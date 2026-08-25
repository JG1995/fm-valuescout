# 0025 — Selective index-driven player table sorts

## Status

Accepted

Implementation status: Implemented and completed in [Player Table Sort Performance](../features/completed/player-table-sort-performance.md).

## Context

Search and Squad expose a broad sortable metric catalog over bounded IPC pages. Common scalar sorts lack suitable indexes, managed-club membership lacks a player index, and persisted score sorts use correlated probes. The application also serializes commands through one `rusqlite` connection, so duplicate or inefficient reads queue.

The product needs truthful retained rows and responsive common sorts on a representative large save. It must preserve authoritative Rust/SQLite ordering, bounded IPC pages, supported active-save/current-snapshot/managed-club behavior, lazy potential-score ownership, and the single-driver database architecture. Universal indexing, a response-generation protocol, or committed performance infrastructure would add disproportionate cost for this solo-maintained project.

## Decision

1. Keep sort replacement data in TanStack Query. Use committed and requested first-page observers, remove duplicate route-loader result ownership, deduplicate page zero, and promote header, rows, total, and page options together. Retain rows only for sort replacement inside the same mounted context.
2. Add stable `searchKeys.playerPages()` and `squadKeys.playerPages()` roots to the existing feature key factories, and make every parameterized `searchKeys.players(...)` and `squadKeys.players(...)` key extend its own root. Put cross-feature query clearing in `src/app/player-result-context.ts`, where the app layer may import both key factories. Its async operation awaits cancellation of both exact roots, then removes both. Keep only the neutral mutation key and any neutral callback type in `src/components/player-table/player-result-context.ts`; shared code must not import features.
3. AppTopBar, Settings, and My Club create and inject the async clearing callback into the feature mutations they compose. Feature hooks and components must not import sibling feature keys or the app coordinator. A supported mutation uses the neutral key, awaits the callback inside `mutationFn`, and only then invokes Tauri. Active-save and current-snapshot deletion use current target flags to select that path. Inactive-save and non-current-snapshot deletion use neither the neutral key nor the callback. Search and My Club use `useIsMutating` with the neutral key plus existing owner-fetch state to block result controllers through mutation and owner refresh. Preserve existing post-success or settled invalidation.
4. Exact clearing removes all Search and Squad player pages while preserving Search suggestions and unrelated Planner queries. Do not broaden removal to `searchKeys.all` or `plannerKeys.all`. The app coordinator owns no rows, generation identity, mutation state, or global result state.
5. Use only existing active-save `{ id, contextToken }`, current snapshot ID/save ID, and managed-club club/status when mounted result discrimination needs them. Do not add response generation metadata, snapshot or managed-club tokens, same-read identity protocols, or new IPC arguments.
6. A canceled Tauri command may finish. Rely on TanStack Query cancellation/removal only after focused coordinator and route tests prove that late fulfillment is ignored. Add explicit request/response generation binding only if a supported app-owned context change can reproduce stale-generation rows after cancellation/removal.
7. Retain the existing Name and CA indexes. Add only directional PA, Age, and Value indexes plus the exact managed-club membership index selected in the feature ledger: seven new player indexes total.
8. Drive current-role, warm potential-role, and Club DNA ordering from their persisted score relations. Preserve absent and nullable rows, both directions, family-specific null behavior, exact cohorts, totals, pages, and UID ties.
9. Keep potential-role scores lazy. Use exact-version cohort completeness for the warm path, retain bounded cold materialization and recheck, and remove only the redundant visible sort-role page pass.
10. Use normal automated correctness gates and simple pass/fail manual product testing on a representative approximately 250,000-player save. Do not put performance timing in `check` and do not commit performance evidence.

## Alternatives considered

### Keep one suspense query or prior-data placeholder

This does not keep one committed observer after replacement failure and can retain data across an identity change. Rejected because sort replacement needs one truthful committed request while context changes must clear rows.

### Bind every result to explicit request and response generations

This can prove stale-response rejection independently of Query cancellation, but it adds response metadata, key fields, IPC coordination, and tests across every owner. Deferred because supported context mutations are app-owned and can clear result queries before mutation. The upgrade trigger is a reproducible stale-generation row after cancellation/removal during one of those supported changes.

### Add immutable snapshot and managed-club response identity

Extra tokens and same-read identity rules protect reused numeric IDs and external mutation patterns outside the supported app contract. Rejected as disproportionate. Existing active-save, snapshot, and managed-club fields plus app-owned clearing are sufficient unless the upgrade trigger fires.

### Add production statement-plan extraction and committed benchmark provenance

Shared executable plans, permanent reports, and exact provenance can support repeatable quantified claims, but they add production structure and long-lived tooling that this project does not need. Rejected as disproportionate. Focused query tests may inspect local SQL or index use when practical, while manual product testing owns performance judgment.

### Replace existing controls or index every sortable metric

This would add storage and ingest cost for untargeted sorts. Rejected in favor of retained Name/CA controls and a narrow seven-index portfolio.

### Add score-order indexes, a connection pool, background work, or eager potential scoring now

Each option changes persistence or execution ownership. Deferred until the direct relation-driven approach or manual product validation shows a concrete need.

### Move global sorting to React or a Web Worker

This would break bounded IPC and duplicate authoritative semantics. Rejected.

## Consequences

### Positive

- Sort-only replacement keeps one truthful committed table without copying Query data.
- Supported context changes clear result ownership before mutation and have a focused late-result proof.
- The design uses existing context data and avoids a speculative cross-process generation protocol.
- Schema growth stays narrow.
- Persisted score relations become the sort source without changing score ownership.
- Manual performance judgment stays proportionate to a solo hobby project.

### Negative

- AppTopBar, Settings, and My Club must keep the injected callback wiring complete. Every supported feature mutation must expose the neutral transition key and await that callback before Tauri; active/current delete must enter the path while inactive/non-current delete must not.
- Frontend tests must prove feature-owned stable prefix extension, app-coordinator cancellation then removal, callback-before-Tauri ordering, conditional deletion, exact all-page removal with Search suggestions and unrelated Planner queries preserved, route blocking through owner refresh, replacement, retry, rapid sort, virtual paging, activation denial, and deferred late-result behavior.
- Seven indexes increase database size and write maintenance.
- Cold potential sorting still occupies the single connection while it materializes missing rows.
- Performance acceptance is developer judgment rather than a quantified historical record.

### Follow-up

- Replan before adding another index or changing concurrency, cache, or protocol ownership.
- Upgrade to explicit request/response generation binding only if a supported app-owned context change can reproduce stale-generation rows after cancellation/removal.
- Reconsider background potential work only under ADR-0019's upgrade trigger.

## Related work

- [ADR-0005 — TanStack Query](./0005-tanstack-query.md)
- [ADR-0015 — SQLite with Rust-owned migrations and queries](./0015-sqlite-rust-owned.md)
- [ADR-0019 — Lazy persistent potential role-score cache](./0019-lazy-potential-role-score-cache.md)
- [ADR-0024 — Eager persisted Club DNA scores](./0024-eager-persisted-club-dna-scores.md)
- [Player Table Sort Performance](../features/completed/player-table-sort-performance.md)
