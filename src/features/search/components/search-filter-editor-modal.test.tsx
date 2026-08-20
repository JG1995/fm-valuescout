import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { FilterRule } from "../types/filter-rule";
import { SearchFilterEditorModal } from "./search-filter-editor-modal";

const appliedRule: FilterRule = {
  id: "applied-ca",
  field: "ca",
  op: "gt",
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

    await user.click(within(dialog).getByRole("button", { name: "Field: CA" }));
    const picker = within(dialog).getByRole("listbox", {
      name: "Field options",
    });
    await user.type(
      within(dialog).getByRole("combobox", { name: "Search fields" }),
      "club",
    );
    await user.click(within(picker).getByRole("option", { name: "Club" }));
    expect(within(dialog).getByRole("button", { name: "Done" })).toBeDisabled();

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

  it("explains the post-score cohort when a Moneyball role rule is present", () => {
    renderEditor({
      view: "moneyball",
      rules: [
        {
          id: "moneyball-role",
          field: "moneyball_role.wbl_wbr_wing_back_ip",
          op: "gt",
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
