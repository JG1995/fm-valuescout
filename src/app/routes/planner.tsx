import { useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { DatabaseZap } from "lucide-react";
import { Suspense } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { plannerClubFamilyQueryOptions } from "@/features/planner/api/get-planner-club-family-query-options";
import { plannerClubsQueryOptions } from "@/features/planner/api/planner-clubs-query-options";
import { PlannerClubFamilyPanel } from "@/features/planner/components/planner-club-family-panel";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";

export const Route = createFileRoute("/planner")({
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(currentSnapshotQueryOptions),
      queryClient.ensureQueryData(plannerClubFamilyQueryOptions),
      queryClient.ensureQueryData(plannerClubsQueryOptions),
    ]),
  component: PlannerPage,
});

function PlannerPageContent() {
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);

  if (!snapshot) {
    return (
      <Panel title="Planner" flush>
        <EmptyState icon={DatabaseZap} title="No data loaded for this save">
          No snapshot loaded for the active save. Use Load Data to scan Football
          Manager and ingest players before setting up your club family.
        </EmptyState>
      </Panel>
    );
  }

  return <PlannerClubFamilyPanel />;
}

function PlannerPage() {
  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-lg text-on-surface">Squad Planner</h1>
      <Suspense
        fallback={
          <div className="flex min-h-40 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
            Loading planner…
          </div>
        }
      >
        <PlannerPageContent />
      </Suspense>
    </div>
  );
}
