import type { CsvImportSummary } from "@/features/csv-import/types/csv-import-summary";

export type CsvImportIpcMockMode = "success" | "error" | "busy";

const DEFAULT_IMPORT: CsvImportSummary = {
  format: "youthTracker",
  totalPlayers: 3,
  storedPlayers: 3,
  skippedPlayers: 0,
};

let mode: CsvImportIpcMockMode = "success";
let summary = DEFAULT_IMPORT;
let error: unknown = new Error("CSV format is not supported");
let lastArgs: unknown;
let busyDeferred: {
  promise: Promise<CsvImportSummary>;
  resolve: (value: CsvImportSummary) => void;
} | null = null;

export function resetCsvImportIpcMock() {
  mode = "success";
  summary = DEFAULT_IMPORT;
  error = new Error("CSV format is not supported");
  lastArgs = undefined;
  busyDeferred = null;
}

export function setCsvImportIpcMockResult(nextSummary: CsvImportSummary) {
  mode = "success";
  summary = nextSummary;
  busyDeferred = null;
}

export function setCsvImportIpcMockError(nextError: unknown) {
  mode = "error";
  error = nextError;
  busyDeferred = null;
}

export function setCsvImportIpcMockBusy() {
  mode = "busy";
}

export function resolveBusyCsvImportRequest(
  result: CsvImportSummary = summary,
) {
  busyDeferred?.resolve(result);
  busyDeferred = null;
}

export function getLastCsvImportIpcArgs() {
  return lastArgs;
}

export function resolveCsvImportIpcMock(args: unknown) {
  lastArgs = args;

  if (mode === "error") {
    return Promise.reject(error);
  }

  if (mode === "busy") {
    if (!busyDeferred) {
      let resolve!: (value: CsvImportSummary) => void;
      const promise = new Promise<CsvImportSummary>((resolvePromise) => {
        resolve = resolvePromise;
      });
      busyDeferred = { promise, resolve };
    }
    return busyDeferred.promise;
  }

  return Promise.resolve(summary);
}
