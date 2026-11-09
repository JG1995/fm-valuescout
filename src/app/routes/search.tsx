import {
  useIsMutating,
  useMutation,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { DatabaseZap, FolderOpen, RefreshCw, X } from "lucide-react";
import { Suspense, useCallback, useEffect, useMemo } from "react";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { GraphicsImage } from "@/features/graphics/components/graphics-image";
import {
  type RefreshSeasonStatsResult,
  refreshSeasonStats,
} from "@/features/memory-read/api/refresh-season-stats";
import { seasonStatsCopy } from "@/features/memory-read/components/load-data-outcome";
import { loadDataMutationKey } from "@/features/memory-read/hooks/use-load-data";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import { MoneyballStatsAsOf } from "@/features/moneyball/components/moneyball-profile-panel";
import type { TacticContextBoundaryState } from "@/features/planner/components/tactic-context-boundary";
import { TacticContextBoundary } from "@/features/planner/components/tactic-context-boundary";
import type {
  PlannerTactic,
  TacticOptions,
} from "@/features/planner/types/tactic";
import {
  orderedTacticLanes,
  validateTacticDraft,
} from "@/features/planner/utils/tactic-editor";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { choosePlayerShortlist } from "@/features/search/api/choose-player-shortlist";
import { removePlayerShortlist } from "@/features/search/api/remove-player-shortlist";
import { searchKeys } from "@/features/search/api/search-keys";
import { SearchFilterBar } from "@/features/search/components/search-filter-bar";
import type { TacticLaneLabel } from "@/features/search/components/search-results-panel";
import { SearchResultsPanel } from "@/features/search/components/search-results-panel";
import { TacticColumnToggles } from "@/features/search/components/tactic-column-toggles";
import type {
  FilterCombineMode,
  FilterRule,
} from "@/features/search/types/filter-rule";
import type { PlayerShortlistErrorKind } from "@/features/search/types/player-shortlist-choice";
import type { PlayerSummary } from "@/features/search/types/player-summary";
import type {
  SearchSortDir,
  SearchSortField,
} from "@/features/search/types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  defaultDirForSortField,
  isSearchSortDir,
} from "@/features/search/types/search-sort";
import {
  type ComparisonPool,
  defaultSearchSort,
  parseComparisonPool,
  parseSearchView,
  type SearchView,
} from "@/features/search/types/search-view";
import { isVisibleSortField } from "@/features/search/utils/dynamic-columns";
import { searchTableGroups } from "@/features/search/utils/search-table-groups";
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
import type { PlayerShortlistSummary } from "@/features/snapshot/types/save";
import { TauriCommandError } from "@/lib/tauri-client";
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import {
  playerShortlistSelectionKey,
  usePlayerShortlistSelectionStore,
} from "@/stores/use-player-shortlist-selection-store";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import { useSearchViewMemory } from "@/stores/use-search-view-memory";
import { cn } from "@/utils/cn";
import { moneyballMetric } from "@/utils/moneyball-metrics";
import {
  isFullTacticGroup,
  isTacticColumnId,
  type TacticColumnGroup,
  tacticColumnGroupsForTable,
} from "@/utils/tactic-ids";

const TACTIC_GROUP_LABELS: Record<
  SearchView,
  Record<TacticColumnGroup, string>
> = {
  general: { current: "Current", potential: "Potential", custom: "Custom" },
  moneyball: { current: "Roles", potential: "Potential", custom: "Custom" },
};

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
  view?: SearchView;
  comparisonPool?: ComparisonPool;
  shortlistOnly?: boolean;
};

