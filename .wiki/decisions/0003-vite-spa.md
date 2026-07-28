# 0003 — Vite SPA

## Status

Accepted (amended 2026-07-27)

## Context

The template default is a client-rendered React application inside a Tauri WebView, not a full-stack framework. We need fast local development, a simple production WebView bundle, and tight integration with Vitest. Solo developers should understand the whole pipeline in one afternoon.

The repository converted from a browser-only SPA starter to a desktop-only Tauri template ([0013](./0013-tauri-v2-desktop-shell.md)). Vite still bundles the UI; deployment as static files to a web host is no longer the template default.

## Decision

Use **Vite 8** as the bundler and dev server. Ship a **single-page application** (SPA) with no server runtime in the template default.

## Alternatives considered

### Next.js (App Router)

Next.js provides SSR, routing, and deployment integrations. It adds framework concepts (RSC, caching rules, server components security surface) that many internal SPAs never need. Next.js 15 maintenance LTS ends October 2026; new work targets Next 16. Valid for forks that need SEO or SSR — not the minimal template default.

### TanStack Start

TanStack Start is the full-stack counterpart to TanStack Router with SSR and server functions. It overlaps with Next.js at a different abstraction level. Choose it when the product needs full-stack TanStack from day one — heavier than a Vite SPA starter.

### Webpack-based CRA lineage

Create React App is unmaintained. Webpack-first setups are slower to configure and slower in dev than Vite for typical React SPAs.

## Consequences

### Positive

- Fast HMR and production builds; Vite 8 actively maintained (npm, July 2026).
- `dist/` WebView bundle integrates with Tauri `beforeBuildCommand` and `tauri-action` installers.
- Vitest shares Vite config — one toolchain for dev, build, and test.
- Plain `pnpm build` still produces a WebView bundle for frontend-only checks; Tauri platform env applies Tauri-specific build options when set.

### Negative

- No built-in SSR or SEO — forks must migrate framework if requirements change.
- Static hosting of `dist/` alone is a fork choice, not the template distribution path.
- Backend work lives in Rust via IPC ([0014](./0014-rust-backend-ipc-boundary.md)), not a Vite server runtime.

### Follow-up

- Done at scaffold (`41effa2`, `fc11b5a`) — `vite.config.ts` with React, Tailwind, Router, and Vitest plugins.
- Done at scaffold — README quick start documents `pnpm dev` and `./scripts/dev` gate; `pnpm build` / `pnpm preview` in [ARCHITECTURE.md](../ARCHITECTURE.md) §3.1.
- Amended 2026-07-27 — Tauri Vite settings (`clearScreen`, `strictPort`, `TAURI_ENV_*`, platform-conditional build) in `58f1683`; static-hosting deployment superseded by desktop installers ([0013](./0013-tauri-v2-desktop-shell.md)).

## Related work

- Desktop shell and installers: [0013](./0013-tauri-v2-desktop-shell.md)
- Commits: `41effa2`, `fc11b5a`, `58f1683`
- Supersedes: static `dist/` deployment as the template default distribution model
