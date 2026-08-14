# FM26 BepInEx bridge

C# plugin that runs inside Football Manager 26 (Windows Steam) via BepInEx 6 IL2CPP. It owns memory layouts and dump files. The Tauri Rust backend talks to it through a LocalAppData file protocol ([ADR-0016](../.wiki/decisions/0016-csharp-bepinex-fm26-bridge.md)).

## Bridge data directory (locked)

```text
%LOCALAPPDATA%\fm-valuescout\fm-bridge\
  ├── request.json      ← Tauri writes a dump or closed player-boost request (30s TTL)
  ├── status.json       ← plugin writes on load and bridge-work phases
  ├── dump.json         ← successful CA/PA candidate dump (replace-only-on-success)
  ├── diagnostics.txt   ← every scan attempt (including failures)
  └── force-scan        ← optional manual fallback (empty file; deleted after run)
```

Exact folder names: `fm-valuescout` / `fm-bridge`.

**Dump contract:** frozen schema v6 field list, null rules, and ingestibility checks — [DUMP_SCHEMA.md](./DUMP_SCHEMA.md). Rust validates shape via `validate_dump_json` (see `src-tauri/src/features/memory_read/dump_validation.rs`).

### Memory access (`Memory/`)

Safe in-process reads use `IMemoryReader` + `WindowsMemoryReader` (`ReadProcessMemory` / `VirtualQuery`). The only write seam is internal `IMemoryWriter`, which exposes byte and `u16` writes for the two fixed player actions. Candidate heap regions are committed, private, writable pages under a size cap. `ModuleLocator` records `game_plugin.dll` / `GameAssembly.dll` base/end. Unit tests use `Tests/Fakes/FakeMemoryReader` — no FM required.

### Layouts, identity extraction, and dump

- `Layouts/` — versioned pins keyed by FM major.minor (`26.3`). Unsupported versions fail closed and write diagnostics without touching a prior good `dump.json`.
- `Fm263Layout` ports FMSuperScout’s 26.3 field pins (author permission — see `.wiki/notes/superscout-permission.md`). Confirm identity fields against known players after first live dump; still marked provisional until then.
- `Scanning/PersonScanner` — heap walk via bounded `TryReadBlock` regions (32 MiB blocks, local aligned-word inspection, in-buffer UID, cached vtable→class-offset), Il2Cpp dynamic class offset, UID/CA/PA sanity (`1..200`), UID dedupe. Request-scoped `maxAccepted`: omit/`null` = unlimited (production Load Data default); a positive integer stops after that many accepted players (`DefaultMaxAccepted` = 500 remains the diagnostic/test constant). Cap hits are signaled in `dump.json` / ready `status.json` as `scanTruncated` + `maxAccepted` (see [DUMP_SCHEMA.md](./DUMP_SCHEMA.md)). See [bridge scan performance](../.wiki/features/completed/bridge-scan-performance.md) for live budgets and phase diagnostics.
- `Extraction/` — dedicated readers for nested/indirect UTF-8 names, FM packed DOB, nationality, height, preferred foot (from foot attrs ÷5), natural positions (suitability ≥ max(15, top−2)), attribute groups (visible/hidden ÷5; personality raw 1–20), contract/value/reputation (person→contract pointer; wages and market value in GBP as stored), and club/loan resolution (contract→team→club parent; squad walk for current club with deterministic multi-hit rules; schedule date-votes for in-game date). Contiguous attribute, position, personality, and bounded string ranges are read via `TryReadBlock` into rented buffers and decoded locally; pointer-chain hops stay scalar and nullable. Empty names or impossible DOBs are skipped and counted in diagnostics. Age is computed from DOB against the resolved game date.
- Dump schema **v6** retains the player payload and adds `nationUid`, `gender`, selected-team `clubReputation` / `teamType`, top-level `gameDateBasis` / `playerDatabaseScope`, complete `staff` records, and optional human `manager` metadata. Attribute keys are stable PascalCase names (e.g. `Acceleration`, `Consistency`, `Ambition`); unread values are JSON `null`, never sentinel zero. `DumpWriter` streams compact schema-v6 JSON to a temp file (per-record flush) and atomically replaces `dump.json` only on non-empty success. Unknown FM builds, including an undetectable `game_plugin.dll` version, fail closed with no layout fallback. The exact field, null, and identity rules live in [DUMP_SCHEMA.md](./DUMP_SCHEMA.md).

