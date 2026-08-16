import { describe, expect, it } from "vitest";
import {
  DEFAULT_STAFF_TABLE_COLUMN_IDS,
  getStaffMetric,
  STAFF_METRICS,
  STAFF_ROLE_METRICS,
} from "./staff-metrics";

describe("staff metrics", () => {
  it("defines the 25 default search columns in catalog order", () => {
    expect(DEFAULT_STAFF_TABLE_COLUMN_IDS).toHaveLength(25);
    expect(DEFAULT_STAFF_TABLE_COLUMN_IDS.slice(0, 5)).toEqual([
      "name",
      "age",
      "nationality",
      "ca",
      "pa",
    ]);
    expect(DEFAULT_STAFF_TABLE_COLUMN_IDS.slice(5)).toEqual(
      STAFF_ROLE_METRICS.map((metric) => metric.id),
    );
  });

  it("keeps role metadata separate and resolves missing score values", () => {
    expect(STAFF_ROLE_METRICS).toHaveLength(20);
    expect(getStaffMetric("role.coach_goalkeeping")?.label).toBe(
      "Coach — Goalkeeping",
    );
    expect(getStaffMetric("role.unknown")).toBeUndefined();
    expect(STAFF_METRICS.some((metric) => metric.id === "attr.Authority")).toBe(
      true,
    );
  });
});
