import {
  useIsFetching,
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";
import { CircleAlert, DatabaseZap, UsersRound } from "lucide-react";
import { Suspense, useRef, useState } from "react";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { SquadCsvImportActions } from "@/features/csv-import/components/squad-csv-import-actions";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import {
  managedClubOptionsQueryOptions,
  managedClubQueryOptions,
} from "@/features/managed-club/api/managed-club-query-options";
import { ManagedClubSelector } from "@/features/managed-club/components/managed-club-selector";
import {
  type MyClubWorkspace,
  MyClubWorkspaceTabs,
  myClubWorkspacePanelProps,
  parseMyClubWorkspace,
} from "@/features/my-club/components/my-club-workspace-tabs";
import { plannerDepthQueryOptions } from "@/features/planner/api/planner-depth-query-options";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { plannerTacticOptionsQueryOptions } from "@/features/planner/api/planner-tactic-options-query-options";
import { plannerTacticQueryOptions } from "@/features/planner/api/planner-tactic-query-options";
import { PlannerDepthMatrix } from "@/features/planner/components/planner-depth-matrix";
import { PlannerTacticEditor } from "@/features/planner/components/planner-tactic-editor";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { boostSquadCurrentAbility } from "@/features/squad/api/boost-squad-current-ability";
import { boostSquadWonderkidMentality } from "@/features/squad/api/boost-squad-wonderkid-mentality";
import { squadPlayersQueryOptions } from "@/features/squad/api/squad-players-query-options";
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

export type MyClubSearch = {
  view?: MyClubWorkspace;
  squadSort?: SquadSortField;
  squadDir?: SquadSortDir;
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

type SquadBoostMutationVariables = {
  snapshotId: number;
  onProgress: (progress: SquadPlayerBoostProgress) => void;
};

export const Route = createFileRoute("/my-club")({
  loaderDeps: ({ search }) => squadSortForSearch(search),
  loader: ({ context: { queryClient }, deps: { sort, dir } }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(managedClubQueryOptions),
      queryClient.prefetchQuery(managedClubOptionsQueryOptions),
      queryClient.ensureQueryData(plannerTacticQueryOptions),
      queryClient.ensureQueryData(plannerTacticOptionsQueryOptions),
      queryClient.ensureQueryData(plannerDepthQueryOptions),
      queryClient.ensureQueryData(
        squadPlayersQueryOptions(0, undefined, sort, dir),
      ),
    ]),
  validateSearch: (search: Record<string, unknown>): MyClubSearch => {
    const view = parseMyClubWorkspace(search.view);
    const squadSort = isSquadSortField(search.squadSort)
      ? search.squadSort
      : undefined;
    const squadDir = isSquadSortDir(search.squadDir)
      ? search.squadDir
      : undefined;
    return {
      ...(view ? { view } : {}),
      ...(squadSort ? { squadSort } : {}),
      ...(squadDir ? { squadDir } : {}),
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

function MyClubPageContent() {
  const queryClient = useQueryClient();
  const [latestSquadBoostAction, setLatestSquadBoostAction] =
    useState<SquadPlayerBoostAction | null>(null);
  const [openSquadBoostAction, setOpenSquadBoostAction] =
    useState<SquadPlayerBoostAction | null>(null);
  const squadBoostFeedbackRef = useRef<HTMLDivElement>(null);
  const { data: snapshot, isRefetchError: snapshotRefreshError } =
    useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: managedClub } = useSuspenseQuery(managedClubQueryOptions);
  const { data: tactic, isRefetchError: tacticRefreshError } = useSuspenseQuery(
    plannerTacticQueryOptions,
  );
  const { data: tacticOptions, isRefetchError: tacticOptionsRefreshError } =
    useSuspenseQuery(plannerTacticOptionsQueryOptions);
  const { data: depth, isRefetchError: depthRefreshError } = useSuspenseQuery(
    plannerDepthQueryOptions,
  );
  const invalidateSquadBoostQueries = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: snapshotKeys.all }),
      queryClient.invalidateQueries({ queryKey: searchKeys.all }),
      queryClient.invalidateQueries({ queryKey: playerKeys.all }),
      queryClient.invalidateQueries({ queryKey: plannerKeys.all }),
      queryClient.invalidateQueries({ queryKey: academyKeys.all }),
    ]);
  };
  const onManagedClubSaved = () => {
    void queryClient.invalidateQueries({ queryKey: plannerKeys.all });
    void queryClient.resetQueries({ queryKey: academyKeys.all });
    void queryClient.invalidateQueries({ queryKey: staffKeys.all });
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
  const activeSaveRefreshError =
    snapshotRefreshError ||
    tacticRefreshError ||
    tacticOptionsRefreshError ||
    depthRefreshError;
  const isActiveSaveUnavailable =
    isPlannerRefreshing || isSnapshotRefreshing || activeSaveRefreshError;
  const search = Route.useSearch();
  const { view } = search;
  const { sort: squadSort, dir: squadDir } = squadSortForSearch(search);
  const navigate = Route.useNavigate();
  const requestedWorkspace = parseMyClubWorkspace(view);
  const activeWorkspace = requestedWorkspace ?? "squad";
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
                <ManagedClubSelector onSaved={onManagedClubSaved} />
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
          <Suspense
            fallback={
              <div className="flex min-h-40 flex-1 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
                Loading squad overview…
              </div>
            }
          >
            <SquadOverviewPanel
              key={`${squadSort}:${squadDir}`}
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
                  />
                </div>
              }
              sortBy={squadSort}
              sortDir={squadDir}
              onSortChange={onSquadSortChange}
            />
          </Suspense>
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
        <PlannerDepthMatrix
          key={snapshot.saveId}
          activeSaveId={snapshot.saveId}
          depth={depth}
          tactic={tactic}
          options={tacticOptions}
        />
      </div>
      <div
        {...myClubWorkspacePanelProps("tactic", activeWorkspace)}
        className="min-h-0 flex-1 overflow-y-auto"
      >
        {/* Key the editor to the active save so its local draft cannot cross a save boundary. */}
        <PlannerTacticEditor
          key={snapshot.saveId}
          activeSaveRefreshError={activeSaveRefreshError}
          isActiveSaveUnavailable={isActiveSaveUnavailable}
          tactic={tactic}
          options={tacticOptions}
        />
      </div>
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
