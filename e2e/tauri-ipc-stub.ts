import type { Page } from "@playwright/test";

export async function stubTauriIpc(page: Page) {
  await page.addInitScript({
    content: `
      let demoValue = "";

      window.__TAURI_INTERNALS__ = {
        invoke: async (cmd, args) => {
          if (cmd === "get_status") {
            return { status: "ok" };
          }

          if (cmd === "get_demo_value") {
            return { value: demoValue };
          }

          if (cmd === "set_demo_value") {
            demoValue = args?.value ?? "";
            return { value: demoValue };
          }

          if (cmd === "get_bridge_status") {
            return {
              protocolVersion: 1,
              pluginVersion: "0.1.0",
              state: "idle",
              updatedAtUtc: "2026-07-28T15:00:00+00:00",
              gamePluginModulePresent: true,
              gameAssemblyModulePresent: true,
            };
          }

          if (cmd === "request_player_dump") {
            return {
              requestId: "req-smoke",
              state: "ready",
              playersFound: 0,
              dumpPresent: true,
              error: null,
            };
          }

          if (cmd === "list_saves") {
            return [
              {
                id: 1,
                name: "Default save",
                isActive: true,
                createdAtUtc: "2026-07-28T12:00:00.000Z",
                updatedAtUtc: "2026-07-28T12:00:00.000Z",
              },
            ];
          }

          if (cmd === "create_save") {
            return {
              id: 2,
              name: args?.name ?? "New save",
              isActive: false,
              createdAtUtc: "2026-07-28T16:00:00.000Z",
              updatedAtUtc: "2026-07-28T16:00:00.000Z",
            };
          }

          if (cmd === "rename_save") {
            return {
              id: args?.saveId ?? 1,
              name: args?.name ?? "Renamed save",
              isActive: true,
              createdAtUtc: "2026-07-28T12:00:00.000Z",
              updatedAtUtc: "2026-07-28T16:05:00.000Z",
            };
          }

          if (cmd === "set_active_save") {
            return {
              id: args?.saveId ?? 1,
              name: "Default save",
              isActive: true,
              createdAtUtc: "2026-07-28T12:00:00.000Z",
              updatedAtUtc: "2026-07-28T16:10:00.000Z",
            };
          }

          if (cmd === "get_current_snapshot") {
            return null;
          }

          if (cmd === "list_sanity_players") {
            return [];
          }

          if (cmd === "load_data") {
            return {
              requestId: "req-smoke",
              playersFound: 0,
              scanTruncated: false,
              maxAccepted: null,
              timings: { scanMs: 0, ingestMs: 0, totalMs: 0 },
              snapshot: {
                id: 1,
                saveId: 1,
                schemaVersion: 5,
                generatedAtUtc: "2026-07-28T15:00:00.000Z",
                gameVersion: "26.0.0",
                supportedGameVersion: "26.0.0",
                bridgeVersion: "0.1.0",
                protocolVersion: 1,
                gameDate: null,
                gameDateSource: "unknown",
                scanTruncated: false,
                maxAccepted: null,
                playerCount: 0,
                loadedAtUtc: "2026-07-28T15:05:00.000Z",
              },
            };
          }

          if (cmd === "get_bridge_install_status") {
            return {
              pluginsPath:
                "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\Football Manager 26\\\\BepInEx\\\\plugins",
              pluginPresent: false,
              bepinexPresent: true,
              pluginsDirPresent: true,
            };
          }

          if (cmd === "install_bridge_plugin") {
            return {
              pluginsPath:
                "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\Football Manager 26\\\\BepInEx\\\\plugins",
              pluginPresent: true,
              bepinexPresent: true,
              pluginsDirPresent: true,
            };
          }

          if (cmd === "remove_bridge_plugin") {
            return {
              pluginsPath:
                "C:\\\\Program Files (x86)\\\\Steam\\\\steamapps\\\\common\\\\Football Manager 26\\\\BepInEx\\\\plugins",
              pluginPresent: false,
              bepinexPresent: true,
              pluginsDirPresent: true,
            };
          }

          throw new Error("Unhandled IPC: " + cmd);
        },
        transformCallback: (callback) => callback,
        convertFileSrc: (filePath) => filePath,
        metadata: {
          currentWindow: { label: "main" },
          currentWebview: { windowLabel: "main", label: "main" },
        },
      };
    `,
  });
}
