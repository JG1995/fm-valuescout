import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { FilterRuleIpc } from "../types/filter-rule";
import { RecruitmentFilterEditor } from "./recruitment-filter-editor";

function renderEditor(filters: FilterRuleIpc[]) {
  const onChange = vi.fn<(filters: FilterRuleIpc[]) => void>();
  render(<RecruitmentFilterEditor filters={filters} onChange={onChange} />);
  return { onChange, user: userEvent.setup() };
}

describe("RecruitmentFilterEditor", () => {
  it("adds a percentile rule ranked among the slot's peers", async () => {
    const height = { field: "height", op: "gte", value: 183 };
    const { onChange, user } = renderEditor([height]);

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    await user.type(
      screen.getByRole("combobox", { name: "Search fields" }),
      "progressive passes",
    );
    await user.click(
      screen.getByRole("option", { name: "Progressive Passes" }),
    );
    await user.click(screen.getByRole("button", { name: "Percentile" }));
    expect(
      screen.getByRole("combobox", { name: "Operator" }),
    ).toHaveAccessibleDescription(
      /^Ranked among players with 450\+ minutes rated 15 or higher in the position families/,
    );
    const value = screen.getByLabelText("Value");
    await user.clear(value);
    await user.type(value, "25");
    await user.click(screen.getByRole("button", { name: "Add" }));

    expect(onChange).toHaveBeenCalledWith([
      height,
      { field: "moneyball.progressive_passes", op: "pct_gte", value: 25 },
    ]);
  });

  it("edits and removes a rule from its chip", async () => {
    const height = { field: "height", op: "gte", value: 183 };
    const foot = { field: "preferred_foot", op: "in", value: ["right"] };
    const { onChange, user } = renderEditor([height, foot]);

    await user.click(
      screen.getByRole("button", { name: "Edit filter Height ≥ 183" }),
    );
    const value = screen.getByLabelText("Value");
    await user.clear(value);
    await user.type(value, "185{Enter}");
    expect(onChange).toHaveBeenLastCalledWith([
      { ...height, value: 185 },
      foot,
    ]);

    await user.click(
      screen.getByRole("button", {
        name: "Remove filter Preferred foot: Right",
      }),
    );
    expect(onChange).toHaveBeenLastCalledWith([height]);
  });

  it("stops adding at ten rules", () => {
    renderEditor(
      Array.from({ length: 10 }, () => ({
        field: "height",
        op: "gte",
        value: 170,
      })),
    );

    expect(screen.getByRole("button", { name: "Add filter" })).toBeDisabled();
  });
});
