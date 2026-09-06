import { useIsMutating, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap, FileUp } from "lucide-react";
import { Suspense, useEffect, useMemo, useState } from "react";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { SquadCsvImportModal } from "@/features/csv-import/components/squad-csv-import-modal";
import type { CsvImportSummary } from "@/features/csv-import/types/csv-import-summary";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
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
import { searchKeys } from "@/features/search/api/search-keys";
import { PlayerShortlistImportModal } from "@/features/search/components/player-shortlist-import-modal";
import { SearchFilterBar } from "@/features/search/components/search-filter-bar";
import type { TacticLaneLabel } from "@/features/search/components/search-results-panel";
import { SearchResultsPanel } from "@/features/search/components/search-results-panel";
import { TacticColumnToggles } from "@/features/search/components/tactic-column-toggles";
import type {
  FilterCombineMode,
  FilterRule,
} from "@/features/search/types/filter-rule";
import type { PlayerShortlistImportSummary } from "@/features/search/types/player-shortlist-import-summary";
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
import { useMoneyballPreferences } from "@/stores/use-moneyball-preferences";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import {
  isFullTacticGroup,
  isTacticColumnId,
  type TacticColumnGroup,
} from "@/utils/tactic-ids";

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
    // Legacy Player Shortlist links resolve to General with filtering on
    // without inspecting persistence.
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
      shortlistOnly:
        legacyShortlist || parseShortlistOnly(search.shortlistOnly)
          ? true
          : undefined,
    };
  },
  beforeLoad: ({ location, search }) => {
    // Legacy Player Shortlist links replace the history entry with the
    // canonical General + shortlistOnly URL instead of only normalizing
    // parsed state. location.search is the raw query; search is validated.
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
      <h1 className="text-headline-lg text-on-surface">Player Search</h1>
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
      const label: TacticLaneLabel = {
        // Compact placement primary: one token when both phases share the
        // placement, otherwise the slash-joined pair (e.g. "AML/ML").
        compact:
          lane.ipPosition === lane.oopPosition
            ? lane.ipPosition
            : `${lane.ipPosition}/${lane.oopPosition}`,
        // Restrained role context: full role names in smaller inline text.
        context: ipRole === oopRole ? ipRole : `${ipRole} / ${oopRole}`,
        full: `${lane.ipPosition} (${ipRole}) / ${lane.oopPosition} (${oopRole})`,
      };
      return [lane.laneId, label];
    }),
  );
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
  const shortlistOnly = routeShortlistOnly === true;
  const tableId = view === "moneyball" ? "moneyball-search" : "search";
  const layout = usePlayerTableStore((state) => state.layouts[tableId]);
  const navigate = Route.useNavigate();
  const filters = useMemo(
    () => parseSearchFilters(filterUrls, view),
    [filterUrls, view],
  );
  const [importOpen, setImportOpen] = useState(false);
  const [lastMoneyballImport, setLastMoneyballImport] =
    useState<CsvImportSummary | null>(null);
  const [shortlistImportOpen, setShortlistImportOpen] = useState(false);
  const [lastShortlistImport, setLastShortlistImport] =
    useState<PlayerShortlistImportSummary | null>(null);
  const resultContext =
    snapshot && activeSave && snapshot.saveId === activeSave.id
      ? { snapshot, activeSave }
      : null;
  // The shortlist import UI and its summary belong to one result context: a
  // null or changed context closes the modal and discards the summary, so
  // opening Upload without a valid snapshot cannot arm a delayed dialog.
  const shortlistUiContextKey = resultContext
    ? `${resultContext.activeSave.id}:${resultContext.activeSave.contextToken}:${resultContext.snapshot.id}:${resultContext.snapshot.contextToken}`
    : null;
  useEffect(() => {
    setLastMoneyballImport(null);
    setLastShortlistImport(null);
    if (shortlistUiContextKey === null) {
      setShortlistImportOpen(false);
    }
  }, [shortlistUiContextKey]);

  const updateSearch = (
    patch: Partial<{
      sort: SearchSortField;
      dir: SearchSortDir;
      filters: FilterRule[];
      combine: FilterCombineMode;
      view: SearchView | undefined;
      comparisonPool: ComparisonPool | undefined;
      shortlistOnly: boolean | undefined;
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
        shortlistOnly:
          "shortlistOnly" in patch
            ? patch.shortlistOnly
            : previous.shortlistOnly,
      }),
      replace: patch.replace ?? true,
    });

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
      );
    });
  };
  const shortlistSwitch = (
    <button
      type="button"
      role="switch"
      aria-checked={shortlistOnly}
      onClick={() => {
        updateSearch({ shortlistOnly: !shortlistOnly ? true : undefined });
      }}
      className="inline-flex items-center gap-2 rounded-full border border-outline px-3 py-1 text-label-md text-on-surface-variant transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
    >
      Shortlist: {shortlistOnly ? "On" : "Off"}
    </button>
  );
  const comparisonPoolToggle = (
    <fieldset className="inline-flex rounded-full border border-outline bg-surface-container-high p-0.5">
      <legend className="sr-only">Comparison pool</legend>
      {(["filtered", "fullCsv"] as const).map((pool) => (
        <button
          key={pool}
          type="button"
          aria-pressed={comparisonPool === pool}
          className={
            comparisonPool === pool
              ? "rounded-full bg-primary px-3 py-1 text-label-md text-on-primary"
              : "rounded-full px-3 py-1 text-label-md text-on-surface-variant hover:text-on-surface"
          }
          onClick={() => updateSearch({ comparisonPool: pool })}
        >
          {pool === "filtered" ? "Filtered cohort" : "Full CSV"}
        </button>
      ))}
    </fieldset>
  );
  const datasetToggles =
    view === "general" ? shortlistSwitch : comparisonPoolToggle;

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
    const currentActive = isFullTacticGroup(layout.columnIds, "current");
    const potentialActive = isFullTacticGroup(layout.columnIds, "potential");
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
        "No active save — configure a save before adding tactic columns";
    } else if (!resultContext) {
      tacticMessage = "No snapshot loaded — use Load Data";
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
      const nextCurrentActive =
        group === "current"
          ? !isFullTacticGroup(columnIds, "current")
          : isFullTacticGroup(columnIds, "current");
      const nextPotentialActive =
        group === "potential"
          ? !isFullTacticGroup(columnIds, "potential")
          : isFullTacticGroup(columnIds, "potential");
      const nextColumnIds = [
        ...columnIds.filter((id) => !isTacticColumnId(id)),
        ...buildTacticColumnOrder(
          orderedLaneIds,
          nextCurrentActive,
          nextPotentialActive,
        ),
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
    // General Upload Shortlist, Moneyball upload): the generic table toolbar
    // below owns only dataset slots and never hosts these controls.
    const pageHeader = (
      <header
        data-testid="search-page-header"
        className="flex w-full flex-wrap items-start justify-between gap-3"
      >
        <h1 className="text-headline-lg text-on-surface">Player Search</h1>
        <div
          className="flex flex-wrap items-center justify-end gap-2"
          data-testid="search-page-actions"
        >
          <TacticColumnToggles
            currentActive={currentActive}
            potentialActive={potentialActive}
            disabled={toggleDisabled}
            onToggleGroup={toggleGroup}
          />
          {view === "general" ? (
            <>
              <Button
                variant="secondary"
                icon={FileUp}
                disabled={!resultContext}
                onClick={() => setShortlistImportOpen(true)}
              >
                Upload Shortlist
              </Button>
              {lastShortlistImport ? (
                <p className="text-body-sm text-on-surface-variant">
                  Last import: {lastShortlistImport.totalPlayers} players,{" "}
                  {lastShortlistImport.storedPlayers} stored,{" "}
                  {lastShortlistImport.skippedPlayers} skipped.
                </p>
              ) : null}
            </>
          ) : null}
          {view === "moneyball" ? (
            <>
              <Button
                variant="secondary"
                icon={FileUp}
                onClick={() => setImportOpen(true)}
              >
                Upload Moneyball CSV
              </Button>
              {lastMoneyballImport ? (
                <p className="text-body-sm text-on-surface-variant">
                  Last import: {lastMoneyballImport.storedPlayers} stored,{" "}
                  {lastMoneyballImport.skippedPlayers} skipped.
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
            <Panel title="Results" flush>
              <p className="p-4 text-body-md text-on-surface-variant">
                Loading player results…
              </p>
            </Panel>
          ) : resultContext ? (
            <SearchResultsPanel
              key={`${resultContext.activeSave.id}:${resultContext.activeSave.contextToken}:${resultContext.snapshot.id}:${resultContext.snapshot.saveId}:${view}:${comparisonPool}:${combine}:${JSON.stringify(filters)}:${shortlistOnly}`}
              sortBy={sort}
              sortDir={dir}
              filters={filters}
              filterCombine={combine}
              view={view}
              comparisonPool={comparisonPool}
              shortlistOnly={shortlistOnly}
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
              onSortChange={(nextSort, nextDir) => {
                updateSearch({ sort: nextSort, dir: nextDir });
              }}
              onRulesChange={handleRulesChange}
              onApplyFilters={handleApplyFilters}
              datasetToggles={datasetToggles}
            />
          ) : (
            <Panel title="Results" flush>
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
      {resultContext ? (
        <SquadCsvImportModal
          activeSaveId={resultContext.snapshot.saveId}
          snapshotId={resultContext.snapshot.id}
          format="moneyball"
          open={importOpen}
          onClose={() => setImportOpen(false)}
          onYouthImported={() => undefined}
          onMoneyballImported={(summary) => {
            setLastMoneyballImport(summary);
            // Moneyball imports never change player shortlist membership,
            // so shortlist-filtered General queries keep their results.
            // Moneyball pages always refresh, even under retained
            // shortlistOnly URL state: Moneyball stays membership-independent.
            void queryClient.invalidateQueries({
              queryKey: searchKeys.playerPages(),
              predicate: (query) => {
                const params = query.queryKey[2] as
                  | { shortlistOnly?: boolean; searchView?: SearchView }
                  | undefined;
                return (
                  params?.shortlistOnly !== true ||
                  params?.searchView === "moneyball"
                );
              },
            });
            void queryClient.invalidateQueries({ queryKey: moneyballKeys.all });
          }}
        />
      ) : null}
      {resultContext ? (
        <PlayerShortlistImportModal
          activeSaveId={resultContext.snapshot.saveId}
          activeSaveContextToken={resultContext.activeSave.contextToken}
          snapshotId={resultContext.snapshot.id}
          snapshotContextToken={resultContext.snapshot.contextToken}
          open={shortlistImportOpen}
          onClose={() => setShortlistImportOpen(false)}
          onImported={async (summary) => {
            setLastShortlistImport(summary);
            setShortlistImportOpen(false);
            await updateSearch({ shortlistOnly: true });
            await queryClient.invalidateQueries({
              queryKey: searchKeys.playerPages(),
            });
          }}
        />
      ) : null}
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
