import { createFileRoute } from "@tanstack/react-router";
import { Suspense } from "react";
import { demoValueQueryOptions } from "@/features/health/api/demo-value-query-options";
import { healthQueryOptions } from "@/features/health/api/health-query-options";
import { HealthStatusPanelWithErrorBoundary } from "@/features/health/components/health-status-panel-with-error-boundary";

export const Route = createFileRoute("/")({
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.ensureQueryData(healthQueryOptions),
      queryClient.ensureQueryData(demoValueQueryOptions),
    ]),
  component: IndexPage,
});

function IndexPage() {
  return (
    <section className="space-y-4">
      <h1 className="text-2xl font-semibold text-on-background">
        Cursor React Tauri Template
      </h1>
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
