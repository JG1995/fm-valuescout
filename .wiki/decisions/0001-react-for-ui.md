# 0001 — React for UI

## Status

Accepted

## Context

This template targets browser-based web applications. We need a UI library with active maintenance, a large ecosystem, and a low ceremony path for solo developers. The choice affects every component, test setup, and skill reference in the repository.

## Decision

Use **React 19** as the default UI library with `react-dom` for browser rendering.

## Alternatives considered

### Angular

Angular ships a full framework with modules, dependency injection, and RxJS-heavy patterns. It fits large enterprise teams with long-lived standards. For a solo hobby template, the learning curve and boilerplate cost more than they save on small projects.

### Svelte

Svelte compiles components and often produces smaller bundles with less runtime code. The ecosystem and hiring pool are smaller than React. Codex skills, TanStack integrations, and third-party component libraries skew toward React for web dashboards and internal tools.

### Vue

Vue is mature and approachable. This template already commits to the React-focused Codex workflow and TanStack Router/Query integrations that assume React. A Vue fork would be a different template repository.

## Consequences

### Positive

- Largest pool of examples, libraries, and hiring familiarity for web UI.
- React 19 is stable and receives regular patches (visible on npm, July 2026).
- TanStack Router, Query, and React Testing Library are React-first.

### Negative

- React is a library, not a framework — routing and data patterns must be chosen explicitly (documented in other ADRs).
- Bundle size is larger than Svelte for equivalent UIs.

### Follow-up

- Done at scaffold (`41effa2`, `fc11b5a`) — React 19 pinned in `package.json`.
- Component patterns follow the React reference in the installed `coding-standards` skill.

## Related work

- Commits: `41effa2`, `fc11b5a`, `ba27c64`, `2c7f69c`
- Supersedes: none
