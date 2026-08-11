import { describe, expect, it } from "vitest";
import { type CsvImportState, importStateForContext } from "./import-state";

describe("importStateForContext", () => {
  it("hides a completed import from an older snapshot synchronously", () => {
    const oldState: CsvImportState = {
      status: "success",
      contextKey: "1:1",
      summary: {
        format: "moneyball",
        totalPlayers: 75,
        storedPlayers: 74,
        skippedPlayers: 1,
      },
    };

    expect(importStateForContext(oldState, "1:2")).toEqual({
      status: "idle",
    });
  });
});
