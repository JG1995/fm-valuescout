# Vite — Tauri desktop build, dev server, and Vitest

Load this file when you implement or review `vite.config.ts`, `index.html`, TypeScript project config for the app, Vitest setup, or `package.json` scripts that invoke Vite or Tauri.

Read `references/universal.md` first. Application patterns live in `references/react.md`. IPC and Tauri CLI live in `references/tauri.md`. Test policy lives in `references/testing.md`. Pipeline overview lives in `.wiki/ARCHITECTURE.md` §3. Security audit depth for the Vite toolchain lives in `security-audit/references/vite.md` — load via `security-audit/SKILL.md` when auditing builds, env exposure, or dev-server bind.

This document is the **baseline** for toolchain code in this template. It adapts Vite official guidance for **Tauri v2 + Vite 8 + React + TanStack Router + Vitest**. Vite bundles the WebView frontend; Tauri wraps it in a native shell and owns the Rust backend in `src-tauri/`.

Forks may extend via `references/project.md`. This template targets a **Tauri WebView client** — not SSR, meta-frameworks, or static hosting.

## Toolchain defaults

| Tool | Role |
| --- | --- |
| Vite 8 | WebView frontend bundle (dev server + production build) |
| `@tauri-apps/cli` | `pnpm tauri dev` / `build` — native shell + Rust backend |
| `@vitejs/plugin-react` | React Fast Refresh, JSX |
| `@tailwindcss/vite` | Tailwind CSS v4 |
| `@tanstack/router-plugin` | File routes, `routeTree.gen.ts` |
| Vitest | Unit and component tests (config in Vite) |
| Biome | Lint and format (`biome.json` — not Vite) |
| `tsc --noEmit` | Typecheck without emit |
| pnpm | Package manager (pinned in `packageManager`) |
| Node 24 | Runtime for dev, build, test, and CI |

## How Vite fits this template

Vite bundles the **thin frontend** consumed inside Tauri's WebView. Heavy work runs in Rust — not in Vite's module graph.

```text
index.html (repo root)     → Vite entry; loads src/main.tsx
src/                       → React UI (processed, tree-shaken, hashed)
src-tauri/                 → Rust backend + tauri.conf.json (not processed by Vite)
vite.config.ts             → Frontend dev/build/test contract + Tauri integration
dist/                      → Vite production output → consumed by Tauri bundle
```

**Dev loops:**

| Command | When |
| --- | --- |
| `pnpm tauri dev` | Default — WebView + Rust IPC, real backend |
| `pnpm dev` | Frontend-only iteration; IPC calls fail unless stubbed (tests, Playwright smoke) |

**Dev vs production:** The dev server serves native ESM with fast HMR. Production uses Rolldown to bundle the WebView assets; Tauri packages them into OS installers. Run `pnpm tauri build` (or `pnpm build` + Tauri bundle step) before release — not only `vite preview` on a hosted SPA.

**Boundary rules:**

| Bucket | Put here | Vite behavior |
| --- | --- | --- |
| App code | `src/` | Transpiled, bundled, tree-shaken |
| Imported assets | `src/assets/` or next to components | Hashed URLs, in module graph |
| Fixed-path static files | `public/` | Copied to `dist/` root, no processing |
| Env secrets | Rust, OS keychain, or `.env` without `VITE_` | Never in client bundle |
| Client config | `import.meta.env.VITE_*` | Inlined at build time |
| Generated routes | `src/routeTree.gen.ts` | Generated; do not edit |

Keep **toolchain logic in `vite.config.ts`**. Application code should not assume custom build steps that are not documented here or in `ARCHITECTURE.md`.

## Command surface

**User-facing scripts** live in `package.json`. **Gate and CI** use `./scripts/dev` — do not bypass the gate with ad-hoc npm scripts in CI.

