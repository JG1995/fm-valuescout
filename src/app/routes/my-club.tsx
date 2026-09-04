import {
  useIsFetching,
  useIsMutating,
  useMutation,
  useQuery,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { createFileRoute, Link, useRouter } from "@tanstack/react-router";
import { CircleAlert, DatabaseZap, UsersRound } from "lucide-react";
import { Suspense, useRef, useState } from "react";
import { clearPlayerResultContext } from "@/app/player-result-context";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import { playerResultContextMutationKey } from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { clubDnaKeys } from "@/features/club-dna/api/club-dna-keys";
import { ClubDnaDefinition } from "@/features/club-dna/components/club-dna-definition";
import type {
  ClubDnaContext,
  ClubDnaRemoveResult,
  ClubDnaUpsertResult,
} from "@/features/club-dna/types/club-dna";
import { SquadCsvImportActions } from "@/features/csv-import/components/squad-csv-import-actions";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import {
  managedClubOptionsQueryOptions,
  managedClubQueryOptions,
} from "@/features/managed-club/api/managed-club-query-options";
import { ManagedClubSelector } from "@/features/managed-club/components/managed-club-selector";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import {
  type MyClubWorkspace,
  MyClubWorkspaceTabs,
  myClubWorkspacePanelProps,
  parseMyClubWorkspace,
} from "@/features/my-club/components/my-club-workspace-tabs";
import { plannerDepthQueryOptions } from "@/features/planner/api/planner-depth-query-options";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { PlannerDepthMatrix } from "@/features/planner/components/planner-depth-matrix";
import { PlannerTacticEditor } from "@/features/planner/components/planner-tactic-editor";
import { TacticContextBoundary } from "@/features/planner/components/tactic-context-boundary";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { boostSquadCurrentAbility } from "@/features/squad/api/boost-squad-current-ability";
import { boostSquadWonderkidMentality } from "@/features/squad/api/boost-squad-wonderkid-mentality";
import { squadKeys } from "@/features/squad/api/squad-keys";
import { SquadOverviewPanel } from "@/features/squad/components/squad-overview-panel";
import {
  SquadBoostOutcome,
  SquadCurrentAbilityBoost,
  type SquadPlayerBoostAction,
  SquadWonderkidMentalityBoost,
} from "@/features/squad/components/squad-player-boost";
import type { SquadPlayerBoostProgress } from "@/features/squad/types/squad-player-boost";
import type {
  SquadSortDir,
  SquadSortField,
} from "@/features/squad/types/squad-sort";
import {
  DEFAULT_SQUAD_SORT_DIR,
  DEFAULT_SQUAD_SORT_FIELD,
  defaultDirForSquadSortField,
  isSquadSortDir,
  isSquadSortField,
} from "@/features/squad/types/squad-sort";
import { staffKeys } from "@/features/staff/api/staff-keys";
import { staffMyStaffQueryOptions } from "@/features/staff/api/staff-query-options";
import { StaffSearchResultsPanel } from "@/features/staff/components/staff-search-results-panel";
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
import { usePlayerTableStore } from "@/stores/use-player-table-store";

export type MyClubSearch = {
  view?: MyClubWorkspace;
  squadSort?: SquadSortField;
  squadDir?: SquadSortDir;
  staffSort?: StaffSortField;
  staffDir?: StaffSortDir;
};

function squadSortForSearch(search: MyClubSearch): {
  sort: SquadSortField;
  dir: SquadSortDir;
} {
  const sort = isSquadSortField(search.squadSort)
    ? search.squadSort
    : DEFAULT_SQUAD_SORT_FIELD;
  const dir = isSquadSortDir(search.squadDir)
    ? search.squadDir
    : isSquadSortField(search.squadSort)
      ? defaultDirForSquadSortField(sort)
      : DEFAULT_SQUAD_SORT_DIR;
  return { sort, dir };
}

function staffSortForSearch(search: MyClubSearch): {
  sort: StaffSortField;
  dir: StaffSortDir;
} {
  const sort = isStaffSortField(search.staffSort)
    ? search.staffSort
    : DEFAULT_STAFF_SORT_FIELD;
  const dir = isStaffSortDir(search.staffDir)
    ? search.staffDir
    : defaultDirForStaffSortField(sort);
  return { sort, dir };
}

type SquadBoostMutationVariables = {
  snapshotId: number;
  onProgress: (progress: SquadPlayerBoostProgress) => void;
};

type MyClubStaffSearchPatch = {
  staffSort?: StaffSortField;
  staffDir?: StaffSortDir;
};

export const Route = createFileRoute("/my-club")({
  loaderDeps: ({ search }) => {
    const { sort, dir } = squadSortForSearch(search);
    const staff = staffSortForSearch(search);
    return {
      view: parseMyClubWorkspace(search.view) ?? "squad",
      sort,
      dir,
      staffSort: staff.sort,
      staffDir: staff.dir,
    };
  },
  loader: ({
    context: { queryClient },
    deps: { view, staffSort, staffDir },
  }) => {
    const staffQuery =
      view === "staff"
        ? queryClient.ensureQueryData(
            staffMyStaffQueryOptions(0, undefined, staffSort, staffDir, []),
          )
        : Promise.resolve();
    return Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(managedClubQueryOptions),
      queryClient.prefetchQuery(managedClubOptionsQueryOptions),
      queryClient.ensureQueryData(plannerDepthQueryOptions),
      staffQuery,
    ]);
  },
  beforeLoad: ({ location }) => {
    // Legacy Staff Shortlist links replace the history entry with the
    // canonical Staff Search + shortlistOnly URL without inspecting
    // persistence. location.search is the raw query.
    const raw = location.search as Record<string, unknown> | undefined;
    if (raw?.view === "staff-shortlist") {
      const shortlistSort = isStaffShortlistSortField(raw.shortlistSort)
        ? raw.shortlistSort
        : DEFAULT_STAFF_SORT_FIELD;
      throw Route.redirect({
        to: "/staff",
        search: {
          view: "search",
          sort: DEFAULT_STAFF_SORT_FIELD,
          dir: defaultDirForStaffSortField(DEFAULT_STAFF_SORT_FIELD),
          searchSort: DEFAULT_STAFF_SORT_FIELD,
          searchDir: defaultDirForStaffSortField(DEFAULT_STAFF_SORT_FIELD),
          myStaffSort: DEFAULT_STAFF_SORT_FIELD,
          myStaffDir: defaultDirForStaffSortField(DEFAULT_STAFF_SORT_FIELD),
          shortlistSort,
          shortlistDir: isStaffSortDir(raw.shortlistDir)
            ? raw.shortlistDir
            : defaultDirForStaffSortField(shortlistSort),
          ...(isStaffShortlistSortField(raw.shortlistContextSort) &&
          isStaffSortDir(raw.shortlistContextDir)
            ? {
                shortlistContextSort: raw.shortlistContextSort,
                shortlistContextDir: raw.shortlistContextDir,
              }
            : {}),
          shortlistOnly: true,
          ...(typeof raw.preferredJob === "string" && raw.preferredJob
            ? { preferredJob: raw.preferredJob }
            : {}),
          unemployedOnly: raw.unemployedOnly === true,
          filters: [],
          combine: "and",
        },
        replace: true,
      });
    }
  },
  validateSearch: (search: Record<string, unknown>): MyClubSearch => {
    const view = parseMyClubWorkspace(search.view);
    const squadSort = isSquadSortField(search.squadSort)
      ? search.squadSort
      : undefined;
    const squadDir = isSquadSortDir(search.squadDir)
      ? search.squadDir
      : undefined;
    const staffSort = isStaffSortField(search.staffSort)
      ? search.staffSort
      : undefined;
    const staffDir = isStaffSortDir(search.staffDir)
      ? search.staffDir
      : undefined;
    return {
      ...(view ? { view } : {}),
      ...(squadSort ? { squadSort } : {}),
      ...(squadDir ? { squadDir } : {}),
      ...(staffSort ? { staffSort } : {}),
      ...(staffDir ? { staffDir } : {}),
    };
  },
  component: MyClubPage,
});

