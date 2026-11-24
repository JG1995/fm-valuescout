import {
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
import { Suspense, useEffect, useMemo } from "react";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { TableSkeleton } from "@/components/ui/skeleton/skeleton";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import { managedClubQueryOptions } from "@/features/managed-club/api/managed-club-query-options";
import type { ManagedClubStatus } from "@/features/managed-club/types/managed-club";
import { plannerDepthQueryOptions } from "@/features/planner/api/planner-depth-query-options";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import type { PlannerDepth } from "@/features/planner/types/depth";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import type { SaveSummary } from "@/features/snapshot/types/save";
import type { SnapshotSummary } from "@/features/snapshot/types/snapshot";
import { staffAssignmentTargetsQueryOptions } from "@/features/staff/api/staff-assignment-targets-query-options";
import { staffSearchQueryOptions } from "@/features/staff/api/staff-query-options";
import { StaffAssignmentOptimizer } from "@/features/staff/components/staff-assignment-optimizer";
import {
  type StaffResultsQueryParams,
  StaffSearchResultsPanel,
  staffResultsFirstPageOptions,
} from "@/features/staff/components/staff-search-results-panel";
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
import type { StaffPage as StaffResultsPage } from "@/features/staff/types/staff-summary";
import { staffShortlistPresentation } from "@/features/staff/utils/staff-shortlist-presentation";
import {
  parseShortlistOnly,
  parseStaffCombine,
  parseStaffFilters,
  parseStaffView,
  staffFiltersForUrl,
} from "@/features/staff/utils/staff-url-search";
import { useIsRefetchingInvalidated } from "@/lib/use-is-refetching-invalidated";
import { warmQuery } from "@/lib/warm-query";
import { usePlayerTableStore } from "@/stores/use-player-table-store";
import {
  shortlistFilterKey,
  useShortlistFilterStore,
} from "@/stores/use-shortlist-filter-store";

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

type StaffPanelParams = Omit<StaffResultsQueryParams, "requestedFields">;

function staffAssignmentContextFor(
  activeSave: SaveSummary | undefined,
  snapshot: SnapshotSummary | null | undefined,
): StaffAssignmentContext | null {
  return activeSave && snapshot && activeSave.id === snapshot.saveId
    ? {
        saveId: activeSave.id,
        saveContextToken: activeSave.contextToken,
        snapshotId: snapshot.id,
        snapshotContextToken: snapshot.contextToken,
      }
    : null;
}

/** Whether the active save stores a staff shortlist, keyed by its context. */
function staffShortlistProbeOptions(context: StaffAssignmentContext | null) {
  return staffSearchQueryOptions(
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
    context,
  );
}

/**
 * A URL choice wins; otherwise the per-save choice, which defaults to whether
 * the save stores a shortlist.
 */
function resolveStaffShortlistOnly(
  routeShortlistOnly: boolean | undefined,
  storedChoice: boolean | undefined,
  context: StaffAssignmentContext | null,
  shortlistPresent: boolean,
) {
  return (
    (routeShortlistOnly ??
      storedChoice ??
      (context !== null && shortlistPresent)) === true
  );
}

function storedStaffShortlistChoice(context: StaffAssignmentContext | null) {
  return context === null
    ? undefined
    : useShortlistFilterStore.getState().choices[
        shortlistFilterKey("staff", {
          saveId: context.saveId,
          contextToken: context.saveContextToken,
        })
      ];
}

// Load Data can re-read the shortlist without replacing the snapshot, so an
// assignment result is also bound to who is shortlisted.
function staffOptimizerContextKey(
  activeSave: SaveSummary | undefined,
  snapshot: SnapshotSummary | null | undefined,
  managedClub: ManagedClubStatus | undefined,
  depth: PlannerDepth | undefined,
  shortlistProbe: StaffResultsPage,
) {
  const shortlistContextKey = `${activeSave?.id ?? "none"}:${activeSave?.contextToken ?? "none"}:${snapshot?.saveId ?? "none"}:${snapshot?.id ?? "none"}:${snapshot?.contextToken ?? "none"}`;
  const staffAssignmentContextKey = `${shortlistContextKey}:${managedClub?.clubName ?? "none"}:${(depth?.teams ?? []).map((team) => `${team.team}:${team.displayName}`).join("|")}`;
  return `${staffAssignmentContextKey}:${shortlistProbe.total}:${(shortlistProbe.preferredJobOptions ?? []).join("|")}`;
}

function effectiveShortlistSortFor(search: StaffSearch): {
  sort: StaffSortField;
  dir: StaffSortDir;
} {
  const {
    preferredJob,
    shortlistSort,
    shortlistDir,
    shortlistContextSort,
    shortlistContextDir,
  } = search;
  const shortlistPresentation = staffShortlistPresentation(preferredJob);
  if (!shortlistPresentation) return { sort: shortlistSort, dir: shortlistDir };
  if (
    shortlistContextSort &&
    shortlistContextDir &&
    shortlistPresentation.columnIds.includes(shortlistContextSort)
  ) {
    return { sort: shortlistContextSort, dir: shortlistContextDir };
  }
  if (shortlistPresentation.sort) {
    return {
      sort: shortlistPresentation.sort,
      dir: shortlistPresentation.dir,
    };
  }
  return shortlistPresentation.columnIds.includes(shortlistSort)
    ? { sort: shortlistSort, dir: shortlistDir }
    : { sort: "ca", dir: "desc" };
}

function staffSearchPanelParams(
  search: StaffSearch,
  shortlistOnly: boolean,
): StaffPanelParams {
  const shortlistSort = effectiveShortlistSortFor(search);
  return {
    sortBy: shortlistOnly ? shortlistSort.sort : search.sort,
    sortDir: shortlistOnly ? shortlistSort.dir : search.dir,
    filters: parseStaffFilters(search.filters),
    filterCombine: search.combine,
    shortlistOnly,
    preferredJob: search.preferredJob,
    unemployedOnly: search.unemployedOnly === true,
  };
}

function myStaffPanelParams(search: StaffSearch): StaffPanelParams {
  return {
    sortBy: search.myStaffSort,
    sortDir: search.myStaffDir,
    filters: [],
    filterCombine: "and",
    shortlistOnly: false,
    unemployedOnly: false,
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
      search.shortlistSort ?? (legacyShortlist ? legacy.sort : undefined),
      search.shortlistDir ?? (legacyShortlist ? legacy.dir : undefined),
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
      shortlistOnly: legacyShortlist
        ? true
        : parseShortlistOnly(search.shortlistOnly),
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
      return;
    }
  },
  // Every search key, so the loader warms the page the panel will request.
  loaderDeps: ({ search }) => search,
  loader: async ({
    context: { queryClient },
    deps: search,
    location,
    cause,
  }) => {
    const currentSnapshot = queryClient.ensureQueryData(
      currentSnapshotQueryOptions,
    );
    if (location.pathname !== "/staff") return currentSnapshot;
    // The loader waits only for fast reads the first paint needs and starts
    // the table's first page without waiting. A sort or filter change on the
    // page ("stay") leaves that page to the panel, whose named request can
    // supersede an older one.
    const warmFirstPage = cause !== "stay";
    if (search.view === "my-staff") {
      if (warmFirstPage) {
        warmQuery(
          queryClient,
          staffResultsFirstPageOptions("my-staff", myStaffPanelParams(search)),
        );
      }
      await Promise.all([
        currentSnapshot,
        queryClient.ensureQueryData(managedClubQueryOptions),
      ]);
      return;
    }
    // The page reports a failed save, Managed Club, or Planner read itself.
    const [snapshot, saves, managedClub, depth] = await Promise.all([
      currentSnapshot,
      queryClient.ensureQueryData(savesQueryOptions).catch(() => undefined),
      queryClient
        .ensureQueryData(managedClubQueryOptions)
        .catch(() => undefined),
      queryClient
        .ensureQueryData(plannerDepthQueryOptions)
        .catch(() => undefined),
    ]);
    const activeSave = saves?.find((save) => save.isActive);
    const context = staffAssignmentContextFor(activeSave, snapshot);
    const shortlistProbe = await queryClient.ensureQueryData(
      staffShortlistProbeOptions(context),
    );
    if (warmFirstPage) {
      const shortlistOnly = resolveStaffShortlistOnly(
        search.shortlistOnly,
        storedStaffShortlistChoice(context),
        context,
        shortlistProbe.state === "ready",
      );
      warmQuery(
        queryClient,
        staffResultsFirstPageOptions(
          "search",
          staffSearchPanelParams(search, shortlistOnly),
        ),
      );
    }
    if (context) {
      await queryClient
        .ensureQueryData(
          staffAssignmentTargetsQueryOptions(
            context,
            staffOptimizerContextKey(
              activeSave,
              snapshot,
              managedClub,
              depth,
              shortlistProbe,
            ),
          ),
        )
        .catch(() => undefined);
    }
  },
  component: StaffPage,
});

