import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import { SearchFilterBar } from "./search-filter-bar";

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

type Apply = (rules: FilterRule[], combine: FilterCombineMode) => void;

function renderBar({
  rules = [],
  view = "general",
}: {
  rules?: FilterRule[];
  view?: SearchView;
} = {}) {
  const onApply = vi.fn<Apply>();
  const onRulesChange = vi.fn<(rules: FilterRule[]) => void>();
  render(
    <SearchFilterBar
      rules={rules}
      combine="and"
      onApply={onApply}
      onRulesChange={onRulesChange}
      view={view}
      comparisonPool="filtered"
    />,
  );
  return { onApply, onRulesChange, user: userEvent.setup() };
}

async function chooseField(
  user: ReturnType<typeof userEvent.setup>,
  search: string,
  option: string,
) {
  await user.type(
    screen.getByRole("combobox", { name: "Search fields" }),
    search,
  );
  await user.click(screen.getByRole("option", { name: option }));
}

describe("SearchFilterBar", () => {
  it("adds a rule from Add filter: the field list opens, Enter applies the value", async () => {
    const { onApply, user } = renderBar({ rules: [ca] });

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    expect(
      screen.getByRole("combobox", { name: "Search fields" }),
    ).toHaveFocus();
    await user.type(
      screen.getByRole("combobox", { name: "Search fields" }),
      "pace",
    );
    await user.keyboard("{Enter}");

    const value = screen.getByLabelText("Value");
    expect(value).toHaveFocus();
    await user.type(value, "15{Enter}");

    expect(onApply).toHaveBeenCalledOnce();
    expect(onApply).toHaveBeenCalledWith(
      [
        ca,
        expect.objectContaining({
          field: "attr.Pace",
          op: "gte",
          value: { type: "integer", value: 15 },
        }),
      ],
      "and",
    );
    expect(screen.queryByRole("form", { name: "Add filter" })).toBeNull();
  });

  it("edits a rule from its chip and removes it from the editor", async () => {
    const { onApply, onRulesChange, user } = renderBar({ rules: [ca, age] });

    await user.click(
      screen.getByRole("button", { name: "Edit filter CA ≥ 150" }),
    );
    const editor = screen.getByRole("form", { name: "Edit filter CA ≥ 150" });
    const value = within(editor).getByLabelText("Value");
    expect(value).toHaveValue(150);
    expect(value).toHaveFocus();
    await user.clear(value);
    await user.type(value, "160");
    await user.click(within(editor).getByRole("button", { name: "Apply" }));

    expect(onApply).toHaveBeenCalledWith(
      [{ ...ca, value: { type: "integer", value: 160 } }, age],
      "and",
    );

    await user.click(
      screen.getByRole("button", { name: "Edit filter Age ≤ 23" }),
    );
    await user.click(screen.getByRole("button", { name: "Remove" }));
    expect(onRulesChange).toHaveBeenCalledWith([ca]);
  });

  it.each([
    [
      "Cancel",
      (user: ReturnType<typeof userEvent.setup>) =>
        user.click(screen.getByRole("button", { name: "Cancel" })),
    ],
    [
      "Escape",
      (user: ReturnType<typeof userEvent.setup>) => user.keyboard("{Escape}"),
    ],
    [
      "an outside click",
      (user: ReturnType<typeof userEvent.setup>) => user.click(document.body),
    ],
  ])("discards a draft on %s", async (_name, dismiss) => {
    const { onApply, user } = renderBar();

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    await chooseField(user, "club", "Club");
    await user.type(screen.getByLabelText("Value"), "Rangers");
    await dismiss(user);

    expect(onApply).not.toHaveBeenCalled();
    expect(screen.queryByRole("form", { name: "Add filter" })).toBeNull();
  });

  it("shows why a value is incomplete and focuses it", async () => {
    const { onApply, user } = renderBar();

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    await chooseField(user, "club", "Club");
    await user.click(screen.getByRole("button", { name: "Add" }));

    const value = screen.getByLabelText("Value");
    expect(onApply).not.toHaveBeenCalled();
    expect(value).toHaveAccessibleDescription("Enter a value");
    expect(value).toHaveFocus();
  });

  it("edits a min–max range and a set of position choices", async () => {
    const { onApply, user } = renderBar();

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    await chooseField(user, "age", "Age");
    await user.selectOptions(
      screen.getByRole("combobox", { name: "Operator" }),
      "between",
    );
    const min = screen.getByLabelText("Min");
    await user.type(min, "23");
    await user.type(screen.getByLabelText("Max"), "18");
    await user.click(screen.getByRole("button", { name: "Add" }));
    expect(min).toHaveAccessibleDescription("Enter both numbers, lowest first");

    await user.clear(min);
    await user.type(min, "16{Enter}");
    expect(onApply).toHaveBeenLastCalledWith(
      [
        expect.objectContaining({
          field: "age",
          op: "between",
          value: { type: "range", value: [16, 18] },
        }),
      ],
      "and",
    );

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    await chooseField(user, "position", "Position");
    await user.click(screen.getByRole("checkbox", { name: "DR" }));
    await user.click(screen.getByRole("checkbox", { name: "DL" }));
    await user.click(screen.getByRole("button", { name: "Add" }));
    expect(onApply).toHaveBeenLastCalledWith(
      [
        expect.objectContaining({
          field: "position",
          op: "in",
          value: { type: "list", value: ["DR", "DL"] },
        }),
      ],
      "and",
    );
  });

  it("compares a Moneyball statistic by value or by percentile", async () => {
    const { onApply, user } = renderBar({ view: "moneyball" });

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    await chooseField(user, "xa / 90", "xA / 90");
    const operatorValues = () =>
      within(screen.getByRole("combobox", { name: "Operator" }))
        .getAllByRole("option")
        .map((option) => (option as HTMLOptionElement).value);
    expect(operatorValues()).toEqual(["gte", "lte", "between"]);

    await user.click(screen.getByRole("button", { name: "Percentile" }));
    expect(screen.getByLabelText("Value")).toHaveValue(90);
    expect(operatorValues()).toEqual(["pct_gte", "pct_lte", "pct_between"]);
    expect(
      screen.getByText(
        /Scored against the players matching your other filters/,
      ),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Add" }));

    expect(onApply).toHaveBeenCalledWith(
      [
        expect.objectContaining({
          field: "moneyball.xa_per_90",
          op: "pct_gte",
          value: { type: "number", value: 90 },
        }),
      ],
      "and",
    );
  });

  it("sets All or Any once two or more rules apply", async () => {
    const { onApply, user } = renderBar({ rules: [ca, age] });

    const match = screen.getByRole("group", { name: "Match" });
    expect(within(match).getByRole("button", { name: "All" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    await user.click(within(match).getByRole("button", { name: "Any" }));

    expect(onApply).toHaveBeenCalledWith([ca, age], "or");
  });
});
