# UI Agent Workflow

## Intent

Provide trusted, developer-only control of the real Tauri application for live UI inspection and polish without exposing the developer's normal application data or adding that control surface to ordinary or release builds.

## Delivered behavior

- `./scripts/dev ui-agent` starts a real development application with a new temporary application-data directory.
- `./scripts/dev ui-agent --dump /absolute/path/dump.json` validates and ingests the read-only dump through the existing Rust snapshot service before control becomes available.
- The pinned `tauri-plugin-mcp-bridge` and `@hypothesi/tauri-mcp-cli` 0.12.0 integration gives a trusted Codex task upstream WebView inspection, interaction, resize, screenshots, logs, JavaScript, and IPC access without depending on Codex Desktop MCP tool mounting.
- `$workflow-ui-polish` is a manual opt-in for cohesive visual and interaction improvements. It requires initial and final evidence, hot-reload reinspection, two viewport checks, keyboard and focus checks, log inspection, and normal repository validation.

## Final architecture

- The optional `ui-agent` Cargo feature enables the Rust bridge only in debug builds. Release builds reject that feature.
- The UI-agent-only Tauri configuration grants `withGlobalTauri` and `mcp-bridge:default` only for that mode. The bridge binds only to `127.0.0.1`.
- The launcher owns its temporary profile and deletes it when the application exits. Product code continues to use Rust-owned SQLite and real IPC; no live-database mode, WebView SQL path, or product-facing test command exists.
- Rust runs migrations and optional snapshot ingest before it registers the bridge. Invalid seed input therefore fails before a controllable endpoint starts.
- The upstream CLI daemon owns session state. In version 0.12.0, its advertised IPC command executor does not dispatch application-defined commands; a necessary read-only product IPC proof uses `window.__TAURI__.core.invoke(...)` through the trusted WebView JavaScript command.
- In WSL UI-agent mode, Vite console forwarding is disabled to avoid an upstream locale-error serialization crash. CLI-accessible frontend logs and launcher output remain the diagnostics sources.

## Important decisions

- Use the pinned upstream CLI directly. The repository does not maintain a custom MCP server, WebDriver layer, session protocol, or filtering wrapper.
- Keep the command contract to an empty temporary database or a supplied absolute dump. The tracked one-player golden dump proves mechanics only; it does not represent populated layouts.
- Treat the upstream arbitrary-JavaScript and IPC tools as trusted local development capability, not general authority. Existing Git, product, FM, plugin, and destructive-action rules remain in force.

## Migration and operational implications

- No product schema or migration changed. UI-agent sessions run existing migrations against their temporary database.
- Start the launcher in another terminal, then connect with `pnpm exec tauri-mcp driver-session start --json` and verify the reported application identity and repository path.
- Repeat the native runtime and release-boundary checks when the pinned upstream packages change.

## Validation

- `./scripts/dev format` passed.
- `./scripts/dev test` passed: 157 Vitest tests.
- `./scripts/dev check` passed, including 198 Rust tests with 2 ignored.
- `./scripts/dev smoke` passed: 12 Playwright checks.
- A featureless `pnpm tauri build --no-bundle` passed.
- A golden-dump native launch migrated and ingested the temporary database and listened only on `127.0.0.1:9223`.
- Fresh Sol High feature review found no blocking, CRITICAL, HIGH, MEDIUM, or NITPICK findings.

## Follow-up

- Repeat a fresh WSL empty-profile lifecycle, including a second run.
- Verify the ordinary featureless development runtime in a fresh native session.
- Run an upstream CLI control proof that covers HMR, restart, both viewports, accessibility, and log forwarding.
- Use a realistic developer dump before drawing conclusions about populated layouts.
