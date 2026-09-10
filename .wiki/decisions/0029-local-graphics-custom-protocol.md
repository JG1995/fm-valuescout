# 0029 — Local graphics custom protocol

## Status

Accepted

## Context

Local graphics currently cross Tauri invoke as JSON number arrays. Three routes convert the bytes to base64 data URLs. This duplicates transport and conversion work, expands memory use, and keeps image delivery on a request/response path designed for bounded DTOs rather than raw image responses.

JAY-64 must support large local FM graphics roots while keeping filesystem authority in Rust. The new delivery path must not expose a root path, image source path, pack name, or arbitrary file request. It must also preserve graphics generations so a stale root cannot supply an image after replacement.

Tauri 2.11.3 supports asynchronous custom URI scheme protocols on `tauri::Builder`. Its Windows origin is `http://<scheme>.localhost/...`; CSP must allow that exact image origin. Source: <https://docs.rs/tauri/latest/tauri/struct.Builder.html>.

The developer supplied a representative graphics root, and the supervisor confirmed that it is available through the current host's mounted Windows filesystem. The root is private runtime input. This ADR does not record its path.

## Decision

Register one asynchronous `graphics` Tauri custom protocol. It accepts only a `GET` request with the exact closed request identity:

```text
http://graphics.localhost/<generation>/<kind>/<uid>
```

`generation` is the committed graphics generation. `kind` is one closed graphics kind. `uid` is a positive decimal UID. The handler rejects every other host, method, segment shape, query, fragment, kind, UID, stale generation, missing mapping, or failed image read.

The handler retrieves the existing Rust-owned `GraphicsRuntime`. It does not open a root, accept a path, create a second cache, or cross invoke. It returns only validated raw PNG, JPEG, or WebP bytes with explicit MIME, `X-Content-Type-Options: nosniff`, and `Cache-Control: no-store` headers. Runtime cache bounds remain the owner of image-byte reuse.

React uses one shared graphics image component. It builds only the closed protocol URL from safe status, kind, and UID, and uses native lazy loading and asynchronous decoding. The component removes an image after load error so existing caller-owned fallback slots remain visible.

This decision narrowly amends [ADR-0014](./0014-rust-backend-ipc-boundary.md): `invoke` remains the sole production request/response boundary for product DTOs, status, and mutations. The `graphics` protocol is the sole exception for local graphics raw image bytes. It is not an HTTP server or a general file-access surface.

JAY-64 first registers and proves this closed protocol while `resolve_graphics` remains available. Its next atomic migration moves every consumer to the shared component, then removes `resolve_graphics`, byte-array result types, result Query keys/options, resolve mocks, and route-local base64 data URL conversion together. Packet 9 audits every remaining image flow and removes `data:` from image CSP only if none requires it; otherwise, it retains `data:`. The custom protocol origin allowance remains exact and narrow. The developer waived manual native Windows Tauri custom-protocol/CSP proof. This changes validation evidence only; it does not change the closed request grammar, narrow CSP, or filesystem-security invariants.

## Alternatives considered

### Keep invoke with JSON number arrays and data URLs

This is implemented today and preserves the narrow invoke boundary. It copies bytes through serde, JavaScript arrays, base64 conversion, and data URLs. It does not meet the production-scale delivery goal. Rejected.

### Use a Tauri asset URL or a broad file protocol

An asset URL is plausible for application resources. A broad file protocol would require more path authority and request validation. Neither naturally encodes the graphics generation, closed kind, and exact UID while retaining the existing capability-root mapping boundary. Rejected.

### Persist image files or the generated index

Persisted derived data could avoid a scan after startup. It adds invalidation, disk growth, and stale-root complexity. The accepted graphics architecture keeps only the selected root in SQLite. Rejected.

## Consequences

### Positive

- Browser image loading receives raw bytes without invoke serialization or base64 conversion.
- One component removes three route-local delivery implementations.
- The request grammar preserves exact UID and committed-generation constraints.
- Asynchronous protocol response and runtime lock release prevent image file I/O from blocking graphics state transitions.

### Negative

- `src-tauri/tauri.conf.json` must permit the exact custom image origin in production and development.
- The developer-approved validation gap leaves native Windows custom-protocol registration and CSP unproven. Unit parser/response tests, compilation, existing automated Linux/Windows Rust checks, frontend tests, and `./scripts/dev smoke` still run; smoke is Chromium-stub evidence and does not prove native Windows custom-protocol registration or CSP.
- Custom protocol request validation and error response tests become part of the graphics trust-boundary portfolio.

### Follow-up

- JAY-64 first implements and validates this protocol while retaining the existing resolve IPC path, then migrates all consumers and removes that path atomically.
- The implementation records final production limits only after path-free representative-pack calibration with no truncation. The local host-aware harness records exact host/OS/filesystem context and does not label WSL timing or memory data as native Windows performance.

## Related work

- Feature plan: [Production-scale FM graphics](../features/active/production-scale-fm-graphics.md)
- Existing boundary: [ADR-0014](./0014-rust-backend-ipc-boundary.md), narrowly amended for local graphics raw image bytes only
- Existing graphics delivery: [Local FM graphics](../features/completed/local-fm-graphics.md)
- Commits: Pending JAY-64 delivery
- Supersedes: JSON number-array/base64 data URL delivery for local graphics
- Directory index: Deliberately unchanged because this bounded JAY-64 replan permits only the active ledger, the JAY-64 Active entry in `.wiki/TODO.md`, and this ADR.
