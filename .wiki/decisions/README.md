# Architecture Decision Records

Files in this directory record consequential decisions, alternatives, trade-offs, and consequences. They are not a log of routine refactoring or local implementation choices.

Create an ADR only when all of these are true:

1. The decision has durable structural consequences.
2. Meaningful alternatives existed.
3. A future maintainer could reasonably ask why this option was chosen.

Use a short, numbered filename such as `0001-decision-title.md`.

## Recorded decisions (template default stack)

| ADR | Decision |
| --- | --- |
| [0001](./0001-react-for-ui.md) | React for UI |
| [0002](./0002-typescript.md) | TypeScript (strict) |
| [0003](./0003-vite-spa.md) | Vite 8 SPA |
| [0004](./0004-tanstack-router.md) | TanStack Router |
| [0005](./0005-tanstack-query.md) | TanStack Query |
| [0006](./0006-zustand-client-state.md) | Zustand (client UI state) |
| [0007](./0007-tailwind-css-v4.md) | Tailwind CSS v4 |
| [0008](./0008-vitest-and-rtl.md) | Vitest + React Testing Library |
| [0009](./0009-biome.md) | Biome (lint and format — no ESLint/Prettier) |
| [0010](./0010-pnpm.md) | pnpm + Node 24 |
| [0011](./0011-husky-git-hooks.md) | Husky (`check-fast` on commit — no lint-staged) |
| [0012](./0012-secretlint.md) | Secretlint (secret scan in `./scripts/dev check`) |
| [0013](./0013-tauri-v2-desktop-shell.md) | Tauri v2 desktop shell |
| [0014](./0014-rust-backend-ipc-boundary.md) | Rust backend and IPC trust boundary |
| [0015](./0015-sqlite-rust-owned.md) | SQLite — Rust-owned migrations and queries |
| [0016](./0016-csharp-bepinex-fm26-bridge.md) | C# BepInEx bridge for FM26 memory read |
| [0017](./0017-action-specific-fm26-player-boosts.md) | Action-specific FM26 player boosts |
| [0018](./0018-squad-wide-player-boosts.md) | Squad-wide action-specific player boosts |
| [0019](./0019-lazy-potential-role-score-cache.md) | Lazy persistent potential role-score cache |
| [0020](./0020-action-specific-fm26-staff-ca-boost.md) | Action-specific FM26 staff CA boost |
| [0021](./0021-sequential-club-family-staff-ca-boost.md) | Sequential club-family staff CA boost |

## ADR format

```markdown
# <Number> — <Decision Title>

## Status

Proposed | Accepted | Superseded | Rejected

## Context

What problem or pressure required a decision?

## Decision

What was chosen?

## Alternatives considered

### <Alternative>

Why it was plausible and why it was not chosen.

## Consequences

### Positive

- ...

### Negative

- ...

### Follow-up

- ...

## Related work

- Feature plan:
- Commits:
- Supersedes:
```
