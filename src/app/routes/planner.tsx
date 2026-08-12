import {
  useIsFetching,
  useMutation,
  useQueryClient,
  useSuspenseQuery,
} from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";
import { DatabaseZap, UsersRound } from "lucide-react";
import { Suspense } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { SquadCsvImportActions } from "@/features/csv-import/components/squad-csv-import-actions";
import { plannerClubFamilyQueryOptions } from "@/features/planner/api/get-planner-club-family-query-options";
import { plannerClubsQueryOptions } from "@/features/planner/api/planner-clubs-query-options";
import { plannerDepthQueryOptions } from "@/features/planner/api/planner-depth-query-options";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { plannerTacticOptionsQueryOptions } from "@/features/planner/api/planner-tactic-options-query-options";
import { plannerTacticQueryOptions } from "@/features/planner/api/planner-tactic-query-options";
import { PlannerDepthMatrix } from "@/features/planner/components/planner-depth-matrix";
import { PlannerTacticEditor } from "@/features/planner/components/planner-tactic-editor";
import {
  type PlannerWorkspace,
  PlannerWorkspaceTabs,
  parsePlannerWorkspace,
  plannerWorkspacePanelProps,
} from "@/features/planner/components/planner-workspace-tabs";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";
import { boostSquadCurrentAbility } from "@/features/squad/api/boost-squad-current-ability";
import { boostSquadWonderkidMentality } from "@/features/squad/api/boost-squad-wonderkid-mentality";
import { squadPlayersQueryOptions } from "@/features/squad/api/squad-players-query-options";
import { SquadOverviewPanel } from "@/features/squad/components/squad-overview-panel";
import {
  SquadCurrentAbilityBoost,
  SquadWonderkidMentalityBoost,
} from "@/features/squad/components/squad-player-boost";
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

export type PlannerSearch = {
  view?: PlannerWorkspace;
  sort?: SquadSortField;
  dir?: SquadSortDir;
};

function squadSortForSearch(search: PlannerSearch): {
  sort: SquadSortField;
  dir: SquadSortDir;
} {
  const sort = isSquadSortField(search.sort)
    ? search.sort
    : DEFAULT_SQUAD_SORT_FIELD;
  const dir = isSquadSortDir(search.dir)
    ? search.dir
    : isSquadSortField(search.sort)
      ? defaultDirForSquadSortField(sort)
      : DEFAULT_SQUAD_SORT_DIR;
  return { sort, dir };
}

export const Route = createFileRoute("/planner")({
  loaderDeps: ({ search }) => squadSortForSearch(search),
  loader: ({ context: { queryClient }, deps: { sort, dir } }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(plannerClubFamilyQueryOptions),
      queryClient.ensureQueryData(plannerClubsQueryOptions),
      queryClient.ensureQueryData(plannerTacticQueryOptions),
      queryClient.ensureQueryData(plannerTacticOptionsQueryOptions),
      queryClient.ensureQueryData(plannerDepthQueryOptions),
      queryClient.ensureQueryData(
        squadPlayersQueryOptions(0, undefined, sort, dir),
      ),
    ]),
  validateSearch: (search: Record<string, unknown>): PlannerSearch => {
    const view = parsePlannerWorkspace(search.view);
    const sort = isSquadSortField(search.sort) ? search.sort : undefined;
    const dir = isSquadSortDir(search.dir) ? search.dir : undefined;
    return {
      ...(view ? { view } : {}),
      ...(sort ? { sort } : {}),
      ...(dir ? { dir } : {}),
    };
  },
  component: PlannerPage,
});