function StaffFallback() {
  return (
    <div className="flex min-h-40 flex-1 flex-col overflow-hidden rounded-lg border border-outline-variant bg-surface-container">
      <TableSkeleton label="Loading staff search…" />
    </div>
  );
}

function MyStaffContent() {
  const { myStaffSort, myStaffDir } = Route.useSearch();
  const navigate = Route.useNavigate();
  const router = useRouter();
  const queryClient = useQueryClient();
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const managedClubQuery = useQuery(managedClubQueryOptions);
  const managedClub = managedClubQuery.data;

  const onMyStaffSortChange = (
    nextSort: StaffSortField,
    nextDir: StaffSortDir,
  ) => {
    void navigate({
      search: (previous) => ({
        ...previous,
        sort: nextSort,
        dir: nextDir,
        myStaffSort: nextSort,
        myStaffDir: nextDir,
      }),
      replace: true,
    });
  };

  const onMyStaffBoostSuccess = () =>
    queryClient.invalidateQueries({ queryKey: snapshotKeys.all });

  return (
    <>
      <header className="flex flex-col items-start gap-2">
        <h1 className="text-headline-md text-on-surface">My Staff</h1>
        {managedClub?.clubName ? (
          <p className="text-body-sm text-on-surface-variant">
            Managed club: {managedClub.clubName}
          </p>
        ) : null}
      </header>
      <div className="flex min-h-0 flex-1 flex-col">
        <Suspense fallback={<StaffFallback />}>
          <StaffSearchResultsPanel
            activeSnapshotId={snapshot?.id ?? null}
            scope="my-staff"
            sortBy={myStaffSort}
            sortDir={myStaffDir}
            filters={[]}
            filterCombine="and"
            onSortChange={onMyStaffSortChange}
            onBoostSuccess={onMyStaffBoostSuccess}
            onRowActivate={(staff) =>
              router.history.push(`/staff/${staff.uid}`)
            }
          />
        </Suspense>
      </div>
    </>
  );
}

