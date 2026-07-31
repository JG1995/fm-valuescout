import { describe, expect, it } from "vitest";
import { scoreBadgeAccessibleName, scoreToTier, tierLabel } from "./score-tier";

describe("scoreToTier", () => {
  it("maps score bands to DESIGN.md tiers", () => {
    expect(scoreToTier(0)).toBe(1);
    expect(scoreToTier(39)).toBe(1);
    expect(scoreToTier(40)).toBe(2);
    expect(scoreToTier(54)).toBe(2);
    expect(scoreToTier(55)).toBe(3);
    expect(scoreToTier(69)).toBe(3);
    expect(scoreToTier(70)).toBe(4);
    expect(scoreToTier(84)).toBe(4);
    expect(scoreToTier(85)).toBe(5);
    expect(scoreToTier(100)).toBe(5);
  });
});

describe("scoreBadgeAccessibleName", () => {
  it("includes role name, score, and tier label", () => {
    expect(scoreBadgeAccessibleName("Deep-Lying Playmaker", 82)).toBe(
      "Deep-Lying Playmaker: 82, Starter",
    );
    expect(tierLabel(5)).toBe("Elite");
  });
});
