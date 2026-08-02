---
name: workflow-ui-polish
description: Improve the visual design and interaction quality of a running isolated Tauri application through the project’s upstream MCP session. Use only when the developer explicitly invokes `$workflow-ui-polish` or asks to explore a running app and make cohesive UI improvements.
---

# Live UI polish

Use the real application as the design reference. Make purposeful visual and interaction improvements without changing the product contract.

## Preconditions

1. Confirm the developer explicitly requested live UI polish.
2. Confirm a fresh Codex task has loaded the project MCP server and the developer has started `./scripts/dev ui-agent` in another terminal. Use `./scripts/dev ui-agent --dump /absolute/path/dump.json` only when the developer supplied that dump.
3. Start or inspect the upstream `driver_session`, then require its status to report `identifier: app.fmvaluescout` and a `cwd` that matches this repository. Do not use screenshots, JavaScript, interaction, IPC, or other broad control tools until both values match. If another app owns the default port, stop that session and reconnect to the FM ValueScout port reported by the launcher, then verify the metadata again. Do not infer readiness from files, ports, or a previous screenshot.
4. If the session is absent, reloading, disconnected, or targets another app, pause live actions. Report the state and reconnect before taking a fresh DOM snapshot.

Use the tracked golden dump only to prove the workflow mechanics. Do not claim that its one-player layout represents a populated save.

## Live design loop

Read `$ui-design` before making visual or interaction decisions. Use `$coding-standards` and `$minimalism` before editing product files. Existing build, checkpoint, review, and Git workflows govern ordinary workspace changes.

1. Inspect organically. Visit the relevant routes and states that the running application reveals. Capture an initial screenshot and `webview_dom_snapshot` for each meaningful surface. Use `webview_find_element`, `webview_interact`, `webview_keyboard`, and `manage_window` as needed.
2. Identify the highest-value cohesive improvement. Prefer hierarchy, spacing, scanning, affordance, responsive fit, or interaction clarity. Keep product behavior, feature boundaries, and Rust-owned data ownership unchanged.
3. Capture a clear before image under `.work/ui-agent/` when the upstream tool supports a file path. Otherwise retain the MCP image response as task evidence.
4. Make one cohesive edit batch. Do not turn open-ended exploration into a scenario suite, visual baseline, custom control layer, or speculative UI framework.
5. Reinspect after hot reload. Wait for the actual updated UI. If Vite reloads or the app restarts, check `driver_session`, reconnect when needed, and take a new DOM snapshot before interacting again.
6. Capture an after image and compare it to the before image for the intended improvement and regressions.

## Required checks

For each completed polish batch:

- Resize the real window to 1280×800 and 1600×900 with `manage_window`. Check clipping, overflow, density, and layout hierarchy at both sizes.
- Use `webview_keyboard` and DOM/accessibility snapshots to check keyboard reachability, visible focus, and sensible focus movement.
- Inspect empty, loading, error, or constrained states that are reachable from the live session without fabricating data or changing product behavior.
- Read frontend output with `read_logs` and Rust startup or migration output in the launcher terminal. Investigate unexpected errors before presenting the batch as complete.
- Run focused tests for changed code, then `./scripts/dev check` and `./scripts/dev smoke`. Report unavailable or environment-blocked checks truthfully.

`webview_execute_js` is a trusted local inspection tool, not general permission. In upstream version 0.12.0, `ipc_execute_command` does not dispatch application-defined commands. If a read-only product IPC check is necessary, use `window.__TAURI__.core.invoke(...)` through `webview_execute_js`; do not use it to mutate product state unless the developer explicitly authorizes that action.

## Authority limits

- Do not change product behavior, architecture, feature-to-feature imports, or data ownership without a separate developer request and the applicable workflow.
- Do not access Football Manager, manage plugins, confirm external destructive actions, use a live database, or synthesize data.
- Do not commit, push, amend, rebase, squash, or start another workflow automatically. Follow the existing Git approval rules after any workspace edits.
- Do not add MCP tools, command modes, a wrapper, a scenario language, visual baselines, or CI. Stop and replan if the live workflow needs any of them.

## Handoff

State the cohesive improvements, changed files, before and after evidence, viewport and accessibility checks, focused and repository validation, log result, and remaining concerns. State explicitly when no realistic developer dump was available or when an environment condition prevented a check.
