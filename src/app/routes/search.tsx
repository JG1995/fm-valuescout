import {
  useIsMutating,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap, RefreshCw } from "lucide-react";
import { Suspense, useCallback, useEffect, useMemo } from "react";
import { clearPlayerResultContext } from "@/app/player-result-context";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { dashboardKeys } from "@/features/dashboard/api/dashboard-keys";
import { GraphicsImage } from "@/features/graphics/components/graphics-image";
import { managedClubQueryOptions } from "@/features/managed-club/api/managed-club-query-options";
import {
  type RefreshSeasonStatsResult,
  refreshSeasonStats,
} from "@/features/memory-read/api/refresh-season-stats";
import { seasonStatsCopy } from "@/features/memory-read/components/load-data-outcome";
import { loadDataMutationKey } from "@/features/memory-read/hooks/use-load-data";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import { MoneyballLeagueToggle } from "@/features/moneyball/components/moneyball-league-toggle";
import { MoneyballStatsAsOf } from "@/features/moneyball/components/moneyball-profile-panel";
import type { TacticContextBoundaryState } from "@/features/planner/components/tactic-context-boundary";
import { TacticContextBoundary } from "@/features/planner/components/tactic-context-boundary";
import type {
  PlannerTactic,
  TacticOptions,
} from "@/features/planner/types/tactic";
import {
  basePosition,
  orderedTacticLanes,
  validateTacticDraft,
} from "@/features/planner/utils/tactic-editor";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { RealisticSigningsToggle } from "@/features/search/components/realistic-signings-toggle";
import {
  SavedFiltersControl,
  type SlotFilterPreset,
} from "@/features/search/components/saved-filters-control";
import { SearchFilterBar } from "@/features/search/components/search-filter-bar";
import type { TacticLaneLabel } from "@/features/search/components/search-results-panel";
import { SearchResultsPanel } from "@/features/search/components/search-results-panel";
import { TacticColumnToggles } from "@/features/search/components/tactic-column-toggles";
import {
  createFilterRuleId,
  type FilterCombineMode,
  type FilterRule,
} from "@/features/search/types/filter-rule";
import type { PlayerSummary } from "@/features/search/types/player-summary";
import type {
  SearchSortDir,
  SearchSortField,
} from "@/features/search/types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  DEFAULT_SEARCH_SORT_FIELD,
  defaultDirForSortField,
  isSearchSortDir,
} from "@/features/search/types/search-sort";
import {
  type MoneyballComparison,
  moneyballComparisonLabel,
  parseMoneyballComparison,
} from "@/features/search/types/search-view";
import { isVisibleSortField } from "@/features/search/utils/dynamic-columns";
import { recruitmentFilterToRule } from "@/features/search/utils/recruitment-filters";
import { SEARCH_TABLE_GROUPS } from "@/features/search/utils/search-table-groups";
import type { FilterRuleUrl } from "@/features/search/utils/search-url-search";
import {
  parseSearchCombine,
  parseSearchFilters,
  parseShortlistOnly,
  searchFiltersForUrl,
} from "@/features/search/utils/search-url-search";
import { buildTacticColumnOrder } from "@/features/search/utils/tactic-columns";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { staffKeys } from "@/features/staff/api/staff-keys";
import {
  playerShortlistSelectionKey,
  usePlayerShortlistSelectionStore,
} from "@/stores/use-player-shortlist-selection-store";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import { useSearchViewMemory } from "@/stores/use-search-view-memory";
import { cn } from "@/utils/cn";
import { moneyballMetric } from "@/utils/moneyball-metrics";
import { POSITION_FAMILIES, positionFamilyOf } from "@/utils/position-families";
import { isSeasonFieldId } from "@/utils/search-metrics";
import {
  isFullTacticGroup,
  isTacticColumnId,
  type TacticColumnGroup,
  tacticColumnGroupsForTable,
} from "@/utils/tactic-ids";

const TACTIC_GROUP_LABELS: Record<TacticColumnGroup, string> = {
  current: "Role Score",
  potential: "Potential",
  moneyball: "Moneyball Score",
};

const TABLE_ID = "search";

