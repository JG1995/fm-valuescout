import type { CsvMatchPreview } from "@/features/csv-import/types/csv-match-preview";

export type CsvPreviewIpcMockMode = "success" | "error" | "busy";

const DEFAULT_PREVIEW: CsvMatchPreview = {
  format: "youthTracker",
  totalPlayers: 3,
  matchedPlayers: 3,
  unmatchedPlayers: 0,
};

let mode: CsvPreviewIpcMockMode = "success";
let preview = DEFAULT_PREVIEW;
let error: unknown = new Error("CSV format is not supported");
let lastArgs: unknown;
let busyDeferred: {
  promise: Promise<CsvMatchPreview>;
  resolve: (value: CsvMatchPreview) => void;
} | null = null;

export function resetCsvPreviewIpcMock() {
  mode = "success";
  preview = DEFAULT_PREVIEW;
  error = new Error("CSV format is not supported");
  lastArgs = undefined;
  busyDeferred = null;
}

export function setCsvPreviewIpcMockResult(nextPreview: CsvMatchPreview) {
  mode = "success";
  preview = nextPreview;
  busyDeferred = null;
}

export function setCsvPreviewIpcMockError(nextError: unknown) {
  mode = "error";
  error = nextError;
  busyDeferred = null;
}

export function setCsvPreviewIpcMockBusy() {
  mode = "busy";
}

export function resolveBusyCsvPreviewRequest(
  result: CsvMatchPreview = preview,
) {
  busyDeferred?.resolve(result);
  busyDeferred = null;
}

export function getLastCsvPreviewIpcArgs() {
  return lastArgs;
}

export function resolveCsvPreviewIpcMock(args: unknown) {
  lastArgs = args;

  if (mode === "error") {
    return Promise.reject(error);
  }

  if (mode === "busy") {
    if (!busyDeferred) {
      let resolve!: (value: CsvMatchPreview) => void;
      const promise = new Promise<CsvMatchPreview>((resolvePromise) => {
        resolve = resolvePromise;
      });
      busyDeferred = { promise, resolve };
    }
    return busyDeferred.promise;
  }

  return Promise.resolve(preview);
}
