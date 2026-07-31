import { describe, expect, it } from "vitest";
import type { PlayerRoleScore } from "../types/player-detail";
import {
  bestRoleScore,
  groupRolesByFamily,
  primaryFamily,
} from "./position-families";

function role(
  partial: Pick<
    PlayerRoleScore,
    "roleId" | "displayName" | "positionTags" | "score"
  > &
    Partial<Pick<PlayerRoleScore, "phase">>,
): PlayerRoleScore {
  return {
    phase: "ip",
    ...partial,
  };
}

describe("primaryFamily", () => {
  it("uses the first tag that maps to a known family", () => {
    expect(primaryFamily(["DM", "MC"])?.id).toBe("defensive-midfield");
    expect(primaryFamily(["MC"])?.id).toBe("central-midfield");
    expect(primaryFamily(["GK"])?.title).toBe("Goalkeeper");
    expect(primaryFamily(["XX"])).toBeNull();
  });
});

describe("groupRolesByFamily", () => {
  it("groups by position family in pitch order and keeps catalog order", () => {
    const scores = [
      role({
        roleId: "gk",
        displayName: "Goalkeeper",
        positionTags: ["GK"],
        score: 40,
      }),
      role({
        roleId: "st",
        displayName: "Advanced Forward",
        positionTags: ["ST"],
        score: 55,
      }),
      role({
        roleId: "dlp",
        displayName: "Deep-Lying Playmaker",
        positionTags: ["DM", "MC"],
        score: 82,
      }),
      role({
        roleId: "cb",
        displayName: "Centre-Back",
        positionTags: ["DC"],
        score: null,
      }),
      role({
        roleId: "cm",
        displayName: "Central Midfielder",
        positionTags: ["MC"],
        score: 72,
      }),
    ];

    const groups = groupRolesByFamily(scores);
    expect(groups.map((g) => g.family.title)).toEqual([
      "Goalkeeper",
      "Centre-back",
      "Defensive midfield",
      "Central midfield",
      "Striker",
    ]);
    expect(groups[2].roles.map((r) => r.displayName)).toEqual([
      "Deep-Lying Playmaker",
    ]);
    expect(groups[3].roles.map((r) => r.displayName)).toEqual([
      "Central Midfielder",
    ]);
  });
});

describe("bestRoleScore", () => {
  it("picks the highest non-null score and keeps catalog order on ties", () => {
    const scores = [
      role({
        roleId: "a",
        displayName: "First",
        positionTags: ["MC"],
        score: 70,
      }),
      role({
        roleId: "b",
        displayName: "Second",
        positionTags: ["MC"],
        score: 82,
      }),
      role({
        roleId: "c",
        displayName: "Null",
        positionTags: ["ST"],
        score: null,
      }),
      role({
        roleId: "d",
        displayName: "Tied",
        positionTags: ["ST"],
        score: 82,
      }),
    ];

    expect(bestRoleScore(scores)?.displayName).toBe("Second");
    expect(bestRoleScore([])).toBeNull();
    expect(
      bestRoleScore([
        role({
          roleId: "n",
          displayName: "Only null",
          positionTags: ["GK"],
          score: null,
        }),
      ]),
    ).toBeNull();
  });
});
