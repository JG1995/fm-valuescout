import { describe, expect, it } from "vitest";
import { playableAcademyPositions } from "./academy-positions";

describe("Academy playable positions", () => {
  it("lists familiarity 16 or higher strongest-first", () => {
    expect(
      playableAcademyPositions({
        AMR: 20,
        MR: 17,
        MC: 16,
        AMC: 15,
        GK: 0,
        SW: null,
      }),
    ).toEqual(["AMR", "MR", "MC"]);
  });

  it("orders equal familiarity from right to left and up the pitch", () => {
    expect(
      playableAcademyPositions({
        AML: 20,
        AMR: 20,
        DL: 20,
        DR: 20,
        GK: 20,
      }),
    ).toEqual(["GK", "DR", "DL", "AMR", "AML"]);
  });
});
