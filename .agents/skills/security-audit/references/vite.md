# Vite — build toolchain and dev-server security audit

Load this file when recon finds `vite.config.ts`, `vitest.config.ts`, `index.html`, `package.json` scripts that invoke Vite, Playwright `webServer` using Vite, or `.env*` files that feed the WebView bundle.

Read `references/universal.md` first. Client bundle exposure (`VITE_*`, XSS sinks) lives in `references/react.md` §Secrets and client env (`VITE_`). This file covers **how the toolchain inlines, serves, and ships** those values. Tauri WebView CSP and `devCsp` live in `references/tauri.md` §Content Security Policy (`tauri.conf.json`).

Adapted from [Vite env and mode docs](https://vite.dev/guide/env-and-mode.html), OWASP configuration guidance, and common CI/build footguns (2025–2026).

## What Vite protects and does not protect

Vite **prevents accidental client exposure** of env vars that lack the configured prefix (`VITE_`, `TAURI_ENV_` in this template). Values without the prefix stay in Node during config evaluation — they are not automatically inlined into `import.meta.env`.

Vite does **not** protect against:

- Secrets placed in `VITE_*` or widened `envPrefix` — they are **public** after `vite build`.
- `define` or `loadEnv` + `define` copying server-only vars into the client bundle.
- Dev server bound to `0.0.0.0` or a shared network host without firewall — anyone on the LAN can load the dev app.
- Shipping full linked source maps or `*.map` beside chunks on a public host.
- `public/` files copied verbatim to `dist/` — no hashing, no access control.
- Malicious or typosquat dependencies in the lockfile — Vite does not audit crates/npm.

## Attack surface

| Layer | What attackers reach | Primary files |
| --- | --- | --- |
| **Client env inline** | Every `import.meta.env.VITE_*` and `TAURI_ENV_*` in the bundle | `.env*`, `vite.config.ts` `envPrefix`, `src/vite-env.d.ts` |
| **Config-time secrets** | Vars read in `vite.config.ts` via `loadEnv` or `process.env` | `vite.config.ts`, CI env |
| **`define` constants** | JSON-serialized globals in every chunk | `vite.config.ts` `define` |
| **Dev server** | Unauthenticated HTTP + HMR on bound host/port | `vite.config.ts` `server`, `TAURI_DEV_HOST`, Playwright `webServer` |
| **Build output** | `dist/` assets, source maps, chunk names | `vite.config.ts` `build`, `dist/` |
| **Static `public/`** | Files at fixed URLs in production | `public/` |
| **HTML injection** | `%VITE_*%` substitution in `index.html` | `index.html` |
| **Dependency graph** | Supply-chain code in dev and bundle | `package.json`, `pnpm-lock.yaml` |

The WebView loads the Vite output — treat `dist/assets/*.js` as **fully attacker-readable** (same as any SPA).

## Recon signals

```text
envPrefix|loadEnv\(|define:\s*\{
import\.meta\.env\.(?!MODE|DEV|PROD|BASE_URL)
%VITE_|%TAURI_
server\.host|strictPort|proxy:|allowedHosts
build\.sourcemap|sourcemap:\s*true
build\.rollupOptions|rolldownOptions|manualChunks
public/|index\.html
webServer.*vite|vite preview
\.env\.(local|production|development)
TAURI_DEV_HOST|TAURI_ENV_DEBUG|TAURI_ENV_PLATFORM
```

When `dist/` or CI artifacts are in scope, grep built JS for `sk_live`, `service_role`, `AKIA`, `ghp_`, `Bearer `, and non-placeholder `VITE_` values.

## Environment variables and prefix gate

### Publication boundary

Only variables matching `envPrefix` reach `import.meta.env` in client code. Template default:

```text
envPrefix: ['VITE_', 'TAURI_ENV_']
```

| Check | Finding |
| --- | --- |
| `envPrefix: ''` or empty prefix | Vite refuses — if bypassed via custom plugin, every loaded env var ships |
| Extra prefix (`PUBLIC_`, `NEXT_PUBLIC_`) added without review | Widens blast radius — one stray admin token in `.env.local` ships silently |
| `TAURI_ENV_*` used for secrets | Same exposure as `VITE_*` — platform flags only |
| Secret-shaped names in `VITE_*` | `VITE_API_SECRET`, `VITE_DB_URL` — naming discipline failure |
| `.env` committed with real values | Git history leak — secretlint may miss non-pattern secrets |
| `.env.local` not gitignored | Local secrets in repo |
| Shell `VITE_*` in CI overrides repo `.env` | Highest precedence — verify CI secret scope |

### Mode and file loading

Vite loads `.env`, `.env.local`, `.env.[mode]`, `.env.[mode].local`. `vite build` defaults to **production** mode; `vite` dev defaults to **development**.

| Risk | Example |
| --- | --- |
| Production build without `--mode` uses wrong file | Staging URL in `.env.development` ships in prod |
| `vite build --mode development` by mistake | Dev API URL in release artifact |
| Missing required `VITE_*` in CI | Build succeeds with `undefined` inlined — silent misconfig |
| No Zod guard in `src/config/env.ts` | Empty string passes; app calls wrong host |

Audit: `.env.example` placeholders only; required vars validated at app startup when forks add client env.

### `loadEnv` in config

`loadEnv(mode, process.cwd(), '')` with empty prefix loads **all** keys into the config file. Safe uses: pick `server.port`, `proxy` target, `build` branches.

| Pattern | Risk |
| --- | --- |
| `define: { __API_KEY__: JSON.stringify(env.API_KEY) }` | Secret inlined into client |
| `define` re-exporting every `loadEnv` key | Accidental publication |
| Config reads `process.env.SECRET` then passes to plugin that emits client code | Plugin-dependent leak |

Trace: every `loadEnv` result → only `server`/`build`/`plugins` — never `define` unless value is public.

## `define` and compile-time constants

`define` replaces identifiers at build time. Values must be JSON-serializable.

- Never put signing keys, DB URLs, or admin tokens in `define`.
- Do not use `define` to fake `import.meta.env` for secrets — use Rust or server-side storage in this stack.
- Audit custom plugins that inject `define` from env without prefix filtering.

## `index.html` and `public/`

### HTML env substitution

Vite replaces `%VITE_APP_TITLE%` in `index.html` at build time. Same exposure rules as `import.meta.env` — only public metadata.

- Do not substitute secrets into `<script>` blocks or meta tags visible in HTML source.
- Third-party `<script src="https://...">` without `integrity` — CDN compromise = full origin access in WebView.

### `public/` directory

Files copy to `dist/` root with **fixed paths** — no content hashing.

| Risk | Example |
| --- | --- |
| `public/config.json` with API keys | `/config.json` in production |
| `public/.env` or backup files | Accidental commit |
| User-upload simulation in `public/` | Static host serves attacker file if name predictable |
| Large sensitive dumps in `public/` for "quick test" | Forgotten before release |

Prefer bundled imports from `src/` for app assets. Use `public/` only for favicon and fixed-path files documented in wiki.

## Dev server exposure

Template `vite.config.ts`:

- `server.port: 5173`, `strictPort: true`
- `host: process.env.TAURI_DEV_HOST || false` — binds LAN only when `TAURI_DEV_HOST` set (WSL/remote dev)
- `watch.ignored: ['**/src-tauri/**']`

| Check | Finding |
| --- | --- |
| `server.host: true` or `'0.0.0.0'` without firewall | LAN-wide dev app — no auth on Vite dev server |
| `server.proxy` to internal URL (fork) | Dev SSRF pivot — proxy reaches metadata services |
| `allowedHosts` too broad | DNS rebinding against dev server |
| Playwright `webServer` with `--host 0.0.0.0` | CI runner exposes dev server to network |
| HMR WebSocket on shared host | Same LAN exposure as HTTP |

`pnpm dev` (frontend-only) does not run Rust — IPC fails unless stubbed. That is functional, not a security boundary; dev server still serves the bundle.

`pnpm tauri dev` uses Vite as WebView source — capabilities and CSP differ from bare `pnpm dev`; audit both configs when dev workflow changes.

## Production build output

### Source maps

Template behavior:

- Tauri platform build: `sourcemap: Boolean(TAURI_ENV_DEBUG)` — maps when debug env set
- Non-Tauri `vite build`: `sourcemap: 'hidden'` — maps exist but not linked from bundle

| Setting | Risk |
| --- | --- |
| `sourcemap: true` in production | Full source in `*.map` beside chunks — route names, internal paths |
| Maps uploaded to public CDN | Same leak without Sentry-style private upload |
| `TAURI_ENV_DEBUG` true in release CI | Debug maps in shipped desktop bundle |

Audit release workflows for `TAURI_ENV_DEBUG`, `DEBUG`, and `NODE_ENV`.

### Chunk names and route leakage

TanStack Router `lazy()` → chunk filenames correlate with routes. Low severity info leak — admin route chunk name reveals feature surface.

`build.rollupOptions.output.manualChunks` naming — avoid `admin-secrets.js`.

### `base` misconfiguration

Wrong `base` breaks assets — functional bug. For forked subpath hosting, wrong `base` can load scripts from attacker-controlled path if combined with open redirect.

Tauri WebView default: `base: '/'`.

## Vitest integration

Vitest shares `vite.config.ts` — same `resolve.alias`, `define`, and env inline rules apply to **test bundles**.

| Risk | Check |
| --- | --- |
| Test-only `define` with secrets | Still in Vitest output if imported |
| `test.env` or `vi.stubEnv` with real keys | Committed or logged |
| Coverage reports in `coverage/` | PII from test fixtures — `.gitignore` and CI artifact scope |

Test infrastructure security depth: `references/testing.md`.

## Playwright + Vite

`playwright.config.ts` runs `pnpm exec vite --host 127.0.0.1 --port 5173` — localhost binding is correct for CI.

Audit forks that change host to `0.0.0.0` or reuse production `dist` without IPC stubs — different exposure class.

## Dependency and lockfile supply chain

| Check | Why |
| --- | --- |
| `pnpm-lock.yaml` present | Reproducible installs |
| CI `pnpm install --frozen-lockfile` | No unexpected dependency drift |
| `pnpm audit` or OSV in CI (fork) | Known CVEs in Vite, React, plugins |
| Postinstall scripts in new deps | Install-time execution |
| Unpinned CDN scripts in `index.html` | Outside lockfile |

Vite plugins run at build time with Node privileges — compromised `@vitejs/plugin-*` affects every build.

## Tauri-specific Vite settings

From template `vite.config.ts` and [Tauri Vite guide](https://v2.tauri.app/start/frontend/vite/):

| Setting | Security note |
| --- | --- |
| `clearScreen: false` | Operational — no security impact |
| `envPrefix` includes `TAURI_ENV_` | Platform flags inlined — not secrets |
| `build.target` per `TAURI_ENV_PLATFORM` | Compatibility — not security |
| `minify` off when `TAURI_ENV_DEBUG` | Larger bundle — easier reverse engineering |
| `watch.ignored: src-tauri` | Prevents rebuild loops — Rust changes not hot-reloaded into client by mistake |

`tauri.conf.json` `beforeDevCommand` / `beforeBuildCommand` invoke pnpm scripts — audit those scripts for env leaks and pre-build network calls.

## Static audit methodology

1. **Read `vite.config.ts`** — `envPrefix`, `define`, `loadEnv`, `server`, `build.sourcemap`, `proxy`.
2. **Trace env files** — which modes, which values, gitignore coverage.
3. **Grep client code** — `import.meta.env`, `%VITE_` in HTML.
4. **Inspect `public/`** — no config dumps or secrets.
5. **One production build** — grep `dist/assets/*.js` for secret patterns and unexpected env inline.
6. **CI workflow** — build-time env injection, debug flags, frozen lockfile; pre-commit uses `check-fast` (full-tree Biome + `tsc -b`; secretlint staged-only) while full `check` runs in CI — see `coding-standards/references/vite.md` §Git hooks and ADR-0011.
7. **Cross-check `react.md`** — bundle XSS and `VITE_*` table; this file owns toolchain path.

Skip server-side-only classes unless fork adds SSR or API proxy in Vite middleware.

## False positives

- `envPrefix: ['VITE_', 'TAURI_ENV_']` with empty `.env.example` — template default.
- `sourcemap: 'hidden'` for standalone `vite build` — reasonable default.
- `strictPort: true` on 5173 — stability for Tauri, not a vulnerability.
- `TAURI_ENV_DEBUG` sourcemaps in local dev only — expected when not in release CI.
- Playwright on `127.0.0.1` — localhost-scoped smoke.
- No `server.proxy` in desktop-only template — no dev proxy SSRF surface.
- Vitest `exclude: e2e/**` — separation of test runners, not a security control.

## Sources

| Source | Use in this file |
| --- | --- |
| [Vite — Env and mode](https://vite.dev/guide/env-and-mode.html) | Prefix gate, mode files, precedence |
| [Vite — `define`](https://vite.dev/config/shared-options.html#define) | Compile-time constant exposure |
| [Tauri — Vite](https://v2.tauri.app/start/frontend/vite/) | Platform env, build targets |
| `references/react.md` | Client `import.meta.env` usage, bundle grep |
| `references/tauri.md` | CSP, `devCsp`, production WebView loading |
