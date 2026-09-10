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
export type GraphicsResultIpcMockMode =
  | "available"
  | "missing"
  | "pending"
  | "error";

type GraphicsResult = import("../types/graphics").GraphicsResult;

let status = { ...DEFAULT_GRAPHICS_STATUS };
let result: GraphicsResult = { status: "missing" };
let resultMode: GraphicsResultIpcMockMode = "missing";
let resultsByCall = new Map<string, GraphicsResult>();

function callKey(args: unknown) {
  const request = args as {
    kind?: string;
    uid?: number;
  };
  return `${request.kind ?? ""}:${request.uid ?? 0}`;
}

function resultForCall(args: unknown) {
  return resultsByCall.get(callKey(args)) ?? result;
}
let pendingResults: Array<{
  args: unknown;
  resolve: (result: GraphicsResult) => void;
}> = [];
let resolveCalls: unknown[] = [];
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
  result = { status: "missing" };
  resultMode = "missing";
  resultsByCall = new Map();
  pendingResults = [];
  resolveCalls = [];
}

export function setGraphicsStatusIpcMock(next: GraphicsStatus) {
  status = next;
}

export function setGraphicsResultIpcMock(next: GraphicsResult) {
  result = next;
  resultMode = "available";
}

export function setGraphicsResultIpcMockForCall(
  kind: string,
  uid: number,
  next: GraphicsResult,
) {
  resultsByCall.set(`${kind}:${uid}`, next);
  resultMode = "available";
}

export function setGraphicsResultIpcMockMode(mode: GraphicsResultIpcMockMode) {
  resultMode = mode;
}

export function resolvePendingGraphicsResultIpcMock() {
  const pending = pendingResults.shift();
  pending?.resolve(resultForCall(pending.args));
}

export function resolveAllPendingGraphicsResultsIpcMock() {
  const pending = pendingResults;
  pendingResults = [];
  for (const deferred of pending) {
    deferred.resolve(resultForCall(deferred.args));
  }
}

export function getPendingGraphicsResultIpcMockCount() {
  return pendingResults.length;
}

export function getGraphicsIpcMockCalls() {
  return resolveCalls;
}

export function resolveGraphicsIpcMock(args?: unknown) {
  resolveCalls = [...resolveCalls, args];
  if (resultMode === "error") {
    throw new Error("graphics result unavailable");
  }
  if (resultMode === "pending") {
    return new Promise<GraphicsResult>((resolve) => {
      pendingResults.push({ args, resolve });
    });
  }
  return resultMode === "available"
    ? resultForCall(args)
    : { status: "missing" };
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
