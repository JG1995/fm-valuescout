# Role scoring engine

## Intent

Compute a 0–100 role-fit score for every player for every FM26 In Possession (IP) and Out of Possession (OOP) role, using FM-designated primary and secondary attributes. Persist scores on Load Data ingest so search, profiles, planner, and optimizer share one scoring model. Expose a pure combined IP+OOP score with caller-supplied weights (persistence and UI deferred to squad planner).

## Delivered behavior

- After a successful **Load Data**, each ingested player has per-role IP and OOP scores in SQLite (`player_role_scores`). A score is `null` only when a required attribute is missing or JSON-null in the dump.
- The sanity list shows a **DLP IP** column (`proofRoleScore`) — the deep-lying playmaker in-possession score for each listed player. This is a thin proof, not a role browser.
- `combine_role_scores` blends IP and OOP scores with caller-supplied weights (default 50/50). No weight settings UI or IPC in this feature.
- Position suitability (`positions`) does not enter role scores. The catalog is static and offline (checked into the repo).

## Final architecture

```text
Rust features/scoring
  → catalog.rs — static FM26 catalog (68 IP/OOP roles; SortItOutSI Key/Preferred; dump PascalCase keys)
  → score.rs — score_role: band means → 75/25 blend (or primary-only) → /20×100 → rounded integer 0–100
  → combine.rs — combine_role_scores(ip, oop, ip_weight); DEFAULT_IP_WEIGHT 0.5

Rust features/snapshot (extend)
  → ingest: after insert_players, insert_role_scores in the same transaction
  → query: list_sanity_players LEFT JOINs proof role deep_lying_playmaker_ip (PROOF_ROLE_ID)

React features/snapshot (extend)
  → sanity table column DLP IP from proofRoleScore

SQLite migration v3
  → player_role_scores (snapshot_id, uid, role_id, phase, score)
  → PRIMARY KEY (snapshot_id, uid, role_id); FK to players ON DELETE CASCADE
  → index on (snapshot_id, role_id)
```

`scoring` owns formula and catalog. `snapshot` owns when scores are written and the sanity-list proof query. The WebView does not reimplement the formula.

## Important decisions

- **Primary/secondary bands:** SortItOutSI Key = primary, Preferred = secondary; 75% / 25% after within-band equal means. Roles with no secondary list use primary mean alone.
- **Null attributes:** Any null in the used primary or secondary set → `null` score (signals incomplete attribute data).
- **Score on ingest:** All catalog roles × all players computed synchronously during ingest. Ponytail in `ingest.rs`: upgrade to lazy/on-demand or batched scoring if ingest scoring time dominates Load Data.
- **Catalog reconciliation:** Sidekick generic OOP hubs map to SortItOutSI named variants; disjoint primary/secondary bands when a guide listed the same attribute in both.
- **Combined weights:** Function parameters only; planner persistence deferred.
- **Scale testing:** Full-matrix 184k-player ingest test is `#[ignore]`; gate keeps a 2k scored ingest timing check.

## Migration and operational implications

- Migration v3 adds `player_role_scores`. Rows cascade-delete with their player row when a snapshot is replaced.
- Load Data ingest time includes role-score computation (~68 roles per player). Monitor on large saves; ponytail trigger is unacceptable Load Data latency.
- Scores are not recomputed for an existing snapshot without a new Load Data.

## Validation

- `./scripts/dev test`, `./scripts/dev check`, Playwright smoke (sanity list with `proofRoleScore` / DLP IP column).
- `cargo test`: catalog invariants, `score_role` and `combine_role_scores` fixtures, ingest golden scores, migration v3 schema, 2k scored ingest timing; 184k scale test `#[ignore]`.
- Manual Windows: Load Data with FM running; sanity list shows DLP IP values.

**Delivery commits (final hashes):** `0b08dd1`, `f504dcf`, `d81fea0`, `00d1e80`, `56e94bf` (feature branch `feat/scoring`, merge base `f3baf5f`).

## Follow-up

- **Next feature:** [Player profiles](../../TODO.md) (order 5) — detail view from search; row activation from Search.
- **Deferred:** profile role grid, weight UI, full role browser, lazy scoring (ponytail if ingest bottleneck). Search filters and sort by role scores shipped in [player-search](./player-search.md).
