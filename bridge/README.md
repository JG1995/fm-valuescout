# FM26 BepInEx bridge

C# plugin that runs inside Football Manager 26 (Windows Steam) via BepInEx 6 IL2CPP. It owns memory layouts and dump files. The Tauri Rust backend talks to it through a LocalAppData file protocol ([ADR-0016](../.wiki/decisions/0016-csharp-bepinex-fm26-bridge.md)).

## Bridge data directory (locked)

```text
%LOCALAPPDATA%\fm-valuescout\fm-bridge\
  ├── request.json      ← Tauri writes to request a dump (30s TTL)
  ├── status.json       ← plugin writes on load / scan phases
  ├── dump.json         ← successful CA/PA candidate dump (replace-only-on-success)
  ├── diagnostics.txt   ← every scan attempt (including failures)
  └── force-scan        ← optional manual fallback (empty file; deleted after run)
```

Exact folder names: `fm-valuescout` / `fm-bridge`.

### Memory access (`Memory/`)

Safe in-process reads use `IMemoryReader` + `WindowsMemoryReader` (`ReadProcessMemory` / `VirtualQuery`). Candidate heap regions are committed, private, writable pages under a size cap. `ModuleLocator` records `game_plugin.dll` / `GameAssembly.dll` base/end. Unit tests use `Tests/Fakes/FakeMemoryReader` — no FM required.

### Layouts, identity extraction, and dump

- `Layouts/` — versioned pins keyed by FM major.minor (`26.3`). Unsupported versions fail closed and write diagnostics without touching a prior good `dump.json`.
- `Fm263Layout` ports FMSuperScout’s 26.3 field pins (author permission — see `.wiki/notes/superscout-permission.md`). Confirm identity fields against known players after first live dump; still marked provisional until then.
- `Scanning/PersonScanner` — aligned heap walk, vtable in GameAssembly/game_plugin, Il2Cpp dynamic class offset, UID/CA/PA sanity (`1..200`), UID dedupe. **Temporary testing cap:** stops after `DefaultMaxAccepted` (10 000) accepted players so Load Data finishes quickly; set `maxAccepted: null` for a full walk. Full-scan optimization is tracked in `.wiki/BACKLOG.md` (High).
- `Extraction/` — dedicated readers for nested/indirect UTF-8 names, FM packed DOB, nationality, height, preferred foot (from foot attrs ÷5), natural positions (suitability ≥ max(15, top−2)), attribute groups (visible/hidden ÷5; personality raw 1–20), and contract/value/reputation (person→contract pointer; wages and market value in GBP as stored). Empty names or impossible DOBs are skipped and counted in diagnostics.
- Dump schema **v4** players: `{ uid, ca, pa, name, birthYear, birthDayOfYear, nationalities, heightCm, preferredFoot, positions, attributes, hiddenAttributes, personality, weeklyWageGbp, contractExpiryYear, contractExpiryDayOfYear, transferListed, loanListed, notForSale, setForRelease, marketValueGbp, reputation }`. Attribute keys are stable PascalCase names (e.g. `Acceleration`, `Consistency`, `Ambition`). Unread or out-of-range attribute values are JSON `null` (never `0` as a sentinel). Free agents / missing contract blocks leave wage, expiry, and transfer flags as `null`; money uses `null` for FM unset (`0xFFFFFFFF`) and unfixed market value (`300000000`).

### In-app request protocol

1. Build and install the plugin; launch FM26 and load a save.
2. In the Tauri app, open the home bridge panel and click **Load Data**.
3. Rust writes `request.json` (`protocolVersion`, `requestId`, `createdAtUtc`, `operation: "full-dump"`).
4. The plugin polls every ~2s, rejects requests older than **30 seconds**, runs the dump off the Unity main thread, and updates `status.json` (`idle` → `scanning` → `ready` / `failed`).
5. The app waits for a terminal status matching the request id (default timeout 120s) and shows success or error — it never loads the full dump over IPC.

### Manual force-scan fallback

1. With FM26 running and a save loaded, create an empty file:
   `%LOCALAPPDATA%\fm-valuescout\fm-bridge\force-scan`
2. The plugin treats it like a request (`requestId: force-scan`), then deletes the file.
3. Inspect `dump.json` / `diagnostics.txt` and `status.json` (`scanning` → `ready` / `failed`).
4. Spot-check several known players’ UID/CA/PA plus name, DOB, nationality, height, foot, positions, a few visible/hidden/personality attributes, wage/expiry/transfer flags, market value, and reputation. If wrong or empty, use diagnostics (class-offset histogram, sample UIDs, `sampleAttributes`, `sampleContracts`, identity skip counts) to adjust `Fm263Layout`.

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

## Interop assemblies (not in git)

On first FM launch with BepInEx installed, BepInEx generates Il2CppInterop assemblies under the game tree (typically `BepInEx/interop/`). Those DLLs are **machine-local**. Do not vendor them, BepInEx core, or `fm.exe` assemblies in this repository.

## Tooling boundary

Linux `./scripts/dev check` does not require the .NET SDK and does not build this tree. Validate the bridge with:

```bash
cd bridge
dotnet test
dotnet build
```

on a machine with the .NET 6 SDK (Windows for FM attach; Linux/WSL is enough for unit tests).

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
