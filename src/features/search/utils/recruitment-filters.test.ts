import { describe, expect, it } from "vitest";
import {
  formatRecruitmentFilter,
  recruitmentFilterFromRule,
  recruitmentFilterToRule,
} from "./recruitment-filters";

describe("recruitment filters", () => {
  it("shows a stored fraction statistic on the percentage scale and stores it back", () => {
    const stored = { field: "moneyball.save_ratio", op: "gte", value: 0.57 };

    const rule = recruitmentFilterToRule(stored, "rule-0");

    expect(rule?.value).toEqual({ type: "number", value: 57 });
    expect(formatRecruitmentFilter(stored)).toBe("Save % ≥ 57%");
    expect(rule && recruitmentFilterFromRule(rule)).toEqual(stored);
  });

  it("keeps percentiles and General fields as stored", () => {
    for (const stored of [
      { field: "moneyball.save_ratio", op: "pct_gte", value: 25 },
      { field: "height", op: "between", value: [183, 195] },
      { field: "preferred_foot", op: "in", value: ["right"] },
    ]) {
      const rule = recruitmentFilterToRule(stored, "rule-0");
      expect(rule && recruitmentFilterFromRule(rule)).toEqual(stored);
    }
    expect(
      formatRecruitmentFilter({
        field: "moneyball.progressive_passes",
        op: "pct_gte",
        value: 25,
      }),
    ).toBe("Progressive Passes percentile ≥ 25");
  });

  it("offers General fields and Moneyball statistics but not slot-incompatible ones", () => {
    for (const field of [
      "club_dna",
      "potential_role.centre_back_ip",
      "realistic_signing",
      "moneyball_role.ball_playing_defender",
    ]) {
      expect(
        recruitmentFilterToRule({ field, op: "gte", value: 50 }, "rule-0"),
      ).toBeUndefined();
    }
    expect(
      recruitmentFilterToRule(
        { field: "moneyball.minutes", op: "gte", value: 900 },
        "rule-0",
      ),
    ).toBeDefined();
  });
});
