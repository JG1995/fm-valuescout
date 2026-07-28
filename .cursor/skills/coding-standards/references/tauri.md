# Tauri — desktop IPC, capabilities, and plugins

Load this file when you implement or review Tauri configuration, capabilities, IPC between the WebView and Rust, `@tauri-apps/*` usage, or plugin wiring. Load alongside `references/rust.md` when the diff touches `src-tauri/src/`.

Read `references/universal.md` first. Read `references/testing.md` when the change adds or changes tests. Rust idioms and backend module layout live in `references/rust.md`. Frontend Query and feature layout live in `references/react.md`. Vite dev/build settings for Tauri live in `references/vite.md` once scaffolded. Deep security audit rules for IPC live in `security-audit/references/tauri.md`. Other stack security refs (`vite.md`, `rust.md`, `testing.md`, `react.md`) mirror this skill's reference set — load from `security-audit/SKILL.md` during audits.

External references (read for depth, do not duplicate here):

- [Tauri v2 docs](https://v2.tauri.app/) — project structure, calling Rust, state, security
- [Capabilities](https://v2.tauri.app/security/capabilities/) and [permissions](https://v2.tauri.app/security/permissions/)
- [SQL plugin](https://v2.tauri.app/plugin/sql/) — reference only; template default does **not** use it for product persistence
- [Testing](https://v2.tauri.app/develop/tests/) — frontend mocks
- [dchuk/claude-code-tauri-skills](https://github.com/dchuk/claude-code-tauri-skills/tree/main/tauri) — broad skill catalog (use as lookup, not as mandatory ceremony)

Calibrate depth to hobbyist solo-dev scope in `AGENTS.md`. This template is **desktop-only** — no dual web/HTTP boundary, no mobile-first layout.

## Stack defaults

| Piece | Role |
| --- | --- |
| Tauri v2 | Desktop shell, WebView, IPC runtime |
| `@tauri-apps/api` | Frontend `invoke`, events, mocks — **runtime** `dependencies` |
| `@tauri-apps/cli` | `pnpm tauri dev` / `build` — devDependency |
| `rusqlite` (`bundled`) | Migrations + queries in Rust `service.rs` / `db/` |

Custom `#[tauri::command]` handlers own app logic and **production database access**. Official plugins own platform APIs (dialog, fs, etc.). Do not reimplement plugin surfaces as bespoke commands unless the plugin cannot do the job.

`@tauri-apps/plugin-sql` is **not** in the template default and is **not** the production data path from the WebView — see §Database.

## Architecture principles

IPC replaces HTTP as the frontend/backend boundary in this template:

- **Thin frontend, thick backend** — React owns UI and presentation; Rust owns computation, aggregation, and I/O. See `references/react.md` §Thin frontend, thick backend.
- The WebView JavaScript is **untrusted** — same as browser JS. Validate in Rust commands and scoped plugin permissions.
- **Deny by default** — capabilities grant only what each window needs. Add permissions when a feature needs them, not preemptively.
- **One invoke wrapper** on the frontend (`src/lib/tauri-client.ts`) — mirrors `api-client.ts` for the old HTTP boundary.
- **Feature `api/` folders** call through the wrapper; components and routes do not call `invoke` directly.
- **Explicit contracts** — every command, plugin, and capability permission is declared. Verbosity is the security model.

Pick **one** IPC primitive per need:

| Need | Use | Avoid |
| --- | --- | --- |
| Request/response, errors, Query cache | **Commands** (`invoke`) | Events for RPC-shaped work |
| Fire-and-forget notifications, lifecycle | **Events** (`listen` / `emit`) | Commands when no return value is needed |
| Large or streaming payloads | **Channel** (plugin or command arg) | Giant JSON over `invoke` |

## Boundary map

```text
WebView (React)
  src/features/<feature>/api/     # fetchers, queryOptions — IPC consumers
  src/lib/tauri-client.ts       # sole invoke wrapper + shared IPC errors
  src/testing/setup.ts          # mockIPC for Vitest

src-tauri/
  capabilities/*.json           # ACL — what the WebView may call
  src/lib.rs                    # plugins + invoke_handler assembly
  src/features/<feature>/commands.rs  # #[tauri::command] providers
  src/db/migrations.rs          # PRAGMA user_version migration registry
```

Frontend feature names should match backend feature folders when both sides exist (`health` ↔ `health`).

## Frontend IPC client

Create one wrapper in `src/lib/tauri-client.ts`:

- Import `invoke` only here (and in test mocks) — not in components or route files.
- Map IPC failures to a typed error class (mirror `ApiError` from the HTTP client).
- When Rust uses tagged `AppError` (`kind` + `message`), parse the rejection into a discriminated union here.

```typescript
// Pattern sketch — adapt names to the feature
import { invoke } from "@tauri-apps/api/core";

export class TauriCommandError extends Error {
  readonly kind?: string;

  constructor(message: string, kind?: string) {
    super(message);
    this.name = "TauriCommandError";
    this.kind = kind;
  }
}

export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    // Map structured Rust errors when AppError uses serde tag = "kind"
    if (typeof error === "object" && error !== null && "kind" in error) {
      const e = error as { kind: string; message?: string };
      throw new TauriCommandError(e.message ?? e.kind, e.kind);
    }
    throw new TauriCommandError(String(error));
  }
}
```

Do not scatter `window.__TAURI__` access — use `@tauri-apps/api` imports through the wrapper.

## Feature `api/` layer (IPC)

Mirror the HTTP pattern from `references/react.md`:

1. Types in `features/<feature>/types/` (or shared `src/types/` when truly shared).
2. Fetcher in `features/<feature>/api/` calls `invokeCommand`, not `invoke`.
3. `queryOptions` / mutation options for TanStack Query — same as HTTP fetchers.

```typescript
// features/health/api/fetch-health-status.ts — pattern sketch
import { invokeCommand } from "@/lib/tauri-client";
import type { HealthStatus } from "../types/health-status";

export async function fetchHealthStatus(): Promise<HealthStatus> {
  return invokeCommand<HealthStatus>("get_status");
}
```

Command names are `snake_case` on both sides. Rust snake_case args map to JS **camelCase** by default (`user_name` → `userName`). Prefer consistent snake_case on both sides via `#[tauri::command(rename_all = "snake_case")]` when it reduces confusion.

Validate untrusted input with Zod in the fetcher or in a small `schemas/` module when args are non-trivial — Rust still validates at the command boundary.

## Rust commands (summary)

Full rules live in `references/rust.md`. Tauri-specific reminders:

- Register every command in one `tauri::generate_handler![...]` in `lib.rs`.
- Do not put `pub #[tauri::command]` functions in `lib.rs` itself — use `features/<name>/commands.rs`.
- Prefer `async` commands for I/O. Use owned args (`String`) in async handlers unless returning `Result`.
- More than two or three args → one serde struct.

## Capabilities and permissions

Tauri v2 uses capability files in `src-tauri/capabilities/` to grant plugin permissions per window.

### Template defaults

- Start with `capabilities/default.json` for the main window label (`main` unless you rename it).
- Include `$schema` pointing at `../gen/schemas/desktop-schema.json` after first build — IDE autocomplete for permission identifiers.
- Grant **only** permissions in use — add plugin permissions when a feature needs them, not preemptively.

Example minimal capability (adjust identifiers after plugins land):

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Main window — walking skeleton permissions",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```

Add plugin permissions when a feature needs them — e.g. `dialog:default` for file pickers. SQL plugin permissions apply only when the WebView calls the SQL plugin (not the production persistence path).

### Rules

| Rule | Why |
| --- | --- |
| Least privilege per window | Limits blast radius if the WebView is compromised |
| Separate capability files per window class | Settings/preview windows get fewer permissions than `main` |
| Scoped paths for `fs` plugins | Use `$APP`, `$HOME/...` scopes — not blanket `fs:default` without scopes |
| Rebuild after capability changes | ACL is compile-time input to the app manifest |
| Match window **labels**, not titles | Labels are stable identifiers in code |

Custom `#[tauri::command]` handlers are allowed to all windows by default until you restrict them via `AppManifest::commands` in `build.rs`. For apps with few commands, Rust-side validation is enough. Restrict manifest exposure when command surface grows.

Do not enable `remote` URL access unless the product loads remote content — desktop-only local apps rarely need it.

## Plugins

### Registration pattern

In `lib.rs` — register plugins where the feature needs them. Template log plugin is debug-only in setup:

```rust
tauri::Builder::default()
    .setup(|app| {
        if cfg!(debug_assertions) {
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            )?;
        }
        // …
        Ok(())
    })
    .invoke_handler(tauri::generate_handler![/* ... */])
```

Every plugin needs:

1. Rust crate in `Cargo.toml` with the correct feature flags for that plugin.
2. `.plugin(...)` registration in `lib.rs` (on the builder or via `AppHandle` in setup).
3. Capability permissions for its **WebView JS API** — only when the WebView calls that plugin.

Skip step 3 for plugins registered only on the Rust side (no WebView API). Step 3 is required when feature `api/` or components call the plugin from JavaScript.

## Database (thick backend)

This template uses **thin frontend, thick backend** for persistence:

| Concern | Owner | Location |
| --- | --- | --- |
| Schema migrations | Rust | `src-tauri/src/db/migrations.rs` — `PRAGMA user_version` |
| Migration application at startup | Rust | `db::open` in `lib.rs` setup (before first query) |
| Query execution (production) | Rust | `features/<name>/service.rs` → `db/` helpers (`rusqlite`) |
| Shared connection | Rust | `Db(Mutex<Connection>)` managed state |
| IPC surface | Rust | `features/<name>/commands.rs` — bounded DTOs (pages, summaries) |
| WebView data access | React | `invokeCommand` only — feature `api/` fetchers, never SQL plugin |

**Production rule:** React does **not** call `@tauri-apps/plugin-sql` or `Database.load`. Do not add `sql:allow-execute` / `sql:allow-select` to the main window capability for product features — SQL runs in Rust, not in the WebView.

```text
React api/ → invokeCommand → commands.rs → service.rs → db/ → SQLite file
```

#### Migrations (`PRAGMA user_version`)

| Topic | Rule |
| --- | --- |
| Migration definitions | `src-tauri/src/db/migrations.rs` — version, description, SQL list |
| Application | `migrations::apply` on the open `rusqlite::Connection` — transactional per pending version |
| Shared filename | One constant (e.g. `APP_DB_FILE = "app.db"`) joined under `app_data_dir` in `db/mod.rs` |
| Version numbers | Unique, monotonic integers — one migration per schema change |

Test migrations in Rust (`db/` unit tests or integration tests) before relying on IPC demos.

**Stack preference:** **`rusqlite` (`bundled`)** owns **migrations and queries**. Do not add `tauri-plugin-sql` or a second query driver without an explicit ADR amendment.

#### Queries (Rust)

| Topic | Rule |
| --- | --- |
| Driver | `rusqlite` with `bundled` in `db/mod.rs` |
| SQL style | Parameterized queries — never concatenate user input |
| File location | `app_data_dir` via Tauri path APIs — never in the repo tree |
| Large results | Paginate or summarize in `service.rs` — return bounded IPC DTOs |

# ponytail: `@tauri-apps/plugin-sql` from feature `api/`
# Upgrade path removed — production persistence is Rust-only. Use JS SQL plugin only in a disposable spike, not in product features.

#### Capabilities

Grant `sql:*` permissions only when the WebView legitimately calls the SQL plugin (not the template default). Command-only persistence needs **no** SQL capability permissions on `main`.

### Adding other plugins

Use `pnpm tauri add <plugin>` when available. Add capability permissions from the plugin docs table. Do not install plugins "for later."

# ponytail: bespoke plugin authoring
# Upgrade to `cargo tauri plugin new` only when no official/community plugin covers the need — see dchuk tauri-plugins skill for custom plugin structure.

## IPC security model

**Brownfield** (default): WebView calls Tauri APIs directly; security is capabilities + Rust command validation. This is the template default — no extra isolation app.

**Isolation** pattern: sandboxed iframe validates every IPC message before Rust. Stronger, but adds build and maintenance cost ([Tauri isolation docs](https://v2.tauri.app/concept/inter-process-communication/isolation/)). Not the template default.

| Practice | Template stance |
| --- | --- |
| Validate paths, IDs, and enums in Rust commands | Required at trust boundaries |
| Typed command args (serde structs) | Preferred over many loose strings |
| CSP in `tauri.conf.json` | Set a sensible default when scaffold lands; tighten with app needs |
| Isolation iframe | Optional upgrade for high-risk remote content |
| Trust frontend validation alone | Never — Zod on the client is UX, not security |

Security audit depth lives in `security-audit/references/tauri.md` — not duplicated here.

## Events

`src/lib/tauri-client.ts` owns **commands** (`invoke` / `invokeCommand`) — not events. Import `listen` / `emit` from `@tauri-apps/api/event` directly when the UI subscribes to backend pushes.

Use events when the backend pushes state without a request:

```rust
use tauri::{AppHandle, Emitter};
app.emit("sync-progress", payload)?;
```

```typescript
import { listen } from "@tauri-apps/api/event";

const unlisten = await listen<Payload>("sync-progress", (event) => {
  // update UI or Query cache
});
unlisten();
```

Prefer commands + Query for request/response. Prefer events for progress, background job completion, or menu/tray signals. Mock events in Vitest with `mockIPC(..., { shouldMockEvents: true })` when needed.

## Testing

Align with `references/testing.md` and the template gate (`./scripts/dev test`, `./scripts/dev smoke`).

### Vitest — mock IPC

Replace MSW with `@tauri-apps/api/mocks`:

```typescript
import { afterEach } from "vitest";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";

afterEach(() => {
  clearMocks();
});

mockIPC((cmd, args) => {
  if (cmd === "get_status") return { status: "ok" };
});
```

- Call `clearMocks()` in `afterEach` — prevents leakage between tests.
- Mock at IPC level in feature tests — do not mock `tauri-client` unless testing the wrapper itself.
- Register handlers that match real command names.

### Playwright smoke — browser without WebView

Playwright drives Chromium against the Vite dev server, not a real Tauri WebView. Stub IPC before the app loads:

```typescript
await page.addInitScript(() => {
  // Minimal stub — expand per command the smoke path needs
  window.__TAURI_INTERNALS__ = {
    invoke: async (cmd: string) => {
      if (cmd === "get_status") return { status: "ok" };
      throw new Error(`Unhandled IPC: ${cmd}`);
    },
    transformCallback: () => {},
  };
});
```

Keep smoke UI-focused. Command correctness belongs in Vitest + `cargo test`.

### WebDriver / tauri-driver

[tauri-driver](https://v2.tauri.app/develop/tests/webdriver/) + WebdriverIO/Selenium can automate the real WebView (Windows/Linux; macOS limited). Useful for fork products, but heavy for the template gate.

# ponytail: Playwright IPC stub only in smoke
# Upgrade to tauri-driver e2e when smoke stubs hide real WebView integration bugs — not at template scaffold.
# Scope table: [.wiki/ARCHITECTURE.md](../../../../.wiki/ARCHITECTURE.md) §6.4 Playwright smoke scope

### Rust tests

Command logic and migrations — `references/rust.md` and `cargo test`. IPC integration across the real WebView is optional and slow.

## Vite integration (summary)

When `src-tauri/` exists, Tauri-specific Vite settings live in `vite.config.ts` — full list in `references/vite.md`:

- `clearScreen: false`, `strictPort`, `envPrefix` including `TAURI_ENV_*`
- Ignore `src-tauri` in dev watch
- Platform-conditional build target / minify for production Tauri builds

Dev loop: `pnpm tauri dev` (not only `pnpm dev`) when exercising IPC.

## Optional upgrades

| Tool / pattern | Trigger to add |
| --- | --- |
| [tauri-specta](https://github.com/specta-rs/tauri-specta) | Hand-written TS types drift from Rust commands |
| Isolation pattern | High-risk remote content or compliance requires pre-Rust IPC filtering |
| `Channel` streaming | Large file/progress payloads over IPC |
| Per-window capability files | Second window class with different permissions |
| WebDriver e2e | Smoke stubs miss WebView-only failures in production |
| Custom Tauri plugins | No official plugin covers the platform API |

## What not to do

- Call `invoke` from components, route files, or hooks outside `api/` fetchers.
- Keep HTTP `api-client` alongside IPC for the same feature — desktop-only template.
- Grant `fs:default` or `shell:default` without scoped paths "just in case."
- Concatenate user input into SQL strings — use parameterized queries in Rust `service.rs` / `db/`.
- Store the SQLite file in the repository or project root.
- Use events for RPC-shaped workflows that need errors and return values.
- Add mobile-only capabilities or platform targets — out of scope for this template default.
- Copy the full [dchuk skill catalog](https://github.com/dchuk/claude-code-tauri-skills/tree/main/tauri) into repo docs — it is a reference library; this file is the curated baseline.

## dchuk skill catalog — what we use vs skip

The [dchuk/claude-code-tauri-skills](https://github.com/dchuk/claude-code-tauri-skills/tree/main/tauri) repo is a wide agent-oriented catalog (~40 topics). Useful distillations already woven above:

| Topic folder | Used for |
| --- | --- |
| `tauri-ipc`, `tauri-calling-rust` | Commands, brownfield vs isolation, invoke patterns |
| `tauri-capabilities`, `tauri-plugin-permissions` | ACL files, scoped permissions |
| `tauri-testing` | mockIPC, clearMocks, WebDriver as optional |
| `tauri-plugins` | Registration checklist (custom plugins deferred) |

Skipped or deferred for template scope: mobile distribution (`*-android-*`, `*-ios-*`), CrabNebula, code signing, sidecars, system tray, splash screen, binary size tuning, HTTP headers — add via fork docs when the product needs them, not in the default template standards.
