import { useQueries, useSuspenseQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { useVirtualizer } from "@tanstack/react-virtual";
import { ChevronDown, ChevronUp, SearchX } from "lucide-react";
import { useMemo, useRef, useState } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import {
  formatCount,
  formatMissable,
  formatMoney,
  formatPlayerDob,
} from "@/utils/format";
import {
  SEARCH_PAGE_SIZE,
  searchPlayersQueryOptions,
} from "../api/search-players-query-options";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import { filterValueToIpc } from "../types/filter-rule";
import type { PlayerSummary } from "../types/player-summary";
import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  type BASIC_SEARCH_SORT_FIELDS,
  defaultDirForSortField,
  isBasicSearchSortField,
} from "../types/search-sort";
import {
  dynamicColumnFields,
  dynamicColumnLabel,
} from "../utils/dynamic-columns";
import { completeFilterRules, getFilterField } from "../utils/filter-registry";

/** Must match `--spacing-table-row-height-two-line` / `h-table-row-height-two-line`. */
const ROW_HEIGHT = 40;
/** Must match `--spacing-table-header-height` / sticky `<thead>` height. */
const HEADER_HEIGHT = 32;
const TEXT_CELL =
  "h-table-row-height-two-line max-w-0 truncate px-2 align-middle text-body-sm";
const NUM_CELL =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-right font-mono text-mono-sm text-on-surface tabular-nums";

const BASIC_COLUMNS = [
  { key: "name", label: "Name", align: "left" as const },
  { key: "age", label: "Age / DOB", align: "left" as const },
  { key: "nationality", label: "Nationality", align: "left" as const },
  { key: "club", label: "Club", align: "left" as const },
  { key: "division", label: "Division", align: "left" as const },
  { key: "ca", label: "CA", align: "right" as const },
  { key: "pa", label: "PA", align: "right" as const },
  { key: "value", label: "Value", align: "right" as const },
] as const satisfies ReadonlyArray<{
  key: (typeof BASIC_SEARCH_SORT_FIELDS)[number];
  label: string;
  align: "left" | "right";
}>;

type TableColumn = {
  key: SearchSortField;
  label: string;
  align: "left" | "right";
  dynamic?: boolean;
};

const BASIC_SORT_LABELS: Record<
  (typeof BASIC_SEARCH_SORT_FIELDS)[number],
  string
> = {
  name: "Name",
  age: "Age / DOB",
  nationality: "Nationality",
  club: "Club",
  division: "Division",
  ca: "CA",
  pa: "PA",
  value: "Value",
};

type SearchResultsPanelProps = {
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  onSortChange: (sortBy: SearchSortField, sortDir: SearchSortDir) => void;
};

function pageIndexesForRange(startIndex: number, endIndex: number): number[] {
  const startPage = Math.floor(startIndex / SEARCH_PAGE_SIZE);
  const endPage = Math.floor(endIndex / SEARCH_PAGE_SIZE);
  const pages: number[] = [];
  for (let page = startPage; page <= endPage; page += 1) {
    pages.push(page);
  }
  return pages;
}

function playerAtIndex(
  pages: Array<{ page: number; players: PlayerSummary[] | undefined }>,
  index: number,
): PlayerSummary | undefined {
  const page = Math.floor(index / SEARCH_PAGE_SIZE);
  const entry = pages.find((item) => item.page === page);
  return entry?.players?.[index % SEARCH_PAGE_SIZE];
}

function nextSort(
  currentBy: SearchSortField,
  currentDir: SearchSortDir,
  clicked: SearchSortField,
): { sortBy: SearchSortField; sortDir: SearchSortDir } {
  if (clicked === currentBy) {
    return {
      sortBy: currentBy,
      sortDir: currentDir === "asc" ? "desc" : "asc",
    };
  }
  return { sortBy: clicked, sortDir: defaultDirForSortField(clicked) };
}

function formatDynamicCell(
  player: PlayerSummary | undefined,
  fieldId: string,
): string {
  if (!player) {
    return "…";
  }
  const value = player.dynamicValues?.[fieldId];
  if (value === undefined || value === null) {
    return "—";
  }
  return String(value);
}

