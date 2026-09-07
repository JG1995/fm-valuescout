import { describe, expect, it } from "vitest";
import { tableColumnForMetric as searchTableColumnForMetric } from "@/features/search/components/search-results-panel";
import { tableColumnForMetric as squadTableColumnForMetric } from "@/features/squad/components/squad-overview-panel";
import { staffTableColumnForMetric } from "@/features/staff/components/staff-search-results-panel";
import { getStaffMetric } from "@/features/staff/utils/staff-metrics";
import { getMoneyballSearchMetric } from "@/utils/moneyball-search-metrics";
import { getPlayerMetric } from "@/utils/player-metrics";

const CASES = [
  {
    layout: "Search",
    column: searchTableColumnForMetric(
      "nationality",
      undefined,
      "general",
      new Map(),
    ),
  },
  {
    layout: "Moneyball Search",
    column: searchTableColumnForMetric(
      "nationality",
      undefined,
      "moneyball",
      new Map(),
    ),
  },
  {
    layout: "Squad",
    column: squadTableColumnForMetric("nationality", undefined),
  },
  // My Staff shares the `isShortlist: false` path with Staff Search above;
  // its rendered behavior is proved by the staff route toolbar test instead.
  {
    layout: "Staff Search",
    column: staffTableColumnForMetric("nationality", false, undefined),
  },
  {
    layout: "Staff Shortlist",
    column: staffTableColumnForMetric("nationality", true, undefined),
  },
];

describe("nationality leaf headers", () => {
  it.each(CASES)(
    "$layout shows Nat. with the Nationality accessible name",
    ({ column }) => {
      expect(column).toMatchObject({
        id: "nationality",
        label: "Nat.",
        accessibleLabel: "Nationality",
      });
    },
  );

  it("keeps domain catalog labels untouched for filters and pickers", () => {
    expect(getPlayerMetric("nationality")?.label).toBe("Nationality");
    expect(getMoneyballSearchMetric("nationality")?.label).toBe("Nationality");
    expect(getStaffMetric("nationality")?.label).toBe("Nation");
  });
});