function ManagedClubError({
  error,
  reset,
}: {
  error: Error;
  reset: () => void;
}) {
  const queryClient = useQueryClient();

  return (
    <Panel>
      <EmptyState
        icon={CircleAlert}
        title="Could not load managed club"
        action={
          <Button
            variant="secondary"
            onClick={() => {
              queryClient.resetQueries({ queryKey: managedClubKeys.all });
              reset();
            }}
          >
            Retry
          </Button>
        }
      >
        {error.message}
      </EmptyState>
    </Panel>
  );
}

function TacticQueryError({
  error,
  title,
  onRetry,
}: {
  error: Error;
  title: string;
  onRetry: () => void;
}) {
  return (
    <div
      className="m-3 flex flex-wrap items-center justify-between gap-3 rounded-lg border border-error/40 bg-error-container p-3 text-on-error-container"
      role="alert"
    >
      <div>
        <p className="text-label-lg">{title}</p>
        <p className="text-body-sm">{error.message}</p>
      </div>
      <Button variant="secondary" onClick={onRetry}>
        Retry
      </Button>
    </div>
  );
}

function MyClubPageContent() {
  const queryClient = useQueryClient();
  const addColumns = usePlayerTableStore((state) => state.addColumns);
  const savesQuery = useQuery(savesQueryOptions);
  const activeSave = savesQuery.data?.find((save) => save.isActive);
  const activeClubDnaContext =
    savesQuery.isSuccess && !savesQuery.isFetching && activeSave
      ? { saveId: activeSave.id, contextToken: activeSave.contextToken }
      : null;
  const activeClubDnaContextRef = useRef<ClubDnaContext | null>(
    activeClubDnaContext,
  );
  activeClubDnaContextRef.current = activeClubDnaContext;
  const [latestSquadBoostAction, setLatestSquadBoostAction] =
    useState<SquadPlayerBoostAction | null>(null);
  const [openSquadBoostAction, setOpenSquadBoostAction] =
    useState<SquadPlayerBoostAction | null>(null);
  const squadBoostFeedbackRef = useRef<HTMLDivElement>(null);
  const { data: snapshot, isRefetchError: snapshotRefreshError } =
    useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: managedClub, isRefetchError: managedClubRefreshError } =
    useSuspenseQuery(managedClubQueryOptions);
  const { data: depth, isRefetchError: depthRefreshError } = useSuspenseQuery(
    plannerDepthQueryOptions,
  );
  const plannerContext = activeClubDnaContext;
  const isMatchedSnapshot =
    snapshot !== null &&
    plannerContext !== null &&
    snapshot.saveId === plannerContext.saveId;
  const invalidateSquadBoostQueries = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: snapshotKeys.all }),
      queryClient.invalidateQueries({ queryKey: searchKeys.all }),
      queryClient.invalidateQueries({ queryKey: playerKeys.all }),
      queryClient.invalidateQueries({ queryKey: plannerKeys.all }),
      queryClient.invalidateQueries({ queryKey: academyKeys.all }),
    ]);
  };
  const clearResults = () => clearPlayerResultContext(queryClient);
  const onManagedClubSaved = () => {
    void queryClient.invalidateQueries({ queryKey: plannerKeys.all });
    void queryClient.resetQueries({ queryKey: academyKeys.all });
    void queryClient.invalidateQueries({ queryKey: staffKeys.all });
  };
  const clubDnaContextIsCurrent = (context: ClubDnaContext) =>
    activeClubDnaContextRef.current?.saveId === context.saveId &&
    activeClubDnaContextRef.current?.contextToken === context.contextToken;
  const invalidateClubDnaConsumers = (context: ClubDnaContext) => {
    if (!clubDnaContextIsCurrent(context)) {
      return;
    }
    void queryClient.invalidateQueries({
      queryKey: clubDnaKeys.definition(context),
    });
    void queryClient.invalidateQueries({ queryKey: searchKeys.all });
    void queryClient.invalidateQueries({ queryKey: squadKeys.all });
  };
  const onClubDnaSaved = (
    result: ClubDnaUpsertResult,
    context: ClubDnaContext,
  ) => {
    if (!clubDnaContextIsCurrent(context)) {
      return;
    }
    if (result.created) {
      addColumns("search", ["club_dna"]);
      addColumns("squad", ["club_dna"]);
    }
    invalidateClubDnaConsumers(context);
  };
  const onClubDnaRemoved = (
    _result: ClubDnaRemoveResult,
    context: ClubDnaContext,
  ) => {
    invalidateClubDnaConsumers(context);
  };
  const squadCurrentAbilityBoost = useMutation({
    mutationFn: ({ onProgress }: SquadBoostMutationVariables) =>
      boostSquadCurrentAbility(onProgress),
    onSuccess: invalidateSquadBoostQueries,
  });
  const squadWonderkidMentalityBoost = useMutation({
    mutationFn: ({ onProgress }: SquadBoostMutationVariables) =>
      boostSquadWonderkidMentality(onProgress),
    onSuccess: invalidateSquadBoostQueries,
  });
  const isPlannerRefreshing = useIsFetching({ queryKey: plannerKeys.all }) > 0;
  const isSnapshotRefreshing =
    useIsFetching({ queryKey: snapshotKeys.all }) > 0;
  const isSavesRefreshing = savesQuery.isFetching;
  const isManagedClubRefreshing =
    useIsFetching({ queryKey: managedClubKeys.all }) > 0;
  const activeSaveRefreshError =
    savesQuery.isError ||
    snapshotRefreshError ||
    managedClubRefreshError ||
    depthRefreshError;
  const isActiveSaveUnavailable =
    isPlannerRefreshing ||
    isSnapshotRefreshing ||
    isSavesRefreshing ||
    isManagedClubRefreshing ||
    activeSaveRefreshError;
  const isPlayerResultContextMutating =
    useIsMutating({ mutationKey: playerResultContextMutationKey }) > 0;
  const isSquadResultBlocked =
    !activeSave ||
    isSavesRefreshing ||
    isSnapshotRefreshing ||
    isManagedClubRefreshing ||
    savesQuery.isError ||
    snapshotRefreshError ||
    managedClubRefreshError ||
    isPlayerResultContextMutating;
  const clubDnaAvailable =
    activeClubDnaContext !== null &&
    snapshot?.saveId === activeClubDnaContext.saveId &&
    managedClub.clubName !== null &&
    !isActiveSaveUnavailable;
  const search = Route.useSearch();
  const { view } = search;
  const { sort: squadSort, dir: squadDir } = squadSortForSearch(search);
  const navigate = Route.useNavigate();
  const router = useRouter();
  const requestedWorkspace = parseMyClubWorkspace(view);
  const activeWorkspace = requestedWorkspace ?? "squad";
  const { sort: staffSort, dir: staffDir } = staffSortForSearch(search);
  const squadCurrentAbilityBoostContextIsCurrent =
    squadCurrentAbilityBoost.variables?.snapshotId === snapshot?.id;
  const squadWonderkidMentalityBoostContextIsCurrent =
    squadWonderkidMentalityBoost.variables?.snapshotId === snapshot?.id;
  const squadBoostPending =
    (squadCurrentAbilityBoostContextIsCurrent &&
      squadCurrentAbilityBoost.isPending) ||
    (squadWonderkidMentalityBoostContextIsCurrent &&
      squadWonderkidMentalityBoost.isPending);
  const squadBoostRecoveryRequired =
    (squadCurrentAbilityBoostContextIsCurrent &&
      squadCurrentAbilityBoost.data?.recoveryRequired === true) ||
    (squadWonderkidMentalityBoostContextIsCurrent &&
      squadWonderkidMentalityBoost.data?.recoveryRequired === true);
  const latestSquadBoostContextIsCurrent =
    latestSquadBoostAction === "currentAbility"
      ? squadCurrentAbilityBoostContextIsCurrent
      : latestSquadBoostAction === "wonderkidMentality"
        ? squadWonderkidMentalityBoostContextIsCurrent
        : false;
  const squadBoostFeedback =
    latestSquadBoostAction &&
    latestSquadBoostContextIsCurrent &&
    openSquadBoostAction === null ? (
      <SquadBoostOutcome
        action={latestSquadBoostAction}
        result={
          latestSquadBoostAction === "currentAbility"
            ? squadCurrentAbilityBoost.data
            : squadWonderkidMentalityBoost.data
        }
        error={
          latestSquadBoostAction === "currentAbility"
            ? (squadCurrentAbilityBoost.error ?? null)
            : (squadWonderkidMentalityBoost.error ?? null)
        }
      />
    ) : null;
  const onWorkspaceChange = (nextWorkspace: MyClubWorkspace) => {
    void navigate({
      search: (previous) => ({ ...previous, view: nextWorkspace }),
      replace: true,
    });
  };
  const onSquadSortChange = (
    nextSort: SquadSortField,
    nextDir: SquadSortDir,
  ) => {
    void navigate({
      search: (previous) => ({
        ...previous,
        squadSort: nextSort,
        squadDir: nextDir,
      }),
      replace: true,
    });
  };
  const updateStaffSearch = (patch: MyClubStaffSearchPatch) =>
    navigate({
      search: (previous) => ({
        ...previous,
        ...(patch.staffSort ? { staffSort: patch.staffSort } : {}),
        ...(patch.staffDir ? { staffDir: patch.staffDir } : {}),
      }),
      replace: true,
    });
  const onStaffSortChange = (
    nextSort: StaffSortField,
    nextDir: StaffSortDir,
  ) => {
    void updateStaffSearch({ staffSort: nextSort, staffDir: nextDir });
  };
  const onStaffBoostSuccess = () =>
    queryClient.invalidateQueries({ queryKey: snapshotKeys.all });
  const myClubHeader = (
    <header className="flex flex-col items-start gap-2">
      <div className="flex w-full flex-wrap items-start justify-between gap-3">
        <div>
          <h1 className="text-headline-lg text-on-surface">My Club</h1>
          {managedClub.clubName ? (
            <p className="text-body-sm text-on-surface-variant">
              Managed club: {managedClub.clubName}
            </p>
          ) : null}
        </div>
        <div className="min-w-64 flex-1" id="managed-club">
          {snapshot ? (
            <ErrorBoundary
              fallback={({ error, reset }) => (
                <ManagedClubError error={error} reset={reset} />
              )}
            >
              <Suspense
                fallback={
                  <div className="flex min-h-16 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-sm text-on-surface-variant">
                    Loading managed club…
                  </div>
                }
              >
                <ManagedClubSelector
                  action={
                    activeClubDnaContext ? (
                      <ClubDnaDefinition
                        key={`${activeClubDnaContext.saveId}:${activeClubDnaContext.contextToken}`}
                        context={activeClubDnaContext}
                        available={clubDnaAvailable}
                        onSaved={onClubDnaSaved}
                        onRemoved={onClubDnaRemoved}
                      />
                    ) : (
                      <Button variant="secondary" disabled>
                        Define DNA
                      </Button>
                    )
                  }
                  onBeforeContextChange={clearResults}
                  onSaved={onManagedClubSaved}
                />
              </Suspense>
            </ErrorBoundary>
          ) : null}
        </div>
      </div>
      {snapshot ? (
        <MyClubWorkspaceTabs
          workspace={activeWorkspace}
          onWorkspaceChange={onWorkspaceChange}
        />
      ) : null}
    </header>
  );

  if (!snapshot) {
    return (
      <div className="space-y-2">
        {myClubHeader}
        <Panel title="Squad" flush>
          <EmptyState icon={DatabaseZap} title="No data loaded for this save">
            No snapshot loaded for the active save. Use Load Data to scan
            Football Manager and ingest players before reviewing your squad.
          </EmptyState>
        </Panel>
      </div>
    );
  }

  return (
    <div className="flex h-full min-w-0 flex-col gap-2">
      {myClubHeader}
      <div
        {...myClubWorkspacePanelProps("squad", activeWorkspace)}
        className="flex min-h-0 flex-1 flex-col"
      >
        {managedClub.clubName ? (
          isSquadResultBlocked ? (
            <Panel title="Squad overview" flush>
              <p className="p-4 text-body-md text-on-surface-variant">
                Loading squad overview…
              </p>
            </Panel>
          ) : (
            <SquadOverviewPanel
              key={`${activeSave?.id}:${activeSave?.contextToken}:${snapshot.id}:${snapshot.saveId}:${managedClub.clubName}:${managedClub.status}`}
              pageContext={{
                activeSave: activeSave
                  ? { id: activeSave.id, contextToken: activeSave.contextToken }
                  : null,
                currentSnapshot: { id: snapshot.id, saveId: snapshot.saveId },
                managedClub: {
                  clubName: managedClub.clubName,
                  status: managedClub.status,
                },
              }}
              feedback={squadBoostFeedback}
              feedbackRef={squadBoostFeedbackRef}
              actions={
                <div className="flex flex-wrap justify-end gap-2">
                  <SquadCurrentAbilityBoost
                    key={`current-ability-${snapshot.id}`}
                    pending={
                      squadCurrentAbilityBoostContextIsCurrent &&
                      squadCurrentAbilityBoost.isPending
                    }
                    disabled={squadBoostPending || squadBoostRecoveryRequired}
                    error={
                      squadCurrentAbilityBoostContextIsCurrent
                        ? (squadCurrentAbilityBoost.error ?? null)
                        : null
                    }
                    onBoost={(onProgress) =>
                      squadCurrentAbilityBoost.mutateAsync({
                        snapshotId: snapshot.id,
                        onProgress,
                      })
                    }
                    onOpenConfirmation={() => {
                      squadCurrentAbilityBoost.reset();
                      setLatestSquadBoostAction("currentAbility");
                    }}
                    onConfirmationChange={(open) =>
                      setOpenSquadBoostAction(open ? "currentAbility" : null)
                    }
                    fallbackFocusTo={() => squadBoostFeedbackRef.current}
                  />
                  <SquadWonderkidMentalityBoost
                    key={`wonderkid-mentality-${snapshot.id}`}
                    pending={
                      squadWonderkidMentalityBoostContextIsCurrent &&
                      squadWonderkidMentalityBoost.isPending
                    }
                    disabled={squadBoostPending || squadBoostRecoveryRequired}
                    error={
                      squadWonderkidMentalityBoostContextIsCurrent
                        ? (squadWonderkidMentalityBoost.error ?? null)
                        : null
                    }
                    onBoost={(onProgress) =>
                      squadWonderkidMentalityBoost.mutateAsync({
                        snapshotId: snapshot.id,
                        onProgress,
                      })
                    }
                    onOpenConfirmation={() => {
                      squadWonderkidMentalityBoost.reset();
                      setLatestSquadBoostAction("wonderkidMentality");
                    }}
                    onConfirmationChange={(open) =>
                      setOpenSquadBoostAction(
                        open ? "wonderkidMentality" : null,
                      )
                    }
                    fallbackFocusTo={() => squadBoostFeedbackRef.current}
                  />
                  <SquadCsvImportActions
                    activeSaveId={snapshot.saveId}
                    snapshotId={snapshot.id}
                    onYouthImported={() => {
                      void queryClient.invalidateQueries({
                        queryKey: academyKeys.all,
                      });
                    }}
                    onMoneyballImported={() => {
                      void queryClient.invalidateQueries({
                        queryKey: searchKeys.all,
                      });
                      void queryClient.invalidateQueries({
                        queryKey: moneyballKeys.all,
                      });
                    }}
                  />
                </div>
              }
              sortBy={squadSort}
              sortDir={squadDir}
              onSortChange={onSquadSortChange}
            />
          )
        ) : (
          <Panel title="Squad" flush>
            <EmptyState
              icon={UsersRound}
              title="Choose your managed club"
              action={
                <Link
                  to="/my-club"
                  hash="managed-club"
                  className="inline-flex h-8 items-center rounded-full border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high"
                >
                  Open Managed Club
                </Link>
              }
            >
              Choose your managed club in My Club before reviewing your squad.
            </EmptyState>
          </Panel>
        )}
      </div>
      <div
        {...myClubWorkspacePanelProps("planner", activeWorkspace)}
        className="min-h-0 flex-1 overflow-y-auto"
      >
        {plannerContext && isMatchedSnapshot ? (
          <TacticContextBoundary context={plannerContext}>
            {({
              tactic,
              options,
              isPending,
              initialError,
              refreshError,
              retryBoth,
            }) => {
              if (isPending) {
                return (
                  <p className="p-4 text-body-md text-on-surface-variant">
                    Loading tactic…
                  </p>
                );
              }
              if (initialError) {
                return (
                  <TacticQueryError
                    error={initialError}
                    title="Could not load tactic"
                    onRetry={retryBoth}
                  />
                );
              }
              if (!tactic || !options) {
                return null;
              }
              return (
                <>
                  {refreshError ? (
                    <TacticQueryError
                      error={refreshError}
                      title="Could not refresh tactic"
                      onRetry={retryBoth}
                    />
                  ) : null}
                  <PlannerDepthMatrix
                    key={`${plannerContext.saveId}:${plannerContext.contextToken}`}
                    activeSaveId={plannerContext.saveId}
                    depth={depth}
                    tactic={tactic}
                    options={options}
                  />
                </>
              );
            }}
          </TacticContextBoundary>
        ) : (
          <p className="p-4 text-body-md text-on-surface-variant">
            Loading planner…
          </p>
        )}
      </div>
      <div
        {...myClubWorkspacePanelProps("tactic", activeWorkspace)}
        className="min-h-0 flex-1 overflow-y-auto"
      >
        {plannerContext && isMatchedSnapshot ? (
          <TacticContextBoundary context={plannerContext}>
            {({
              tactic,
              options,
              isPending,
              initialError,
              refreshError,
              readOnly,
              retryBoth,
            }) => {
              if (isPending) {
                return (
                  <p className="p-4 text-body-md text-on-surface-variant">
                    Loading tactic…
                  </p>
                );
              }
              if (initialError) {
                return (
                  <TacticQueryError
                    error={initialError}
                    title="Could not load tactic"
                    onRetry={retryBoth}
                  />
                );
              }
              if (!tactic || !options) {
                return null;
              }
              return (
                <>
                  {refreshError ? (
                    <TacticQueryError
                      error={refreshError}
                      title="Could not refresh tactic"
                      onRetry={retryBoth}
                    />
                  ) : null}
                  <PlannerTacticEditor
                    key={`${plannerContext.saveId}:${plannerContext.contextToken}`}
                    context={plannerContext}
                    activeSaveRefreshError={activeSaveRefreshError}
                    isActiveSaveUnavailable={isActiveSaveUnavailable}
                    readOnly={readOnly}
                    tactic={tactic}
                    options={options}
                  />
                </>
              );
            }}
          </TacticContextBoundary>
        ) : null}
      </div>
      <div
        {...myClubWorkspacePanelProps("staff", activeWorkspace)}
        className="flex min-h-0 flex-1 flex-col"
      >
        <Suspense fallback={<StaffWorkspaceFallback />}>
          <StaffSearchResultsPanel
            activeSnapshotId={snapshot.id}
            scope="my-staff"
            sortBy={staffSort}
            sortDir={staffDir}
            filters={[]}
            filterCombine="and"
            onSortChange={onStaffSortChange}
            onBoostSuccess={onStaffBoostSuccess}
            onRowActivate={(staff) =>
              router.history.push(`/staff/${staff.uid}`)
            }
          />
        </Suspense>
      </div>
    </div>
  );
}

function StaffWorkspaceFallback() {
  return (
    <div
      className="flex min-h-40 flex-1 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant"
      aria-busy="true"
    >
      Loading staff…
    </div>
  );
}

function MyClubPage() {
  return (
    <Suspense
      fallback={
        <div className="space-y-gutter">
          <h1 className="text-headline-lg text-on-surface">My Club</h1>
          <div className="flex min-h-40 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
            Loading My Club…
          </div>
        </div>
      }
    >
      <MyClubPageContent />
    </Suspense>
  );
}
