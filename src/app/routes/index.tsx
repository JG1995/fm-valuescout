import { createFileRoute } from "@tanstack/react-router";
import { Suspense } from "react";
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

function IndexPage() {
  return (
    <section className="space-y-4">
      <h1 className="text-2xl font-semibold text-on-background">
        FM ValueScout
      </h1>
      <Suspense
        fallback={
          <p className="text-on-background/80">Loading snapshot data…</p>
        }
      >
        <SnapshotPanelsWithErrorBoundary />
      </Suspense>
      <Suspense
        fallback={
          <p className="text-on-background/80">Loading bridge status…</p>
        }
      >
        <BridgeStatusPanelWithErrorBoundary />
      </Suspense>
      <Suspense
        fallback={
          <p className="text-on-background/80">Loading health status…</p>
        }
      >
        <HealthStatusPanelWithErrorBoundary />
      </Suspense>
    </section>
  );
}
