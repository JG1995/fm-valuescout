# Planned Work

> **Authority:** This document owns work specifically planned for the near future — features and initiatives with committed or imminent delivery intent. It does not duplicate PR/commit details in active ledgers.

Items that are not actively planned but worth remembering belong in [BACKLOG.md](./BACKLOG.md).

## Development sequence (approved proposal)

> **Lifecycle:** Approved 2026-07-28 from [CONCEPT.md](./CONCEPT.md) MVP scope. Ordering is **provisional — revisit after speccing** individual features. No planned specs yet; confidence is mostly inferred from CONCEPT bullets.
>
> **Gating context:** FM26 memory read is unproven in this repo (notes from a working third-party plugin; implementation approach TBD). Role scores use FM-designated important attributes per role with a simple custom weighting algorithm.

| Order | Feature | Spec | Confidence | Why this position |
| --- | --- | --- | --- | --- |
| 1 | FM26 memory read | CONCEPT | medium | Foundation — no other MVP feature works without live game data; highest technical risk |
| 2 | Snapshot ingest + Load Data | CONCEPT | high | Persist memory reads to SQLite; explicit refresh workflow |
| 3 | Role scoring engine | CONCEPT | medium | One scoring model on ingest; FM role-relevant attributes + custom algorithm |
| 4 | Player search | CONCEPT | high | First full UI value path after Load Data; validates DB and scores |
| 5 | Player profiles | CONCEPT | high | Detail view from search; traditional scouting path |
| 6 | Squad planner | CONCEPT | medium | Tactic + squad slots; same scores as search |
| 7 | Squad optimizer | CONCEPT | medium | Maximize combined team score; closes success vignette |

**Dependency graph:**

```text
[FM26 memory read]
        │
        ▼
[Snapshot ingest + SQLite] ──► [Role scoring on ingest]
        │                              │
        └──────────────┬───────────────┘
                       ▼
              [Player search]
                       │
              ┌────────┴────────┐
              ▼                 ▼
      [Player profiles]   [Squad planner + tactic]
                                  │
                                  ▼
                          [Squad optimizer]
```

**MVP spine:** Load Data (FM running) → searchable players with role scores → profile → planner gap → optimize XI.

**Parallel tracks:** After player search (order 4), profiles (5) and squad planner shell (6, without optimizer) can overlap once tactic modeling is specced.

## Active

_(none)_

## Plan next

Run `/plan-feature` on **FM26 memory read** (order 1). Start from CONCEPT bullet; consider optional `/spike` first to prove attach + minimal player read on current FM26 build before multi-commit feature work.

## Next

_(filled by Plan next above — first feature after roadmap approval)_

## Completed

_(none — completed features move here with a link to [features/completed/](features/completed/README.md))_
