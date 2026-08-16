import {
  type QueryKey,
  type UseQueryOptions,
  useQueries,
} from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { ReactNode } from "react";
import { useEffect, useRef, useState } from "react";
import type { ConfigurableTableFixedColumn } from "./player-table-header";

/** Must match `--spacing-table-row-height-two-line` / `h-table-row-height-two-line`. */
const ROW_HEIGHT = 40;
/** Must match `--spacing-table-header-height` / sticky `<thead>` height. */
const HEADER_HEIGHT = 32;

type TablePage = {
  total: number;
};

export type ConfigurableVirtualizedTableProps<
  TPage extends TablePage,
  TRow,
  TQueryKey extends QueryKey,
> = {
  caption: string;
  columnCount: number;
  columns: ReadonlyArray<{ id: string; width: number }>;
  fixedColumns?: readonly ConfigurableTableFixedColumn[];
  getPageRows: (page: TPage) => readonly TRow[];
  header: ReactNode;
  pageQueryOptions: (
    offset: number,
    limit: number,
  ) => UseQueryOptions<TPage, Error, TPage, TQueryKey>;
  pageSize: number;
  renderCells: (row: TRow | undefined) => ReactNode;
  testId: string;
  total: number;
  getRowKey?: (row: TRow, index: number) => string | number;
  renderFixedCells?: (row: TRow | undefined) => ReactNode;
  onRowActivate?: (row: TRow) => void;
};

type PlayerPage<TPlayer> = {
  players: TPlayer[];
  total: number;
};

type PlayerOf<TPage extends PlayerPage<unknown>> = TPage["players"][number];

type VirtualizedPlayerTableProps<
  TPage extends PlayerPage<unknown>,
  TQueryKey extends QueryKey,
> = Omit<
  ConfigurableVirtualizedTableProps<TPage, PlayerOf<TPage>, TQueryKey>,
  "getPageRows" | "getRowKey" | "onRowActivate" | "renderCells"
> & {
  renderCells: (player: PlayerOf<TPage> | undefined) => ReactNode;
  onPlayerActivate: (player: PlayerOf<TPage>) => void;
};

function pageIndexesForRange(
  startIndex: number,
  endIndex: number,
  pageSize: number,
): number[] {
  const startPage = Math.floor(startIndex / pageSize);
  const endPage = Math.floor(endIndex / pageSize);
  const pages: number[] = [];
  for (let page = startPage; page <= endPage; page += 1) {
    pages.push(page);
  }
  return pages;
}

function rowAtIndex<TRow>(
  pages: Array<{ page: number; rows: readonly TRow[] | undefined }>,
  pageSize: number,
  index: number,
): TRow | undefined {
  const page = Math.floor(index / pageSize);
  const entry = pages.find((item) => item.page === page);
  return entry?.rows?.[index % pageSize];
}

function isRowActionTarget(target: EventTarget | null) {
  return (
    target instanceof Element &&
    target.closest("button, a, input, select, textarea, [role=dialog]") !== null
  );
}

export function ConfigurableVirtualizedTable<
  TPage extends TablePage,
  TRow,
  TQueryKey extends QueryKey,