function basicCell(
  player: PlayerSummary | undefined,
  key: (typeof BASIC_SEARCH_SORT_FIELDS)[number],
): { text: string; title?: string; numeric: boolean } {
  if (!player) {
    return { text: "…", numeric: key !== "name" && key !== "age" };
  }
  switch (key) {
    case "name":
      return { text: player.name, title: player.name, numeric: false };
    case "age": {
      const dob = formatPlayerDob(
        player.birthYear,
        player.birthDayOfYear,
        player.age,
      );
      return { text: dob, title: dob, numeric: false };
    }
    case "nationality": {
      const nationalities = String(
        formatMissable(player.nationalities.join(", ")),
      );
      return { text: nationalities, title: nationalities, numeric: false };
    }
    case "club": {
      const club = String(formatMissable(player.club));
      return {
        text: club,
        title: club !== "—" ? club : undefined,
        numeric: false,
      };
    }
    case "division": {
      const division = String(formatMissable(player.division));
      return {
        text: division,
        title: division !== "—" ? division : undefined,
        numeric: false,
      };
    }
    case "ca":
      return { text: String(player.ca), numeric: true };
    case "pa":
      return { text: String(player.pa), numeric: true };
    case "value":
      return {
        text:
          player.marketValueGbp === null
            ? "—"
            : formatMoney(player.marketValueGbp),
        numeric: true,
      };
  }
}

