import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  type ConfigurableTableColumn,
  ConfigurableTableHeader,
} from "./player-table-header";
import { ConfigurableVirtualizedTable } from "./virtualized-player-table";

const STAFF_METRICS = [
  {
    id: "role.scout",
    label: "Scout",
    category: "current-role-scores",
    align: "right" as const,
    defaultWidth: 96,
    sortable: true,
  },
  {
    id: "role.assistant_manager",
    label: "Assistant Manager",
    category: "current-role-scores",
    align: "right" as const,
    defaultWidth: 96,
    sortable: true,
  },
  {
    id: "attr.Adaptability",
    label: "Adaptability",
    category: "staff-attributes",
    align: "right" as const,
    defaultWidth: 96,
    sortable: true,
  },
];

const columns: ConfigurableTableColumn[] = [
  {
    id: "role.scout",
    label: "Scout",
    align: "right",
    width: 96,
  },
];

const FIXED_COLUMNS = [
  {
    id: "actions",
    label: "Actions",
    align: "left" as const,
    width: 128,
  },
];

describe("configurable table contracts", () => {
  it("accepts a caller-owned metric catalog in the header", async () => {
    const user = userEvent.setup();
    const onAddColumn = vi.fn();

    render(
      <ConfigurableTableHeader
        columns={columns}
        metrics={STAFF_METRICS}
        sortBy="role.scout"
        sortDir="desc"
        onSortChange={vi.fn()}
        onAddColumn={onAddColumn}
        onRemoveColumn={vi.fn()}
        onMoveColumn={vi.fn()}
        onResizeColumn={vi.fn()}
      />,
    );

    const header = screen.getByRole("columnheader", { name: "Scout" });
    fireEvent.contextMenu(header);
    await user.click(screen.getByRole("menuitem", { name: "Add column" }));
    await user.click(
      screen.getByRole("button", { name: "Column: Choose a metric" }),
    );
    await user.type(
      screen.getByRole("combobox", { name: "Search columns" }),
      "assistant manager",
    );
    await user.click(screen.getByRole("option", { name: "Assistant Manager" }));

    expect(onAddColumn).toHaveBeenCalledWith("role.assistant_manager");
  });

  it("renders a caller-owned row shape without making rows interactive by default", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    render(
      <QueryClientProvider client={queryClient}>
        <ConfigurableVirtualizedTable
          caption="Staff rows"
          columnCount={1}
          columns={[columns[0]]}
          getPageRows={(page: { rows: Array<{ uid: string; name: string }> }) =>
            page.rows
          }
          header={
            <thead>
              <tr>
                <th scope="col">Scout</th>
              </tr>
            </thead>
          }
          pageQueryOptions={() => ({
            queryKey: ["staff", "rows"],
            queryFn: async () => ({
              rows: [{ uid: "staff-1", name: "Coach One" }],
              total: 1,
            }),
          })}
          pageSize={50}
          renderCells={(row) => <td>{row?.name ?? "…"}</td>}
          testId="staff-rows-scroller"
          total={1}
        />
      </QueryClientProvider>,
    );

    const table = await screen.findByRole("table", { name: "Staff rows" });
    const row = (await within(table).findByText("Coach One")).closest("tr");
    expect(row).not.toHaveClass("cursor-pointer");
    expect(row).not.toHaveAttribute("tabindex");
  });

  it("keeps fixed action cells outside configurable metric controls", async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const onActivate = vi.fn();
    const getRowKey = vi.fn((row: { uid: string }) => row.uid);

    render(
      <QueryClientProvider client={queryClient}>
        <ConfigurableVirtualizedTable
          caption="Staff action rows"
          columnCount={columns.length}
          columns={columns}
          fixedColumns={FIXED_COLUMNS}
          getPageRows={(page: { rows: Array<{ uid: string; name: string }> }) =>
            page.rows
          }
          header={
            <ConfigurableTableHeader
              columns={columns}
              fixedColumns={FIXED_COLUMNS}
              metrics={STAFF_METRICS}
              sortBy="role.scout"
              sortDir="desc"
              onSortChange={vi.fn()}
              onAddColumn={vi.fn()}
              onRemoveColumn={vi.fn()}
              onMoveColumn={vi.fn()}
              onResizeColumn={vi.fn()}
            />
          }
          onRowActivate={onActivate}
          pageQueryOptions={() => ({
            queryKey: ["staff", "fixed"],
            queryFn: async () => ({
              rows: [{ uid: "staff-1", name: "Coach One" }],
              total: 1,
            }),
          })}
          pageSize={50}
          renderCells={(row) => <td>{row?.name ?? "…"}</td>}
          renderFixedCells={() => <td>Boost CA</td>}
          testId="staff-action-rows-scroller"
          getRowKey={getRowKey}
          total={1}
        />
      </QueryClientProvider>,
    );

    const table = await screen.findByRole("table", {
      name: "Staff action rows",
    });
    const actionsHeader = within(table).getByRole("columnheader", {
      name: "Actions",
    });
    expect(actionsHeader).toBeInTheDocument();
    expect(within(table).getByText("Boost CA")).toBeInTheDocument();
    fireEvent.contextMenu(actionsHeader);
    expect(screen.queryByRole("menu")).toBeNull();

    const row = (await within(table).findByText("Coach One")).closest("tr");
    expect(getRowKey).toHaveBeenCalledWith(
      { uid: "staff-1", name: "Coach One" },
      0,
    );
    fireEvent.click(row as HTMLElement);
    expect(onActivate).toHaveBeenCalledWith({
      uid: "staff-1",
      name: "Coach One",
    });
    fireEvent.keyDown(row as HTMLElement, { key: "Enter" });
    expect(onActivate).toHaveBeenCalledTimes(2);
  });
});
