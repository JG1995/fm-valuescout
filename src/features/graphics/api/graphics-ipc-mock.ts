import type { GraphicsStatus } from "../types/graphics";

export const DEFAULT_GRAPHICS_STATUS: GraphicsStatus = {
  generation: 0,
  selected: false,
  rebuilding: false,
  candidate: { available: false, source: "absent" },
  summary: {
    configs: 0,
    mappings: 0,
    truncated: false,
    diagnostics: {
      configLimit: 0,
      entryLimit: 0,
      depthLimit: 0,
      mappingLimit: 0,
      configTooLarge: 0,
      configUnreadable: 0,
      malformedConfig: 0,
      invalidMapping: 0,
      sourceUnreadable: 0,
    },
  },
};

export type GraphicsStatusIpcMockMode = "ready" | "failed";
export type GraphicsMutationIpcMockMode = "ready" | "failed";
export type GraphicsChooseIpcMockMode = "select" | "cancel";

let status = { ...DEFAULT_GRAPHICS_STATUS };
let statusMode: GraphicsStatusIpcMockMode = "ready";
let mutationMode: GraphicsMutationIpcMockMode = "ready";
let chooseMode: GraphicsChooseIpcMockMode = "select";

export function setGraphicsStatusIpcMockMode(mode: GraphicsStatusIpcMockMode) {
  statusMode = mode;
}

export function setGraphicsMutationIpcMockMode(
  mode: GraphicsMutationIpcMockMode,
) {
  mutationMode = mode;
}

export function setGraphicsChooseIpcMockMode(mode: GraphicsChooseIpcMockMode) {
  chooseMode = mode;
}

export function resetGraphicsIpcMock() {
  status = { ...DEFAULT_GRAPHICS_STATUS };
  statusMode = "ready";
  mutationMode = "ready";
  chooseMode = "select";
}

export function setGraphicsStatusIpcMock(next: GraphicsStatus) {
  status = next;
}

export function resolveGraphicsStatusIpcMock() {
  if (statusMode === "failed") {
    throw new Error("graphics status unavailable");
  }
  return status;
}

export function resolveGraphicsMutationIpcMock(command: string) {
  if (mutationMode === "failed") {
    throw new Error("graphics update unavailable");
  }
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