### In-app request protocol

1. Build and install the plugin; launch FM26 and load a save.
2. In the Tauri app, click **Load Data** in the top bar.
3. Rust writes `request.json` (`protocolVersion`, `requestId`, `createdAtUtc`, `operation: "full-dump"`, optional `maxAccepted`: `null` = unlimited, positive integer = cap).
4. The plugin polls every ~2s, rejects requests older than **30 seconds**, runs the dump off the Unity main thread, and updates `status.json` (`idle` → `scanning` → `ready` / `failed`).
5. The app waits for a terminal status matching the request id (default timeout 120s; unlimited reference save bridge dump measured ~26s on 2026-07-30). On success, Rust reads `dump.json` from disk, validates schema v6, and atomically ingests player, staff, manager, scope, and date-basis data into SQLite for the active app save (`load_data` IPC). The dump body never crosses IPC. Scan or ingest failure leaves the prior snapshot unchanged; failed status errors replace machine-local paths with generic failure text before publication; the UI shows a typed error or ingest summary (player count, truncated banner when `scanTruncated`).

The same protocol v1 file supports two closed player actions after a successful **live** full dump:

- `boost-current-ability` requires the source dump request ID, player UID, expected CA and PA, and a fixed increment of `5` or `10`. The bridge caps the target at live PA and `200`.
- `wonderkid-mentality` requires the same source and expected CA/PA preconditions plus a snapshot expectation for each known mentality field. A `null` field is neither read nor written; a known value above `10` is rechecked and left unchanged. The bridge generates independent inclusive `11..20` targets only for known values at `10` or below.

The bridge retains candidate locations only in memory for the one successful live dump. A failed dump does not replace the previous live index; snapshot-retry candidates, restarts, source-request mismatches, changed UIDs, changed CA/PA, and failed boost operations all fail closed. `status.json` can contain optional `playerBoostsSupported` and `playerBoost` fields with verified scalar results and rollback state; it never contains a UID, address, raw memory, or an arbitrary target supplied by the app. Full-dump consumers remain protocol-v1 compatible because the new request and status fields are optional.

### Manual force-scan fallback

1. With FM26 running and a save loaded, create an empty file:
   `%LOCALAPPDATA%\fm-valuescout\fm-bridge\force-scan`
2. The plugin creates a unique `force-scan-*` request ID (reported in `status.json`), treats it as an unlimited request, then deletes the file. A later manual scan has a different source ID and replaces the prior live candidate index.
3. Inspect `dump.json` / `diagnostics.txt` and `status.json` (`scanning` → `ready` / `failed`).
4. Record player and staff counts, then spot-check representative player fields, staff identity/attributes/contracts, manager club metadata, scope, and date basis against FM. If wrong or empty, use diagnostics (class-offset histogram, sample UIDs, `sampleAttributes`, `sampleContracts`, `sampleClubs`, `multiClubSamples`, identity skip counts, `clubUnresolved`) to adjust `Fm263Layout`.

### Live schema-v6 baseline

One loaded FM 26.3.2 save completed an unlimited Windows **Load Data** run on 2026-08-08. The app showed its success banner. This is a reference run for semantic validation, not a performance target or a result for women's or combined databases.

| Check | Result |
| --- | --- |
| Player database scope | `men` |
| Date source and basis | `derived`; `next-fixture-consensus` |
| Scan cap and truncation | Unlimited (`null`); not truncated |
| Player records | 247,781 dump records and SQLite rows; no duplicate player UIDs |
| Staff records | 134,316 dump records and SQLite rows; no duplicate staff UIDs |
| Manager and club links | Manager metadata present; 237,023 player and 47,154 staff rows have a club value |
| Player/staff overlap | 0 in this run |
| Dump size | 491,761,405 bytes (491.8 MB) |
| App database after ingest | 7,107,915,776 bytes (7.11 GB) |
| Bridge scan | 38.365 s total; selected phases were 0.060 s region enumeration, 21.444 s candidate discovery, 10.458 s extraction, 2.492 s club indexing, and 3.183 s dump writing |
| Ready-to-committed snapshot interval | 55.7 s observed from bridge `ready` to the active snapshot commit; this is not the app-reported `ingestMs` value |

