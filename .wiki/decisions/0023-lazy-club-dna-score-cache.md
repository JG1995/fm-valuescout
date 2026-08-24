# 0023 — Lazy persistent Club DNA score cache

## Status

Accepted

Implementation status: The v31/v32 cache and exact-count planning foundation is implemented through `8250dbe9aac7853ed90ba674f83a67da870a8ecb`. Indexed Search/Squad integration remains planned from that clean HEAD in the active [Club DNA](../features/active/club-dna.md) ledger.

## Context

Club DNA averages a user-selected set of visible, goalkeeper, hidden, and personality player attributes into one nullable 0–100 score. Search must display, filter, and sort that score across the active current snapshot. Squad must display and sort it across the exact managed-club cohort. Both interfaces must continue to return bounded pages.

The initial plan derived the score directly in SQLite from player JSON. A discarded complete-catalog 2,000-player experiment measured 20 Search filter samples of `[1875,1860,1862,1865,1902,1913,1881,1878,1865,1858,1863,1868,1877,1895,1889,1880,1900,1859,1896,1855]` ms. Its nearest-rank p95 was 1902 ms. The 20 Search sort samples were `[914,918,920,917,923,920,923,922,923,922,920,913,904,912,902,908,907,911,916,922]` ms. Their nearest-rank p95 was 923 ms. Both breached the required `<500 ms` 2,000-player threshold. No representative roughly 180,000-player run was attempted, and the failed code was discarded.

A discarded exact-count/indexed implementation then measured all required shapes. At 2,000 players, Search filter measured 22.367 ms cold and 3.575 ms warm p95, Search sort measured 25.969/5.841 ms, and Squad sort measured 25.543/7.280 ms. At 184,000 players, the same shapes measured 2496.594/373.344 ms, 2644.854/509.068 ms, and 2904.600/856.517 ms. Cold measurements describe delayed first use and do not count toward warm acceptance. Exact-count completeness removed warm cache scans; the remaining representative cost is sorting and joining large exact queries.

The score is derived and disposable. Current ingest can retain numeric player attribute values outside Football Manager's integer 1–20 score domain, so the scorer and cache schema must enforce the Club DNA domain instead of trusting stored JSON. Definitions can change, the formula can change, snapshots can be deleted, and supported player boosts can update one current player after ingest. The cache must not become authoritative player data or add mandatory Load Data work.

## Decision

Migration v32 adds a positive `definition_version` to `club_dna_definitions` without modifying committed migration v31. It also adds a nullable `club_dna_scores` cache with identity `(snapshot_id, uid, definition_version, score_model_version)` and `CHECK (score IS NULL OR score BETWEEN 0 AND 100)`. Each cache row belongs to the matching player/snapshot through a cascading foreign key. An index supports lookup, filtering, and ordering by snapshot, definition version, score-model version, and score. Fresh and v31-upgrade migration tests characterize null, 0, and 100 as valid cache values and reject values outside 0–100.

Rust reads and validates the save-owned definition once for each materialization request. A pure Rust scorer loads the selected values from `attributes_json`, `hidden_attributes_json`, and `personality_json`. Every selected value must be an integer from 1 through 20. A missing key, explicit null, non-integer, 0, 21, or any other out-of-domain value makes the whole score null. For a valid complete player, the scorer scales each value by 5, averages all selected values equally, and rounds once. Tests keep 1 and 20 valid and prove that 0 and 21 produce null across each applicable JSON source, including visible and goalkeeper values in `attributes_json`. The materializer stores a row even when the result is null.

Materialization is lazy and scoped to the operation:

- A display-only Search or Squad request materializes only the requested page UIDs.
- A Search filter or sort materializes the complete active current-snapshot Search cohort before SQLite counts, filters, or orders it.
- A Squad sort materializes the exact configured managed-club cohort before SQLite orders it.

The materializer validates the request, snapshot, and definition before it opens a write transaction. The existing synchronous Search or Squad command retains its current `Db` mutex guard through materialization and the dependent query, so another database command cannot interleave an incomplete global cohort. The materializer loads and scores players in bounded batches and persists each batch in its own short transaction. No SQLite write transaction spans another batch's calculation, and no mutex guard crosses an async wait. Partial derived rows after an error are safe and resumable, but a global filter or sort does not run until its required cohort is complete. The unique identity and idempotent upsert keep retries correct without another feature-wide lock.

A missing definition performs no cache work and produces null values. Invalid requests follow the existing safe error path before materialization. Ingest and current-snapshot promotion do not precompute Club DNA.

