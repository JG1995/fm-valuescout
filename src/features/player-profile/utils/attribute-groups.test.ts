import { describe, expect, it } from "vitest";
import {
  attributeRows,
  HIDDEN_ATTRIBUTE_KEYS,
  labelFromPascal,
  PERSONALITY_ATTRIBUTE_KEYS,
  VISIBLE_ATTRIBUTE_GROUPS,
} from "./attribute-groups";

describe("attribute-groups", () => {
  it("exposes Technical Mental Physical and Goalkeeping with known dump keys", () => {
    const titles = VISIBLE_ATTRIBUTE_GROUPS.map((group) => group.title);
    expect(titles).toEqual(["Technical", "Mental", "Physical", "Goalkeeping"]);
    expect(VISIBLE_ATTRIBUTE_GROUPS[0].keys).toContain("Crossing");
    expect(VISIBLE_ATTRIBUTE_KEYS_FLAT()).toContain("Acceleration");
    expect(VISIBLE_ATTRIBUTE_KEYS_FLAT()).toContain("Handling");
    expect(HIDDEN_ATTRIBUTE_KEYS).toContain("Consistency");
    expect(PERSONALITY_ATTRIBUTE_KEYS).toContain("Ambition");
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

  it("splits PascalCase keys into readable labels", () => {
    expect(labelFromPascal("OffTheBall")).toBe("Off The Ball");
    expect(labelFromPascal("JumpingReach")).toBe("Jumping Reach");
  });
});

function VISIBLE_ATTRIBUTE_KEYS_FLAT(): string[] {
  return VISIBLE_ATTRIBUTE_GROUPS.flatMap((group) => [...group.keys]);
}
