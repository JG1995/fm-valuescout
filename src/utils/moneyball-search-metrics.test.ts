import { describe, expect, it } from "vitest";
import { MONEYBALL_ROLE_CATALOG } from "./moneyball-role-catalog";
import {
  getMoneyballSearchMetric,
  MONEYBALL_SEARCH_METRICS,
} from "./moneyball-search-metrics";
import { getPlayerMetric, PLAYER_METRICS } from "./player-metrics";
import {
  getSearchMetric,
  isSeasonFieldId,
  SEARCH_METRICS,
} from "./search-metrics";

describe("Moneyball Search metrics", () => {
  it("holds only season fields, which Search offers beside every player field", () => {
    expect(
      MONEYBALL_SEARCH_METRICS.every(
        (metric) =>
          metric.id.startsWith("moneyball.") ||
          metric.id.startsWith("moneyball_role."),
      ),
    ).toBe(true);
    expect(SEARCH_METRICS).toHaveLength(
      PLAYER_METRICS.length + MONEYBALL_SEARCH_METRICS.length,
    );
    expect(new Set(SEARCH_METRICS.map((metric) => metric.id)).size).toBe(
      SEARCH_METRICS.length,
    );
    for (const id of ["attr.Pace", "role.goalkeeper_ip", "club_dna", "ca"]) {
      expect(getSearchMetric(id)).toBe(getPlayerMetric(id));
      expect(isSeasonFieldId(id)).toBe(false);
    }
    for (const id of [
      "moneyball.minutes",
      "moneyball.goals",
      "moneyball_role.wbl_wbr_wing_back_ip",
    ]) {
      expect(getSearchMetric(id)).toBe(getMoneyballSearchMetric(id));
      expect(isSeasonFieldId(id)).toBe(true);
    }
    expect(isSeasonFieldId("tactic_moneyball.goalkeeper")).toBe(true);
    expect(isSeasonFieldId("tactic_current.goalkeeper")).toBe(false);
  });

  it("offers no column, filter, or sort for metrics the save cannot provide", () => {
    for (const id of [
      "goals_from_outside_the_box",
      "goals_from_outside_the_box_per_90",
    ]) {
      expect(getMoneyballSearchMetric(`moneyball.${id}`)).toBeUndefined();
    }
    for (const [id, label] of [
      ["pressures_completed_per_90", "Pressures Completed / 90"],
      ["expected_save_percentage", "Expected Save %"],
    ]) {
      expect(getMoneyballSearchMetric(`moneyball.${id}`)?.metric?.label).toBe(
        label,
      );
    }
    expect(getMoneyballSearchMetric("moneyball.xgp")?.metric?.label).toBe(
      "xGP",
    );
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
            "gte,lte,between,eq",
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
      label: "Moneyball Score · Wing-Back (IP · WBR/WBL)",
      category: "Moneyball Scores · Wing-back",
      role: true,
      roleId: "wbl_wbr_wing_back_ip",
    });
    expect(
      roleMetrics.find(
        (metric) => metric.id === "moneyball_role.dl_dr_wing_back_ip",
      )?.label,
    ).toBe("Moneyball Score · Wing-Back (IP · DR/DL)");
  });
});
