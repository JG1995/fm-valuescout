import { useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap } from "lucide-react";
import { Suspense, useMemo } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
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
  DEFAULT_SEARCH_SORT_FIELD,
  defaultDirForSortField,
  isSearchSortDir,
} from "@/features/search/types/search-sort";
import {
  dynamicColumnFields,
  isVisibleSortField,
} from "@/features/search/utils/dynamic-columns";
import type { FilterRuleUrl } from "@/features/search/utils/search-url-search";
import {
  parseSearchCombine,
  parseSearchFilters,
  searchFiltersForUrl,
} from "@/features/search/utils/search-url-search";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";

export type SearchRouteSearch = {
  sort: SearchSortField;
  dir: SearchSortDir;
  /** Flat URL shape — validateSearch always normalizes to this. */
  filters: FilterRuleUrl[];
  combine: FilterCombineMode;
};

export const Route = createFileRoute("/search")({
  validateSearch: (search: Record<string, unknown>): SearchRouteSearch => {
    const filters = searchFiltersForUrl(parseSearchFilters(search.filters));
    const filterRules = parseSearchFilters(filters);
    const sort = isVisibleSortField(search.sort, filterRules)
      ? search.sort
      : DEFAULT_SEARCH_SORT_FIELD;
    const dir = isSearchSortDir(search.dir)
      ? search.dir
      : isVisibleSortField(search.sort, filterRules)
        ? defaultDirForSortField(sort)
        : DEFAULT_SEARCH_SORT_DIR;
    return {
      sort,
      dir,
      filters,
      combine: parseSearchCombine(search.combine),
    };
  },
  loaderDeps: ({ search: { sort, dir, filters, combine } }) => ({
    sort,
    dir,
    filters,
    combine,
  }),
  loader: ({
    context: { queryClient },
    deps: { sort, dir, filters, combine },
  }) => {
    const rules = parseSearchFilters(filters);
    return Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(
        searchPlayersQueryOptions(
          0,
          undefined,
          sort,
          dir,
          rules,
          combine,
          dynamicColumnFields(rules),
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

function SearchPageContent() {
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const { sort, dir, filters: filterUrls, combine } = Route.useSearch();
  const navigate = Route.useNavigate();
  const filters = useMemo(() => parseSearchFilters(filterUrls), [filterUrls]);

  const updateSearch = (
    patch: Partial<{
      sort: SearchSortField;
      dir: SearchSortDir;
      filters: FilterRule[];
      combine: FilterCombineMode;
    }>,
  ) => {
    void navigate({
      search: (previous) => ({
        sort: patch.sort ?? previous.sort,
        dir: patch.dir ?? previous.dir,
        filters:
          patch.filters !== undefined
            ? searchFiltersForUrl(patch.filters)
            : previous.filters,
        combine: patch.combine ?? previous.combine,
      }),
      replace: true,
    });
  };

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
      <SearchFilterBar
        rules={filters}
        combine={combine}
        onRulesChange={(rules) => {
          updateSearch({ filters: rules });
        }}
        onApply={(rules, nextCombine) => {
          updateSearch({ filters: rules, combine: nextCombine });
        }}
      />
      <div className="flex min-h-0 flex-1 flex-col">
        <Suspense fallback={<PanelFallback />}>
          <SearchResultsPanel
            sortBy={sort}
            sortDir={dir}
            filters={filters}
            filterCombine={combine}
            onSortChange={(nextSort, nextDir) => {
              updateSearch({ sort: nextSort, dir: nextDir });
            }}
          />
        </Suspense>
      </div>
    </>
  );
}

function SearchPage() {
  return (
    <div className="flex h-full min-w-0 flex-col gap-gutter">
      <h1 className="text-headline-lg text-on-surface">Search</h1>
      <Suspense fallback={<PanelFallback />}>
        <SearchPageContent />
      </Suspense>
    </div>
  );
}
