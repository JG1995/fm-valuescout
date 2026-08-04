# Architecture — FM ValueScout

> Authority: This document describes the implemented stack, application layout, and tooling for FM ValueScout.

This document describes how **FM ValueScout** is constructed: stack, thin-frontend / thick-backend boundaries, build and test pipeline, and conventions enforced by tooling.

Application layout follows [Bulletproof React](https://github.com/alan2207/bulletproof-react) adapted for TanStack Router and Query on the frontend, and feature modules under `src-tauri/src/features/` on the backend. Line-level rules come from the installed `coding-standards` skill and its React, Tauri, Rust, and Vite references.

For product purpose, see [CONCEPT.md](./CONCEPT.md). For rationale behind each default choice, see [.wiki/decisions/](./decisions/README.md).

---

## 1. Top-Level Shape

**FM ValueScout** is a Tauri desktop application built on the React + Tauri v2 stack below, with a Codex workflow (skills, specialist agents, wiki, `./scripts/dev`), a **walking skeleton** (health IPC demo, SQLite persistence), an implemented **FM26 memory-read bridge** (C# BepInEx plugin + Rust file protocol — [ADR-0016](./decisions/0016-csharp-bepinex-fm26-bridge.md), [completed record](./features/completed/fm26-memory-read.md)), **snapshot ingest** (multi-save slots, Load Data scan+ingest into SQLite — [completed record](./features/completed/snapshot-ingest.md)), **role scoring** (FM26 IP/OOP scores computed and persisted on ingest — [completed record](./features/completed/role-scoring-engine.md)), **player search** (virtualized Search page, operator filters, global Ctrl+K name suggest — [completed record](./features/completed/player-search.md)), and the **Squad Planner and Optimizer** (save-scoped club family, dual-phase tactic, depth matrix, and exact automatic allocation at `/planner`).

**Client / UI:** React 19 in a Tauri WebView — presentation layer only

**Bundler / dev server:** Vite 8 with `@vitejs/plugin-react` — bundles the WebView frontend

**Routing:** TanStack Router (file-based routes, typed search params, loaders coordinated with Query)

**Server / async state:** TanStack Query v5 — caches **IPC command results**, not HTTP responses. Global defaults in `src/app/router.tsx` disable focus/reconnect refetch and query retry for local IPC ([ADR-0005](./decisions/0005-tanstack-query.md)).

**Client UI state:** Zustand v5 (modals, layout chrome, selections not in the URL)

**Styling:** Tailwind CSS v4 via `@tailwindcss/vite`; design tokens bridge to [DESIGN.md](./DESIGN.md). IBM Plex Sans/Mono self-hosted via `@fontsource`; Lucide icons via `lucide-react`. Shared primitives in `src/components/ui/` (Button, Panel, StatusChip, EmptyState, TextField, SelectField, **Modal**, **ScoreBadge**). App shell: `AppNavRail` + `AppTopBar` (**GlobalPlayerSearch**, active save, snapshot freshness, optional Load Data player-cap toggle/limit, **Load Data**); `useLayoutStore` persists nav-rail expansion; `useLoadDataPreferences` persists the Load Data cap toggle and limit. Player search results use **@tanstack/react-virtual** for row virtualization.

**Language:** TypeScript (strict) on the frontend; Rust on the backend

**Package manager:** pnpm (pinned in `packageManager`)

**Runtime for tooling:** Node.js 24

**Desktop shell:** Tauri v2 — native window, WebView, IPC runtime, capabilities ACL

**Backend / computation:** Rust in `src-tauri/` — commands, services, SQLite queries, validation at trust boundaries

**Data:** SQLite via **rusqlite** (bundled) in Rust — migrations (`PRAGMA user_version`) and queries; WebView never opens the database directly. Live FM26 player dumps land on disk via the bridge file protocol (`%LOCALAPPDATA%\fm-valuescout\fm-bridge\`); **Load Data** validates and ingests `dump.json` into the active app save’s current snapshot (migrations v2–v10: `saves`, `snapshots`, `players`, `player_role_scores`, and save-scoped Planner club-family, tactic, depth-string, and assignment rows with provenance).

**FM26 bridge:** C# BepInEx 6 IL2CPP plugin in `bridge/` — memory layouts, safe block-based heap scanning (`TryReadBlock`), `status.json` / `dump.json` / diagnostics with phase timings. Rust `features/memory_read` orchestrates requests, validates dump shape, and installs the plugin DLL into Steam `BepInEx/plugins`; React `features/memory-read` shows install controls and bridge status. **Load Data** lives in `AppTopBar`. Windows Steam FM26 only. See [bridge/README.md](../bridge/README.md), [bridge scan performance](./features/completed/bridge-scan-performance.md), and [bridge-plugin-install](./features/completed/bridge-plugin-install.md).

**Snapshot ingest:** Rust `features/snapshot` owns save slots, transactional ingest from `dump.json`, and query IPC; React `features/snapshot` owns the save switcher, snapshot overview, and sanity list. `load_data` captures `active_save_id` under a brief Db lock, runs the bridge scan without holding the Db mutex, then ingests via `ingest_dump_file_for_save` with the captured id. See [snapshot-ingest](./features/completed/snapshot-ingest.md).

**Role scoring:** Rust `features/scoring` owns a static FM26 IP/OOP catalog (68 roles), `score_role`, and `combine_role_scores`. During snapshot ingest, `features/snapshot` computes and persists all role scores in the same transaction. See [role-scoring-engine](./features/completed/role-scoring-engine.md).

**Player search:** Rust `features/search` owns `search_players` and `suggest_players` — parameterized SQLite queries against the active save's current snapshot (`players`, `player_role_scores`, JSON attribute columns). React `features/search` owns the `/search` route, virtualized results table, compact filter strip + editor modal, and top-bar global name search. Filter rules compile to a flat AND|OR AST in Rust; filters, combine mode, and sort live in TanStack Router search params. See [player-search](./features/completed/player-search.md).

**Player profiles:** Rust `features/player` owns `get_player` — one player by `uid` from the active save's current snapshot, including attribute JSON maps and `player_role_scores` merged in-process with the scoring catalog (`displayName`, `phase`, `positionTags`). React `features/player-profile` owns Overview / Attributes / Roles tab panels; route `/players/$uid` with validated `tab` search param. Shared **ScoreBadge** (`table` / `card` / `hero` / `muted`) in `src/components/ui/score-badge/`. Search row activation and GlobalPlayerSearch navigate by route path only (no cross-feature imports). See [player-profiles](./features/completed/player-profiles.md).

**Planner club family:** Rust `features/planner` owns save-scoped `planner_club_settings` and `planner_club_sources`, current-snapshot club discovery, and validation for `get_planner_club_family`, `list_planner_clubs`, and `save_planner_club_family`. React `features/planner` owns the `/planner` setup panel. The primary club seeds Senior, Reserves, and Youth sources. Pool membership matches the configured club name and ignores dump `teamLevel`, so every primary-club player is eligible for all three Planner teams. Attached sources preserve explicit separate B-team or youth club mappings and add every player at that club to the target team's pool. App-shell save switching and Load Data invalidate planner queries alongside snapshot and player queries.

**Planner tactic:** Rust `features/planner` owns eleven ordered, save-scoped `planner_tactic_lanes` rows. Each lane links compatible IP and OOP positions and roles, owns a 0–1 IP weight, one unique optional importance rank from 1 through 11, and a preferred-foot rule. Migration v8 resets only tactic rows and removes the obsolete tactic parent table; migration v9 adds the nullable unique rank; migration v10 adds preferred foot (`any`, `left`, `right`, or `both`) and a Preferred or Strict mode. Planner assignments remain because they reference stable lane IDs. `get_planner_tactic` seeds a validated 4-3-3 DM In-Possession / 4-1-4-1 DM Out-of-Possession tactic; `get_planner_tactic_options` exposes catalog-backed placements and phase/position metadata; `save_planner_tactic` validates and replaces the complete lane set. React `features/planner` loads the tactic and options through TanStack Query. The editor keeps linked phase edits and selected-lane score-weight, rank, and foot-rule drafts local until save.

**Planner depth and optimizer:** Rust `features/planner` owns save-scoped `planner_strings` and `planner_assignments`. Migration v7 adds assignment provenance: existing rows and manual assign or move mutations are `manual`; optimized rows are `optimizer`. It creates one string for each of Senior, Reserves, and Youth; validates add, remove, clear, assign, and move mutations; keeps player UIDs unique across the save; and resolves assignments against the active snapshot. A resolved assignment has current identity and a combined score calculated with its lane's IP weight; an outside-pool assignment still resolves but no longer belongs to its team's configured sources; an unresolved assignment retains its last-known name when its UID is absent. Missing phase scores remain unknown. `optimize_planner_depth` runs one database transaction: it retains manual rows, removes earlier optimizer rows, then allocates eligible players for each team and ordered string in Senior, Reserves, Youth order. Per string, it skips manually occupied lanes, greedily assigns ranked lanes in ascending order with immediate UID reservation, then gives only unranked lanes to the exact matcher. For optimizer allocation only, a matching or unrestricted foot keeps the combined score, a Preferred mismatch subtracts five with a zero floor, and a Strict mismatch removes the lane edge. The matcher maximizes that allocation score, then filled lanes, with a stable UID tie-break. `clear_planner_team` requires confirmation and removes all assignments in only the selected team. `get_planner_depth` returns the complete three-team read model, while `get_planner_slot_candidates` filters the target team's configured sources and ranks candidates by Rust-computed lane-weighted combined score with any current assignment location.

React `features/planner` owns query, picker, confirmation, focus, menu, and presentation state. The `/planner` route composes club-family setup, the tactic editor, and the depth matrix after the snapshot is available. The matrix has keyboard-operable Senior, Reserves, and Youth tabs, sticky lane labels, horizontal overflow, and header menus available by button or right-click. Its Optimize action shows pending, success, and error feedback, replaces the returned depth cache, and invalidates slot candidates. Its destructive Clear Squad action confirms before clearing the selected team. Picker and string mutations reconcile the depth cache and invalidate candidate queries; tactic saves invalidate both because roles and weight change their results. Load Data, active-save changes, and club-family saves invalidate the entire Planner query tree. React displays Rust-provided unresolved, outside-pool, and unknown-score states without recomputing domain values.

**Auth:** None in the template default — chosen per fork via `/stack`

**Distribution:** OS installers built by `tauri-action` on version-tag push (unsigned by default)

**Testing:** Vitest + jsdom + React Testing Library with `mockIPC` (`./scripts/dev test`); Playwright smoke with IPC stub (`./scripts/dev smoke`, `e2e/smoke.spec.ts`); Rust unit tests (`cargo test` inside `./scripts/dev check`); C# bridge unit tests (`./scripts/dev bridge-test` in Windows CI)

**Client env validation:** not shipped in the template default — forks can add `src/config/env.ts` with Zod for `VITE_*` when needed (follow the Vite reference in the installed `coding-standards` skill; `.env.example` documents optional variables)

**Lint / format / types:** Biome + `tsc -b`; secretlint in `./scripts/dev check`; Rust `cargo fmt`, `clippy`, and `test` in the same gate

**Secret scanning:** secretlint (`./scripts/dev secrets`, included in check)

**Observability:** None in the template default

```text
┌─────────────────────────────────────────────────────────────┐
│  Tauri WebView — React 19 components + Tailwind v4          │
├─────────────────────────────────────────────────────────────┤
│  TanStack Router — routes, search params, loaders           │
├─────────────────────────────────────────────────────────────┤
│  TanStack Query — IPC result cache                          │
│  Zustand — client UI state                                  │
│  useState — local widget state                              │
├─────────────────────────────────────────────────────────────┤
│  src/lib/tauri-client.ts — sole invoke wrapper              │
├─────────────────────────────────────────────────────────────┤
│  IPC (invoke) — frontend/backend boundary                   │
├─────────────────────────────────────────────────────────────┤
│  Rust — features/<name>/commands.rs → service.rs → db/     │
│  SQLite — rusqlite migrations + queries                     │
├─────────────────────────────────────────────────────────────┤
│  Vite 8 — WebView bundle; Tauri — native shell + installers │
├─────────────────────────────────────────────────────────────┤
│  Vitest + mockIPC — unit/component tests                    │
│  Playwright + IPC stub — browser smoke                      │
│  cargo test — Rust unit tests                               │
│  Biome + tsc + secretlint + cargo fmt/clippy — gate         │
│  scripts/dev — stable product-test command surface          │
└─────────────────────────────────────────────────────────────┘

Fork chooses: auth, signing, auto-update, additional plugins
```

**Architecture rules:**

- **Thin frontend, thick backend** — React owns UI and presentation; Rust owns computation, aggregation, file/DB I/O, and validation at trust boundaries.
- Put **async data from IPC** in TanStack Query, not in Zustand.
- Put **URL-shareable state** in TanStack Router search params when practical.
- Put **client UI state** in Zustand only when it does not belong in the URL or Query cache.
- Organize **product code in `src/features/`** on the frontend and **`src-tauri/src/features/`** on the backend; keep `src/app/routes/` thin.
- **Do not import across features** — compose features in route files.
- **One invoke wrapper** — `src/lib/tauri-client.ts` is the sole `invoke` import site; feature `api/` folders call through it.
- **No WebView SQL** — do not use `@tauri-apps/plugin-sql` from JavaScript for product features.
- Use `./scripts/dev` for test and check commands — do not bypass with ad-hoc npm scripts in CI. `check-app` is the frontend-only CI gate; `check` remains the full local gate.

---

## 2. Project Layout

### 2.0 Repository layout

```text
your-repo/
├── .codex/            # Project workflow policy, agents, MCP config, and guide
├── .wiki/             # Durable docs (this file, ADRs, TODO)
├── .husky/            # Git hooks (pre-commit → check-fast + conditional check-rust)
├── scripts/
│   └── dev            # test | check | bridge-test | format | smoke | mutate | bridge-install
├── bridge/            # C# BepInEx FM26 plugin (see bridge/README.md, DUMP_SCHEMA.md)
├── src/               # WebView frontend (see below)
├── src-tauri/         # Rust backend + Tauri config (see below)
├── public/            # Static assets served as-is
├── index.html         # Vite HTML entry
├── e2e/               # Playwright smoke specs (excluded from Vitest)
├── .env.example       # Documented optional VITE_* variables
├── playwright.config.ts
├── vite.config.ts     # Vite + React + Tailwind + Router + Vitest + Tauri
├── biome.json         # Lint, format, import zones
├── tsconfig.json      # TypeScript project references
├── package.json       # pnpm scripts → scripts/dev + pnpm tauri
├── pnpm-lock.yaml     # Locked dependency tree
├── AGENTS.md          # Development contract
└── README.md
```

Frontend source follows Bulletproof React: features-first, unidirectional imports, app shell for routing.

```text
src/
├── app/                    # Application shell — compose features here
│   ├── routes/             # TanStack Router file routes (thin wiring)
│   ├── components/         # App-shell UI (AppShellLayout, AppNavRail, AppTopBar, not-found)
│   ├── provider.tsx        # Global providers (Query, Router)
│   └── router.tsx          # Router factory when needed
├── features/               # Primary code home — one folder per feature
│   └── <feature>/
│       ├── api/            # queryOptions, IPC fetchers, mutations
│       ├── components/
│       ├── hooks/
│       ├── stores/         # Feature-scoped Zustand when needed
│       ├── types/
│       ├── utils/
│       └── assets/
├── components/             # Shared UI — ui/ (Button, Panel, StatusChip, EmptyState, field/), error-boundary/
├── hooks/                  # Shared hooks
├── lib/                    # tauri-client.ts (sole invoke wrapper)
├── config/                 # Env exports, app constants
├── types/                  # Shared app types
├── utils/                  # Shared presentation helpers (format.ts)
├── assets/                 # Static imports (images, fonts)
├── stores/                 # Global UI Zustand only
├── testing/                # Vitest setup, mockIPC helpers
├── styles/                 # Global CSS, Tailwind @theme → DESIGN.md
├── main.tsx                # Entry — mount app
└── routeTree.gen.ts        # Generated by TanStack Router plugin (do not edit)
```

Rust backend follows feature modules with shared database helpers:

```text
src-tauri/
├── Cargo.toml              # Rust dependencies (rusqlite bundled, etc.)
├── build.rs                # Tauri build script
├── tauri.conf.json         # Product identity, CSP, build hooks
├── capabilities/
│   └── default.json        # Deny-by-default ACL for the main window
├── icons/                  # App icons for installers
└── src/
    ├── main.rs             # Thin entry — calls lib::run()
    ├── lib.rs              # App shell — plugins, setup, invoke_handler only
    ├── db/
    │   ├── mod.rs          # DB path resolution, open + migrate, APP_DB_FILE
    │   └── migrations.rs   # PRAGMA user_version migration registry
    └── features/
        ├── mod.rs
        └── <feature>/
            ├── mod.rs
            ├── commands.rs # #[tauri::command] handlers
            ├── service.rs    # Business logic, rusqlite queries (when I/O appears)
            └── …             # e.g. memory_read/dump_validation.rs for dump ingestibility checks
```

**Import alias:** `@/` → `src/` (declared in `vite.config.ts` and `tsconfig.json`).

**Dependency direction (frontend):** shared (`components`, `config`, `hooks`, `lib`, `types`, `utils`, `stores`) → `features` → `app`. No reverse imports.

**Naming:** kebab-case files and folders; PascalCase component exports. Frontend feature names match backend feature folders when both sides exist (`health` ↔ `health`).

### 2.1 Source layout rules

| Rule | Enforcement | Effect on code |
| --- | --- | --- |
| File routes under `src/app/routes/` | TanStack Router plugin | New pages add route files; `routeTree.gen.ts` updates on build |
| Feature code under `src/features/<feature>/` | Convention + review | Product logic colocated per feature |
| Query options and IPC fetchers in `features/<feature>/api/` | Convention + review | Single invoke wrapper in `lib/tauri-client.ts` |
| No cross-feature imports | Biome zones + reviewer | Compose features in `app/routes/` |
| Unidirectional imports (shared → features → app) | Biome zones + reviewer | Predictable dependency flow |
| Global UI Zustand in `src/stores/` | Convention + review | Feature UI stores in `features/<feature>/stores/` |
| Shared UI in `src/components/` (especially `ui/`) | Convention + review | Route files stay thin |
| kebab-case file and folder names | Biome + reviewer | `discussion-list.tsx`, not `DiscussionList.tsx` |
| No barrel `index.ts` re-exports | Convention + reviewer | Direct imports for tree-shaking |
| Design tokens in CSS `@theme`, sourced from DESIGN.md | Convention + review | Prefer token classes over ad-hoc hex in components |
| Vitest setup and mockIPC in `src/testing/` | Convention + `vite.config.ts` | `setup.ts` registers IPC mocks |
| Rust commands in `features/<name>/commands.rs` | Convention + review | Not as bare `#[tauri::command]` in `lib.rs` |
| Business logic in `features/<name>/service.rs` | Convention + review | Commands stay thin; services own rusqlite queries |
| Shared DB helpers in `src-tauri/src/db/` | Convention + review | Path resolution, connection, migration registry |

### 2.2 State and reactivity patterns

- **Component-local state** — `useState` / `useReducer` for state inside one component (toggle, open section).
- **Derived state** — compute in the component, or derive in a Zustand selector; do not duplicate Query cache in Zustand.
- **URL / route state** — TanStack Router params and validated search params (filters, tabs, shareable view state).
- **Server / remote state** — TanStack Query (`useQuery`, `useMutation`, query options). Route loaders call `queryClient.ensureQueryData` or `prefetchQuery` to seed the cache before render. Fetchers call `invokeCommand` — not HTTP.
- **Client-only shared state** — Zustand (nav rail expansion, command palette, ephemeral multi-step UI before submit). `useLayoutStore` persists `railExpanded` across launches.
- **Form state** — local state for trivial fields. Add React Hook Form + Zod when the first non-trivial form arrives (not shipped in the template).
- **Low-velocity global** — React Context for theme or auth display snapshot; not high-frequency updates.
- **Side effects** — React `useEffect` for non-data subscriptions; Query handles fetch lifecycle; Router loaders handle navigation-time prefetch.
- **Devtools** — `@tanstack/react-query-devtools` and `@tanstack/react-router-devtools` render only when `import.meta.env.DEV` is true.

### 2.3 Interface contract (fork boundary)

The template ships IPC commands as the frontend/backend contract. Forked projects define contracts in matched feature folders:

- Frontend: query options name the command, key, and stale behavior in `features/<feature>/api/`.
- Backend: `#[tauri::command]` handlers in `src-tauri/src/features/<feature>/commands.rs` return bounded DTOs.
- Types live in `features/<feature>/types/` on the frontend and as Rust structs in commands or `types.rs` on the backend.
- Mutations invalidate Query keys explicitly — document cross-key invalidation in the feature ledger when one mutation clears multiple caches.
- Validate inputs in Rust `service.rs` or commands — the WebView is untrusted.

---

## 3. Build, Test, and Gate Pipeline

### 3.1 Build commands

| Command | Purpose |
| --- | --- |
| `pnpm install` | Install Node dependencies from lockfile; Husky hooks via `prepare` |
| `pnpm tauri dev` | **Default dev loop** — WebView + Rust IPC, real backend |
| `pnpm dev` | Frontend-only Vite dev server; IPC calls fail unless stubbed |
| `pnpm build` | Production WebView bundle to `dist/` (plain Vite — no Tauri platform env) |
| `pnpm tauri build` | Full desktop build — Vite bundle + Rust compile + OS installer |
| `pnpm preview` | Serve production WebView build locally (no Rust backend) |
| `./scripts/dev test` | Vitest (`vitest run`); no args runs full suite |
| `./scripts/dev test <pattern>` | Vitest with file or name filter |
| `./scripts/dev format` | Biome lint/format fixes (`biome check --write`), then `cargo fmt` in `src-tauri/`; optional path args forward to Biome only |
| `./scripts/dev secrets` | secretlint full-tree scan; `--staged` scans staged files only |
| `./scripts/dev check` | Code-quality gate — Biome + `tsc -b` + secretlint + Rust |
| `./scripts/dev check-app` | Frontend code-quality checks — Biome + `tsc -b` + secretlint |
| `./scripts/dev check-fast` | Fast pre-commit path — Biome + `tsc -b` + secretlint `--staged` |
| `./scripts/dev check-rust` | `cargo fmt --check`, clippy, and test in `src-tauri/` |
| `./scripts/dev bridge-test` | C# bridge unit tests; requires the .NET 6 SDK |
| `./scripts/dev smoke` | Playwright (`e2e/smoke.spec.ts`); starts Vite via `playwright.config.ts` when needed |
| `./scripts/dev bridge-install` | Build `bridge/` and copy `FmDataBridge.dll` into Steam `BepInEx/plugins` (Windows path via `FM_BRIDGE_PLUGINS` / `FM_STEAM_ROOT` / WSL default) |

### 3.2 Validation gate

1. **Biome** — verify lint and format (`biome check`); fail on violations. Autofix via `./scripts/dev format` (also runs `cargo fmt`), not in `check`.
2. **TypeScript** — `tsc -b`; fail on type errors.
3. **secretlint** — `./scripts/dev secrets` (full tree, respects `.gitignore`); included in `check`. Optional `./scripts/dev secrets --staged` without lint-staged.
4. **Rust** — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test` in `src-tauri/`; gated behind `require_rust_toolchain` (requires `cargo` on PATH).
5. **Vitest** — `./scripts/dev test`; CI runs the full suite when frontend or CI files change.
6. **Playwright smoke** — `./scripts/dev smoke`; CI installs Chromium and runs it when frontend or CI files change. Requires `pnpm exec playwright install chromium` once after install locally.
7. **Bridge tests** — `./scripts/dev bridge-test`; CI runs the C# unit suite on Windows when bridge or CI files change. Full FM attach tests remain manual on Windows.

`mutate` remains unconfigured until mutation targets exist.

### 3.3 Git hooks

Pre-commit runs a **fast local gate** (`check-fast`); local pre-merge validation runs the full code-quality gate, while CI selects applicable product suites from changed paths.

| Piece | Choice | Notes |
| --- | --- | --- |
| Hook runner | **Husky** | Installs on `pnpm install` via `prepare` script |
| Pre-commit | `./scripts/dev check-fast` (+ `check-rust` when `src-tauri/` staged) | Full-tree Biome + `tsc`; staged secretlint only |
| Code-quality gate (manual) | `./scripts/dev check` | Biome + TypeScript + full secretlint + Rust |
| lint-staged | **Not used** | Avoid split between staged lint and full gate |

Bypass for one commit: `git commit --no-verify`. Do not disable hooks globally.

### 3.4 Commit message convention

[Conventional Commits 1.0.0](https://www.conventionalcommits.org/). Load the installed `conventional-commits` skill before writing a message.

---

## 4. Configuration Files Reference

| File | Role |
| ---- | ---- |
| `package.json` | Dependencies, `packageManager`, scripts (`tauri`, `dev`, `build`) |
| `pnpm-lock.yaml` | Locked dependency tree |
| `vite.config.ts` | Vite, React plugin, Tailwind, Router plugin, Vitest, Tauri integration |
| `playwright.config.ts` | Playwright smoke — webServer, Chromium project |
| `e2e/` | Playwright specs (`smoke.spec.ts`, `tauri-ipc-stub.ts`) |
| `.env.example` | Optional `VITE_*` variables |
| `tsconfig.json` / `tsconfig.app.json` | TypeScript strict options, path aliases |
| `biome.json` | Lint and format (Biome only — no ESLint/Prettier) |
| `.secretlintrc.json` | secretlint rules (`@secretlint/secretlint-rule-preset-recommend`) |
| `.secretlintignore` | secretlint allowlist (`pnpm-lock.yaml`, `src-tauri/Cargo.lock`) |
| `.editorconfig` | LF + 2-space default; 4-space for `*.rs` (rustfmt) |
| `.husky/pre-commit` | Runs `check-fast`; runs `check-rust` when staged paths include `src-tauri/` |
| `index.html` | Vite HTML entry |
| `src-tauri/tauri.conf.json` | Product identity, CSP, build hooks |
| `src-tauri/capabilities/default.json` | Main-window capability ACL |
| `src-tauri/Cargo.toml` | Rust crate dependencies and features |
| `.github/workflows/check.yml` | CI — selects frontend, browser, Rust, and bridge checks from changed paths; required `check` aggregates applicable results |
| `.github/workflows/release.yml` | Tag-triggered multi-OS installer build via `tauri-action` |
| `scripts/dev` | Stable `test` / `check` / `check-app` / `bridge-test` / `format` / `secrets` / `smoke` / `mutate` surface |
| `.codex/config.toml` | Context7 MCP and shell-environment configuration |
| `.vscode/extensions.json` | Recommended Biome, rust-analyzer, Even Better TOML |
| `.vscode/settings.json` | Format on save (Biome / rust-analyzer); rust-analyzer linked to `src-tauri` |
| `.gitignore` | Build, test, and tool artifacts; `.tanstack/` cache; `.env.*` except `.env.example`; `src-tauri/target/`; editor noise (`.idea/`, vim swap) |
| `.gitattributes` | LF for text sources; binary for images including `.icns` / `.ico` |

---

## 5. Data Flow

Examples use the scaffold **health** demo feature. Forked apps follow the same patterns with real commands and services.

### 5.0 App shell (layout chrome)

```text
AppShellLayout (all routes via __root)
  → AppNavRail — Dashboard + Search + Planner; railExpanded persisted in useLayoutStore (localStorage)
  → AppTopBar — GlobalPlayerSearch (Ctrl+K / Meta+K), ActiveSaveSelect, SnapshotFreshnessChip,
                Load Data cap toggle/limit, Load Data + LoadDataOutcome banner
  → Main content — route Outlet (Dashboard, /search, …)
  → Skip link to #main-content on first Tab
```

Presentation formatters (`formatRelativeAge`, `formatAbsoluteUtc`, `formatCount`, `formatMissable`) live in `src/utils/format.ts` per [DESIGN.md](./DESIGN.md).

### 5.1 Read path (IPC + SQLite)

```text
User navigates to /
  → TanStack Router matches route in app/routes/, runs loader
  → loader: queryClient.ensureQueryData(healthQueryOptions)
  → Query fetcher: invokeCommand("get_status") via lib/tauri-client
  → Rust: features/health/commands.rs → returns HealthStatus DTO
  → Route component: useSuspenseQuery(healthQueryOptions)
  → Feature component renders status; Tailwind + DESIGN tokens style UI

Demo value persistence:
  → invokeCommand("get_demo_value") / invokeCommand("set_demo_value", { value })
  → commands.rs → service.rs → rusqlite query on demo_value table
  → SQLite file under app_data_dir
```

### 5.2 Write path

```text
User clicks Save
  → useMutation(setDemoValue)
  → invokeCommand("set_demo_value", { value })
  → Rust service validates input, writes with parameterized rusqlite statement
  → onSuccess: queryClient.setQueryData or invalidateQueries
  → UI updates from Query cache
  → onError: show error state (inline or toast — per DESIGN.md)
```

### 5.3 Database bootstrap

```text
App startup (lib.rs setup):
  1. db::resolve_db_path joins APP_DB_FILE on app_data_dir
  2. db::open creates parent dirs, opens rusqlite Connection
  3. migrations::apply runs pending versions via PRAGMA user_version
  4. app.manage(Db(Mutex<Connection>)) for IPC commands
```

Migrations apply on open — there is no separate plugin preload step.

If a mutation must clear more than one cache key, document the invalidation map in the feature ledger.

### 5.4 Memory read path (FM26 bridge status)

```text
User opens home route
  → BridgeStatusPanel: useSuspenseQuery(bridgeStatusQueryOptions)
  → invokeCommand("get_bridge_status")
  → Rust memory_read: resolve %LOCALAPPDATA%\fm-valuescout\fm-bridge\, parse status.json
  → Panel shows ready / missing / error / unsupported platform

Dump contract: bridge/DUMP_SCHEMA.md schema v5 (frozen). Scan writes dump.json on disk; ingest reads it in Rust (§5.5).
```

### 5.5 Load Data and snapshot ingest

**Load Data** is one user action: bridge scan, then SQLite ingest for the **active app save**. The dump body never crosses IPC.

```text
User clicks Load Data (AppTopBar)
  → useLoadData mutation → invokeCommand("load_data", { maxAccepted })
      maxAccepted omitted or null = unlimited (production default)
      positive integer = diagnostic cap (UI toggle via useLoadDataPreferences)
  → Rust snapshot/commands load_data:
      Brief Db lock → active_save_id (target save for this load; released before scan)
      memory_read::request_player_dump — no Db lock during scan:
        writes request.json (30s TTL), polls status.json until terminal (120s default)
        Bridge plugin (off Unity main thread): block heap scan → dump.json + diagnostics.txt
  → On scan failure: LoadDataError { phase: "scan", kind, message }; prior snapshot unchanged
  → On scan success: lock Db → load_data_after_scan → ingest::ingest_dump_file_for_save(save_id, dump path)
      validate_dump_json (memory_read::dump_validation) — hard-fail before insert
      Transaction: insert new snapshot + players + player_role_scores, promote to current, delete prior current
      On ingest failure: roll back; prior current snapshot remains
  → Returns LoadDataResult { requestId, playersFound, scanTruncated, maxAccepted, snapshot,
      timings: { scanMs, ingestMs, totalMs } }
  → onSettled: invalidate snapshot query keys (current snapshot, sanity list)
  → LoadDataOutcome banner in AppTopBar (aria-live; success shows phase timings; cleared when user switches save)
  → Snapshot panels show ingest outcome (player count, truncated banner when scanTruncated)
```

**Saves model** (migrations v2–v10, `src-tauri/src/db/migrations.rs`):

| Table | Role |
| --- | --- |
| `saves` | App-side game save slots (not FM save files). Exactly one row has `is_active = 1` (partial unique index). Default save created when the DB has none. |
| `snapshots` | One **current** snapshot per save (`is_current = 1`, partial unique index per `save_id`). Stores dump metadata: schema/game/bridge versions, `game_date`, `scan_truncated`, `max_accepted`, `player_count`, `loaded_at_utc`. Snapshot **history** is out of scope — schema allows future rows. |
| `players` | Rows keyed by `(snapshot_id, uid)`. Scalars for list/search foundations; attribute maps and arrays as JSON text columns. `null` in dump JSON means unknown — never coerced to 0 on ingest. |
| `player_role_scores` | Per-player role-fit scores keyed by `(snapshot_id, uid, role_id)` with `phase` (`in_possession` \| `out_of_possession`) and nullable integer `score` (0–100). FK to `players` with `ON DELETE CASCADE`. Index on `(snapshot_id, role_id)` for role-filtered queries. |
| `planner_club_settings` | One optional club-family configuration per save. Stores the explicitly selected primary club and survives current-snapshot replacement. |
| `planner_club_sources` | Primary and attached club sources assigned to Senior, Reserves, or Youth. The legacy optional `team_level` value remains persisted but does not restrict Planner eligibility. Source rows reference the save, not a snapshot. |
| `planner_tactic_lanes` | Eleven ordered, stable lanes per save. Each lane links an IP placement and role to an OOP placement and role, owns a 0–1 IP weight, an optional unique 1–11 importance rank, a preferred-foot rule, and references the save directly. Both role references are validated against the scoring catalog. |
| `planner_strings` | Ordered depth-chart strings for Senior, Reserves, and Youth. Rows reference the save, not a snapshot, and each team keeps at least one string. |
| `planner_assignments` | Save-wide unique player assignments to a tactic lane and string. Rows retain the player UID and last-known name while current snapshot resolution changes. Migration v7 records `manual` or `optimizer` provenance; legacy rows migrate to `manual`. |

**Query and save-management IPC** (`features/snapshot/commands.rs`):

```text
Active save (AppTopBar ActiveSaveSelect)
  → list_saves / set_active_save
  → set_active_save switches which save’s current snapshot queries target

Save switcher panel (home route)
  → create_save / rename_save (create and rename only; switch is in the top bar)

Snapshot freshness (AppTopBar SnapshotFreshnessChip)
  → derives tone from get_current_snapshot age and scanTruncated

Snapshot overview + sanity list
  → get_current_snapshot → active save’s current snapshot metadata (or null)
  → list_sanity_players(limit ≤ 20) → name, ca, club, proofRoleScore (DLP IP — deep_lying_playmaker_ip)

Route loader prefetches saves, current snapshot, and sanity list alongside health/demo queries.

```

**Planner club-family IPC** (`features/planner/commands.rs`):

```text
User opens /planner
  → route loader: ensureQueryData(current snapshot + club family + distinct current-snapshot clubs)
  → no snapshot: show Load Data guidance
  → snapshot present: PlannerClubFamilyPanel reads get_planner_club_family and list_planner_clubs
  → save: invokeCommand("save_planner_club_family", { primaryClub, sources })
  → Rust validates team, team level, name length, and duplicate sources, then replaces only that save's source rows
  → Load Data and active-save changes invalidate planner query keys from AppTopBar
```

**Planner tactic IPC** (`features/planner/commands.rs`):

```text
User opens /planner
  → route loader: ensureQueryData(get_planner_tactic + get_planner_tactic_options)
  → get_planner_tactic creates the default tactic for a save when none exists
  → save_planner_tactic receives the complete 11-lane draft with one IP weight, optional rank, preferred foot, and foot preference per lane
  → Rust rejects incomplete lanes, unknown or phase-incompatible roles, unsupported positions, lane weights outside 0–1, duplicate or out-of-range ranks, and invalid foot rules
  → planner query keys remain save-scoped and are invalidated with the rest of the planner tree on save/snapshot changes
```

**Planner depth IPC** (`features/planner/commands.rs`):

```text
User opens /planner
  → route loader: ensureQueryData(get_planner_depth)
  → get_planner_depth returns all three ordered team strings with current assignment state and combined score
  → React renders one selected team at a time over the depth read model; tabs change presentation state only
  → get_planner_slot_candidates(team, laneId, search) returns Rust-ranked candidates from that team's configured sources
  → add_planner_string, remove_planner_string, clear_planner_assignment, assign_planner_player, and move_planner_player validate and mutate in Rust
  → optimize_planner_depth returns reconciled depth after transactional, ordered allocation; clear_planner_team requires confirmation and returns reconciled depth after clearing only the selected team
  → player UIDs are unique per save; each team retains at least one string; populated-string removal is confirmed in React and still validated in Rust
  → successful depth mutations reconcile the depth cache and invalidate candidate queries; tactic saves invalidate both
  → Load Data, active-save changes, and club-family saves invalidate the Planner query tree
```

`request_player_dump` remains registered for tests and low-level scan-only use; the **Load Data** button in `AppTopBar` calls `load_data`.

### 5.6 Role scoring on ingest

Role scores are computed in Rust during snapshot ingest — not in the WebView and not as a separate post-ingest job.

```text
ingest transaction (after insert_players):
  → scoring::catalog::all_roles() — 68 static FM26 IP/OOP roles (SortItOutSI Key/Preferred; dump PascalCase keys)
  → for each player: parse attributes_json → score_role per role
      within-band equal means → 0.75 × primary + 0.25 × secondary (primary-only when no secondary list)
      scale / 20 × 100 → rounded integer 0–100
      any null required attribute → null score
  → INSERT player_role_scores (one row per role × player)

Pure helpers (no IPC yet):
  → combine_role_scores(ip, oop, ip_weight) — default 0.5; null if either input null or weight ∉ [0, 1]

Sanity proof:
  → list_sanity_players LEFT JOINs role_id deep_lying_playmaker_ip as proofRoleScore
  → React sanity table column "DLP IP"
```

Position suitability does not enter role scores. Planner tactic lanes persist the caller-supplied combined IP/OOP weights. Ponytail in `ingest.rs`: upgrade to lazy/on-demand or batched scoring if ingest scoring time dominates Load Data. Full-matrix 184k-player ingest test is `#[ignore]`; gate keeps a 2k scored ingest timing check.

### 5.7 Player search

Search reads the **active save's current snapshot** only. The WebView never opens SQLite; all filtering, sorting, and pagination run in Rust.

```text
User opens Search (nav rail or /search)
  → Route loader: ensureQueryData(current snapshot + search_players first page)
  → validateSearch normalizes sort, dir, filters[], combine in URL search params
  → SearchFilterBar — compact strip + SearchFilterEditorModal (shared Modal primitive)
  → SearchResultsPanel — TanStack Virtual table; useQueries fetches 50-row windows (offset/limit)
      as the virtualizer scrolls; total match count from IPC for scrollbar extent
  → Whole-row click or Enter on a focused row navigates to /players/$uid;
      Arrow Up/Down move row focus within the virtualized list

search_players IPC (features/search/commands.rs)
  → offset (default 0), limit (default 50, max 200), sortBy, sortDir
  → optional filters[] + filterCombine ("and" | "or") — max 32 rules
  → filter.rs: validate field/op/value per field kind; compile FilterAst to parameterized WHERE
  → query.rs: SELECT basic columns + dynamicValues for active non-basic filter fields;
      role.* filters use EXISTS on player_role_scores; attr./hidden./personality.* use json_extract;
      nationality uses json_each; position / pos.* for presence and suitability
  → Returns { players[], total }

suggest_players IPC
  → query string (trimmed; blank → []), optional limit (default 10, max 20)
  → Rank: exact name → prefix → contains (COLLATE NOCASE), then CA desc; escape_like on patterns
  → Returns { uid, name, ca }[]

GlobalPlayerSearch (AppTopBar, all routes)
  → Ctrl+K / Meta+K focus; 200ms debounce → suggest_players
  → Combobox + listbox; Escape clears input before closing
  → Selecting a hit navigates to /players/$uid

Cache invalidation: Load Data and set_active_save invalidate snapshot + search + player query keys (`playerKeys.all`)
```

**Invariants:** `null` dump values never coerce to 0 for filter or display; role scores come from `player_role_scores` (not recomputed in the WebView). Basic columns are always shown; dynamic columns follow active non-basic filters (`position` presence excluded). Default sort CA descending.

Truncated-scan warning: `SnapshotFreshnessChip` in the top bar reflects `scanTruncated`; Search results count line does not yet append a cap annotation — see [player-search](./features/completed/player-search.md) follow-up.

### 5.8 Player profile read path

Profile reads the **active save's current snapshot** only. The WebView never opens SQLite; role scores are not recomputed in the WebView.

```text
User opens /players/$uid (from Search row, Enter on focused row, or GlobalPlayerSearch hit)
  → Route loader: ensureQueryData(current snapshot + get_player)
  → validateSearch normalizes tab (overview | attributes | roles) in URL search params
  → Suspense fallback: tab-shaped loading skeletons (Overview grid, Attributes sections, Roles families)
  → PlayerProfileTabs segmented control; one panel visible per tab

get_player IPC (features/player/commands.rs)
  → uid from route param
  → query.rs: SELECT player scalars + JSON attribute maps for current snapshot;
      SELECT role_id, score from player_role_scores; merge with in-process all_roles()
      (displayName, phase, positionTags) in catalog order; missing DB score → null;
      missing player row → null response (not-found empty state)
  → Returns PlayerDetailDto (identity, attributes, hidden, personality, roleScores[])

Overview tab
  → identity block + hero ScoreBadge for best non-null role (catalog-order ties)
  → preferredFoot title-cased for display

Attributes tab
  → static attribute-groups.ts membership (Technical / Mental / Physical / Goalkeeping, Hidden, Personality)
  → null → —

Roles tab
  → position-families.ts groups all 68 roles by pitch family; every role shown
  → card ScoreBadge per role; rolePhaseLabel maps in_possession/out_of_possession → IP/OOP

Cache invalidation: Load Data and set_active_save invalidate snapshot + search + player query keys
```

**Invariants:** `null` dump/DB values never display as `0`. One scoring model shared with Search. No cross-feature component imports — routes compose; Search/GlobalPlayerSearch navigate by route path only.

### 5.9 Bridge plugin install path

```text
User opens home route
  → BridgePluginInstallSection: useSuspenseQuery(bridgeInstallStatusQueryOptions)
  → invokeCommand("get_bridge_install_status")
  → Rust memory_read/install.rs: resolve Steam BepInEx/plugins path, check FmDataBridge.dll presence

User clicks Install / Update plugin
  → useMutation → invokeCommand("install_bridge_plugin")
  → Rust copies bundled src-tauri/resources/FmDataBridge.dll → plugins/
  → User restarts FM so BepInEx loads the DLL

User clicks Remove plugin
  → useMutation → invokeCommand("remove_bridge_plugin")
  → Rust deletes only FmDataBridge.dll (never BepInEx core or other plugins)

Path resolution: FM_BRIDGE_PLUGINS → FM_STEAM_ROOT/BepInEx/plugins → default Windows Steam path
(same order as ./scripts/dev bridge-install). Developer build-and-copy from source stays on bridge-install.
```

Non-Windows hosts return `unsupportedPlatform` for bridge install commands. Full FM attach tests are manual on Windows. CI runs Rust, frontend, browser, and bridge checks only when their source paths or CI configuration change.

---

## 6. Testing Strategy

### 6.1 How tests are organised

- **Component and hook tests** — colocated `*.test.tsx` or `*.test.ts` beside source; Vitest + jsdom.
- **Integration tests** — feature flows under `features/<feature>/` or `app/routes/`; preferred over shallow unit tests for confidence.
- **IPC mocks** — `mockIPC` in `src/testing/setup.ts`; prefer over ad-hoc invoke stubs.
- **E2E / smoke** — Playwright in `e2e/` with `tauri-ipc-stub.ts`; `./scripts/dev smoke` runs walking-skeleton checks. Vitest excludes `e2e/**`.
- **Rust unit tests** — `#[cfg(test)]` modules in `src-tauri/src/`; run via `cargo test` in the gate.
- **Bridge unit tests** — `bridge/Tests/` run through `./scripts/dev bridge-test` in Windows CI.

### 6.2 What each layer covers

- **Presentational components** — RTL queries by role/label; user-visible outcomes.
- **Hooks and stores** — Vitest with minimal mocks; test Zustand store actions and selectors.
- **Query logic** — test query options and IPC fetchers; `mockIPC` when integration matters.
- **Routes** — smoke critical navigation in component tests and Playwright; avoid testing framework router internals.
- **Rust services** — unit tests against temp SQLite files with migrations applied.

### 6.3 Test quality guidelines

Test behaviour the user sees, not implementation details. Do not assert on Zustand or Query internal cache shape unless the contract is the subject. Follow the testing reference in the installed `coding-standards` skill.

### 6.4 Playwright smoke scope

`./scripts/dev smoke` runs Playwright against the **Vite dev server** in Chromium, not `pnpm tauri dev`. `e2e/tauri-ipc-stub.ts` injects `window.__TAURI_INTERNALS__` before the app loads so IPC calls never reach Rust. Demo value "persistence" in smoke is **in-page JavaScript memory** in the stub — not SQLite.

| Playwright smoke covers | Playwright smoke does not cover |
| --- | --- |
| Vite shell loads; TanStack Router renders home, 404, and layout chrome | Real Tauri WebView runtime or platform WebView differences |
| Walking-skeleton UI with stubbed IPC: app shell (nav rail with Search and Planner, top bar with global search), status panels, demo-value form flow, Search route, and Planner no-snapshot, first-use, tactic, and three-team string-add paths | Real `#[tauri::command]` handlers in Rust |
| User-visible navigation and form interaction in Chromium | SQLite persistence, migrations, or `app_data_dir` file I/O |
| Stub IPC for `get_status`, `get_demo_value`, `set_demo_value`, `get_bridge_status`, `get_bridge_install_status`, `install_bridge_plugin`, `remove_bridge_plugin`, `request_player_dump`, `list_saves`, `create_save`, `rename_save`, `set_active_save`, `get_current_snapshot`, `list_sanity_players`, `search_players`, `suggest_players`, `get_player`, Planner club-family and tactic commands, `get_planner_depth`, `add_planner_string`, `remove_planner_string`, `optimize_planner_depth`, and `load_data` (sanity rows include `proofRoleScore`) | Capabilities ACL, plugin permissions, or menu/tray integration |
| Bridge panel, save switcher, snapshot overview, plugin install section, top-bar save selector, and Load Data button render with stubbed IPC | Real BepInEx plugin, FM attach, LocalAppData file protocol, SQLite ingest, or Steam-folder DLL install |

| Concern | Owner in this template |
| --- | --- |
| Frontend IPC wiring and React UI around commands | Vitest + `mockIPC` (`./scripts/dev test`) |
| Command validation, services, migrations, SQLite | `cargo test` in `./scripts/dev check` |
| Bridge scan, dump writers, file protocol | `./scripts/dev bridge-test` in Windows CI (fakes; no FM attach) |
| Full-stack manual verification | `pnpm tauri dev` |
| Automated real WebView e2e | Deferred — see [BACKLOG.md](./BACKLOG.md) (tauri-driver) |

Green smoke does **not** prove SQLite persistence works in production. Rust unit tests own the database; smoke owns browser UI with a stub.

---

## 7. Deployable Artifacts

- **Development** — Install Node 24, pnpm, and the Rust toolchain, then `pnpm install`, `pnpm exec playwright install chromium` (once), then `pnpm tauri dev`. On Linux/WSL, install WebKitGTK and related system packages (see §11). WSLg or an X server is required for the native window on WSL.
- **Production build** — `pnpm tauri build` produces OS-specific installers (`.deb`, `.msi`, `.dmg`, etc.) in `src-tauri/target/release/bundle/`. CI builds unsigned installers on `v*` tag push via `.github/workflows/release.yml`.
- **WebView bundle only** — `pnpm build` produces static files in `dist/` for frontend-only checks; this is not the shipped desktop artifact.
- **Source maps** — default `build.sourcemap: "hidden"` for plain Vite builds (maps on disk, not linked from the public bundle). Tauri production builds use platform-conditional settings when `TAURI_ENV_PLATFORM` is set.
- **Signing** — not configured in the template. Unsigned installers trigger OS security warnings on first run. Add platform signing secrets before shipping a real product.
- **Network / telemetry** — No telemetry in the template. Forks choose online-only or offline-first per product.

---

## 8. Lint & Architecture Enforcement Matrix

### 8.1 TypeScript / React

| Rule | Mechanism | Enforcement |
| ---- | --------- | ----------- |
| Type errors | `tsc -b` | Hard error in `./scripts/dev check` |
| Lint and format | Biome | Hard error in `./scripts/dev check` |
| No IPC data in Zustand | Convention + reviewer | Manual — not lint-enforced |
| Query for async IPC results | Convention + reviewer | Manual |
| Import alias `@/` | `tsconfig` paths | Hard error if path wrong |
| kebab-case filenames | Biome `useFilenamingConvention` when configured | Hard error or reviewer |
| No cross-feature imports | Biome `noRestrictedImports` + reviewer | Hard error in `src/features/**` |
| Unidirectional imports | Biome `noRestrictedImports` + reviewer | Hard error in shared folders |
| No barrel re-exports | Convention + reviewer | Manual |
| Sole invoke wrapper | Convention + reviewer | Manual — only `lib/tauri-client.ts` imports `invoke` |

### 8.2 Rust

| Rule | Mechanism | Enforcement |
| ---- | --------- | ----------- |
| Format | `cargo fmt --check` | Hard error in `./scripts/dev check` |
| Lint | `cargo clippy -D warnings` | Hard error in `./scripts/dev check` |
| Unit tests | `cargo test` | Hard error in `./scripts/dev check` |
| Commands in feature modules | Convention + reviewer | Manual |
| Parameterized queries | Convention + reviewer | Manual — no string-concat SQL |

**Known tooling gaps:** Layer boundaries (Query vs Zustand vs URL state) are convention-only — not lint-enforced. Full `jsx-a11y` lint is not in the default gate — use `ui-design` skill and review; add ESLint only if a product requirement needs plugin coverage Biome lacks.

---

## 9. Notable Trade-offs and Decisions

Each item links to an ADR with alternatives and consequences.

| Decision | ADR |
| --- | --- |
| React for UI | [0001](./decisions/0001-react-for-ui.md) |
| TypeScript | [0002](./decisions/0002-typescript.md) |
| Vite SPA | [0003](./decisions/0003-vite-spa.md) |
| TanStack Router | [0004](./decisions/0004-tanstack-router.md) |
| TanStack Query | [0005](./decisions/0005-tanstack-query.md) |
| Zustand for client state | [0006](./decisions/0006-zustand-client-state.md) |
| Tailwind CSS v4 | [0007](./decisions/0007-tailwind-css-v4.md) |
| Vitest and RTL | [0008](./decisions/0008-vitest-and-rtl.md) |
| Biome | [0009](./decisions/0009-biome.md) |
| pnpm | [0010](./decisions/0010-pnpm.md) |
| Husky (no lint-staged) | [0011](./decisions/0011-husky-git-hooks.md) |
| Secretlint in check | [0012](./decisions/0012-secretlint.md) |
| Tauri v2 desktop shell | [0013](./decisions/0013-tauri-v2-desktop-shell.md) |
| Rust backend and IPC boundary | [0014](./decisions/0014-rust-backend-ipc-boundary.md) |
| SQLite (Rust-owned) | [0015](./decisions/0015-sqlite-rust-owned.md) |
| C# BepInEx FM26 bridge | [0016](./decisions/0016-csharp-bepinex-fm26-bridge.md) |

**@tanstack/react-virtual** is in the stack for the player search results table. TanStack Table, Form, and TanStack Start remain intentionally **not** in the default stack — add per feature when needed. The FM26 bridge is implemented per [ADR-0016](./decisions/0016-csharp-bepinex-fm26-bridge.md), [fm26-memory-read](./features/completed/fm26-memory-read.md), and [bridge-plugin-install](./features/completed/bridge-plugin-install.md); dump schema v5 is frozen in `bridge/DUMP_SCHEMA.md`.

---

## 10. Where to Look Next

- **Add a feature:** Create `src/features/<feature>/` and `src-tauri/src/features/<feature>/` with the subfolders each side needs. Register commands in `lib.rs` via `.invoke_handler(tauri::generate_handler![...])`. Add route wiring in `src/app/routes/`.
- **Add a page:** Create a file under `src/app/routes/`, add Query options in `features/<feature>/api/` if the page loads IPC data.
- **Add client UI state:** Global store in `src/stores/`; feature-scoped store in `features/<feature>/stores/`. Do not store IPC responses in Zustand.
- **Add shared UI:** `src/components/ui/` for primitives (see [DESIGN.md](./DESIGN.md) component specs); wrap third-party components there.
- **Change visual language:** update [DESIGN.md](./DESIGN.md) first, then mirror tokens in `src/styles/global.css` `@theme`.
- **Add persistence:** Migration in `db/migrations.rs`, service in `features/<feature>/service.rs`, commands in `commands.rs`. Open path stays `app_data_dir` + `APP_DB_FILE` via `db::open`.
- **Change stack defaults:** Read ADRs, update decisions, then reconcile this file and scaffold configs.
- **Coding standards detail:** load the installed `coding-standards` skill and its React, Tauri, Rust, and Vite references

---

## 11. Operational Notes

### Prerequisites (WSL Ubuntu or any Linux/macOS dev machine)

Install on the **host OS** before `pnpm install`:

| Tool | Why |
| --- | --- |
| **Node.js 24** | Runs Vite, Vitest, Biome, and all build tooling |
| **pnpm** | Package manager for this template (`corepack enable` after Node, or `npm install -g pnpm`) |
| **Rust toolchain** | `rustc` and `cargo` for the Tauri backend — install via [rustup](https://rustup.rs/) |
| **git** | Version control |

On **Linux and WSL**, install Tauri system dependencies before `pnpm tauri dev` or `pnpm tauri build`:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
```

On **WSL**, you also need a display server for the native window:

- **WSLg** (Windows 11) — GUI apps work out of the box when WSLg is enabled.
- **X server** (older setups) — run an X server on Windows and set `DISPLAY` before `pnpm tauri dev`.

Headless gate commands (`./scripts/dev check`, `cargo test`) do not require a display.

**FM26 bridge:** Build and install the plugin on a **Windows** host with .NET 6 SDK and BepInEx 6 IL2CPP on the Steam FM26 folder. End users can install the bundled DLL from the app (**Bridge plugin install** on the home screen); developers use `./scripts/dev bridge-install` or manual copy from WSL when `FM_STEAM_ROOT` or the default Windows Steam path is set. See [bridge/README.md](../bridge/README.md).

### What `pnpm install` does

`pnpm install` reads `package.json` and `pnpm-lock.yaml` and downloads **Node packages** into `node_modules`. It also runs the Husky `prepare` script to install Git hooks. It does not install Node itself, pnpm itself, the Rust toolchain, or system libraries.

### Typical first-time setup on clean WSL

```bash
# Install Node 24 (example: nvm)
nvm install 24
nvm use 24
corepack enable
corepack prepare pnpm@latest --activate

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Install Tauri Linux dependencies (see apt list above)

# Clone template, then in repo root
pnpm install
pnpm exec playwright install chromium
./scripts/dev check
./scripts/dev test
pnpm tauri dev
```

Husky runs `./scripts/dev check-fast` on every commit (and `check-rust` when `src-tauri/` is staged). Run `./scripts/dev check` before merge — CI selects the applicable product suites.

### CI parity

GitHub Actions selects product checks from changed paths. Frontend changes run `./scripts/dev check-app` and `./scripts/dev test`, then browser smoke. Rust changes install the Rust toolchain and Tauri Linux dependencies before `./scripts/dev check-rust`. Bridge changes run `./scripts/dev bridge-test` on Windows. The required `check` status aggregates every applicable job. Match local Node major version for fewer surprises.

Release builds run on `v*` tag push via `.github/workflows/release.yml` — Windows, Ubuntu, and both macOS architectures. Installers are unsigned draft assets until signing secrets are configured.
