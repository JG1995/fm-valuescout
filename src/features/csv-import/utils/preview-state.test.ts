import { describe, expect, it } from "vitest";
import { type CsvPreviewState, previewStateForContext } from "./preview-state";

describe("previewStateForContext", () => {
  it("hides a completed preview from an older snapshot synchronously", () => {
    const oldState: CsvPreviewState = {
      status: "success",
      contextKey: "1:1",
      preview: {
        format: "moneyball",
        totalPlayers: 75,
        matchedPlayers: 74,
        unmatchedPlayers: 1,
      },
    };

    expect(previewStateForContext(oldState, "1:2")).toEqual({
      status: "idle",
    });
  });
});
