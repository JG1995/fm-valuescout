import "@testing-library/jest-dom/vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

let demoValue = "";

function registerHealthIpcMock() {
  mockIPC((cmd, args) => {
    if (cmd === "get_status") {
      return { status: "ok" };
    }

    if (cmd === "get_demo_value") {
      return { value: demoValue };
    }

    if (cmd === "set_demo_value") {
      const nextValue =
        typeof args === "object" &&
        args !== null &&
        "value" in args &&
        typeof args.value === "string"
          ? args.value
          : "";
      demoValue = nextValue;
      return { value: demoValue };
    }

    throw new Error(`Unhandled IPC command: ${cmd}`);
  });
}

registerHealthIpcMock();

afterEach(() => {
  cleanup();
  clearMocks();
  demoValue = "";
  registerHealthIpcMock();
});
