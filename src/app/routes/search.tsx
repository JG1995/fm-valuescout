import { useIsMutating, useQuery, useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap, FileUp } from "lucide-react";
import { Suspense, useEffect, useMemo, useRef, useState } from "react";
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
import { SearchFilterBar } from "@/features/search/components/search-filter-bar";
import { SearchResultsPanel } from "@/features/search/components/search-results-panel";
import { TacticColumnToggles } from "@/features/search/components/tactic-column-toggles";
import type {
  FilterCombineMode,
  FilterRule,
} from "@/features/search/types/filter-rule";
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
};

export const Route = createFileRoute("/search")({
  validateSearch: (search: Record<string, unknown>): SearchRouteSearch => {
    const parsedView = parseSearchView(search.view);
    const explicitView =
      search.view === "general" ||
      search.view === "moneyball" ||
      search.view === "shortlist"
        ? parsedView
        : undefined;
    const view =
      explicitView ?? useMoneyballPreferences.getState().defaultAnalysisView;
    const filters = searchFiltersForUrl(
      parseSearchFilters(search.filters, view),
    );
    const filterRules = parseSearchFilters(filters, view);
    const tableId =
      view === "moneyball"
        ? "moneyball-search"
        : view === "shortlist"
          ? "shortlist"
          : "search";
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
    };
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
    <div className="flex min-h-40 flex-1 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
      Loading search results…
    </div>
  );
}

function tacticLaneLabels(
  tactic: PlannerTactic,
  options: TacticOptions,
): Map<string, string> {
  const roleNames = new Map(
    options.roles.map((role) => [role.roleId, role.displayName]),
  );
  return new Map(
    tactic.lanes.map((lane) => [
      lane.laneId,
      `${lane.ipPosition} (${roleNames.get(lane.ipRoleId) ?? lane.ipRoleId}) / ${lane.oopPosition} (${roleNames.get(lane.oopRoleId) ?? lane.oopRoleId})`,
    ]),
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
  const replaceLayout = usePlayerTableStore((state) => state.replaceLayout);
  const {
    sort,
    dir,
    filters: filterUrls,
    combine,
    view: routeView,
    comparisonPool: routeComparisonPool,
  } = Route.useSearch();
  const defaultAnalysisView = useMoneyballPreferences(
    (state) => state.defaultAnalysisView,
  );
  const view = routeView ?? defaultAnalysisView;
  const comparisonPool = routeComparisonPool ?? "filtered";
  const tableId =
    view === "moneyball"
      ? "moneyball-search"
      : view === "shortlist"
        ? "shortlist"
        : "search";
  const layout = usePlayerTableStore((state) => state.layouts[tableId]);
  const navigate = Route.useNavigate();
  const filters = useMemo(
    () => parseSearchFilters(filterUrls, view),
    [filterUrls, view],
  );
  const [importOpen, setImportOpen] = useState(false);
  const [lastMoneyballImport, setLastMoneyballImport] =
    useState<CsvImportSummary | null>(null);
  const snapshotContext = snapshot ? `${snapshot.saveId}:${snapshot.id}` : null;
  const resultContext =
    snapshot && activeSave && snapshot.saveId === activeSave.id
      ? { snapshot, activeSave }
      : null;
  const tabRefs = useRef<Record<SearchView, HTMLButtonElement | null>>({
    general: null,
    moneyball: null,
    shortlist: null,
  });
  useEffect(() => {
    if (!snapshotContext) return;
    setLastMoneyballImport(null);
  }, [snapshotContext]);

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
      replaceLayout(tableId, nextColumnIds);
      if (isTacticColumnId(sort) && !nextColumnIds.includes(sort)) {
        const nextSort = defaultSearchSort(view);
        updateSearch({
          sort: nextSort,
          dir: defaultDirForSortField(nextSort),
        });
      }
    };

    return (
      <>
        <SearchFilterBar
          rules={filters}
          combine={combine}
          onRulesChange={(rules) => {
            updateSearch({ filters: rules });
          }}
          onApply={(rules, nextCombine) => {
            void updateSearch({ filters: rules, combine: nextCombine }).then(
              () => {
                addColumns(
                  tableId,
                  rules.map((rule) => rule.field),
                );
              },
            );
          }}
          actions={
            <TacticColumnToggles
              currentActive={currentActive}
              potentialActive={potentialActive}
              disabled={toggleDisabled}
              onToggleGroup={toggleGroup}
            />
          }
          view={view}
        />
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
              key={`${resultContext.activeSave.id}:${resultContext.activeSave.contextToken}:${resultContext.snapshot.id}:${resultContext.snapshot.saveId}:${view}:${comparisonPool}:${combine}:${JSON.stringify(filters)}`}
              sortBy={sort}
              sortDir={dir}
              filters={filters}
              filterCombine={combine}
              view={view}
              comparisonPool={comparisonPool}
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
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div
          role="tablist"
          aria-label="Search view"
          className="inline-flex rounded-full bg-surface-container-high p-0.5"
          onKeyDown={(event) => {
            const views: SearchView[] = ["general", "moneyball", "shortlist"];
            const index = views.indexOf(view);
            const nextIndex =
              event.key === "ArrowRight" || event.key === "ArrowDown"
                ? (index + 1) % views.length
                : event.key === "ArrowLeft" || event.key === "ArrowUp"
                  ? (index - 1 + views.length) % views.length
                  : event.key === "Home"
                    ? 0
                    : event.key === "End"
                      ? views.length - 1
                      : -1;
            if (nextIndex < 0) return;
            event.preventDefault();
            const next = views[nextIndex];
            updateSearch({
              view: next,
              comparisonPool: next === "moneyball" ? "filtered" : undefined,
              replace: false,
              sort: defaultSearchSort(next),
              dir: defaultDirForSortField(defaultSearchSort(next)),
              filters: [],
            });
            tabRefs.current[next]?.focus();
          }}
        >
          {(["general", "moneyball", "shortlist"] as const).map((candidate) => (
            <button
              key={candidate}
              type="button"
              role="tab"
              aria-selected={view === candidate}
              tabIndex={view === candidate ? 0 : -1}
              ref={(element) => {
                tabRefs.current[candidate] = element;
              }}
              className={
                view === candidate
                  ? "rounded-full bg-primary px-3 py-1.5 text-label-md text-on-primary"
                  : "rounded-full px-3 py-1.5 text-label-md text-on-surface-variant hover:text-on-surface"
              }
              onClick={() =>
                updateSearch({
                  view: candidate,
                  comparisonPool:
                    candidate === "moneyball" ? "filtered" : undefined,
                  replace: false,
                  sort: defaultSearchSort(candidate),
                  dir: defaultDirForSortField(defaultSearchSort(candidate)),
                  filters: [],
                })
              }
            >
              {candidate === "general"
                ? "General"
                : candidate === "moneyball"
                  ? "Moneyball"
                  : "Shortlist"}
            </button>
          ))}
        </div>
        {view === "moneyball" ? (
          <div className="flex flex-wrap items-center gap-2">
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
          </div>
        ) : null}
      </div>
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
            void queryClient.invalidateQueries({ queryKey: searchKeys.all });
            void queryClient.invalidateQueries({ queryKey: moneyballKeys.all });
          }}
        />
      ) : null}
    </>
  );
}

function SearchPage() {
  return (
    <div className="flex h-full min-w-0 flex-col gap-gutter">
      <h1 className="text-headline-lg text-on-surface">Player Search</h1>
      <Suspense fallback={<PanelFallback />}>
        <SearchPageContent />
      </Suspense>
    </div>
  );
}
