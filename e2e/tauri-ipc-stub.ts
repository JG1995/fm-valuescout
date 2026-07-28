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
