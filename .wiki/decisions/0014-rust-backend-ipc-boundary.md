# 0014 — Rust backend and IPC trust boundary

## Status

Accepted

## Context

A desktop app needs computation, validation, and I/O outside the WebView. The WebView JavaScript is untrusted — same as browser JS. We need one clear frontend/backend boundary, aligned feature folders on both sides, and Rust ownership of trust-boundary work.

The template previously used HTTP fetch + MSW for the walking skeleton. That boundary does not apply to a Tauri-only product.

## Decision

Use **Rust in `src-tauri/`** as the backend. **IPC (`invoke`)** is the sole production frontend/backend boundary — no HTTP API in the template default.

Adopt **thin frontend, thick backend**:

- React owns UI, routing, TanStack Query cache of **already-computed** IPC results, and client UI state.
- Rust owns SQL, aggregation, file/DB I/O, and input validation at command boundaries.

Layout:

- `src/lib/tauri-client.ts` — sole `invoke` / `invokeCommand` wrapper on the frontend.
- `src/features/<name>/api/` — Query options and fetchers call through the wrapper; components do not call `invoke` directly.
- `src-tauri/src/lib.rs` — app shell only (plugins + `generate_handler!` registration).
- `src-tauri/src/features/<name>/` — `commands.rs` (IPC boundary), `service.rs` (business logic when I/O appears).
- `src-tauri/capabilities/` — deny-by-default ACL; grant only commands and plugin permissions in use.

Custom `#[tauri::command]` handlers validate their own inputs. Return bounded DTOs over IPC — not raw large datasets.

## Alternatives considered

### HTTP server alongside IPC

REST or RPC from the WebView to a local HTTP port. Familiar pattern but redundant when Tauri IPC already provides typed commands. Adds port binding, CORS, and a second error surface. Rejected for the template default.

### Fat frontend — SQL and heavy logic in the WebView

Use `@tauri-apps/plugin-sql` or similar from JavaScript feature code. Faster to prototype but moves trust-boundary work into untrusted JS and splits persistence rules across languages. Rejected — see [0015](./0015-sqlite-rust-owned.md).

### Business logic in `lib.rs`

Single-file backend. Scales poorly as features grow. Rejected — feature folders mirror the frontend Bulletproof layout.

### Events instead of commands for request/response work

Tauri events fit fire-and-forget notifications. RPC-shaped work (Query cache, error handling) stays on commands. Mixed model documented in `coding-standards/references/tauri.md`.

## Consequences

### Positive

- One boundary to test — Vitest uses `mockIPC`; Playwright uses `page.addInitScript` stubs.
- Rust `cargo test` covers service logic without a WebView.
- Feature name alignment (`health` ↔ `health`) keeps solo-dev navigation simple.
- Capabilities enforce least privilege per window.

### Negative

- Every new capability needs Rust command + registration + capability entry + frontend fetcher.
- Browser-only `pnpm dev` cannot exercise real IPC without stubs.
- Async command and `State` patterns add Rust learning curve for frontend-only developers.

### Follow-up

- Done (`4582448`) — HTTP/MSW removed; `get_status` IPC command; `tauri-client.ts`; `mockIPC` in Vitest; Playwright IPC stub.
- Done (`807afe7`) — `cargo fmt`, `clippy`, and `test` in `./scripts/dev check`.
- Typed `AppError` and streaming channels deferred until a feature needs them.

## Related work

- Desktop shell: [0013](./0013-tauri-v2-desktop-shell.md)
- SQLite (Rust-owned): [0015](./0015-sqlite-rust-owned.md)
- TanStack Query (IPC result cache): [0005](./0005-tanstack-query.md)
- Commits: `4582448`, `807afe7`, `39b145a`
- Supersedes: HTTP + MSW as the template default frontend/backend boundary