>({
  caption,
  columnCount,
  columns,
  fixedColumns = [],
  getPageRows,
  header,
  onRowActivate,
  pageQueryOptions,
  pageSize,
  renderCells,
  testId,
  total,
  getRowKey,
  renderFixedCells,
}: ConfigurableVirtualizedTableProps<TPage, TRow, TQueryKey>) {
  const parentRef = useRef<HTMLDivElement>(null);
  const [keyboardFocusIndex, setKeyboardFocusIndex] = useState(0);
  const [pendingFocusIndex, setPendingFocusIndex] = useState<number | null>(
    null,
  );
  const virtualizer = useVirtualizer({
    count: total,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
    scrollPaddingStart: HEADER_HEIGHT,
    // jsdom reports 0×0; fall back so tests and first paint still open a window.
    initialRect: { width: 1200, height: 600 },
    observeElementRect: (instance, cb) => {
      const measure = () => {
        const element = instance.scrollElement;
        if (!element) {
          cb({ width: 1200, height: 600 });
          return;
        }
        const height = element.clientHeight;
        const width = element.clientWidth;
        cb({
          width: width > 0 ? width : 1200,
          height: height > 0 ? height : 600,
        });
      };
      measure();
      const element = instance.scrollElement;
      if (!element || typeof ResizeObserver === "undefined") {
        return () => {};
      }
      const observer = new ResizeObserver(measure);
      observer.observe(element);
      return () => {
        observer.disconnect();
      };
    },
  });

  useEffect(() => {
    setKeyboardFocusIndex((index) => Math.min(index, Math.max(0, total - 1)));
    setPendingFocusIndex((index) => {
      if (index === null || total === 0) {
        return null;
      }
      return Math.min(index, total - 1);
    });
    const element = parentRef.current;
    if (!element) {
      return;
    }
    const maximumOffset = Math.max(
      0,
      element.scrollHeight - element.clientHeight,
    );
    if (element.scrollTop > maximumOffset) {
      virtualizer.scrollToOffset(maximumOffset);
    }
  }, [total, virtualizer]);

  const focusRow = (index: number) => {
    if (index < 0 || index >= total) {
      return;
    }
    if (!parentRef.current?.querySelector(`[data-index="${index}"]`)) {
      virtualizer.scrollToIndex(index, { align: "auto" });
    }
    setPendingFocusIndex(index);
  };

  const virtualRows = virtualizer.getVirtualItems();
  const rangeStart = virtualRows[0]?.index ?? 0;
  const rangeEnd = virtualRows[virtualRows.length - 1]?.index ?? 0;
  const pages = Array.from(
    new Set([
      ...pageIndexesForRange(rangeStart, rangeEnd, pageSize),
      ...(pendingFocusIndex === null
        ? []
        : [Math.floor(pendingFocusIndex / pageSize)]),
    ]),
  ).sort((left, right) => left - right);
  const pageQueries = useQueries({
    queries: pages.map((page) => pageQueryOptions(page * pageSize, pageSize)),
  });
  const pageData = pages.map((page, index) => ({
    page,
    rows: pageQueries[index]?.data
      ? getPageRows(pageQueries[index].data)
      : undefined,
  }));

  useEffect(() => {
    if (pendingFocusIndex === null) {
      return;
    }
    if (!rowAtIndex(pageData, pageSize, pendingFocusIndex)) {
      return;
    }
    const row = parentRef.current?.querySelector<HTMLElement>(
      `[data-index="${pendingFocusIndex}"]`,
    );
    if (!row) {
      virtualizer.scrollToIndex(pendingFocusIndex, { align: "auto" });
      return;
    }
    setKeyboardFocusIndex(pendingFocusIndex);
    setPendingFocusIndex(null);
    row.focus();
  }, [pageData, pageSize, pendingFocusIndex, virtualizer]);

  const visibleLoadedIndexes = virtualRows
    .map((row) => row.index)
    .filter((index) => rowAtIndex(pageData, pageSize, index) !== undefined);
  const tabStopIndex = visibleLoadedIndexes.includes(keyboardFocusIndex)
    ? keyboardFocusIndex
    : (visibleLoadedIndexes[0] ?? 0);
  const paddingTop = virtualRows.length > 0 ? virtualRows[0].start : 0;
  const paddingBottom =
    virtualRows.length > 0
      ? virtualizer.getTotalSize() - virtualRows[virtualRows.length - 1].end
      : 0;
  const failedPageQuery = pageQueries.find((query) => query.isError);
  const allColumns = [...columns, ...fixedColumns];
  const minimumTableWidth = allColumns.reduce(
    (sum, column) => sum + column.width,
    0,
  );

  return (
    <div className="relative min-h-0 flex-1">
      <div
        ref={parentRef}
        data-testid={testId}
        className="h-full min-h-0 overflow-auto"
      >
        <table
          className="table-fixed border-collapse text-left"
          style={{
            width: "100%",
            minWidth: minimumTableWidth,
          }}
        >
          <caption className="sr-only">{caption}</caption>
          <colgroup>
            {allColumns.map((column) => (
              <col
                key={column.id}
                style={{
                  width: `${(column.width / minimumTableWidth) * 100}%`,
                }}
              />
            ))}
          </colgroup>
          {header}
          <tbody>
            {paddingTop > 0 ? (
              <tr>
                <td
                  colSpan={columnCount + fixedColumns.length}
                  style={{ height: paddingTop }}
                />
              </tr>
            ) : null}
            {virtualRows.map((virtualRow) => {
              const row = rowAtIndex(pageData, pageSize, virtualRow.index);
              const isInteractive = row !== undefined && onRowActivate;
              const isTabStop = virtualRow.index === tabStopIndex;

              return (
                <tr
                  key={
                    row && getRowKey
                      ? getRowKey(row, virtualRow.index)
                      : virtualRow.key
                  }
                  data-index={virtualRow.index}
                  tabIndex={isInteractive ? (isTabStop ? 0 : -1) : undefined}
                  className={
                    isInteractive
                      ? "cursor-pointer border-t border-outline-variant transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary"
                      : "border-t border-outline-variant transition-colors duration-150 ease-out hover:bg-surface-container-high"
                  }
                  style={{ height: `${virtualRow.size}px` }}
                  onFocus={
                    isInteractive
                      ? () => {
                          setPendingFocusIndex(null);
                          setKeyboardFocusIndex(virtualRow.index);
                        }
                      : undefined
                  }
                  onBlur={
                    isInteractive
                      ? () => {
                          setPendingFocusIndex(null);
                        }
                      : undefined
                  }
                  onClick={
                    isInteractive && row
                      ? (event) => {
                          if (isRowActionTarget(event.target)) return;
                          onRowActivate(row);
                        }
                      : undefined
                  }
                  onKeyDown={
                    isInteractive
                      ? (event) => {
                          if (isRowActionTarget(event.target)) return;
                          if (event.key === "ArrowDown") {
                            event.preventDefault();
                            focusRow(virtualRow.index + 1);
                            return;
                          }
                          if (event.key === "ArrowUp") {
                            event.preventDefault();
                            focusRow(virtualRow.index - 1);
                            return;
                          }
                          if (event.key === "Enter" && row) {
                            event.preventDefault();
                            onRowActivate(row);
                          }
                        }
                      : undefined
                  }
                >
                  {renderCells(row)}
                  {renderFixedCells?.(row)}
                </tr>
              );
            })}
            {paddingBottom > 0 ? (
              <tr>
                <td
                  colSpan={columnCount + fixedColumns.length}
                  style={{ height: paddingBottom }}
                />
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
      {failedPageQuery ? (
        <div
          role="alert"
          className="absolute inset-x-2 bottom-2 flex items-center justify-between gap-3 rounded-md border border-error bg-surface-container-high px-3 py-2 text-body-sm text-on-surface shadow-md"
        >
          <span>Couldn't load this part of the table.</span>
          <button
            type="button"
            className="shrink-0 rounded-full border border-outline px-3 py-1 text-label-md text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-highest focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
            onClick={() => {
              void failedPageQuery.refetch();
            }}
          >
            Retry
          </button>
        </div>
      ) : null}
    </div>
  );
}

/** Compatibility wrapper for existing player Search and Squad callers. */
export function VirtualizedPlayerTable<
  TPage extends PlayerPage<unknown>,
  TQueryKey extends QueryKey,
>(props: VirtualizedPlayerTableProps<TPage, TQueryKey>) {
  return (
    <ConfigurableVirtualizedTable<TPage, PlayerOf<TPage>, TQueryKey>
      {...props}
      getPageRows={(page) => page.players}
      onRowActivate={props.onPlayerActivate}
    />
  );
}