export const Route = createFileRoute("/search")({
  validateSearch: (search: Record<string, unknown>): SearchRouteSearch => {
    // Legacy Player Shortlist links resolve to General and keep
    // `shortlistOnly` as a marker; the page then clears the shortlist
    // selection and removes the marker.
    const legacyShortlist = search.view === "shortlist";
    const parsedView = parseSearchView(search.view);
    const explicitView = legacyShortlist
      ? ("general" as const)
      : search.view === "general" || search.view === "moneyball"
        ? parsedView
        : undefined;
    const view =
      explicitView ?? useMoneyballPreferences.getState().defaultAnalysisView;
    const filters = searchFiltersForUrl(
      parseSearchFilters(search.filters, view),
    );
    const filterRules = parseSearchFilters(filters, view);
    const tableId = view === "moneyball" ? "moneyball-search" : "search";
    const visibleColumnIds =
      usePlayerTableStore.getState().layouts[tableId].columnIds;
    const visibleSort = isVisibleSortField(
      search.sort,
      filterRules,
      view,
      visibleColumnIds,
    )
      ? search.sort
      : null;
    const sort = visibleSort ?? defaultSearchSort(view);
    const dir = isSearchSortDir(search.dir)
      ? search.dir
      : visibleSort !== null
        ? defaultDirForSortField(sort)
        : DEFAULT_SEARCH_SORT_DIR;
    return {
      sort,
      dir,
      filters,
      combine: parseSearchCombine(search.combine),
      view: explicitView,
      comparisonPool:
        view === "moneyball"
          ? parseComparisonPool(search.comparisonPool)
          : undefined,
      shortlistOnly: legacyShortlist
        ? true
        : parseShortlistOnly(search.shortlistOnly),
    };
  },
  beforeLoad: ({ location, search }) => {
    // Legacy Player Shortlist links replace the history entry with the
    // General + shortlistOnly marker URL instead of only normalizing parsed
    // state. location.search is the raw query; search is validated.
    if (
      (location.search as Record<string, unknown> | undefined)?.view ===
      "shortlist"
    ) {
      throw Route.redirect({
        to: "/search",
        search: {
          ...search,
          view: "general",
          shortlistOnly: true,
        },
        replace: true,
      });
    }
  },
  loaderDeps: ({
    search: { sort, dir, filters, combine, view, comparisonPool },
  }) => ({
    sort,
    dir,
    filters,
    combine,
    view: view ?? useMoneyballPreferences.getState().defaultAnalysisView,
    comparisonPool: comparisonPool ?? "filtered",
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
      const label: TacticLaneLabel = {
        compact,
        // Restrained role context: full role names in smaller inline text.
        context: ipRole === oopRole ? ipRole : `${ipRole} / ${oopRole}`,
        full: `${lane.ipPosition} (${ipRole}) / ${lane.oopPosition} (${oopRole})`,
        custom: {
          context: "Custom",
          full:
            metricLabels.length > 0
              ? `${compact} custom: ${metricLabels.join(", ")}`
              : `${compact} custom: no metrics chosen`,
        },
      };
      return [lane.laneId, label];
    }),
  );
}

type SeasonStatsRefreshContext = { saveId: number; contextToken: string };

/** The save context and current snapshot an added file's confirmation belongs to. */
type ShortlistChoiceRequest = SeasonStatsRefreshContext & {
  snapshotId: number;
  path: string;
};

type ShortlistRemoval = SeasonStatsRefreshContext & { shortlistId: number };

/** Opens the native picker for one FM shortlist file. Resolves to null when cancelled. */
async function chooseShortlistFile() {
  const path = await openFileDialog({
    multiple: false,
    directory: false,
    filters: [{ name: "FM shortlist", extensions: ["fmf"] }],
  });
  return typeof path === "string" ? path : null;
}

function fileNameOf(path: string) {
  return path.split(/[\\/]/).pop() ?? path;
}

const shortlistChoiceFailureReasons: Record<
  PlayerShortlistErrorKind,
  (fileName: string) => string
> = {
  unreadable: (fileName) => `${fileName} could not be opened.`,
  too_large: (fileName) => `${fileName} is too large to be an FM shortlist.`,
  not_fm_shortlist: (fileName) => `${fileName} is not a readable FM shortlist.`,
  not_player_shortlist: (fileName) => `${fileName} is not a player shortlist.`,
  unsupported_format: (fileName) =>
    `${fileName} uses a shortlist format that is not supported.`,
  no_snapshot: () => "Use Load Data before adding a shortlist.",
  no_matching_players: (fileName) =>
    `${fileName} does not match this save: none of its players are in the current snapshot.`,
  stale_context: () =>
    "The save changed while the shortlist was read. Add the file again.",
  database: () => "The shortlist could not be saved. Try again.",
};

/** The reason an add was rejected; every shortlist is unchanged. */
function shortlistChoiceFailureCopy(error: Error, path: string) {
  const reason =
    error instanceof TauriCommandError
      ? shortlistChoiceFailureReasons[error.kind as PlayerShortlistErrorKind]
      : undefined;
  return (reason ?? shortlistChoiceFailureReasons.database)(fileNameOf(path));
}

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

