# In-app bridge plugin install

## Status

Feature complete — run `/finish-feature` for validation and docs reconciliation

## Intent

Let the Tauri app install, update, and remove `FmDataBridge.dll` in the Steam FM26 `BepInEx/plugins` folder so Load Data no longer depends on a manual copy or `./scripts/dev bridge-install`.

Promoted from [BACKLOG.md](../../BACKLOG.md) Medium item. Breaks from the roadmap sequence (snapshot ingest remains Plan next after this feature).

## User-visible behavior

- Bridge panel shows whether the ValueScout plugin DLL is present at the resolved Steam path.
- User can **Install / Update plugin** — copies the bundled DLL into `BepInEx/plugins`.
- User can **Remove plugin** — deletes only `FmDataBridge.dll` owned by this app.
- Clear failure states when the path is missing, BepInEx is not present, or the write is denied.
- After install, user still restarts FM so BepInEx loads the new DLL (documented in UI copy).

## Invariants

- Windows Steam FM26 only; non-Windows returns `unsupportedPlatform` (same as other bridge commands).
- Default Steam path only for v1 (same assumption as `bridge-install`): `…/Steam/steamapps/common/Football Manager 26/BepInEx/plugins`.
- Never remove or modify BepInEx core, config, or other plugins.
- Never download or unpack BepInEx itself in this feature.
- Frontend never touches arbitrary filesystem paths — all install I/O stays in Rust via IPC.
- Install is explicit (button), not silent on app launch.

## Non-goals

- Installing or removing BepInEx itself
- Multi-library Steam folder discovery or folder picker
- UAC / elevation helper dialogs
- Building the C# plugin from source inside the running app
- Changing the LocalAppData file protocol or dump schema
- Snapshot ingest into SQLite

## Current-state map

- Relevant components: `BridgeStatusPanel`, `memory_read` Rust service/commands, `scripts/dev bridge-install`, `bridge/README.md`
- Data model: none new in SQLite; filesystem only
- Persistence: DLL under Steam `BepInEx/plugins`; source DLL as Tauri bundled resource
- Existing behavioral assumptions: manual install; status comes from `status.json` after FM loads the plugin
- Architectural seams: Rust owns FS; React presents status + triggers; C# plugin unchanged
- Tests: Rust unit tests with temp dirs; Vitest + mockIPC for panel actions
- Primary risks: Program Files write permission; missing BepInEx; shipping a real DLL binary

## Feature architecture (this feature)

```text
React BridgeStatusPanel
  → IPC get_bridge_install_status / install_bridge_plugin / remove_bridge_plugin
      → Rust memory_read install service
          → resolve default Steam plugins path
          → copy bundled FmDataBridge.dll → plugins/
          → delete only FmDataBridge.dll on remove
```

`./scripts/dev bridge-install` remains the developer build-and-copy path (dotnet build). In-app install uses a prebuilt DLL resource, not `dotnet` at runtime.

## Uncertainty register

### Known

- BACKLOG completion criteria and constraints
- `bridge-install` path resolution order and default Steam location
- Workstream 14 notes are aspirational; this feature is a lean subset

### Assumptions

- Author uses default Steam library path (user confirmed)
- BepInEx 6 IL2CPP is already installed in the FM folder
- Copying over an existing `FmDataBridge.dll` is the update path
- Permission failures are acceptable as clear errors (no elevation in v1)

### Decisions

- **DLL-only** — require existing `BepInEx/plugins`; do not bootstrap BepInEx
- **Default path only** — no picker / multi-library scan in v1
- **Bundle resource** — ship `FmDataBridge.dll` via Tauri resources; tests use a fixture file
- **Remove = our DLL only** — never touch BepInEx

### Unknowns

- Whether default Program Files path is writable without elevation on the author’s machine (manual smoke after GREEN)

### Risks

- AV or Windows Defender flags writing a DLL into a game folder
- Resource packaging on Linux CI (must not require Windows-only files for gate) — use a placeholder fixture in repo for tests; real DLL packaging documented for Windows release/dev

## Walking skeleton

Resolve default plugins path → report install status → copy fixture DLL into a temp “plugins” dir in tests → IPC + Install button on the bridge panel.

## Delivery plan

### PR 1 — In-app plugin install and remove

**Status:** Completed — `983c6f2`

**Provisional PR title:** `feat(bridge-install): install and remove FmDataBridge from Steam plugins`

**Purpose:** End-to-end reversible plugin install from the app for default Steam FM26.

