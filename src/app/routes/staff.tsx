import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Suspense, useMemo } from "react";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import {
  staffMyStaffQueryOptions,
  staffSearchQueryOptions,
} from "@/features/staff/api/staff-query-options";
import { StaffFilterBar } from "@/features/staff/components/staff-filter-bar";
import { StaffSearchResultsPanel } from "@/features/staff/components/staff-search-results-panel";
import {
  StaffWorkspaceTabs,
  staffWorkspacePanelProps,
} from "@/features/staff/components/staff-workspace-tabs";
import type { StaffFilterRule } from "@/features/staff/types/staff-filter-rule";
import type {
  StaffSortDir,
  StaffSortField,
} from "@/features/staff/types/staff-sort";
import {
  DEFAULT_STAFF_SORT_FIELD,
  defaultDirForStaffSortField,
  isStaffSortDir,
  isStaffSortField,
} from "@/features/staff/types/staff-sort";
import {
  parseStaffCombine,
  parseStaffFilters,
  parseStaffView,
  staffFiltersForUrl,
} from "@/features/staff/utils/staff-url-search";
import { usePlayerTableStore } from "@/stores/use-player-table-store";

export type StaffSearch = {
  view: "search" | "my-staff";
  sort: StaffSortField;
  dir: StaffSortDir;
  searchSort: StaffSortField;
  searchDir: StaffSortDir;
  myStaffSort: StaffSortField;
  myStaffDir: StaffSortDir;
  filters: ReturnType<typeof staffFiltersForUrl>;
  combine: "and" | "or";
};

function normalizedStaffSort(
  rawSort: unknown,
  rawDir: unknown,
  fallbackSort: StaffSortField,
): { sort: StaffSortField; dir: StaffSortDir } {
  const sort = isStaffSortField(rawSort) ? rawSort : fallbackSort;
  return {
    sort,
    dir: isStaffSortDir(rawDir) ? rawDir : defaultDirForStaffSortField(sort),
  };
}

export const Route = createFileRoute("/staff")({
  validateSearch: (search: Record<string, unknown>): StaffSearch => {
    const view = parseStaffView(search.view);
    const legacy = normalizedStaffSort(
      search.sort,
      search.dir,
      DEFAULT_STAFF_SORT_FIELD,
    );
    const searchState = normalizedStaffSort(
      search.searchSort ?? (view === "search" ? legacy.sort : undefined),
      search.searchDir ?? (view === "search" ? legacy.dir : undefined),
      DEFAULT_STAFF_SORT_FIELD,
    );
    const myStaffState = normalizedStaffSort(
      search.myStaffSort ?? (view === "my-staff" ? legacy.sort : undefined),
      search.myStaffDir ?? (view === "my-staff" ? legacy.dir : undefined),
      DEFAULT_STAFF_SORT_FIELD,
    );
    const activeState = view === "search" ? searchState : myStaffState;
    const filters = parseStaffFilters(search.filters);
    return {
      view,
      sort: activeState.sort,
      dir: activeState.dir,
      searchSort: searchState.sort,
      searchDir: searchState.dir,
      myStaffSort: myStaffState.sort,
      myStaffDir: myStaffState.dir,
      filters: staffFiltersForUrl(filters),
      combine: parseStaffCombine(search.combine),
    };
  },
  loaderDeps: ({ search: { view, sort, dir, filters, combine } }) => ({
    view,
    sort,
    dir,
    filters,
    combine,
  }),
  loader: ({
    context: { queryClient },
    deps: { view, sort, dir, filters, combine },
  }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(
        view === "my-staff"
          ? staffMyStaffQueryOptions(0, undefined, sort, dir, [])
          : staffSearchQueryOptions(
              0,
              undefined,
              sort,
              dir,
              parseStaffFilters(filters),
              combine,
              [],
            ),
      ),
    ]),
  component: StaffPage,
});

function StaffFallback() {
  return (
    <div
      className="flex min-h-40 flex-1 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant"
      aria-busy="true"
    >
      Loading staff…
    </div>
  );
}

