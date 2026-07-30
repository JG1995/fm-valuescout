import { useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap } from "lucide-react";
import { Suspense } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { searchPlayersQueryOptions } from "@/features/search/api/search-players-query-options";
import { SearchResultsPanel } from "@/features/search/components/search-results-panel";
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

function SearchPageBody() {
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
    <SearchResultsPanel
      sortBy={sort}
      sortDir={dir}
      onSortChange={(nextSort, nextDir) => {
        void navigate({
          search: { sort: nextSort, dir: nextDir },
          replace: true,
        });
      }}
    />
  );
}

function SearchPage() {
  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-lg text-on-surface">Search</h1>
      <Suspense fallback={<PanelFallback />}>
        <SearchPageBody />
      </Suspense>
    </div>
  );
}
