# 0002 — TypeScript

## Status

Accepted

## Context

The default stack includes TanStack Router and Query, which rely on TypeScript inference for routes, search params, and query keys. Solo developers still benefit from catching contract errors before runtime without enterprise ceremony.

## Decision

Use **TypeScript** with **strict** compiler options for all application source. Run `tsc --noEmit` in `./scripts/dev check`.

## Alternatives considered

### JavaScript only

JavaScript removes compile step friction and suits tiny scripts. It forfeits typed routes, typed query options, and early detection of API shape drift. TanStack Router’s main advantages weaken without TypeScript.

### TypeScript with loose options

Loose `strict` settings allow implicit `any` and weaken the value of the type checker. Strict mode matches Vite and TanStack defaults and costs little on small projects.

### JSDoc typed JavaScript

JSDoc can type-check JavaScript without `.ts` files. It is harder to maintain than TypeScript for TanStack file routes and shared API types.

## Consequences

### Positive

- Typed routes and query options catch navigation and cache mistakes at compile time.
- Aligns with Vite, Vitest, and Biome TypeScript support out of the box.
- Forks inherit a clear contract layer for API types.

### Negative

- Slightly slower first-time setup than plain JavaScript.
- Contributors must understand basic TypeScript errors.

### Follow-up

- Done at scaffold (`41effa2`) — `tsconfig.json`, `tsconfig.app.json`, `tsconfig.node.json`.
- Done at scaffold (`2c7f69c`) — `tsc --noEmit` in `./scripts/dev check`.

## Related work

- Commits: `41effa2`, `2c7f69c`
- Supersedes: none
