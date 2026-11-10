import { describe, expect, it } from "vitest";
import { MONEYBALL_ROLE_CATALOG } from "./moneyball-role-catalog";
import {
  DEFAULT_MONEYBALL_TABLE_COLUMN_IDS,
  getMoneyballSearchMetric,
  MONEYBALL_SEARCH_METRICS,
} from "./moneyball-search-metrics";
import { getPlayerMetric } from "./player-metrics";

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
      "height",
      "club",
      "division",
      "parent_club",
      "preferred_foot",
      "value",
      "position",
      "ca",
      "pa",
      "reputation",
      "world_reputation",
      "wage",
      "contract_year",
      "transfer_listed",
      "loan_listed",
      "not_for_sale",
      "set_for_release",
      "on_loan",
      "team_level",
    ]);
    expect(getMoneyballSearchMetric("height")?.defaultWidth).toBe(88);
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

  it("shares General's ability and contract fields but not FM attributes or role scores", () => {
    for (const id of ["ca", "wage", "transfer_listed", "team_level"]) {
      const general = getPlayerMetric(id);
      expect(getMoneyballSearchMetric(id)).toMatchObject({
        label: general?.label,
        kind: general?.kind,
        operators: general?.operators,
      });
    }
    expect(getMoneyballSearchMetric("ca")?.category).toBe(
      "Ability and reputation",
    );
    expect(getMoneyballSearchMetric("wage")?.category).toBe("Club and value");
    for (const id of [
      "attr.Pace",
      "hidden.Consistency",
      "role.goalkeeper_ip",
      "club_dna",
    ]) {
      expect(getMoneyballSearchMetric(id)).toBeUndefined();
    }
  });

  it("filters Position with the same position choices as General Search", () => {
    const general = getPlayerMetric("position");

    expect(getMoneyballSearchMetric("position")).toMatchObject({
      kind: "enum",
      operators: general?.operators,
      enumOptions: general?.enumOptions,
    });
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

  it("labels the Age metric as age-only", () => {
    expect(getMoneyballSearchMetric("age")?.label).toBe("Age");
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
