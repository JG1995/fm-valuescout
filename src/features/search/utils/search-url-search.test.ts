import { describe, expect, it } from "vitest";
import type { FilterRule } from "../types/filter-rule";
import {
  MAX_FILTER_RULES,
  parseSearchCombine,
  parseSearchFilters,
  searchFiltersForUrl,
} from "./search-url-search";

describe("search URL search params", () => {
  it("round-trips filter rules through flat URL encoding", () => {
    const rules: FilterRule[] = [
      {
        id: "rule-1",
        field: "ca",
        op: "gt",
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
      { id: "rule-1", field: "ca", op: "gt", value: 150 },
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
        op: "gt",
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
      { id: "ok", field: "ca", op: "gt", value: 100 },
      { id: "bad-field", field: "not_a_field", op: "gt", value: 1 },
      { id: "bad-shape", field: "ca" },
      "not-an-object",
      null,
    ]);

    expect(parsed).toEqual([
      {
        id: "ok",
        field: "ca",
        op: "gt",
        value: { type: "integer", value: 100 },
      },
    ]);
  });
});
