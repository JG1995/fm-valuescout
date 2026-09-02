# Planned Work

> **Authority:** This document owns work specifically planned for the near future — features and initiatives with committed or imminent delivery intent. It does not duplicate PR/commit details in active ledgers.

Items that are not actively planned but worth remembering belong in [BACKLOG.md](./BACKLOG.md).

## Development sequence (approved proposal)

> **Lifecycle:** Approved 2026-07-28 from [CONCEPT.md](./CONCEPT.md) MVP scope. Ordering is **provisional — revisit after speccing** individual features.
>
> **Gating context:** FM26 memory read is complete — C# BepInEx IL2CPP bridge + Rust file protocol ([completed record](./features/completed/fm26-memory-read.md), current dump schema v7 in `bridge/DUMP_SCHEMA.md`; v6 snapshots remain readable as legacy data). Role scores use FM-designated important attributes per role with a simple custom weighting algorithm. Multi-save (thin) shipped with snapshot ingest; [snapshot history and management](./features/completed/snapshot-history.md) is complete.

| Order | Feature | Spec | Confidence | Why this position |
| --- | --- | --- | --- | --- |
| 1 | FM26 memory read | [Completed](./features/completed/fm26-memory-read.md) | medium | Foundation — live dump from FM; **done** |
| 2 | Snapshot ingest + Load Data | [Completed](./features/completed/snapshot-ingest.md) | medium-high | Persist dumps to SQLite; multi-save; Load Data = scan+ingest — **done** |
| 2a | Bridge scan performance | [Completed](./features/completed/bridge-scan-performance.md) | medium-high | Block heap scanning, streaming dump, optimized ingest, unlimited default, UI cap controls — **done** |
| 3 | Role scoring engine | [Completed](./features/completed/role-scoring-engine.md) | medium | One scoring model on ingest; primary/secondary bands; combined IP/OOP helper — **done** |
| 4 | Player search | [Completed](./features/completed/player-search.md) | high | First full UI value path after Load Data; validates DB and scores — **done** |
| 5 | Player profiles | [Completed](./features/completed/player-profiles.md) | high | Detail view from search; traditional scouting path — **done** |
| 6 | Squad planner | [Completed](./features/completed/squad-planner.md) | medium | Save-scoped club family, dual-phase tactic, and three-team depth chart — **done** |
| 7 | Squad optimizer | [Completed](./features/completed/squad-optimizer.md) | medium | Exact, tactic-aware Planner allocation with preserved manual assignments — **done** |

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

- [Player Shortlist](./features/active/player-shortlist.md) — third Search tab filtered to the current Moneyball cohort with General metrics and independent persisted layout (Linear JAY-52).

## Next

- **Player gender data integrity and filtering** — investigate the memory-reader request and the bridge, dump, and ingest path to determine why representative stored rows contain only `gender = 'unknown'`. Plan a permanent Men / Women / Both Search filter only after representative snapshots contain trustworthy values.

## Completed

