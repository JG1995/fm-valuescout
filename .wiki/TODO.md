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
| 2a | Bridge scan performance | [Completed](./features/completed/bridge-scan-performance.md) | medium-high | Block heap scanning, streaming dump, optimized ingest, unlimited default, UI cap controls — **done** |
| 3 | Role scoring engine | [Completed](./features/completed/role-scoring-engine.md) | medium | One scoring model on ingest; primary/secondary bands; combined IP/OOP helper — **done** |
| 4 | Player search | [Completed](./features/completed/player-search.md) | high | First full UI value path after Load Data; validates DB and scores — **done** |
| 5 | Player profiles | [Completed](./features/completed/player-profiles.md) | high | Detail view from search; traditional scouting path — **done** |
| 6 | Squad planner | [Active](./features/active/squad-planner.md) | medium | Tactic + three-team squad strings; same scores as search — **in progress** |
| 7 | Squad optimizer | CONCEPT | medium | Maximize combined team score; closes success vignette |

**Dependency graph:**

```text
[FM26 memory read]
        │
        ▼
[Snapshot ingest + SQLite]
        │
        ▼
[Bridge scan performance]
        │
        ├──────────────────────► [Role scoring on ingest]
        │                                │
        └────────────────┬───────────────┘
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

[Squad planner](./features/active/squad-planner.md) — shared dual-phase tactic; Senior, Reserves, and Youth depth strings; explicit club-family sources for separate B/youth clubs.

## Plan next

Squad planner PR 2, commit 1 — persist squad depth assignments.

## Next

Squad optimizer (order 7) — maximize combined team score; closes success vignette.

## Completed

- [Codex workflow migration](./features/completed/codex-workflow-migration.md) — Codex-only repository guidance, specialist agents, and product-focused validation
- [Player profiles](./features/completed/player-profiles.md) — `/players/$uid` with Overview / Attributes / Roles tabs; entry from Search row and Ctrl+K; ScoreBadge; position-family role grouping
- [Player search](./features/completed/player-search.md) — `/search` route, virtualized table, operator filters (scalars through role scores), URL-persisted filter/sort state, global Ctrl+K name suggest
- [Role scoring engine](./features/completed/role-scoring-engine.md) — FM26 IP/OOP catalog (68 roles), scores on ingest into `player_role_scores`, DLP IP sanity proof, `combine_role_scores` helper
- [FM26 memory read](./features/completed/fm26-memory-read.md) — BepInEx bridge, file protocol, dump schema v5, Rust validation, bridge status + Load Data UI
- [In-app bridge plugin install](./features/completed/bridge-plugin-install.md) — install / update / remove `FmDataBridge.dll` for default Steam FM26 from the app
- [Snapshot ingest + Load Data](./features/completed/snapshot-ingest.md) — app save slots, current snapshot ingest, Load Data scan+ingest, sanity list
- [Bridge scan performance](./features/completed/bridge-scan-performance.md) — block heap scanning, streaming dump, optimized ingest, unlimited Load Data default, UI cap controls and timings
