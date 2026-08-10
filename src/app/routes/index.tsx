import { useSuspenseQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Suspense } from "react";
import { CsvReconciliationPreview } from "@/features/csv-import/components/csv-reconciliation-preview";
import { demoValueQueryOptions } from "@/features/health/api/demo-value-query-options";
import { healthQueryOptions } from "@/features/health/api/health-query-options";
import { HealthStatusPanelWithErrorBoundary } from "@/features/health/components/health-status-panel-with-error-boundary";
import { BridgeStatusPanelWithErrorBoundary } from "@/features/memory-read/components/bridge-status-panel-with-error-boundary";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { sanityPlayersQueryOptions } from "@/features/snapshot/api/sanity-players-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { SnapshotPanelsWithErrorBoundary } from "@/features/snapshot/components/snapshot-panels-with-error-boundary";

export const Route = createFileRoute("/")({
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(healthQueryOptions),
      queryClient.ensureQueryData(demoValueQueryOptions),
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
  const { data: saves } = useSuspenseQuery(savesQueryOptions);
  const { data: snapshot } = useSuspenseQuery(currentSnapshotQueryOptions);
  const activeSaveId = saves.find((save) => save.isActive)?.id;

  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-lg text-on-surface">Dashboard</h1>
      <Suspense fallback={<PanelFallback label="Loading snapshot data…" />}>
        <SnapshotPanelsWithErrorBoundary />
      </Suspense>
      <Suspense fallback={<PanelFallback label="Loading CSV preview…" />}>
        <CsvReconciliationPreview
          activeSaveId={activeSaveId}
          snapshotId={snapshot?.id}
        />
      </Suspense>
      <Suspense fallback={<PanelFallback label="Loading bridge status…" />}>
        <BridgeStatusPanelWithErrorBoundary />
      </Suspense>
      <Suspense fallback={<PanelFallback label="Loading health status…" />}>
        <HealthStatusPanelWithErrorBoundary />
      </Suspense>
    </div>
  );
}
