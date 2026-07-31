# Rust — Tauri backend crate

Load this file when you implement or review Rust code under `src-tauri/`, or when the diff touches `Cargo.toml`, `build.rs`, or Rust lints in the gate.

Read `references/universal.md` first. Read `references/testing.md` when the change adds or changes tests. IPC commands, capabilities, and plugin wiring live in `references/tauri.md`. This file owns Rust idioms and module layout. Layer boundaries and pipeline overview live in `.wiki/ARCHITECTURE.md`. Security audit depth for Rust sinks (SQL, fs, process, Serde) lives in `security-audit/references/rust.md` — pair with `security-audit/references/tauri.md` for IPC ACL during audits.

External references (read for depth, do not duplicate here):

- [Apollo GraphQL Rust Best Practices](https://github.com/apollographql/rust-best-practices) — idioms, errors, testing, clippy discipline
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/about.html) — public API design when a module becomes a reusable boundary
- [Rust Analyzer style guide](https://rust-analyzer.github.io/book/contributing/style.html)
- [Tauri — calling Rust](https://v2.tauri.app/develop/calling-rust/) — command modules, async commands, IPC error serialization
- [Tauri — state management](https://v2.tauri.app/develop/state-management/) — `State`, `Mutex`, async mutex trade-offs

Calibrate depth to hobbyist solo-dev scope in `AGENTS.md`. Structural quality stays high; skip paranoid padding on paths that cannot fail in practice.

## Stack defaults

| Piece | Role |
| --- | --- |
| Rust (stable) | Backend language for the Tauri crate |
| `cargo fmt` | Format — gate runs `cargo fmt --check` |
| `cargo clippy` | Lint — gate runs with `-D warnings` |
| `cargo test` | Unit, integration, and doc tests |
| `thiserror` | Crate- and feature-level error enums (add when first real error type is needed) |
| `serde` | Serialize IPC args and results at the command boundary |

Reserve `anyhow` for thin binary entry (`main.rs`) only — not in `lib.rs` or feature modules. See Apollo handbook Chapter 4.

## Architecture principles

Mirror the frontend's Bulletproof React adaptation:

- **Clear boundaries** between app shell (`lib.rs`), shared infrastructure, and feature modules.
- **One direction of dependencies** — shared → features → shell. Never import shell code from a feature.
- **Feature folders** hold most product logic; `lib.rs` wires plugins and registers commands.
- **Consistency** — same patterns for errors, command naming, and test layout across the crate.
- **IPC commands are trust boundaries** — validate untrusted input in Rust, not only in the WebView.
- **Return bounded results** — paginate, summarize, or stream large data; the WebView is not a batch processor.

Pick **one** owner per concern. Shared DB connection and migrations live in `db/`, not duplicated inside each feature.

## Thin frontend, thick backend

Rust owns computation, aggregation, file/DB I/O, and validation at trust boundaries. The WebView receives **bounded** IPC results — pages, summaries, scalars — not raw large datasets.

| Work | Owner |
| --- | --- |
| SQL queries, joins, pagination | `features/<name>/service.rs` → `db/` |
| IPC boundary | `features/<name>/commands.rs` |
| Schema migrations | `db/migrations.rs` + `PRAGMA user_version` apply at open |

See `references/react.md` §Thin frontend, thick backend for the frontend mirror. Database path rules: §Database below and `references/tauri.md` §Database.

## Project structure

Tauri keeps the Rust crate in `src-tauri/`. Application logic lives under `src-tauri/src/`.

Organize by **domain feature**, not by file type at the crate root. This parallels `src/features/<feature>/` on the frontend so a solo dev can map UI feature ↔ backend feature by name.

```text
src-tauri/
├── Cargo.toml
├── build.rs
├── capabilities/           # Tauri ACL — see tauri.md
├── icons/
└── src/
    ├── main.rs             # Thin entry — calls lib::run(); rarely edited
    ├── lib.rs              # App shell — Builder, plugins, invoke_handler assembly
    ├── error.rs            # Crate-level AppError (thiserror) when needed
    ├── db/                 # Shared persistence (migrations registry, connection helpers)
    │   ├── mod.rs
    │   └── migrations.rs
    ├── utils/              # Shared pure helpers (no feature-specific logic)
    └── features/           # Domain modules — one folder per product feature
        └── <feature>/
            ├── mod.rs
            ├── commands.rs # #[tauri::command] — IPC boundary
            ├── service.rs  # Pure business logic (add when commands grow)
            └── types.rs    # Serde types for IPC args/results
```

Only add submodules a feature needs. A walking-skeleton feature may be only `mod.rs` + `commands.rs`. Do not create empty `service.rs` or `types.rs` folders "for later."

### Why `features/` instead of flat `commands/`

Tauri’s docs often show a flat `commands/` module. That is fine for a single demo command. This template uses `features/<name>/` because:

- It mirrors `src/features/<name>/` on the frontend (same name, same mental model).
- It matches **vertical-slice** layout — product code grouped by domain, not by technical layer at the crate root ([vertical slices in Rust](https://github.com/irahardianto/awesome-agv/blob/main/.agents/rules/project-structure-rust-cargo.md) applies the same idea at module level).

When a feature grows, add files inside its folder — not new top-level `services/` or `controllers/` directories:

```text
features/<feature>/
├── commands.rs   # IPC boundary — validate, map errors, call inward
├── service.rs    # orchestration (optional)
├── logic.rs      # pure rules, no I/O (optional — prefer when tests need isolation)
├── types.rs      # IPC / domain types
└── error.rs      # feature error enum nested in AppError (optional)
```

`logic.rs` vs `service.rs`: use `logic.rs` for pure functions with no I/O; use `service.rs` when orchestration touches files, DB, or other side effects. One file is enough until both kinds of code exist.

### Single crate vs Cargo workspace

Stay in **one `src-tauri` crate** for the template default and typical solo-dev forks. A [Cargo workspace](https://hda.daz.is/getting-started/project-structure/) with separate `domain/`, `infrastructure/`, and `apps/` crates enforces boundaries through the compiler — valuable at scale, but costly for a hobbyist template.

# ponytail: single-crate `features/` layout
# Upgrade to a workspace when compile times hurt, multiple binaries share logic, or forbidden cross-crate imports need compiler enforcement — not at first persistence or IPC feature.

### Naming collision note

Rust **Cargo features** (`[features]` in `Cargo.toml`) are compile-time flags. The `features/` directory here means **product features** — the same word Bulletproof React uses on the frontend. When you mean a Cargo feature, say "Cargo feature" explicitly.

### App shell (`lib.rs`)

`lib.rs` is the backend app shell — the counterpart to `src/app/` on the frontend.

- Register Tauri plugins and window setup.
- Collect `#[tauri::command]` functions from feature modules into `generate_handler!`.
- Do not put business logic here beyond wiring.

```rust
// lib.rs — illustrative
tauri::Builder::default()
    .plugin(/* ... */)
    .invoke_handler(tauri::generate_handler![
        features::health::commands::get_status,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

### Shared modules

| Path | Role | Frontend analogue |
| --- | --- | --- |
| `error.rs` | Crate-level error enum with `thiserror` | `src/lib/` error helpers |
| `db/` | Migrations registry, shared DB helpers | N/A — persistence is backend-only |
| `utils/` | Pure helpers used by 3+ callers | `src/utils/` |

Keep cross-feature types in `utils/` or a small `types.rs` at crate root only when truly shared. Prefer feature-local `types.rs` when the type belongs to one domain.

## Database

Persistence is **backend-only** — the WebView never executes SQL in product features.

| Piece | Role |
| --- | --- |
| `db/migrations.rs` | Versioned schema changes applied via `PRAGMA user_version` |
| `db/mod.rs` | Connection helper — `app_data_dir()` + filename → absolute path → `rusqlite::Connection` |
| `features/<name>/service.rs` | Parameterized queries, joins, aggregation — returns domain types |
| `features/<name>/commands.rs` | Validates IPC args, calls `service.rs`, maps to bounded serde DTOs |

**Flow:** `db::open` in `lib.rs` setup creates the parent directory, opens `rusqlite` on the absolute path, and calls `migrations::apply` before managing `Db(Mutex<Connection>)`. Queries and migrations share that one connection. Parameterized statements only — never concatenate user input into SQL.

Commands never embed raw SQL strings when `service.rs` exists — keep SQL in `service.rs` or `db/` so unit tests can exercise queries without IPC.

**Thick backend:** filter, sort, paginate, and join in Rust before serializing IPC results. If a query would return thousands of rows, return a page + total count (or a scalar summary) instead.

# ponytail: sync `Mutex<Connection>` for demo IPC commands
# Upgrade to `spawn_blocking` (or an async pool driver) if queries block the WebView under measurable load.

IPC wiring and capability rules for SQL permissions live in `references/tauri.md` §Database.

## Import and module rules

### Dependency direction

```text
shared (error, db, utils) → features/<feature> → lib.rs (shell)
```

- **No cross-feature imports.** If two features must interact, route through a shared module or compose in `lib.rs` — same rule as the frontend.
- **No business logic in `main.rs`.** Tauri convention: `main.rs` calls `lib::run()`.
- **Features do not import from `lib.rs`.** The shell depends on features, not vice versa.

### `use` declaration order

Follow the Rust ecosystem default (Apollo handbook §1.7):

1. `std` / `core` / `alloc`
2. External crates from `[dependencies]`
3. Workspace crates (if any)
4. `super::` and `crate::`

Prefer `rustfmt` with `group_imports = "StdExternalCrate"` when the project adds a `rustfmt.toml`.

### Module visibility

- `#[tauri::command]` functions in **submodules** are `pub fn` — `lib.rs` registers them by path.
- Do **not** define `pub #[tauri::command]` functions directly in `lib.rs` — the macro can conflict with `pub use` in the same module ([Tauri tutorials](https://tauritutorials.com/blog/tauri-command-fundamentals)).
- Business logic in `service.rs` or `logic.rs` is usually `pub(crate)` until integration tests need a wider API.
- Do not expose internal helpers as `pub` without reason.

Feature `mod.rs` declares child modules (`mod commands;`, `mod service;`). It may re-export types the shell or other shared modules need. That is a module boundary, not a frontend-style barrel that hides every import path.

## Commands (Rust side)

IPC wiring details (capabilities, permissions, frontend invoke wrapper) live in `references/tauri.md`. Rust-specific command rules:

| Topic | Rule |
| --- | --- |
| Sync vs async | Prefer `async` commands for I/O or work that can block — keeps the WebView responsive ([Tauri docs](https://v2.tauri.app/develop/calling-rust/)). |
| Async + borrows | `async` commands that take `State<'_, T>` or `&str` must return `Result<T, E>` — not bare `T`. |
| Arguments | Prefer a serde struct when a command has more than two or three parameters. |
| Handler registration | One `generate_handler![...]` call with every command — multiple `invoke_handler` calls do not merge. |
| Body size | Validate at the boundary, map errors, then call `service.rs` / `logic.rs` — not a 40-line handler. |

## Shared state

Use Tauri `State` only when multiple commands share runtime data (connection pool, cache, config snapshot). Feature-local state usually belongs in SQLite or the frontend Query cache — not a global `AppState` by default.

When you need shared mutable state:

1. Start with `std::sync::Mutex` — preferred for short critical sections ([Tauri state docs](https://v2.tauri.app/develop/state-management/), [Tokio guidance](https://tokio.rs/tokio/tutorial/shared-state)).
2. Use `std::sync::RwLock` when reads heavily outnumber writes.
3. Use `tokio::sync::Mutex` (or `tauri::async_runtime::Mutex`) **only** when you must hold the lock across an `.await` point.
4. Never hold a `std::sync::MutexGuard` across `.await` — release the lock first, or use the tokio mutex.

Register state in `setup` or before `run` with `.manage(...)`. Wrap the managed type in `Mutex` once — do not double-wrap.

# ponytail: one big `AppState` struct with every mutex field
# Upgrade to per-domain managed state or a dedicated task owning I/O when contention appears or lock scopes grow — not at scaffold time.

## Naming

| Item | Convention | Example |
| --- | --- | --- |
| Modules and files | `snake_case` | `commands.rs`, `health/` |
| Functions and commands | `snake_case` | `get_status`, `save_preferences` |
| Types and structs | `PascalCase` | `HealthStatus`, `AppError` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_RETRY_COUNT` |
| IPC command names | `snake_case`, verb-first | `get_status`, not `status` or `healthStatus` |

Align IPC command names with the frontend invoke wrapper when practical (`get_status` on both sides). Prefix with the feature name only when a bare verb would collide across features (`health_get_status` vs `sync_get_status`).

## Error handling

Follow Apollo handbook Chapter 4 and [Tauri’s IPC error guidance](https://v2.tauri.app/develop/calling-rust/). Summary for this template:

- Return `Result<T, E>` from functions that can fail. Use `panic!` only for bugs, tests, or truly unrecoverable states.
- Avoid `unwrap` and `expect` in production paths. Allowed in tests and when failure is provably impossible.
- Use `thiserror` for `AppError` and feature-specific error enums. Nest with `#[from]` when wrapping lower layers.
- Use `?` to bubble errors. Prefer `let Ok(x) = ... else { ... }` when you need early return without matching noise.

### IPC errors — prefer typed, not `String`

`Result<T, String>` is fine for the first scaffold command. Switch to a crate-level `AppError` before the app ships — retrofitting typed errors is painful.

Tauri requires command error types to implement `serde::Serialize`. Two patterns:

**A — Tagged JSON (preferred once the frontend matches on error kind):**

```rust
#[derive(Debug, thiserror::Error, serde::Serialize)]
#[serde(tag = "kind", content = "message")]
#[serde(rename_all = "camelCase")]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
```

The WebView receives `{ kind, message }` — mirror with a TypeScript discriminated union on the frontend.

**B — String serialization (minimal scaffold):**

```rust
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
```

Do not leak stack traces, file paths, or internal IDs to the frontend. Map internal errors to safe `AppError` variants at the command boundary. Log the full `Debug` representation in Rust before returning.

Log command failures once at the boundary (`log::error!("get_status failed: {:?}", e)`) — not scattered through every helper.

At trust boundaries (IPC args, file paths, SQL parameters): validate explicitly. Prefer small serde structs with constrained fields over many loose strings. One clear validation at the boundary is enough for solo-dev scope inside trusted code.

## Idioms and performance

Follow Apollo handbook Chapter 1 and Chapter 3. Highlights:

- Prefer borrowing (`&T`, `&str`) over cloning unless you need owned data or an API requires it.
- Pass small `Copy` types by value (`u32`, `bool`, small structs).
- Prefer iterators over manual index loops when clarity allows.
- Do not extract a helper for 1–2 duplicated lines — wait for the third real repetition (Rule of Three). Wrong abstractions cost more than duplication.
- Let structure and naming replace long comments. Use `///` doc comments for public API; link to ADRs for design rationale.

## Clippy and format

The gate runs:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
```

`./scripts/dev format` applies `cargo fmt` (after Biome) so local autofix matches the gate's `cargo fmt --check`.

- Fix warnings; do not silence them without `#[expect(clippy::...)]` and a one-line reason.
- Prefer workspace or package `[lints.clippy]` in `Cargo.toml` when you need consistent lint levels across the crate.
- Run `cargo clippy` locally before checkpoint — same surface as CI.
- Editor: recommend rust-analyzer with `linkedProjects` → `src-tauri/Cargo.toml` and `check.command` → `clippy` (see `.vscode/settings.json`).

## Testing

Follow Apollo handbook Chapter 5 and `references/testing.md`. Rust-specific placement:

| Kind | Location | Access | Purpose |
| --- | --- | --- | --- |
| Unit | `#[cfg(test)] mod tests` in the same file or `service.rs` | Private functions, `pub(crate)` | Edge cases, error paths, pure logic |
| Integration | `src-tauri/tests/` | Public API only | End-to-end command + DB paths |
| Doc | `///` examples in public items | Compiled by `cargo test` | Living documentation |

Conventions:

- One behavior per test. Name tests like sentences: `returns_error_when_input_is_empty`.
- Group related tests under `mod get_status { ... }` inside `#[cfg(test)] mod tests`.
- Exercise error paths — assert on `to_string()` or `PartialEq` when the error type supports it.
- Share **setup** in helpers; keep each test's action and assertion inline (DAMP over DRY in tests).
- Colocate migration tests with `db/` or run migrations against a temp file in integration tests.

`lib.rs` + `main.rs` split enables integration tests against the library crate without spawning the full desktop binary.

Optional tooling (add only when a commit needs it):

- `insta` — snapshot tests for serialized output (YAML feature for readable diffs)
- `rstest` — parametrized cases with descriptive names

Do not add snapshot testing for trivial primitives or critical-path logic that needs precise `assert_eq!`.

Export binding tests: if you add `tauri-specta`, export TypeScript bindings in a `#[test]` (or `#[cfg(debug_assertions)]`) — not only on app startup — so CI catches drift.

## Optional upgrades (not template defaults)

Add these when a measurable trigger fires — not at scaffold time:

| Tool | Trigger to add |
| --- | --- |
| [tauri-specta](https://github.com/specta-rs/tauri-specta) | Manual `invoke` strings and hand-written TS types drift from Rust commands |
| `insta` | Serialized output is large or structural; precise `assert_eq!` is unreadable |
| `rstest` | Many similar cases with descriptive names; not a substitute for one clear test |
| Cargo workspace | Compile times, shared libraries, or compiler-enforced crate boundaries hurt |
| Isolation pattern ([Tauri](https://v2.tauri.app/concept/inter-process-communication/isolation/)) | High-risk IPC surface, untrusted remote content, or compliance requires pre-Rust validation |

Brownfield IPC (Rust validation + capabilities) is the template default. Isolation adds a sandboxed iframe and ceremony most solo-dev apps do not need.

## What not to do

- Put all commands in a flat `commands.rs` at crate root once you have more than one feature — use `features/<name>/commands.rs`.
- Import across `features/<a>/` → `features/<b>/`.
- Use `anyhow::Result` in feature modules or shared libraries.
- `#[allow(clippy::...)]` without `expect` and a reason.
- Barrel `mod` re-exports that hide the real file path — import the concrete module (same rule as no `index.ts` barrels on the frontend).
- Business logic in `#[tauri::command]` bodies beyond validation and mapping — call `service.rs` when the handler grows.
- Hold `std::sync::Mutex` across `.await` in async commands.
- Top-level `services/`, `controllers/`, or `repositories/` folders — use `features/<name>/` instead.
- `pub #[tauri::command]` in `lib.rs` itself.

## Frontend parity checklist

When a frontend feature needs IPC or persisted data:

1. Add `src-tauri/src/features/<name>/` with `commands.rs` (and `service.rs` when SQL or file I/O appears).
2. Keep the folder name aligned with `src/features/<name>/` so routes, ledger, and commands read as one story.
3. Frontend `api/` calls `invokeCommand` only — no `@tauri-apps/plugin-sql` in product features.

Pure UI features with no backend work do not need a Rust module until they invoke commands or read persisted data.
