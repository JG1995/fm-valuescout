# 0016 — C# BepInEx bridge for FM26 memory read

## Status

Accepted

## Context

FM ValueScout’s MVP needs live player data from a running **Football Manager 26** session on **Windows (Steam)**. The product is a Tauri app with a Rust backend ([0014](./0014-rust-backend-ipc-boundary.md)). Memory layout discovery and in-process scanning are unproven in this repository, but a public companion tool ([FMSuperScout](https://github.com/mavarobli/FMSuperScout)) shows a working pattern: a **BepInEx 6 IL2CPP** plugin in C# that reads FM memory safely and exchanges files with a desktop client.

We needed a durable choice between staying Rust-only for extraction and adopting a small C# plugin alongside Rust.

## Decision

Use a **C# BepInEx 6 IL2CPP plugin** (`bridge/`) as the FM memory reader. Use **Rust** in the Tauri backend as the protocol client: resolve the bridge data directory, write scan requests, read status/dump metadata, and (in a later feature) import dumps into SQLite.

Boundaries:

- The plugin owns versioned memory layouts, safe `ReadProcessMemory` scanning, and writing `status` / `dump` / diagnostics under LocalAppData.
- Rust owns IPC to the WebView, path resolution, request/status orchestration, and dump validation for ingest — not offset tables in the frontend.
- Memory reading targets **Windows Steam FM26 only** for now.
- Scans are triggered **from the app** (file protocol), not by an in-game hotkey in the first delivery.
- Do **not** vendor BepInEx or FM interop assemblies in git; use local path overrides.
- Do **not** ship SuperScout’s DLL as our product binary. Layout offsets and algorithms may be ported into our `bridge/` structure: the SuperScout author ([mark17072](https://github.com/mavarobli)) granted explicit permission (“do whatever you like”) — see [superscout-permission.md](../notes/superscout-permission.md). Prefer independent module layout; record provenance when pins stabilize.

Delivery record: [features/completed/fm26-memory-read.md](../features/completed/fm26-memory-read.md).

## Alternatives considered

### Pure Rust external process reader

Open `fm.exe` from outside and scan with `ReadProcessMemory` / `VirtualQuery` only in Rust. Keeps one language and avoids BepInEx. Rejected for the first path: unproven here, harder attach/trigger story, and we already know the in-process BepInEx approach works on FM26. Revisit only if the C# bridge becomes untenable.

### Rust code loaded inside FM

BepInEx’s supported plugin surface for this game stack is .NET/IL2CPP. A native Rust in-process plugin would fight the mod loader rather than use it. Rejected.

### Save-file parsing instead of memory read

CONCEPT explicitly keeps save-file import out of MVP. Rejected for this decision (may return as a later product option).

## Consequences

### Positive

- Follows a proven FM26 extraction shape (plugin + file protocol + versioned pins).
- Keeps large dumps on disk; Rust/IPC stay bounded ([0014](./0014-rust-backend-ipc-boundary.md)).
- Clear split: C# for game-process work, Rust for app orchestration and later SQLite.

### Negative

- Second language and toolchain (.NET 6 SDK, Windows host for plugin builds, BepInEx on the Steam install).
- Linux CI cannot meaningfully run FM attach tests; bridge unit tests use fakes; full verification is manual on Windows.
- Patch churn still requires version fail-closed behaviour and occasional offset repins.

### Follow-up

- Implemented per [fm26-memory-read](../features/completed/fm26-memory-read.md) (toolchain → status → dump fields → schema freeze).
- [ARCHITECTURE.md](../ARCHITECTURE.md) current-state sections describe the bridge boundary and file protocol.
- In-app DLL install delivered in [bridge-plugin-install](../features/completed/bridge-plugin-install.md). In-app BepInEx bootstrap remains deferred.
- Optional later: Windows CI `dotnet test` only if tests run without machine-local FM interop.

## Related work

- Feature record: [fm26-memory-read](../features/completed/fm26-memory-read.md)
- Related notes: [superscout-permission.md](../notes/superscout-permission.md)
- Planning notes: [notes/memory-read-initial-notes.md](../notes/memory-read-initial-notes.md)
- Rust IPC boundary: [0014](./0014-rust-backend-ipc-boundary.md)
- Tauri shell: [0013](./0013-tauri-v2-desktop-shell.md)
- Supersedes: none
