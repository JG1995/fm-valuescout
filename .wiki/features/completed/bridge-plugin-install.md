# In-app bridge plugin install

## Intent

Let the Tauri app install, update, and remove `FmDataBridge.dll` in the default Steam FM26 `BepInEx/plugins` folder so Load Data no longer depends on a manual copy or `./scripts/dev bridge-install`. Promoted from [BACKLOG.md](../../BACKLOG.md) Medium item; delivered off the approved roadmap sequence.

## Delivered behavior

- **Bridge plugin install** section on the home screen reports whether `FmDataBridge.dll` is present at the resolved Steam plugins path.
- **Install plugin** / **Update plugin** copies the bundled DLL into `BepInEx/plugins` (creates `plugins` only when BepInEx root exists).
- **Remove plugin** deletes only `FmDataBridge.dll` — never BepInEx core, config, or other plugins.
- Clear failure states for missing path, missing BepInEx, permission denied, or missing bundled source DLL.
- User must restart FM after install so BepInEx loads the new DLL (documented in UI copy).
- Install section sits above the bridge status error boundary so install works when `status.json` is absent.
- Non-Windows hosts return `unsupportedPlatform` (same as other bridge commands).

## Final architecture

```text
React BridgePluginInstallSection (features/memory-read)
  → IPC get_bridge_install_status / install_bridge_plugin / remove_bridge_plugin
      → Rust features/memory_read/install.rs
          → resolve default Steam FM26 BepInEx/plugins path
             (FM_BRIDGE_PLUGINS → FM_STEAM_ROOT/BepInEx/plugins → default Windows Steam path)
          → copy src-tauri/resources/FmDataBridge.dll → plugins/
          → remove only FmDataBridge.dll
```

- Frontend never touches arbitrary filesystem paths — all install I/O stays in Rust via IPC.
- `./scripts/dev bridge-install` remains the developer build-and-copy path (`dotnet build` from source). In-app install uses a prebuilt bundled resource, not `dotnet` at runtime.
- Linux CI gates use a placeholder `src-tauri/resources/FmDataBridge.dll`; Windows release/dev must replace it with a real build (see [bridge/README.md](../../../bridge/README.md)).

## Important decisions

- **DLL-only** — require existing `BepInEx/plugins`; do not bootstrap BepInEx in-app.
- **Default Steam path only** — no folder picker or multi-library scan in v1.
- **Bundle resource** — ship `FmDataBridge.dll` via Tauri `bundle.resources`; tests use injectable paths and a fixture file.
- **Remove = our DLL only** — never touch BepInEx or other plugins.
- **Explicit install** — button-triggered; not silent on app launch.
- No ADR — packaging and path policy are covered here and in [bridge/README.md](../../../bridge/README.md).

## Migration and operational implications

- End users with default Steam FM26 + BepInEx 6 IL2CPP can install from the app. Developers still use `./scripts/dev bridge-install` or manual copy when building from source.
- Writing into `Program Files (x86)\Steam\…` may require elevation or AV approval; the app surfaces permission errors and does not request UAC in v1.
- Replace the placeholder bundled DLL before Windows release builds or real in-app install testing.

## Validation

- **Automated:** `cargo test` for path resolve, install status, copy/remove (temp dirs + fixture); Vitest + mockIPC for install section and panel wiring; `./scripts/dev test` and `./scripts/dev check` green on Linux CI.
- **Manual (Windows):** Install → restart FM → `status.json` appears → Remove leaves other plugins untouched.

**Delivery commits:** `ed47e1a` (path + status), `7a496bb` (copy/remove), `983c6f2` (IPC, UI, intrinsic docs).

## Follow-up

- **Next feature:** Snapshot ingest + Load Data (order 2 in [TODO.md](../../TODO.md)).
- **Still deferred:** in-app BepInEx bootstrap, multi-library Steam discovery, folder picker, UAC elevation — see original non-goals in Git history for this feature.
