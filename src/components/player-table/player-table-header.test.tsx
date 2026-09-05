import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  type ConfigurableTableColumn,
  ConfigurableTableHeader,
  type ConfigurableTableMetric,
} from "./player-table-header";

const METRICS: ConfigurableTableMetric[] = [
  {
    id: "ca",
    label: "CA",
    align: "right",
    defaultWidth: 72,
    sortable: true,
  },
  {
    id: "suggested_training",
    label: "Suggested Training",
    align: "left",
    defaultWidth: 176,
    sortable: false,
  },
];

const COLUMNS: ConfigurableTableColumn[] = [
  { id: "ca", label: "CA", align: "right", width: 72 },
  {
    id: "suggested_training",
    label: "Suggested Training",
    align: "left",
    width: 176,
  },
];

function renderHeader({
  columns = COLUMNS,
  sortBy = "ca",
}: {
  columns?: ConfigurableTableColumn[];
  sortBy?: string;
} = {}) {
  const onSortChange = vi.fn();
  const onAddColumn = vi.fn();
  render(
    <ConfigurableTableHeader
      columns={columns}
      metrics={METRICS}
      sortBy={sortBy}
      sortDir="desc"
      onSortChange={onSortChange}
      onAddColumn={onAddColumn}
      onRemoveColumn={vi.fn()}
      onMoveColumn={vi.fn()}
      onResizeColumn={vi.fn()}
    />,
  );
  return { onSortChange, onAddColumn };
}

describe("player table header per-column sortability", () => {
  it("gives a non-sortable column no sort click, title, or aria-sort", async () => {
    const user = userEvent.setup();
    const { onSortChange } = renderHeader();

    const header = screen.getByRole("columnheader", {
      name: "Suggested Training",
    });
    expect(header).not.toHaveAttribute("aria-sort");
    const button = within(header).getByRole("button", {
      name: "Suggested Training",
    });
    expect(button).toHaveAttribute("title", "Suggested Training");
    await user.click(button);
    expect(onSortChange).not.toHaveBeenCalled();
  });

  it("keeps sortable columns clickable with title and aria-sort", async () => {
    const user = userEvent.setup();
    const { onSortChange } = renderHeader();

    const header = screen.getByRole("columnheader", { name: "CA" });
    expect(header).toHaveAttribute("aria-sort", "descending");
    const button = within(header).getByRole("button", { name: "CA" });
    expect(button.getAttribute("title")).toContain("click to sort");
    await user.click(button);
    expect(onSortChange).toHaveBeenCalledWith("ca");
  });

  it("lists a valid non-sortable metric in the Add column menu", async () => {
    const user = userEvent.setup();
    const { onAddColumn } = renderHeader({ columns: [COLUMNS[0]] });

    fireEvent.contextMenu(screen.getByRole("columnheader", { name: "CA" }));
    await user.click(screen.getByRole("menuitem", { name: "Add column" }));
    await user.click(
      screen.getByRole("button", { name: "Column: Choose a metric" }),
    );
    await user.type(
      screen.getByRole("combobox", { name: "Search columns" }),
      "training",
    );
    await user.click(
      screen.getByRole("option", { name: "Suggested Training" }),
    );

    expect(onAddColumn).toHaveBeenCalledWith("suggested_training");
  });
});
