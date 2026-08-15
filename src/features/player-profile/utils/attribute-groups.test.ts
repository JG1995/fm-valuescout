import { describe, expect, it } from "vitest";
import {
  attributeRows,
  attributeTierLabel,
  attributeValueTier,
  GOALKEEPER_OUTFIELD_ATTRIBUTE_GROUPS,
  GOALKEEPER_PRIMARY_ATTRIBUTE_GROUP,
  HIDDEN_ATTRIBUTE_KEYS,
  labelFromPascal,
  PERSONALITY_ATTRIBUTE_KEYS,
  VISIBLE_ATTRIBUTE_GROUPS,
} from "./attribute-groups";

describe("attribute-groups", () => {
  it("exposes Technical Mental Physical and Goalkeeping with known dump keys", () => {
    const titles = VISIBLE_ATTRIBUTE_GROUPS.map((group) => group.title);
    expect(titles).toEqual(["Technical", "Mental", "Physical", "Goalkeeping"]);
    const keys = VISIBLE_ATTRIBUTE_KEYS_FLAT();
    expect(VISIBLE_ATTRIBUTE_GROUPS[0].keys).toContain("Crossing");
    expect(keys).toContain("Acceleration");
    expect(keys).toContain("Handling");
    expect(keys).toContain("Corners");
    expect(new Set(keys).size).toBe(keys.length);
    expect(HIDDEN_ATTRIBUTE_KEYS).toContain("Consistency");
    expect(PERSONALITY_ATTRIBUTE_KEYS).toContain("Ambition");
  });

  it("keeps outfield groups together and separates technical set pieces", () => {
    const technical = VISIBLE_ATTRIBUTE_GROUPS[0];
    expect(technical.subgroups).toEqual([
      {
        title: "Set Pieces",
        keys: ["Corners", "FreeKicks", "LongThrows", "PenaltyTaking"],
      },
    ]);
    expect(technical.keys).not.toContain("Corners");
    expect(
      technical.subgroups?.flatMap((subgroup) => [...subgroup.keys]),
    ).toContain("PenaltyTaking");
  });

  it("moves goalkeeper ball-playing attributes into the alphabetical Goalkeeping group", () => {
    expect(GOALKEEPER_PRIMARY_ATTRIBUTE_GROUP.keys).toEqual([
      "AerialReach",
      "CommandOfArea",
      "Communication",
      "Eccentricity",
      "FirstTouch",
      "Handling",
      "Kicking",
      "OneOnOnes",
      "Passing",
      "Punching",
      "Reflexes",
      "RushingOut",
      "Technique",
      "Throwing",
    ]);
    const goalkeeperOutfieldKeys = GOALKEEPER_OUTFIELD_ATTRIBUTE_GROUPS.flatMap(
      (group) => [...group.keys],
    );
    expect(goalkeeperOutfieldKeys).not.toContain("FirstTouch");
    expect(goalkeeperOutfieldKeys).not.toContain("Passing");
    expect(goalkeeperOutfieldKeys).not.toContain("Technique");
  });

  it("keeps null and missing values as null so display can show an em dash", () => {
    const rows = attributeRows(["Crossing", "Acceleration", "Pace"], {
      Crossing: null,
      Acceleration: 14,
    });

    expect(rows).toEqual([
      { key: "Crossing", label: "Crossing", value: null },
      { key: "Acceleration", label: "Acceleration", value: 14 },
      { key: "Pace", label: "Pace", value: null },
    ]);
  });

  it("pairs visible current values with potential values without changing nulls", () => {
    const rows = attributeRows(
      ["Crossing", "Acceleration", "Pace"],
      { Crossing: null, Acceleration: 14 },
      { Crossing: null, Acceleration: 16 },
    );

    expect(rows).toEqual([
      {
        key: "Crossing",
        label: "Crossing",
        value: null,
        potentialValue: null,
      },
      {
        key: "Acceleration",
        label: "Acceleration",
        value: 14,
        potentialValue: 16,
      },
      {
        key: "Pace",
        label: "Pace",
        value: null,
        potentialValue: null,
      },
    ]);
  });

  it("splits PascalCase keys into readable labels", () => {
    expect(labelFromPascal("OffTheBall")).toBe("Off The Ball");
    expect(labelFromPascal("JumpingReach")).toBe("Jumping Reach");
  });

  it("maps the FM attribute scale to its four familiar display tiers", () => {
    expect([1, 5, 6, 10, 11, 15, 16, 20].map(attributeValueTier)).toEqual([
      1, 1, 2, 2, 3, 3, 4, 4,
    ]);
    expect(attributeTierLabel(4)).toBe("Excellent");
  });
});

function VISIBLE_ATTRIBUTE_KEYS_FLAT(): string[] {
  return VISIBLE_ATTRIBUTE_GROUPS.flatMap((group) => [
    ...group.keys,
    ...(group.subgroups?.flatMap((subgroup) => [...subgroup.keys]) ?? []),
  ]);
}
