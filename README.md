# FM ValueScout

FM ValueScout is an offline-first desktop companion for Football Manager 26. It reads a running game through a local BepInEx bridge, stores snapshots locally, and provides role-aware player search, profiles, squad planning, and squad optimization.

## Early-alpha status

This is a solo-developer, super-early alpha intended first for the author's own dogfooding. The repository is public, but it is not a supported product or a promise of compatibility with every FM installation. Use it only if you are comfortable testing unsigned software, backing up your own data, and reporting problems with enough context to reproduce them.

FM ValueScout is an unofficial independent project. It is not affiliated with, endorsed by, or supported by Sports Interactive or SEGA.

There is no published installer yet. When the first GitHub prerelease is available, it will be an unsigned Windows x64 installer with a matching SHA-256 file in the release assets. Do not download executables from forks, pull-request artifacts, or third-party mirrors.

## Supported environment and limits

| Area | Supported early-alpha boundary |
| --- | --- |
| Desktop OS | Windows x64 only |
| Game | Steam Football Manager 26.3 on Windows (live validated on 26.3.2) |
| Bridge prerequisite | BepInEx 6 IL2CPP already installed in the FM26 folder |
| Database scope | Men's database only |
| Data refresh | Explicit **Load Data** while FM26 is running with a save loaded |
| Network use | Core use is local; there are no accounts, cloud sync, telemetry, or automatic data upload |

FM patches can change memory layouts. Current Load Data support is limited to the FM26.3 layout; other FM26 builds are unsupported. If a game build is unsupported, the bridge is designed to fail closed rather than guess. The two player-boost actions are especially constrained: they are optional, require a fresh successful live load, and are supported only on the exact live-validated FM26.3.2 build. Back up the FM save before using either action.

The app does not support macOS, Linux, non-Steam FM26, other Football Manager editions, women's or combined databases, automatic updates, BepInEx installation, save-file import, or downgrade/rollback of an upgraded app database.

## Before installing

1. Install BepInEx 6 IL2CPP into the Steam FM26 folder and launch FM once. ValueScout does not install BepInEx for you.
2. Back up your FM save. The boost features can write only their specific guarded values, but they are still experimental.
3. Download the installer and its `.sha256` asset from the matching GitHub prerelease.
4. Verify the checksum before running the installer. In PowerShell:

   ```powershell
   (Get-FileHash .\FM-ValueScout_<version>_x64-setup.exe -Algorithm SHA256).Hash
   ```

   Compare it with the hash in the release's `.sha256` file. Do not continue if they differ.

Windows SmartScreen or antivirus software may warn because the installer is unsigned. That warning is expected for this alpha; it is not a reason to bypass a checksum mismatch or an unexpected publisher prompt.

## Install and first use

Once a prerelease is available:

1. Run its verified Windows x64 installer.
2. Start FM26, load the save you want to inspect, then start FM ValueScout.
3. In the app, use **Install plugin** in the bridge section. This copies only `FmDataBridge.dll` into FM's existing `BepInEx/plugins` directory.
4. Restart FM26 so BepInEx loads the bridge.
5. Return to ValueScout and click **Load Data**. A successful load creates a local snapshot for the active ValueScout save slot.
6. Use Search, profiles, Squad, Planner, and Academy against that current snapshot. Click **Load Data** again after in-game changes when you want a new snapshot.

If plugin installation fails, check that the Steam FM26 directory and BepInEx are correct. Windows may require permission to write under `Program Files (x86)`. The app does not request elevation; use a manual bridge install only if you understand the folder you are targeting. See the [bridge guide](bridge/README.md) for the exact developer and manual paths.

## Update and uninstall

Updates are manual in this alpha:

1. Close FM ValueScout.
2. Copy `%APPDATA%\app.fmvaluescout\` to a safe location before installing the update.
3. Verify and run the newer installer from its GitHub prerelease.
4. Open the app and let its forward-only local database migrations finish.
5. If the bundled bridge changes, use **Update plugin** and restart FM26.

Do not use an older installer after a newer version has migrated the database. Restore a backup instead.

To uninstall, close FM ValueScout and use Windows **Installed apps**. To remove the bridge as well, use **Remove plugin** in the app before uninstalling, or delete only `FmDataBridge.dll` from FM's `BepInEx/plugins` directory. Do not delete BepInEx itself or unrelated plugins.

Windows uninstallation does not replace a data-deletion decision. Only after keeping a backup you no longer need, remove these local folders if you want to erase ValueScout data and diagnostics:

```text
%APPDATA%\app.fmvaluescout\                 app database (`app.db`)
%LOCALAPPDATA%\app.fmvaluescout\logs\      app logs
%LOCALAPPDATA%\fm-valuescout\fm-bridge\    bridge requests, status, dump, and diagnostics
```

## Backup and recovery

ValueScout's SQLite database can be several gigabytes. Back it up only while the app is closed:

1. Close FM ValueScout.
2. Copy the whole `%APPDATA%\app.fmvaluescout\` folder to a location you control.
3. To restore, keep the app closed and replace that folder with the backup.
4. Start the app. If the backup came from a newer app version, reinstall that version rather than attempting a downgrade.

A failed Load Data operation leaves the previous snapshot intact. If an update, migration, or experimental boost leaves you unsure, restore the closed-app backup before doing more work.

## Diagnostics and issue reports

Release builds keep bounded local logs in:

```text
%LOCALAPPDATA%\app.fmvaluescout\logs\fm-valuescout.log
```

The active file is limited to 1 MB; up to three rotated files are retained. The log records app startup and database migration state. Bridge diagnostics are separate under `%LOCALAPPDATA%\fm-valuescout\fm-bridge\`.

For a non-security bug, open a GitHub issue with the app version, Windows version, FM26 build, a short reproduction, and whether the bridge/plugin was installed or updated. Do **not** attach your `app.db`, `dump.json`, full bridge diagnostics, memory addresses, or player data to a public issue. Review and redact logs before sharing them.

For a security problem, follow [SECURITY.md](SECURITY.md) and do not open a public issue.

## Develop from source

For development rather than use of a prerelease, install Node 24, pnpm, Rust, and the platform prerequisites in [Architecture §11](.wiki/ARCHITECTURE.md#11-operational-notes). Then run:

```bash
pnpm install
pnpm exec playwright install chromium
./scripts/dev check
pnpm tauri dev
```

Bridge development requires the .NET 6 SDK; attaching it to FM still requires a Windows host, Steam FM26, and BepInEx 6 IL2CPP. Run `./scripts/dev bridge-test` for bridge unit tests, and run `./scripts/dev package-windows` on native Windows for an unsigned x64 release candidate. See [CONTRIBUTING.md](CONTRIBUTING.md) for the repository workflow and [bridge/README.md](bridge/README.md) for the bridge contract.

## Maintainers and contributors

The release procedure and the boundary between a local Windows candidate and an automatically published prerelease live in the [early-alpha release runbook](.wiki/notes/early-alpha-release-runbook.md). Repository development commands, tests, and architecture are documented in [CONTRIBUTING.md](CONTRIBUTING.md), [AGENTS.md](AGENTS.md), and the [project wiki](.wiki/INDEX.md).

## License

[MIT](LICENSE)