/**
 * Dropdown labels: the list name, or the file name when the name is empty,
 * with the file name added when another shortlist shares the name.
 */
function shortlistOptionLabels(shortlists: PlayerShortlistSummary[]) {
  return shortlists.map((shortlist) => {
    const shared = shortlists.some(
      (other) => other.id !== shortlist.id && other.name === shortlist.name,
    );
    return {
      id: shortlist.id,
      label:
        shortlist.name === ""
          ? shortlist.fileName
          : shared
            ? `${shortlist.name} (${shortlist.fileName})`
            : shortlist.name,
    };
  });
}

function SearchPageContent() {
  const snapshotQuery = useQuery(currentSnapshotQueryOptions);
  const savesQuery = useQuery(savesQueryOptions);
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
    view: routeView,
    comparisonPool: routeComparisonPool,
    shortlistOnly: routeShortlistOnly,
  } = Route.useSearch();
  const defaultAnalysisView = useMoneyballPreferences(
    (state) => state.defaultAnalysisView,
  );
  const view = routeView ?? defaultAnalysisView;
  const comparisonPool = routeComparisonPool ?? "filtered";
  // Remember this view's state so the top navigation can return to it.
  useEffect(() => {
    useSearchViewMemory.getState().remember(view, {
      sort,
      dir,
      filters: filterUrls,
      combine,
      comparisonPool: routeComparisonPool,
    });
  }, [combine, dir, filterUrls, routeComparisonPool, sort, view]);
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
  // A stored shortlist the save does not list yet, such as one just added
  // while the saves refetch, selects nothing until it is listed.
  const shortlistId =
    storedShortlistId !== undefined &&
    shortlists.some((shortlist) => shortlist.id === storedShortlistId)
      ? storedShortlistId
      : null;
  const tableId = view === "moneyball" ? "moneyball-search" : "search";
  const layout = usePlayerTableStore((state) => state.layouts[tableId]);
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
    (missingShortlistId: number) => {
      if (activeSaveId === null || activeSaveContextToken === null) return;
      usePlayerShortlistSelectionStore
        .getState()
        .clear(
          { saveId: activeSaveId, contextToken: activeSaveContextToken },
          missingShortlistId,
        );
    },
    [activeSaveId, activeSaveContextToken],
  );
  const filters = useMemo(
    () => parseSearchFilters(filterUrls, view),
    [filterUrls, view],
  );
  const isLoadDataRunning =
    useIsMutating({ mutationKey: loadDataMutationKey }) > 0;
  const refreshStats = useMutation({
    mutationFn: ({ saveId, contextToken }: SeasonStatsRefreshContext) =>
      refreshSeasonStats(saveId, contextToken),
    onSuccess: async (result) => {
      if (result.status !== "attached") return;
      await Promise.all([
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
      view: SearchView | undefined;
      comparisonPool: ComparisonPool | undefined;
      replace: boolean;
    }>,
  ) =>
    navigate({
      search: (previous) => ({
        sort: patch.sort ?? previous.sort,
        dir: patch.dir ?? previous.dir,
        filters:
          patch.filters !== undefined
            ? searchFiltersForUrl(patch.filters)
            : previous.filters,
        combine: patch.combine ?? previous.combine,
        view: "view" in patch ? patch.view : previous.view,
        comparisonPool:
          "comparisonPool" in patch
            ? patch.comparisonPool
            : previous.comparisonPool,
      }),
      replace: patch.replace ?? true,
    });

  const refreshShortlistQueries = () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: snapshotKeys.saves() }),
      queryClient.invalidateQueries({ queryKey: searchKeys.playerPages() }),
    ]);
  // Rust revalidates the save context before it writes, so a file picked for a
  // save that changed meanwhile is rejected rather than linked to another save.
  const chooseShortlist = useMutation({
    mutationFn: ({ saveId, contextToken, path }: ShortlistChoiceRequest) =>
      choosePlayerShortlist(saveId, contextToken, path),
    onSuccess: async (choice, { saveId, contextToken }) => {
      usePlayerShortlistSelectionStore
        .getState()
        .select({ saveId, contextToken }, choice.id);
      await refreshShortlistQueries();
    },
  });
  const removeShortlist = useMutation({
    mutationFn: ({ saveId, contextToken, shortlistId }: ShortlistRemoval) =>
      removePlayerShortlist(saveId, contextToken, shortlistId),
    onSuccess: async (_result, { saveId, contextToken, shortlistId }) => {
      usePlayerShortlistSelectionStore
        .getState()
        .clear({ saveId, contextToken }, shortlistId);
      chooseShortlist.reset();
      await refreshShortlistQueries();
    },
  });
  const pickShortlistFile = async () => {
    if (!resultContext) return;
    const { id: saveId, contextToken } = resultContext.activeSave;
    const snapshotId = resultContext.snapshot.id;
    const path = await chooseShortlistFile();
    if (path) {
      chooseShortlist.mutate({ saveId, contextToken, snapshotId, path });
    }
  };
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
    removeShortlist.reset();
  };
  // The outcome belongs to the save context and snapshot it added for; a
  // switched save or a new snapshot hides it.
  const shortlistChoiceOutcome =
    chooseShortlist.variables?.saveId === activeSaveId &&
    chooseShortlist.variables.contextToken === activeSaveContextToken &&
    chooseShortlist.variables.snapshotId === resultContext?.snapshot.id
      ? chooseShortlist.isError
        ? {
            text: shortlistChoiceFailureCopy(
              chooseShortlist.error,
              chooseShortlist.variables.path,
            ),
            isProblem: true,
          }
        : chooseShortlist.data
          ? {
              text: `${chooseShortlist.data.name}: ${chooseShortlist.data.totalPlayers} players, ${chooseShortlist.data.matchedPlayers} in this snapshot.`,
              isProblem: false,
            }
          : null
      : null;
  const removeFailed =
    removeShortlist.isError &&
    removeShortlist.variables.saveId === activeSaveId &&
    removeShortlist.variables.contextToken === activeSaveContextToken;

  const handleRulesChange = (rules: FilterRule[]) => {
    updateSearch({ filters: rules });
  };
  const handleApplyFilters = (
    rules: FilterRule[],
    nextCombine: FilterCombineMode,
  ) => {
    void updateSearch({ filters: rules, combine: nextCombine }).then(() => {
      addColumns(
        tableId,
        rules.map((rule) => rule.field),
        searchTableGroups(view),
      );
    });
  };
  const shortlistControl =
    resultContext && shortlists.length > 0 ? (
      <>
        <label className="flex items-center gap-2 text-body-md text-on-surface">
          Shortlist
          <select
            className="h-8 max-w-64 rounded-md border border-outline bg-surface-container-high px-2 text-body-md text-on-surface"
            value={shortlistId ?? ""}
            onChange={(event) => selectShortlist(event.target.value)}
          >
            <option value="">No shortlist</option>
            {shortlistOptionLabels(shortlists).map(({ id, label }) => (
              <option key={id} value={id}>
                {label}
              </option>
            ))}
          </select>
        </label>
        {shortlistId !== null ? (
          <Button
            variant="ghost"
            size="icon"
            icon={X}
            aria-label="Remove shortlist"
            disabled={removeShortlist.isPending}
            onClick={() =>
              removeShortlist.mutate({
                saveId: resultContext.activeSave.id,
                contextToken: resultContext.activeSave.contextToken,
                shortlistId,
              })
            }
          />
        ) : null}
      </>
    ) : null;
  const comparisonPoolToggle = (
    <>
      {resultContext ? (
        <MoneyballStatsAsOf
          moneyballStatsGameDate={resultContext.snapshot.moneyballStatsGameDate}
          snapshotGameDate={resultContext.snapshot.gameDate}
        />
      ) : null}
      <span
        aria-hidden
        className="text-label-md text-on-surface-variant"
        title="Percentile badges and percentile filters compare each player with this group."
      >
        Percentiles vs
      </span>
      <fieldset className="inline-flex rounded-md border border-outline bg-surface-container-high p-0.5">
        <legend className="sr-only">Percentiles vs</legend>
        {(["filtered", "fullCsv"] as const).map((pool) => (
          <button
            key={pool}
            type="button"
            aria-pressed={comparisonPool === pool}
            className={
              comparisonPool === pool
                ? "h-8 rounded-sm bg-primary px-3 text-label-md text-on-primary"
                : "h-8 rounded-sm px-3 text-label-md text-on-surface-variant hover:text-on-surface"
            }
            onClick={() => updateSearch({ comparisonPool: pool })}
          >
            {pool === "filtered" ? "Filtered cohort" : "All players"}
          </button>
        ))}
      </fieldset>
    </>
  );
  const datasetToggles =
    view === "general" ? (
      shortlistControl
    ) : (
      <>
        {shortlistControl}
        {comparisonPoolToggle}
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
    const viewGroups = tacticColumnGroupsForTable(tableId);
    const hasCustomMetrics =
      tactic?.lanes.some((lane) => lane.customMetricKeys.length > 0) ?? false;
    const customActive = isFullTacticGroup(layout.columnIds, "custom");
    const tacticToggles = viewGroups.map((group) => ({
      group,
      label: TACTIC_GROUP_LABELS[view][group],
      active: isFullTacticGroup(layout.columnIds, group),
      canAdd: group !== "custom" || hasCustomMetrics,
    }));
    const showCustomUnavailable =
      viewGroups.includes("custom") &&
      tactic !== undefined &&
      !hasCustomMetrics &&
      !customActive;
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
        usePlayerTableStore.getState().layouts[tableId].columnIds;
      const groupActive = isFullTacticGroup(columnIds, group);
      if (group === "custom" && !groupActive && !hasCustomMetrics) {
        return;
      }
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
          removeColumn(tableId, id);
        }
      } else {
        replaceLayout(tableId, nextColumnIds);
      }
      if (isTacticColumnId(sort) && !nextColumnIds.includes(sort)) {
        const nextSort = defaultSearchSort(view);
        updateSearch({
          sort: nextSort,
          dir: defaultDirForSortField(nextSort),
        });
      }
    };

    // Page-header owns the title and every page action (Add Tactic groups,
    // Add FM Shortlist, Moneyball Refresh stats): the generic table toolbar
    // below owns only dataset slots and never hosts these controls.
    const pageHeader = (
      <header
        data-testid="search-page-header"
        className="flex w-full flex-wrap items-start justify-between gap-2"
      >
        <h1 className="text-headline-md text-on-surface">
          {view === "moneyball" ? "Moneyball" : "Player Search"}
        </h1>
        <div
          className="flex flex-wrap items-center justify-end gap-2"
          data-testid="search-page-actions"
        >
          <TacticColumnToggles
            groups={tacticToggles}
            disabled={toggleDisabled}
            onToggleGroup={toggleGroup}
          />
          {showCustomUnavailable ? (
            <p className="text-body-sm text-on-surface-variant">
              Choose custom metrics per slot in the{" "}
              <Link
                to="/my-club"
                search={{ view: "tactic" }}
                className="rounded-sm text-on-surface underline decoration-dotted underline-offset-4 hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
              >
                Tactic editor
              </Link>
              .
            </p>
          ) : null}
          <Button
            variant="secondary"
            icon={FolderOpen}
            disabled={!resultContext || chooseShortlist.isPending}
            onClick={() => void pickShortlistFile()}
          >
            Add FM Shortlist
          </Button>
          {removeFailed ? (
            <p role="alert" className="text-body-sm text-error">
              The shortlist could not be removed. Try again.
            </p>
          ) : null}
          {shortlistChoiceOutcome ? (
            <p
              role={shortlistChoiceOutcome.isProblem ? "alert" : "status"}
              className={cn(
                "text-body-sm",
                shortlistChoiceOutcome.isProblem
                  ? "text-error"
                  : "text-on-surface-variant",
              )}
            >
              {shortlistChoiceOutcome.text}
            </p>
          ) : null}
          {view === "moneyball" ? (
            <>
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
            </>
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
            view={view}
            comparisonPool={comparisonPool}
            datasetToggles={
              view === "moneyball" ? comparisonPoolToggle : undefined
            }
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
              key={`${resultContext.activeSave.id}:${resultContext.activeSave.contextToken}:${resultContext.snapshot.id}:${resultContext.snapshot.saveId}:${view}`}
              sortBy={sort}
              sortDir={dir}
              filters={filters}
              filterCombine={combine}
              view={view}
              comparisonPool={comparisonPool}
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
