# 0006 — Zustand for client state

## Status

Accepted

## Context

Applications need shared client state that does not belong in the URL or the server cache: modals, drawers, layout chrome, ephemeral wizard steps, and UI selections that are not shareable links. TanStack Query handles remote data; TanStack Router handles URL state. We still need a small global store for client UI state.

## Decision

Use **Zustand v5** for **client UI state only**. Do not store API response data or server cache in Zustand stores.

## Alternatives considered

### TanStack Store

TanStack Store is the TanStack team's signal-based store. It integrates with other TanStack packages internally but has smaller standalone adoption than Zustand. Most downloads come from TanStack libraries, not direct app use. Choosing Store over Zustand buys little for a solo SPA template today.

### Redux Toolkit

Redux Toolkit fits large teams with audit trails and middleware-heavy workflows. Setup and boilerplate exceed solo hobby scope for typical UI state.

### React Context only

Context works for theme or rare global values. Frequent updates through Context cause broad re-renders. Zustand selectors avoid that without provider nesting.

### No global store (useState only)

Possible for tiny apps. Real products quickly add modals, sidebars, and cross-page UI state. Starting without a store pattern invites ad-hoc Context sprawl.

## Consequences

### Positive

- Minimal API; no providers required for basic stores.
- Small bundle; mature ecosystem (devtools, persist middleware when needed).
- Complements TanStack Query without overlapping responsibilities.

### Negative

- Another library to learn alongside Query and Router.
- Layer boundary (Query vs Zustand) is convention-enforced, not lint-enforced.

### Follow-up

- Done at scaffold (`fc11b5a`) — layout store in `src/stores/use-layout-store.ts`.
- State boundaries follow the React reference in the installed `coding-standards` skill.

## Related work

- Commits: `fc11b5a`
- Supersedes: none
