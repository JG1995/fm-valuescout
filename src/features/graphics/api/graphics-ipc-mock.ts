import type { GraphicsStatus } from "../types/graphics";

export const DEFAULT_GRAPHICS_STATUS: GraphicsStatus = {
  generation: 0,
  selected: false,
  candidate: { available: false, source: "absent" },
  summary: {
    configs: 0,
    mappings: 0,
    truncated: false,
    diagnostics: {
      configLimit: 10000,
      entryLimit: 1000000,
      depthLimit: 32,
      mappingLimit: 500000,
      configTooLarge: 0,
      configUnreadable: 0,
      malformedConfig: 0,
      invalidMapping: 0,
      sourceUnreadable: 0,
    },
  },
};

export type GraphicsStatusIpcMockMode = "ready" | "failed";
export type GraphicsChooseIpcMockMode = "select" | "cancel";

let status = { ...DEFAULT_GRAPHICS_STATUS };
let statusMode: GraphicsStatusIpcMockMode = "ready";
let chooseMode: GraphicsChooseIpcMockMode = "select";

export function setGraphicsStatusIpcMockMode(mode: GraphicsStatusIpcMockMode) {
  statusMode = mode;
}

export function setGraphicsChooseIpcMockMode(mode: GraphicsChooseIpcMockMode) {
  chooseMode = mode;
}

export function resetGraphicsIpcMock() {
  status = { ...DEFAULT_GRAPHICS_STATUS };
  statusMode = "ready";
  chooseMode = "select";
}
export function resolveGraphicsStatusIpcMock() {
  if (statusMode === "failed") {
    throw new Error("graphics status unavailable");
  }
  return status;
}
export function resolveGraphicsMutationIpcMock(command: string) {
  if (command === "choose_graphics_root" && chooseMode === "cancel") {
    return status;
  }
  status = {
    ...status,
    generation: status.generation + 1,
    selected: command !== "clear_graphics_root",
  };
  return status;
}
