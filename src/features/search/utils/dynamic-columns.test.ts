import { describe, expect, it } from "vitest";
import { ROLE_CATALOG } from "@/utils/role-catalog";
import { createFilterRuleId } from "../types/filter-rule";
import { dynamicColumnFields, isVisibleSortField } from "./dynamic-columns";

describe("dynamicColumnFields", () => {
  it("returns unique non-basic complete filter fields", () => {
    const fields = dynamicColumnFields([
      {
        id: createFilterRuleId(),
        field: "ca",
        op: "gt",
        value: { type: "integer", value: 100 },
      },
      {
        id: createFilterRuleId(),
        field: "role.deep_lying_playmaker_ip",
        op: "gt",
        value: { type: "integer", value: 70 },
      },
      {
        id: createFilterRuleId(),
        field: "attr.Acceleration",
        op: "gt",
        value: { type: "integer", value: 12 },
      },
      {
        id: createFilterRuleId(),
        field: "role.deep_lying_playmaker_ip",
        op: "lt",
        value: { type: "integer", value: 95 },
      },
    ]);

    expect(fields).toEqual([
      "role.deep_lying_playmaker_ip",
      "attr.Acceleration",
    ]);
  });

  it("allows sorting by a visible dynamic column only", () => {
    const filters = [
      {
        id: createFilterRuleId(),
        field: "role.deep_lying_playmaker_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];
    expect(isVisibleSortField("role.deep_lying_playmaker_ip", filters)).toBe(
      true,
    );
    expect(isVisibleSortField("attr.Acceleration", filters)).toBe(false);
    expect(isVisibleSortField("ca", filters)).toBe(true);
  });

  it("excludes position presence from dynamic columns", () => {
    expect(
      dynamicColumnFields([
        {
          id: createFilterRuleId(),
          field: "position",
          op: "is",
          value: { type: "text", value: "MC" },
        },
        {
          id: createFilterRuleId(),
          field: "pos.MC",
          op: "gt",
          value: { type: "integer", value: 15 },
        },
      ]),
    ).toEqual(["pos.MC"]);
  });

  it("keeps potential role filters out of the legacy dynamic-column contract", () => {
    const filters = [
      {
        id: createFilterRuleId(),
        field: "potential_role.goalkeeper_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];

    expect(dynamicColumnFields(filters)).toEqual([]);
    expect(isVisibleSortField("potential_role.goalkeeper_ip", filters)).toBe(
      false,
    );
  });
});

describe("ROLE_CATALOG", () => {
  it("mirrors the FM26 catalog size and DLP id", () => {
    // Keep in sync with scoring::catalog::all_roles (68 roles).
    expect(ROLE_CATALOG).toHaveLength(68);
    expect(
      ROLE_CATALOG.some((role) => role.id === "deep_lying_playmaker_ip"),
    ).toBe(true);
  });
});
