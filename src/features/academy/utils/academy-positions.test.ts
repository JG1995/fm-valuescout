import { describe, expect, it } from "vitest";
import { recordedAcademyPositions } from "./academy-positions";

describe("Academy recorded positions", () => {
  it("lists positive familiarity strongest-first and excludes zero or unread slots", () => {
    expect(
      recordedAcademyPositions({
        AMR: 20,
        MR: 17,
        AMC: 14,
        GK: 0,
        SW: null,
      }),
    ).toEqual(["AMR", "MR", "AMC"]);
  });
});
