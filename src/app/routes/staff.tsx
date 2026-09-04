import {
  useIsFetching,
  useIsMutating,
  useQuery,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import {
  createFileRoute,
  Outlet,
  useLocation,
  useRouter,
} from "@tanstack/react-router";
import { Suspense, useMemo, useState } from "react";
import type { MyClubSearch } from "@/app/routes/my-club";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import {
  managedClubOptionsQueryOptions,
  managedClubQueryOptions,
} from "@/features/managed-club/api/managed-club-query-options";
import { plannerDepthQueryOptions } from "@/features/planner/api/planner-depth-query-options";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { staffSearchQueryOptions } from "@/features/staff/api/staff-query-options";
import { StaffAssignmentOptimizer } from "@/features/staff/components/staff-assignment-optimizer";
import { StaffFilterBar } from "@/features/staff/components/staff-filter-bar";
import { StaffSearchResultsPanel } from "@/features/staff/components/staff-search-results-panel";
import type { StaffShortlistImportSummary } from "@/features/staff/components/staff-shortlist-import-modal";
import { StaffShortlistImportModal } from "@/features/staff/components/staff-shortlist-import-modal";
import type { StaffAssignmentContext } from "@/features/staff/types/staff-assignment";
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
  parseShortlistOnly,
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
  shortlistSort: StaffSortField;
  shortlistDir: StaffSortDir;
  shortlistContextSort?: StaffSortField;
  shortlistContextDir?: StaffSortDir;
  shortlistOnly?: boolean;
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
  return {
    view: "staff",
    staffSort: search.myStaffSort,
    staffDir: search.myStaffDir,
  };
}