The documented row counts matched the dump declarations and the active app-save snapshot. The app database check used only aggregate counts and did not retain names, addresses, dump contents, or machine paths.

### Hardened scan validation

One unlimited Windows **Load Data** run on 2026-08-08 exercised the final scan-hardening DLL after an FM26 restart. The developer confirmed the visible result was correct. Its aggregate player and staff totals and dump size matched the schema-v6 baseline above; this is one supported-machine observation, not a performance target.

| Check | Result |
| --- | --- |
| Result contract | 247,781 players; 134,316 staff; manager metadata present; `men` scope with derived `next-fixture-consensus` date basis; unlimited, not truncated; no player/staff overlap; the same 491,761,405-byte dump size as the baseline |
| Read quality | 5,018,877,952 / 5,023,731,712 bytes readable (99.9034%); 4,853,760 unread bytes (0.0966%), below the fixed `>10%` failure threshold; no internal read-failure bytes |
| Source and retry | `live`; zero retries; no VA-clone snapshot was needed for this healthy attempt |
| Fixed resource bound | 8 workers with one 32 MiB buffer each (256 MiB maximum scan-buffer allocation) |
| Bridge phases | 0.065 s region enumeration, 6.422 s candidate discovery, 11.188 s extraction, 2.452 s club indexing, and 3.110 s dump writing; 23.618 s total |
| Process-memory I/O | 19,837,038 reads requesting 5,378,910,136 bytes (5.010 GiB) |

The prior-dump and snapshot-preservation paths remain covered by the deterministic bridge and Rust tests. This live run did not manufacture a low-memory or failed-scan condition merely to force a snapshot retry.

The developer also compared a representative player's fields and club link, a staff record's identity/attributes/contract and club, and the human manager with managed club against the PR 1 result. All matched; no names, UIDs, or field values were retained.

## Prerequisites (Windows host)

