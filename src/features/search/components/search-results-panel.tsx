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
import { TablePendingStatus } from "@/components/player-table/table-pending-status";
import {
  type ConfigurableTableIdentity,
  type ConfigurableTableRenderHeader,
  VirtualizedPlayerTable,
} from "@/components/player-table/virtualized-player-table";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { PlayerMetricPicker } from "@/components/ui/player-metric-picker";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { TableSkeleton } from "@/components/ui/skeleton/skeleton";
import {
  isIdentityColumnId,
  usePlayerTableStore,
} from "@/stores/use-player-table-store";
import { formatCount, formatLongGameDate } from "@/utils/format";
import {
  formatMoneyballMetric,
  getMoneyballSearchMetric,
} from "@/utils/moneyball-search-metrics";
import { getSearchMetric, SEARCH_METRICS } from "@/utils/search-metrics";
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
  searchFirstPageQueryOptions,
  searchPlayersQueryOptions,
  searchRequestedFields,
} from "../api/search-players-query-options";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import { filterValueToIpc } from "../types/filter-rule";
import type { PlayerSummary } from "../types/player-summary";
import type { SearchSortDir, SearchSortField } from "../types/search-sort";
import {
  type BASIC_SEARCH_SORT_FIELDS,
  DEFAULT_SEARCH_SORT_FIELD,
  defaultDirForSortField,
  isBasicSearchSortField,
} from "../types/search-sort";
import type { MoneyballComparison } from "../types/search-view";
import {
  completeFilterRules,
  filterFieldsForView,
  getFilterField,
} from "../utils/filter-registry";
import { SEARCH_TABLE_GROUPS } from "../utils/search-table-groups";
import { buildTacticColumnOrder } from "../utils/tactic-columns";
import { type FilterFieldRequest, SearchFilterBar } from "./search-filter-bar";

type TableColumn = PlayerTableColumn;

/**
 * Wraps the rows or the empty result. It stays mounted while either replaces
 * the other, so it fades in once: when the first page replaces the skeleton.
 */
const RESULTS_BODY_CLASS = "flex min-h-0 flex-1 flex-col animate-enter";

const TABLE_ID = "search";

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
  /** Context and full definition for the lane's Moneyball Score column. */
  moneyball?: { context: string; full: string };
};