| Command | Typical implementation |
| --- | --- |
| `pnpm tauri dev` | Tauri dev — Vite + WebView + Rust (primary dev loop) |
| `pnpm dev` | `vite` — frontend only |
| `pnpm build` | `tsc -b && vite build` (frontend; Tauri build wraps `dist/`) |
| `pnpm tauri build` | Production desktop installers |
| `pnpm preview` | `vite preview` — WebView bundle only, no IPC |
| `pnpm test` | `./scripts/dev test` → Vitest |
| `pnpm check` | `./scripts/dev check` → Biome + `tsc -b` + secretlint + Rust quality gates when scaffolded |

Keep `package.json` scripts thin. Heavy orchestration belongs in `scripts/dev` or small shell wrappers under `scripts/`.

Run **`pnpm tauri build`** before release when validating the full desktop artifact. Use **`pnpm preview`** only to sanity-check the Vite bundle without the Rust shell.

## Tauri + Vite integration

When `src-tauri/` exists, apply Tauri's documented Vite settings in `vite.config.ts` (see [Tauri — Vite](https://v2.tauri.app/start/frontend/vite/)):

| Setting | Purpose |
| --- | --- |
| `clearScreen: false` | Tauri CLI owns the terminal output |
| `server.strictPort: true` | Fixed dev URL for WebView |
| `envPrefix: ['VITE_', 'TAURI_ENV_']` | Expose Tauri platform env to client when needed |
| `server.watch.ignored: ['**/src-tauri/**']` | Avoid rebuild loops when Rust changes |
| Platform-conditional `build.target`, `minify`, `sourcemap` | Match Tauri WebView expectations per OS |

`tauri.conf.json` points `build.beforeDevCommand` / `beforeBuildCommand` at pnpm scripts that run Vite. Keep those scripts thin — orchestration stays in `package.json`, gate stays in `./scripts/dev`.

Do not put Rust sources in Vite's `include` or alias — `src-tauri/` is a separate Cargo crate.

## `vite.config.ts`

### Baseline

- Use **`defineConfig`** from `vite` in **`vite.config.ts`** at the repo root.
- Use **ESM** in the config file. Do not use CommonJS `require`.
- Add `/// <reference types="vitest/config" />` when the `test` block lives in the same file.
- Prefer **TypeScript** for the config so plugin options stay typed.

### Conditional config

When dev and build need different options, export a function. **Desktop default:** omit HTTP API `proxy` — IPC replaces `/api`. Keep proxy only for legacy scaffold removal or forked hybrid apps.

```typescript
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(({ command }) => ({
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  server: {
    strictPort: true,
    host: host || false,
    port: 5173,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  build:
    command === 'serve'
      ? undefined
      : {
          target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
          minify: process.env.TAURI_ENV_DEBUG ? false : 'esbuild',
          sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
        },
}));
```

- `command`: `'serve'` during dev, `'build'` for production WebView bundle.
- `mode`: `'development'`, `'production'`, or custom from `--mode`.

Use **`loadEnv(mode, process.cwd(), '')`** when `.env` values affect config. Prefer `TAURI_ENV_*` for platform-specific build branches injected by the Tauri CLI.

### Path aliases

Use **one alias:** `@` → `src/`. Match `tsconfig` paths.

**Option A (scaffold default):** set `resolve.alias` in Vite and mirror in `tsconfig.app.json`.

**Option B:** add `vite-tsconfig-paths` so Vite reads `tsconfig` paths — avoids duplicate alias config (Bulletproof React uses this).

Do not add `@components`, `@hooks`, and other aliases unless a fork documents why `@/` is insufficient.

### Plugins

Register plugins in this order unless a plugin doc requires otherwise:

1. `@vitejs/plugin-react`
2. `@tailwindcss/vite`
3. `@tanstack/router-plugin` (often last among compile-time plugins)

Use `enforce: 'pre' | 'post'` only when a plugin doc requires a specific order relative to Vite core.

**Template plugin notes:**