- [Staff Assignment FM26 Layout](./features/completed/staff-assignment-fm26-layout.md) — redesign Configure slots around FM26 staff limits, exact coach composition and lead pools, Recruitment Analyst support, and collapsible assignment results (Linear JAY-44).
- [Cumulative Moneyball Imports](./features/completed/cumulative-moneyball-imports.md) — cumulative current-snapshot Moneyball upserts with atomic full-cohort percentiles and shared Search/My Club Squad upload entry points (Linear JAY-33).
- [Ingest-Time Potential Scoring](./features/completed/ingest-potential-scores.md) — current-only eager projected attributes and complete potential role scores with atomic lifecycle and read-only consumers.
- [Player Table Sort Performance](./features/completed/player-table-sort-performance.md) — truthful sort replacement, context-bound result clearing, targeted player indexes, and relation-driven Search and Squad sorts.
- [Club DNA](./features/completed/club-dna.md) — save-owned full-catalog attribute definitions with eager nullable 0–100 scores, Search and Squad table integration, and a context-safe My Club definition Modal (Linear JAY-32).
- [Todo UX Quality Pass](./features/completed/todo-ux-quality-pass.md) — My Club containment, app history controls, shared player-table refinements, and Moneyball profile comparison improvements.
- [Planner Best-Role Reference](./features/completed/planner-best-role-reference.md) — read-only phase-specific best-lane reference for every current managed-club player, with Current/Potential assignment, adjusted scores, and a sortable Planner Modal; final PR not published.
- [Moneyball Role Scores](./features/completed/moneyball-role-scores.md) — versioned, position-family-specific performance role definitions with full-import Player Profile scores, filtered-cohort Search columns and filters, and explicit unavailable attribute placeholders (Linear JAY-20; final PR unpublished)
- [Optional Moneyball Analysis Views](./features/completed/moneyball-views.md) — current-snapshot Moneyball percentile cohorts, optional Search and Player Profile views, virtualized raw-metric filtering, and one shared default-view preference (Linear JAY-19)
- [My Club Workspace](./features/completed/my-club-workspace.md) — `/my-club` unifies Squad, Planner, Tactic, managed-club Staff, and Staff Shortlist; it owns the managed-club selector while Player Search and Staff Search remain separate (Linear JAY-25, PR [#69](https://github.com/JG1995/fm-valuescout/pull/69))
- [Settings and Managed Club](./features/completed/settings-managed-club.md) — Settings owns save, snapshot, managed-club, and bridge management; Dashboard is a placeholder; one save-scoped managed club supplies exact current-snapshot cohorts to Squad, Planner, Academy, Staff, and club-wide boosts (Linear JAY-26, JAY-27)
- [Optional Planner Teams](./features/completed/optional-planner-teams.md) — user-managed one-to-three Senior, Reserves, and Youth categories with per-save display names, safe assignment cleanup, dynamic Planner presentation, and available-team-only optimization (Linear JAY-28)
- [Staff Shortlist CSV Enrichment](./features/completed/staff-shortlist.md) — save-owned replacement imports joined to current staff by UID, with exact Preferred Job filtering, adaptive job-score columns, and unemployment filtering in a third Staff workspace
- [Staff Workspace](./features/completed/staff-workspace.md) — scored Staff Search, configured club-family overview, staff-specific profiles with shared concealment, and fixed individual and configured-family +10 CA actions
- [Player Profile Information Controls and Layout](./features/completed/player-profile-information-controls.md) — save-scoped profile concealment, FM-style four-tab attributes, separate current/potential IP/OOP header summaries, and profile-shell containment (Linear JAY-5, JAY-8, JAY-9)
- [Complete Player Position Familiarity](./features/completed/player-position-familiarity.md) — schema-v7 complete nullable familiarity maps with explicit recorded, playable, and natural-position rules across Search, profiles, Academy, Planner, and optimizer (Linear JAY-14)
- [Squad Boost Progress and Feedback](./features/completed/squad-boost-progress.md) — determinate per-player squad boost progress in the confirmation Modal and layout-stable final feedback in the Squad overview
- [Early Alpha Release Readiness](./features/completed/early-alpha-release-readiness.md) — source-built unsigned Windows validation, local diagnostics, guarded dependency automation, and exact-SHA prepared release publication; the historical alpha release remains documented in the completed record
- [Squad Workspace](./features/completed/squad-workspace.md) — club-scoped overview, Squad / Planner / Tactic navigation, Dashboard Club Setup, explicit CSV imports, and sequential squad-wide development actions
- [Snapshot history and management](./features/completed/snapshot-history.md) — retained snapshots selected by greatest valid in-game date, snapshot-owned Moneyball enrichment, immutable management context, and Dashboard save/snapshot controls while product reads remain current-only
- [CSV enrichment persistence and derived statistics](./features/completed/csv-enrichment-persistence.md) — save-scoped Youth career and snapshot-versioned Moneyball enrichment for current memory-backed player UIDs, canonical 138-key statistics, and Academy career projections while keeping memory authoritative
- [CSV parsing and player reconciliation](./features/completed/csv-player-reconciliation.md) — bounded Youth Tracker and Moneyball parsing plus exact UID reconciliation foundation; the parser remains non-mutating while persistence lives in the downstream enrichment feature
- [Player Development Boosts](./features/completed/player-development-boosts.md) — two guarded player-profile actions for snapshot-derived CA and mentality boosts through a live-validated FM26 write bridge
- [Potential role scores](./features/completed/potential-role-scores.md) — CA-to-PA visible-attribute projection, Current/Potential profile and assigned-role scores, and explicit current or potential Planner optimization
- [FM SuperScout reader parity](./features/completed/fm-superscout-parity.md) — schema-v6 direct player, staff, manager, scope, and club parity plus deterministic scan hardening and prior-output safety; PR 2 remains unpublished and unmerged
- [Planner workspace redesign](./features/completed/planner-workspace-redesign.md) — URL-backed Planner workspaces, adaptive dual-phase tactic pitches, responsive three-team depth matrix, and transactional Clear all
- [Youth Academy](./features/completed/youth-academy.md) — save-scoped `Class of YYYY` cohorts, club-family player tracking, manual sale/release outcomes, and nullable career statistics that CSV enrichment can populate
- [Codex workflow migration](./features/completed/codex-workflow-migration.md) — Codex-only repository guidance, specialist agents, and product-focused validation
- [Player profiles](./features/completed/player-profiles.md) — `/players/$uid` with a compact player summary, tabbed attribute groups, pitch-selected position roles, and entry from Search row and Ctrl+K
- [Squad planner](./features/completed/squad-planner.md) — save-scoped club family, one dual-phase tactic, and Senior / Reserves / Youth depth strings with manual player assignment
- [Squad optimizer](./features/completed/squad-optimizer.md) — exact team/string allocation by combined score, persisted assignment provenance, and selected-team Clear Squad
- [Planner optimizer preferences](./features/completed/planner-optimizer-preferences.md) — per-lane IP/OOP weights, ranked-lane allocation, and preferred-foot optimizer rules
- [Planner module refactor](./features/completed/planner-module-refactor.md) — behavior-preserving Planner-private Rust and React component decomposition
- [Player search](./features/completed/player-search.md) — `/search` route, virtualized table, operator filters (scalars through role scores), URL-persisted filter/sort state, global Ctrl+K name suggest
- [Role scoring engine](./features/completed/role-scoring-engine.md) — FM26 IP/OOP catalog (68 roles), scores on ingest into `player_role_scores`, DLP IP sanity proof, `combine_role_scores` helper
- [FM26 memory read](./features/completed/fm26-memory-read.md) — initial BepInEx bridge, file protocol, dump schema v5, Rust validation, bridge status + Load Data UI; the historical schema-v6 parity baseline is recorded in [FM SuperScout reader parity](./features/completed/fm-superscout-parity.md), while the current schema-v7 familiarity contract is recorded in [Complete Player Position Familiarity](./features/completed/player-position-familiarity.md)
- [In-app bridge plugin install](./features/completed/bridge-plugin-install.md) — install / update / remove `FmDataBridge.dll` for default Steam FM26 from the app
- [Snapshot ingest + Load Data](./features/completed/snapshot-ingest.md) — app save slots, current snapshot ingest, Load Data scan+ingest, sanity list
- [Bridge scan performance](./features/completed/bridge-scan-performance.md) — block heap scanning, streaming dump, optimized ingest, unlimited Load Data default, UI cap controls and timings
- [Compact Snapshot Metrics and Load Progress](./features/completed/compact-snapshot-metrics.md) — compact current-only player and staff role metrics in `app-v2.db` (migrations v38/v39), bounded preparation outside the database mutex, command-scoped best-effort phased Load Data progress, and disjoint timing reporting
