import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { TableToolbar, type TableToolbarProps } from "./table-toolbar";

function renderToolbar(overrides: Partial<TableToolbarProps> = {}) {
  const onClearAll = vi.fn();
  const onEditFilters = vi.fn();
  render(
    <TableToolbar
      toolbarLabel="Player results toolbar"
      summary={<p>194,417 players · sorted by CA (descending)</p>}
      filterChips={<span>Age 16–30</span>}
      onClearAll={onClearAll}
      onEditFilters={onEditFilters}
      columnsControl={<button type="button">Columns</button>}
      datasetToggles={<button type="button">Shortlist: Off</button>}
      {...overrides}
    />,
  );
  return { onClearAll, onEditFilters };
}

describe("table toolbar contract", () => {
  it("renders summary, chips, clear, edit, columns, and dataset toggles in one labelled toolbar", () => {
    renderToolbar();

    const toolbar = screen.getByRole("toolbar", {
      name: "Player results toolbar",
    });
    expect(
      within(toolbar).getByText(/194,417 players · sorted by CA/),
    ).toBeInTheDocument();
    expect(within(toolbar).getByText("Age 16–30")).toBeInTheDocument();
    expect(
      within(toolbar).getByRole("button", { name: "Clear all" }),
    ).toBeInTheDocument();
    expect(
      within(toolbar).getByRole("button", { name: "Edit filters" }),
    ).toBeInTheDocument();
    expect(
      within(toolbar).getByRole("button", { name: "Columns" }),
    ).toBeInTheDocument();
    expect(
      within(toolbar).getByRole("button", { name: "Shortlist: Off" }),
    ).toBeInTheDocument();
  });

  it("clears and edits through caller callbacks", async () => {
    const user = userEvent.setup();
    const { onClearAll, onEditFilters } = renderToolbar();

    await user.click(screen.getByRole("button", { name: "Clear all" }));
    expect(onClearAll).toHaveBeenCalledTimes(1);
    await user.click(screen.getByRole("button", { name: "Edit filters" }));
    expect(onEditFilters).toHaveBeenCalledTimes(1);
  });

  it("omits Clear all and Edit affordances when the caller supplies no callbacks", () => {
    renderToolbar({ onClearAll: undefined, onEditFilters: undefined });

    const toolbar = screen.getByRole("toolbar", {
      name: "Player results toolbar",
    });
    expect(
      within(toolbar).queryByRole("button", { name: "Clear all" }),
    ).toBeNull();
    expect(
      within(toolbar).queryByRole("button", { name: "Edit filters" }),
    ).toBeNull();
    // Dataset slots still render without filter actions.
    expect(
      within(toolbar).getByRole("button", { name: "Columns" }),
    ).toBeInTheDocument();
  });
});
