---
name: use-tauri-mcp
description: Use when working with, debugging, or testing a Tauri application.
alwaysApply: true
---

# Tauri MCP Usage Rule

## When to Use
- Debugging a Tauri app (frontend or backend)
- Interacting with Tauri app UI elements
- Inspecting DOM, taking screenshots, or verifying UI state
- Monitoring or debugging IPC calls between frontend and Rust backend
- Reading logs (console, system, android, ios)
- Managing Tauri windows

## Required Workflow
1. Start a `driver_session` before any UI automation or inspection.
2. Use `mcp__tauri_webview_dom_snapshot` for DOM inspection instead of generic browser tools.
3. Use `mcp__tauri_webview_screenshot` for visual verification.
4. Use `mcp__tauri_ipc_monitor` and `ipc_get_captured` to debug frontend-backend communication.
5. Use `mcp__tauri_read_logs` for log analysis (console, system, android, ios).
6. Use `mcp__tauri_webview_interact` and `webview_keyboard` for UI actions.
7. Use `mcp__tauri_manage_window` for window operations.

## Forbidden Alternatives
- **NEVER** use `puppeteer` for Tauri app interaction when Tauri MCP is available.
- **NEVER** use manual shell commands to interact with Tauri processes when MCP tools exist.
- **NEVER** inspect backend state via manual log parsing when `ipc_get_backend_state` is available.

## Fallback Policy
If no Tauri app session is running and a driver_session cannot be established, fall back and explicitly state why the Tauri MCP tools were not used.