function SearchResultsVirtualTable({
  total,
  sortBy,
  sortDir,
  filters,
  filterCombine,
  columns,
  requestedFields,
  onSortChange,
}: {
  total: number;
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  columns: TableColumn[];
  requestedFields: string[];
  onSortChange: (sortBy: SearchSortField, sortDir: SearchSortDir) => void;
}) {
  const navigate = useNavigate();
  const parentRef = useRef<HTMLDivElement>(null);
  const pageDataRef = useRef<
    Array<{ page: number; players: PlayerSummary[] | undefined }>
  >([]);
  const [keyboardFocusIndex, setKeyboardFocusIndex] = useState(0);
  const columnCount = columns.length;
  const clampedFocusIndex =
    total <= 0 ? 0 : Math.min(keyboardFocusIndex, total - 1);
  if (clampedFocusIndex !== keyboardFocusIndex) {
    setKeyboardFocusIndex(clampedFocusIndex);
  }
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
        const el = instance.scrollElement;
        if (!el) {
          cb({ width: 1200, height: 600 });
          return;
        }
        const height = el.clientHeight;
        const width = el.clientWidth;
        cb({
          width: width > 0 ? width : 1200,
          height: height > 0 ? height : 600,
        });
      };
      measure();
      const el = instance.scrollElement;
      if (!el || typeof ResizeObserver === "undefined") {
        return () => {};
      }
      const observer = new ResizeObserver(measure);
      observer.observe(el);
      return () => {
        observer.disconnect();
      };
    },
  });

  const openPlayer = (uid: number) => {
    void navigate({
      to: "/players/$uid",
      params: { uid: String(uid) },
      search: { tab: "technical" },
    });
  };

  const focusRow = (index: number) => {
    if (index < 0 || index >= total) {
      return;
    }
    virtualizer.scrollToIndex(index, { align: "auto" });
    const tryFocus = (attemptsLeft: number) => {
      if (!playerAtIndex(pageDataRef.current, index)) {
        if (attemptsLeft <= 0) {
          return;
        }
        requestAnimationFrame(() => {
          tryFocus(attemptsLeft - 1);
        });
        return;
      }
      setKeyboardFocusIndex(index);
      const row = parentRef.current?.querySelector<HTMLElement>(
        `[data-index="${index}"]`,
      );
      if (row) {
        row.focus();
        return;
      }
      if (attemptsLeft <= 0) {
        return;
      }
      requestAnimationFrame(() => {
        tryFocus(attemptsLeft - 1);
      });
    };
    requestAnimationFrame(() => {
      tryFocus(16);
    });
  };

  const virtualRows = virtualizer.getVirtualItems();
  const rangeStart = virtualRows[0]?.index ?? 0;
  const rangeEnd = virtualRows[virtualRows.length - 1]?.index ?? 0;
  const pages = pageIndexesForRange(rangeStart, rangeEnd);

  const pageQueries = useQueries({
    queries: pages.map((page) =>
      searchPlayersQueryOptions(
        page * SEARCH_PAGE_SIZE,
        SEARCH_PAGE_SIZE,
        sortBy,
        sortDir,
        filters,
        filterCombine,
        requestedFields,
      ),
    ),
  });

  const pageData = pages.map((page, index) => ({
    page,
    players: pageQueries[index]?.data?.players,
  }));
  pageDataRef.current = pageData;

  const visibleLoadedIndexes = virtualRows
    .map((row) => row.index)
    .filter((index) => playerAtIndex(pageData, index) !== undefined);
  const tabStopIndex = visibleLoadedIndexes.includes(keyboardFocusIndex)
    ? keyboardFocusIndex
    : (visibleLoadedIndexes[0] ?? 0);

  const paddingTop = virtualRows.length > 0 ? virtualRows[0].start : 0;
  const paddingBottom =
    virtualRows.length > 0
      ? virtualizer.getTotalSize() - virtualRows[virtualRows.length - 1].end
      : 0;

  return (
    <div
      ref={parentRef}
      data-testid="search-results-scroller"
      className="max-h-[min(70vh,720px)] overflow-auto"
    >
      <table className="w-full border-collapse text-left">
        <caption className="sr-only">Player search results</caption>
        <thead className="sticky top-0 z-10">
          <tr className="bg-surface-container-lowest">
            {columns.map((column) => {
              const active = column.key === sortBy;
              const ariaSort = active
                ? sortDir === "asc"
                  ? "ascending"
                  : "descending"
                : "none";
              const Caret = sortDir === "asc" ? ChevronUp : ChevronDown;
              return (
                <th
                  key={column.key}
                  scope="col"
                  aria-sort={ariaSort}
                  className={
                    column.align === "right"
                      ? "h-table-header-height px-2 text-right"
                      : "h-table-header-height px-2 text-left"
                  }
                >
                  <button
                    type="button"
                    className={
                      active
                        ? "inline-flex items-center gap-1 text-label-md text-primary uppercase"
                        : "inline-flex items-center gap-1 text-label-md text-on-surface-variant uppercase"
                    }
                    onClick={() => {
                      const next = nextSort(sortBy, sortDir, column.key);
                      onSortChange(next.sortBy, next.sortDir);
                    }}
                  >
                    {column.label}
                    {active ? (
                      <Caret
                        aria-hidden
                        className="size-3.5 shrink-0"
                        strokeWidth={2}
                      />
                    ) : null}
                  </button>
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {paddingTop > 0 ? (
            <tr>
              <td colSpan={columnCount} style={{ height: paddingTop }} />
            </tr>
          ) : null}
          {virtualRows.map((virtualRow) => {
            const player = playerAtIndex(pageData, virtualRow.index);
            const isTabStop = virtualRow.index === tabStopIndex;

            return (
              <tr
                key={virtualRow.key}
                data-index={virtualRow.index}
                tabIndex={player ? (isTabStop ? 0 : -1) : undefined}
                className={
                  player
                    ? "cursor-pointer border-t border-outline-variant transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary"
                    : "border-t border-outline-variant transition-colors duration-150 ease-out hover:bg-surface-container-high"
                }
                style={{ height: `${virtualRow.size}px` }}
                onFocus={() => {
                  setKeyboardFocusIndex(virtualRow.index);
                }}
                onClick={() => {
                  if (player) {
                    openPlayer(player.uid);
                  }
                }}
                onKeyDown={(event) => {
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
                  if (event.key === "Enter" && player) {
                    event.preventDefault();
                    openPlayer(player.uid);
                  }
                }}
              >
                {columns.map((column) => {
                  if (column.dynamic) {
                    const text = formatDynamicCell(player, column.key);
                    return (
                      <td
                        key={column.key}
                        className={
                          column.align === "right"
                            ? NUM_CELL
                            : `${TEXT_CELL} text-on-surface`
                        }
                        title={text !== "—" && text !== "…" ? text : undefined}
                      >
                        {text}
                      </td>
                    );
                  }
                  const cell = basicCell(
                    player,
                    column.key as (typeof BASIC_SEARCH_SORT_FIELDS)[number],
                  );
                  return (
                    <td
                      key={column.key}
                      className={
                        cell.numeric
                          ? NUM_CELL
                          : `${TEXT_CELL} ${column.key === "age" || column.key === "division" ? "text-on-surface-variant" : "text-on-surface"}`
                      }
                      title={cell.title}
                    >
                      {cell.text}
                    </td>
                  );
                })}
              </tr>
            );
          })}
          {paddingBottom > 0 ? (
            <tr>
              <td colSpan={columnCount} style={{ height: paddingBottom }} />
            </tr>
          ) : null}
        </tbody>
      </table>
    </div>
  );
}

/** Assumes a current snapshot exists — the route handles the no-snapshot empty. */
export function SearchResultsPanel({
  sortBy,
  sortDir,
  filters,
  filterCombine,
  onSortChange,
}: SearchResultsPanelProps) {
  const dynamicFields = useMemo(() => dynamicColumnFields(filters), [filters]);
  const columns = useMemo<TableColumn[]>(
    () => [
      ...BASIC_COLUMNS.map((column) => ({ ...column })),
      ...dynamicFields.map((fieldId) => {
        const kind = getFilterField(fieldId)?.kind;
        const numeric = kind === "integer" || kind === "boolean";
        return {
          key: fieldId,
          label: dynamicColumnLabel(fieldId),
          align: numeric ? ("right" as const) : ("left" as const),
          dynamic: true,
        };
      }),
    ],
    [dynamicFields],
  );

  const { data: page } = useSuspenseQuery(
    searchPlayersQueryOptions(
      0,
      SEARCH_PAGE_SIZE,
      sortBy,
      sortDir,
      filters,
      filterCombine,
      dynamicFields,
    ),
  );
  const listKey = useMemo(
    () =>
      [
        filterCombine,
        ...filters.map(
          (rule) =>
            `${rule.field}:${rule.op}:${String(filterValueToIpc(rule.value))}`,
        ),
      ].join("|"),
    [filterCombine, filters],
  );

  if (page.total === 0) {
    const appliedFilters = completeFilterRules(filters);
    if (appliedFilters.length > 0) {
      return (
        <Panel title="Results" flush>
          <EmptyState icon={SearchX} title="No players match these filters">
            Adjust or clear filters in the strip above to widen the result set.
          </EmptyState>
        </Panel>
      );
    }

    return (
      <Panel title="Results" flush>
        <EmptyState icon={SearchX} title="No players in snapshot">
          The snapshot exists but holds no player rows. Run Load Data again with
          Football Manager in an active save.
        </EmptyState>
      </Panel>
    );
  }

  const dirLabel = sortDir === "asc" ? "ascending" : "descending";
  const sortLabel = isBasicSearchSortField(sortBy)
    ? BASIC_SORT_LABELS[sortBy]
    : dynamicColumnLabel(sortBy);

  return (
    <Panel title="Results" flush>
      <p className="px-4 pb-3 text-body-md text-on-surface-variant">
        <span className="text-on-surface">{formatCount(page.total)}</span>{" "}
        players · sorted by {sortLabel} ({dirLabel})
      </p>
      <SearchResultsVirtualTable
        key={listKey}
        total={page.total}
        sortBy={sortBy}
        sortDir={sortDir}
        filters={filters}
        filterCombine={filterCombine}
        columns={columns}
        requestedFields={dynamicFields}
        onSortChange={onSortChange}
      />
    </Panel>
  );
}
