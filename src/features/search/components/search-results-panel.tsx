import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { SearchX } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { NationalityCell } from "@/components/player-table/nationality-cell";
import {
  type PlayerTableColumn,
  PlayerTableHeader,
} from "@/components/player-table/player-table-header";
import { VirtualizedPlayerTable } from "@/components/player-table/virtualized-player-table";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import {
  formatCount,
  formatMissable,
  formatMoney,
  formatPlayerDob,
} from "@/utils/format";
import {
  formatMoneyballMetric,
  getMoneyballSearchMetric,
  MONEYBALL_SEARCH_METRICS,
} from "@/utils/moneyball-search-metrics";
import { getPlayerMetric } from "@/utils/player-metrics";
import {
  isFullTacticGroup,
  isTacticColumnId,
  isValidTacticColumnId,
  TACTIC_COLUMN_DEFAULT_WIDTH,
  TACTIC_LANE_IDS,
  tacticGroupForId,
  tacticLaneIdForId,
} from "@/utils/tactic-ids";
import type { SearchPlayerPageContext } from "../api/search-keys";
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
import type { ComparisonPool, SearchView } from "../types/search-view";
import { defaultSearchSort } from "../types/search-view";
import { completeFilterRules } from "../utils/filter-registry";
import { buildTacticColumnOrder } from "../utils/tactic-columns";

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
  view: SearchView;
  comparisonPool: ComparisonPool;
  pageContext: SearchPlayerPageContext;
  orderedLaneIds: readonly string[];
  laneLabels: ReadonlyMap<string, string>;
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
  view: SearchView,
  laneLabels: ReadonlyMap<string, string>,
): TableColumn | undefined {
  if (isValidTacticColumnId(metricId)) {
    const laneId = tacticLaneIdForId(metricId);
    return {
      id: metricId,
      label: laneLabels.get(laneId ?? "") ?? laneId ?? metricId,
      align: "right",
      width: width ?? TACTIC_COLUMN_DEFAULT_WIDTH,
    };
  }
  if (view === "moneyball") {
    const metric = getMoneyballSearchMetric(metricId);
    if (metric) {
      return {
        id: metric.id,
        label: metric.label,
        align: metric.align,
        width: width ?? metric.defaultWidth,
      };
    }
  }
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
  view,
  comparisonPool,
  pageContext,
  firstPageQueryOptions,
  isReplacementActive,
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
  view: SearchView;
  comparisonPool: ComparisonPool;
  pageContext: SearchPlayerPageContext;
  firstPageQueryOptions: ReturnType<typeof searchPlayersQueryOptions>;
  isReplacementActive: boolean;
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
          metrics={view === "moneyball" ? MONEYBALL_SEARCH_METRICS : undefined}
        />
      }
      firstPageQueryOptions={firstPageQueryOptions}
      isReplacementActive={isReplacementActive}
      pageQueryOptions={(offset, limit) =>
        searchPlayersQueryOptions(
          offset,
          limit,
          sortBy,
          sortDir,
          filters,
          filterCombine,
          requestedFields,
          view,
          comparisonPool,
          pageContext,
        )
      }
      pageSize={SEARCH_PAGE_SIZE}
      renderCells={(player) =>
        columns.map((column) => {
          if (isValidTacticColumnId(column.id)) {
            const score = player?.dynamicValues?.[column.id];
            return (
              <td key={column.id} className={NUM_CELL}>
                {typeof score === "number" ? (
                  <ScoreBadge score={score} roleName={column.label} />
                ) : (
                  <span className="text-on-surface-variant">
                    {player === undefined ? "…" : "—"}
                  </span>
                )}
              </td>
            );
          }
          const moneyballMetric =
            view === "moneyball"
              ? getMoneyballSearchMetric(column.id)
              : undefined;
          if (moneyballMetric?.metric || moneyballMetric?.context) {
            const raw = player?.dynamicValues?.[column.id];
            const value = typeof raw === "number" ? raw : null;
            const text = moneyballMetric.metric
              ? formatMoneyballMetric(moneyballMetric.metric, value)
              : value === null
                ? player === undefined
                  ? "…"
                  : "—"
                : formatCount(value);
            const score = player?.moneyballPercentiles?.[column.id];
            return (
              <td
                key={column.id}
                className={NUM_CELL}
                title={text !== "—" && text !== "…" ? text : undefined}
              >
                <span className="inline-flex items-center justify-end gap-2">
                  <span
                    className={
                      value === null ? "text-on-surface-variant" : undefined
                    }
                  >
                    {text}
                  </span>
                  {moneyballMetric.metric && typeof score === "number" ? (
                    <ScoreBadge score={score} roleName={column.label} />
                  ) : null}
                </span>
              </td>
            );
          }
          if (moneyballMetric?.role) {
            const score = player?.dynamicValues?.[column.id];
            return (
              <td key={column.id} className={NUM_CELL}>
                {typeof score === "number" ? (
                  <ScoreBadge
                    score={score}
                    roleName={`Moneyball role · ${column.label}`}
                  />
                ) : (
                  <span className="text-on-surface-variant">
                    {player === undefined ? "…" : "—"}
                  </span>
                )}
              </td>
            );
          }
          if (!isBasicSearchSortField(column.id)) {
            if (
              column.id === "club_dna" ||
              column.id.startsWith("role.") ||
              column.id.startsWith("potential_role.")
            ) {
              const score = player?.dynamicValues?.[column.id];
              return (
                <td key={column.id} className={NUM_CELL}>
                  {typeof score === "number" ? (
                    <ScoreBadge score={score} roleName={column.label} />
                  ) : (
                    <span className="text-on-surface-variant">
                      {player === undefined ? "…" : "—"}
                    </span>
                  )}
                </td>
              );
            }
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
          if (column.id === "name" && player) {
            const identityContext = [player.club, player.division]
              .filter(
                (value): value is string => value !== null && value !== "",
              )
              .join(" · ");
            return (
              <td
                key={column.id}
                className={`${TEXT_CELL} text-on-surface`}
                title={player.name}
              >
                <span className="block truncate">{player.name}</span>
                {identityContext ? (
                  <span className="block truncate text-[11px] leading-4 text-on-surface-variant">
                    {identityContext}
                  </span>
                ) : null}
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
          search: { view: view === "shortlist" ? "general" : view },
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
  view,
  comparisonPool,
  pageContext,
  orderedLaneIds,
  laneLabels,
}: SearchResultsPanelProps) {
  const navigate = useNavigate();
  const tableId =
    view === "moneyball"
      ? "moneyball-search"
      : view === "shortlist"
        ? "shortlist"
        : "search";
  const layout = usePlayerTableStore((state) => state.layouts[tableId]);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const removeStoredColumn = usePlayerTableStore((state) => state.removeColumn);
  const replaceLayout = usePlayerTableStore((state) => state.replaceLayout);
  const moveColumn = usePlayerTableStore((state) => state.moveColumn);
  const setColumnWidth = usePlayerTableStore((state) => state.setColumnWidth);
  const columns = useMemo<TableColumn[]>(
    () =>
      layout.columnIds.flatMap((metricId) => {
        const column = tableColumnForMetric(
          metricId,
          layout.widths[metricId],
          view,
          laneLabels,
        );
        return column ? [column] : [];
      }),
    [laneLabels, layout, view],
  );
  const requestedFields = useMemo(
    () =>
      columns
        .filter((column) => !isBasicSearchSortField(column.id))
        .map((column) => column.id)
        .sort(),
    [columns],
  );

  const requested = useMemo(
    () => ({
      sortBy,
      sortDir,
      filters,
      filterCombine,
      requestedFields,
      view,
      comparisonPool,
      pageContext,
    }),
    [
      comparisonPool,
      filterCombine,
      filters,
      pageContext,
      requestedFields,
      sortBy,
      sortDir,
      view,
    ],
  );
  const [committed, setCommitted] = useState(requested);
  const committedOptions = searchPlayersQueryOptions(
    0,
    SEARCH_PAGE_SIZE,
    committed.sortBy,
    committed.sortDir,
    committed.filters,
    committed.filterCombine,
    committed.requestedFields,
    committed.view,
    committed.comparisonPool,
    committed.pageContext,
  );
  const requestedOptions = searchPlayersQueryOptions(
    0,
    SEARCH_PAGE_SIZE,
    requested.sortBy,
    requested.sortDir,
    requested.filters,
    requested.filterCombine,
    requested.requestedFields,
    requested.view,
    requested.comparisonPool,
    requested.pageContext,
  );
  const committedQuery = useQuery(committedOptions);
  const requestedQuery = useQuery(requestedOptions);
  const queryClient = useQueryClient();
  const requestedKey = JSON.stringify(requestedOptions.queryKey);
  const requestedDataUpdateCount =
    queryClient.getQueryState(requestedOptions.queryKey)?.dataUpdateCount ?? 0;
  const requestedVersion = useRef({
    key: requestedKey,
    dataUpdateCount: requestedDataUpdateCount,
  });
  if (requestedVersion.current.key !== requestedKey) {
    requestedVersion.current = {
      key: requestedKey,
      dataUpdateCount: requestedDataUpdateCount,
    };
  }
  const requestMatchesCommitted =
    JSON.stringify(committedOptions.queryKey) ===
    JSON.stringify(requestedOptions.queryKey);
  const isSortReplacement =
    !requestMatchesCommitted &&
    JSON.stringify({
      filters: committed.filters,
      filterCombine: committed.filterCombine,
      requestedFields: committed.requestedFields,
      view: committed.view,
      comparisonPool: committed.comparisonPool,
      pageContext: committed.pageContext,
    }) ===
      JSON.stringify({
        filters: requested.filters,
        filterCombine: requested.filterCombine,
        requestedFields: requested.requestedFields,
        view: requested.view,
        comparisonPool: requested.comparisonPool,
        pageContext: requested.pageContext,
      });
  const isReplacementActive = !requestMatchesCommitted;
  const isReplacementPending = isSortReplacement && requestedQuery.isFetching;
  const replacementError =
    isSortReplacement && requestedQuery.isError ? requestedQuery.error : null;
  const replacementLabel = requested.sortBy.startsWith("potential_role.")
    ? "Calculating and sorting…"
    : "Sorting…";

  useEffect(() => {
    if (
      requestedQuery.isSuccess &&
      !requestedQuery.isFetching &&
      (!requestedQuery.isStale ||
        requestedDataUpdateCount > requestedVersion.current.dataUpdateCount) &&
      !requestMatchesCommitted
    ) {
      setCommitted(requested);
    }
  }, [
    requestMatchesCommitted,
    requested,
    requestedDataUpdateCount,
    requestedQuery.isFetching,
    requestedQuery.isStale,
    requestedQuery.isSuccess,
  ]);

  const listKey = useMemo(
    () =>
      [
        committed.view,
        committed.comparisonPool,
        committed.filterCombine,
        ...committed.filters.map(
          (rule) =>
            `${rule.field}:${rule.op}:${String(filterValueToIpc(rule.value))}`,
        ),
      ].join("|"),
    [committed],
  );
  const page =
    requestMatchesCommitted || isSortReplacement
      ? committedQuery.data
      : undefined;
  if (!page) {
    return (
      <Panel title="Results" flush>
        <EmptyState
          icon={SearchX}
          title={
            requestedQuery.isError
              ? "Could not load players"
              : "Loading players"
          }
          action={
            requestedQuery.isError ? (
              <button
                type="button"
                onClick={() => void requestedQuery.refetch()}
              >
                Retry
              </button>
            ) : undefined
          }
        >
          {requestedQuery.isError
            ? requestedQuery.error.message
            : "Loading player results…"}
        </EmptyState>
      </Panel>
    );
  }
  if (page.total === 0) {
    const appliedFilters = completeFilterRules(filters, view);
    if (appliedFilters.length > 0) {
      return (
        <Panel title="Results" flush>
          <EmptyState icon={SearchX} title="No players match these filters">
            Adjust or clear filters in the strip above to widen the result set.
          </EmptyState>
        </Panel>
      );
    }

    if (view === "shortlist") {
      return (
        <Panel title="Results" flush>
          <EmptyState
            icon={SearchX}
            title="No shortlist yet"
            action={
              <button
                type="button"
                className="rounded-full bg-primary px-4 py-1.5 text-label-md text-on-primary hover:bg-primary/90 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
                onClick={() => {
                  void navigate({
                    to: "/search",
                    search: {
                      view: "moneyball",
                      sort: defaultSearchSort("moneyball"),
                      dir: defaultDirForSortField(
                        defaultSearchSort("moneyball"),
                      ),
                      filters: [],
                      combine: "and",
                      comparisonPool: "filtered",
                    },
                  });
                }}
              >
                Go to Moneyball
              </button>
            }
          >
            No players have been shortlisted yet. Upload a Moneyball CSV in the
            Moneyball tab to create your shortlist.
          </EmptyState>
        </Panel>
      );
    }

    return (
      <Panel title="Results" flush>
        <EmptyState
          icon={SearchX}
          title={
            view === "moneyball"
              ? "No players in this Moneyball import"
              : "No players in snapshot"
          }
        >
          {view === "moneyball"
            ? "Upload a Moneyball CSV for the current snapshot to analyse its matched players."
            : "The snapshot exists but holds no player rows. Run Load Data again with Football Manager in an active save."}
        </EmptyState>
      </Panel>
    );
  }

  const dirLabel = committed.sortDir === "asc" ? "ascending" : "descending";
  const sortMetric =
    committed.view === "moneyball"
      ? getMoneyballSearchMetric(committed.sortBy)
      : getPlayerMetric(committed.sortBy);
  const sortLabel = sortMetric
    ? sortMetric.id === "age"
      ? "Age / DOB"
      : sortMetric.label
    : (columns.find((column) => column.id === committed.sortBy)?.label ??
      committed.sortBy);
  const removeColumn = (metricId: string) => {
    let nextColumnIds = layout.columnIds.filter((id) => id !== metricId);
    if (nextColumnIds.length === layout.columnIds.length) {
      return;
    }
    if (isTacticColumnId(metricId)) {
      const currentSurvives = isFullTacticGroup(nextColumnIds, "current");
      const potentialSurvives = isFullTacticGroup(nextColumnIds, "potential");
      const survivingGroup = currentSurvives
        ? "current"
        : potentialSurvives
          ? "potential"
          : null;
      const persistedLaneIds = nextColumnIds.flatMap((id) => {
        const laneId = tacticLaneIdForId(id);
        return laneId && tacticGroupForId(id) === survivingGroup
          ? [laneId]
          : [];
      });
      const removalOrder =
        orderedLaneIds.length === TACTIC_LANE_IDS.length
          ? orderedLaneIds
          : [...new Set(persistedLaneIds)];
      nextColumnIds = [
        ...nextColumnIds.filter((id) => !isTacticColumnId(id)),
        ...buildTacticColumnOrder(
          removalOrder,
          currentSurvives,
          potentialSurvives,
        ),
      ];
      replaceLayout(tableId, nextColumnIds);
    } else {
      removeStoredColumn(tableId, metricId);
    }
    const persistedColumnIds =
      usePlayerTableStore.getState().layouts[tableId].columnIds;
    if (persistedColumnIds.includes(requested.sortBy)) {
      return;
    }
    const nextSort = persistedColumnIds.includes(defaultSearchSort(view))
      ? defaultSearchSort(view)
      : persistedColumnIds[0];
    if (!nextSort) {
      return;
    }
    onSortChange(nextSort, defaultDirForSortField(nextSort));
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
      {isReplacementPending ? (
        <p
          className="shrink-0 px-4 pb-3 text-body-sm text-on-surface-variant"
          role="status"
        >
          {replacementLabel}
        </p>
      ) : null}
      {replacementError ? (
        <div
          className="flex shrink-0 items-center justify-between gap-3 px-4 pb-3 text-body-sm text-error"
          role="alert"
        >
          <span>Could not sort players. {replacementError.message}</span>
          <button
            type="button"
            className="shrink-0 rounded-full border border-outline px-3 py-1 text-label-md text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
            onClick={() => void requestedQuery.refetch()}
          >
            Retry
          </button>
        </div>
      ) : null}
      <SearchResultsVirtualTable
        key={listKey}
        total={page.total}
        sortBy={committed.sortBy}
        sortDir={committed.sortDir}
        filters={committed.filters}
        filterCombine={committed.filterCombine}
        columns={columns}
        requestedFields={committed.requestedFields}
        view={committed.view}
        comparisonPool={committed.comparisonPool}
        pageContext={committed.pageContext}
        firstPageQueryOptions={committedOptions}
        isReplacementActive={isReplacementActive}
        onSortChange={onSortChange}
        onAddColumn={(metricId) => addColumns(tableId, [metricId])}
        onRemoveColumn={removeColumn}
        onMoveColumn={(metricId, targetIndex) =>
          moveColumn(tableId, metricId, targetIndex)
        }
        onResizeColumn={(metricId, width) =>
          setColumnWidth(tableId, metricId, width)
        }
      />
    </Panel>
  );
}
