# React — Tauri desktop UI (Bulletproof React adapted)

Load this file when you implement or review application code under `src/`, or when the diff touches React, TanStack Router, TanStack Query, or Zustand.

Read `references/universal.md` first. Read `references/testing.md` when the change adds or changes tests. IPC, capabilities, and plugins live in `references/tauri.md`. Rust command implementation lives in `references/rust.md`. Layer boundaries live in `.wiki/ARCHITECTURE.md`. Security audit depth for the WebView client lives in `security-audit/references/react.md` — load matching `security-audit/references/` files via `security-audit/SKILL.md` when auditing XSS, client env, or client-only auth.

This document is the **baseline** for how React code in this template should look and behave. It adapts [Bulletproof React](https://github.com/alan2207/bulletproof-react) for **Tauri v2 + Vite + TanStack Router + TanStack Query + Zustand**. The WebView is a **thin presentation layer** — business logic and heavy computation live in Rust (`references/rust.md`).

Forks may add `references/project.md` to override or extend these rules. This template targets a **Tauri WebView client** — not SSR, Server Components, or static hosting.

## Stack defaults

| Layer | Library | Role |
| --- | --- | --- |
| UI | React 19 | Components, layout, interaction |
| Routing | TanStack Router | Paths, search params, loaders |
| Async / domain data | TanStack Query v5 | IPC command results — cache, refetch, mutations |
| Client UI state | Zustand v5 | Chrome and ephemeral UI — not domain data |
| Styling | Tailwind CSS v4 | Utilities backed by design tokens |
| Forms | React Hook Form + Zod | Non-trivial forms (fork adds when needed) |
| IPC boundary | `src/lib/tauri-client.ts` | Sole `invoke` wrapper — see `references/tauri.md` |
| Language | TypeScript strict | Types at compile time |

Bundler, Vitest, and Tauri dev settings live in `references/vite.md`.

## Architecture principles

Follow these principles from Bulletproof React:

- **Clear boundaries** between app shell, features, and shared code.
- **One direction of dependencies** — shared → features → app.
- **Feature folders** hold most product code; routes wire features together.
- **Consistency** — same patterns for state, API calls, and file naming across the repo.
- **Detect issues early** — types, lint, tests, and localized error boundaries.

Pick **one** owner per piece of state. Do not mirror the same data in two systems.

## Thin frontend, thick backend

This template uses a **thin frontend, thick backend** split:

| Layer | Owns | Does not own |
| --- | --- | --- |
| **Rust** (`src-tauri/`) | Business rules, aggregation, filtering, sorting, joins, file/DB I/O, heavy loops, validation at trust boundaries | JSX, CSS, route URLs |
| **React** (`src/`) | UI, user input, routing, Query cache of **already-computed** IPC results, presentation formatting | SQL execution, large dataset transforms, file/DB I/O, blocking work on the main thread |

**Why:** Data-heavy work in the WebView blocks the UI thread and duplicates logic Rust can run faster with direct DB and file access. TanStack Query still caches IPC results — it is not a substitute for moving computation to the backend.

**Default placement:**

```text
User action → React (params, UI state) → IPC command → Rust (compute, persist) → typed result → Query cache → component render
```

| Work type | Where |
| --- | --- |
| Filter/sort/page a large table | Rust command — pass filter params from Router search or form; return a **page** of rows + total count |
| Join across tables / analytics | Rust `service.rs` — SQL runs in Rust, not in the WebView |
| Read/write persisted rows | Rust command → `service.rs` → SQLite; React calls `invokeCommand` only |
| Format a date for display | React component |
| Validate form shape for UX | Zod on client; **same rules enforced in Rust** |
| Derive `isSelected` from UI state | `useState` / Zustand |
| Sum 10 visible rows for a footer | React — data already small |
| Sum millions of rows | Rust command — return the scalar |

When a command response is still too large, use pagination, narrower projections, or `Channel` streaming (`references/tauri.md`) — do not grow the WebView payload until the UI locks up.

**Database:** React never calls `@tauri-apps/plugin-sql` or `Database.load` in product features. Schema, migrations, and queries live in Rust — see `references/tauri.md` §Database and `references/rust.md` §Database.

## Project structure

```text
src/
├── app/                    # Application shell (compose features here)
│   ├── routes/             # TanStack Router file routes — thin wiring
│   ├── provider.tsx        # Global providers (Query, Router context)
│   └── router.tsx          # Router factory when not fully file-driven
├── features/               # Primary code home — one folder per feature
│   └── <feature>/
│       ├── api/            # Request declarations, queryOptions, mutations
│       ├── components/     # Feature-scoped UI
│       ├── hooks/
│       ├── stores/         # Feature-scoped Zustand when needed
│       ├── types/
│       ├── utils/
│       └── assets/
├── components/             # Shared UI (design system / ui primitives)
│   └── ui/                 # button/, panel/, status-chip/, empty-state/, field/
├── hooks/                  # Shared hooks (UI behavior, not domain logic)
├── lib/                    # tauri-client.ts — sole IPC wrapper
├── config/                 # Env exports, app constants
├── types/                  # Shared app types (DTOs mirroring IPC shapes)
├── utils/                  # Shared presentation helpers (formatting, cn)
├── assets/                 # Static imports (images, fonts)
├── stores/                 # Global UI Zustand only (notifications, layout)
├── testing/                # Vitest setup, mockIPC helpers, renderWithProviders
├── styles/                 # Global CSS, Tailwind @theme token bridge
├── main.tsx                # Entry — mount app, import global styles
└── routeTree.gen.ts        # Generated by TanStack Router plugin (do not edit)
```

Only add subfolders a feature needs. Do not create empty `api/` or `stores/` folders for every feature.

### Import rules

- **Unidirectional flow:** `shared` (components, hooks, lib, types, utils) → `features` → `app`. Never import from `app` inside a feature.
- **No cross-feature imports.** Compose features in `app/routes/` route files.
- **No barrel `index.ts` re-exports.** Import the concrete file. Barrel files hurt tree-shaking in Vite.
- Use the `@/` import alias for `src/` paths. Do not use deep relative imports like `../../../components`.

### Naming

- **Files and folders:** kebab-case (`discussion-list.tsx`, `use-discussion-list.ts`).
- **Component exports:** PascalCase (`DiscussionList`).
- **Store hooks:** `use-something-store.ts` exporting `useSomethingStore`.
- **Query options:** suffix with `QueryOptions` or use a `queryKeys` object — stay consistent within the project.
- **Tests:** colocate `discussion-list.test.tsx` next to the source file.

Do not edit `src/routeTree.gen.ts`. The TanStack Router Vite plugin generates it.

## State ownership

| State kind | Owner | Examples |
| --- | --- | --- |
| Widget-local | `useState` / `useReducer` | Open section, hover, single-field draft before submit |
| URL-shareable | TanStack Router `params` / `search` | Tab, sort, filter — **send to Rust**, do not apply client-side on full datasets |
| Domain / async | TanStack Query | IPC command results — lists, detail records, mutation outcomes |
| Client shared UI | Zustand | Nav rail expansion, command palette, selection chrome — not entity maps |
| Form | React Hook Form or local state | Multi-field forms with validation |
| Low-velocity global | React Context | Theme snapshot — not high-frequency updates |

**Do not** store large API/IPC response arrays or entity maps in Zustand. **Do not** fetch in `useEffect` when TanStack Query can own the request. **Do not** run `O(n)` or worse transforms on large arrays in render, `useMemo`, or Zustand — push the work to a Rust command and query with parameters.

Derived **presentation** values belong in the component (`formatDate(row.createdAt)`). Derived **domain** values on large data belong in Rust (`compute_totals`, SQL aggregate).

Localize UI state as close as possible to the components that use it. Do not copy Query cache into Zustand to "make it easier to read."

## API layer (IPC)

Domain data flows through Tauri commands, not HTTP. Full IPC rules live in `references/tauri.md`.

### Single IPC client

Create one wrapper in `src/lib/tauri-client.ts`. Every feature fetcher calls `invokeCommand` from there — not `invoke` directly in components.

- Map IPC failures to a typed error class (mirror tagged `AppError` from Rust when used).
- Centralize user-visible error handling (toast hook point) in the wrapper or a thin error helper.

### Request declarations

Define IPC calls in `features/<feature>/api/`, not inline in components. Each declaration should include:

1. Types in `features/<feature>/types/` (or shared `src/types/` when truly shared).
2. A fetcher that calls `invokeCommand` with **narrow args** — filters, page, sort — so Rust returns a bounded result.
3. `queryOptions` or mutation options for TanStack Query — not ad-hoc `useQuery` in route files.

Feature `api/` is for **IPC fetchers and Query options only** — not SQL plugin calls, not raw `invoke`.

```typescript
// Pattern sketch — server-side filter + page
export const itemsQueryOptions = (params: ItemListParams) =>
  queryOptions({
    queryKey: queryKeys.items(params),
    queryFn: () => fetchItems(params),
  });
```

Pass Router search params (or form state) into the query key and fetcher so refetches hit Rust with new parameters — do not fetch everything once and filter in JavaScript.

Do not call `invoke` or `fetch` directly inside inline `useQuery` callbacks in route files.

### HTTP client (legacy walking skeleton only)

`src/lib/api-client.ts` exists only until the IPC migration commit removes the HTTP health demo. **Do not** add new HTTP fetchers alongside IPC for the same feature. Desktop-only template — no dual boundary.

## TanStack Router

Configure the router with **`defaultPreloadStaleTime: 0`** when using TanStack Query so Router preload does not override Query freshness.

File routes live under `src/app/routes/`. Route files are thin: loaders, search validation, and composition of feature components.

### File routes

- Define routes using the file-route API from `@tanstack/react-router`.
- Register search params with validation when they drive UI (filters, pagination). Use Router schemas instead of manual `URLSearchParams` parsing in components.
- Keep **loader functions thin**: seed the Query cache or trigger prefetch; **no** heavy data transforms in the loader — those belong in Rust commands.
- **Code-split** route components with `lazy()` when the route bundle grows.

### Loader + Query pattern (default for reads)

1. Define `queryOptions` in `features/<feature>/api/`.
2. In the route `loader`, call `context.queryClient.ensureQueryData(queryOptions)` for data the route needs before first paint.
3. In the route component, read with `useSuspenseQuery(queryOptions)` so the component subscribes to cache updates.
4. Optionally prefetch slower queries in the loader without `await`, and wrap the slow section in `<Suspense>`.

Do not return loader data and also read Query unless you have a documented exception. Pick Query as the single source of truth in the component.

### Navigation

Use Router `Link` and `useNavigate` from `@tanstack/react-router`. Do not use raw `<a href>` for internal routes.

## TanStack Query

### Desktop Query defaults

Production **`queryClient`** defaults in `src/app/router.tsx` target **desktop IPC**, not remote HTTP:

| Option | Value | Why |
| --- | --- | --- |
| `staleTime` | `60_000` | Avoid immediate re-invoke on navigation within the stale window |
| `refetchOnWindowFocus` | `false` | Local IPC does not go stale on focus like remote APIs |
| `refetchOnReconnect` | `false` | No network reconnect semantics for invoke |
| `retry` (queries) | `false` | IPC failures are usually immediate; retry at the query level when needed |

Override in feature `queryOptions` when a resource uses HTTP, needs retry, or should refetch on focus. Test helpers (`renderWithProviders`, isolated router tests) use their own `QueryClient` instances — do not assume they mirror production defaults. Assert production defaults in `src/app/router.test.ts`. See [ADR-0005](../../../../.wiki/decisions/0005-tanstack-query.md).

### Query options and fetchers

- Define **query options** with `queryOptions()` helper (v5) in feature `api/` folders.
- Use a **query key factory** (`queryKeys.items()`, `queryKeys.item(id)`) in one module when keys are reused across loaders, hooks, and invalidation.
- Set **`staleTime` and `gcTime`** intentionally per resource. Large lists: prefer shorter `gcTime` or paginated keys so the cache does not hold unbounded rows.
- **Pagination:** query key includes page/limit; fetcher passes them to Rust. Do not use Query to cache an entire table when the UI shows one page.
- **Errors:** surface user-visible error UI in the component (`isError`, `error`) or route error boundary. Do not swallow failed IPC calls.
- **Mutations:** use `useMutation` with `onSuccess` invalidation of affected keys. Heavy writes run in Rust — mutation `mutationFn` is a thin `invokeCommand` call.
- **Prefetch:** use `queryClient.prefetchQuery` in loaders or on hover when navigation is predictable.

Fetch functions live in feature `api/`. They return typed data or throw. Validate untrusted **input** with Zod before `invokeCommand`; Rust still validates at the command boundary.

## Zustand

- **Global UI:** `src/stores/` — one store per UI domain (`use-layout-store.ts` → `useLayoutStore`).
- **Feature UI:** `features/<feature>/stores/` when only that feature needs the state.
- Export a hook; avoid exporting the raw store unless tests need it.
- Keep actions in the store object. Name actions as verbs (`toggleRail`, `setPanelOpen`).
- Use selectors: `useLayoutStore((s) => s.railExpanded)` to limit re-renders.
- **Persist** only when product needs it (`persist` middleware). Do not persist IPC/Query data or large collections.

```typescript
// Pattern sketch
export const useLayoutStore = create<LayoutState>()(
  persist(
    (set) => ({
      railExpanded: false,
      toggleRail: () => set((s) => ({ railExpanded: !s.railExpanded })),
    }),
    { name: "fm-valuescout-layout" },
  ),
);
```

## Forms

- **Trivial forms** (one or two fields): controlled local state.
- **Non-trivial forms:** React Hook Form + Zod validation.
- Abstract shared field components in `components/ui/field/` (`text-field.tsx`, `select-field.tsx`, `field-styles.ts`).
- Validate on the client for UX; Rust commands must still validate at trust boundaries.

## Components

### General rules

- Prefer **function components** and hooks. No class components unless a library forces them.
- **Colocate** components, hooks, and helpers inside the feature that owns them.
- Keep components **small**. Split when a file mixes layout, data loading, and heavy markup.
- **Route components** wire data hooks and layout. Move reusable markup to `features/<feature>/components/` or `components/`.
- Props: use a `type` or `interface` for props when the component has more than one prop or non-trivial props. Name it `ComponentNameProps`.
- **Limit props.** When a component accepts too many props, split it or use composition (`children`, slots).
- **No nested render functions** like `renderItems()` inside a component. Extract a separate component.
- Keys in lists: use stable ids from data, not array index, when the list can reorder or filter.
- Avoid `default export` for shared components unless a route file requires it for the plugin; prefer named exports.

### Shared UI library

- Put reusable primitives in `components/ui/` (`button/`, `panel/`, `status-chip/`, `empty-state/`, `field/`).
- **Wrap third-party components** (headless libraries, Router `Link`) so you can change the underlying implementation without touching feature code.
- Identify repetition before abstracting — avoid wrong abstractions.
- Headless libraries (Radix UI, Base UI, Headless UI) are optional; wrap them in `components/ui/` when you add them.

### Composition

- Use `children` for composition; avoid prop drilling through more than two layers — use Zustand or Query at that point.
- Use **Context** to pass data to descendants. Do not use `React.cloneElement` on `children`.
- Context fits **low-velocity** data (theme, layout snapshot). For medium- or high-velocity updates, use Zustand selectors instead of Context to avoid broad re-renders.

### WebView-only effects

Use `useEffect` for subscriptions, DOM listeners, and non-Query side effects. Do not use `useEffect` to load domain data — use Query.

## Component robustness (WebView)

This template runs in a Tauri WebView — not on a server. Skip SSR, hydration, Server Components, and taint APIs unless you fork to a meta-framework.

### WebView-safe render

Do not read `window`, `document`, `localStorage`, or `sessionStorage` during the first render if the value is required synchronously. Read after mount in `useEffect`, or default safely until storage loads.

### Instance-proof

Never hardcode DOM `id` values when a component can mount more than once. Use `useId()` for ids and `aria-*` wiring.

### Portal-proof

Global listeners on `window` break when the component lives in a portal subtree. Attach listeners to the correct view:

```typescript
useEffect(() => {
  const win = ref.current?.ownerDocument?.defaultView ?? window;
  win.addEventListener('keydown', onKeyDown);
  return () => win.removeEventListener('keydown', onKeyDown);
}, []);
```

### Future-proof

`useMemo` is a performance hint, not a correctness guarantee. When correctness depends on a value persisting across renders, use `useState` with a lazy initializer or sync state when props change:

```typescript
const [colors, setColors] = useState(() => generateAccentColors(baseTheme));
const [prevTheme, setPrevTheme] = useState(baseTheme);
if (baseTheme !== prevTheme) {
  setPrevTheme(baseTheme);
  setColors(generateAccentColors(baseTheme));
}
```

## Styling

- Use **Tailwind utility classes** in JSX for layout and spacing.
- Use **design tokens** from `src/styles/` (`@theme` mapped to `.wiki/DESIGN.md`) for colors and typography. Do not hardcode hex or oklch values in components when a token exists.
- Prefer `cn()` or template literals for conditional classes — keep class strings readable; split long lists across lines.
- Global CSS stays in `src/styles/`. Component-specific CSS is rare; use utilities first.
- Prefer **zero-runtime** styling (Tailwind) over runtime CSS-in-JS for performance.

Visual and interaction rules (contrast, focus, motion) live in `.wiki/DESIGN.md` and `.cursor/skills/ui-design/SKILL.md`.

## TypeScript in React code

- Prefer **`type` for props** and API DTOs; use `interface` when declaration merging is needed.
- Avoid `any`. Use `unknown` and narrow at boundaries.
- Let Query and Router inference work — pass typed `queryOptions` and validated search params instead of casting.
- Do not use non-null assertion (`!`) to silence missing data — handle loading and error states from Query and Router.
- On large refactors, update type declarations first, then fix TypeScript errors project-wide.

## Errors and loading

- **Error boundaries:** place multiple localized boundaries — route, feature, or layout — not only one root boundary.
- **IPC errors:** handle in `tauri-client` (toast, logging hook) and in component UI for inline recovery. Match on `kind` when Rust uses tagged `AppError`.
- **Suspense boundaries:** place at route or layout boundaries when using `useSuspenseQuery`.
- **Empty states:** show explicit empty UI when lists have zero items — not a blank main area.
- **Loading:** prefer Suspense fallbacks or Query `isPending` for mutations — avoid unlabeled spinners with no context.
- **Destructive actions:** confirm before execute per DESIGN.md checklist.
- **Devtools:** Query and Router Devtools mount only under `import.meta.env.DEV`. Do not ship them in production.
- **Production tracking:** wire Sentry or similar at error boundaries — fork choice; leave hook points in scaffold.

## Security

The template does not ship auth. Forks that add auth should follow these rules:

- Store session tokens in OS-backed storage via a Tauri plugin when XSS is a concern — not large secrets in `localStorage`.
- Treat session display data as Query-managed IPC state, not a Zustand mirror of backend payloads.
- **Sanitize** user HTML before render (e.g. markdown preview components).
- Implement permission guard components in `lib/authorization.tsx` — hide UI; **authorization still runs in Rust commands**.
- Run `.cursor/skills/security-audit/SKILL.md` before first deploy with auth or sensitive data. Load `security-audit/references/react.md`, `tauri.md`, and `rust.md` for WebView and IPC boundaries.

## Performance

- **Compute in Rust:** filtering, sorting, grouping, statistical aggregates, and large file parsing — not in React render or `useMemo` over big arrays.
- **Bounded IPC payloads:** return pages, summaries, or IDs — not full tables when the UI shows a slice.
- **Code-split** at route level with `lazy()` — do not split so finely that load time suffers from too many chunks.
- **State locality:** keep UI state close to consumers; split global Zustand into domain stores.
- **Expensive initial state:** use `useState(() => expensive())` only when `expensive()` is cheap presentation setup — not domain computation.
- **Children optimization:** pass static subtrees as `children` so parent state updates do not re-render them.
- **Context:** use for low-velocity data only. Use Zustand selectors for frequent UI updates.
- **Prefetch** in route loaders when the next screen is predictable.
- **Images:** lazy-load below-the-fold images; use modern formats and `srcset` when size varies by viewport.
- **Web vitals:** less critical than web SEO for desktop apps, but INP still reflects main-thread blocking — treat lock-up as a signal to move work to Rust.

Avoid `useWebWorker` for domain logic that belongs in Rust unless the worker is a deliberate fork choice — IPC to Rust is usually simpler than duplicating logic in JS workers.

## Testing (React-specific)

Follow `references/testing.md` for TDD scope and the quality gate.

Bulletproof React favors **integration tests** for feature flows and **unit tests** for shared `ui/` primitives and pure utils. Test what the user sees, not implementation details.

- **mockIPC** in `src/testing/setup.ts` — see `references/tauri.md`. Call `clearMocks()` in `afterEach`.
- **Test helpers:** `renderWithProviders()` wrapping QueryClient and Router.
- Query components: wrap with `QueryClientProvider` and a fresh `QueryClient` per test.
- Router components: use Router test utilities or memory history — do not mock Router internals.
- Zustand: reset store state in `beforeEach` when tests mutate global UI state.
- Prefer **RTL queries by role and accessible name** (`getByRole('button', { name: 'Save' })`).
- **E2E:** Playwright smoke in `e2e/` — IPC stub via `page.addInitScript` when needed (`references/tauri.md`). Run `./scripts/dev smoke`.
- Test **presentation** in React; test **computation and persistence** in Rust `cargo test` — do not duplicate heavy domain logic tests only in Vitest.

## Anti-patterns

| Do not | Do instead |
| --- | --- |
| `useEffect` + `fetch` / `invoke` for domain lists | TanStack Query + feature `api/` fetcher |
| Store IPC/API responses in Zustand | Query cache |
| Filter/sort/page 10k+ rows in `useMemo` | Rust command with params; Query caches the page |
| Load full dataset "for flexibility" | Paginated or summarized IPC responses |
| Parse `window.location` manually | Router search params |
| Large route files with all markup | Split into feature `components/` |
| Barrel `index.ts` that re-exports everything | Direct imports |
| Cross-feature import | Compose in `app/routes/` |
| `renderItems()` nested functions | Extract a component |
| `cloneElement` for data passing | Context |
| `useMemo` for correctness-critical values | `useState` + sync on prop change |
| `window` / `localStorage` in render | Deferred read or safe default |
| Global `window` listener | `ownerDocument.defaultView` |
| Central `src/api/` for all IPC | Feature `api/` + `lib/tauri-client.ts` |
| Domain logic in `features/*/utils/` | Rust `service.rs` / `logic.rs` |
| `Database.load` or `@tauri-apps/plugin-sql` in features | Rust command + `service.rs` SQL |
| PascalCase file names | kebab-case files, PascalCase exports |
| One app-wide error boundary | Localized boundaries |
| Prop-drill through many layers | Composition or Context first; then Zustand or Query |
| `console.log` for user errors | Visible error UI |
| Index as React key for dynamic lists | Stable id from data |

## Related decisions and resources

| Topic | ADR |
| --- | --- |
| React | [0001](../../../../.wiki/decisions/0001-react-for-ui.md) |
| TypeScript | [0002](../../../../.wiki/decisions/0002-typescript.md) |
| TanStack Router | [0004](../../../../.wiki/decisions/0004-tanstack-router.md) |
| TanStack Query | [0005](../../../../.wiki/decisions/0005-tanstack-query.md) |
| Zustand | [0006](../../../../.wiki/decisions/0006-zustand-client-state.md) |
| Tailwind v4 | [0007](../../../../.wiki/decisions/0007-tailwind-css-v4.md) |
| Vitest / RTL | [0008](../../../../.wiki/decisions/0008-vitest-and-rtl.md) |

**External references:**

- [Bulletproof React](https://github.com/alan2207/bulletproof-react) — feature layout (adapted for IPC + thin frontend)
- [Building Bulletproof React Components](https://shud.in/thoughts/build-bulletproof-react-components) — component robustness
- [React TypeScript Cheatsheet](https://react-typescript-cheatsheet.netlify.app/)
- IPC and capabilities — `references/tauri.md`
- Rust commands and modules — `references/rust.md`

Paths from this file: `.cursor/skills/coding-standards/references/react.md` → `.wiki/decisions/`.
