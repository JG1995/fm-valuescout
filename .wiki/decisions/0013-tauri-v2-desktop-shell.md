# 0013 — Tauri v2 desktop shell

## Status

Accepted

## Context

The template started as a browser-only React SPA with static `dist/` deployment. The product direction changed to a **desktop-only** template: native window, OS installers, and no hosted web deployment path. We need a shell that embeds the React UI in a WebView, exposes a secure IPC boundary to Rust, and ships multi-OS installers from CI.

Electron is the common alternative for React desktop apps. Tauri v2 uses the OS WebView, a smaller Rust runtime, and a capability-based security model aligned with deny-by-default plugin permissions.

## Decision

Use **Tauri v2** as the desktop shell. The React app runs inside a Tauri WebView. Rust code lives in `src-tauri/`. The primary dev loop is `pnpm tauri dev`. Production distribution is OS installers built by `tauri-action` on version-tag push (unsigned in the template default).

Vite remains the WebView bundler — see [0003](./0003-vite-spa.md) (amended). Tauri hooks (`beforeDevCommand`, `beforeBuildCommand`) run the Vite dev server and production bundle.

## Alternatives considered

### Electron

Mature ecosystem and familiar to many teams. Larger runtime footprint and heavier default security surface (Node in the renderer unless carefully isolated). Valid for forks that need Electron-specific integrations — not the minimal Rust-native path this template targets.

### Keep browser SPA template only

No native shell, no IPC, no installer pipeline. Rejected — the repository purpose is a Tauri desktop template, not a hosted SPA starter.

### Tauri v1

Superseded by v2 capabilities model, plugin APIs, and documentation. v2 is the current stable line (July 2026).

### Dual web + desktop mode

Ship HTTP alongside IPC so one codebase targets browser and desktop. Rejected — doubles the boundary, testing surface, and security model. Forks choose one primary mode with `/skill:workflow-stack`.

## Consequences

### Positive

- Native window and OS installers without maintaining a separate backend server.
- Capability ACL limits what the WebView can invoke — explicit security model.
- Smaller artifact size than typical Electron bundles for equivalent UI.
- `tauri-action` provides a standard multi-OS release matrix.

### Negative

- WSL/Linux dev requires WebKitGTK system packages and a display (WSLg or X server) for manual `pnpm tauri dev`.
- Unsigned installers trigger OS security warnings until a fork adds signing.
- WebView-only dev (`pnpm dev` in a browser) needs IPC stubs — real backend requires `pnpm tauri dev`.

### Follow-up

- Done (`58f1683`) — `src-tauri/` scaffold, `tauri.conf.json`, capabilities, Vite Tauri settings, `pnpm tauri` script.
- Done (`9e9f5fc`) — tag-triggered `release.yml` with `tauri-action` matrix.
- Signing, notarization, and auto-update remain fork responsibilities — documented gap.

## Related work

- IPC and Rust backend: [0014](./0014-rust-backend-ipc-boundary.md)
- SQLite persistence: [0015](./0015-sqlite-rust-owned.md)
- Vite bundler (amended): [0003](./0003-vite-spa.md)
- Commits: `58f1683`, `9e9f5fc`
- Supersedes: browser-only static-hosting deployment as the template default
