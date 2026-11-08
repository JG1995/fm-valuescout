import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { FilterRule } from "../types/filter-rule";
import { SearchFilterEditorModal } from "./search-filter-editor-modal";

const appliedRule: FilterRule = {
  id: "applied-ca",
  field: "ca",
  op: "gte",
  value: { type: "integer", value: 150 },
};

type ApplyFilters = (rules: FilterRule[], combine: "and" | "or") => void;
type CloseEditor = () => void;

function renderEditor({
  open = true,
  rules = [],
  combine = "and" as const,
  view = "general" as const,
  onApply = vi.fn<ApplyFilters>(),
  onClose = vi.fn<CloseEditor>(),
}: {
  open?: boolean;
  rules?: FilterRule[];
  combine?: "and" | "or";
  view?: "general" | "moneyball";
  onApply?: ApplyFilters;
  onClose?: CloseEditor;
} = {}) {
  return {
    onApply,
    onClose,
    ...render(
      <SearchFilterEditorModal
        open={open}
        rules={rules}
        combine={combine}
        view={view}
        onApply={onApply}
        onClose={onClose}
      />,
    ),
  };
}

describe("SearchFilterEditorModal", () => {
  it("keeps every draft edit local and applies one complete draft on Done", async () => {
    const user = userEvent.setup();
    const { onApply } = renderEditor();
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });

    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );
    const picker = within(dialog).getByRole("listbox", {
      name: "Field options",
    });
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "club",
    );
    await user.click(within(picker).getByRole("option", { name: "Club" }));

    await user.selectOptions(
      within(dialog).getByRole("combobox", { name: "Operator" }),
      "is",
    );
    await user.type(within(dialog).getByLabelText("Value"), "Rangers");
    await user.click(within(dialog).getByRole("button", { name: "or" }));

    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );
    const removeButtons = within(dialog).getAllByRole("button", {
      name: "Remove filter rule",
    });
    const secondRule = removeButtons[1];
    expect(secondRule).toBeDefined();
    if (!secondRule) {
      throw new Error("expected the second draft rule");
    }
    await user.click(secondRule);

    expect(onApply).not.toHaveBeenCalled();

    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    expect(onApply).toHaveBeenCalledOnce();
    expect(onApply).toHaveBeenCalledWith(
      [
        expect.objectContaining({
          field: "club",
          op: "is",
          value: { type: "text", value: "Rangers" },
        }),
      ],
      "or",
    );
  });

  it("opens an empty editor on one blank rule that Done ignores", async () => {
    const user = userEvent.setup();
    const { onApply } = renderEditor();
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });

    expect(
      within(dialog).getByRole("button", { name: "Field: Choose a field" }),
    ).toBeInTheDocument();
    expect(within(dialog).queryByLabelText("Value")).toBeNull();

    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    expect(onApply).toHaveBeenCalledWith([], "and");
  });

  it("adds a blank rule with the field picker open and replaces an untouched blank rule", async () => {
    const user = userEvent.setup();
    renderEditor({ rules: [appliedRule] });
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });

    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );
    expect(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
    ).toHaveFocus();

    await user.click(
      within(dialog).getByRole("heading", { name: "Edit filters" }),
    );
    expect(
      within(dialog).queryByRole("listbox", { name: "Field options" }),
    ).toBeNull();

    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );
    expect(
      within(dialog).getAllByRole("button", { name: "Remove filter rule" }),
    ).toHaveLength(2);
    expect(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
    ).toHaveFocus();
  });

  it("marks an incomplete rule when Done is selected and applies once it is fixed", async () => {
    const user = userEvent.setup();
    const { onApply } = renderEditor({ rules: [appliedRule] });
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });

    await user.click(
      within(dialog).getByRole("button", { name: "Add filter" }),
    );
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "pace",
    );
    await user.keyboard("{Enter}");

    const values = within(dialog).getAllByLabelText("Value");
    const paceValue = values[1];
    if (!paceValue) {
      throw new Error("expected the Pace value field");
    }
    expect(paceValue).toHaveValue(null);
    expect(within(dialog).queryByText("Enter a number")).toBeNull();

    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    expect(onApply).not.toHaveBeenCalled();
    expect(paceValue).toHaveAttribute("aria-invalid", "true");
    expect(paceValue).toHaveAccessibleDescription("Enter a number");
    expect(paceValue).toHaveFocus();

    await user.type(paceValue, "15");
    expect(within(dialog).queryByText("Enter a number")).toBeNull();
    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    expect(onApply).toHaveBeenCalledWith(
      [
        appliedRule,
        expect.objectContaining({
          field: "attr.Pace",
          op: "gte",
          value: { type: "integer", value: 15 },
        }),
      ],
      "and",
    );
  });

  it("edits a min–max range and a set of position choices", async () => {
    const user = userEvent.setup();
    const { onApply } = renderEditor();
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });
    const chooseField = async (search: string, option: string) => {
      await user.click(
        within(dialog).getByRole("button", { name: "Add filter" }),
      );
      await user.type(
        within(dialog).getByRole("combobox", { name: "Search fields" }),
        search,
      );
      await user.click(within(dialog).getByRole("option", { name: option }));
    };

    await chooseField("age", "Age");
    await user.selectOptions(
      within(dialog).getByRole("combobox", { name: "Operator" }),
      "between",
    );
    const min = within(dialog).getByLabelText("Min");
    await user.type(min, "23");
    await user.type(within(dialog).getByLabelText("Max"), "18");

    await chooseField("position", "Position");
    await user.click(within(dialog).getByRole("checkbox", { name: "DR" }));
    await user.click(within(dialog).getByRole("checkbox", { name: "DL" }));

    await user.click(within(dialog).getByRole("button", { name: "Done" }));
    expect(onApply).not.toHaveBeenCalled();
    expect(min).toHaveAccessibleDescription("Enter both numbers, lowest first");

    await user.clear(min);
    await user.type(min, "16");
    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    expect(onApply).toHaveBeenCalledWith(
      [
        expect.objectContaining({
          field: "age",
          op: "between",
          value: { type: "range", value: [16, 18] },
        }),
        expect.objectContaining({
          field: "position",
          op: "in",
          value: { type: "list", value: ["DR", "DL"] },
        }),
      ],
      "and",
    );
  });

  it.each([
    [
      "Cancel",
      async (user: ReturnType<typeof userEvent.setup>) => {
        await user.click(screen.getByRole("button", { name: "Cancel" }));
      },
    ],
    [
      "close control",
      async (user: ReturnType<typeof userEvent.setup>) => {
        await user.click(screen.getByRole("button", { name: "Close" }));
      },
    ],
    [
      "Escape",
      async (user: ReturnType<typeof userEvent.setup>) => {
        await user.keyboard("{Escape}");
      },
    ],
    [
      "backdrop",
      async (user: ReturnType<typeof userEvent.setup>) => {
        await user.click(screen.getByRole("button", { name: "Close dialog" }));
      },
    ],
  ])("discards drafts on %s", async (_name, dismiss) => {
    const user = userEvent.setup();
    const { onApply, onClose } = renderEditor();

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    await user.type(
      screen.getByRole("combobox", { name: "Search fields" }),
      "club",
    );
    await user.keyboard("{Enter}");
    await dismiss(user);

    expect(onApply).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("reopens from the latest applied rules instead of a dismissed draft", async () => {
    const user = userEvent.setup();
    const { rerender } = renderEditor();

    await user.click(screen.getByRole("button", { name: "Add filter" }));
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    rerender(
      <SearchFilterEditorModal
        open={false}
        rules={[appliedRule]}
        combine="or"
        onApply={vi.fn()}
        onClose={vi.fn()}
      />,
    );
    rerender(
      <SearchFilterEditorModal
        open
        rules={[appliedRule]}
        combine="or"
        onApply={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    expect(
      await screen.findByRole("button", { name: "Field: CA" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "or" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });

  it("searches grouped current role scores and selects them by keyboard", async () => {
    const user = userEvent.setup();
    renderEditor({ rules: [appliedRule] });

    const fieldTrigger = screen.getByRole("button", { name: "Field: CA" });
    await user.click(fieldTrigger);
    const search = screen.getByRole("combobox", { name: "Search fields" });
    await user.type(search, "goalkeeper");

    expect(
      screen.getByRole("group", {
        name: "Current role scores · Goalkeepers",
      }),
    ).toBeInTheDocument();
    await user.keyboard("{Enter}");

    expect(
      screen.getByRole("button", { name: "Field: Role · Goalkeeper (IP)" }),
    ).toBeInTheDocument();
    expect(fieldTrigger).toHaveFocus();
  });

  it("returns focus to the field trigger when the picker is dismissed", async () => {
    const user = userEvent.setup();
    renderEditor({ rules: [appliedRule] });

    const fieldTrigger = screen.getByRole("button", { name: "Field: CA" });
    await user.click(fieldTrigger);
    expect(
      screen.getByRole("combobox", { name: "Search fields" }),
    ).toHaveFocus();

    await user.keyboard("{Escape}");

    expect(fieldTrigger).toHaveFocus();
  });

  it("switches a Moneyball metric rule to a 0–100 percentile", async () => {
    const user = userEvent.setup();
    const { onApply } = renderEditor({
      view: "moneyball",
      rules: [
        {
          id: "xa",
          field: "moneyball.xa_per_90",
          op: "gte",
          value: { type: "number", value: 0.35 },
        },
      ],
    });
    const dialog = screen.getByRole("dialog", { name: "Edit filters" });

    await user.selectOptions(
      within(dialog).getByRole("combobox", { name: "Operator" }),
      "pct_gte",
    );
    const value = within(dialog).getByLabelText("Value");
    expect(value).toHaveValue(90);
    expect(
      within(dialog).getByRole("note", {
        name: /role filters apply after the comparison cohort is calculated/i,
      }),
    ).toBeInTheDocument();

    await user.clear(value);
    await user.type(value, "150");
    await user.click(within(dialog).getByRole("button", { name: "Done" }));
    expect(onApply).not.toHaveBeenCalled();
    expect(value).toHaveAccessibleDescription(
      "Enter a percentile from 0 to 100",
    );

    await user.clear(value);
    await user.type(value, "75");
    await user.click(within(dialog).getByRole("button", { name: "Done" }));

    expect(onApply).toHaveBeenCalledWith(
      [
        expect.objectContaining({
          field: "moneyball.xa_per_90",
          op: "pct_gte",
          value: { type: "number", value: 75 },
        }),
      ],
      "and",
    );
  });

  it("explains the post-score cohort when a Moneyball role rule is present", () => {
    renderEditor({
      view: "moneyball",
      rules: [
        {
          id: "moneyball-role",
          field: "moneyball_role.wbl_wbr_wing_back_ip",
          op: "gte",
          value: { type: "integer", value: 70 },
        },
      ],
    });

    expect(
      screen.getByRole("note", {
        name: /role filters apply after the comparison cohort is calculated/i,
      }),
    ).toHaveTextContent(/With AND they narrow that scored cohort/i);
  });
});
