import { describe, expect, it } from "vitest";
import type { FilterRule } from "../types/filter-rule";
import { withRecruitingDefaults } from "./recruiting-defaults";
import { MAX_FILTER_RULES } from "./search-url-search";

const height: FilterRule = {
  id: "height",
  field: "height",
  op: "gte",
  value: { type: "integer", value: 180 },
};
const pace: FilterRule = {
  id: "pace",
  field: "attr.pace",
  op: "gte",
  value: { type: "integer", value: 14 },
};
const realistic = expect.objectContaining({
  field: "realistic_signing",
  op: "is",
  value: { type: "bool", value: true },
});
const age28 = expect.objectContaining({
  field: "age",
  op: "lte",
  value: { type: "integer", value: 28 },
});

function apply(
  rules: FilterRule[],
  options: { combine?: "and" | "or"; realisticAvailable?: boolean } = {},
) {
  return withRecruitingDefaults({
    rules,
    combine: options.combine ?? "and",
    realisticAvailable: options.realisticAvailable ?? true,
  });
}

describe("withRecruitingDefaults", () => {
  it("adds realistic signings and the age limit after the filter's own rules", () => {
    expect(apply([height, pace])).toEqual({
      rules: [height, pace, realistic, age28],
      combine: "and",
    });
  });

  it("keeps the filter's own age and realistic-signing rules", () => {
    const ownAge: FilterRule = {
      id: "age",
      field: "age",
      op: "gte",
      value: { type: "integer", value: 30 },
    };
    const unrealistic: FilterRule = {
      id: "unrealistic",
      field: "realistic_signing",
      op: "is_not",
      value: { type: "bool", value: true },
    };

    expect(apply([ownAge]).rules).toEqual([ownAge, realistic]);
    expect(apply([unrealistic]).rules).toEqual([unrealistic, age28]);
    expect(apply([ownAge, unrealistic]).rules).toEqual([ownAge, unrealistic]);
  });

  it("leaves realistic signings out without a club to measure", () => {
    expect(apply([height], { realisticAvailable: false }).rules).toEqual([
      height,
      age28,
    ]);
  });

  it("leaves a filter that matches any rule alone, since further rules would widen it", () => {
    expect(apply([height, pace], { combine: "or" })).toEqual({
      rules: [height, pace],
      combine: "or",
    });
  });

  it("matches every rule when the only rule of a match-any filter gains the defaults", () => {
    expect(apply([height], { combine: "or" })).toEqual({
      rules: [height, realistic, age28],
      combine: "and",
    });
  });

  it("stays within the rule limit", () => {
    const rules = Array.from({ length: MAX_FILTER_RULES - 1 }, (_, index) => ({
      ...height,
      id: `rule-${index}`,
    }));

    expect(apply(rules).rules).toHaveLength(MAX_FILTER_RULES);
    expect(apply([...rules, pace]).rules).toEqual([...rules, pace]);
  });
});