function SearchIdentityGraphics({
  player,
  kind,
}: {
  player: PlayerSummary | undefined;
  kind: "portrait" | "crest";
}) {
  const uid = kind === "portrait" ? player?.uid : player?.currentClubUid;
  return (
    <GraphicsImage
      kind={kind === "portrait" ? "personPortrait" : "clubLogo"}
      uid={uid ?? 0}
      slot={{
        alt: "",
        className:
          kind === "portrait"
            ? "size-7 rounded-sm object-contain"
            : "size-3 object-contain",
      }}
    />
  );
}

export type SearchRouteSearch = {
  sort: SearchSortField;
  dir: SearchSortDir;
  /** Flat URL shape — validateSearch always normalizes to this. */
  filters: FilterRuleUrl[];
  combine: FilterCombineMode;
  /** Whom statistic percentiles rank against; `own` is left out of the URL. */
  comparison?: MoneyballComparison;
  shortlistOnly?: boolean;
};

export const Route = createFileRoute("/search")({
  validateSearch: (search: Record<string, unknown>): SearchRouteSearch => {
    // Legacy Player Shortlist links keep `shortlistOnly` as a marker; the page
    // then clears the shortlist selection and removes the marker. The old
    // General and Moneyball `view` is dropped.
    const legacyShortlist = search.view === "shortlist";
    const filters = searchFiltersForUrl(parseSearchFilters(search.filters));
    const visibleColumnIds =
      usePlayerTableStore.getState().layouts[TABLE_ID].columnIds;
    const visibleSort = isVisibleSortField(search.sort, visibleColumnIds)
      ? search.sort
      : null;
    const sort = visibleSort ?? DEFAULT_SEARCH_SORT_FIELD;
    const dir = isSearchSortDir(search.dir)
      ? search.dir
      : visibleSort !== null
        ? defaultDirForSortField(sort)
        : DEFAULT_SEARCH_SORT_DIR;
    const comparison = parseMoneyballComparison(search.comparison);
    return {
      sort,
      dir,
      filters,
      combine: parseSearchCombine(search.combine),
      comparison: comparison === "own" ? undefined : comparison,
      shortlistOnly: legacyShortlist
        ? true
        : parseShortlistOnly(search.shortlistOnly),
    };
  },
  beforeLoad: ({ location, search }) => {
    // An old `view` link replaces its history entry with a clean URL; a legacy
    // Player Shortlist link keeps the shortlistOnly marker. location.search is
    // the raw query and search the validated one; the router keeps raw keys
    // the redirect does not name, so it clears `view` explicitly.
    const view = (location.search as Record<string, unknown> | undefined)?.view;
    if (view !== undefined) {
      throw Route.redirect({
        to: "/search",
        search: {
          ...search,
          view: undefined,
          ...(view === "shortlist" ? { shortlistOnly: true } : {}),
        } as SearchRouteSearch,
        replace: true,
      });
    }
  },
  loaderDeps: ({ search: { sort, dir, filters, combine, comparison } }) => ({
    sort,
    dir,
    filters,
    combine,
    comparison: comparison ?? "own",
  }),
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(savesQueryOptions),
    ]),
  component: SearchPage,
});

function PanelFallback() {
  return (
    <>
      <h1 className="text-headline-md text-on-surface">Player Search</h1>
      <div className="flex min-h-40 flex-1 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
        Loading search results…
      </div>
    </>
  );
}

function tacticLaneLabels(
  tactic: PlannerTactic,
  options: TacticOptions,
): Map<string, TacticLaneLabel> {
  const roleNames = new Map(
    options.roles.map((role) => [role.roleId, role.displayName]),
  );
  return new Map(
    tactic.lanes.map((lane) => {
      const ipRole = roleNames.get(lane.ipRoleId) ?? lane.ipRoleId;
      const oopRole = roleNames.get(lane.oopRoleId) ?? lane.oopRoleId;
      // Compact placement primary: one token when both phases share the
      // placement, otherwise the slash-joined pair (e.g. "AML/ML").
      const compact =
        lane.ipPosition === lane.oopPosition
          ? lane.ipPosition
          : `${lane.ipPosition}/${lane.oopPosition}`;
      const metricLabels = lane.customMetricKeys.map(
        (key) => moneyballMetric(key).label,
      );
      const roles = ipRole === oopRole ? ipRole : `${ipRole} / ${oopRole}`;
      const label: TacticLaneLabel = {
        compact,
        // Restrained role context: full role names in smaller inline text.
        context: roles,
        full: `${lane.ipPosition} (${ipRole}) / ${lane.oopPosition} (${oopRole})`,
        // The slot's Moneyball Score: its chosen metrics, or its roles.
        moneyball: {
          context: "Moneyball Score",
          full:
            metricLabels.length > 0
              ? `${compact} Moneyball Score: ${metricLabels.join(", ")}`
              : `${compact} Moneyball Score: ${roles}`,
        },
      };
      return [lane.laneId, label];
    }),
  );
}

