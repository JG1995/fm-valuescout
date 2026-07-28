# 0005 — TanStack Query

## Status

Accepted (amended — desktop IPC framing; Query defaults `a4cc97f`)

## Context

Desktop apps load async data through **IPC** (`invoke` to Rust), not HTTP. Command results can fail, go stale after mutations, and must be cached and invalidated like any async state. Putting that logic in Zustand or component state leads to duplicate invokes, manual loading flags, and stale UI. **IPC-backed async state** has different rules than client UI state — and different defaults than remote HTTP APIs.

TanStack Query’s stock defaults (`refetchOnWindowFocus`, automatic retries) target browser apps fetching remote servers. Local IPC calls are fast and deterministic; focus and reconnect refetch are usually noise.

## Decision

Use **TanStack Query v5** for all **async state from IPC**: invoke reads, mutation invalidation, cache updates, and background refresh when a feature explicitly requests it.

Route loaders may call `queryClient.ensureQueryData` or `prefetchQuery`. Components subscribe with `useSuspenseQuery` or `useQuery` per feature needs. Fetchers call `invokeCommand` through `src/lib/tauri-client.ts` — not `fetch` to HTTP endpoints.

**Production defaults** in `src/app/router.tsx` tune the global `QueryClient` for desktop IPC:

- `staleTime: 60_000`
- `refetchOnWindowFocus: false`
- `refetchOnReconnect: false`
- `retry: false` for queries

Forks override per `queryOptions` when a resource needs HTTP-style refetch or retry (hybrid apps, remote APIs). See `coding-standards/references/react.md` § Desktop Query defaults.

## Alternatives considered

### SWR

SWR offers a smaller API surface and good defaults for simple fetch-and-display. TanStack Query has stronger mutation, invalidation, and devtools support for dashboards and forms that mutate data.

### Raw fetch + useState / Zustand

Manual invoke or HTTP fetch in components or global stores recreates caching, deduplication, and refetch logic poorly. Solo projects pay the cost later when IPC surfaces grow.

### Redux Toolkit Query

RTK Query fits Redux-heavy apps. This template avoids Redux ceremony. Query pairs with Zustand without a global Redux store.

## Consequences

### Positive

- Industry-default async state layer for React, adapted for IPC in this template.
- Pairs with TanStack Router loaders for prefetch and deferred loading.
- Clear boundary: Query owns IPC-backed async data; Zustand owns client UI state.
- Global defaults avoid pointless refetch on window focus for local commands.

### Negative

- Requires discipline — developers must not store invoke result lists in Zustand.
- Forks adding HTTP APIs must set per-query refetch/retry — global defaults assume local IPC.
- QueryClient setup and provider wiring add scaffold boilerplate.

### Follow-up

- Done at scaffold (`fc11b5a`) — `QueryClientProvider` in app providers.
- Done later — `@tanstack/react-query-devtools` gated on `import.meta.env.DEV` in `AppProvider`.
- Done at scaffold (`ba27c64`) — health feature query options, loader, and `useSuspenseQuery` example.
- Done (`a4cc97f`) — desktop IPC query defaults in `src/app/router.tsx`; `src/app/router.test.ts` asserts production defaults.

## Related work

- Commits: `fc11b5a`, `ba27c64`, `a4cc97f`, `4c58a65` (desktop IPC framing amendment)
- Supersedes: none
