import { useSuspenseQuery } from "@tanstack/react-query";
import {
  createFileRoute,
  Outlet,
  useLocation,
  useRouter,
} from "@tanstack/react-router";
import { Suspense, useMemo } from "react";
import type { MyClubSearch } from "@/app/routes/my-club";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { staffSearchQueryOptions } from "@/features/staff/api/staff-query-options";
import { StaffFilterBar } from "@/features/staff/components/staff-filter-bar";
import { StaffSearchResultsPanel } from "@/features/staff/components/staff-search-results-panel";
import type { StaffFilterRule } from "@/features/staff/types/staff-filter-rule";
import type {
  StaffSortDir,
  StaffSortField,
} from "@/features/staff/types/staff-sort";
import {
  DEFAULT_STAFF_SORT_FIELD,
  defaultDirForStaffSortField,
  isStaffShortlistSortField,
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
  view: "search" | "my-staff" | "shortlist";
  sort: StaffSortField;
  dir: StaffSortDir;
  searchSort: StaffSortField;
  searchDir: StaffSortDir;
  myStaffSort: StaffSortField;
  myStaffDir: StaffSortDir;
  shortlistSort: StaffSortField;
  shortlistDir: StaffSortDir;
  shortlistContextSort?: StaffSortField;
  shortlistContextDir?: StaffSortDir;
  preferredJob?: string;
  unemployedOnly: boolean;
  filters: ReturnType<typeof staffFiltersForUrl>;
  combine: "and" | "or";
};

function normalizedStaffSort(
  rawSort: unknown,
  rawDir: unknown,
  fallbackSort: StaffSortField,
  isValidSortField = isStaffSortField,
): { sort: StaffSortField; dir: StaffSortDir } {
  const sort = isValidSortField(rawSort) ? rawSort : fallbackSort;
  return {
    sort,
    dir: isStaffSortDir(rawDir) ? rawDir : defaultDirForStaffSortField(sort),
  };
}

function toMyClubSearch(search: StaffSearch): MyClubSearch {
  if (search.view === "my-staff") {
    return {
      view: "staff",
      staffSort: search.myStaffSort,
      staffDir: search.myStaffDir,
    };
  }

  return {
    view: "staff-shortlist",
    shortlistSort: search.shortlistSort,
    shortlistDir: search.shortlistDir,
    ...(search.shortlistContextSort
      ? { shortlistContextSort: search.shortlistContextSort }
      : {}),
    ...(search.shortlistContextDir
      ? { shortlistContextDir: search.shortlistContextDir }
      : {}),
    ...(search.preferredJob ? { preferredJob: search.preferredJob } : {}),
    ...(search.unemployedOnly ? { unemployedOnly: true } : {}),
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
    const shortlistState = normalizedStaffSort(
      search.shortlistSort ?? (view === "shortlist" ? legacy.sort : undefined),
      search.shortlistDir ?? (view === "shortlist" ? legacy.dir : undefined),
      DEFAULT_STAFF_SORT_FIELD,
      isStaffShortlistSortField,
    );
    const shortlistContextState =
      isStaffShortlistSortField(search.shortlistContextSort) &&
      isStaffSortDir(search.shortlistContextDir)
        ? {
            sort: search.shortlistContextSort,
            dir: search.shortlistContextDir,
          }
        : undefined;
    const activeState =
      view === "search"
        ? searchState
        : view === "my-staff"
          ? myStaffState
          : shortlistState;
    const filters = parseStaffFilters(search.filters);
    return {
      view,
      sort: activeState.sort,
      dir: activeState.dir,
      searchSort: searchState.sort,
      searchDir: searchState.dir,
      myStaffSort: myStaffState.sort,
      myStaffDir: myStaffState.dir,
      shortlistSort: shortlistState.sort,
      shortlistDir: shortlistState.dir,
      shortlistContextSort: shortlistContextState?.sort,
      shortlistContextDir: shortlistContextState?.dir,
      preferredJob:
        typeof search.preferredJob === "string"
          ? search.preferredJob
          : undefined,
      unemployedOnly: search.unemployedOnly === true,
      filters: staffFiltersForUrl(filters),
      combine: parseStaffCombine(search.combine),
    };
  },
  beforeLoad: ({ location, search }) => {
    if (location.pathname === "/staff" && search.view !== "search") {
      throw Route.redirect({
        to: "/my-club",
        search: toMyClubSearch(search),
        replace: true,
      });
    }
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
    location,
  }) => {
    const currentSnapshot = queryClient.ensureQueryData(
      currentSnapshotQueryOptions,
    );
    if (location.pathname !== "/staff") return currentSnapshot;
    return Promise.all([
      currentSnapshot,
      queryClient.ensureQueryData(
        staffSearchQueryOptions(
          0,
          undefined,
          sort,
          dir,
          parseStaffFilters(filters),
          combine,
          [],
        ),
      ),
    ]);
  },
  component: StaffPage,
});

function StaffFallback() {
  return (
    <div
      className="flex min-h-40 flex-1 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant"
      aria-busy="true"
    >
      Loading staff search…
    </div>
  );
}

function StaffPageContent() {
  const { sort, dir, filters: filterUrls, combine } = Route.useSearch();
  const navigate = Route.useNavigate();
  const router = useRouter();
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const filters = useMemo(() => parseStaffFilters(filterUrls), [filterUrls]);

  const updateSearch = (
    patch: Partial<{
      sort: StaffSortField;
      dir: StaffSortDir;
      filters: StaffFilterRule[];
      combine: "and" | "or";
    }>,
  ) =>
    navigate({
      search: (previous) => ({
        ...previous,
        sort: patch.sort ?? previous.sort,
        dir: patch.dir ?? previous.dir,
        searchSort: patch.sort ?? previous.searchSort,
        searchDir: patch.dir ?? previous.searchDir,
        filters:
          patch.filters !== undefined
            ? staffFiltersForUrl(patch.filters)
            : previous.filters,
        combine: patch.combine ?? previous.combine,
      }),
      replace: true,
    });

  return (
    <>
      <header className="flex flex-col items-start gap-2">
        <h1 className="text-headline-lg text-on-surface">Staff Search</h1>
      </header>
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
              activeSnapshotId={snapshot?.id ?? null}
              sortBy={sort}
              sortDir={dir}
              filters={filters}
              filterCombine={combine}
              onSortChange={(nextSort, nextDir) =>
                updateSearch({ sort: nextSort, dir: nextDir })
              }
              onRowActivate={(staff) =>
                router.history.push(`/staff/${staff.uid}`)
              }
            />
          </Suspense>
        </div>
      </div>
    </>
  );
}

function StaffPage() {
  const location = useLocation();
  if (location.pathname.startsWith("/staff/")) {
    return <Outlet />;
  }
  return (
    <div className="flex h-full min-w-0 flex-col gap-gutter">
      <Suspense fallback={<StaffFallback />}>
        <StaffPageContent />
      </Suspense>
    </div>
  );
}
