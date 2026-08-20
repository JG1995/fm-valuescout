import { describe, expect, it } from "vitest";
import {
  DEFAULT_MONEYBALL_TABLE_COLUMN_IDS,
  MONEYBALL_SEARCH_METRICS,
} from "./moneyball-search-metrics";

describe("Moneyball Search metrics", () => {
  it("includes every required recruitment field and the exact default columns", () => {
    expect(
      MONEYBALL_SEARCH_METRICS.filter(
        (metric) => !metric.id.startsWith("moneyball."),
      ).map((metric) => metric.id),
    ).toEqual([
      "name",
      "age",
      "nationality",
      "club",
      "division",
      "parent_club",
      "preferred_foot",
      "value",
      "position",
    ]);
    expect(DEFAULT_MONEYBALL_TABLE_COLUMN_IDS).toEqual([
      "name",
      "age",
      "nationality",
      "club",
      "division",
      "moneyball.minutes",
      "moneyball.average_rating",
      "moneyball.goals_per_90",
      "moneyball.assists_per_90",
      "moneyball.xg_per_90",
      "moneyball.xa_per_90",
    ]);
  });
});
