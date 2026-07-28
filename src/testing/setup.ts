import "@testing-library/jest-dom/vitest";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";
import {
  resolveBridgeStatusIpcMock,
  resolveBusyDumpRequest,
  resolveDumpRequestIpcMock,
  setBridgeStatusIpcMockMode,
  setDumpRequestIpcMockMode,
} from "@/features/memory-read/api/bridge-status-ipc-mock";

let demoValue = "";

function registerIpcMocks() {
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

    if (cmd === "get_bridge_status") {
      return resolveBridgeStatusIpcMock();
    }

    if (cmd === "request_player_dump") {
      return resolveDumpRequestIpcMock();
    }

    throw new Error(`Unhandled IPC command: ${cmd}`);
  });
}

registerIpcMocks();

afterEach(() => {
  resolveBusyDumpRequest();
  cleanup();
  clearMocks();
  demoValue = "";
  setBridgeStatusIpcMockMode("ready");
  setDumpRequestIpcMockMode("success");
  registerIpcMocks();
});
