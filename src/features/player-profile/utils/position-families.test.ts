import { describe, expect, it } from "vitest";
import type { PlayerRoleScore } from "../types/player-detail";
import {
  bestPotentialRoleScore,
  bestRoleScore,
  defaultProfilePosition,
  isGoalkeeper,
  PROFILE_POSITION_TAGS,
  rolesForPhase,
  rolesForPlayablePositions,
  rolesForProfilePosition,
} from "./position-families";

function role(
  partial: Pick<
    PlayerRoleScore,
    "roleId" | "displayName" | "positionTags" | "score"
  > &
    Partial<Pick<PlayerRoleScore, "phase" | "potentialScore">>,
): PlayerRoleScore {
  return {
    phase: "in_possession",
    potentialScore: null,
    ...partial,
  };
}

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

describe("bestPotentialRoleScore", () => {
  it("picks the highest non-null potential score and keeps catalog order on ties", () => {
    const scores = [
      role({
        roleId: "a",
        displayName: "Current specialist",
        positionTags: ["MC"],
        score: 82,
        potentialScore: 88,
      }),
      role({
        roleId: "b",
        displayName: "Potential specialist",
        positionTags: ["ST"],
        score: 70,
        potentialScore: 94,
      }),
      role({
        roleId: "c",
        displayName: "Potential tie",
        positionTags: ["ST"],
        score: 69,
        potentialScore: 94,
      }),
      role({
        roleId: "d",
        displayName: "No potential score",
        positionTags: ["GK"],
        score: 40,
        potentialScore: null,
      }),
    ];

    expect(bestPotentialRoleScore(scores)?.displayName).toBe(
      "Potential specialist",
    );
    expect(bestPotentialRoleScore([])).toBeNull();
    expect(
      bestPotentialRoleScore([
        role({
          roleId: "n",
          displayName: "Only null",
          positionTags: ["GK"],
          score: 42,
          potentialScore: null,
        }),
      ]),
    ).toBeNull();
  });
});

describe("profile position selection", () => {
  const scores = [
    role({
      roleId: "gk",
      displayName: "Goalkeeper",
      positionTags: ["GK"],
      score: 40,
    }),
    role({
      roleId: "dlp",
      displayName: "Deep-Lying Playmaker",
      positionTags: ["DM", "MC"],
      score: 82,
      potentialScore: 75,
    }),
    role({
      roleId: "cm",
      displayName: "Central Midfielder",
      positionTags: ["MC"],
      score: 72,
      potentialScore: 88,
    }),
  ];

  it("starts from the strongest recorded familiarity", () => {
    expect(defaultProfilePosition({ MC: 20, ST: 15 }, scores)).toBe("MC");
  });

  it("keeps complete nullable maps positive-only and excludes SW from the pitch", () => {
    const positions = {
      AMR: 20,
      MR: 17,
      AMC: 14,
      SW: null,
      GK: 0,
    };

    expect(PROFILE_POSITION_TAGS).not.toContain("SW");
    expect(defaultProfilePosition(positions, scores)).toBe("AMR");
    expect(isGoalkeeper({ ...positions, GK: 14 })).toBe(false);
    expect(defaultProfilePosition({ SW: 20 }, scores)).toBe("DM");
  });

  it("recognizes goalkeeper profiles at the playable familiarity threshold", () => {
    expect(isGoalkeeper({ GK: 15 })).toBe(true);
    expect(isGoalkeeper({ GK: 14 })).toBe(false);
    expect(isGoalkeeper({ MC: 20 })).toBe(false);
  });

  it("falls back to the best role position when familiarity is unavailable", () => {
    expect(defaultProfilePosition({}, scores)).toBe("DM");
  });

  it("returns only exact-position roles ranked by the selected score basis", () => {
    expect(
      rolesForProfilePosition(scores, "MC").map((item) => item.roleId),
    ).toEqual(["dlp", "cm"]);
    expect(
      rolesForProfilePosition(scores, "MC", {
        basis: "potential",
        direction: "descending",
      }).map((item) => item.roleId),
    ).toEqual(["cm", "dlp"]);
    expect(
      rolesForProfilePosition(scores, "MC", {
        basis: "potential",
        direction: "ascending",
      }).map((item) => item.roleId),
    ).toEqual(["dlp", "cm"]);
    expect(
      rolesForProfilePosition(scores, "GK").map((item) => item.roleId),
    ).toEqual(["gk"]);
  });

  it("keeps only roles for positions with familiarity 15 or higher", () => {
    expect(
      rolesForPlayablePositions(scores, { GK: 16, MC: 15, DM: 14 }).map(
        (item) => item.roleId,
      ),
    ).toEqual(["gk", "dlp", "cm"]);
    expect(rolesForPlayablePositions(scores, { MC: 14 })).toEqual([]);
  });

  it("keeps catalog ties stable and unavailable scores last in either direction", () => {
    const sortableScores = [
      role({
        roleId: "first-tie",
        displayName: "First tie",
        positionTags: ["MC"],
        score: 70,
        potentialScore: 80,
      }),
      role({
        roleId: "unavailable",
        displayName: "Unavailable",
        positionTags: ["MC"],
        score: null,
        potentialScore: null,
      }),
      role({
        roleId: "second-tie",
        displayName: "Second tie",
        positionTags: ["MC"],
        score: 70,
        potentialScore: 80,
      }),
      role({
        roleId: "lower",
        displayName: "Lower",
        positionTags: ["MC"],
        score: 60,
        potentialScore: 70,
      }),
    ];

    expect(
      rolesForProfilePosition(sortableScores, "MC", {
        basis: "current",
        direction: "descending",
      }).map((item) => item.roleId),
    ).toEqual(["first-tie", "second-tie", "lower", "unavailable"]);
    expect(
      rolesForProfilePosition(sortableScores, "MC", {
        basis: "current",
        direction: "ascending",
      }).map((item) => item.roleId),
    ).toEqual(["lower", "first-tie", "second-tie", "unavailable"]);
    expect(
      rolesForProfilePosition(sortableScores, "MC", {
        basis: "potential",
        direction: "ascending",
      }).map((item) => item.roleId),
    ).toEqual(["lower", "first-tie", "second-tie", "unavailable"]);
  });
});

describe("profile role phases", () => {
  it("partitions roles by the exact in-possession or out-of-possession phase", () => {
    const scores = [
      role({
        roleId: "ip",
        displayName: "In possession",
        positionTags: ["MC"],
        phase: "in_possession",
        score: 80,
      }),
      role({
        roleId: "oop",
        displayName: "Out of possession",
        positionTags: ["MC"],
        phase: "out_of_possession",
        score: 75,
      }),
      role({
        roleId: "unknown",
        displayName: "Unknown phase",
        positionTags: ["MC"],
        phase: "unknown",
        score: 99,
      }),
    ];

    expect(
      rolesForPhase(scores, "in_possession").map((item) => item.roleId),
    ).toEqual(["ip"]);
    expect(
      rolesForPhase(scores, "out_of_possession").map((item) => item.roleId),
    ).toEqual(["oop"]);
  });
});
