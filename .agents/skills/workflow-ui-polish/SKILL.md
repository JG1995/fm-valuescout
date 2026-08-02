---
name: workflow-ui-polish
description: Improve the visual design and interaction quality of a running isolated Tauri application through the pinned upstream Tauri control CLI. Use only when the developer explicitly invokes `$workflow-ui-polish` or asks to explore a running app and make cohesive UI improvements.
---

# Live UI polish

Use the real application as the design reference. Make purposeful visual and interaction improvements without changing the product contract.

## Preconditions

1. Confirm the developer explicitly requested live UI polish.
2. Confirm the developer has started `./scripts/dev ui-agent` in another terminal. Use `./scripts/dev ui-agent --dump /absolute/path/dump.json` only when the developer supplied that dump.
3. Start or inspect the upstream CLI session, then require its status to report `connected: true`, `identifier: app.fmvaluescout`, and a `cwd` that matches this repository. Do not use screenshots, JavaScript, interaction, IPC, or other broad control commands until all three values match. If another app owns the default port, leave that session untouched, connect to the FM ValueScout port reported by the launcher, and verify the metadata again. Do not infer readiness from files, ports, or a previous screenshot.
4. If the session is absent, reloading, disconnected, or targets another app, pause live actions. Report the state and reconnect before taking a fresh DOM snapshot.

Use the tracked golden dump only to prove the workflow mechanics. Do not claim that its one-player layout represents a populated save.

## Tauri control CLI

Use the pinned upstream CLI through terminal commands. Do not depend on model-mounted `mcp__tauri__*` tools.

Before the first control command, read `node_modules/@hypothesi/tauri-mcp-cli/skills/tauri-mcp-cli/SKILL.md` completely as the pinned upstream command reference. Prefix its bare `tauri-mcp` examples with `pnpm exec`. This project skill overrides the upstream skill for application identity, temporary-data, authority, validation, and Git boundaries.

```bash
pnpm exec tauri-mcp <command> --json
```

The CLI daemon preserves the driver session between terminal calls. Use `pnpm exec tauri-mcp --help` to discover the complete command list and `pnpm exec tauri-mcp <command> --help` before the first call when arguments are uncertain. Prefer `--json` for machine-readable results.

| Purpose | Command |
| --- | --- |
| Connect, inspect, and stop | `driver-session start`, `driver-session status`, scoped `driver-session stop` |
| Inspect semantics and elements | `webview-dom-snapshot`, `webview-find-element`, `webview-get-styles` |
| Capture visual evidence | `webview-screenshot` |
| Interact and wait | `webview-interact`, `webview-keyboard`, `webview-wait-for` |
| Resize and inspect windows | `manage-window` |
| Read diagnostics | `read-logs`, `ipc-monitor`, `ipc-get-captured` |
| Inspect trusted runtime state | `webview-execute-js`, `ipc-get-backend-state` |

Write screenshots beneath `.work/ui-agent/` with `webview-screenshot --file <path>`, then inspect them with `view_image`. When the workflow ends, stop only the FM ValueScout session with `pnpm exec tauri-mcp driver-session stop --app-identifier app.fmvaluescout --json` or the verified FM port. Never issue an unscoped `driver-session stop`, because it stops every session held by the CLI daemon.

## Live design loop

Read `$ui-design` before making visual or interaction decisions. Use `$coding-standards` and `$minimalism` before editing product files. Existing build, checkpoint, review, and Git workflows govern ordinary workspace changes.

1. Inspect organically. Visit the relevant routes and states that the running application reveals. Capture an initial screenshot and `webview-dom-snapshot` for each meaningful surface. Use `webview-find-element`, `webview-interact`, `webview-keyboard`, and `manage-window` as needed.
2. Identify the highest-value cohesive improvement. Prefer hierarchy, spacing, scanning, affordance, responsive fit, or interaction clarity. Keep product behavior, feature boundaries, and Rust-owned data ownership unchanged.
3. Capture a clear before image under `.work/ui-agent/` and inspect it with `view_image`.
4. Make one cohesive edit batch. Do not turn open-ended exploration into a scenario suite, visual baseline, custom control layer, or speculative UI framework.
5. Reinspect after hot reload. Wait for the actual updated UI. If Vite reloads or the app restarts, check `driver-session status`, reconnect when needed, and take a new DOM snapshot before interacting again.
6. Capture an after image and compare it to the before image for the intended improvement and regressions.

## Required checks

For each completed polish batch:

- Resize the real window to 1280×800 and 1600×900 with `manage-window`. Check clipping, overflow, density, and layout hierarchy at both sizes.
- Use `webview-keyboard` and DOM/accessibility snapshots to check keyboard reachability, visible focus, and sensible focus movement.
- Inspect empty, loading, error, or constrained states that are reachable from the live session without fabricating data or changing product behavior.
- Read frontend output with `read-logs` and Rust startup or migration output in the launcher terminal. Investigate unexpected errors before presenting the batch as complete.
- Run focused tests for changed code, then `./scripts/dev check` and `./scripts/dev smoke`. Report unavailable or environment-blocked checks truthfully.

`webview-execute-js` is a trusted local inspection command, not general permission. In upstream version 0.12.0, `ipc-execute-command` does not dispatch application-defined commands. If a read-only product IPC check is necessary, use `window.__TAURI__.core.invoke(...)` through `webview-execute-js`; do not use it to mutate product state unless the developer explicitly authorizes that action.

## Authority limits

- Do not change product behavior, architecture, feature-to-feature imports, or data ownership without a separate developer request and the applicable workflow.
- Do not access Football Manager, manage plugins, confirm external destructive actions, use a live database, or synthesize data.
- Do not commit, push, amend, rebase, squash, or start another workflow automatically. Follow the existing Git approval rules after any workspace edits.
- Do not add MCP tools, command modes, a wrapper, a scenario language, visual baselines, or CI. Stop and replan if the live workflow needs any of them.

## Handoff

State the cohesive improvements, changed files, before and after evidence, viewport and accessibility checks, focused and repository validation, log result, and remaining concerns. State explicitly when no realistic developer dump was available or when an environment condition prevented a check.