**Depends on:** Completed FM26 memory read (bridge panel + protocol already on trunk).

**Merge to trunk when:** Install status, install, and remove work with tests; UI wired; docs updated; gate green.

#### Commit 1 — Resolve Steam plugins path and install status

**Status:** Completed — `ed47e1a`

**Work:** Add Rust helpers to resolve the default Windows Steam FM26 `BepInEx/plugins` path (mirror `bridge-install` defaults, Windows-native). Detect whether `FmDataBridge.dll` is present and whether the plugins directory / BepInEx tree exists. Expose a structured install-status result for IPC. Unit-test path joining and presence checks against temp directories.

**Out of scope for this commit:**
- Copying or deleting the DLL
- IPC registration and UI
- Tauri resource bundling

**Validation:** `cargo test` for path/status helpers; `./scripts/dev check`.

**Provisional commit:** `feat(bridge-install): resolve Steam plugins path and install status`

#### Commit 2 — Install and remove plugin DLL

**Status:** Completed — `7a496bb`

**Work:** Copy the source DLL (resource path in production; injectable path for tests) into the resolved plugins directory (create `plugins` only if BepInEx root exists — fail closed if BepInEx is missing). Remove deletes only `FmDataBridge.dll`. Map permission and missing-path errors to clear variants. Unit-test install/update/remove with temp dirs and a fixture file.

**Out of scope for this commit:**
- UI
- Full BepInEx install
- Elevation

**Validation:** `cargo test` covering success, missing BepInEx, missing source DLL, remove when absent; `./scripts/dev check`.

**Provisional commit:** `feat(bridge-install): copy and remove FmDataBridge.dll`

#### Commit 3 — IPC, bridge panel actions, and docs

**Status:** Completed — `983c6f2`

**Work:** Register Tauri commands; wire React API + bridge panel Install/Update and Remove actions with mockIPC tests; document in-app install in `bridge/README.md` and note AV/permission expectations; move packaging note for the bundled DLL (how Windows release/dev supplies the real binary vs test fixture). Soften manual-only copy language on the panel.

**Out of scope for this commit:**
- Snapshot ingest
- BepInEx bootstrap
- Changing `bridge-install` script behaviour beyond cross-links

**Validation:** Vitest panel tests; `./scripts/dev test`; `./scripts/dev check`.

**Provisional commit:** `feat(bridge-install): add install controls to bridge panel`

## Active work

None — PR 1 complete. Run `/finish-feature`.

## Discoveries and replanning

- Commit 1 landed in `memory_read/install.rs` with env overrides (`FM_BRIDGE_PLUGINS`, `FM_STEAM_ROOT`) matching `bridge-install`; `#![allow(dead_code)]` until commit 3 registers IPC.
- Commit 2 added `install_bridge_plugin_at` / `remove_bridge_plugin_at` plus resolve wrappers; error variants `bepinexMissing`, `sourceMissing`, `writeFailed`, `removeFailed`.
- Commit 3 registered `get_bridge_install_status`, `install_bridge_plugin`, `remove_bridge_plugin`; bundled placeholder at `src-tauri/resources/FmDataBridge.dll`; `BridgePluginInstallSection` sits above bridge status error boundary so install works when `status.json` is missing; `#[cfg_attr(not(windows), allow(dead_code))]` on Windows-only path helpers for Linux CI `-D warnings`.
- Review fix: resolve bundled DLL at `resources/FmDataBridge.dll` (matches `bundle.resources` path); install section wrapped in `BridgePluginInstallError` boundary; install/remove mutations reset each other on success.

## Completed work

| PR 1 | Resolve Steam plugins path and install status | `ed47e1a` | `memory_read/install.rs` — path resolve + `BridgeInstallStatus` |
| PR 1 | Install and remove plugin DLL | `7a496bb` | `install.rs` — copy/remove helpers + 7 new unit tests |
| PR 1 | IPC, bridge panel actions, and docs | `983c6f2` | Tauri commands + `BridgePluginInstallSection` + `bridge/README.md` in-app install |

## Final validation

At feature end: full `./scripts/dev test`, `./scripts/dev check`; manual Windows smoke — Install → restart FM → status.json appears → Remove leaves other plugins alone.

## Documentation impact

- Promote item out of BACKLOG (done at plan time)
- Update ARCHITECTURE / bridge README / completed cross-links at `/finish-feature`
- ADR only if resource-packaging or path policy proves consequential beyond this ledger
