import { describe, expect, it } from "vitest";
import { MONEYBALL_ROLE_CATALOG } from "./moneyball-role-catalog";
import {
  DEFAULT_MONEYBALL_TABLE_COLUMN_IDS,
  MONEYBALL_SEARCH_METRICS,
} from "./moneyball-search-metrics";

describe("Moneyball Search metrics", () => {
  it("includes every required recruitment field and the exact default columns", () => {
    expect(
      MONEYBALL_SEARCH_METRICS.filter(
        (metric) =>
          !metric.id.startsWith("moneyball.") &&
          !metric.id.startsWith("moneyball_role."),
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

  it("registers every backend role as a grouped numeric field", () => {
    const roleMetrics = MONEYBALL_SEARCH_METRICS.filter((metric) =>
      metric.id.startsWith("moneyball_role."),
    );

    expect(roleMetrics).toHaveLength(MONEYBALL_ROLE_CATALOG.length);
    expect(
      roleMetrics.map((metric) => metric.id.slice("moneyball_role.".length)),
    ).toEqual(MONEYBALL_ROLE_CATALOG.map((role) => role.id));
    expect(
      roleMetrics.every(
        (metric) =>
          metric.kind === "integer" &&
          metric.sortable &&
          metric.operators.map((operator) => operator.id).join(",") ===
            "gt,lt,eq,neq",
      ),
    ).toBe(true);
    expect(
      new Set(roleMetrics.map((metric) => metric.category)).size,
    ).toBeGreaterThan(1);
    expect(
      roleMetrics.find(
        (metric) => metric.id === "moneyball_role.wbl_wbr_wing_back_ip",
      ),
    ).toMatchObject({
      label: "Wing-Back (IP · WBR/WBL)",
      category: "Moneyball roles · Wing-back",
      role: true,
      roleId: "wbl_wbr_wing_back_ip",
    });
    expect(
      roleMetrics.find(
        (metric) => metric.id === "moneyball_role.dl_dr_wing_back_ip",
      )?.label,
    ).toBe("Wing-Back (IP · DR/DL)");
  });
});
