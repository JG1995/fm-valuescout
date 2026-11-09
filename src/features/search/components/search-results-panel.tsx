import { useQuery, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { SearchX } from "lucide-react";
import {
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { NationalityCell } from "@/components/player-table/nationality-cell";
import {
  ConfigurableColumnsControl,
  type PlayerTableColumn,
  PlayerTableHeader,
} from "@/components/player-table/player-table-header";
import {
  formatTableDynamicCell as formatDynamicCell,
  formatPlayerBasicCell,
  TABLE_NUMERIC_CELL_CLASS as NUM_CELL,
  TableScoreContent,
  TABLE_TEXT_CELL_CLASS as TEXT_CELL,
} from "@/components/player-table/table-cells";
import {
  type ConfigurableTableIdentity,
  type ConfigurableTableRenderHeader,
  VirtualizedPlayerTable,
} from "@/components/player-table/virtualized-player-table";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { PlayerMetricPicker } from "@/components/ui/player-metric-picker";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import {
  isIdentityColumnId,
  usePlayerTableStore,
} from "@/stores/use-player-table-store";
import { formatCount } from "@/utils/format";
import {
  formatMoneyballMetric,
  getMoneyballSearchMetric,
  MONEYBALL_SEARCH_METRICS,
} from "@/utils/moneyball-search-metrics";
import { getPlayerMetric, PLAYER_METRICS } from "@/utils/player-metrics";
import {
  isFullTacticGroup,
  isTacticColumnId,
  isValidTacticColumnId,
  TACTIC_COLUMN_DEFAULT_WIDTH,
  TACTIC_LANE_IDS,
  tacticColumnGroupsForTable,
  tacticGroupForId,
  tacticLaneIdForId,
} from "@/utils/tactic-ids";
import { isSupersededError } from "../api/fetch-search-players";
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
import {
  completeFilterRules,
  filterFieldsForView,
  getFilterField,
} from "../utils/filter-registry";
import { searchTableGroups } from "../utils/search-table-groups";
import { buildTacticColumnOrder } from "../utils/tactic-columns";
import { type FilterFieldRequest, SearchFilterBar } from "./search-filter-bar";

type TableColumn = PlayerTableColumn;

/**
 * Route-owned tactic lane label model. The compact placement identifier is
 * the primary leaf text, the role context renders smaller beside it, and
 * the full `IP Position (Role) / OOP Position (Role)` definition stays the
 * complete accessible name with a focus-revealed visible disclosure.
 */
export type TacticLaneLabel = {
  compact: string;
  context: string;
  full: string;
  /** Context and full definition for the lane's Moneyball Custom column. */
  custom?: { context: string; full: string };
};

type SearchResultsPanelProps = {
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  onSortChange: (sortBy: SearchSortField, sortDir: SearchSortDir) => void;
  onRulesChange: (rules: FilterRule[]) => void;
  onApplyFilters: (rules: FilterRule[], combine: FilterCombineMode) => void;
  datasetToggles?: ReactNode;
  view: SearchView;
  comparisonPool: ComparisonPool;
  shortlistOnly: boolean;
  pageContext: SearchPlayerPageContext;
  orderedLaneIds: readonly string[];
  laneLabels: ReadonlyMap<string, TacticLaneLabel>;
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

export function tableColumnForMetric(
  metricId: string,
  width: number | undefined,
  view: SearchView,
  laneLabels: ReadonlyMap<string, TacticLaneLabel>,
): TableColumn | undefined {
  if (isValidTacticColumnId(metricId)) {
    const laneId = tacticLaneIdForId(metricId);
    const laneLabel = laneLabels.get(laneId ?? "");
    const groupLabel =
      tacticGroupForId(metricId) === "custom" ? laneLabel?.custom : laneLabel;
    return {
      id: metricId,
      label: laneLabel?.compact ?? laneId ?? metricId,
      secondaryLabel: groupLabel?.context,
      accessibleLabel: groupLabel?.full ?? laneId ?? metricId,
      align: "right",
      width: width ?? TACTIC_COLUMN_DEFAULT_WIDTH,
    };
  }
  // Compact leaf header: visible `Nat.` keeps the full `Nationality` as
  // the accessible name (sort, menu, disclosure). Catalog labels stay
  // untouched so filters and pickers keep their full names.
  if (view === "moneyball") {
    const metric = getMoneyballSearchMetric(metricId);
    if (metric) {
      return {
        id: metric.id,
        label: metric.id === "nationality" ? "Nat." : metric.label,
        accessibleLabel:
          metric.id === "nationality" ? "Nationality" : undefined,
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
    label: metric.id === "nationality" ? "Nat." : metric.label,
    accessibleLabel: metric.id === "nationality" ? "Nationality" : undefined,
    align: metric.align,
    width: width ?? metric.defaultWidth,
  };
}

const SEARCH_HEADER_METRICS = PLAYER_METRICS.filter(
  (metric) => !isIdentityColumnId(metric.id),
);

const MONEYBALL_HEADER_METRICS = MONEYBALL_SEARCH_METRICS.filter(
  (metric) => !isIdentityColumnId(metric.id),
);

function PlayerIdentityCell({
  name,
  club,
  division,
  portrait,
  crest,
}: {
  name: string | undefined;
  club: string | null | undefined;
  division: string | null | undefined;
  portrait?: ReactNode;
  crest?: ReactNode;
}) {
  const context =
    name === undefined
      ? null
      : [club, division]
          .filter((value): value is string => value !== null && value !== "")
          .join(" · ");
  return (
    <div className="flex h-table-row-height-two-line items-center gap-2 px-2">
      <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-sm bg-surface-container-high">
        {portrait}
      </span>
      <span className="min-w-0 flex-1">
        <span
          className="block truncate text-body-sm text-on-surface"
          title={name}
        >
          {name ?? "…"}
        </span>
        {name !== undefined ? (
          <span className="flex min-w-0 items-center gap-1 text-[11px] leading-4 text-on-surface-variant">
            <span className="flex h-3 w-3 shrink-0 items-center justify-center rounded-[2px] bg-surface-container-high">
              {crest}
            </span>
            {context ? <span className="block truncate">{context}</span> : null}
          </span>
        ) : null}
      </span>
    </div>
  );
}

function SearchResultsVirtualTable({
  total,
  sortBy,
  sortDir,
  filters,
  filterCombine,
  columns,
  identity,
  requestedFields,
  onSortChange,
  onAddColumn,
  onRemoveColumn,
  onMoveColumn,
  onResizeColumn,
  onFilterColumn,
  view,
  comparisonPool,
  shortlistOnly,
  pageContext,
  firstPageQueryOptions,
  isReplacementActive,
  unresolvedFieldIds,
}: {
  total: number;
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  columns: TableColumn[];
  identity: ConfigurableTableIdentity<PlayerSummary>;
  requestedFields: string[];
  onSortChange: (sortBy: SearchSortField, sortDir: SearchSortDir) => void;
  onAddColumn: (metricId: string) => void;
  onRemoveColumn: (metricId: string) => void;
  onMoveColumn: (metricId: string, targetIndex: number) => void;
  onResizeColumn: (metricId: string, width: number) => void;
  onFilterColumn: (metricId: string) => void;
  view: SearchView;
  comparisonPool: ComparisonPool;
  shortlistOnly: boolean;
  pageContext: SearchPlayerPageContext;
  firstPageQueryOptions: ReturnType<typeof searchPlayersQueryOptions>;
  isReplacementActive: boolean;
  unresolvedFieldIds: ReadonlySet<string>;
}) {
  const navigate = useNavigate();
  // Stable inputs keep the memoized header and rows from re-rendering on
  // scroll; each hook lists every value its output reads.
  const handleHeaderSort = useCallback(
    (metricId: SearchSortField) => {
      const next = nextSort(sortBy, sortDir, metricId);
      onSortChange(next.sortBy, next.sortDir);
    },
    [onSortChange, sortBy, sortDir],
  );
  const handleSortDirection = useCallback(
    (metricId: SearchSortField, dir: SearchSortDir) =>
      onSortChange(metricId, dir),
    [onSortChange],
  );
  const canFilterColumn = useCallback(
    (metricId: string) => getFilterField(metricId, view) !== undefined,
    [view],
  );
  const renderHeader = useCallback<
    ConfigurableTableRenderHeader<PlayerSummary>
  >(
    ({ identity, columns: tableColumns, fixedColumns }) => (
      <PlayerTableHeader
        columns={tableColumns}
        fixedColumns={fixedColumns}
        groups={searchTableGroups(view)}
        identity={identity}
        sortBy={sortBy}
        sortDir={sortDir}
        onSortChange={handleHeaderSort}
        onAddColumn={onAddColumn}
        onRemoveColumn={onRemoveColumn}
        onMoveColumn={onMoveColumn}
        onResizeColumn={onResizeColumn}
        onSortDirection={handleSortDirection}
        onFilterColumn={onFilterColumn}
        canFilterColumn={canFilterColumn}
        metrics={
          view === "moneyball"
            ? MONEYBALL_HEADER_METRICS
            : SEARCH_HEADER_METRICS
        }
      />
    ),
    [
      canFilterColumn,
      handleHeaderSort,
      handleSortDirection,
      onAddColumn,
      onFilterColumn,
      onMoveColumn,
      onRemoveColumn,
      onResizeColumn,
      sortBy,
      sortDir,
      view,
    ],
  );
  const pageQueryOptions = useCallback(
    (offset: number, limit: number) =>
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
        shortlistOnly,
        pageContext,
      ),
    [
      comparisonPool,
      filterCombine,
      filters,
      pageContext,
      requestedFields,
      shortlistOnly,
      sortBy,
      sortDir,
      view,
    ],
  );
  const handlePlayerActivate = useCallback(
    (player: PlayerSummary) => {
      void navigate({
        to: "/players/$uid",
        params: { uid: String(player.uid) },
        search: { view },
      });
    },
    [navigate, view],
  );
  const renderCells = useCallback(
    (row: PlayerSummary | undefined) =>
      columns.map((column) => {
        // Fields the replacement query has not delivered yet render as
        // loading, never as truthful-missing.
        const player = unresolvedFieldIds.has(column.id) ? undefined : row;
        if (isValidTacticColumnId(column.id)) {
          const score = player?.dynamicValues?.[column.id];
          const roleName = column.accessibleLabel ?? column.label;
          return (
            <td key={column.id} className={NUM_CELL}>
              <TableScoreContent
                score={score}
                roleName={roleName}
                isLoading={player === undefined}
              />
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
          const text =
            player === undefined
              ? "…"
              : moneyballMetric.metric
                ? formatMoneyballMetric(moneyballMetric.metric, value)
                : value === null
                  ? "—"
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
              <TableScoreContent
                score={score}
                roleName={`Moneyball role · ${column.label}`}
                isLoading={player === undefined}
              />
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
                <TableScoreContent
                  score={score}
                  roleName={column.label}
                  isLoading={player === undefined}
                />
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
        const cell = formatPlayerBasicCell(
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
      }),
    [columns, unresolvedFieldIds, view],
  );

  return (
    <VirtualizedPlayerTable
      caption="Player search results"
      columnCount={columns.length}
      columns={columns}
      identity={identity}
      renderHeader={renderHeader}
      firstPageQueryOptions={firstPageQueryOptions}
      isReplacementActive={isReplacementActive}
      pageQueryOptions={pageQueryOptions}
      pageSize={SEARCH_PAGE_SIZE}
      renderCells={renderCells}
      testId="search-results-scroller"
      total={total}
      onPlayerActivate={handlePlayerActivate}
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
  onRulesChange,
  onApplyFilters,
  datasetToggles,
  view,
  comparisonPool,
  shortlistOnly,
  pageContext,
  orderedLaneIds,
  laneLabels,
  identityGraphics,
}: SearchResultsPanelProps & {
  identityGraphics?: (player: PlayerSummary | undefined) => {
    portrait?: ReactNode;
    crest?: ReactNode;
  };
}) {
  const tableId = view === "moneyball" ? "moneyball-search" : "search";
  const layout = usePlayerTableStore((state) => state.layouts[tableId]);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const removeStoredColumn = usePlayerTableStore((state) => state.removeColumn);
  const replaceLayout = usePlayerTableStore((state) => state.replaceLayout);
  const moveStoredColumn = usePlayerTableStore((state) => state.moveColumn);
  const setColumnWidth = usePlayerTableStore((state) => state.setColumnWidth);
  const identityWidth = usePlayerTableStore(
    (state) => state.layouts[tableId].identityWidth,
  );
  const setIdentityWidth = usePlayerTableStore(
    (state) => state.setIdentityWidth,
  );
  const identity = useMemo<ConfigurableTableIdentity<PlayerSummary>>(
    () => ({
      id: "identity",
      label: "Player",
      width: identityWidth,
      onResize: (width) => setIdentityWidth(tableId, width),
      renderCell: (player) => (
        <PlayerIdentityCell
          name={player?.name}
          club={player?.club}
          division={player?.division}
          portrait={identityGraphics?.(player)?.portrait}
          crest={identityGraphics?.(player)?.crest}
        />
      ),
    }),
    [identityGraphics, identityWidth, setIdentityWidth, tableId],
  );
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
      shortlistOnly,
      pageContext,
    }),
    [
      comparisonPool,
      filterCombine,
      filters,
      pageContext,
      requestedFields,
      shortlistOnly,
      sortBy,
      sortDir,
      view,
    ],
  );
  const [committed, setCommitted] = useState(requested);
  const [fieldRequest, setFieldRequest] = useState<FilterFieldRequest | null>(
    null,
  );
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
    committed.shortlistOnly,
    committed.pageContext,
  );
  const namedRequestOptions = searchPlayersQueryOptions(
    0,
    SEARCH_PAGE_SIZE,
    requested.sortBy,
    requested.sortDir,
    requested.filters,
    requested.filterCombine,
    requested.requestedFields,
    requested.view,
    requested.comparisonPool,
    requested.shortlistOnly,
    requested.pageContext,
    tableId,
  );
  const requestMatchesCommitted =
    JSON.stringify(committedOptions.queryKey) ===
    JSON.stringify(namedRequestOptions.queryKey);
  // A shared key shares one Query whose options the last observer sets, so a
  // request matching the committed page must not name the table: a refetch
  // of the committed page could otherwise be superseded.
  const requestedOptions = requestMatchesCommitted
    ? committedOptions
    : namedRequestOptions;
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
  // The committed rows stay rendered, inactive, for every change inside the
  // same view and page context; a view or context change clears them.
  const isRetainedReplacement =
    !requestMatchesCommitted &&
    committed.view === requested.view &&
    JSON.stringify(committed.pageContext) ===
      JSON.stringify(requested.pageContext);
  const replacementKind = !isRetainedReplacement
    ? null
    : JSON.stringify([
          committed.filters,
          committed.filterCombine,
          committed.comparisonPool,
          committed.shortlistOnly,
        ]) !==
        JSON.stringify([
          requested.filters,
          requested.filterCombine,
          requested.comparisonPool,
          requested.shortlistOnly,
        ])
      ? "filter"
      : committed.sortBy !== requested.sortBy ||
          committed.sortDir !== requested.sortDir
        ? "sort"
        : "columns";
  const unresolvedFieldIds = useMemo(
    () =>
      new Set(
        isRetainedReplacement
          ? requested.requestedFields.filter(
              (field) => !committed.requestedFields.includes(field),
            )
          : [],
      ),
    [
      committed.requestedFields,
      isRetainedReplacement,
      requested.requestedFields,
    ],
  );
  const isReplacementActive = !requestMatchesCommitted;
  // A newer request stopped this one in Rust; it is pending again, never failed.
  const isRequestedSuperseded =
    requestedQuery.isError && isSupersededError(requestedQuery.error);
  const requestedError =
    requestedQuery.isError && !isRequestedSuperseded
      ? requestedQuery.error
      : null;
  const replacementFeedback =
    replacementKind === null ? null : requestedError ? (
      <div
        className="flex shrink-0 items-center justify-between gap-3 px-4 pb-3 text-body-sm text-error"
        role="alert"
      >
        <span>
          {replacementKind === "filter"
            ? "Could not filter players."
            : replacementKind === "sort"
              ? "Could not sort players."
              : "Could not load player columns."}{" "}
          {requestedError.message}
        </span>
        <button
          type="button"
          className="h-8 shrink-0 rounded-md border border-outline px-3 text-label-md text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
          onClick={() => void requestedQuery.refetch()}
        >
          Retry
        </button>
      </div>
    ) : (
      <p
        className="shrink-0 px-4 pb-3 text-body-sm text-on-surface-variant"
        role="status"
      >
        {replacementKind === "filter"
          ? "Filtering…"
          : replacementKind === "columns"
            ? "Loading columns…"
            : requested.sortBy.startsWith("potential_role.")
              ? "Calculating and sorting…"
              : "Sorting…"}
      </p>
    );

  // The user came back to a query whose request was superseded while it ran.
  useEffect(() => {
    if (isRequestedSuperseded && !requestedQuery.isFetching) {
      void requestedQuery.refetch();
    }
  }, [
    isRequestedSuperseded,
    requestedQuery.isFetching,
    requestedQuery.refetch,
  ]);

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
        String(committed.shortlistOnly),
        committed.filterCombine,
        ...committed.filters.map(
          (rule) =>
            `${rule.field}:${rule.op}:${String(filterValueToIpc(rule.value))}`,
        ),
      ].join("|"),
    [committed],
  );
  const addColumn = useCallback(
    (metricId: string) =>
      addColumns(tableId, [metricId], searchTableGroups(view)),
    [addColumns, tableId, view],
  );
  const removeColumn = useCallback(
    (metricId: string) => {
      let nextColumnIds = layout.columnIds.filter((id) => id !== metricId);
      if (nextColumnIds.length === layout.columnIds.length) {
        return;
      }
      if (isTacticColumnId(metricId)) {
        const survivingGroups = tacticColumnGroupsForTable(tableId).filter(
          (group) => isFullTacticGroup(nextColumnIds, group),
        );
        const persistedLaneIds = nextColumnIds.flatMap((id) => {
          const laneId = tacticLaneIdForId(id);
          return laneId && tacticGroupForId(id) === survivingGroups[0]
            ? [laneId]
            : [];
        });
        const removalOrder =
          orderedLaneIds.length === TACTIC_LANE_IDS.length
            ? orderedLaneIds
            : [...new Set(persistedLaneIds)];
        nextColumnIds = [
          ...nextColumnIds.filter((id) => !isTacticColumnId(id)),
          ...buildTacticColumnOrder(removalOrder, survivingGroups),
        ];
        if (nextColumnIds.length === 0) {
          // Identity-only is valid and never rolls to defaults: emptying the
          // last tactic leaf must persist `columnIds: []`. `replaceLayout`
          // treats an empty list as malformed and restores defaults, so remove
          // through the existing per-column store path instead.
          for (const id of layout.columnIds) {
            removeStoredColumn(tableId, id);
          }
        } else {
          replaceLayout(tableId, nextColumnIds);
        }
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
    },
    [
      layout.columnIds,
      onSortChange,
      orderedLaneIds,
      removeStoredColumn,
      replaceLayout,
      requested.sortBy,
      tableId,
      view,
    ],
  );
  const moveColumn = useCallback(
    (metricId: string, targetIndex: number) =>
      moveStoredColumn(tableId, metricId, targetIndex),
    [moveStoredColumn, tableId],
  );
  const resizeColumn = useCallback(
    (metricId: string, width: number) =>
      setColumnWidth(tableId, metricId, width),
    [setColumnWidth, tableId],
  );
  const filterColumn = useCallback(
    (field: string) => setFieldRequest({ field, key: Date.now() }),
    [],
  );
  const page =
    requestMatchesCommitted || isRetainedReplacement
      ? committedQuery.data
      : undefined;
  const columnsControl = (
    <ConfigurableColumnsControl
      groups={searchTableGroups(view)}
      metrics={
        view === "moneyball" ? MONEYBALL_HEADER_METRICS : SEARCH_HEADER_METRICS
      }
      visibleColumnIds={layout.columnIds}
      onAddColumn={addColumn}
      onRemoveColumn={removeColumn}
    />
  );
  const renderToolbar = (summary?: ReactNode) => (
    <SearchFilterBar
      rules={filters}
      combine={filterCombine}
      onRulesChange={onRulesChange}
      onApply={onApplyFilters}
      view={view}
      comparisonPool={comparisonPool}
      fieldRequest={fieldRequest}
      summary={summary}
      columnsControl={columnsControl}
      datasetToggles={datasetToggles}
    />
  );
  if (!page) {
    return (
      <Panel flush>
        {renderToolbar()}
        <EmptyState
          icon={SearchX}
          title={requestedError ? "Could not load players" : "Loading players"}
          action={
            requestedError ? (
              <button
                type="button"
                onClick={() => void requestedQuery.refetch()}
              >
                Retry
              </button>
            ) : undefined
          }
        >
          {requestedError ? requestedError.message : "Loading player results…"}
        </EmptyState>
      </Panel>
    );
  }
  const dirLabel = committed.sortDir === "asc" ? "ascending" : "descending";
  const sortMetric =
    committed.view === "moneyball"
      ? getMoneyballSearchMetric(committed.sortBy)
      : getPlayerMetric(committed.sortBy);
  const sortColumn = columns.find((column) => column.id === committed.sortBy);
  const sortLabel = sortMetric
    ? sortMetric.label
    : (sortColumn?.accessibleLabel ?? sortColumn?.label ?? committed.sortBy);
  // "N players · sorted by [field] ([direction])": both parts are controls.
  // Choosing a field also shows its column, so the sort is never invisible.
  const summary = (
    <div
      className="text-body-md text-on-surface-variant"
      data-testid="search-results-summary"
    >
      <span className="text-on-surface">{formatCount(page.total)}</span> players
      · sorted by{" "}
      <PlayerMetricPicker
        label="Sort by"
        searchLabel="Search fields"
        variant="inline"
        metrics={filterFieldsForView(view).filter(
          (metric) => metric.sortable !== false,
        )}
        groups={searchTableGroups(view)}
        value={committed.sortBy}
        placeholder={sortLabel}
        onChange={(field) => {
          addColumns(tableId, [field], searchTableGroups(view));
          onSortChange(field, defaultDirForSortField(field));
        }}
      />{" "}
      (
      <button
        type="button"
        aria-label={`Sort direction: ${dirLabel}`}
        title="Reverse the sort direction"
        className="rounded-sm text-on-surface underline decoration-dotted underline-offset-4 hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
        onClick={() =>
          onSortChange(
            committed.sortBy,
            committed.sortDir === "asc" ? "desc" : "asc",
          )
        }
      >
        {dirLabel}
      </button>
      )
    </div>
  );
  if (page.total === 0) {
    // The copy describes the committed rows, not a request still loading.
    const appliedFilters = completeFilterRules(
      committed.filters,
      committed.view,
    );
    const shortlistEmpty = committed.shortlistOnly;
    const emptyTitle = shortlistEmpty
      ? page.state === "no_shortlist"
        ? "No shortlist linked"
        : appliedFilters.length > 0
          ? "No shortlisted players match these filters"
          : committed.view === "moneyball"
            ? "No shortlisted player has season stats"
            : "No shortlisted player in this snapshot"
      : appliedFilters.length > 0
        ? "No players match these filters"
        : committed.view === "moneyball"
          ? "No Moneyball stats for this snapshot"
          : "No players in snapshot";
    const emptyBody = shortlistEmpty
      ? page.state === "no_shortlist"
        ? "Use Choose FM Shortlist to link a shortlist file exported from Football Manager, or turn Shortlist off to browse every snapshot player."
        : appliedFilters.length > 0
          ? "Adjust or clear filters in the toolbar above, or turn Shortlist off to widen the result set."
          : committed.view === "moneyball"
            ? "None of the shortlist's players have season stats in this snapshot. Turn Shortlist off to compare every player with season stats."
            : "None of the shortlist's players are in this snapshot. Turn Shortlist off to browse every snapshot player."
      : appliedFilters.length > 0
        ? "Adjust or clear filters in the toolbar above to widen the result set."
        : committed.view === "moneyball"
          ? "Moneyball stats come from the FM save file linked to this save. Use Refresh stats to read it, or link a save file in Settings."
          : "The snapshot exists but holds no player rows. Run Load Data again with Football Manager in an active save.";

    return (
      <Panel
        flush
        className="flex min-h-0 flex-1 flex-col"
        contentClassName="flex min-h-0 flex-1 flex-col"
      >
        {renderToolbar(summary)}
        {replacementFeedback}
        <EmptyState icon={SearchX} title={emptyTitle}>
          {emptyBody}
        </EmptyState>
      </Panel>
    );
  }

  return (
    <Panel
      flush
      className="flex min-h-0 flex-1 flex-col"
      contentClassName="flex min-h-0 flex-1 flex-col"
    >
      {renderToolbar(summary)}
      {replacementFeedback}
      <SearchResultsVirtualTable
        key={listKey}
        total={page.total}
        sortBy={committed.sortBy}
        sortDir={committed.sortDir}
        filters={committed.filters}
        filterCombine={committed.filterCombine}
        columns={columns}
        identity={identity}
        requestedFields={committed.requestedFields}
        view={committed.view}
        comparisonPool={committed.comparisonPool}
        shortlistOnly={committed.shortlistOnly}
        pageContext={committed.pageContext}
        firstPageQueryOptions={committedOptions}
        isReplacementActive={isReplacementActive}
        unresolvedFieldIds={unresolvedFieldIds}
        onSortChange={onSortChange}
        onAddColumn={addColumn}
        onRemoveColumn={removeColumn}
        onMoveColumn={moveColumn}
        onFilterColumn={filterColumn}
        onResizeColumn={resizeColumn}
      />
    </Panel>
  );
}
