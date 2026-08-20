import { describe, expect, it } from "vitest";
import { moneyballMetric } from "@/utils/moneyball-metrics";
import { formatMoneyballMetric } from "./format-moneyball-metric";

describe("formatMoneyballMetric", () => {
  it("formats raw values without replacing a real zero or missing value", () => {
    expect(formatMoneyballMetric(moneyballMetric("goals"), 0)).toBe("0");
    expect(formatMoneyballMetric(moneyballMetric("goals_per_90"), 1.234)).toBe(
      "1.23",
    );
    expect(formatMoneyballMetric(moneyballMetric("minutes_per_goal"), 90)).toBe(
      "90.0",
    );
    expect(formatMoneyballMetric(moneyballMetric("save_ratio"), 0.8125)).toBe(
      "81.3%",
    );
    expect(
      formatMoneyballMetric(moneyballMetric("distance_covered"), 12.34),
    ).toBe("12.3 km");
    expect(formatMoneyballMetric(moneyballMetric("average_rating"), 7.25)).toBe(
      "7.25",
    );
    expect(formatMoneyballMetric(moneyballMetric("goals"), null)).toBe("—");
  });
});
