import { useQueryClient, useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Suspense } from "react";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { CsvImportPanel } from "@/features/csv-import/components/csv-import-panel";
import { BridgeStatusPanelWithErrorBoundary } from "@/features/memory-read/components/bridge-status-panel-with-error-boundary";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { PlannerClubFamilyPanel } from "@/features/planner/components/planner-club-family-panel";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { sanityPlayersQueryOptions } from "@/features/snapshot/api/sanity-players-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { SnapshotPanelsWithErrorBoundary } from "@/features/snapshot/components/snapshot-panels-with-error-boundary";
import { staffKeys } from "@/features/staff/api/staff-keys";

export const Route = createFileRoute("/")({
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(savesQueryOptions),
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(sanityPlayersQueryOptions),
    ]),
  component: IndexPage,
});

/** Panels load independently so one slow or failing area cannot blank the page. */
function PanelFallback({ label }: { label: string }) {
  return (
    <div className="flex min-h-40 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
      {label}
    </div>
  );
}

function IndexPage() {
  const queryClient = useQueryClient();
  const { data: saves } = useSuspenseQuery(savesQueryOptions);
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const activeSaveId = saves.find((save) => save.isActive)?.id;

  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-lg text-on-surface">Dashboard</h1>
      <Suspense fallback={<PanelFallback label="Loading snapshot data…" />}>
        <SnapshotPanelsWithErrorBoundary
          onCurrentContextChanged={() => {
            void queryClient.invalidateQueries({ queryKey: searchKeys.all });
            void queryClient.invalidateQueries({ queryKey: playerKeys.all });
            void queryClient.invalidateQueries({ queryKey: plannerKeys.all });
            void queryClient.resetQueries({ queryKey: academyKeys.all });
            void queryClient.invalidateQueries({ queryKey: staffKeys.all });
          }}
        />
      </Suspense>
      {snapshot ? (
        <section aria-label="Club Setup" id="club-setup">
          <Suspense fallback={<PanelFallback label="Loading club setup…" />}>
            <PlannerClubFamilyPanel
              onSaved={() => {
                void queryClient.invalidateQueries({ queryKey: staffKeys.all });
              }}
            />
          </Suspense>
        </section>
      ) : null}
      <Suspense fallback={<PanelFallback label="Loading CSV import…" />}>
        <CsvImportPanel
          activeSaveId={activeSaveId}
          snapshotId={snapshot?.id}
          onYouthImported={() => {
            void queryClient.invalidateQueries({ queryKey: academyKeys.all });
          }}
        />
      </Suspense>
      <Suspense fallback={<PanelFallback label="Loading bridge status…" />}>
        <BridgeStatusPanelWithErrorBoundary />
      </Suspense>
    </div>
  );
}
