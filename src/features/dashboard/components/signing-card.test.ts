import { describe, expect, it } from "vitest";
import type { DashboardRecommendation } from "../types/dashboard";
import { signingValueLine } from "./signing-card";

const recommendation: DashboardRecommendation = {
  uid: 10,
  name: "Better Keeper",
  age: 24,
  club: "Other FC",
  clubUid: 2000,
  marketValueGbp: 12_500_000,
  minutes: 1240,
  borrowedStatsDate: null,
  laneId: "goalkeeper",
  score: 88,
  improvement: 30,
  agePenalty: 0,
  otherLanes: [],
};

describe("signingValueLine", () => {
  it("dates Moneyball minutes borrowed from a kept snapshot", () => {
    expect(signingValueLine(recommendation, "moneyball")).toBe(
      "€12.5M · 1,240 min",
    );
    expect(
      signingValueLine(
        { ...recommendation, borrowedStatsDate: "2026-05-20" },
        "moneyball",
      ),
    ).toBe("€12.5M · 1,240 min as of 20 May 2026");
    expect(signingValueLine(recommendation, "general")).toBe("€12.5M");
  });
});
