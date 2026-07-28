# Rust — Tauri backend crate security audit

Load this file when recon finds Rust under `src-tauri/`, `Cargo.toml`, `build.rs`, or IPC commands implemented in Rust — even when `references/tauri.md` is also loaded.

Read `references/universal.md` first. IPC capabilities, CSP, plugin permissions, and WebView→IPC escalation live in `references/tauri.md`. This file covers **Rust-side sinks and idioms** in the backend crate: SQL, filesystem, process, deserialization, dependencies, and error handling.

## Ownership vs `tauri.md`

| Topic | Owner |
| --- | --- |
| **`rust.md` (this file)** | Sink tracing in `commands.rs` / `service.rs` — SQLi patterns, path canonicalization, `Command` args, Serde structs, error strings over IPC, logs, `cargo audit` |
| **`tauri.md`** | Capability ACL, CSP/`devCsp`, plugin scopes, XSS→IPC escalation, WebView SQL plugin, multi-window permissions |

When both files load, audit capabilities in `tauri.md` first, then trace each command's args to sinks using this file — do not duplicate full sink checklists in both places.

Adapted from [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/), OWASP injection guidance, [rusqlite](https://docs.rs/rusqlite) parameter docs, and Rust security checklists (2025–2026).

## What Rust protects and does not protect

Rust's memory safety reduces classic buffer overflows and use-after-free in safe code. `unsafe` blocks, FFI, and some driver crates reintroduce memory risk.

Rust does **not** protect against:

- Logic bugs — missing authz, IDOR implemented in safe Rust
- SQL injection when queries are built with `format!` or string concat
- Command injection when `Command::new("sh").arg("-c").arg(user_input)` or shell plugins pass unsanitized input
- Path traversal when paths are joined without `canonicalize` + prefix check
- Deserialization bombs or confused deputies via loose Serde types
- Leaking secrets through logs, IPC errors, or `Debug` formatting
- Vulnerable **dependencies** — `cargo audit` finds known CVEs; safe code cannot fix a bad crate

## Attack surface

| Layer | What attackers reach | Primary files |
| --- | --- | --- |
| **IPC commands** | All `#[tauri::command]` args | `features/**/commands.rs` |
| **Service / DB layer** | SQL, file I/O, outbound HTTP | `features/**/service.rs`, `db/` |
| **Process execution** | Shell, sidecars, `std::process` | Commands, shell plugin wiring |
| **Filesystem** | Read/write/delete paths | `std::fs`, fs plugin scopes |
| **Deserialization** | Serde from IPC JSON | Command arg structs |
| **Shared state** | `State<Mutex<T>>` data races or leaks | `lib.rs`, commands |
| **Build script** | `build.rs` network or file access | `build.rs`, `tauri_build` |
| **Dependencies** | Transitive crate code | `Cargo.toml`, `Cargo.lock` |

IPC is the primary entry point in this template. Treat every command arg as attacker-controlled unless the command has zero parameters and no ambient authority.

## Recon signals

```text
# SQL
format!\(.*SELECT|format!\(.*INSERT|format!\(.*UPDATE
\.execute\(|\.prepare\(|\.query_row\(|params!\[
execute_batch\(|raw_sql

# Process
Command::new|std::process|tokio::process
shell:|tauri_plugin_shell

# Files
std::fs::|read_to_string|write\(|remove\(
canonicalize|PathBuf::from\(.*arg
\.\.\/|path\.join\(

# Deserialization
serde_json::from_str|from_value
serde_json::Value|deny_unknown_fields
#\[serde\(default\)\]

# Secrets and errors
log::|tracing::|println!.*password|token
impl.*Serialize.*Error|to_string\(\)
unwrap\(|expect\(|panic!

# Unsafe and FFI
unsafe\s*\{|extern\s+"C"
```

Map each command: args → validation → sink (SQL, fs, network, process).

## SQL injection (`rusqlite` / SQLite)

Production path: parameterized queries in `service.rs` with static SQL strings and `params!` / bound parameters — see ADR-0015 and `coding-standards/references/rust.md` §Database.

| Pattern | Risk |
| --- | --- |
| `format!("SELECT ... {}", input)` | Classic SQLi |
| `conn.execute(&dynamic_string, [])` where string includes user fragments | SQLi |
| Dynamic table/column names from IPC without allowlist | Schema escape / SQLi |
| String-built `WHERE` clauses with user input | SQLi |
| SQLite `PRAGMA` or extension load from user input | Logic escape |

Safe patterns:

- Fixed SQL + `params!` / bound parameters for all dynamic **values**
- Allowlist enum for dynamic identifiers (table names, sort columns)

**Migration SQL** in `db/migrations.rs` is developer-controlled — low runtime risk. Flag any command that accepts migration content or runs arbitrary SQL from IPC.

**WebView SQL plugin:** if a fork grants `sql:*` to the WebView, audit as client-trusted SQL — see `references/tauri.md` §SQL injection and persistence. Template default rejects this path.

Database file on wrong path (not under `app_data_dir` per ADR-0015) is integrity/availability — not SQLi — but local user can read SQLite file on disk without encryption.

## Command and shell injection

| Pattern | Risk |
| --- | --- |
| `Command::new("sh").arg("-c").arg(user_input)` | Full shell injection |
| Single string passed to shell plugin execute | Same |
| `Command::new(program).arg(user_input)` when `program` is user-chosen | Arbitrary binary execution |
| Sidecar path from IPC arg | Run attacker binary |
| Environment vars from IPC passed to child process | `LD_PRELOAD` class on some platforms |

Safe pattern: fixed program name, discrete `.arg()` values, no shell — `Command::new("my_tool").arg("--file").arg(validated_path)`.

Audit `tauri-plugin-shell` capability scopes when present — unscoped `shell:execute` after WebView XSS is Critical.

## Path traversal and filesystem

| Pattern | Risk |
| --- | --- |
| `base.join(user_path).read()` without canonicalize | `../../../etc/passwd` |
| Strip `../` manually instead of canonicalize | Bypass via encoding or `..\\` |
| Symlink following outside intended root | Read sensitive file via symlink |
| Write to path derived from IPC without root check | Overwrite config, SQLite DB, or binaries |
| Temp files in world-readable dir with predictable names | TOCTOU / leak |

Safe pattern:

```rust
let base = app.path().app_data_dir()?.canonicalize()?;
let path = base.join(safe_filename).canonicalize()?;
if !path.starts_with(&base) {
    return Err(/* ... */);
}
```

Reject absolute paths from IPC when only relative names are intended. Use allowlist regex for filenames (`^[a-zA-Z0-9._-]+$`).

`tauri-plugin-fs` scope JSON must match Rust path checks — plugin ACL details live in `references/tauri.md` §High-risk plugins.

## IPC deserialization and type safety

Serde deserializes IPC JSON into Rust types at the command boundary.

| Risk | Check |
| --- | --- |
| `serde_json::Value` or `String` then manual parse | Weak validation |
| Missing `#[serde(deny_unknown_fields)]` on security-sensitive structs | Unexpected fields accepted |
| Large strings/arrays from IPC | DoS — no size limit at boundary |
| `#[serde(default)]` on security fields | Missing field → insecure default |
| Untagged enum from IPC | Confused deputy between variants |

Prefer typed structs per command; validate length and charset after deserialize; reject oversize payloads early.

`tauri-specta` (fork) reduces TS/Rust drift — does not replace validation.

## Authentication and authorization in Rust

Template has no auth — forks must enforce in **commands** or `service.rs`, not only in React `beforeLoad`.

| Anti-pattern | Finding |
| --- | --- |
| Command uses `user_id` from IPC arg without session check | IDOR |
| Admin command registered without role check | Broken function-level auth |
| Shared `State` holds "current user" set only from frontend | Tamperable if another command can set it |
| "Public" read command returns rows for all users | Data leak |

Every mutating command and sensitive read: trace identity source — OS user, session token in Rust, or capability-gated single-user desktop assumption (document if intentional).

## Shared state and concurrency

| Risk | Check |
| --- | --- |
| `std::sync::Mutex` held across `.await` in async command | Deadlock or stall |
| `Mutex` poison ignored with `unwrap` on lock | Panic → DoS |
| Global cache without eviction | Memory DoS from IPC spam |
| Secrets in `State` returned by command | IPC exfiltration |

Log command failures once at boundary — not raw `Debug` of state containing secrets.

## Error handling and information leakage

Template uses `Result<T, String>` for early commands. Audit:

| Pattern | Risk |
| --- | --- |
| `map_err(|e| e.to_string())` on `rusqlite::Error` | Schema/table names in WebView |
| `?` on `io::Error` straight to IPC | Full paths leaked |
| Different errors for "not found" vs "forbidden" | User enumeration |
| `unwrap()` / `expect()` in command path | Panic may crash app or leak in crash telemetry |

Prefer `AppError` with safe client message and internal log line — see `coding-standards/references/rust.md` §Error handling.

## Logging and tracing

| Risk | Example |
| --- | --- |
| `log::info!("token={}", token)` | Secrets in log files |
| IPC args logged at info level | PII, passwords in query strings |
| Log plugin writes WebView console to disk | XSS output persisted |

Audit `log` / `tracing` in `commands.rs` and `service.rs`. Redact or omit sensitive fields.

## `unsafe`, FFI, and crypto

| Signal | Audit |
| --- | --- |
| `unsafe` blocks | Document invariant; prefer safe wrappers |
| `extern "C"` callbacks | Caller-controlled pointers |
| Custom crypto (`md5`, `sha1` for passwords) | Use established crates (`argon2`, `ring`) |
| Hardcoded keys in Rust source | Same as hardcoded secrets in JS |
| `rand` without `OsRng` for security tokens | Predictable tokens |

`rusqlite` / `libsqlite3-sys` use `unsafe` via FFI — expected; not a finding by itself.

## `build.rs` and compile-time behavior

| Risk | Check |
| --- | --- |
| `build.rs` network fetch without pinning | Supply chain at compile time |
| `include_str!` of sensitive file | Secret embedded in binary |
| `AppManifest::commands` misconfiguration | Command ACL drift — pair with `references/tauri.md` |

## Dependency auditing

Gate runs `cargo clippy` with `-D warnings` — not full CVE audit unless fork adds it.

| Check | Tool / practice |
| --- | --- |
| Known CVEs in deps | `cargo audit` in CI |
| Banned crates or licenses | `cargo deny` |
| `cargo update` without review | Unexpected semver changes |
| Git dependencies without rev pin | Moving target |

Flag unmaintained crates on security-critical paths (TLS, crypto, SQL drivers).

## Testing vs production paths

| Risk | Check |
| --- | --- |
| `#[cfg(test)]` command registered in production `generate_handler!` | Test-only IPC in release |
| Integration tests use `unwrap` on setup | Masks failure modes — test quality, not prod |
| Test helper exposes `pub` command for tests | Callable from production if registered |

Security of test mocks: `references/testing.md`.

## Static audit methodology

1. **List commands** — `generate_handler!` in `lib.rs`.
2. **Per command** — args, validation, calls into `service.rs`, sinks.
3. **SQL audit** — grep `format!`, dynamic `query`, `.bind()` usage.
4. **Process/fs audit** — `Command`, `std::fs`, plugin permissions.
5. **Error paths** — what crosses IPC on failure.
6. **Cross-check `tauri.md`** — capabilities, CSP, plugin scopes.
7. **Dependencies** — `Cargo.lock` age, audit tooling in CI.

## False positives

- Parameterized `rusqlite` with static SQL and `params!` — not SQLi.
- `get_status` with no args — low injection surface; still check capability exposure.
- `app_data_dir()` for SQLite path per ADR-0015 — correct location, not traversal.
- `thiserror` + safe `Serialize` on `AppError` — good pattern.
- `#[cfg(test)]` modules in same file — not in production handler unless mis-registered.
- `cargo clippy -D warnings` in gate — does not replace `cargo audit`; not a false positive if audit missing — report as "not assessed" not confirmed gap.

## Sources

| Source | Use in this file |
| --- | --- |
| [rusqlite docs](https://docs.rs/rusqlite/) | Parameter binding, injection prevention |
| [Tauri — calling Rust](https://v2.tauri.app/develop/calling-rust/) | Command boundary |
| `references/tauri.md` | Capabilities, plugins, IPC escalation |
| `coding-standards/references/rust.md` | Module layout, error types, DB flow |
| ADR-0014, ADR-0015 | IPC boundary, SQLite ownership |
