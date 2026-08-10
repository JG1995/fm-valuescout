import { describe, expect, it } from "vitest";
import { scoreBadgeAccessibleName, scoreToTier, tierLabel } from "./score-tier";

describe("scoreToTier", () => {
  it("maps FM-style score bands to DESIGN.md tiers", () => {
    expect(scoreToTier(0)).toBe(1);
    expect(scoreToTier(40)).toBe(1);
    expect(scoreToTier(41)).toBe(2);
    expect(scoreToTier(60)).toBe(2);
    expect(scoreToTier(61)).toBe(3);
    expect(scoreToTier(80)).toBe(3);
    expect(scoreToTier(81)).toBe(4);
    expect(scoreToTier(100)).toBe(4);
  });
});

describe("scoreBadgeAccessibleName", () => {
  it("includes role name, score, and tier label", () => {
    expect(scoreBadgeAccessibleName("Deep-Lying Playmaker", 82)).toBe(
      "Deep-Lying Playmaker: 82, Excellent",
    );
    expect(tierLabel(4)).toBe("Excellent");
  });
});
