import { useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap } from "lucide-react";
import { Suspense, useState } from "react";
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
  isSearchSortField,
} from "@/features/search/types/search-sort";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";

export type SearchRouteSearch = {
  sort: SearchSortField;
  dir: SearchSortDir;
};

export const Route = createFileRoute("/search")({
  validateSearch: (search: Record<string, unknown>): SearchRouteSearch => {
    const sort = isSearchSortField(search.sort)
      ? search.sort
      : DEFAULT_SEARCH_SORT_FIELD;
    const dir = isSearchSortDir(search.dir)
      ? search.dir
      : isSearchSortField(search.sort)
        ? defaultDirForSortField(sort)
        : DEFAULT_SEARCH_SORT_DIR;
    return { sort, dir };
  },
  loaderDeps: ({ search: { sort, dir } }) => ({ sort, dir }),
  loader: ({ context: { queryClient }, deps: { sort, dir } }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(
        searchPlayersQueryOptions(0, undefined, sort, dir),
      ),
    ]),
  component: SearchPage,
});

function PanelFallback() {
  return (
    <div className="flex min-h-40 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
      Loading search results…
    </div>
  );
}

type SearchFiltersProps = {
  filters: FilterRule[];
  filterCombine: FilterCombineMode;
  onFiltersChange: (rules: FilterRule[]) => void;
  onCombineChange: (combine: FilterCombineMode) => void;
};

function SearchPageContent({
  filters,
  filterCombine,
  onFiltersChange,
  onCombineChange,
}: SearchFiltersProps) {
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const { sort, dir } = Route.useSearch();
  const navigate = Route.useNavigate();

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
        combine={filterCombine}
        onRulesChange={onFiltersChange}
        onCombineChange={onCombineChange}
      />
      <Suspense fallback={<PanelFallback />}>
        <SearchResultsPanel
          sortBy={sort}
          sortDir={dir}
          filters={filters}
          filterCombine={filterCombine}
          onSortChange={(nextSort, nextDir) => {
            void navigate({
              search: { sort: nextSort, dir: nextDir },
              replace: true,
            });
          }}
        />
      </Suspense>
    </>
  );
}

function SearchPage() {
  const [filters, setFilters] = useState<FilterRule[]>([]);
  const [filterCombine, setFilterCombine] = useState<FilterCombineMode>("and");

  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-lg text-on-surface">Search</h1>
      <Suspense fallback={<PanelFallback />}>
        <SearchPageContent
          filters={filters}
          filterCombine={filterCombine}
          onFiltersChange={setFilters}
          onCombineChange={setFilterCombine}
        />
      </Suspense>
    </div>
  );
}
