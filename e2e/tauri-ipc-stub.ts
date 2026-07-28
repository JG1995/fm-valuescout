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