/**
 * The lowest rating at a slot's position that lets the slot score a player.
 * Mirrors `suitable_familiarity` in `src-tauri/src/features/planner/fit.rs`.
 */
const SLOT_POSITION_FAMILIARITY = 12;

/**
 * Each tactic slot's recruitment filters as a ready-made filter set, keeping
 * the players its positions let it score and ranking percentiles among its two
 * position families, as the Dashboard does.
 */
function slotFilterPresets(
  tactic: PlannerTactic,
  laneLabels: ReadonlyMap<string, TacticLaneLabel>,
): SlotFilterPreset[] {
  return orderedTacticLanes(tactic.lanes).flatMap((lane) => {
    const recruitmentRules = lane.recruitmentFilters.flatMap(
      (filter) => recruitmentFilterToRule(filter, createFilterRuleId()) ?? [],
    );
    if (recruitmentRules.length === 0) {
      return [];
    }
    const positions = [
      ...new Set([
        basePosition(lane.ipPosition),
        basePosition(lane.oopPosition),
      ]),
    ];
    const rules: FilterRule[] = [
      ...positions.map(
        (position): FilterRule => ({
          id: createFilterRuleId(),
          field: `pos.${position}`,
          op: "gte",
          value: { type: "integer", value: SLOT_POSITION_FAMILIARITY },
        }),
      ),
      ...recruitmentRules,
    ];
    const families = positions.map(positionFamilyOf);
    const comparison = families.includes(null)
      ? "own"
      : parseMoneyballComparison([...new Set(families)].join("+"));
    const label = laneLabels.get(lane.laneId);
    return [
      {
        laneId: lane.laneId,
        label: label?.compact ?? lane.ipPosition,
        description: label?.context ?? "",
        rules,
        comparison,
      },
    ];
  });
}

type SeasonStatsRefreshContext = { saveId: number; contextToken: string };

/** The Refresh stats outcome sentence; a running Load Data is Refresh-only. */
function refreshSeasonStatsCopy(result: RefreshSeasonStatsResult): {
  text: string;
  isProblem: boolean;
} {
  if (result.status === "loadDataRunning") {
    return {
      text: "Load Data is running. Refresh stats after it finishes.",
      isProblem: true,
    };
  }
  return seasonStatsCopy(result);
}

