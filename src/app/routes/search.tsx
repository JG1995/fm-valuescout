import { useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap, FileUp } from "lucide-react";
import { Suspense, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { SquadCsvImportModal } from "@/features/csv-import/components/squad-csv-import-modal";
import type { CsvImportSummary } from "@/features/csv-import/types/csv-import-summary";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { searchPlayersQueryOptions } from "@/features/search/api/search-players-query-options";
import { SearchFilterBar } from "@/features/search/components/search-filter-bar";
import { SearchResultsPanel } from "@/features/search/components/search-results-panel";
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
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { usePlayerTableStore } from "@/stores/use-player-table-store";

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
    const view = parseSearchView(search.view);
    const filters = searchFiltersForUrl(
      parseSearchFilters(search.filters, view),
    );
    const filterRules = parseSearchFilters(filters, view);
    const sort = isVisibleSortField(search.sort, filterRules, view)
      ? search.sort
      : defaultSearchSort(view);
    const dir = isSearchSortDir(search.dir)
      ? search.dir
      : isVisibleSortField(search.sort, filterRules, view)
        ? defaultDirForSortField(sort)
        : DEFAULT_SEARCH_SORT_DIR;
    return {
      sort,
      dir,
      filters,
      combine: parseSearchCombine(search.combine),
      view: view === "moneyball" ? view : undefined,
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
    view: view ?? "general",
    comparisonPool: comparisonPool ?? "filtered",
  }),
  loader: ({
    context: { queryClient },
    deps: { sort, dir, filters, combine, view, comparisonPool },
  }) => {
    return Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(
        searchPlayersQueryOptions(
          0,
          undefined,
          sort,
          dir,
          parseSearchFilters(filters, view),
          combine,
          [],
          view,
          comparisonPool,
        ),
      ),
    ]);
  },
  component: SearchPage,
});

function PanelFallback() {
  return (
    <div className="flex min-h-40 flex-1 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
      Loading search results…
    </div>
  );
}

function MoneyballCohortPresence({
  onChange,
  snapshotContext,
}: {
  onChange: (hasCohort: boolean) => void;
  snapshotContext: string | null;
}) {
  const { data } = useSuspenseQuery(
    searchPlayersQueryOptions(
      0,
      1,
      "moneyball.average_rating",
      "desc",
      [],
      "and",
      [],
      "moneyball",
      "fullCsv",
    ),
  );
  useEffect(() => {
    if (snapshotContext) {
      onChange(data.total > 0);
    }
  }, [data.total, onChange, snapshotContext]);
  return null;
}

function SearchPageContent() {
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const queryClient = useQueryClient();
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const {
    sort,
    dir,
    filters: filterUrls,
    combine,
    view: routeView,
    comparisonPool: routeComparisonPool,
  } = Route.useSearch();
  const view = routeView ?? "general";
  const comparisonPool = routeComparisonPool ?? "filtered";
  const navigate = Route.useNavigate();
  const filters = useMemo(
    () => parseSearchFilters(filterUrls, view),
    [filterUrls, view],
  );
  const [importOpen, setImportOpen] = useState(false);
  const [hasMoneyballCohort, setHasMoneyballCohort] = useState(false);
  const [lastMoneyballImport, setLastMoneyballImport] =
    useState<CsvImportSummary | null>(null);
  const snapshotContext = snapshot ? `${snapshot.saveId}:${snapshot.id}` : null;
  const tabRefs = useRef<Record<SearchView, HTMLButtonElement | null>>({
    general: null,
    moneyball: null,
  });
  useEffect(() => {
    if (!snapshotContext) return;
    setHasMoneyballCohort(false);
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

  if (!snapshot) {
    return (
      <Panel title="Results" flush>
        <EmptyState icon={DatabaseZap} title="No data loaded for this save">
          No snapshot loaded for the active save. Use Load Data to scan Football
          Manager and ingest players into the database.
        </EmptyState>
      </Panel>
    );
  }

  return (
    <>
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div
          role="tablist"
          aria-label="Search view"
          className="inline-flex rounded-full bg-surface-container-high p-0.5"
          onKeyDown={(event) => {
            const views: SearchView[] = ["general", "moneyball"];
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
              view: next === "moneyball" ? next : undefined,
              comparisonPool: next === "moneyball" ? "filtered" : undefined,
              replace: false,
              sort: defaultSearchSort(next),
              dir: defaultDirForSortField(defaultSearchSort(next)),
              filters: [],
            });
            tabRefs.current[next]?.focus();
          }}
        >
          {(["general", "moneyball"] as const).map((candidate) => (
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
                  view: candidate === "moneyball" ? candidate : undefined,
                  comparisonPool:
                    candidate === "moneyball" ? "filtered" : undefined,
                  replace: false,
                  sort: defaultSearchSort(candidate),
                  dir: defaultDirForSortField(defaultSearchSort(candidate)),
                  filters: [],
                })
              }
            >
              {candidate === "general" ? "General" : "Moneyball"}
            </button>
          ))}
        </div>
        {view === "moneyball" ? (
          <div className="flex flex-wrap items-center gap-2">
            <Suspense fallback={null}>
              <MoneyballCohortPresence
                onChange={setHasMoneyballCohort}
                snapshotContext={snapshotContext}
              />
            </Suspense>
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
              {hasMoneyballCohort
                ? "Replace Moneyball CSV"
                : "Upload Moneyball CSV"}
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
      <SearchFilterBar
        rules={filters}
        combine={combine}
        onRulesChange={(rules) => {
          updateSearch({ filters: rules });
        }}
        onApply={(rules, nextCombine) => {
          void updateSearch({ filters: rules, combine: nextCombine }).then(() =>
            addColumns(
              view === "moneyball" ? "moneyball-search" : "search",
              rules.map((rule) => rule.field),
            ),
          );
        }}
        view={view}
      />
      <div className="flex min-h-0 flex-1 flex-col">
        <Suspense fallback={<PanelFallback />}>
          <SearchResultsPanel
            sortBy={sort}
            sortDir={dir}
            filters={filters}
            filterCombine={combine}
            view={view}
            comparisonPool={comparisonPool}
            onSortChange={(nextSort, nextDir) => {
              updateSearch({ sort: nextSort, dir: nextDir });
            }}
          />
        </Suspense>
      </div>
      <SquadCsvImportModal
        activeSaveId={snapshot.saveId}
        snapshotId={snapshot.id}
        format="moneyball"
        open={importOpen}
        onClose={() => setImportOpen(false)}
        onYouthImported={() => undefined}
        onMoneyballImported={(summary) => {
          setLastMoneyballImport(summary);
          setHasMoneyballCohort(summary.storedPlayers > 0);
          void queryClient.invalidateQueries({ queryKey: searchKeys.all });
          void queryClient.invalidateQueries({ queryKey: moneyballKeys.all });
        }}
        replace={hasMoneyballCohort}
      />
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
