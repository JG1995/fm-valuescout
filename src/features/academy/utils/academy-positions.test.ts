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

  it("orders equal familiarity from right to left and up the pitch", () => {
    expect(
      recordedAcademyPositions({
        AML: 20,
        AMR: 20,
        DL: 20,
        DR: 20,
        GK: 20,
      }),
    ).toEqual(["GK", "DR", "DL", "AMR", "AML"]);
  });
});