function StaffSearchContent() {
  const search = Route.useSearch();
  const { shortlistOnly: routeShortlistOnly, preferredJob: routePreferredJob } =
    search;
  const navigate = Route.useNavigate();
  const router = useRouter();
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const savesQuery = useQuery(savesQueryOptions);
  const activeSave = savesQuery.data?.find((save) => save.isActive);
  const managedClubQuery = useQuery(managedClubQueryOptions);
  const managedClub = managedClubQuery.data;
  const depthQuery = useQuery(plannerDepthQueryOptions);
  const depth = depthQuery.data;
  const addColumns = usePlayerTableStore((state) => state.addColumns);

  const shortlistPresentation = staffShortlistPresentation(routePreferredJob);
  const staffAssignmentContext = staffAssignmentContextFor(
    activeSave,
    snapshot,
  );
  const isPlannerRefreshing = useIsRefetchingInvalidated(plannerKeys.all);
  const isSnapshotRefreshing = useIsRefetchingInvalidated(snapshotKeys.all);
  const isSavesRefreshing = useIsRefetchingInvalidated(snapshotKeys.saves());
  const isManagedClubRefreshing = useIsRefetchingInvalidated(
    managedClubKeys.all,
  );
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
    isPlayerResultContextMutating;

  const { data: shortlistOptionsPage } = useSuspenseQuery(
    staffShortlistProbeOptions(staffAssignmentContext),
  );
  // The shortlist probe reports `state` per active save: `ready` only when
  // the save stores shortlist entries (stored-but-zero-current-match stays
  // `ready`); the per-save Zustand choice derives its default from it once
  // and later staff refreshes never overwrite a manual toggle. Keying the
  // probe by the mounted save/snapshot context keeps a switched save from
  // consuming a previous save's cached presence.
  const staffSaveId = staffAssignmentContext?.saveId ?? null;
  const staffSaveContextToken =
    staffAssignmentContext?.saveContextToken ?? null;
  const shortlistPresent = shortlistOptionsPage.state === "ready";
  const optimizerContextKey = staffOptimizerContextKey(
    activeSave,
    snapshot,
    managedClub,
    depth,
    shortlistOptionsPage,
  );
  const shortlistChoice = useShortlistFilterStore((state) =>
    staffSaveId === null || staffSaveContextToken === null
      ? undefined
      : state.choices[
          shortlistFilterKey("staff", {
            saveId: staffSaveId,
            contextToken: staffSaveContextToken,
          })
        ],
  );
  useEffect(() => {
    if (staffSaveId === null || staffSaveContextToken === null) {
      return;
    }
    useShortlistFilterStore
      .getState()
      .initialize(
        "staff",
        { saveId: staffSaveId, contextToken: staffSaveContextToken },
        shortlistPresent,
      );
  }, [staffSaveId, staffSaveContextToken, shortlistPresent]);
  // Before the effect above stores the default, the presence it will store
  // already decides the first paint.
  const shortlistOnly = resolveStaffShortlistOnly(
    routeShortlistOnly,
    shortlistChoice,
    staffAssignmentContext,
    shortlistPresent,
  );
  const panelParams = useMemo(
    () => staffSearchPanelParams(search, shortlistOnly),
    [search, shortlistOnly],
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

  return (
    <>
      <h1 className="sr-only">Staff Search</h1>
      <div data-testid="staff-page-actions">
        {staffAssignmentContext ? (
          <StaffAssignmentOptimizer
            context={staffAssignmentContext}
            contextKey={optimizerContextKey}
            contextUnavailable={staffAssignmentContextUnavailable}
            shortlistReady={shortlistPresent}
            onReviewShortlist={() => {
              if (staffSaveId !== null && staffSaveContextToken !== null) {
                useShortlistFilterStore.getState().set(
                  "staff",
                  {
                    saveId: staffSaveId,
                    contextToken: staffSaveContextToken,
                  },
                  true,
                );
              }
              void updateSearch({ shortlistOnly: undefined });
            }}
            onOpenStaff={(uid) => router.history.push(`/staff/${uid}`)}
          />
        ) : null}
      </div>
      <div className="flex min-h-0 flex-1 flex-col gap-gutter">
        <div className="flex min-h-0 flex-1 flex-col">
          <Suspense fallback={<StaffFallback />}>
            <StaffSearchResultsPanel
              activeSnapshotId={snapshot?.id ?? null}
              sortBy={panelParams.sortBy}
              sortDir={panelParams.sortDir}
              filters={panelParams.filters}
              filterCombine={panelParams.filterCombine}
              preferredJob={panelParams.preferredJob}
              preferredJobOptions={
                shortlistOptionsPage.preferredJobOptions ?? []
              }
              unemployedOnly={panelParams.unemployedOnly}
              shortlistOnly={shortlistOnly}
              onSortChange={
                shortlistOnly
                  ? onShortlistSortChange
                  : (nextSort, nextDir) =>
                      updateSearch({ sort: nextSort, dir: nextDir })
              }
              onRulesChange={(rules) => updateSearch({ filters: rules })}
              onApplyFilters={(rules, nextCombine) => {
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
              onShortlistOnlyChange={(next) => {
                if (staffSaveId !== null && staffSaveContextToken !== null) {
                  useShortlistFilterStore.getState().set(
                    "staff",
                    {
                      saveId: staffSaveId,
                      contextToken: staffSaveContextToken,
                    },
                    next,
                  );
                }
                void updateSearch({ shortlistOnly: undefined });
              }}
              onPreferredJobChange={onPreferredJobChange}
              onUnemployedOnlyChange={(value: boolean) =>
                void updateSearch({ unemployedOnly: value })
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

function StaffPageContent() {
  const { view } = Route.useSearch();
  if (view === "my-staff") {
    return <MyStaffContent />;
  }
  return <StaffSearchContent />;
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