| Plugin | Rule |
| --- | --- |
| `@vitejs/plugin-react` | Default for React; avoid anonymous default-export wrappers that break Fast Refresh display names |
| `@tailwindcss/vite` | Tailwind v4; import global CSS from `src/main.tsx` |
| `@tanstack/router-plugin` | `routesDirectory: 'src/app/routes'`, `generatedRouteTree: 'src/routeTree.gen.ts'` |

Do not hand-edit `routeTree.gen.ts`.

### `define` (global constants)

Use `define` for small compile-time constants only. Values must be JSON-serializable or a single identifier.

Do not put secrets in `define`. Do not use `define` to replace `import.meta.env` for values that already live in `.env`.

### TanStack Router plugin

- **`routesDirectory`:** `src/app/routes` per Bulletproof React app layer and `ARCHITECTURE.md`.
- **`generatedRouteTree`:** `src/routeTree.gen.ts` (or the path documented in architecture).
- Regenerate via dev server or build. Never edit the generated file.

## Dev server

### Defaults

- Port `5173` with `strictPort: true` when Tauri is wired — WebView URL must stay stable.
- Use `TAURI_DEV_HOST` when developing against a remote or WSL-hosted dev server (see Tauri docs).

### API proxy (legacy / fork only)

Do not add `/api` proxy for the desktop-only template. The walking skeleton HTTP demo is removed when IPC lands. Forks that keep a remote HTTP API document proxy in `vite.config.ts` and use a separate client — not alongside IPC for the same feature.

### Preview server

`vite preview` serves `dist/` without Tauri or IPC. Use for frontend bundle checks only. Full app validation: `pnpm tauri dev` or packaged build.

## Assets — `public/` vs `src/`

| Need | Location | Example |
| --- | --- | --- |
| Favicon, optional fixed-path assets | `public/` | `/favicon.ico` |
| Image or font imported in a component | `src/assets/` or colocated | `import logo from './logo.svg'` |
| Asset referenced by absolute path in HTML | `public/` | `<link href="/manifest.json">` |

**Do not** put `index.html` in `public/`. Vite expects `index.html` at the **repo root**.

**Do not** import application TypeScript from `public/`. `public/` is not part of the module graph.

**Do not** put large application bundles in `public/` to skip bundling. You lose hashing, tree-shaking, and cache busting.

## `import.meta` features

Use Vite-native imports instead of webpack-era patterns.

### Dynamic module collections

Prefer **`import.meta.glob`** over barrel `index.ts` files when you need many modules (icons, locale files, feature flags):

```typescript
const icons = import.meta.glob('./icons/*.svg', { eager: true, import: 'default' });
```

Lazy glob imports split into separate chunks at build time. Eager glob loads everything in one chunk — use only when the set is small.

### Asset query parameters

| Query | Use |
| --- | --- |
| default import | Resolved URL (dev path or hashed build URL) |
| `?url` | Explicit URL string |
| `?raw` | File contents as string (SVG, shader, markdown) |
| `?worker` | Web Worker bundle |

Preferred worker pattern:

```typescript
const worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
```

### Built-in env flags

```typescript
import.meta.env.MODE; // 'development' | 'production' | custom
import.meta.env.DEV;  // true in dev server
import.meta.env.PROD; // true in production build
import.meta.env.BASE_URL; // from `base` config
```

## Environment variables

### Client exposure

Variables prefixed with **`VITE_`** or listed in `envPrefix` (including **`TAURI_ENV_`** when configured) are exposed to client code via `import.meta.env`. They are inlined at build time.

Never put secrets in `VITE_*` or `TAURI_ENV_*` variables exposed to the WebView.

### File loading order

```
.env                # loaded in all modes
.env.local          # loaded in all modes, gitignored
.env.[mode]         # loaded only in that mode (e.g. .env.development)
.env.[mode].local   # mode-specific local overrides, gitignored
```

