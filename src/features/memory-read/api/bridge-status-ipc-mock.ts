import type { BridgeStatus, DumpRequestResult } from "../types/bridge-status";

export type BridgeStatusIpcMockMode =
  | "ready"
  | "missing"
  | "unsupportedPlatform"
  | "unsupportedVersion"
  | "corrupt";

export type DumpRequestIpcMockMode =
  | "success"
  | "truncatedSuccess"
  | "failed"
  | "timeout"
  | "busy";

const READY_STATUS: BridgeStatus = {
  protocolVersion: 1,
  pluginVersion: "0.1.0",
  state: "idle",
  updatedAtUtc: "2026-07-28T15:00:00+00:00",
  gamePluginModulePresent: true,
  gameAssemblyModulePresent: true,
};

let mockMode: BridgeStatusIpcMockMode = "ready";
let dumpRequestMode: DumpRequestIpcMockMode = "success";
let busyDeferred: {
  promise: Promise<DumpRequestResult>;
  resolve: (value: DumpRequestResult) => void;
} | null = null;

export function setBridgeStatusIpcMockMode(mode: BridgeStatusIpcMockMode) {
  mockMode = mode;
}

export function getBridgeStatusIpcMockMode() {
  return mockMode;
}

export function setDumpRequestIpcMockMode(mode: DumpRequestIpcMockMode) {
  dumpRequestMode = mode;
  if (mode !== "busy") {
    busyDeferred = null;
  }
}

/** Resolves an in-flight busy dump request so tests can settle without afterEach races. */
export function resolveBusyDumpRequest(result?: DumpRequestResult) {
  busyDeferred?.resolve(
    result ?? {
      requestId: "req-mock",
      state: "ready",
      playersFound: 12,
      dumpPresent: true,
      error: null,
    },
  );
  busyDeferred = null;
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

export function resolveDumpRequestIpcMock(): Promise<DumpRequestResult> {
  if (dumpRequestMode === "busy") {
    if (!busyDeferred) {
      let resolve!: (value: DumpRequestResult) => void;
      const promise = new Promise<DumpRequestResult>((res) => {
        resolve = res;
      });
      busyDeferred = { promise, resolve };
    }
    return busyDeferred.promise;
  }

  if (dumpRequestMode === "timeout") {
    return Promise.reject({
      kind: "timeout",
      message: "timed out waiting for dump request req-mock",
    });
  }

  if (dumpRequestMode === "failed") {
    return Promise.resolve({
      requestId: "req-mock",
      state: "failed",
      playersFound: null,
      dumpPresent: false,
      error: "scan produced zero player candidates",
    });
  }

  if (dumpRequestMode === "truncatedSuccess") {
    return Promise.resolve({
      requestId: "req-mock",
      state: "ready",
      playersFound: 10_000,
      dumpPresent: true,
      error: null,
      scanTruncated: true,
      maxAccepted: 10_000,
    });
  }

  return Promise.resolve({
    requestId: "req-mock",
    state: "ready",
    playersFound: 12,
    dumpPresent: true,
    error: null,
  });
}