| Requirement | Notes |
| --- | --- |
| **.NET 6 SDK** | Feature band pinned in [`global.json`](./global.json). Install from [dotnet.microsoft.com](https://dotnet.microsoft.com/download/dotnet/6.0). |
| **Windows host** | Build and run the plugin on Windows for FM attach. Unit tests (`dotnet test`) can run on Linux/WSL with the SDK. |
| **Football Manager 26 (Steam)** | Memory reading targets Windows Steam FM26 only. |
| **BepInEx 6 IL2CPP** | Install into the FM26 game folder per [BepInEx docs](https://docs.bepinex.dev/). Use a known-compatible Unity IL2CPP build (bleeding-edge / `be` line). |

## Build

Plugin host APIs come from the **BepInEx NuGet** feed (`BepInEx.Unity.IL2CPP`). FM Il2CppInterop assemblies stay machine-local and are **not** needed for the status scaffold or the safe memory-reader unit tests.

```powershell
cd bridge
dotnet restore
dotnet build
dotnet test
```

Output DLL: `bin/Debug/net6.0/FmDataBridge.dll` (or `Release`).

### Optional local Interop paths

For later memory-scan commits that reference FM types, copy [`Directory.Build.props.example`](./Directory.Build.props.example) to `Directory.Build.user.props` and set `InteropDir` to your Steam `BepInEx/interop` folder. Keep that file untracked.

```powershell
Copy-Item Directory.Build.props.example Directory.Build.user.props
# Edit InteropDir (and optional BepInExCore) for your Steam install
```

## Manual install and first status check

1. Install BepInEx 6 IL2CPP into the FM26 Steam folder.
2. Launch FM once so BepInEx generates interop under `BepInEx/interop/` (needed for later scan work; status writer does not require those DLLs at build time).
3. Build this project (`dotnet build`).
4. Copy `FmDataBridge.dll` into `Football Manager 26/BepInEx/plugins/`.
5. Launch FM26. Check the BepInEx log for `FM Data Bridge ... loaded`.
6. Confirm `%LOCALAPPDATA%\fm-valuescout\fm-bridge\status.json` exists and looks like:

```json
{
  "protocolVersion": 1,
  "pluginVersion": "0.1.0",
  "state": "idle",
  "updatedAtUtc": "2026-07-28T15:00:00+00:00",
  "gamePluginModulePresent": true,
  "gameAssemblyModulePresent": true
}
```

`gamePluginModulePresent` / `gameAssemblyModulePresent` are cheap process-module checks (no memory scan).

## In-app install (Tauri)

The desktop app can install, update, or remove `FmDataBridge.dll` into the default Steam FM26 `BepInEx/plugins` folder from the **Bridge plugin install** section on the home screen.

1. Ensure BepInEx 6 IL2CPP is already installed in the FM26 Steam folder (the app does **not** install BepInEx itself).
2. Click **Install plugin** (or **Update plugin** when a copy is already present).
3. Restart Football Manager 26 so BepInEx loads the new DLL.
4. Use **Remove plugin** to delete only `FmDataBridge.dll` — other plugins and BepInEx core are left untouched.

Path resolution matches `./scripts/dev bridge-install`: `FM_BRIDGE_PLUGINS`, then `FM_STEAM_ROOT/BepInEx/plugins`, then the default Windows Steam path.

### Permissions and antivirus

Writing a DLL into `Program Files (x86)\Steam\…` may require administrator approval or trigger Windows Defender / third-party antivirus prompts. The app surfaces permission failures as install errors; it does not request elevation in v1. If install fails, use `./scripts/dev bridge-install` from an elevated shell or add a Steam-folder exclusion in your AV product.

### Bundled DLL packaging

Linux CI and local gates use a **placeholder** `src-tauri/resources/FmDataBridge.dll` so Tauri can bundle a file without a Windows `dotnet build`. Do not overwrite that tracked file for a release.

On a Windows host, run:

```bash
./scripts/dev package-windows
```

The command restores and builds the bridge from the checked-out source, verifies its identity, bundles that generated DLL into one unsigned x64 NSIS installer, and writes the installer plus checksum under `.release/windows/<version>/`. It does not publish a release or change the tracked placeholder.

`./scripts/dev bridge-install` remains the developer path that builds from source and copies directly into the game folder for manual in-app plugin testing.

## Interop assemblies (not in git)

On first FM launch with BepInEx installed, BepInEx generates Il2CppInterop assemblies under the game tree (typically `BepInEx/interop/`). Those DLLs are **machine-local**. Do not vendor them, BepInEx core, or `fm.exe` assemblies in this repository.

## Tooling boundary

Linux `./scripts/dev check` does not require the .NET SDK and does not build this tree. Validate the bridge with:

```bash
./scripts/dev bridge-test
cd bridge && dotnet build
```

on a machine with the .NET 6 SDK (Windows for FM attach; Linux/WSL is enough for unit tests). CI runs the bridge unit suite on Windows; full FM attach testing remains manual on Windows.

### Install into FM (WSL → Steam)

From the repo root:

```bash
./scripts/dev bridge-install
```

Builds `FmDataBridge.dll` and copies it to `BepInEx/plugins`. Path resolution:

1. `FM_BRIDGE_PLUGINS` — explicit plugins directory
2. `FM_STEAM_ROOT/BepInEx/plugins` — if `FM_STEAM_ROOT` is set
3. Default WSL Steam path: `/mnt/c/Program Files (x86)/Steam/steamapps/common/Football Manager 26/BepInEx/plugins`

Example override:

```bash
export FM_STEAM_ROOT="/mnt/c/Program Files (x86)/Steam/steamapps/common/Football Manager 26"
./scripts/dev bridge-install
```

Then restart FM26 so BepInEx loads the new DLL.
