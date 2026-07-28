# 0015 — SQLite with Rust-owned migrations and queries

## Status

Accepted (amended `1e313f9` — rusqlite-only path)

## Context

The desktop template needs local persistence with versioned schema changes. SQLite is the default embedded database for Tauri apps. The choice is not only which database — it is **where** migrations run and **where** queries execute relative to the IPC trust boundary ([0014](./0014-rust-backend-ipc-boundary.md)).

An earlier template default used `tauri-plugin-sql` for migrations (with `plugins.sql.preload`) and `sqlx` for queries. That split forced two SQLite touchpoints and tied the file path to `app_config_dir` only to match plugin preload. The WebView SQL plugin API (`Database.load`) remains rejected for product features.

## Decision

Use **SQLite** for local persistence with a **single rusqlite-owned** data path:

1. **Driver** — **`rusqlite`** with the **`bundled`** feature in `service.rs` / `db/mod.rs`. Parameterized queries only. Do not ship `sqlx` or `tauri-plugin-sql` in the template default.
2. **Migrations** — versioned registry in `src-tauri/src/db/migrations.rs`, applied at connection open via **`PRAGMA user_version`** (transactional per pending version).
3. **Startup** — `db::open` in `lib.rs` setup opens the file, applies pending migrations, and manages `Db(Mutex<Connection>)`. No `plugins.sql.preload`.
4. **Path resolution** — `APP_DB_FILE` under **`app_data_dir`** (absolute path via Tauri path APIs). Never store the DB in the repo tree.
5. **Frontend access** — `invokeCommand` only. Product features do **not** use `@tauri-apps/plugin-sql` or `Database.load`. Do **not** grant `sql:*` on the main capability for product persistence.

Data flow:

```text
React api/ → invokeCommand → commands.rs → service.rs → db/ → SQLite
```

## Alternatives considered

### `@tauri-apps/plugin-sql` from the WebView

Official plugin with a JS API. Fast for prototypes. SQL and validation live in untrusted JS; capabilities must expose SQL permissions to the main window. Rejected for production data in this template.

### `tauri-plugin-sql` migrations + `sqlx` queries (prior default)

Documented Tauri v2 plugin path for migrations, with async `sqlx` for queries. Rejected for the template after the dual-stack cost outweighed the benefit: two drivers, preload/path coupling to `app_config_dir`, and unused WebView plugin surface. See amendment commit `1e313f9`.

### `sqlx` instead of `rusqlite`

Valid async SQLite driver with pool support. Rejected for the template default in favor of rusqlite familiarity and a smaller dependency tree. Forks may choose `sqlx` with an ADR amendment — do not ship both drivers for the same app.

### Dual drivers (`sqlx` + `rusqlite`)

Redundant connection and migration assumptions. Rejected.

### `rusqlite_migration` crate

Adds a dependency for registry helpers. Rejected until the in-repo `PRAGMA user_version` registry becomes painful.

### Server-side database (Postgres, etc.)

Requires a separate server process or hosted service. Out of scope for a local-first desktop template default.

## Consequences

### Positive

- All SQL stays in Rust — one language for queries, validation, and tests.
- One driver owns migrations and queries — no plugin URI vs absolute-path sync.
- `cargo test` can run service logic against temp absolute-path databases.
- Capabilities stay minimal — no WebView SQL permissions for product data.
- Versioned migrations ship with the app and apply before first query.
- Bundled SQLite avoids a system `libsqlite` dependency on target OS.

### Negative

- Sync `rusqlite` work under `Mutex` can briefly block the UI on slow disk I/O — use `spawn_blocking` (or async commands) when queries become heavy.
- ORM/query-builder layers deferred — raw parameterized SQL in services for the template scope.
- No automatic upgrade path from older template demo DBs under `app_config_dir` (discard and recreate).

### Follow-up

- Done (`39b145a`) — first `demo_value` persistence (plugin + sqlx era).
- Done (`1e313f9`) — rusqlite-only swap, `app_data_dir`, `PRAGMA user_version` migrations.
- Schema beyond the demo table, encryption, and backup strategy remain fork decisions.

## Related work

- IPC boundary: [0014](./0014-rust-backend-ipc-boundary.md)
- Desktop shell: [0013](./0013-tauri-v2-desktop-shell.md)
- Commits: `39b145a`, `1e313f9`
- Supersedes: WebView SQL as a production data path; dual plugin+sqlx stack as the template default
