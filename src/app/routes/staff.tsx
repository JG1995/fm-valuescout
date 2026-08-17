import { useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import {
  createFileRoute,
  Outlet,
  useLocation,
  useRouter,
} from "@tanstack/react-router";
import { Suspense, useMemo, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { staffKeys } from "@/features/staff/api/staff-keys";
import {
  staffMyStaffQueryOptions,
  staffSearchQueryOptions,
  staffShortlistQueryOptions,
} from "@/features/staff/api/staff-query-options";
import { StaffFilterBar } from "@/features/staff/components/staff-filter-bar";
import { StaffSearchResultsPanel } from "@/features/staff/components/staff-search-results-panel";
import { StaffShortlistImportModal } from "@/features/staff/components/staff-shortlist-import-modal";
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
  isStaffShortlistSortField,
  isStaffSortDir,
  isStaffSortField,
} from "@/features/staff/types/staff-sort";
import { staffShortlistPresentation } from "@/features/staff/utils/staff-shortlist-presentation";
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
  loaderDeps: ({
    search: { view, sort, dir, filters, combine, preferredJob, unemployedOnly },
  }) => ({
    view,
    sort,
    dir,
    filters,
    combine,
    preferredJob,
    unemployedOnly,
  }),
  loader: ({
    context: { queryClient },
    deps: { view, sort, dir, filters, combine, preferredJob, unemployedOnly },
    location,
  }) => {
    const currentSnapshot = queryClient.ensureQueryData(
      currentSnapshotQueryOptions,
    );
    if (location.pathname !== "/staff") return currentSnapshot;
    return Promise.all([
      currentSnapshot,
      queryClient.ensureQueryData(
        view === "my-staff"
          ? staffMyStaffQueryOptions(0, undefined, sort, dir, [])
          : view === "shortlist"
            ? staffShortlistQueryOptions(
                0,
                undefined,
                sort,
                dir,
                preferredJob,
                unemployedOnly,
                [],
              )
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
      Loading staff…
    </div>
  );
}

function StaffPageContent() {
  const {
    view,
    sort,
    dir,
    filters: filterUrls,
    combine,
    preferredJob,
    unemployedOnly,
    shortlistSort,
    shortlistDir,
    shortlistContextSort,
    shortlistContextDir,
  } = Route.useSearch();
  const navigate = Route.useNavigate();
  const router = useRouter();
  const queryClient = useQueryClient();
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const filters = useMemo(() => parseStaffFilters(filterUrls), [filterUrls]);
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const [importOpen, setImportOpen] = useState(false);
  const shortlistPage = useSuspenseQuery(
    staffShortlistQueryOptions(0, 1, "ca", "desc", undefined, false, []),
  ).data;
  const shortlistPresentation = staffShortlistPresentation(preferredJob);
  const shortlistSortIsVisible =
    !shortlistPresentation ||
    shortlistPresentation.columnIds.includes(shortlistSort);
  const shortlistContextSortIsVisible =
    shortlistPresentation &&
    shortlistContextSort &&
    shortlistContextDir &&
    shortlistPresentation.columnIds.includes(shortlistContextSort);
  const effectiveShortlistSort = shortlistPresentation
    ? shortlistContextSortIsVisible
      ? { sort: shortlistContextSort, dir: shortlistContextDir }
      : shortlistPresentation.sort
        ? { sort: shortlistPresentation.sort, dir: shortlistPresentation.dir }
        : shortlistSortIsVisible
          ? { sort: shortlistSort, dir: shortlistDir }
          : { sort: "ca", dir: "desc" as const }
    : { sort: shortlistSort, dir: shortlistDir };
  const onBoostSuccess = () =>
    queryClient.invalidateQueries({ queryKey: snapshotKeys.all });

  const updateSearch = (
    patch: Partial<{
      view: "search" | "my-staff" | "shortlist";
      sort: StaffSortField;
      dir: StaffSortDir;
      filters: StaffFilterRule[];
      combine: "and" | "or";
      preferredJob?: string;
      unemployedOnly: boolean;
      shortlistContextSort: StaffSortField | null;
      shortlistContextDir: StaffSortDir | null;
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
        const nextShortlistSort =
          nextView === "shortlist" && patch.sort !== undefined
            ? patch.sort
            : previous.shortlistSort;
        const nextShortlistDir =
          nextView === "shortlist" && patch.dir !== undefined
            ? patch.dir
            : previous.shortlistDir;
        const nextActiveSort =
          nextView === "search"
            ? nextSearchSort
            : nextView === "my-staff"
              ? nextMyStaffSort
              : nextShortlistSort;
        const nextActiveDir =
          nextView === "search"
            ? nextSearchDir
            : nextView === "my-staff"
              ? nextMyStaffDir
              : nextShortlistDir;
        return {
          ...previous,
          view: nextView,
          sort: nextActiveSort,
          dir: nextActiveDir,
          searchSort: nextSearchSort,
          searchDir: nextSearchDir,
          myStaffSort: nextMyStaffSort,
          myStaffDir: nextMyStaffDir,
          shortlistSort: nextShortlistSort,
          shortlistDir: nextShortlistDir,
          shortlistContextSort:
            patch.shortlistContextSort !== undefined
              ? patch.shortlistContextSort || undefined
              : previous.shortlistContextSort,
          shortlistContextDir:
            patch.shortlistContextDir !== undefined
              ? patch.shortlistContextDir || undefined
              : previous.shortlistContextDir,
          preferredJob:
            patch.preferredJob !== undefined
              ? patch.preferredJob || undefined
              : previous.preferredJob,
          unemployedOnly: patch.unemployedOnly ?? previous.unemployedOnly,
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
                  activeSnapshotId={snapshot?.id ?? null}
                  sortBy={sort}
                  sortDir={dir}
                  filters={filters}
                  filterCombine={combine}
                  onSortChange={(nextSort, nextDir) =>
                    updateSearch({ sort: nextSort, dir: nextDir })
                  }
                  onBoostSuccess={onBoostSuccess}
                  onRowActivate={(staff) =>
                    router.history.push(`/staff/${staff.uid}`)
                  }
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
                activeSnapshotId={snapshot?.id ?? null}
                scope="my-staff"
                sortBy={sort}
                sortDir={dir}
                filters={[]}
                filterCombine="and"
                onSortChange={(nextSort, nextDir) =>
                  updateSearch({ sort: nextSort, dir: nextDir })
                }
                onBoostSuccess={onBoostSuccess}
                onRowActivate={(staff) =>
                  router.history.push(`/staff/${staff.uid}`)
                }
              />
            </Suspense>
          </div>
        ) : null}
      </div>
      <div {...staffWorkspacePanelProps("shortlist", view)}>
        {view === "shortlist" ? (
          <div className="flex min-h-0 flex-1 flex-col gap-gutter">
            <div className="flex flex-wrap items-center gap-4 rounded-lg border border-outline-variant bg-surface-container px-4 py-3">
              <Button onClick={() => setImportOpen(true)}>Upload CSV</Button>
              <label className="flex items-center gap-2 text-body-md text-on-surface">
                Preferred Job
                <select
                  className="rounded-md border border-outline bg-surface px-2 py-1 text-on-surface"
                  value={preferredJob ?? ""}
                  onChange={(event) => {
                    const nextPreferredJob = event.target.value;
                    const presentation =
                      staffShortlistPresentation(nextPreferredJob);
                    updateSearch({
                      preferredJob: nextPreferredJob,
                      ...(presentation?.sort
                        ? {
                            shortlistContextSort: presentation.sort,
                            shortlistContextDir: presentation.dir,
                          }
                        : {
                            shortlistContextSort: null,
                            shortlistContextDir: null,
                          }),
                    });
                  }}
                >
                  <option value="">All jobs</option>
                  {(shortlistPage.preferredJobOptions ?? []).map((job) => (
                    <option key={job} value={job}>
                      {job}
                    </option>
                  ))}
                </select>
              </label>
              <label className="flex items-center gap-2 text-body-md text-on-surface">
                <input
                  type="checkbox"
                  checked={unemployedOnly}
                  onChange={(event) =>
                    updateSearch({ unemployedOnly: event.target.checked })
                  }
                />
                Only unemployed
              </label>
            </div>
            <div className="flex min-h-0 flex-1 flex-col">
              <Suspense fallback={<StaffFallback />}>
                <StaffSearchResultsPanel
                  activeSnapshotId={snapshot?.id ?? null}
                  scope="shortlist"
                  sortBy={effectiveShortlistSort.sort}
                  sortDir={effectiveShortlistSort.dir}
                  filters={[]}
                  filterCombine="and"
                  preferredJob={preferredJob}
                  unemployedOnly={unemployedOnly}
                  visibleColumnIds={shortlistPresentation?.columnIds}
                  onSortChange={(nextSort, nextDir) =>
                    updateSearch(
                      shortlistPresentation
                        ? {
                            shortlistContextSort: nextSort,
                            shortlistContextDir: nextDir,
                          }
                        : { sort: nextSort, dir: nextDir },
                    )
                  }
                  onRowActivate={(staff) =>
                    router.history.push(`/staff/${staff.uid}`)
                  }
                />
              </Suspense>
            </div>
          </div>
        ) : null}
      </div>
      <StaffShortlistImportModal
        open={importOpen}
        replacesExisting={shortlistPage.state !== "no_shortlist"}
        onClose={() => setImportOpen(false)}
        onImported={async () => {
          await queryClient.invalidateQueries({ queryKey: staffKeys.all });
          await updateSearch({ preferredJob: "", unemployedOnly: false });
        }}
      />
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