function StaffPageContent() {
  const { view, sort, dir, filters: filterUrls, combine } = Route.useSearch();
  const navigate = Route.useNavigate();
  const queryClient = useQueryClient();
  const filters = useMemo(() => parseStaffFilters(filterUrls), [filterUrls]);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const onBoostSuccess = () =>
    queryClient.invalidateQueries({ queryKey: snapshotKeys.all });

  const updateSearch = (
    patch: Partial<{
      view: "search" | "my-staff";
      sort: StaffSortField;
      dir: StaffSortDir;
      filters: StaffFilterRule[];
      combine: "and" | "or";
    }>,
  ) =>
    navigate({
      search: (previous) => {
        const nextView = patch.view ?? previous.view;
        const nextSearchSort =
          nextView === "search" && patch.sort !== undefined
            ? patch.sort
            : previous.searchSort;
        const nextSearchDir =
          nextView === "search" && patch.dir !== undefined
            ? patch.dir
            : previous.searchDir;
        const nextMyStaffSort =
          nextView === "my-staff" && patch.sort !== undefined
            ? patch.sort
            : previous.myStaffSort;
        const nextMyStaffDir =
          nextView === "my-staff" && patch.dir !== undefined
            ? patch.dir
            : previous.myStaffDir;
        const nextActiveSort =
          nextView === "search" ? nextSearchSort : nextMyStaffSort;
        const nextActiveDir =
          nextView === "search" ? nextSearchDir : nextMyStaffDir;
        return {
          ...previous,
          view: nextView,
          sort: nextActiveSort,
          dir: nextActiveDir,
          searchSort: nextSearchSort,
          searchDir: nextSearchDir,
          myStaffSort: nextMyStaffSort,
          myStaffDir: nextMyStaffDir,
          filters:
            patch.filters !== undefined
              ? staffFiltersForUrl(patch.filters)
              : previous.filters,
          combine: patch.combine ?? previous.combine,
        };
      },
      replace: true,
    });

  return (
    <>
      <header className="flex flex-col items-start gap-2">
        <h1 className="text-headline-lg text-on-surface">Staff</h1>
        <StaffWorkspaceTabs
          view={view}
          onViewChange={(nextView) => updateSearch({ view: nextView })}
        />
      </header>
      <div {...staffWorkspacePanelProps("search", view)}>
        {view === "search" ? (
          <div className="flex min-h-0 flex-1 flex-col gap-gutter">
            <StaffFilterBar
              rules={filters}
              combine={combine}
              onRulesChange={(rules) => updateSearch({ filters: rules })}
              onApply={(rules, nextCombine) => {
                void updateSearch({
                  filters: rules,
                  combine: nextCombine,
                }).then(() =>
                  addColumns(
                    "staff-search",
                    rules.map((rule) => rule.field),
                  ),
                );
              }}
            />
            <div className="flex min-h-0 flex-1 flex-col">
              <Suspense fallback={<StaffFallback />}>
                <StaffSearchResultsPanel
                  sortBy={sort}
                  sortDir={dir}
                  filters={filters}
                  filterCombine={combine}
                  onSortChange={(nextSort, nextDir) =>
                    updateSearch({ sort: nextSort, dir: nextDir })
                  }
                  onBoostSuccess={onBoostSuccess}
                />
              </Suspense>
            </div>
          </div>
        ) : null}
      </div>
      <div {...staffWorkspacePanelProps("my-staff", view)}>
        {view === "my-staff" ? (
          <div className="flex min-h-0 flex-1 flex-col">
            <Suspense fallback={<StaffFallback />}>
              <StaffSearchResultsPanel
                scope="my-staff"
                sortBy={sort}
                sortDir={dir}
                filters={[]}
                filterCombine="and"
                onSortChange={(nextSort, nextDir) =>
                  updateSearch({ sort: nextSort, dir: nextDir })
                }
                onBoostSuccess={onBoostSuccess}
              />
            </Suspense>
          </div>
        ) : null}
      </div>
    </>
  );
}

function StaffPage() {
  return (
    <div className="flex h-full min-w-0 flex-col gap-gutter">
      <Suspense fallback={<StaffFallback />}>
        <StaffPageContent />
      </Suspense>
    </div>
  );
}
