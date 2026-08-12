import { useIsFetching, useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute, Link } from "@tanstack/react-router";
import { DatabaseZap, UsersRound } from "lucide-react";
import { Suspense } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
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
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { snapshotKeys } from "@/features/snapshot/api/snapshot-keys";

export type PlannerSearch = {
  view?: PlannerWorkspace;
};

export const Route = createFileRoute("/planner")({
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(plannerClubFamilyQueryOptions),
      queryClient.ensureQueryData(plannerClubsQueryOptions),
      queryClient.ensureQueryData(plannerTacticQueryOptions),
      queryClient.ensureQueryData(plannerTacticOptionsQueryOptions),
      queryClient.ensureQueryData(plannerDepthQueryOptions),
    ]),
  validateSearch: (search: Record<string, unknown>): PlannerSearch => {
    const view = parsePlannerWorkspace(search.view);
    return view ? { view } : {};
  },
  component: PlannerPage,
});

function PlannerPageContent() {
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
  const { view } = Route.useSearch();
  const navigate = Route.useNavigate();
  const requestedWorkspace = parsePlannerWorkspace(view);
  const activeWorkspace = requestedWorkspace ?? "squad";
  const onWorkspaceChange = (nextWorkspace: PlannerWorkspace) => {
    void navigate({
      search: (previous) => ({ ...previous, view: nextWorkspace }),
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
    <div className="space-y-2">
      {plannerHeader}
      <div {...plannerWorkspacePanelProps("squad", activeWorkspace)}>
        {clubFamily.primaryClub ? (
          <Panel title="Squad overview" flush>
            <EmptyState
              icon={UsersRound}
              title="Squad overview"
              action={
                <Link
                  to="/planner"
                  search={{ view: "planner" }}
                  className="inline-flex h-8 items-center rounded-full border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high"
                >
                  Open Planner
                </Link>
              }
            >
              Use Planner to manage your squad depth.
            </EmptyState>
          </Panel>
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
      <div {...plannerWorkspacePanelProps("planner", activeWorkspace)}>
        <PlannerDepthMatrix
          activeSaveId={snapshot.saveId}
          depth={depth}
          tactic={tactic}
          options={tacticOptions}
        />
      </div>
      <div {...plannerWorkspacePanelProps("tactic", activeWorkspace)}>
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
