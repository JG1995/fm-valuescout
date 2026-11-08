import { describe, expect, it } from "vitest";
import type { FilterRule } from "../types/filter-rule";
import { isFilterRuleComplete } from "./filter-registry";
import { formatFilterTagLabel } from "./format-filter-label";
import {
  MAX_FILTER_RULES,
  parseSearchCombine,
  parseSearchFilters,
  parseShortlistOnly,
  searchFiltersForUrl,
} from "./search-url-search";

describe("search URL search params", () => {
  it("round-trips filter rules through flat URL encoding", () => {
    const rules: FilterRule[] = [
      {
        id: "rule-1",
        field: "ca",
        op: "gte",
        value: { type: "integer", value: 150 },
      },
      {
        id: "rule-2",
        field: "name",
        op: "contains",
        value: { type: "text", value: "Alex" },
      },
    ];

    const encoded = searchFiltersForUrl(rules);
    expect(encoded).toEqual([
      { id: "rule-1", field: "ca", op: "gte", value: 150 },
      { id: "rule-2", field: "name", op: "contains", value: "Alex" },
    ]);

    const decoded = parseSearchFilters(encoded);
    expect(decoded).toEqual(rules);
  });

  it("parses combine mode and defaults invalid values to and", () => {
    expect(parseSearchCombine("or")).toBe("or");
    expect(parseSearchCombine("and")).toBe("and");
    expect(parseSearchCombine(undefined)).toBe("and");
    expect(parseSearchCombine("xor")).toBe("and");
  });

  it("caps parsed filter rules at MAX_FILTER_RULES", () => {
    const tooMany = Array.from(
      { length: MAX_FILTER_RULES + 5 },
      (_, index) => ({
        id: `r-${index}`,
        field: "ca",
        op: "gte",
        value: index,
      }),
    );

    const parsed = parseSearchFilters(tooMany);
    expect(parsed).toHaveLength(MAX_FILTER_RULES);
    expect(parsed[0]?.value).toEqual({ type: "integer", value: 0 });
    expect(parsed[MAX_FILTER_RULES - 1]?.value).toEqual({
      type: "integer",
      value: MAX_FILTER_RULES - 1,
    });
  });

  it("drops unknown fields and malformed entries", () => {
    const parsed = parseSearchFilters([
      { id: "ok", field: "ca", op: "gte", value: 100 },
      { id: "bad-field", field: "not_a_field", op: "gte", value: 1 },
      { id: "bad-shape", field: "ca" },
      "not-an-object",
      null,
    ]);

    expect(parsed).toEqual([
      {
        id: "ok",
        field: "ca",
        op: "gte",
        value: { type: "integer", value: 100 },
      },
    ]);
  });

  it("keeps percentile rules only on Moneyball statistics", () => {
    const parsed = parseSearchFilters(
      [
        { id: "xa", field: "moneyball.xa_per_90", op: "pct_gte", value: 90 },
        { id: "minutes", field: "moneyball.minutes", op: "pct_gte", value: 90 },
      ],
      "moneyball",
    );

    expect(parsed).toEqual([
      {
        id: "xa",
        field: "moneyball.xa_per_90",
        op: "pct_gte",
        value: { type: "number", value: 90 },
      },
    ]);
    expect(formatFilterTagLabel(parsed[0] as FilterRule, "moneyball")).toBe(
      "xA / 90 percentile ≥ 90",
    );
  });

  it("labels Moneyball role chips as roles, like General role chips", () => {
    expect(
      formatFilterTagLabel(
        {
          id: "role",
          field: "moneyball_role.wbl_wbr_wing_back_ip",
          op: "gt",
          value: { type: "integer", value: 70 },
        },
        "moneyball",
      ),
    ).toBe("Role · Wing-Back (IP · WBR/WBL) > 70");
  });

  it("migrates old comparison links to inclusive operators and choice lists", () => {
    const strip = (rules: FilterRule[]) =>
      rules.map(({ field, op, value }) => ({ field, op, value }));

    expect(
      strip(
        parseSearchFilters([
          { field: "ca", op: "gt", value: 150 },
          { field: "age", op: "lt", value: 23 },
          { field: "pa", op: "neq", value: 150 },
          { field: "preferred_foot", op: "is", value: "left" },
          { field: "position", op: "is_not", value: "GK" },
        ]),
      ),
    ).toEqual([
      { field: "ca", op: "gte", value: { type: "integer", value: 151 } },
      { field: "age", op: "lte", value: { type: "integer", value: 22 } },
      {
        field: "preferred_foot",
        op: "in",
        value: { type: "list", value: ["left"] },
      },
      {
        field: "position",
        op: "not_in",
        value: { type: "list", value: ["GK"] },
      },
    ]);
    expect(
      strip(
        parseSearchFilters(
          [{ field: "moneyball.xa_per_90", op: "gt", value: 0.3 }],
          "moneyball",
        ),
      ),
    ).toEqual([
      {
        field: "moneyball.xa_per_90",
        op: "gte",
        value: { type: "number", value: 0.3 },
      },
    ]);
  });

  it("parses ranges and choice lists and labels their chips", () => {
    const general = parseSearchFilters([
      { field: "age", op: "between", value: [18, 23] },
      { field: "position", op: "in", value: ["DR", "DL"] },
      { field: "position", op: "not_in", value: ["GK"] },
      { field: "ca", op: "gte", value: 140 },
      { field: "age", op: "between", value: [23] },
    ]);
    expect(general.map((rule) => formatFilterTagLabel(rule))).toEqual([
      "Age 18–23",
      "Position: DR, DL",
      "Position: not GK",
      "CA ≥ 140",
    ]);
    expect(searchFiltersForUrl(general).map((rule) => rule.value)).toEqual([
      [18, 23],
      ["DR", "DL"],
      ["GK"],
      140,
    ]);

    const moneyball = parseSearchFilters(
      [
        { field: "moneyball.xa_per_90", op: "pct_between", value: [80, 100] },
        {
          field: "moneyball.pass_completion_ratio",
          op: "between",
          value: [70, 90],
        },
      ],
      "moneyball",
    );
    expect(
      moneyball.map((rule) => formatFilterTagLabel(rule, "moneyball")),
    ).toEqual(["xA / 90 percentile 80–100", "Pass Completion % 70–90%"]);
  });

  it("treats only ordered, in-scale ranges and non-empty valid choices as complete", () => {
    const complete = (
      rule: Omit<FilterRule, "id">,
      view: "general" | "moneyball" = "general",
    ) => isFilterRuleComplete({ id: "rule", ...rule }, view);

    expect(
      complete({
        field: "age",
        op: "between",
        value: { type: "range", value: [18, 23] },
      }),
    ).toBe(true);
    expect(
      complete({
        field: "age",
        op: "between",
        value: { type: "range", value: [23, 18] },
      }),
    ).toBe(false);
    expect(
      complete({
        field: "age",
        op: "between",
        value: { type: "range", value: [18, Number.NaN] },
      }),
    ).toBe(false);
    expect(
      complete(
        {
          field: "moneyball.xa_per_90",
          op: "pct_between",
          value: { type: "range", value: [80, 120] },
        },
        "moneyball",
      ),
    ).toBe(false);
    expect(
      complete({
        field: "position",
        op: "in",
        value: { type: "list", value: [] },
      }),
    ).toBe(false);
    expect(
      complete({
        field: "position",
        op: "in",
        value: { type: "list", value: ["XX"] },
      }),
    ).toBe(false);
    expect(
      complete({
        field: "position",
        op: "in",
        value: { type: "list", value: ["DR"] },
      }),
    ).toBe(true);
  });

  it("parses the shortlist toggle as tri-state with deliberate invalid behavior", () => {
    expect(parseShortlistOnly(true)).toBe(true);
    expect(parseShortlistOnly("true")).toBe(true);
    expect(parseShortlistOnly(false)).toBe(false);
    expect(parseShortlistOnly("false")).toBe(false);
    // Missing and invalid values leave the choice unset so the per-save
    // Zustand choice / presence default can restore it.
    expect(parseShortlistOnly(undefined)).toBeUndefined();
    expect(parseShortlistOnly("yes")).toBeUndefined();
    expect(parseShortlistOnly(1)).toBeUndefined();
  });
});
