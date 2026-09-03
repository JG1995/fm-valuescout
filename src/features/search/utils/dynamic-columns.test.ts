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

  it("requests Club DNA filters and accepts the fixed metric for sorting", () => {
    const fields = dynamicColumnFields([
      {
        id: createFilterRuleId(),
        field: "club_dna",
        op: "gt",
        value: { type: "integer", value: 70 },
      },
    ]);

    expect(fields).toEqual(["club_dna"]);
    expect(isVisibleSortField("club_dna", [])).toBe(true);
  });

  it("allows sorting by known metrics whether or not they are requested", () => {
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
    expect(isVisibleSortField("attr.Acceleration", filters)).toBe(true);
    expect(isVisibleSortField("ca", filters)).toBe(true);
  });

  it("includes Position and position-suitability display fields", () => {
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
    ).toEqual(["position", "pos.MC"]);
  });

  it("includes potential role filters in requested table fields", () => {
    const filters = [
      {
        id: createFilterRuleId(),
        field: "potential_role.goalkeeper_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];

    expect(dynamicColumnFields(filters)).toEqual([
      "potential_role.goalkeeper_ip",
    ]);
    expect(isVisibleSortField("potential_role.goalkeeper_ip", filters)).toBe(
      true,
    );
  });

  it("allows a known metric to sort even while its column is not requested", () => {
    expect(isVisibleSortField("position", [])).toBe(true);
    expect(isVisibleSortField("attr.Acceleration", [])).toBe(true);
    expect(isVisibleSortField("unknown.metric", [])).toBe(false);
  });

  it("allows only canonical tactic sorts present in the current table layout", () => {
    const tacticSort = "tactic_current.goalkeeper";

    expect(isVisibleSortField(tacticSort, [], "general", [tacticSort])).toBe(
      true,
    );
    expect(isVisibleSortField(tacticSort, [], "general", [])).toBe(false);
    expect(
      isVisibleSortField("tactic_current.unknown", [], "general", [
        "tactic_current.unknown",
      ]),
    ).toBe(false);
  });

  it("requests and sorts Moneyball role fields in the Moneyball view only", () => {
    const filters = [
      {
        id: createFilterRuleId(),
        field: "moneyball_role.wbl_wbr_wing_back_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];

    expect(dynamicColumnFields(filters, "moneyball")).toEqual([
      "moneyball_role.wbl_wbr_wing_back_ip",
    ]);
    expect(
      isVisibleSortField(
        "moneyball_role.wbl_wbr_wing_back_ip",
        [],
        "moneyball",
      ),
    ).toBe(true);
    expect(
      isVisibleSortField("moneyball_role.wbl_wbr_wing_back_ip", [], "general"),
    ).toBe(false);
  });

  it("treats Shortlist as General-family for visible sorts", () => {
    expect(isVisibleSortField("ca", [], "shortlist")).toBe(true);
    expect(isVisibleSortField("pa", [], "shortlist")).toBe(true);
    expect(
      isVisibleSortField("role.deep_lying_playmaker_ip", [], "shortlist"),
    ).toBe(true);
    expect(
      isVisibleSortField("potential_role.goalkeeper_ip", [], "shortlist"),
    ).toBe(true);
    expect(isVisibleSortField("attr.Acceleration", [], "shortlist")).toBe(true);
    expect(isVisibleSortField("club_dna", [], "shortlist")).toBe(true);
    expect(isVisibleSortField("moneyball.goals", [], "shortlist")).toBe(false);
    expect(
      isVisibleSortField(
        "moneyball_role.wbl_wbr_wing_back_ip",
        [],
        "shortlist",
      ),
    ).toBe(false);
  });

  it("keeps Moneyball basic identity and value sorts while rejecting CA and PA", () => {
    expect(isVisibleSortField("name", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("age", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("nationality", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("club", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("division", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("value", [], "moneyball")).toBe(true);
    expect(isVisibleSortField("ca", [], "moneyball")).toBe(false);
    expect(isVisibleSortField("pa", [], "moneyball")).toBe(false);
    expect(
      isVisibleSortField("moneyball.average_rating", [], "moneyball"),
    ).toBe(true);
  });

  it("mirrors General dynamic columns for Shortlist and rejects Moneyball fields", () => {
    const generalFields = [
      {
        id: createFilterRuleId(),
        field: "role.deep_lying_playmaker_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
      {
        id: createFilterRuleId(),
        field: "potential_role.goalkeeper_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
      {
        id: createFilterRuleId(),
        field: "attr.Acceleration",
        op: "gt",
        value: { type: "integer" as const, value: 12 },
      },
      {
        id: createFilterRuleId(),
        field: "club_dna",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];

    expect(dynamicColumnFields(generalFields, "shortlist")).toEqual(
      dynamicColumnFields(generalFields, "general"),
    );
    expect(dynamicColumnFields(generalFields, "shortlist")).toEqual([
      "role.deep_lying_playmaker_ip",
      "potential_role.goalkeeper_ip",
      "attr.Acceleration",
      "club_dna",
    ]);

    const moneyballFields = [
      {
        id: createFilterRuleId(),
        field: "moneyball.goals",
        op: "gt",
        value: { type: "integer" as const, value: 5 },
      },
      {
        id: createFilterRuleId(),
        field: "moneyball_role.wbl_wbr_wing_back_ip",
        op: "gt",
        value: { type: "integer" as const, value: 70 },
      },
    ];
    expect(dynamicColumnFields(moneyballFields, "shortlist")).toEqual([]);
    expect(dynamicColumnFields(moneyballFields, "general")).toEqual([]);
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
