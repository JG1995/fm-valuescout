import { describe, expect, it } from "vitest";
import {
  attributeTierLabel,
  attributeValueTier,
} from "@/components/ui/attribute-value/attribute-value";
import { VISIBLE_ATTRIBUTE_GROUPS as CANONICAL_VISIBLE_ATTRIBUTE_GROUPS } from "@/utils/player-attributes";
import {
  attributeRows,
  GOALKEEPER_OUTFIELD_ATTRIBUTE_GROUPS,
  GOALKEEPER_PRIMARY_ATTRIBUTE_GROUPS,
  HIDDEN_ATTRIBUTE_KEYS,
  labelFromPascal,
  OUTFIELD_ATTRIBUTE_GROUPS,
  PERSONALITY_ATTRIBUTE_KEYS,
  VISIBLE_ATTRIBUTE_GROUPS,
} from "./attribute-groups";

describe("attribute-groups", () => {
  it("uses the canonical visible attribute groups", () => {
    expect(VISIBLE_ATTRIBUTE_GROUPS).toBe(CANONICAL_VISIBLE_ATTRIBUTE_GROUPS);
  });

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

  it("keeps the Profile group order", () => {
    expect(OUTFIELD_ATTRIBUTE_GROUPS.map((group) => group.id)).toEqual([
      "technical",
      "mental",
      "physical",
    ]);
    expect(
      GOALKEEPER_PRIMARY_ATTRIBUTE_GROUPS.map((group) => group.id),
    ).toEqual(["goalkeeping", "mental", "physical"]);
    expect(
      GOALKEEPER_OUTFIELD_ATTRIBUTE_GROUPS.map((group) => group.id),
    ).toEqual(["technical"]);
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

  it("keeps goalkeeper technicals separate from their primary mental and physical groups", () => {
    expect(
      GOALKEEPER_PRIMARY_ATTRIBUTE_GROUPS.map((group) => group.title),
    ).toEqual(["Goalkeeping", "Mental", "Physical"]);
    expect(GOALKEEPER_PRIMARY_ATTRIBUTE_GROUPS[0].keys).toEqual([
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
    expect(
      GOALKEEPER_OUTFIELD_ATTRIBUTE_GROUPS.map((group) => group.title),
    ).toEqual(["Technical"]);
    const goalkeeperOutfieldKeys = flattenAttributeGroups(
      GOALKEEPER_OUTFIELD_ATTRIBUTE_GROUPS,
    );
    expect(goalkeeperOutfieldKeys).not.toContain("FirstTouch");
    expect(goalkeeperOutfieldKeys).not.toContain("Passing");
    expect(goalkeeperOutfieldKeys).not.toContain("Technique");
    const goalkeeperKeys = [
      ...goalkeeperOutfieldKeys,
      ...flattenAttributeGroups(GOALKEEPER_PRIMARY_ATTRIBUTE_GROUPS),
    ];
    expect(new Set(goalkeeperKeys)).toEqual(
      new Set(VISIBLE_ATTRIBUTE_KEYS_FLAT()),
    );
    expect(new Set(goalkeeperKeys).size).toBe(goalkeeperKeys.length);
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
  return flattenAttributeGroups(VISIBLE_ATTRIBUTE_GROUPS);
}

function flattenAttributeGroups(
  groups: readonly {
    keys: readonly string[];
    subgroups?: readonly { keys: readonly string[] }[];
  }[],
): string[] {
  return groups.flatMap((group) => [
    ...group.keys,
    ...(group.subgroups?.flatMap((subgroup) => [...subgroup.keys]) ?? []),
  ]);
}
