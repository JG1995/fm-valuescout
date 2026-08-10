import type { CsvMatchPreview } from "../types/csv-match-preview";

export type CsvPreviewState =
  | { status: "idle" }
  | { status: "pending"; contextKey: string }
  | { status: "success"; contextKey: string; preview: CsvMatchPreview }
  | { status: "error"; contextKey: string; error: Error };

export function previewStateForContext(
  state: CsvPreviewState,
  contextKey: string,
): CsvPreviewState {
  if (state.status === "idle" || state.contextKey === contextKey) {
    return state;
  }

  return { status: "idle" };
}
