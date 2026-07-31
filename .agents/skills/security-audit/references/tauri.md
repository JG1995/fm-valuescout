# Tauri — desktop IPC and capabilities security audit

Load this file when recon finds `src-tauri/`, `tauri.conf.json`, `capabilities/`, `invoke` / `invokeCommand`, or `@tauri-apps/*` in the scoped diff. Read `references/universal.md` first.

For WebView XSS, `VITE_*` exposure, TanStack Query cache, and client-only auth patterns, also read `references/react.md` — the Tauri WebView is still attacker-controlled JavaScript. This file covers the **desktop boundary**: IPC, capabilities, Rust commands, plugins, and `tauri.conf.json` security settings.

**Sink tracing** (SQLi in `service.rs`, path canonicalization, `Command` args, Serde, IPC error strings) lives in `references/rust.md` — load both files when auditing `src-tauri/`. See `rust.md` §Ownership vs `tauri.md`.

**Production persistence path** (rusqlite migrations + queries, no WebView SQL) lives in `coding-standards/references/tauri.md` §Database (thick backend) and ADR-0015. Do not duplicate those rules here — audit whether implementation follows them.

Adapted from [Tauri v2 security](https://v2.tauri.app/security/), [capabilities](https://v2.tauri.app/security/capabilities/), OWASP injection and XSS guidance, and common Tauri plugin footguns (2025–2026).

## What Tauri protects and does not protect

Tauri splits the app into two trust groups:

| Trust group | Runs | System access |
| --- | --- | --- |
| **WebView (frontend)** | React bundle, any script that executes in the WebView | Only what capabilities grant via IPC |
| **Rust core + plugins** | Commands, `service.rs`, plugin code in the binary | Full process access — files, network, shell if wired |

The **IPC layer** is the bridge. Capabilities decide which windows/webviews may call which commands and plugin APIs. Rust command code still runs with full privileges — a missing validation in `commands.rs` is as serious as a missing check on a public HTTP handler.

Tauri does **not** protect against:

- XSS or prototype pollution in the WebView — if the attacker runs JS in your origin, they can call every IPC surface the window's capability allows.
- Weak Rust command validation — trusting IPC args without length, type, path, or ownership checks.
- Over-broad capabilities — `shell:default`, unscoped `fs:*`, or `sql:*` on `main` multiply blast radius after any WebView compromise.
- Secrets in the frontend bundle — `VITE_*` is still fully visible inside the WebView (see `react.md` §Secrets and client env (`VITE_`)).
- Unsigned or unverified updates when the updater plugin is added without signature checks.

## Attack surface

| Layer | What attackers reach | Primary files |
| --- | --- | --- |
| **IPC commands** | Any `#[tauri::command]` registered in `generate_handler!` | `src-tauri/src/features/**/commands.rs`, `lib.rs` |
| **Capabilities ACL** | Which commands/plugins each window may invoke | `src-tauri/capabilities/*.json`, `tauri.conf.json` |
| **Plugins** | log, fs, shell, http, dialog, opener, updater (when added); SQL plugin only if a fork adds it | `lib.rs`, `Cargo.toml`, capabilities |
| **WebView config** | CSP, dev CSP, asset protocol scope, remote URL access | `tauri.conf.json` `app.security` |
| **Frontend invoke** | Command names and args from feature `api/` | `src/lib/tauri-client.ts`, `features/*/api/` |
| **Build / release** | Unsigned installers, embedded keys in binary | CI workflows, env at build time |
| **Deep links / custom schemes** | OS opens app with attacker-controlled URL | `tauri.conf.json`, command handlers for deep links |

There is no separate HTTP API in the template default — IPC **is** the API.

## Recon signals

Search scoped code:

```text
# IPC and commands
#\[tauri::command\]|generate_handler!|invokeCommand|invoke\(
allow-origin|remoteUrls|dangerousDisableAssetCspModification|withGlobalTauri
macOSPrivateApi|freezePrototype|assetProtocol

# Capabilities — over-permissioning
shell:|fs:|http:|dialog:|opener:|sql:allow
core:default|permissions":\s*\[
"\$HOME|\$APPDATA|allow-read|allow-write|allow-execute

# Rust sinks
Command::new|std::process|tokio::process
\.execute\(|\.prepare\(|\.query_row\(|params!\[|format!\(.*SELECT|format!\(.*INSERT
std::fs::|read_to_string|write\(|remove\(
reqwest::|hyper::|ureq::
serde_json::from_str|from_value.*args

# Frontend — dynamic invoke (bypass wrapper discipline)
invoke\([^'"]|invokeCommand\([^'"]
from ['"]@tauri-apps/api

# Config
devUrl|beforeDevCommand|csp|devCsp|unsafe-eval|unsafe-inline
Database\.load|@tauri-apps/plugin-sql
```

Map every registered command to: (1) capability permission if required, (2) validation in `commands.rs` or `service.rs`, (3) data returned to the WebView.

## IPC as the trust boundary

### Core rules

- Treat every IPC argument as **attacker-controlled** — same as an HTTP request body or query param.
- Validate in **Rust** at the command boundary. Client Zod or TypeScript types are UX, not security.
- Return **bounded DTOs** — large row sets, file contents, or internal error chains should not cross IPC by default.
- **Authorize** when commands act on user-specific or role-specific resources — the template has no auth; forks must add checks in Rust, not only hide UI routes.

### High-risk command patterns

| Pattern | Risk | Audit trace |
| --- | --- | --- |
| Command accepts filesystem path string | Path traversal, read/write arbitrary files | Arg → `std::fs` / plugin path without canonicalization + root check |
| Command accepts URL string | SSRF if Rust fetches it | Arg → `reqwest` / `http` plugin |
| Command accepts shell string | Command injection | Arg → `std::process::Command` / `shell` plugin |
| Command accepts SQL fragment | SQL injection | Arg concatenated into query in `service.rs` |
| Command returns raw `String` error from `rusqlite` / IO | Info leak — schema, paths, internal state | Error mapped 1:1 to WebView |
| `State` holds secrets exposed via command | Secret exfiltration via IPC | Command returns admin token or DB URL |
| Unregistered command callable anyway | Should not happen if ACL correct — verify capability deny default | Window with no capability cannot invoke |

### Dynamic invoke from the WebView

`invokeCommand` should use **fixed command name strings** in feature `api/` fetchers. Dynamic command names from user input or API JSON let an attacker call any registered command the capability allows.

Audit: grep for `invoke(` or `invokeCommand(` outside `tauri-client.ts` and test mocks.

### Command registration drift

Custom `#[tauri::command]` handlers are **allowed to all windows by default** in Tauri v2. They do **not** need per-command `allow-get_status` entries in capability JSON — plugin permissions use `${plugin}:${permission}`; app commands use a separate mechanism.

**Template default:** `capabilities/default.json` grants only `core:default` on `main`. Commands such as `get_status`, `get_demo_value`, and `set_demo_value` are reachable without extra capability entries — **expected**, not an ACL gap. `build.rs` is bare `tauri_build::build()` with no `AppManifest::commands` restriction.

**When command surface grows:** restrict which windows may invoke which commands via `AppManifest::commands` in `build.rs` (see `coding-standards/references/tauri.md` §Capabilities). Audit `build.rs` for command allowlists and pair with Rust validation in `commands.rs`.

Opposite failures:

- Plugin permission in capability JSON but plugin removed — stale permission (lower risk).
- Command blocked in `AppManifest::commands` while UI still invokes it — broken app (functional, not a security finding).

Do **not** flag missing capability allow entries for custom app commands under the template default.

## Capabilities and permissions

Capabilities (`src-tauri/capabilities/*.json`) map **windows/webviews** → **permissions**. Deny-by-default: a webview with no matching capability has **no IPC access**.

### Audit checklist

| Check | Why |
| --- | --- |
| **Least privilege on `main`** | Template uses `core:default` only — verify no accidental `sql:*`, `fs:*`, `shell:*` |
| **Glob patterns** | `windows: ["*"]` or `webviews: ["*"]` spreads permissions to every future window |
| **Separate capabilities per window class** | Settings/preview windows should not inherit admin FS or shell scope |
| **`remoteUrls` enabled** | Remote HTTPS content using app IPC — rare; expands XSS to IPC if misconfigured |
| **Platform-specific capabilities** | Mobile/desktop differences — ensure desktop fork does not ship mobile-only broad perms |
| **Rebuild after ACL change** | Capabilities are compile-time input — stale manual edits in only `tauri.conf.json` vs `capabilities/` |
| **Plugin permission scopes** | `fs:allow-read` without path scope = broad read |

### Common over-permissioning (fork additions)

| Permission | Typical mistake | Impact after WebView XSS |
| --- | --- | --- |
| `shell:allow-execute` | Unscoped or full `shell:default` | Arbitrary OS command execution |
| `fs:allow-read` / `write` | No `$APP` / scoped subpath | Read secrets, overwrite app data |
| `http:allow-request` | No URL scope | SSRF from Rust on behalf of app |
| `sql:allow-select` / `execute` on `main` | WebView SQL for product data | SQL injection + direct DB access in JS |
| `dialog:allow-open` without filters | User tricked to pick sensitive file | Exfil via subsequent read command |

**Scoped paths:** prefer `$APP`, `$APPDATA`, `$CONFIG`, explicit subdirectories — not blanket home directory. See plugin docs for scope grammar.

Point implementers at `coding-standards/references/tauri.md` §Capabilities — audit verifies compliance.

## Content Security Policy (`tauri.conf.json`)

CSP limits what the WebView can load and execute. It is **defense-in-depth** against XSS — it does not replace capability least privilege.

Template default (production `csp` in `tauri.conf.json`):

- `default-src 'self' asset:` — bundled app + asset protocol
- `connect-src ipc: http://ipc.localhost` — IPC bridge
- `img-src 'self' asset: http://asset.localhost blob: data:` — images and blobs
- `style-src 'self' 'unsafe-inline'` — Tailwind/runtime inline styles (common trade-off)
- No explicit `script-src` in config — Tauri injects script policy for bundled assets

Dev `devCsp` adds `http://localhost:5173`, `ws://localhost:5173`, and `script-src 'self' 'unsafe-eval'` for Vite HMR — **must not** ship as production CSP.

| Check | Finding |
| --- | --- |
| `dangerousDisableAssetCspModification: true` | Tauri stops injecting asset CSP nonces — easier XSS gadget use |
| `script-src` includes `unsafe-eval` in production CSP | Dynamic code execution in WebView |
| `default-src` includes `https:` or `*` | Remote script load → XSS → IPC |
| Missing `connect-src` for required endpoints | Broken app — or overly broad `*` |
| User markdown/HTML without CSP backup | Relying only on CSP `unsafe-inline` for styles |

Also review `app.security.headers` when set — COOP, CORP, frame ancestors for clickjacking.

## SQL injection and persistence

Sink-level SQLi (dynamic `format!`, `params!` discipline, dynamic table/column names) — audit in `references/rust.md` §SQL injection (`rusqlite` / SQLite).

This file adds **Tauri-specific persistence** checks only:

| Check | Risk |
| --- | --- |
| **WebView SQL plugin** | Fork grants `sql:*` and uses `@tauri-apps/plugin-sql` from feature code — client-trusted SQL within plugin scope; architectural violation in template default |
| **Runtime migration content** | Any command accepting migration SQL or arbitrary SQL strings from IPC — should not exist |
| **Wrong DB directory** | DB file not under `app_data_dir` per ADR-0015 — integrity/availability |

Migration files in `db/migrations.rs` are developer-controlled — low runtime risk unless wired to IPC at runtime.

## High-risk plugins (when forks add them)

### Filesystem (`tauri-plugin-fs`)

- Path traversal via plugin API or capability scope — pair capability JSON audit (this file) with custom command path checks in `references/rust.md` §Path traversal and filesystem.
- Symlink following outside scope.
- Overwrite of config or SQLite DB from WebView-triggered write.

Audit: capability scope JSON and plugin permissions here; canonicalize + prefix checks in Rust commands in `rust.md`.

### Shell (`tauri-plugin-shell`)

- **Never** pass unsanitized user input to shell commands.
- `shell:open` with `javascript:` or attacker URL — opens malicious handler.
- Sidecar binaries: verify path is fixed at build time, not chosen from IPC.

### HTTP (`tauri-plugin-http`)

- SSRF: user-supplied URL fetched from Rust with server/network privileges.
- Credential leakage: cookies or auth headers sent to attacker host.
- Audit capability URL allowlists and every outbound request in commands.

### Dialog / opener

- Social engineering: "Select file to continue" → read sensitive path.
- `opener` to arbitrary URL — phishing, `file:` handlers.

### Asset protocol

- `assetProtocol.enable` with broad `scope` — exfil local files via `asset://` URLs in WebView.
- CSP `asset:` without tight scope — combine with XSS.

### SQL plugin (not template default)

- Template default does **not** register `tauri-plugin-sql`. Flag if a fork adds it with WebView `sql:*` permissions.
- If migrations-only plugin use returns, require static versioned SQL and no product `Database.load` from JS.

### Log plugin

- Logging IPC args or errors with PII, tokens, or query parameters — logs readable on disk.

### Updater (common fork addition)

- Update manifest from HTTP without signature verification → full app takeover.
- Downgrade attacks if version not monotonic — verify updater plugin signing docs.

## Remote content and dev configuration

| Config | Risk |
| --- | --- |
| `devUrl` pointing to remote host | Dev-only — remote code in WebView with dev capabilities |
| `build.devUrl` / loading `https://` in production WebView | Remote code execution surface |
| `withGlobalTauri: true` | Exposes Tauri internals globally — easier abuse from injected script |
| `beforeDevCommand` runs untrusted script | Supply chain at dev time |

Production builds should load **bundled** `frontendDist` (`dist/`), not a remote origin.

## Secrets and sensitive configuration

| Location | Rule |
| --- | --- |
| `VITE_*` / WebView bundle | Public — same as SPA audit |
| Rust `std::env` / compile-time env in binary | Not secret from reverse engineering — use OS keychain plugins for real secrets |
| `tauri.conf.json` | No API keys; CSP and window config only |
| SQLite file on disk | Local user can read — encrypt at rest only if product requires |
| Installer signing keys | CI secrets only — never in repo |

IPC should not return connection strings, signing keys, or service tokens to the WebView.

## XSS chaining → IPC escalation

Any WebView XSS (see `react.md`) becomes **IPC abuse** within capability limits:

1. Attacker script calls `invoke('set_demo_value', { value: '...' })` or other allowed commands.
2. If `sql:*` or `fs:*` granted, direct data exfiltration or file read without custom commands.
3. If `shell:*` granted, OS command execution.

**Severity:** WebView XSS in a Tauri app is often **Critical** when capabilities are broad — not Medium as in a static SPA without backend.

Mitigations to verify:

- Narrow capabilities on `main`
- CSP reduces script injection likelihood
- Rust validation limits what commands can do even when invoked

## Multi-window and privilege separation

Tauri recommends separate capabilities for windows with different needs (e.g. main vs settings vs preview).

Audit:

- Secondary windows created with `WebviewWindow` — which capability label?
- Preview window with `fs:read` on `$HOME` while main stays minimal — good pattern.
- All windows on `default` capability with fork-added `shell:execute` — bad pattern.

## Deserialization and type confusion

IPC args deserialize via Serde into Rust types at the command boundary. Full Serde audit patterns (typed structs, `deny_unknown_fields`, size limits) live in `references/rust.md` §IPC deserialization and type safety — apply them to every `#[tauri::command]` arg list.

## Error handling and information leakage

What crosses IPC to the WebView on failure is a **Tauri trust-boundary** issue — audit whether raw `rusqlite` / `io` strings reach the frontend. Sink-level error mapping patterns live in `references/rust.md` §Error handling and information leakage.

## Isolation pattern (optional hardening)

Tauri offers an **Isolation** pattern (separate secure JS context intercepting IPC). Template uses **brownfield** — WebView calls Tauri directly. Isolation adds complexity; audit only if fork enables `app.security.pattern`.

## Testing and mocks

See `references/testing.md` for fixture secrets, permissive mock gaps, and CI artifact exposure. Summary:

| Check | Note |
| --- | --- |
| `mockIPC` in Vitest | Test-only — must not ship in production bundle |
| Playwright `__TAURI_INTERNALS__` stub | E2E only |
| Test commands registered in production `lib.rs` | `#[cfg(test)]` or feature flag only |

## Static audit methodology

1. **List commands** — `generate_handler!` in `lib.rs` → table of command → file → side effects.
2. **Map capabilities** — each permission → windows/webviews → commands/plugins enabled.
3. **Trace IPC args** — each user-controlled field → validation → sink (SQL, fs, network, shell).
4. **Review `tauri.conf.json` security** — CSP prod vs dev, asset protocol, headers, remote URLs.
5. **Cross-check coding standards** — persistence path, `tauri-client` sole invoke site, no WebView SQL.
6. **Layer WebView audit** — run `react.md` checks on `src/` — XSS is the usual entry to IPC abuse.
7. **Release** — unsigned installers (template default) are integrity risk on download, not runtime XSS.

Skip pure SPA server classes (CSRF on cookie API) unless fork adds local HTTP server.

## False positives

- `core:default` on `main` with no plugin permissions — template walking skeleton.
- Custom app commands in `generate_handler!` without per-command capability JSON entries — expected under Tauri v2 defaults with `core:default` only.
- `style-src 'self' 'unsafe-inline'` for Tailwind — common; not equivalent to `script-src unsafe-inline`.
- `devCsp` with `unsafe-eval` — dev only when not used in production `csp`.
- Parameterized `rusqlite` with static SQL and `params!` — not SQLi.
- Rust-owned `PRAGMA user_version` migrations without WebView `sql:*` — not WebView SQL surface.
- Rust validation in `service.rs` for demo value length — correct boundary pattern.
- Commands with no user args (`get_status`) — low IPC injection surface; still check capability exposure.

## Fork note — local HTTP server or hybrid apps

If recon finds Axum/Tauri hybrid, localhost HTTP, or custom protocols alongside IPC:

- Treat HTTP endpoints as a **second trust boundary** — same authz rules as commands.
- CORS on `localhost` does not mean safe — other local processes may call the port.
- Re-run SSRF and CSRF classes on that surface.

## Sources

| Source | Use in this file |
| --- | --- |
| [Tauri v2 Security](https://v2.tauri.app/security/) | Trust groups, IPC bridge, CSP role |
| [Capabilities](https://v2.tauri.app/security/capabilities/) | Deny default, window/webview scoping, remote URLs |
| [Tauri config security](https://v2.tauri.app/reference/config/) | CSP, assetProtocol, freezePrototype, headers |
| OWASP SQL Injection / XSS cheat sheets | Parameterized queries, XSS → capability escalation |
| `coding-standards/references/tauri.md` | Production persistence, capability implementation rules |
| ADR-0014, ADR-0015 | IPC boundary, SQLite Rust-owned model |
