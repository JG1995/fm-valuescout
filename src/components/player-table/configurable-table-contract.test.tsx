import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";
import {
  type ConfigurableTableColumn,
  type ConfigurableTableFixedColumn,
  ConfigurableTableHeader,
} from "./player-table-header";
import {
  type ConfigurableTableIdentity,
  ConfigurableVirtualizedTable,
} from "./virtualized-player-table";

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
          renderHeader={() => (
            <thead>
              <tr>
                <th scope="col">Scout</th>
              </tr>
            </thead>
          )}
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
          renderHeader={() => (
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
          )}
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
          renderHeader={() => (
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
          )}
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

function zLayer(className: string): number {
  const arbitrary = className.match(/z-\[(\d+)\]/);
  if (arbitrary) {
    return Number(arbitrary[1]);
  }
  const flat = className.match(/(?:^|\s)z-(\d+)(?:\s|$)/);
  if (flat) {
    return Number(flat[1]);
  }
  throw new Error(`no z-layer in: ${className}`);
}

describe("sticky identity shell", () => {
  type IdentityRow = { uid: string; name: string };
  type IdentityPage = { rows: IdentityRow[]; total: number };

  function renderIdentityShell({
    identity,
    headerFactory,
  }: {
    identity?: ConfigurableTableIdentity<IdentityRow>;
    headerFactory: (args: {
      identity: ConfigurableTableIdentity<IdentityRow> | undefined;
      columns: readonly ConfigurableTableColumn[];
      fixedColumns: readonly ConfigurableTableFixedColumn[];
    }) => ReactNode;
  }) {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const onRenderHeader = vi.fn(headerFactory);

    render(
      <QueryClientProvider client={queryClient}>
        <ConfigurableVirtualizedTable
          caption="Staff identity rows"
          columnCount={columns.length}
          columns={columns}
          fixedColumns={FIXED_COLUMNS}
          getPageRows={(page: IdentityPage) => page.rows}
          identity={identity}
          renderHeader={onRenderHeader}
          pageQueryOptions={() => ({
            queryKey: ["staff", "identity"],
            queryFn: async (): Promise<IdentityPage> => ({
              rows: [{ uid: "staff-1", name: "Coach One" }],
              total: 1,
            }),
          })}
          pageSize={50}
          renderCells={(row) => <td>{row?.name ?? "…"}</td>}
          renderFixedCells={() => <td>Boost CA</td>}
          testId="staff-identity-rows-scroller"
          getRowKey={(row: IdentityRow) => row.uid}
          total={1}
        />
      </QueryClientProvider>,
    );
    return { onRenderHeader };
  }

  function staffHeaderFactory({
    identity,
    columns: tableColumns,
  }: {
    identity: ConfigurableTableIdentity<IdentityRow> | undefined;
    columns: readonly ConfigurableTableColumn[];
    fixedColumns: readonly ConfigurableTableFixedColumn[];
  }) {
    return (
      <ConfigurableTableHeader
        columns={tableColumns}
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
        identity={identity}
      />
    );
  }

  it("hands the identity object to renderHeader and renders identity first and sticky", async () => {
    const identity: ConfigurableTableIdentity<IdentityRow> = {
      id: "identity",
      label: "Player",
      width: 280,
      renderCell: (row) => <span>{row ? `Identity ${row.name}` : "…"}</span>,
      onResize: vi.fn(),
    };
    const { onRenderHeader } = renderIdentityShell({
      identity,
      headerFactory: staffHeaderFactory,
    });

    const table = await screen.findByRole("table", {
      name: "Staff identity rows",
    });

    expect(onRenderHeader).toHaveBeenCalled();
    const args = onRenderHeader.mock.calls[0][0];
    expect(args.identity).toBe(identity);
    expect(args.columns).toEqual(columns);
    expect(args.fixedColumns).toEqual(FIXED_COLUMNS);

    const headers = within(table).getAllByRole("columnheader");
    expect(headers[0]).toHaveTextContent("Player");
    expect(headers[0]).toHaveAttribute("scope", "col");
    expect(headers[0].getAttribute("rowspan")).toBe("2");
    expect(headers[0].className).toContain("sticky");
    expect(headers[0].className).toContain("left-0");
    // Grouped + leaf context stays visible beside the identity corner.
    expect(
      within(table).getByRole("columnheader", { name: "Role Fit" }),
    ).toBeInTheDocument();

    const row = (await within(table).findByText("Identity Coach One")).closest(
      "tr",
    ) as HTMLElement;
    const cells = Array.from(row.querySelectorAll("td"));
    expect(cells).toHaveLength(3);
    expect(cells[0]).toHaveTextContent("Identity Coach One");
    expect(cells[0].className).toContain("sticky");
    expect(cells[0].className).toContain("left-0");
    expect(cells[1]).toHaveTextContent("Coach One");
    expect(cells[2]).toHaveTextContent("Boost CA");
  });

  it("keeps the sticky thead layered above scrolled identity cells", async () => {
    renderIdentityShell({
      identity: {
        id: "identity",
        label: "Player",
        width: 280,
        renderCell: (row) => <span>{row ? `Identity ${row.name}` : "…"}</span>,
        onResize: vi.fn(),
      },
      headerFactory: staffHeaderFactory,
    });

    const table = await screen.findByRole("table", {
      name: "Staff identity rows",
    });
    const thead = table.querySelector("thead") as HTMLElement;
    expect(thead.className).toContain("z-10");
    const identityHeader = within(table).getByRole("columnheader", {
      name: "Player",
    });
    // The corner cell floats above both the scrolled columns and the
    // sticky body cells.
    expect(identityHeader.className).toContain("z-20");
    const row = (await within(table).findByText("Identity Coach One")).closest(
      "tr",
    ) as HTMLElement;
    expect(row.querySelector("td")?.className).toContain("z-[1]");
  });

  it("renders an open analysis-column menu above the sticky identity cells", async () => {
    renderIdentityShell({
      identity: {
        id: "identity",
        label: "Player",
        width: 280,
        renderCell: (row) => <span>{row ? `Identity ${row.name}` : "…"}</span>,
        onResize: vi.fn(),
      },
      headerFactory: staffHeaderFactory,
    });

    const table = await screen.findByRole("table", {
      name: "Staff identity rows",
    });
    fireEvent.contextMenu(
      within(table).getByRole("columnheader", { name: "Scout" }),
    );
    const menu = await within(table).findByRole("menu", {
      name: "Scout column actions",
    });
    expect(menu).toBeVisible();
    // A stacking regression that drops the menu to or below the sticky
    // identity corner (z-20) or body cells (z-[1]) would cover its actions.
    const identityHeader = within(table).getByRole("columnheader", {
      name: "Player",
    });
    const row = (await within(table).findByText("Identity Coach One")).closest(
      "tr",
    ) as HTMLElement;
    expect(zLayer(menu.className)).toBeGreaterThan(
      zLayer(identityHeader.className),
    );
    expect(zLayer(menu.className)).toBeGreaterThan(
      zLayer((row.querySelector("td") as HTMLElement).className),
    );
  });

  it("accounts the default 280 identity width in the table minimum", async () => {
    renderIdentityShell({
      identity: {
        id: "identity",
        label: "Player",
        renderCell: () => <span>Identity</span>,
        onResize: vi.fn(),
      },
      headerFactory: staffHeaderFactory,
    });

    // 280 identity + 96 analysis + 128 fixed actions.
    const table = await screen.findByRole("table", {
      name: "Staff identity rows",
    });
    expect(table).toHaveStyle({ minWidth: "504px" });
  });

  it("renders no identity region when the object is omitted", async () => {
    const { onRenderHeader } = renderIdentityShell({
      identity: undefined,
      headerFactory: staffHeaderFactory,
    });

    const table = await screen.findByRole("table", {
      name: "Staff identity rows",
    });
    expect(onRenderHeader.mock.calls[0][0].identity).toBeUndefined();
    expect(
      within(table).queryByRole("columnheader", { name: "Player" }),
    ).toBeNull();
    const row = (await within(table).findByText("Coach One")).closest(
      "tr",
    ) as HTMLElement;
    expect(Array.from(row.querySelectorAll("td"))).toHaveLength(2);
  });
});