### TypeScript

Declare every `VITE_*` variable in `src/vite-env.d.ts`. `TAURI_ENV_*` vars are optional — add when the UI branches on platform at build time.

```typescript
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_APP_TITLE?: string;
  // Add VITE_* as forks need them — not VITE_API_URL for desktop default
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
```

Parse and validate client env in `src/config/env.ts` with Zod when more than one variable exists. Desktop apps rarely need client env beyond display metadata — domain config lives in Rust or SQLite.

### HTML substitution

Vite can replace `%VITE_APP_TITLE%` in `index.html`. Use for static HTML metadata only — not for secrets.

## TypeScript config

- Use **project references**: `tsconfig.json` (solution), `tsconfig.app.json` (app + `src/`), `tsconfig.node.json` (Vite config and Node-side scripts).
- Enable **strict** mode in the app config.
- Mirror the `@/` path alias:

```json
"paths": {
  "@/*": ["./src/*"]
}
```

- `include` should cover `src/`. Do not typecheck `dist/` or `node_modules`.
- Run **`tsc --noEmit`** (or `tsc -b`) in `./scripts/dev check`, not only during `build`, so CI catches type errors without a full bundle.

## Production build

### Output

- Output directory: **`dist/`** (Vite default). Add `dist/` to `.gitignore`.
- Run **`pnpm build`** before release. Verify `dist/` contains hashed assets and `index.html`.

### `base`

| Deploy target | `base` value |
| --- | --- |
| Tauri WebView (template default) | `'/'` — assets load from bundled `dist/` |
| Subpath static hosting (fork) | `'./'` or subpath — not the desktop default |

Wrong `base` breaks asset URLs inside the WebView bundle.

### Tauri production build

`pnpm build` produces `dist/` for the WebView. `pnpm tauri build` runs the frontend build (via `beforeBuildCommand`) and packages installers. CI release workflow uses `tauri-action` — see feature ledger.

`build.sourcemap` may follow `TAURI_ENV_DEBUG` during Tauri builds. For non-Tauri `vite build` alone, `build.sourcemap: 'hidden'` remains a reasonable default.

### `build.target`

Vite defaults target modern browsers. Lower `build.target` only when the product must support older browsers — document the change in an ADR or wiki.

### Code splitting

1. **Route-level `lazy()`** in TanStack Router (see `references/react.md`) — Vite splits at dynamic `import()` automatically.
2. **`import.meta.glob`** without `eager: true` — separate chunks per module.
3. **`manualChunks`** / **`rolldownOptions.output.manualChunks`** — only after bundle analysis shows a problem. Do not add a default `vendor` chunk “because guides say so.”

On Vite 8, prefer **`build.rolldownOptions`** for Rollup-compatible options. Use `rollupOptions` only when your Vite version or plugin still expects that key — check the Vite 8 config docs when upgrading.

### Source maps

Template `vite.config.ts` branches on Tauri platform env:

- **Tauri platform build** (`TAURI_ENV_PLATFORM` set): `sourcemap: Boolean(TAURI_ENV_DEBUG)` — linked maps when debug env is true.
- **Standalone `vite build`** (no platform env): `sourcemap: 'hidden'` — maps exist for tooling but are not referenced from the bundle output.

| Setting | Use |
| --- | --- |
| **`"hidden"`** | Default for non-Tauri `vite build`; reasonable when maps stay private |
| **`true`** | Full linked maps — upload privately (e.g. Sentry), not to a public CDN |
| **`TAURI_ENV_DEBUG` true in release CI** | Debug maps in shipped desktop bundle — audit release workflows |

Do not ship full linked source maps to a public CDN without a reason.

### Chunk size warnings

Tune `build.chunkSizeWarningLimit` when warnings are noise. Investigate when a single chunk grows unexpectedly after a dependency or route change.

## Vitest

