import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { FilterRule } from "../types/filter-rule";
import { SearchFilterStrip } from "./search-filter-strip";

const ca: FilterRule = {
  id: "ca",
  field: "ca",
  op: "gte",
  value: { type: "integer", value: 150 },
};
const age: FilterRule = {
  id: "age",
  field: "age",
  op: "lte",
  value: { type: "integer", value: 23 },
};

describe("SearchFilterStrip", () => {
  it("names the combine mode only when it joins two or more rules", () => {
    const { rerender } = render(
      <SearchFilterStrip rules={[ca]} combine="or" onRulesChange={vi.fn()} />,
    );
    expect(screen.queryByText(/combined with/i)).toBeNull();

    rerender(
      <SearchFilterStrip
        rules={[ca, age]}
        combine="or"
        onRulesChange={vi.fn()}
      />,
    );
    expect(screen.getByText("Combined with or")).toBeInTheDocument();
  });
});
