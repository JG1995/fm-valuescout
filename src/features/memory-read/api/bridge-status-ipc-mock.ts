import type { BridgeStatus } from "../types/bridge-status";

export type BridgeStatusIpcMockMode =
  | "ready"
  | "missing"
  | "unsupportedPlatform"
  | "unsupportedVersion"
  | "corrupt";

const READY_STATUS: BridgeStatus = {
  protocolVersion: 1,
  pluginVersion: "0.1.0",
  state: "idle",
  updatedAtUtc: "2026-07-28T15:00:00+00:00",
  gamePluginModulePresent: true,
  gameAssemblyModulePresent: true,
};

let mockMode: BridgeStatusIpcMockMode = "ready";

export function setBridgeStatusIpcMockMode(mode: BridgeStatusIpcMockMode) {
  mockMode = mode;
}

export function getBridgeStatusIpcMockMode() {
  return mockMode;
}

export function resolveBridgeStatusIpcMock() {
  if (mockMode === "ready") {
    return READY_STATUS;
  }

  const errors: Record<
    Exclude<BridgeStatusIpcMockMode, "ready">,
    { kind: string; message: string }
  > = {
    missing: {
      kind: "missing",
      message: "status.json not found",
    },
    unsupportedPlatform: {
      kind: "unsupportedPlatform",
      message: "FM26 memory read requires Windows",
    },
    unsupportedVersion: {
      kind: "unsupportedVersion",
      message: "unsupported bridge protocol version 99; expected 1",
    },
    corrupt: {
      kind: "corrupt",
      message: "status.json is not valid JSON",
    },
  };

  throw errors[mockMode];
}