Definition edits increment `definition_version` and delete all Club DNA cache rows owned by that save in the same transaction. Definition removal deletes those cache rows and the definition atomically. Re-creation is a new definition and can start a new version lineage because removal leaves no old rows. Formula changes increment the Rust-owned `score_model_version`; stale-model rows behave as misses. Snapshot or player deletion cascades rows. Every successful supported player boost deletes that snapshot player's Club DNA rows in the same reconciliation transaction.

## Alternatives considered

### Eager ingest or promotion computation

Calculate Club DNA for every player during Load Data or current-snapshot promotion. This would make later reads warm, but it would add mandatory work for a feature the user may not define or use. Definition edits would still require a full-save recalculation. Rejected because lazy requested-scope work preserves ingest and promotion behavior.

### Request-local calculation without persistence

Load the required players into Rust, calculate the cohort for one request, then discard it. This avoids a migration and persisted invalidation rules. It repeats the same full-cohort work after every filter, sort, page revisit, app restart, or Query cache eviction. It also makes first-use cost the cost of every use. Rejected because the measured direct path already breaches the interaction budget and the derived result has clear version and invalidation owners.

### Migration v33 directional, null-ordering, and player-join indexes

Add query-specific indexes for score direction, null ordering, and player joins to reduce the remaining large exact-query sort/join cost. The measured v32 index plus exact-count completeness already keeps every representative warm shape within the accepted one-second contract. The developer rejected v33 because it would add persistent storage, write amplification, query-specific variants, another migration, broader EXPLAIN obligations, and maintenance complexity without a required product gain.

## Consequences

### Positive

- Warm Search and Squad queries use indexed nullable scores instead of repeated JSON aggregation.
- The v32/count-fast-path design meets one shared representative warm contract without v33 query-specific indexes.
- Display remains page-lazy while global filter and sort remain correct over complete cohorts.
- Ingest and promotion do not pay for an unused definition or metric.
- Definition and formula versions make stale rows unambiguous misses.
- Null rows prevent repeated work for players with missing, null, non-integer, or out-of-domain selected attributes.
- The scorer and schema check keep every cached non-null score inside 0–100 even when ingest retains broader numeric JSON values.
- Rust and SQLite retain computation, persistence, global ordering, filtering, and pagination ownership.

### Negative

- Migration v32 adds another potentially large derived table and index.
- Representative warm Search filter, Search sort, and Squad sort operations may approach one second at 184,000 players.
- First use of a new definition, score-model version, page, or full cohort has a visible cold materialization cost.
- Cold global work holds the established command-level `Db` mutex, so other database commands wait even though each SQLite write transaction is bounded.
- Definition mutation and supported player boosts gain explicit cache invalidation responsibilities.
- The synchronous mutex boundary favors complete-cohort correctness over concurrent database responsiveness during cold first use.

### Invalidation owners

- `club_dna::service` owns atomic save-wide invalidation for definition edit and removal.
- `player::service::reconcile_verified_boost` owns same-transaction invalidation for the successfully reconciled snapshot player.
- SQLite foreign keys own snapshot/player deletion cleanup.
- The Rust Club DNA scorer owns `score_model_version`; formula changes increment it.
- Ingest and promotion own no Club DNA invalidation or prefill because new snapshot identities start cold.

### Thresholds and follow-up

- Complete-catalog 2,000-player validation clears the cache and records cold first-use duration separately for each materially distinct Search filter, Search sort, and Squad sort. After prefill, each shape runs 3 warm-ups and 20 warm measured queries. Nearest-rank p95 is sorted sample index 18 and must be `<500 ms`.
- A representative 184,000-player validation records cold first use separately. After prefill, each Search filter, Search sort, and Squad sort shape runs 3 warm-ups and 20 warm measured queries. Nearest-rank p95 must be `<=1,000 ms` for each shape.
- An unavailable representative environment stops publication until the developer explicitly accepts that validation gap. A measured warm breach requires replanning and cannot be accepted as a gap. Do not add v33 indexes after a breach without a new developer decision.
- The existing Search or Squad query-loading state remains visible during cold materialization, and no partial global result appears. The measured representative cold range of 2496.594–2904.600 ms is delayed first-use evidence and remains separate from warm measurements. If measured cold behavior makes the app unusable, choose a progress and cancellation design in a new decision instead of weakening complete-cohort correctness.

## Related work

- Feature plan: [Club DNA](../features/active/club-dna.md)
- Prior cache decision: [ADR-0019](./0019-lazy-potential-role-score-cache.md)
- Completed definition commits: `ddd4961e6d90ca24faa435955c6ae7eb5a716f0b`, `d2682ee5c50cb99cd0b7f9facf5fd4f9060d5001`
- Supersedes: the active Club DNA plan's direct read-time SQL derivation decision; no implemented current-state architecture is superseded
