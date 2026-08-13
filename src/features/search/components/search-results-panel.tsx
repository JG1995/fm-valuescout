import { useSuspenseQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { SearchX } from "lucide-react";
import { useMemo } from "react";
import { NationalityCell } from "@/components/player-table/nationality-cell";
import {
  type PlayerTableColumn,
  PlayerTableHeader,
} from "@/components/player-table/player-table-header";
import { VirtualizedPlayerTable } from "@/components/player-table/virtualized-player-table";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import {
  formatCount,
  formatMissable,
  formatMoney,
  formatPlayerDob,
} from "@/utils/format";
import { getPlayerMetric } from "@/utils/player-metrics";
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
import { completeFilterRules } from "../utils/filter-registry";

const TEXT_CELL =
  "h-table-row-height-two-line max-w-0 truncate px-2 align-middle text-body-sm";
const NUM_CELL =
  "h-table-row-height-two-line whitespace-nowrap px-2 align-middle text-right font-mono text-mono-sm text-on-surface tabular-nums";

type TableColumn = PlayerTableColumn;

type SearchResultsPanelProps = {
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  onSortChange: (sortBy: SearchSortField, sortDir: SearchSortDir) => void;
};

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

function tableColumnForMetric(
  metricId: string,
  width: number | undefined,
): TableColumn | undefined {
  const metric = getPlayerMetric(metricId);
  if (!metric) {
    return undefined;
  }
  return {
    id: metric.id,
    label: metric.id === "age" ? "Age / DOB" : metric.label,
    align: metric.align,
    width: width ?? metric.defaultWidth,
  };
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
  onAddColumn,
  onRemoveColumn,
  onMoveColumn,
  onResizeColumn,
}: {
  total: number;
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  columns: TableColumn[];
  requestedFields: string[];
  onSortChange: (sortBy: SearchSortField, sortDir: SearchSortDir) => void;
  onAddColumn: (metricId: string) => void;
  onRemoveColumn: (metricId: string) => void;
  onMoveColumn: (metricId: string, targetIndex: number) => void;
  onResizeColumn: (metricId: string, width: number) => void;
}) {
  const navigate = useNavigate();

  return (
    <VirtualizedPlayerTable
      caption="Player search results"
      columnCount={columns.length}
      columns={columns}
      header={
        <PlayerTableHeader
          columns={columns}
          sortBy={sortBy}
          sortDir={sortDir}
          onSortChange={(metricId) => {
            const next = nextSort(sortBy, sortDir, metricId);
            onSortChange(next.sortBy, next.sortDir);
          }}
          onAddColumn={onAddColumn}
          onRemoveColumn={onRemoveColumn}
          onMoveColumn={onMoveColumn}
          onResizeColumn={onResizeColumn}
        />
      }
      pageQueryOptions={(offset, limit) =>
        searchPlayersQueryOptions(
          offset,
          limit,
          sortBy,
          sortDir,
          filters,
          filterCombine,
          requestedFields,
        )
      }
      pageSize={SEARCH_PAGE_SIZE}
      renderCells={(player) =>
        columns.map((column) => {
          if (!isBasicSearchSortField(column.id)) {
            const text = formatDynamicCell(player, column.id);
            return (
              <td
                key={column.id}
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
          if (column.id === "nationality" && player) {
            return (
              <td
                key={column.id}
                className="h-table-row-height-two-line px-2 align-middle text-on-surface"
              >
                <NationalityCell nationalities={player.nationalities} />
              </td>
            );
          }
          const cell = basicCell(
            player,
            column.id as (typeof BASIC_SEARCH_SORT_FIELDS)[number],
          );
          return (
            <td
              key={column.id}
              className={
                cell.numeric
                  ? NUM_CELL
                  : `${TEXT_CELL} ${column.id === "age" || column.id === "division" ? "text-on-surface-variant" : "text-on-surface"}`
              }
              title={cell.title}
            >
              {cell.text}
            </td>
          );
        })
      }
      testId="search-results-scroller"
      total={total}
      onPlayerActivate={(player) => {
        void navigate({
          to: "/players/$uid",
          params: { uid: String(player.uid) },
          search: { tab: "technical" },
        });
      }}
    />
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
  const layout = usePlayerTableStore((state) => state.layouts.search);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const removeStoredColumn = usePlayerTableStore((state) => state.removeColumn);
  const moveColumn = usePlayerTableStore((state) => state.moveColumn);
  const setColumnWidth = usePlayerTableStore((state) => state.setColumnWidth);
  const columns = useMemo<TableColumn[]>(
    () =>
      layout.columnIds.flatMap((metricId) => {
        const column = tableColumnForMetric(metricId, layout.widths[metricId]);
        return column ? [column] : [];
      }),
    [layout],
  );
  const requestedFields = useMemo(
    () =>
      columns
        .filter((column) => !isBasicSearchSortField(column.id))
        .map((column) => column.id)
        .sort(),
    [columns],
  );

  const { data: page } = useSuspenseQuery(
    searchPlayersQueryOptions(
      0,
      SEARCH_PAGE_SIZE,
      sortBy,
      sortDir,
      filters,
      filterCombine,
      requestedFields,
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
  const sortMetric = getPlayerMetric(sortBy);
  const sortLabel = sortMetric
    ? sortMetric.id === "age"
      ? "Age / DOB"
      : sortMetric.label
    : sortBy;
  const removeColumn = (metricId: string) => {
    const remainingColumns = columns.filter((column) => column.id !== metricId);
    if (remainingColumns.length === columns.length) {
      return;
    }
    removeStoredColumn("search", metricId);
    if (sortBy !== metricId) {
      return;
    }
    const nextColumn =
      remainingColumns.find((column) => column.id === "ca") ??
      remainingColumns[0];
    if (!nextColumn) {
      return;
    }
    onSortChange(nextColumn.id, defaultDirForSortField(nextColumn.id));
  };

  return (
    <Panel
      title="Results"
      flush
      className="flex min-h-0 flex-1 flex-col"
      contentClassName="flex min-h-0 flex-1 flex-col"
    >
      <p className="shrink-0 px-4 pb-3 text-body-md text-on-surface-variant">
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
        requestedFields={requestedFields}
        onSortChange={onSortChange}
        onAddColumn={(metricId) => addColumns("search", [metricId])}
        onRemoveColumn={removeColumn}
        onMoveColumn={(metricId, targetIndex) =>
          moveColumn("search", metricId, targetIndex)
        }
        onResizeColumn={(metricId, width) =>
          setColumnWidth("search", metricId, width)
        }
      />
    </Panel>
  );
}
