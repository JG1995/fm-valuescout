# Planned Work

> **Authority:** This document owns work specifically planned for the near future — features and initiatives with committed or imminent delivery intent. It does not duplicate PR/commit details in active ledgers.

Items that are not actively planned but worth remembering belong in [BACKLOG.md](./BACKLOG.md).

## Development sequence (approved proposal)

> **Lifecycle:** Approved 2026-07-28 from [CONCEPT.md](./CONCEPT.md) MVP scope. Ordering is **provisional — revisit after speccing** individual features.
>
> **Gating context:** FM26 memory read is complete — C# BepInEx IL2CPP bridge + Rust file protocol ([completed record](./features/completed/fm26-memory-read.md), dump schema v5 in `bridge/DUMP_SCHEMA.md`). Role scores use FM-designated important attributes per role with a simple custom weighting algorithm. Multi-save (thin) shipped with snapshot ingest; snapshot **history** is backlog.

| Order | Feature | Spec | Confidence | Why this position |
| --- | --- | --- | --- | --- |
| 1 | FM26 memory read | [Completed](./features/completed/fm26-memory-read.md) | medium | Foundation — live dump from FM; **done** |
| 2 | Snapshot ingest + Load Data | [Completed](./features/completed/snapshot-ingest.md) | medium-high | Persist dumps to SQLite; multi-save; Load Data = scan+ingest — **done** |
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

_None — run `/plan-feature` on the item in **Plan next** when ready to start._

## Plan next

`/plan-feature` on **Role scoring engine** (order 3).

## Next

Role scoring engine (order 3) — scores computed on ingest from FM role-relevant attributes.

## Completed

- [FM26 memory read](./features/completed/fm26-memory-read.md) — BepInEx bridge, file protocol, dump schema v5, Rust validation, bridge status + Load Data UI
- [In-app bridge plugin install](./features/completed/bridge-plugin-install.md) — install / update / remove `FmDataBridge.dll` for default Steam FM26 from the app
- [Snapshot ingest + Load Data](./features/completed/snapshot-ingest.md) — app save slots, current snapshot ingest, Load Data scan+ingest, sanity list