export const Route = createFileRoute("/staff")({
  validateSearch: (search: Record<string, unknown>): StaffSearch => {
    // Legacy Staff Shortlist links resolve to Staff Search with filtering on
    // without inspecting persistence.
    const legacyShortlist = search.view === "shortlist";
    const parsedView = parseStaffView(search.view);
    const view =
      legacyShortlist || parsedView === "shortlist" ? "search" : parsedView;
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
      search.shortlistSort,
      search.shortlistDir,
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
      shortlistSort: shortlistState.sort,
      shortlistDir: shortlistState.dir,
      shortlistContextSort: shortlistContextState?.sort,
      shortlistContextDir: shortlistContextState?.dir,
      shortlistOnly:
        legacyShortlist || parseShortlistOnly(search.shortlistOnly)
          ? true
          : undefined,
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
    // Legacy Staff Shortlist links replace the history entry with the
    // canonical Staff Search + shortlistOnly URL instead of only normalizing
    // parsed state. location.search is the raw query; search is validated.
    if (
      location.pathname === "/staff" &&
      (location.search as Record<string, unknown> | undefined)?.view ===
        "shortlist"
    ) {
      throw Route.redirect({
        to: "/staff",
        search: {
          ...search,
          view: "search",
          shortlistOnly: true,
        },
        replace: true,
      });
    }
    if (location.pathname === "/staff" && search.view !== "search") {
      throw Route.redirect({
        to: "/my-club",
        search: toMyClubSearch(search),
        replace: true,
      });
    }
  },
  loaderDeps: ({
    search: {
      sort,
      dir,
      filters,
      combine,
      shortlistOnly,
      preferredJob,
      unemployedOnly,
    },
  }) => ({
    sort,
    dir,
    filters,
    combine,
    shortlistOnly,
    preferredJob,
    unemployedOnly,
  }),
  loader: ({
    context: { queryClient },
    deps: {
      sort,
      dir,
      filters,
      combine,
      shortlistOnly,
      preferredJob,
      unemployedOnly,
    },
    location,
  }) => {
    const currentSnapshot = queryClient.ensureQueryData(
      currentSnapshotQueryOptions,
    );
    if (location.pathname !== "/staff") return currentSnapshot;
    return Promise.all([
      currentSnapshot,
      queryClient.prefetchQuery(managedClubQueryOptions),
      queryClient.prefetchQuery(managedClubOptionsQueryOptions),
      queryClient.prefetchQuery(plannerDepthQueryOptions),
      queryClient.ensureQueryData(
        staffSearchQueryOptions(
          0,
          undefined,
          sort,
          dir,
          parseStaffFilters(filters),
          combine,
          [],
          shortlistOnly === true,
          preferredJob,
          unemployedOnly,
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
  const {
    sort,
    dir,
    filters: filterUrls,
    combine,
    shortlistOnly: routeShortlistOnly,
    preferredJob: routePreferredJob,
    unemployedOnly: routeUnemployedOnly,
    shortlistSort,
    shortlistDir,
    shortlistContextSort,
    shortlistContextDir,
  } = Route.useSearch();
  const navigate = Route.useNavigate();
  const router = useRouter();
  const queryClient = useQueryClient();
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const savesQuery = useQuery(savesQueryOptions);
  const activeSave = savesQuery.data?.find((save) => save.isActive);
  const managedClubQuery = useQuery(managedClubQueryOptions);
  const managedClub = managedClubQuery.data;
  const depthQuery = useQuery(plannerDepthQueryOptions);
  const depth = depthQuery.data;
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const filters = useMemo(() => parseStaffFilters(filterUrls), [filterUrls]);
  const shortlistOnly = routeShortlistOnly === true;
  const unemployedOnly = routeUnemployedOnly === true;

  const shortlistPresentation = staffShortlistPresentation(routePreferredJob);
  const shortlistSortIsVisible =
    !shortlistPresentation ||
    shortlistPresentation.columnIds.includes(shortlistSort);
  const shortlistContextSortIsVisible =
    shortlistPresentation &&
    shortlistContextSort &&
    shortlistContextDir &&
    shortlistPresentation.columnIds.includes(shortlistContextSort);
  const effectiveShortlistSort = shortlistPresentation
    ? shortlistContextSortIsVisible &&
      shortlistContextSort &&
      shortlistContextDir
      ? {
          sort: shortlistContextSort,
          dir: shortlistContextDir,
        }
      : shortlistPresentation.sort
        ? {
            sort: shortlistPresentation.sort,
            dir: shortlistPresentation.dir,
          }
        : shortlistSortIsVisible
          ? { sort: shortlistSort, dir: shortlistDir }
          : { sort: "ca", dir: "desc" as const }
    : { sort: shortlistSort, dir: shortlistDir };

  const [importOpen, setImportOpen] = useState(false);
  const [shortlistImport, setShortlistImport] = useState<
    { contextKey: string; summary: StaffShortlistImportSummary } | undefined
  >();
  const [shortlistImportRevision, setShortlistImportRevision] = useState(0);
  const [shortlistImportPending, setShortlistImportPending] = useState(false);
  const shortlistContextKey = `${activeSave?.id ?? "none"}:${activeSave?.contextToken ?? "none"}:${snapshot?.saveId ?? "none"}:${snapshot?.id ?? "none"}:${snapshot?.contextToken ?? "none"}`;
  const staffAssignmentContext: StaffAssignmentContext | null =
    activeSave && snapshot && activeSave.id === snapshot.saveId
      ? {
          saveId: activeSave.id,
          saveContextToken: activeSave.contextToken,
          snapshotId: snapshot.id,
          snapshotContextToken: snapshot.contextToken,
        }
      : null;
  const staffAssignmentContextKey = `${shortlistContextKey}:${managedClub?.clubName ?? "none"}:${(depth?.teams ?? []).map((team) => `${team.team}:${team.displayName}`).join("|")}:${shortlistImportRevision}`;
  const isPlannerRefreshing = useIsFetching({ queryKey: plannerKeys.all }) > 0;
  const isSnapshotRefreshing =
    useIsFetching({ queryKey: snapshotKeys.all }) > 0;
  const isSavesRefreshing = savesQuery.isFetching;
  const isManagedClubRefreshing =
    useIsFetching({ queryKey: managedClubKeys.all }) > 0;
  const isPlayerResultContextMutating =
    useIsMutating({ mutationKey: playerResultContextMutationKey }) > 0;
  const staffAssignmentContextUnavailable =
    isPlannerRefreshing ||
    depthQuery.isPending ||
    depthQuery.isError ||
    isSnapshotRefreshing ||
    isSavesRefreshing ||
    savesQuery.isPending ||
    savesQuery.isError ||
    isManagedClubRefreshing ||
    managedClubQuery.isPending ||
    managedClubQuery.isError ||
    isPlayerResultContextMutating ||
    shortlistImportPending;

  const { data: shortlistOptionsPage } = useSuspenseQuery(
    staffSearchQueryOptions(
      0,
      1,
      "ca",
      "desc",
      [],
      "and",
      [],
      true,
      undefined,
      false,
    ),
  );

  const updateSearch = (
    patch: Partial<{
      sort: StaffSortField;
      dir: StaffSortDir;
      filters: StaffFilterRule[];
      combine: "and" | "or";
      shortlistOnly: boolean | undefined;
      preferredJob: string;
      unemployedOnly: boolean;
      shortlistSort: StaffSortField;
      shortlistDir: StaffSortDir;
      shortlistContextSort: StaffSortField | null;
      shortlistContextDir: StaffSortDir | null;
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
        shortlistOnly:
          "shortlistOnly" in patch
            ? patch.shortlistOnly
            : previous.shortlistOnly,
        preferredJob:
          patch.preferredJob !== undefined
            ? patch.preferredJob || undefined
            : previous.preferredJob,
        unemployedOnly: patch.unemployedOnly ?? previous.unemployedOnly,
        shortlistSort: patch.shortlistSort ?? previous.shortlistSort,
        shortlistDir: patch.shortlistDir ?? previous.shortlistDir,
        shortlistContextSort:
          patch.shortlistContextSort !== undefined
            ? patch.shortlistContextSort || undefined
            : previous.shortlistContextSort,
        shortlistContextDir:
          patch.shortlistContextDir !== undefined
            ? patch.shortlistContextDir || undefined
            : previous.shortlistContextDir,
      }),
      replace: true,
    });

  const onShortlistSortChange = (
    nextSort: StaffSortField,
    nextDir: StaffSortDir,
  ) => {
    void updateSearch(
      shortlistPresentation
        ? {
            shortlistContextSort: nextSort,
            shortlistContextDir: nextDir,
          }
        : { shortlistSort: nextSort, shortlistDir: nextDir },
    );
  };

  const onPreferredJobChange = (nextPreferredJob: string) => {
    const presentation = staffShortlistPresentation(nextPreferredJob);
    void updateSearch({
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
  };

  const onShortlistImported = async (summary: StaffShortlistImportSummary) => {
    await queryClient.invalidateQueries({ queryKey: staffKeys.all });
    await updateSearch({
      preferredJob: "",
      unemployedOnly: false,
      shortlistOnly: true,
    });
    setShortlistImport({ contextKey: shortlistContextKey, summary });
    setShortlistImportRevision((revision) => revision + 1);
  };

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
                shortlistOnly && !shortlistPresentation
                  ? "staff-shortlist"
                  : "staff-search",
                rules.map((rule) => rule.field),
              ),
            );
          }}
          headerActions={
            <>
              <Button onClick={() => setImportOpen(true)}>Upload CSV</Button>
              {staffAssignmentContext ? (
                <StaffAssignmentOptimizer
                  context={staffAssignmentContext}
                  contextKey={staffAssignmentContextKey}
                  contextUnavailable={staffAssignmentContextUnavailable}
                />
              ) : null}
            </>
          }
          shortlistOnly={shortlistOnly}
          preferredJob={routePreferredJob}
          preferredJobOptions={shortlistOptionsPage.preferredJobOptions ?? []}
          unemployedOnly={unemployedOnly}
          onPreferredJobChange={onPreferredJobChange}
          onUnemployedOnlyChange={(value) =>
            void updateSearch({ unemployedOnly: value })
          }
        />
        {shortlistImport?.contextKey === shortlistContextKey ? (
          <p role="status" className="text-body-sm text-on-surface-variant">
            Stored {shortlistImport.summary.storedStaff} of{" "}
            {shortlistImport.summary.totalStaff} staff IDs;{" "}
            {shortlistImport.summary.skippedStaff} skipped.
          </p>
        ) : null}
        <div className="flex min-h-0 flex-1 flex-col">
          <Suspense fallback={<StaffFallback />}>
            <StaffSearchResultsPanel
              activeSnapshotId={snapshot?.id ?? null}
              sortBy={shortlistOnly ? effectiveShortlistSort.sort : sort}
              sortDir={shortlistOnly ? effectiveShortlistSort.dir : dir}
              filters={filters}
              filterCombine={combine}
              preferredJob={routePreferredJob}
              unemployedOnly={unemployedOnly}
              shortlistOnly={shortlistOnly}
              onSortChange={
                shortlistOnly
                  ? onShortlistSortChange
                  : (nextSort, nextDir) =>
                      updateSearch({ sort: nextSort, dir: nextDir })
              }
              onShortlistOnlyChange={(next) =>
                void updateSearch({
                  shortlistOnly: next ? true : undefined,
                })
              }
              onRowActivate={(staff) =>
                router.history.push(`/staff/${staff.uid}`)
              }
            />
          </Suspense>
        </div>
      </div>
      <StaffShortlistImportModal
        activeSaveId={snapshot?.saveId}
        snapshotId={snapshot?.id}
        open={importOpen}
        replacesExisting={shortlistOptionsPage.state !== "no_shortlist"}
        onClose={() => setImportOpen(false)}
        onImported={onShortlistImported}
        onPendingChange={setShortlistImportPending}
        contextKey={shortlistContextKey}
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