Configure Vitest in the **`test`** block inside `vite.config.ts` unless the file becomes unwieldy — then split to `vitest.config.ts` and document why.

### Defaults

- **`environment: 'jsdom'`** for component tests.
- **`setupFiles`:** `src/testing/setup.ts` — import `@testing-library/jest-dom`; register `mockIPC` per `references/tauri.md` (not MSW for domain data).
- **`globals: false`** — import `describe`, `it`, `expect`, `vi` from `vitest`.
- **`css: true`** when components import CSS or Tailwind-processed classes in tests.
- Reuse Vite **`resolve.alias`** so tests import `@/` like production code.

### Excludes and coverage

```typescript
test: {
  exclude: ['**/node_modules/**', '**/e2e/**'],
  coverage: {
    include: ['src/**'],
    exclude: ['src/routeTree.gen.ts', '**/*.d.ts'],
  },
},
```

IPC mocks live in `src/testing/` or inline in tests via `mockIPC`. Prefer command-level mocks over stubbing `tauri-client` unless testing the wrapper itself — see `references/react.md` and `references/tauri.md`.

Run tests with **`vitest run`** in CI and gate scripts. Use `vitest` (watch) only for local iteration.

### Coverage tooling

Add `@vitest/coverage-v8` when the project needs coverage reports. Wire `vitest run --coverage` through `./scripts/dev test` only when requested — not required for the template skeleton.

## Dependencies and pre-bundling

Vite pre-bundles dependencies with esbuild in dev for faster cold starts.

- Add **`optimizeDeps.include`** when a dependency fails to pre-bundle (common with some CJS packages).
- Add **`optimizeDeps.exclude`** only for known issues — e.g. `fsevents` on optional native deps.

Do not add `optimizeDeps` entries speculatively. Fix when the dev server logs a pre-bundle warning.

Application dependencies: **caret ranges** (`^`) at template publish; lockfile holds exact versions. Runtime IPC (`@tauri-apps/api`) goes in **`dependencies`**. Dev-only tools (`vitest`, `@tauri-apps/cli`, `@vitejs/plugin-react`, etc.) go in **`devDependencies`**.

Pin **`packageManager`** in `package.json`. Set **`engines.node`** to `>=24` (or exact major per team policy).

Add new dependencies only through the minimalism ladder in `.agents/skills/minimalism/SKILL.md`.

## Biome (paired tool)

Biome is the **only** lint and format tool. Do not add ESLint or Prettier unless a fork documents a specific gap Biome cannot cover (for example mandatory corporate ESLint configs or proven `jsx-a11y` plugin needs).

Biome does not run inside Vite. It uses `biome.json` at the repo root.

