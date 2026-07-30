# FM26 memory read

## Intent

Extract a full CONCEPT MVP player dump from a running Football Manager 26 (Windows Steam) session via a BepInEx IL2CPP C# bridge, orchestrated by the Tauri Rust backend through a file protocol. This is the live-data foundation for snapshot ingest and every later MVP feature.

Planning input (not authority): [memory-read-initial-notes.md](../../notes/memory-read-initial-notes.md).

## Delivered behavior

- With FM26 running, a save loaded, and the bridge plugin installed, the user triggers a data scan from the app top bar (**Load Data**). There is no in-game hotkey.
- The app shows bridge readiness (plugin present, FM modules, scan phase) and surfaces structured status-read errors: missing bridge, unsupported platform, corrupt status, or unsupported bridge protocol (`unsupportedVersion` when `status.json` uses an unknown `protocolVersion`). Unsupported or undetectable FM builds fail closed at scan time (`status` `failed` with an error message — not the `unsupportedVersion` kind).
- A successful scan writes `dump.json` under `%LOCALAPPDATA%\fm-valuescout\fm-bridge\` with the CONCEPT MVP player field set at dump schema **v5** (frozen in [bridge/DUMP_SCHEMA.md](../../../bridge/DUMP_SCHEMA.md)).
- Unknown or unsupported FM builds **fail closed** — no layout fallback. A failed scan does not replace a prior good dump (replace-only-on-success in the bridge).
- `scanTruncated` and `maxAccepted` on dump and ready `status.json` signal when the person scanner stopped at a request-scoped cap. Production Load Data requests `maxAccepted: null` (unlimited). Ingest must not treat truncated dumps as a complete world database.
- Plugin install at delivery: manual copy into `BepInEx/plugins` or `./scripts/dev bridge-install` from WSL. Superseded by in-app **Install / Update / Remove** in [bridge-plugin-install](./bridge-plugin-install.md); developers building from source still use `bridge-install` or manual copy.

## Final architecture

```text
FM26 (Windows, Steam) + BepInEx 6 IL2CPP
  └── bridge/ (C#) — status, request poll, safe memory scan, dump + diagnostics

%LOCALAPPDATA%\fm-valuescout\fm-bridge\
  ├── request.json      ← Rust writes (30s TTL)
  ├── status.json       ← bridge writes
  ├── dump.json         ← bridge writes (replace-only-on-success)
  ├── diagnostics.txt   ← bridge writes
  └── force-scan        ← optional manual fallback

Tauri
  ├── Rust features/memory_read — paths, get_bridge_status, request_player_dump,
  │   dump validation (validate_dump_json / validate_dump_at_bridge_directory)
  └── React features/memory-read — BridgeStatusPanel (status only); Load Data in AppTopBar
```

- Memory layouts and offset pins live in `bridge/Layouts/` (versioned; initial pin `Fm263Layout` for FM 26.3.x).
- Large dumps stay on disk — never shipped over IPC.
- Windows Steam FM26 only for memory reading; non-Windows returns `unsupportedPlatform`.

**Intentional gaps (not in v5):** asking price, managed club, loan direction relative to the human manager, contract-seed-only club walk (clubs with zero contracted players in the accepted set are invisible), staff and non-player records. See [DUMP_SCHEMA.md](../../../bridge/DUMP_SCHEMA.md) § Intentional gaps.

## Important decisions

- [ADR-0016 — C# BepInEx bridge for FM26 memory read](../../decisions/0016-csharp-bepinex-fm26-bridge.md)
- C# bridge + Rust file protocol (not Rust-only external reader)
- In-app trigger only; in-app DLL install added in [bridge-plugin-install](./bridge-plugin-install.md)
- SuperScout layout pins reimplemented independently with author permission ([superscout-permission.md](../../notes/superscout-permission.md))

## Migration and operational implications

- Developers need a Windows host with .NET 6 SDK, BepInEx 6 IL2CPP on the Steam FM26 install, and the plugin DLL in `BepInEx/plugins`. See [bridge/README.md](../../../bridge/README.md).
- Linux CI runs `./scripts/dev check` and `./scripts/dev test` (Rust protocol/validation, Vitest, Playwright stubs). Bridge `dotnet test` with fakes runs locally on a machine with .NET 6 — not on Linux CI. Full FM attach verification is manual on Windows.
- Snapshot ingest hard-validates via `validate_dump_json` inside `ingest_dump_file_for_save` ([snapshot-ingest](./snapshot-ingest.md)). Scan-path `validate_dump_at_bridge_directory` in `request_player_dump` logs warnings only — it does not gate ingest.
- Full unlimited scans are slow on large saves (~3m+ observed); a 500-player cap keeps Load Data testable while [bridge scan performance](../active/bridge-scan-performance.md) is active.

## Validation

- **Manual:** install plugin, load save, trigger scan, spot-check CONCEPT fields vs FM (known players, loans, non-ASCII names, contracts, clubs).
- **Automated:** Linux CI (`./scripts/dev check`, `./scripts/dev test`): `cargo test` (status fixtures, request watch, dump validation), Vitest + mockIPC, Playwright stub for bridge IPC. Bridge `dotnet test` (fakes) is local only — not in Linux CI.
- No SQLite player schema landed in this feature.

**Delivery commits (final hashes):** `31b7670` … `743dcb4` (see Git history on `main` for the full sequence).

## Follow-up

- **Delivered downstream:** [Snapshot ingest + Load Data](./snapshot-ingest.md) — persist dumps to SQLite; Load Data scan+ingest wired.
- **Active:** [bridge scan performance](../active/bridge-scan-performance.md).
- **BACKLOG:** in-app BepInEx bootstrap (Medium — see [BACKLOG.md](../../BACKLOG.md)).
- **Repin:** FM patches may require layout updates and fail-closed version checks until repinned.
