# 0004 — TanStack Router

## Status

Accepted (amended — desktop routing framing)

## Context

Desktop apps need **in-app navigation** between views, panels, and settings — not only a single static shell. Typed routes and validated search params support **deep links** (custom URL schemes, protocol handlers), **multi-window** routing, and predictable test URLs in development. URL state is still useful; the primary goal is not “shareable SPA links” alone.

TanStack Query integration via route loaders reduces loading flashes and duplicate invoke logic. The template commits to the TanStack ecosystem for routing and IPC-backed async state ([0005](./0005-tanstack-query.md)).

## Decision

Use **TanStack Router** with the **Vite file-based route plugin** (`@tanstack/router-plugin`). Use loaders to seed the TanStack Query cache; read data in components with Query hooks.

Configure **`defaultPreloadStaleTime: 0`** when using TanStack Query so Router preload does not override Query freshness.

## Alternatives considered

### React Router v7 (library mode)

React Router has the largest tutorial pool and the lowest learning curve for basic routes. TanStack Query integration is manual. Typed search params and loader-to-cache patterns are less central in docs and examples.

### React Router v7 (framework / Remix mode)

Framework mode adds server rendering and data APIs. That moves the template toward full-stack web hosting and increases concepts beyond desktop scope.

### No router (single view)

A single-page shell avoids routing setup. Almost every real desktop product adds navigation within weeks. Omitting a router creates immediate rework.

## Consequences

### Positive

- Typed routes and search params catch invalid navigation state early.
- Documented Router + Query loader pattern (`ensureQueryData`, `useSuspenseQuery`).
- Aligns with TanStack Query ADR for one navigation and cache story.
- Deep links and multi-window routes stay possible without a separate routing model.

### Negative

- Steeper learning curve than React Router for newcomers.
- File-based routes and generated `routeTree.gen.ts` need documentation in coding standards.

### Follow-up

- Done at scaffold (`fc11b5a`) — `defaultPreloadStaleTime: 0` in router config.
- Done later — `@tanstack/react-router-devtools` gated on `import.meta.env.DEV` in the root route.
- Route and loader patterns follow the React reference in the installed `coding-standards` skill.

## Related work

- Commits: `fc11b5a`, `ba27c64`, `4c58a65` (desktop routing framing amendment)
- Supersedes: none
