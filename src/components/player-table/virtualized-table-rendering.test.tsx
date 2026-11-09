import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  type ConfigurableTableColumn,
  PlayerTableHeader,
} from "./player-table-header";
import { ConfigurableVirtualizedTable } from "./virtualized-player-table";

type Row = { uid: number; name: string };
type Page = { rows: Row[]; total: number };
type Request = { offset: number; tableId: string | undefined };

const TOTAL = 200;
const PAGE_SIZE = 50;
const ROW_HEIGHT = 40;
const COLUMNS: ConfigurableTableColumn[] = [
  { id: "ca", label: "CA", align: "right", width: 96 },
];
const METRICS = [
  {
    id: "ca",
    label: "CA",
    category: "ability-reputation",
    align: "right" as const,
    defaultWidth: 96,
    sortable: true,
  },
];
const noop = () => {};

function pageAt(offset: number, limit: number): Page {
  return {
    rows: Array.from(
      { length: Math.max(0, Math.min(limit, TOTAL - offset)) },
      (_, index) => ({ uid: offset + index, name: `Row ${offset + index}` }),
    ),
    total: TOTAL,
  };
}

// Stands in for a panel: everything the table receives keeps its identity
// across the table's own re-renders, as the panels' memoized inputs do.
function renderTable({ namedFirstPage = false } = {}) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 60_000 } },
  });
  const requests: Request[] = [];
  const pageQueryOptions = (offset: number, limit: number) => ({
    queryKey: ["rows", offset, limit],
    queryFn: async () => {
      requests.push({ offset, tableId: undefined });
      return pageAt(offset, limit);
    },
  });
  // A panel's requested first page names its table; window options never do.
  const firstPageQueryOptions = namedFirstPage
    ? {
        queryKey: ["rows", "named"],
        queryFn: async () => {
          requests.push({ offset: 0, tableId: "search" });
          return pageAt(0, PAGE_SIZE);
        },
      }
    : undefined;
  const renderCells = vi.fn((row: Row | undefined) => (
    <td>{row?.name ?? "…"}</td>
  ));
  const groupForColumn = vi.fn(() => "ability");
  const groups = {
    groups: [{ id: "ability", label: "Ability" }],
    groupForColumn,
  };
  const onRowActivate = vi.fn();

  render(
    <QueryClientProvider client={queryClient}>
      <ConfigurableVirtualizedTable<Page, Row, unknown[]>
        caption="Rendered rows"
        columnCount={COLUMNS.length}
        columns={COLUMNS}
        getPageRows={(page) => page.rows}
        renderHeader={({ identity, columns, fixedColumns }) => (
          <PlayerTableHeader
            columns={columns}
            fixedColumns={fixedColumns}
            groups={groups}
            identity={identity}
            metrics={METRICS}
            sortBy="ca"
            sortDir="desc"
            onSortChange={noop}
            onAddColumn={noop}
            onRemoveColumn={noop}
            onMoveColumn={noop}
            onResizeColumn={noop}
          />
        )}
        firstPageQueryOptions={firstPageQueryOptions}
        pageQueryOptions={pageQueryOptions}
        pageSize={PAGE_SIZE}
        renderCells={renderCells}
        testId="rendered-rows-scroller"
        total={TOTAL}
        onRowActivate={onRowActivate}
      />
    </QueryClientProvider>,
  );

  return { requests, renderCells, groupForColumn };
}

function renderedIndexes() {
  return Array.from(
    screen
      .getByTestId("rendered-rows-scroller")
      .querySelectorAll("tr[data-index]"),
  ).map((row) => Number(row.getAttribute("data-index")));
}

function scrollToRow(index: number) {
  const scroller = screen.getByTestId("rendered-rows-scroller");
  scroller.scrollTop = index * ROW_HEIGHT;
  fireEvent.scroll(scroller);
}

describe("virtualized table rendering", () => {
  it("renders only rows entering the view and never re-renders the header on scroll", async () => {
    const { renderCells, groupForColumn } = renderTable();
    await screen.findByText("Row 0");

    for (const target of [3, 6]) {
      const before = new Set(renderedIndexes());
      renderCells.mockClear();
      groupForColumn.mockClear();

      scrollToRow(target);

      const entering = renderedIndexes().filter((index) => !before.has(index));
      expect(entering.length).toBeGreaterThan(0);
      const renderedRows = renderCells.mock.calls.map(([row]) => row?.uid);
      expect(renderedRows.sort()).toEqual(entering.sort());
      expect(groupForColumn).not.toHaveBeenCalled();
    }
  });

  it("prefetches the neighbouring windows without naming the table", async () => {
    const { requests } = renderTable({ namedFirstPage: true });
    await screen.findByText("Row 0");

    // The next window loads before any of its rows is visible.
    await waitFor(() =>
      expect(requests).toContainEqual({ offset: 50, tableId: undefined }),
    );
    expect(renderedIndexes().every((index) => index < 50)).toBe(true);

    scrollToRow(60);
    await screen.findByText("Row 60");

    // Both neighbours of the visible window, and nothing further. Only the
    // requested first page names the table; every window and prefetch is
    // anonymous, so none can interrupt or be interrupted.
    await waitFor(() =>
      expect(requests).toContainEqual({ offset: 100, tableId: undefined }),
    );
    expect(requests).toContainEqual({ offset: 0, tableId: undefined });
    expect(new Set(requests.map((request) => request.offset))).toEqual(
      new Set([0, 50, 100]),
    );
    expect(requests.filter((request) => request.tableId)).toEqual([
      { offset: 0, tableId: "search" },
    ]);
  });
});