- Run **`biome check`** in `./scripts/dev check` (full project, not staged files only).
- React hook dependency arrays: enable **`useExhaustiveDependencies`** (Biome's equivalent of `react-hooks/exhaustive-deps`).
- Do not add a separate ESLint pass for hook deps by default.

### Filename and import conventions

| Rule | Target enforcement |
| --- | --- |
| kebab-case source files | Biome `useFilenamingConvention` when available; otherwise reviewer |
| No cross-feature imports | `noRestrictedImports` zones in `biome.json` when scaffold exists |
| Unidirectional imports (shared → features → app) | `noRestrictedImports` + reviewer |
| `@/` alias only | `tsconfig` paths — hard error on bad paths |

Example `noRestrictedImports` zones to add at scaffold (adjust feature names as the app grows):

```json
{
  "linter": {
    "rules": {
      "style": {
        "noRestrictedImports": {
          "level": "error",
          "options": {
            "paths": {
              "src/features/*": "Import across features only in app/routes. Compose features at the route layer."
            }
          }
        }
      }
    }
  }
}
```

Full zone matrix (features cannot import from `app`, shared cannot import from `features` or `app`) lives in `ARCHITECTURE.md` §8. Start with cross-feature restrictions; expand as the codebase grows.

## `scripts/dev` integration

After scaffold:

- **`check`** runs Biome verify (`biome check`), `tsc -b`, secretlint (via `run_secretlint_full_tree`), and Rust quality gates.
- **`format`** runs **`biome check --write`** on the project or forwarded paths, then **`cargo fmt`** in `src-tauri/`. Path args apply to Biome only. Use before `$workflow-build` checkpoint staging — not in CI or Husky.
- **`test`** with no arguments runs **`vitest run`** for the full suite.
- **`test`** with arguments forwards to Vitest (file pattern or `--grep`).

Invoke Vitest through the repository script so forks clone and run one command.

- **`smoke`** runs Playwright (`e2e/smoke.spec.ts`). Requires `pnpm exec playwright install chromium` once after install. CI installs browsers with `--with-deps`.
- **`bridge-test`** runs the C# bridge unit suite. It requires the .NET 6 SDK; CI runs it on Windows.
- **`secrets`** runs `run_secretlint_full_tree` or `run_secretlint_staged` (`--staged` via `git diff --cached` and `secretlint --no-glob`). Same full-tree path as **`check`** after Biome and `tsc`.
- **`mutate`** stays unconfigured until mutation tooling lands (exits 69).

## Git hooks (Husky)

Husky installs Git hooks on `pnpm install` via a `prepare` script at scaffold. **Do not use lint-staged.**

| Piece | Scaffold target |
| --- | --- |
| `husky` | `devDependencies` |
| `package.json` `"prepare"` | `"husky"` (or equivalent init) |
| `.husky/pre-commit` | `./scripts/dev check-fast` (+ `check-rust` when `src-tauri/` staged) |

Pre-commit runs **`check-fast`** — full-tree Biome (`biome check`) and TypeScript (`tsc -b`), plus **staged-only** secretlint (`./scripts/dev secrets --staged`). It does **not** run full-tree secretlint, smoke, or Rust unless `src-tauri/` is staged (`check-rust`). CI and pre-merge validation run the full product suite separately from the full code-quality gate (`./scripts/dev check`). See [ADR-0011](../../../../.wiki/decisions/0011-husky-git-hooks.md) and [ADR-0012](../../../../.wiki/decisions/0012-secretlint.md).

## Scaffold checklist

| File / directory | Purpose |
| --- | --- |
| `vite.config.ts` | Plugins, alias, Vitest, Router plugin, Tauri settings |
| `src-tauri/` | Rust backend, `tauri.conf.json`, capabilities (not Vite-processed) |
| `index.html` | Entry at repo root; script points to `src/main.tsx` |
| `src/vite-env.d.ts` | `ImportMetaEnv` for `VITE_*` vars |
| `tsconfig.json` | Solution references |
| `tsconfig.app.json` | App strict compile, `@/` paths |
| `tsconfig.node.json` | Vite config typing |
| `biome.json` | Lint, format, import zones (Biome only) |
| `.secretlintrc.json` | secretlint preset recommend |
| `.secretlintignore` | secretlint allowlist |
| `.husky/pre-commit` | `check-fast` on commit; full `check` in CI |
| `src/main.tsx` | App entry, global CSS import |
| `src/app/routes/` | TanStack Router file routes |
| `src/app/provider.tsx` | Global providers |
| `src/routeTree.gen.ts` | Generated route tree (do not edit) |
| `src/features/` | Feature modules (presentation + IPC fetchers) |
| `src/components/ui/` | Shared UI primitives |
| `src/lib/tauri-client.ts` | Sole IPC invoke wrapper |
| `src/testing/setup.ts` | Vitest setup, jest-dom, mockIPC |
| `src/testing/mocks/` | IPC test helpers (optional) |
| `src/styles/` | Global CSS, Tailwind `@theme` |
| `public/` | Favicon, optional fixed-path static files |
| `playwright.config.ts` | Playwright smoke — Vite dev server, IPC stub in specs |
| `e2e/` | Playwright specs (excluded from Vitest) |
| `.env.example` | Documented optional `VITE_*` variables |
| `.gitignore` | `dist/`, `src-tauri/target/`, `.tanstack/`, coverage, Playwright artifacts, `.env.*` (except `.env.example`) |

## Anti-patterns

| Do not | Do instead |
| --- | --- |
| Put `index.html` in `public/` | Root `index.html` |
| Import TS/TSX from `public/` | Code in `src/` |
| Large app bundles in `public/` | Bundled imports from `src/` |
| Secrets in `VITE_*` or `define` | Rust env, OS keychain, Tauri secure-storage plugin |
| Skip `tsc` in check; rely only on Vite transpile | `tsc --noEmit` in gate |
| Duplicate alias config without reason | `vite-tsconfig-paths` or documented `@/` sync |
| `manualChunks` before measuring | Route `lazy()` and dynamic import first |
| Barrel `index.ts` for tree-shaking | Direct imports or `import.meta.glob` |
| Dev-only behavior without `import.meta.env.DEV` | Explicit dev checks when needed |
| Skip `vite preview` before release | `pnpm tauri build` or at least `pnpm preview` for WebView bundle |
| Dev with `pnpm dev` expecting IPC | `pnpm tauri dev` for real backend |
| HTTP `/api` proxy in desktop default | IPC commands in Rust |
| MSW for domain data in Vitest | `mockIPC` |
| `src/lib/api-client.ts` for new features | `tauri-client.ts` |
| Webpack or second bundler alongside Vite | Vite only |
| Commit `dist/` or `node_modules/` | Build in CI or locally; ignore artifacts |
| Custom `scripts/dev` bypass in CI | `./scripts/dev check` always |
| Edit `routeTree.gen.ts` | Regenerate via dev server or build |
| Routes under `src/routes/` without plugin update | `src/app/routes/` per architecture |
| `src/test/` for Vitest setup | `src/testing/` per Bulletproof React |
| ESLint + Prettier alongside Biome | Biome only — see ADR-0009 |
| lint-staged on commit | Fast `check-fast` via Husky; full `check` in CI |
| Manual `core.hooksPath` | Husky `prepare` on `pnpm install` |
| Multiple path aliases (`@hooks`, `@utils`, …) | Single `@/` → `src/` |
| `optimizeDeps` entries without a dev warning | Fix when pre-bundle fails |
| Library mode or multi-page config without need | SPA default until fork documents need |

## Related decisions and resources

| Topic | ADR |
| --- | --- |
| Vite SPA (historical) | [0003](../../../../.wiki/decisions/0003-vite-spa.md) — Vite remains; static-hosting deployment no longer applies. Desktop packaging follows these coding-standards refs and a future Tauri ADR. |
| Vitest + RTL | [0008](../../../../.wiki/decisions/0008-vitest-and-rtl.md) |
| Biome | [0009](../../../../.wiki/decisions/0009-biome.md) |
| Husky (no lint-staged) | [0011](../../../../.wiki/decisions/0011-husky-git-hooks.md) |
| Secretlint in check | [0012](../../../../.wiki/decisions/0012-secretlint.md) |
| pnpm + Node 24 | [0010](../../../../.wiki/decisions/0010-pnpm.md) |
| Tailwind v4 | [0007](../../../../.wiki/decisions/0007-tailwind-css-v4.md) |

**External references:**

- [Vite documentation](https://vite.dev/)
- [antfu/skills — vite](https://github.com/antfu/skills/tree/main/skills/vite) — config, `import.meta`, Rolldown migration
- [Tauri — Vite integration](https://v2.tauri.app/start/frontend/vite/)
- Application layout and thin-frontend rules — `references/react.md`
- IPC and capabilities — `references/tauri.md`

Paths from this file: `.agents/skills/coding-standards/references/vite.md` → `.wiki/decisions/`.
