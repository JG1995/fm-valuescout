import type { CsvImportSummary } from "../types/csv-import-summary";

export type CsvImportState =
  | { status: "idle" }
  | { status: "pending"; contextKey: string }
  | { status: "success"; contextKey: string; summary: CsvImportSummary }
  | { status: "error"; contextKey: string; error: Error };

export function importStateForContext(
  state: CsvImportState,
  contextKey: string,
): CsvImportState {
  if (state.status === "idle" || state.contextKey === contextKey) {
    return state;
  }

  return { status: "idle" };
}
