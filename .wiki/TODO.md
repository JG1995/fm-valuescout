# Planned Work

> **Authority:** This document owns work specifically planned for the near future — features and initiatives with committed or imminent delivery intent. It does not duplicate PR/commit details in active ledgers.

Items that are not actively planned but worth remembering belong in [BACKLOG.md](./BACKLOG.md).

## Development sequence (approved proposal)

> **Lifecycle:** Approved 2026-07-28 from [CONCEPT.md](./CONCEPT.md) MVP scope. Ordering is **provisional — revisit after speccing** individual features.
>
> **Gating context:** FM26 memory read uses a C# BepInEx IL2CPP bridge + Rust file protocol (Windows Steam only); approach locked in [features/active/fm26-memory-read.md](./features/active/fm26-memory-read.md). Role scores use FM-designated important attributes per role with a simple custom weighting algorithm.

| Order | Feature | Spec | Confidence | Why this position |
| --- | --- | --- | --- | --- |
| 1 | FM26 memory read | Active ledger | medium | Foundation — live dump from FM; in progress |
| 2 | Snapshot ingest + Load Data | CONCEPT | high | Persist memory dumps to SQLite; explicit refresh workflow |
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

- [FM26 memory read](./features/active/fm26-memory-read.md) — PR 1 / commit 1 Active: bridge toolchain prerequisites and ignores

## Plan next

After FM26 memory read finishes: `/plan-feature` on **Snapshot ingest + Load Data** (order 2), against the frozen dump schema from memory-read PR 4.

## Next

Snapshot ingest + Load Data (order 2)

## Completed

_(none — completed features move here with a link to [features/completed/](features/completed/README.md))_
