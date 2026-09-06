import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import {
  type ConfigurableTableColumn,
  ConfigurableTableHeader,
} from "./player-table-header";
import { ConfigurableVirtualizedTable } from "./virtualized-player-table";

const { virtualizerOptionsSeen, scrollToIndexCalls } = vi.hoisted(() => ({
  virtualizerOptionsSeen: [] as Array<{ scrollPaddingStart: unknown }>,
  scrollToIndexCalls: [] as Array<{ index: number; align: unknown }>,
}));

vi.mock("@tanstack/react-virtual", async (importOriginal) => {
  const actual =
    await importOriginal<typeof import("@tanstack/react-virtual")>();
  return {
    ...actual,
    useVirtualizer: (options: { scrollPaddingStart?: unknown }) => {
      virtualizerOptionsSeen.push({
        scrollPaddingStart: options?.scrollPaddingStart,
      });
      const instance = actual.useVirtualizer(options as never);
      const scrollToIndex = instance.scrollToIndex.bind(instance);
      instance.scrollToIndex = ((
        index: number,
        config?: { align?: unknown },
      ) => {
        scrollToIndexCalls.push({ index, align: config?.align });
        return scrollToIndex(index, config as never);
      }) as typeof instance.scrollToIndex;
      return instance;
    },
  };
});

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

describe("grouped table header contract", () => {
  it("keeps keyboard-focused rows below the two-row sticky header", async () => {
    virtualizerOptionsSeen.length = 0;
    scrollToIndexCalls.length = 0;
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    type KeyboardRow = { uid: string; name: string };
    type KeyboardPage = { rows: KeyboardRow[]; total: number };
    const total = 120;
    const pageSize = 50;

    render(
      <QueryClientProvider client={queryClient}>
        <ConfigurableVirtualizedTable
          caption="Staff keyboard rows"
          columnCount={1}
          columns={[columns[0]]}
          getPageRows={(page: KeyboardPage) => page.rows}
          header={
            <ConfigurableTableHeader
              columns={columns}
              metrics={STAFF_METRICS}
              sortBy="role.scout"
              sortDir="desc"
              onSortChange={vi.fn()}
              onAddColumn={vi.fn()}
              onRemoveColumn={vi.fn()}
              onMoveColumn={vi.fn()}
              onResizeColumn={vi.fn()}
              groups={{
                groups: [{ id: "role-fit", label: "Role Fit" }],
                groupForColumn: () => "role-fit",
              }}
            />
          }
          onRowActivate={vi.fn()}
          pageQueryOptions={(offset, limit) => ({
            queryKey: ["staff", "keyboard", offset],
            queryFn: async (): Promise<KeyboardPage> => ({
              rows: Array.from(
                { length: Math.min(limit, total - offset) },
                (_, index) => ({
                  uid: `staff-${offset + index}`,
                  name: `Coach ${offset + index}`,
                }),
              ),
              total,
            }),
          })}
          pageSize={pageSize}
          renderCells={(row) => <td>{row?.name ?? "…"}</td>}
          testId="staff-keyboard-rows-scroller"
          getRowKey={(row: KeyboardRow) => row.uid}
          total={total}
        />
      </QueryClientProvider>,
    );

    const table = await screen.findByRole("table", {
      name: "Staff keyboard rows",
    });

    // The virtualizer must reserve the full two-row sticky header height,
    // not a single 32px row, so scroll-into-view never hides a focused row.
    expect(virtualizerOptionsSeen.at(-1)?.scrollPaddingStart).toBe(64);

    const firstRow = await within(table).findByText("Coach 0");
    (firstRow.closest("tr") as HTMLElement).focus();
    const scroller = screen.getByTestId("staff-keyboard-rows-scroller");
    // Arrow keys move row focus inside the rendered window.
    for (let step = 0; step < 5; step += 1) {
      fireEvent.keyDown(document.activeElement as HTMLElement, {
        key: "ArrowDown",
      });
    }
    expect(
      (document.activeElement as HTMLElement).getAttribute("data-index"),
    ).toBe("5");

    // Past the rendered window, ArrowDown scrolls the focused row into view
    // below the sticky header instead of dropping focus.
    const renderedIndexes = Array.from(
      table.querySelectorAll("tr[data-index]"),
    ).map((row) => Number(row.getAttribute("data-index")));
    const edge = Math.max(...renderedIndexes);
    (table.querySelector(`tr[data-index="${edge}"]`) as HTMLElement).focus();
    fireEvent.keyDown(document.activeElement as HTMLElement, {
      key: "ArrowDown",
    });
    expect(scrollToIndexCalls.length).toBeGreaterThan(0);
    expect(scrollToIndexCalls.every((call) => call.align === "auto")).toBe(
      true,
    );
    // Browser half of scroll-into-view: the virtualizer's scroll listener
    // advances the window (jsdom neither scrolls on focus nor implements
    // element.scrollTo, so the scroll position is moved by hand).
    scroller.scrollTop = (edge + 1) * 40;
    fireEvent.scroll(scroller);
    expect(
      (document.activeElement as HTMLElement).getAttribute("data-index"),
    ).toBe(String(edge + 1));
  });

  it("composes caller-owned groups without changing fixed-column behavior", () => {
    render(
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
        groups={{
          groups: [{ id: "role-fit", label: "Role Fit" }],
          groupForColumn: () => "role-fit",
        }}
      />,
    );

    const group = screen.getByRole("columnheader", { name: "Role Fit" });
    expect(group).toHaveAttribute("scope", "colgroup");
    expect(group.getAttribute("colspan")).toBe("1");
    const actions = screen.getByRole("columnheader", { name: "Actions" });
    expect(actions.getAttribute("rowspan")).toBe("2");
    fireEvent.contextMenu(actions);
    expect(screen.queryByRole("menu")).toBeNull();
  });
});
