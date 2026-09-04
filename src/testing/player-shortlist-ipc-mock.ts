import type { PlayerShortlistImportSummary } from "@/features/search/types/player-shortlist-import-summary";

export type PlayerShortlistImportIpcMockMode = "success" | "error";

const DEFAULT_SUMMARY: PlayerShortlistImportSummary = {
  totalPlayers: 2,
  storedPlayers: 2,
  skippedPlayers: 0,
};

let mode: PlayerShortlistImportIpcMockMode = "success";
let summary = DEFAULT_SUMMARY;
let error: unknown = new Error(
  "CSV does not contain players in the current snapshot",
);
let lastArgs: unknown;

export function resetPlayerShortlistImportIpcMock() {
  mode = "success";
  summary = DEFAULT_SUMMARY;
  error = new Error("CSV does not contain players in the current snapshot");
  lastArgs = undefined;
}

export function setPlayerShortlistImportIpcMockResult(
  nextSummary: PlayerShortlistImportSummary,
) {
  mode = "success";
  summary = nextSummary;
}

export function setPlayerShortlistImportIpcMockError(nextError: unknown) {
  mode = "error";
  error = nextError;
}

export function getLastPlayerShortlistImportIpcArgs() {
  return lastArgs;
}

export function resolvePlayerShortlistImportIpcMock(args: unknown) {
  lastArgs = args;

  if (mode === "error") {
    return Promise.reject(error);
  }

  return Promise.resolve(summary);
}
