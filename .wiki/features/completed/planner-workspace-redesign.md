# Planner Workspace Redesign

## Intent

Make Squad Planner a focused desktop workspace instead of one long page, while keeping the established Planner data and mutation boundaries.

## Delivered behavior

- `/planner` has URL-backed **Squad**, **Tactic**, and **Club setup** workspaces. Valid URL state wins; configured saves default to Squad and first-use saves default to Club setup. Hidden mounted panels preserve local drafts and selections.
- The Tactic workspace uses an attack-up IP/OOP pitch layout, adaptive three-to-five-slot card density, role-independent spatial qualifiers, and one selected-position settings shelf. Its command bar contains phase controls, conditional save status, and **Save tactic**.
- The Squad workspace adapts between one semantic Senior/Reserves/Youth matrix and selected-team tabs according to available width. It keeps compact rows, sticky context, bounded matrix overflow, and one latest successful-action status.
- **Clear all** replaces team-scoped clears. After confirmation, one Rust transaction clears manual and optimized assignments for every team in the active save without changing strings, tactics, club-family settings, scores, or other saves.
- The desktop main window starts maximized.

## Final architecture

- The `/planner` route owns validated `view` search state, workspace defaults, and hidden mounted tab panels. Planner-local components retain their existing draft, query, mutation, and focus ownership.
- Rust remains the Planner authority for persistence, validation, scoring, optimizer allocation, and the save-scoped clear transaction. The schema, migrations, tactic payloads, and other Planner IPC contracts did not change.
- The current architecture and design contracts are in [ARCHITECTURE.md](../../ARCHITECTURE.md) and [DESIGN.md](../../DESIGN.md).

## Important decisions

- Stable lane IDs remain internal. User-facing labels use current positions and roles, with deterministic spatial qualifiers only when needed.
- The tactic command bar does not repeat linked-position context. The pitches and settings shelf provide it instead (`e290b4c`).
- No ADR was required: the work stays within established React, Router, Tauri, and Planner boundaries.

## Delivery and validation

- **PR 1:** not published; not merged. The branch is `feat/planner-workspace-redesign`. `b8ce03102277a0bfe2512e5c2d7b8728e80b89ee` is the local close-out correction commit.
- **Profiles:** Luna Max implementation; Sol High commit reviews and feature-complete review.
- **Feature-complete review:** the initial Blocking No review found a stale command-bar contract and missing Escape focus restoration. The corrections cleared those paths. The final reviewer retained only the accepted right-click limitation below.
- **Final checks:** format made no changes; focused Planner route tests passed 53/53; frontend tests passed 170/170; `./scripts/dev check` passed with Rust 211 passed and 2 ignored; elevated smoke passed 15/15; Git diff checks passed. `mutate` remains unsupported and was not run.
- **Delivery commits:** `a908934`, `2d72067`, `c5d6bce`, `133089d`, `4d45d0f`, `dd29c41`, `31c3c7e`, `9dcc5a4`, `500c081`, `2b7df36`, `13d7320`, `22efa5d`, `15b639a`, `4a1b7ca`, `57176b1`, `01287f3`, `0282949`, `31c3749`, `dd2382e`, `cb98815`, `e290b4c`, `3e8be7a`, `058c044`, and `b8ce031`.
- **Follow-up commits:** `3e8be7a` added tactic phase-control padding; `058c044` configured maximized startup; `b8ce031` restores focus when Escape is pressed on the string trigger or within its menu, with route regressions.

## Follow-up

- Accepted limitation: right-clicking a non-focusable string header opens its menu without moving focus. Escape then has no trigger or menu focus path to restore. The developer accepted this remaining MEDIUM feature-review finding. Revisit it if header context menus need complete keyboard dismissal and focus-restoration support.