function PlannerPageContent() {
  const queryClient = useQueryClient();
  const { data: snapshot, isRefetchError: snapshotRefreshError } =
    useSuspenseQuery(currentSnapshotQueryOptions);
  const { data: clubFamily } = useSuspenseQuery(plannerClubFamilyQueryOptions);
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
  const squadCurrentAbilityBoost = useMutation({
    mutationFn: (_: { snapshotId: number }) => boostSquadCurrentAbility(),
    onSuccess: invalidateSquadBoostQueries,
  });
  const squadWonderkidMentalityBoost = useMutation({
    mutationFn: (_: { snapshotId: number }) => boostSquadWonderkidMentality(),
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
  const requestedWorkspace = parsePlannerWorkspace(view);
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
  const onWorkspaceChange = (nextWorkspace: PlannerWorkspace) => {
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
        sort: nextSort,
        dir: nextDir,
      }),
      replace: true,
    });
  };
  const plannerHeader = (
    <header className="flex flex-col items-start gap-2">
      <div>
        <h1 className="text-headline-lg text-on-surface">Squad</h1>
        {clubFamily.primaryClub ? (
          <p className="text-body-sm text-on-surface-variant">
            Primary club: {clubFamily.primaryClub}
          </p>
        ) : null}
      </div>
      {snapshot ? (
        <PlannerWorkspaceTabs
          workspace={activeWorkspace}
          onWorkspaceChange={onWorkspaceChange}
        />
      ) : null}
    </header>
  );

  if (!snapshot) {
    return (
      <div className="space-y-2">
        {plannerHeader}
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
    <div className="flex min-h-full min-w-0 flex-col gap-2">
      {plannerHeader}
      <div
        {...plannerWorkspacePanelProps("squad", activeWorkspace)}
        className="flex min-h-0 flex-1 flex-col"
      >
        {clubFamily.primaryClub ? (
          <Suspense
            fallback={
              <div className="flex min-h-40 flex-1 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
                Loading squad overview…
              </div>
            }
          >
            <SquadOverviewPanel
              key={`${squadSort}:${squadDir}`}
              actions={
                <div className="flex flex-wrap justify-end gap-2">
                  <SquadCurrentAbilityBoost
                    key={`current-ability-${snapshot.id}`}
                    pending={
                      squadCurrentAbilityBoostContextIsCurrent &&
                      squadCurrentAbilityBoost.isPending
                    }
                    disabled={squadBoostPending || squadBoostRecoveryRequired}
                    result={
                      squadCurrentAbilityBoostContextIsCurrent
                        ? squadCurrentAbilityBoost.data
                        : undefined
                    }
                    error={
                      squadCurrentAbilityBoostContextIsCurrent
                        ? squadCurrentAbilityBoost.error
                        : null
                    }
                    onBoost={() =>
                      squadCurrentAbilityBoost.mutateAsync({
                        snapshotId: snapshot.id,
                      })
                    }
                    onOpenConfirmation={squadCurrentAbilityBoost.reset}
                  />
                  <SquadWonderkidMentalityBoost
                    key={`wonderkid-mentality-${snapshot.id}`}
                    pending={
                      squadWonderkidMentalityBoostContextIsCurrent &&
                      squadWonderkidMentalityBoost.isPending
                    }
                    disabled={squadBoostPending || squadBoostRecoveryRequired}
                    result={
                      squadWonderkidMentalityBoostContextIsCurrent
                        ? squadWonderkidMentalityBoost.data
                        : undefined
                    }
                    error={
                      squadWonderkidMentalityBoostContextIsCurrent
                        ? squadWonderkidMentalityBoost.error
                        : null
                    }
                    onBoost={() =>
                      squadWonderkidMentalityBoost.mutateAsync({
                        snapshotId: snapshot.id,
                      })
                    }
                    onOpenConfirmation={squadWonderkidMentalityBoost.reset}
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
              title="Set up your club family"
              action={
                <Link
                  to="/"
                  hash="club-setup"
                  className="inline-flex h-8 items-center rounded-full border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high"
                >
                  Open Club Setup
                </Link>
              }
            >
              Configure your club family in Dashboard before reviewing your
              squad.
            </EmptyState>
          </Panel>
        )}
      </div>
      <div
        {...plannerWorkspacePanelProps("planner", activeWorkspace)}
        className="min-h-0 flex-1 overflow-y-auto"
      >
        <PlannerDepthMatrix
          activeSaveId={snapshot.saveId}
          depth={depth}
          tactic={tactic}
          options={tacticOptions}
        />
      </div>
      <div
        {...plannerWorkspacePanelProps("tactic", activeWorkspace)}
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

function PlannerPage() {
  return (
    <Suspense
      fallback={
        <div className="space-y-gutter">
          <h1 className="text-headline-lg text-on-surface">Squad</h1>
          <div className="flex min-h-40 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
            Loading planner…
          </div>
        </div>
      }
    >
      <PlannerPageContent />
    </Suspense>
  );
}
