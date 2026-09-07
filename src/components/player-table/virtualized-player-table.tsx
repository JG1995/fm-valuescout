import {
  type QueryKey,
  type UseQueryOptions,
  useQueries,
} from "@tanstack/react-query";
import { useVirtualizer } from "@tanstack/react-virtual";
import type { ReactNode } from "react";
import { useEffect, useRef, useState } from "react";
import {
  type ConfigurableTableColumn,
  type ConfigurableTableFixedColumn,
  type ConfigurableTableIdentityHeader,
  clampIdentityWidth,
} from "./player-table-header";

/** Must match `--spacing-table-row-height-two-line` / `h-table-row-height-two-line`. */
const ROW_HEIGHT = 40;
/**
 * Must match two `--spacing-table-header-height` rows: the grouped group
 * row plus the leaf row. The virtualizer `scrollPaddingStart` below keeps
 * keyboard-focused rows fully under the two-row sticky `<thead>`.
 */
export const HEADER_HEIGHT = 64;

type TablePage = {
  total: number;
};

/**
 * Optional storage-agnostic identity region. The caller supplies one object;
 * the shell owns the identity `<col>`, body-cell order and rendering
 * (`renderCell` first, identity → analysis → fixed actions), spacer
 * `colSpan`, and minimum-width accounting, and applies the supplied width
 * without persisting it. This file imports no store and knows no table IDs.
 */
export type ConfigurableTableIdentity<TRow> =
  ConfigurableTableIdentityHeader & {
    renderCell: (row: TRow | undefined) => ReactNode;
  };

export type ConfigurableTableRenderHeader<TRow> = (args: {
  identity: ConfigurableTableIdentity<TRow> | undefined;
  columns: readonly ConfigurableTableColumn[];
  fixedColumns: readonly ConfigurableTableFixedColumn[];
}) => ReactNode;

export type ConfigurableVirtualizedTableProps<
  TPage extends TablePage,
  TRow,
  TQueryKey extends QueryKey,
> = {
  caption: string;
  columnCount: number;
  columns: readonly ConfigurableTableColumn[];
  fixedColumns?: readonly ConfigurableTableFixedColumn[];
  getPageRows: (page: TPage) => readonly TRow[];
  identity?: ConfigurableTableIdentity<TRow>;
  renderHeader: ConfigurableTableRenderHeader<TRow>;
  firstPageQueryOptions?: UseQueryOptions<TPage, Error, TPage, TQueryKey>;
  isReplacementActive?: boolean;
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
  identity,
  renderHeader,
  firstPageQueryOptions,
  isReplacementActive = false,
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
    if (isReplacementActive) {
      setPendingFocusIndex(null);
      return;
    }
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
  }, [isReplacementActive, total, virtualizer]);

  const focusRow = (index: number) => {
    if (isReplacementActive || index < 0 || index >= total) {
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
    queries: pages.map((page) =>
      page === 0 && firstPageQueryOptions
        ? firstPageQueryOptions
        : pageQueryOptions(page * pageSize, pageSize),
    ),
  });
  const pageData = pages.map((page, index) => ({
    page,
    rows: pageQueries[index]?.data
      ? getPageRows(pageQueries[index].data)
      : undefined,
  }));

  useEffect(() => {
    if (isReplacementActive || pendingFocusIndex === null) {
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
  }, [isReplacementActive, pageData, pageSize, pendingFocusIndex, virtualizer]);

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
  // Strict region order identity → analysis → fixed actions. The identity
  // width is applied as supplied (defaulting to 280) without persistence.
  const identityWidth = identity ? clampIdentityWidth(identity.width) : 0;
  const allColumns = [
    ...(identity ? [{ id: identity.id, width: identityWidth }] : []),
    ...columns,
    ...fixedColumns,
  ];
  const minimumTableWidth = allColumns.reduce(
    (sum, column) => sum + column.width,
    0,
  );
  const thead = renderHeader({ identity, columns, fixedColumns });

  return (
    <div className="relative min-h-0 flex-1">
      <div
        ref={parentRef}
        data-testid={testId}
        className="h-full max-w-full w-fit min-h-0 overflow-auto rounded-lg border border-outline-variant"
      >
        <table
          className="table-fixed border-collapse text-left"
          style={{
            // Bounded containment model: the table is fixed to exactly the
            // column-width sum with fixed pixel columns. Readable minimums
            // hold, horizontal overflow stays in the same scroller, and
            // ultrawide viewports reveal more columns instead of stretching
            // cells without bound.
            width: minimumTableWidth,
            minWidth: minimumTableWidth,
            maxWidth: minimumTableWidth,
          }}
        >
          <caption className="sr-only">{caption}</caption>
          <colgroup>
            {allColumns.map((column) => (
              <col key={column.id} style={{ width: column.width }} />
            ))}
          </colgroup>
          {thead}
          <tbody>
            {paddingTop > 0 ? (
              <tr>
                <td
                  colSpan={
                    columnCount + fixedColumns.length + (identity ? 1 : 0)
                  }
                  style={{ height: paddingTop }}
                />
              </tr>
            ) : null}
            {virtualRows.map((virtualRow) => {
              const row = rowAtIndex(pageData, pageSize, virtualRow.index);
              const isInteractive =
                !isReplacementActive && row !== undefined && onRowActivate;
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
                  {identity ? (
                    <td
                      key={identity.id}
                      className="sticky left-0 z-[1] bg-surface-container"
                    >
                      {identity.renderCell(row)}
                    </td>
                  ) : null}
                  {renderCells(row)}
                  {renderFixedCells?.(row)}
                </tr>
              );
            })}
            {paddingBottom > 0 ? (
              <tr>
                <td
                  colSpan={
                    columnCount + fixedColumns.length + (identity ? 1 : 0)
                  }
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