function SearchPageContent() {
  const snapshotQuery = useQuery(currentSnapshotQueryOptions);
  const savesQuery = useQuery(savesQueryOptions);
  const managedClub = useQuery(managedClubQueryOptions).data;
  const snapshot = snapshotQuery.data;
  const activeSave = savesQuery.data?.find((save) => save.isActive);
  const isResultContextChanging =
    useIsMutating({ mutationKey: playerResultContextMutationKey }) > 0 ||
    snapshotQuery.isFetching ||
    savesQuery.isFetching ||
    snapshotQuery.isError ||
    savesQuery.isError;
  const queryClient = useQueryClient();
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const removeColumn = usePlayerTableStore((state) => state.removeColumn);
  const replaceLayout = usePlayerTableStore((state) => state.replaceLayout);
  const {
    sort,
    dir,
    filters: filterUrls,
    combine,
    comparison: routeMoneyballComparison,
    shortlistOnly: routeShortlistOnly,
  } = Route.useSearch();
  const comparison = routeMoneyballComparison ?? "own";
  // Remember the state so the top navigation can return to it.
  useEffect(() => {
    useSearchViewMemory.getState().remember({
      sort,
      dir,
      filters: filterUrls,
      combine,
      comparison: routeMoneyballComparison,
    });
  }, [combine, dir, filterUrls, routeMoneyballComparison, sort]);
  // Only a matching active-save + current-snapshot pair shows results, and
  // its save context scopes the shortlist selection.
  const resultContext =
    snapshot && activeSave && snapshot.saveId === activeSave.id
      ? { snapshot, activeSave }
      : null;
  const activeSaveId = resultContext?.activeSave.id ?? null;
  const activeSaveContextToken = resultContext?.activeSave.contextToken ?? null;
  const shortlists = resultContext?.activeSave.playerShortlists ?? [];
  const storedShortlistId = usePlayerShortlistSelectionStore((state) =>
    activeSaveId === null || activeSaveContextToken === null
      ? undefined
      : state.selections[
          playerShortlistSelectionKey({
            saveId: activeSaveId,
            contextToken: activeSaveContextToken,
          })
        ],
  );
  // A stored shortlist the save no longer lists, such as one emptied in FM
  // before the last read, selects nothing.
  const shortlistId =
    storedShortlistId !== undefined &&
    shortlists.some((shortlist) => shortlist.id === storedShortlistId)
      ? storedShortlistId
      : null;
  const layout = usePlayerTableStore((state) => state.layouts[TABLE_ID]);
  const navigate = Route.useNavigate();
  // A legacy shortlist URL opens Search with no shortlist selected; the URL
  // never carries the selection.
  useEffect(() => {
    if (routeShortlistOnly === undefined) return;
    if (activeSaveId !== null && activeSaveContextToken !== null) {
      usePlayerShortlistSelectionStore
        .getState()
        .clear({ saveId: activeSaveId, contextToken: activeSaveContextToken });
    }
    void navigate({
      search: (previous) => ({ ...previous, shortlistOnly: undefined }),
      replace: true,
    });
  }, [activeSaveId, activeSaveContextToken, navigate, routeShortlistOnly]);
  const clearMissingShortlist = useCallback(
    async (missingShortlistId: number) => {
      if (activeSaveId === null || activeSaveContextToken === null) return;
      // Search checks the shortlist against the backend's current snapshot,
      // which may already belong to another save while this one's views
      // refresh. Only that save's answer may clear its selection; a failed
      // check keeps it.
      const current = await queryClient
        .fetchQuery({ ...currentSnapshotQueryOptions, staleTime: 0 })
        .catch(() => null);
      if (current?.saveId !== activeSaveId) return;
      usePlayerShortlistSelectionStore
        .getState()
        .clear(
          { saveId: activeSaveId, contextToken: activeSaveContextToken },
          missingShortlistId,
        );
    },
    [activeSaveId, activeSaveContextToken, queryClient],
  );
  const filters = useMemo(() => parseSearchFilters(filterUrls), [filterUrls]);
  // Percentiles vs, the score adjustment, and the stats date matter only
  // while a season column, rule, or sort is in use.
  const usesSeasonStats =
    isSeasonFieldId(sort) ||
    layout.columnIds.some(isSeasonFieldId) ||
    filters.some((rule) => isSeasonFieldId(rule.field));
  const isLoadDataRunning =
    useIsMutating({ mutationKey: loadDataMutationKey }) > 0;
  const refreshStats = useMutation({
    mutationFn: ({ saveId, contextToken }: SeasonStatsRefreshContext) =>
      refreshSeasonStats(saveId, contextToken),
    onSuccess: async (result) => {
      if (result.status === "loadDataRunning") return;
      // The read replaces the save's staff shortlist whether or not stats attached.
      const staffShortlistRead =
        result.staffShortlist.status === "read"
          ? [queryClient.invalidateQueries({ queryKey: staffKeys.all })]
          : [];
      if (result.status !== "attached") {
        await Promise.all(staffShortlistRead);
        return;
      }
      await Promise.all([
        ...staffShortlistRead,
        queryClient.invalidateQueries({ queryKey: snapshotKeys.all }),
        queryClient.invalidateQueries({ queryKey: searchKeys.all }),
        queryClient.invalidateQueries({ queryKey: moneyballKeys.all }),
        queryClient.invalidateQueries({ queryKey: playerKeys.all }),
      ]);
    },
  });
  // The outcome belongs to the save context it refreshed; a switched save hides it.
  const refreshOutcome =
    refreshStats.variables?.saveId === activeSaveId &&
    refreshStats.variables.contextToken === activeSaveContextToken
      ? refreshStats.isError
        ? {
            text: "Season stats could not be refreshed. Try again.",
            isProblem: true,
          }
        : refreshStats.data
          ? refreshSeasonStatsCopy(refreshStats.data)
          : null
      : null;

  const updateSearch = (
    patch: Partial<{
      sort: SearchSortField;
      dir: SearchSortDir;
      filters: FilterRule[];
      combine: FilterCombineMode;
      comparison: MoneyballComparison;
      replace: boolean;
    }>,
  ) =>
    navigate({
      search: (previous) => {
        const nextComparison = patch.comparison ?? previous.comparison;
        return {
          sort: patch.sort ?? previous.sort,
          dir: patch.dir ?? previous.dir,
          filters:
            patch.filters !== undefined
              ? searchFiltersForUrl(patch.filters)
              : previous.filters,
          combine: patch.combine ?? previous.combine,
          comparison: nextComparison === "own" ? undefined : nextComparison,
        };
      },
      replace: patch.replace ?? true,
    });

  const selectShortlist = (value: string) => {
    if (activeSaveId === null || activeSaveContextToken === null) return;
    const context = {
      saveId: activeSaveId,
      contextToken: activeSaveContextToken,
    };
    const store = usePlayerShortlistSelectionStore.getState();
    if (value === "") {
      store.clear(context);
    } else {
      store.select(context, Number(value));
    }
  };
  const handleRulesChange = (rules: FilterRule[]) => {
    updateSearch({ filters: rules });
  };
  const handleApplyFilters = (
    rules: FilterRule[],
    nextCombine: FilterCombineMode,
    nextComparison?: MoneyballComparison,
  ) => {
    void updateSearch({
      filters: rules,
      combine: nextCombine,
      ...(nextComparison === undefined ? {} : { comparison: nextComparison }),
    }).then(() => {
      addColumns(
        TABLE_ID,
        rules.map((rule) => rule.field),
        SEARCH_TABLE_GROUPS,
      );
    });
  };
  const shortlistControl =
    resultContext && shortlists.length > 0 ? (
      <label className="flex items-center gap-2 text-body-md text-on-surface">
        Shortlist
        <select
          className="h-8 max-w-64 rounded-md border border-outline bg-surface-container-high px-2 text-body-md text-on-surface"
          value={shortlistId ?? ""}
          onChange={(event) => selectShortlist(event.target.value)}
        >
          <option value="">No shortlist</option>
          {shortlists.map(({ id, name }) => (
            <option key={id} value={id}>
              {name}
            </option>
          ))}
        </select>
      </label>
    ) : null;
  const comparisonControl = (
    <>
      {resultContext ? (
        <MoneyballStatsAsOf
          moneyballStatsGameDate={resultContext.snapshot.moneyballStatsGameDate}
          snapshotGameDate={resultContext.snapshot.gameDate}
        />
      ) : null}
      <label
        className="flex items-center gap-2 text-body-md text-on-surface"
        title="Percentile badges and percentile filters rank each player among this group's peers: players with 450+ minutes rated 16 or higher there. Role and tactic scores always rank within their own positions."
      >
        Percentiles vs
        <select
          className="h-8 max-w-64 rounded-md border border-outline bg-surface-container-high px-2 text-body-md text-on-surface"
          value={comparison}
          onChange={(event) =>
            updateSearch({
              comparison: parseMoneyballComparison(event.target.value),
            })
          }
        >
          <option value="own">Own position</option>
          {POSITION_FAMILIES.map((family) => (
            <option key={family.id} value={family.id}>
              {family.label}
            </option>
          ))}
          {comparison.includes("+") ? (
            <option value={comparison}>
              {moneyballComparisonLabel(comparison)}
            </option>
          ) : null}
        </select>
      </label>
      <MoneyballLeagueToggle
        onBeforeChange={() => clearPlayerResultContext(queryClient)}
        onChanged={() => {
          void queryClient.invalidateQueries({ queryKey: moneyballKeys.all });
          void queryClient.invalidateQueries({ queryKey: dashboardKeys.all });
        }}
      />
    </>
  );
  const datasetToggles = (
    <>
      {shortlistControl}
      <RealisticSigningsToggle
        rules={filters}
        onRulesChange={handleRulesChange}
        // The estimate measures the club's squad by its UID in the snapshot.
        clubName={
          managedClub?.status === "available" && managedClub.clubUid !== null
            ? managedClub.clubName
            : null
        }
      />
      {usesSeasonStats ? comparisonControl : null}
    </>
  );

  const renderSearchBody = (tacticState: TacticContextBoundaryState | null) => {
    const tactic = tacticState?.tactic;
    const options = tacticState?.options;
    const orderedLaneIds = tactic
      ? orderedTacticLanes(tactic.lanes).map((lane) => lane.laneId)
      : [];
    const laneLabels =
      tactic && options ? tacticLaneLabels(tactic, options) : new Map();
    const validationError =
      tactic && options ? validateTacticDraft(tactic, options) : null;
    const viewGroups = tacticColumnGroupsForTable(TABLE_ID);
    const tacticToggles = viewGroups.map((group) => ({
      group,
      label: TACTIC_GROUP_LABELS[group],
      active: isFullTacticGroup(layout.columnIds, group),
    }));
    const slotPresets =
      tactic && validationError === null
        ? slotFilterPresets(tactic, laneLabels)
        : [];
    const savedFilters = (
      <SavedFiltersControl
        rules={filters}
        combine={combine}
        comparison={comparison}
        slotPresets={slotPresets}
        onApply={handleApplyFilters}
      />
    );
    const toggleDisabled =
      isResultContextChanging ||
      tacticState === null ||
      tacticState.isPending ||
      tacticState.initialError !== null ||
      tactic === undefined ||
      options === undefined ||
      validationError !== null ||
      tacticState.readOnly;

    let tacticMessage: string | null = null;
    let tacticMessageIsError = false;
    let retryTactic: (() => void) | null = null;
    if (!activeSave) {
      tacticMessage =
        "No active save: configure a save before adding tactic columns";
    } else if (!resultContext) {
      tacticMessage = "No snapshot loaded: use Load Data";
    } else if (isResultContextChanging || tacticState?.isPending) {
      tacticMessage = "Loading tactic…";
    } else if (tacticState?.initialError) {
      tacticMessage = "Could not load tactic";
      tacticMessageIsError = true;
      retryTactic = tacticState.retryBoth;
    } else if (!tactic || !options) {
      tacticMessage = "Loading tactic…";
    } else if (validationError) {
      tacticMessage = validationError;
      tacticMessageIsError = true;
    } else if (tacticState?.readOnly) {
      tacticMessage = "Could not refresh tactic. Cached labels are read-only.";
      tacticMessageIsError = true;
      retryTactic = tacticState.retryBoth;
    }

    const toggleGroup = (group: TacticColumnGroup) => {
      if (toggleDisabled || !tactic || !options) {
        return;
      }
      if (validateTacticDraft(tactic, options) !== null) {
        return;
      }
      const columnIds =
        usePlayerTableStore.getState().layouts[TABLE_ID].columnIds;
      const groupActive = isFullTacticGroup(columnIds, group);
      const nextActiveGroups = viewGroups.filter((viewGroup) =>
        viewGroup === group
          ? !groupActive
          : isFullTacticGroup(columnIds, viewGroup),
      );
      const nextColumnIds = [
        ...columnIds.filter((id) => !isTacticColumnId(id)),
        ...buildTacticColumnOrder(orderedLaneIds, nextActiveGroups),
      ];
      // Identity-only is valid: emptying the last tactic group removes
      // those leaves through removeColumn instead of replaceLayout([]),
      // which restores defaults and must keep that semantic.
      if (nextColumnIds.length === 0) {
        for (const id of columnIds.filter((id) => isTacticColumnId(id))) {
          removeColumn(TABLE_ID, id);
        }
      } else {
        replaceLayout(TABLE_ID, nextColumnIds);
      }
      if (isTacticColumnId(sort) && !nextColumnIds.includes(sort)) {
        updateSearch({
          sort: DEFAULT_SEARCH_SORT_FIELD,
          dir: defaultDirForSortField(DEFAULT_SEARCH_SORT_FIELD),
        });
      }
    };

    // Page-header owns the title and every page action (Add Tactic groups,
    // Refresh stats): the generic table toolbar below owns only dataset slots
    // and never hosts these controls.
    const pageHeader = (
      <header
        data-testid="search-page-header"
        className="flex w-full flex-wrap items-start justify-between gap-2"
      >
        <h1 className="text-headline-md text-on-surface">Player Search</h1>
        <div
          className="flex flex-wrap items-center justify-end gap-2"
          data-testid="search-page-actions"
        >
          <TacticColumnToggles
            groups={tacticToggles}
            disabled={toggleDisabled}
            onToggleGroup={toggleGroup}
          />
          <Button
            variant="secondary"
            icon={RefreshCw}
            disabled={
              !resultContext || refreshStats.isPending || isLoadDataRunning
            }
            onClick={() => {
              if (!resultContext) return;
              refreshStats.mutate({
                saveId: resultContext.activeSave.id,
                contextToken: resultContext.activeSave.contextToken,
              });
            }}
          >
            Refresh stats
          </Button>
          {refreshOutcome ? (
            <p
              role="status"
              className={cn(
                "text-body-sm",
                refreshOutcome.isProblem
                  ? "text-error"
                  : "text-on-surface-variant",
              )}
            >
              {refreshOutcome.text}
            </p>
          ) : null}
        </div>
      </header>
    );

    return (
      <>
        {pageHeader}
        {resultContext ? null : (
          <SearchFilterBar
            rules={filters}
            combine={combine}
            onRulesChange={handleRulesChange}
            onApply={handleApplyFilters}
            comparison={comparison}
            datasetToggles={usesSeasonStats ? comparisonControl : undefined}
          />
        )}
        {tacticMessage ? (
          <div
            className={`flex items-center justify-between gap-3 text-body-sm ${tacticMessageIsError ? "text-error" : "text-on-surface-variant"}`}
            role={tacticMessageIsError ? "alert" : "status"}
          >
            <span>{tacticMessage}</span>
            {retryTactic ? (
              <Button variant="secondary" onClick={retryTactic}>
                Retry
              </Button>
            ) : null}
          </div>
        ) : null}
        <div className="flex min-h-0 flex-1 flex-col">
          {isResultContextChanging ? (
            <Panel flush>
              <p className="p-4 text-body-md text-on-surface-variant">
                Loading player results…
              </p>
            </Panel>
          ) : resultContext ? (
            <SearchResultsPanel
              key={`${resultContext.activeSave.id}:${resultContext.activeSave.contextToken}:${resultContext.snapshot.id}:${resultContext.snapshot.saveId}`}
              sortBy={sort}
              sortDir={dir}
              filters={filters}
              filterCombine={combine}
              comparison={comparison}
              shortlistId={shortlistId}
              onShortlistMissing={clearMissingShortlist}
              pageContext={{
                activeSave: {
                  id: resultContext.activeSave.id,
                  contextToken: resultContext.activeSave.contextToken,
                },
                currentSnapshot: {
                  id: resultContext.snapshot.id,
                  saveId: resultContext.snapshot.saveId,
                },
              }}
              orderedLaneIds={orderedLaneIds}
              laneLabels={laneLabels}
              identityGraphics={(player) => ({
                portrait: (
                  <SearchIdentityGraphics player={player} kind="portrait" />
                ),
                crest: <SearchIdentityGraphics player={player} kind="crest" />,
              })}
              onSortChange={(nextSort, nextDir) => {
                updateSearch({ sort: nextSort, dir: nextDir });
              }}
              onRulesChange={handleRulesChange}
              onApplyFilters={handleApplyFilters}
              filterActions={savedFilters}
              datasetToggles={datasetToggles}
            />
          ) : (
            <Panel flush>
              <EmptyState
                icon={DatabaseZap}
                title="No data loaded for this save"
              >
                No snapshot loaded for the active save. Use Load Data to scan
                Football Manager and ingest players into the database.
              </EmptyState>
            </Panel>
          )}
        </div>
      </>
    );
  };

  return (
    <>
      {resultContext ? (
        <TacticContextBoundary
          context={{
            saveId: resultContext.activeSave.id,
            contextToken: resultContext.activeSave.contextToken,
          }}
        >
          {renderSearchBody}
        </TacticContextBoundary>
      ) : (
        renderSearchBody(null)
      )}
    </>
  );
}

function SearchPage() {
  return (
    <div className="flex h-full min-w-0 flex-col gap-gutter">
      <Suspense fallback={<PanelFallback />}>
        <SearchPageContent />
      </Suspense>
    </div>
  );
}