type SearchResultsPanelProps = {
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  onSortChange: (sortBy: SearchSortField, sortDir: SearchSortDir) => void;
  onRulesChange: (rules: FilterRule[]) => void;
  onApplyFilters: (rules: FilterRule[], combine: FilterCombineMode) => void;
  /** Controls beside Add filter, such as Saved filters. */
  filterActions?: ReactNode;
  datasetToggles?: ReactNode;
  comparison: MoneyballComparison;
  /** The selected shortlist, or null for no shortlist restriction. */
  shortlistId: number | null;
  /** Called with a requested shortlist that the save no longer has. */
  onShortlistMissing?: (shortlistId: number) => void;
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
  laneLabels: ReadonlyMap<string, TacticLaneLabel>,
): TableColumn | undefined {
  if (isValidTacticColumnId(metricId)) {
    const laneId = tacticLaneIdForId(metricId);
    const laneLabel = laneLabels.get(laneId ?? "");
    const groupLabel =
      tacticGroupForId(metricId) === "moneyball"
        ? laneLabel?.moneyball
        : laneLabel;
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
  const metric = getSearchMetric(metricId);
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

const SEARCH_HEADER_METRICS = SEARCH_METRICS.filter(
  (metric) => !isIdentityColumnId(metric.id),
);

/** A season value's hover text: when it comes from a kept snapshot, its date. */
function seasonValueTitle(
  text: string,
  statsDate: string | undefined,
): string | undefined {
  if (text === "—" || text === "…") {
    return undefined;
  }
  return statsDate === undefined
    ? text
    : `${text}, as of ${formatLongGameDate(statsDate)}: under 450 minutes this season`;
}

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
  comparison,
  shortlistId,
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
  comparison: MoneyballComparison;
  shortlistId: number | null;
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
    (metricId: string) => getFilterField(metricId) !== undefined,
    [],
  );
  const renderHeader = useCallback<
    ConfigurableTableRenderHeader<PlayerSummary>
  >(
    ({ identity, columns: tableColumns, fixedColumns }) => (
      <PlayerTableHeader
        columns={tableColumns}
        fixedColumns={fixedColumns}
        groups={SEARCH_TABLE_GROUPS}
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
        metrics={SEARCH_HEADER_METRICS}
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
        comparison,
        shortlistId,
        pageContext,
      ),
    [
      comparison,
      filterCombine,
      filters,
      pageContext,
      requestedFields,
      shortlistId,
      sortBy,
      sortDir,
    ],
  );
  const handlePlayerActivate = useCallback(
    (player: PlayerSummary) => {
      void navigate({
        to: "/players/$uid",
        params: { uid: String(player.uid) },
      });
    },
    [navigate],
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
        const moneyballMetric = getMoneyballSearchMetric(column.id);
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
          // Kept-snapshot values read muted, with their date on hover.
          const borrowed = player?.moneyballStatsDate;
          return (
            <td
              key={column.id}
              className={NUM_CELL}
              title={seasonValueTitle(text, borrowed)}
            >
              <span className="inline-flex items-center justify-end gap-2">
                <span
                  className={
                    value === null || borrowed !== undefined
                      ? "text-on-surface-variant"
                      : undefined
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
                roleName={column.label}
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
    [columns, unresolvedFieldIds],
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
  filterActions,
  datasetToggles,
  comparison,
  shortlistId,
  onShortlistMissing,
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
  const layout = usePlayerTableStore((state) => state.layouts[TABLE_ID]);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const removeStoredColumn = usePlayerTableStore((state) => state.removeColumn);
  const replaceLayout = usePlayerTableStore((state) => state.replaceLayout);
  const moveStoredColumn = usePlayerTableStore((state) => state.moveColumn);
  const setColumnWidth = usePlayerTableStore((state) => state.setColumnWidth);
  const identityWidth = usePlayerTableStore(
    (state) => state.layouts[TABLE_ID].identityWidth,
  );
  const setIdentityWidth = usePlayerTableStore(
    (state) => state.setIdentityWidth,
  );
  const identity = useMemo<ConfigurableTableIdentity<PlayerSummary>>(
    () => ({
      id: "identity",
      label: "Player",
      width: identityWidth,
      onResize: (width) => setIdentityWidth(TABLE_ID, width),
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
    [identityGraphics, identityWidth, setIdentityWidth],
  );
  const columns = useMemo<TableColumn[]>(
    () =>
      layout.columnIds.flatMap((metricId) => {
        const column = tableColumnForMetric(
          metricId,
          layout.widths[metricId],
          laneLabels,
        );
        return column ? [column] : [];
      }),
    [laneLabels, layout],
  );
  const requestedFields = useMemo(
    () => searchRequestedFields(layout.columnIds),
    [layout.columnIds],
  );

  const requested = useMemo(
    () => ({
      sortBy,
      sortDir,
      filters,
      filterCombine,
      requestedFields,
      comparison,
      shortlistId,
      pageContext,
    }),
    [
      comparison,
      filterCombine,
      filters,
      pageContext,
      requestedFields,
      shortlistId,
      sortBy,
      sortDir,
    ],
  );
  const [committed, setCommitted] = useState(requested);
  const [fieldRequest, setFieldRequest] = useState<FilterFieldRequest | null>(
    null,
  );
  const committedOptions = searchFirstPageQueryOptions(committed);
  const namedRequestOptions = searchFirstPageQueryOptions(requested, TABLE_ID);
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
  // same page context; a context change clears them.
  const isRetainedReplacement =
    !requestMatchesCommitted &&
    JSON.stringify(committed.pageContext) ===
      JSON.stringify(requested.pageContext);
  const replacementKind = !isRetainedReplacement
    ? null
    : JSON.stringify([
          committed.filters,
          committed.filterCombine,
          committed.comparison,
          committed.shortlistId,
        ]) !==
        JSON.stringify([
          requested.filters,
          requested.filterCombine,
          requested.comparison,
          requested.shortlistId,
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
      <TablePendingStatus>
        {replacementKind === "filter"
          ? "Filtering…"
          : replacementKind === "columns"
            ? "Loading columns…"
            : requested.sortBy.startsWith("potential_role.")
              ? "Calculating and sorting…"
              : "Sorting…"}
      </TablePendingStatus>
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

  // The requested shortlist is no longer one of the save's shortlists.
  const missingShortlistId =
    requestedQuery.data?.state === "no_shortlist"
      ? requested.shortlistId
      : null;
  useEffect(() => {
    if (missingShortlistId !== null) {
      onShortlistMissing?.(missingShortlistId);
    }
  }, [missingShortlistId, onShortlistMissing]);

  const listKey = useMemo(
    () =>
      [
        committed.comparison,
        String(committed.shortlistId),
        committed.filterCombine,
        ...committed.filters.map(
          (rule) =>
            `${rule.field}:${rule.op}:${String(filterValueToIpc(rule.value))}`,
        ),
      ].join("|"),
    [committed],
  );
  const addColumn = useCallback(
    (metricId: string) => addColumns(TABLE_ID, [metricId], SEARCH_TABLE_GROUPS),
    [addColumns],
  );
  const removeColumn = useCallback(
    (metricId: string) => {
      let nextColumnIds = layout.columnIds.filter((id) => id !== metricId);
      if (nextColumnIds.length === layout.columnIds.length) {
        return;
      }
      if (isTacticColumnId(metricId)) {
        const survivingGroups = tacticColumnGroupsForTable(TABLE_ID).filter(
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
            removeStoredColumn(TABLE_ID, id);
          }
        } else {
          replaceLayout(TABLE_ID, nextColumnIds);
        }
      } else {
        removeStoredColumn(TABLE_ID, metricId);
      }
      const persistedColumnIds =
        usePlayerTableStore.getState().layouts[TABLE_ID].columnIds;
      if (persistedColumnIds.includes(requested.sortBy)) {
        return;
      }
      const nextSort = persistedColumnIds.includes(DEFAULT_SEARCH_SORT_FIELD)
        ? DEFAULT_SEARCH_SORT_FIELD
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
    ],
  );
  const moveColumn = useCallback(
    (metricId: string, targetIndex: number) =>
      moveStoredColumn(TABLE_ID, metricId, targetIndex),
    [moveStoredColumn],
  );
  const resizeColumn = useCallback(
    (metricId: string, width: number) =>
      setColumnWidth(TABLE_ID, metricId, width),
    [setColumnWidth],
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
      groups={SEARCH_TABLE_GROUPS}
      metrics={SEARCH_HEADER_METRICS}
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
      comparison={comparison}
      fieldRequest={fieldRequest}
      filterActions={filterActions}
      summary={summary}
      columnsControl={columnsControl}
      datasetToggles={datasetToggles}
    />
  );
  if (!page) {
    return (
      <Panel
        flush
        className="flex min-h-0 flex-1 flex-col"
        contentClassName="flex min-h-0 flex-1 flex-col"
      >
        {renderToolbar()}
        {requestedError ? (
          <EmptyState
            icon={SearchX}
            title="Could not load players"
            action={
              <button
                type="button"
                onClick={() => void requestedQuery.refetch()}
              >
                Retry
              </button>
            }
          >
            {requestedError.message}
          </EmptyState>
        ) : (
          <TableSkeleton label="Loading player results…" />
        )}
      </Panel>
    );
  }
  const dirLabel = committed.sortDir === "asc" ? "ascending" : "descending";
  const sortMetric = getSearchMetric(committed.sortBy);
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
        metrics={filterFieldsForView().filter(
          (metric) => metric.sortable !== false,
        )}
        groups={SEARCH_TABLE_GROUPS}
        value={committed.sortBy}
        placeholder={sortLabel}
        onChange={(field) => {
          addColumns(TABLE_ID, [field], SEARCH_TABLE_GROUPS);
          onSortChange(field, defaultDirForSortField(field));
        }}
      />{" "}
      (
      <button
        type="button"
        aria-label={`Sort direction: ${dirLabel}`}
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
    const appliedFilters = completeFilterRules(committed.filters);
    const shortlistEmpty = committed.shortlistId !== null;
    const emptyTitle = shortlistEmpty
      ? appliedFilters.length > 0
        ? "No shortlisted players match these filters"
        : "No shortlisted player in this snapshot"
      : appliedFilters.length > 0
        ? "No players match these filters"
        : "No players in snapshot";
    const seasonStatFiltered = appliedFilters.some((rule) =>
      rule.field.startsWith("moneyball"),
    );
    const emptyBody = shortlistEmpty
      ? appliedFilters.length > 0
        ? "Clear filters, or select No shortlist."
        : "Select No shortlist to browse every player."
      : appliedFilters.length > 0
        ? seasonStatFiltered
          ? "Clear filters to see more players. Players under 450 minutes drop out of season-stat filters."
          : "Clear filters to see more players."
        : "Run Load Data again with a save open in Football Manager.";

    return (
      <Panel
        flush
        className="flex min-h-0 flex-1 flex-col"
        contentClassName="flex min-h-0 flex-1 flex-col"
      >
        {renderToolbar(summary)}
        {replacementFeedback}
        <div className={RESULTS_BODY_CLASS}>
          <EmptyState icon={SearchX} title={emptyTitle}>
            {emptyBody}
          </EmptyState>
        </div>
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
      <div className={RESULTS_BODY_CLASS}>
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
          comparison={committed.comparison}
          shortlistId={committed.shortlistId}
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
      </div>
    </Panel>
  );
}
