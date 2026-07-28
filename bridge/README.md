# FM26 BepInEx bridge

C# plugin that runs inside Football Manager 26 (Windows Steam) via BepInEx 6 IL2CPP. It owns memory layouts and dump files. The Tauri Rust backend talks to it through a LocalAppData file protocol ([ADR-0016](../.wiki/decisions/0016-csharp-bepinex-fm26-bridge.md)).

The plugin project lands in a later commit. This directory starts with toolchain docs and path overrides only.

## Prerequisites (Windows host)

| Requirement | Notes |
| --- | --- |
| **.NET 6 SDK** | Feature band pinned in [`global.json`](./global.json). Install from [dotnet.microsoft.com](https://dotnet.microsoft.com/download/dotnet/6.0). |
| **Windows host** | Build and run the plugin on Windows. Day-to-day app work can stay on WSL; `dotnet build` for the bridge does not run in Linux CI. |
| **Football Manager 26 (Steam)** | Memory reading targets Windows Steam FM26 only. |
| **BepInEx 6 IL2CPP** | Install into the FM26 game folder per [BepInEx docs](https://docs.bepinex.dev/). Use a known-compatible IL2CPP build for Unity games. |

## Interop assemblies (not in git)

On first FM launch with BepInEx installed, BepInEx generates Il2CppInterop assemblies under the game tree (typically `BepInEx/interop/`). Those DLLs are **machine-local**. Do not vendor them, BepInEx core, or `fm.exe` assemblies in this repository.

## Local path overrides

1. Copy [`Directory.Build.props.example`](./Directory.Build.props.example) to `Directory.Build.user.props` in this folder (or set the same properties in a user props file MSBuild will load).
2. Point `BepInExCore` and `InteropDir` at your Steam FM26 install.
3. Keep `Directory.Build.user.props` untracked — it is gitignored.

```powershell
Copy-Item Directory.Build.props.example Directory.Build.user.props
# Edit Directory.Build.user.props with your Steam paths, then:
dotnet build
```

`dotnet build` succeeds only after the plugin project exists (next commit) and local paths are set.

## Manual install (preview)

Full copy-DLL / first-launch steps land with the plugin scaffold. Until then: install BepInEx into FM26, confirm interop generation on first launch, then return here when the project files exist.

## Tooling boundary

Linux `./scripts/dev check` does not require the .NET SDK and does not build this tree. Validate the bridge with `dotnet build` / `dotnet test` on a Windows machine that has local props configured.
