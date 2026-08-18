import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { CircleAlert } from "lucide-react";
import { Suspense } from "react";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import {
  managedClubOptionsQueryOptions,
  managedClubQueryOptions,
} from "@/features/managed-club/api/managed-club-query-options";
import { ManagedClubPanel } from "@/features/managed-club/components/managed-club-panel";
import { bridgeInstallQueryOptions } from "@/features/memory-read/api/bridge-install-query-options";
import { bridgeStatusQueryOptions } from "@/features/memory-read/api/bridge-status-query-options";
import { BridgeStatusPanelWithErrorBoundary } from "@/features/memory-read/components/bridge-status-panel-with-error-boundary";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { SnapshotPanelsWithErrorBoundary } from "@/features/snapshot/components/snapshot-panels-with-error-boundary";
import { staffKeys } from "@/features/staff/api/staff-keys";

export const Route = createFileRoute("/settings")({
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.prefetchQuery(savesQueryOptions),
      queryClient.prefetchQuery(currentSnapshotQueryOptions),
      queryClient.prefetchQuery(managedClubQueryOptions),
      queryClient.prefetchQuery(managedClubOptionsQueryOptions),
      queryClient.prefetchQuery(bridgeInstallQueryOptions),
      queryClient.prefetchQuery(bridgeStatusQueryOptions),
    ]),
  component: SettingsPage,
});

function SectionFallback({ label }: { label: string }) {
  return (
    <div className="flex min-h-40 items-center justify-center rounded-lg border border-outline-variant bg-surface-container text-body-md text-on-surface-variant">
      {label}
    </div>
  );
}

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

function SettingsPage() {
  const queryClient = useQueryClient();
  const invalidateCurrentContext = () => {
    void queryClient.invalidateQueries({ queryKey: searchKeys.all });
    void queryClient.invalidateQueries({ queryKey: playerKeys.all });
    void queryClient.invalidateQueries({ queryKey: managedClubKeys.all });
    void queryClient.invalidateQueries({ queryKey: plannerKeys.all });
    void queryClient.resetQueries({ queryKey: academyKeys.all });
    void queryClient.invalidateQueries({ queryKey: staffKeys.all });
  };

  return (
    <div className="space-y-gutter">
      <h1 className="text-headline-lg text-on-surface">Settings</h1>

      <section aria-labelledby="save-data-heading" className="space-y-3">
        <h2 className="text-title-lg text-on-surface" id="save-data-heading">
          Save data
        </h2>
        <Suspense fallback={<SectionFallback label="Loading save data…" />}>
          <SnapshotPanelsWithErrorBoundary
            onCurrentContextChanged={invalidateCurrentContext}
          />
        </Suspense>
      </section>

      <section
        aria-labelledby="managed-club-heading"
        className="space-y-3"
        id="managed-club"
      >
        <h2 className="text-title-lg text-on-surface" id="managed-club-heading">
          Managed club
        </h2>
        <ErrorBoundary
          fallback={({ error, reset }) => (
            <ManagedClubError error={error} reset={reset} />
          )}
        >
          <Suspense
            fallback={<SectionFallback label="Loading managed club…" />}
          >
            <ManagedClubPanel
              onSaved={() => {
                void queryClient.invalidateQueries({
                  queryKey: plannerKeys.all,
                });
                void queryClient.resetQueries({ queryKey: academyKeys.all });
                void queryClient.invalidateQueries({ queryKey: staffKeys.all });
              }}
            />
          </Suspense>
        </ErrorBoundary>
      </section>

      <section aria-labelledby="bridge-heading" className="space-y-3">
        <h2 className="text-title-lg text-on-surface" id="bridge-heading">
          Bridge
        </h2>
        <Suspense fallback={<SectionFallback label="Loading bridge status…" />}>
          <BridgeStatusPanelWithErrorBoundary />
        </Suspense>
      </section>
    </div>
  );
}
