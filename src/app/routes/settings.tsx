import { useQueryClient } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { Suspense } from "react";
import { academyKeys } from "@/features/academy/api/academy-keys";
import { managedClubKeys } from "@/features/managed-club/api/managed-club-keys";
import { bridgeInstallQueryOptions } from "@/features/memory-read/api/bridge-install-query-options";
import { bridgeStatusQueryOptions } from "@/features/memory-read/api/bridge-status-query-options";
import { BridgeStatusPanelWithErrorBoundary } from "@/features/memory-read/components/bridge-status-panel-with-error-boundary";
import { moneyballKeys } from "@/features/moneyball/api/moneyball-keys";
import { plannerKeys } from "@/features/planner/api/planner-keys";
import { playerKeys } from "@/features/player-profile/api/player-keys";
import { searchKeys } from "@/features/search/api/search-keys";
import { currentSnapshotQueryOptions } from "@/features/snapshot/api/current-snapshot-query-options";
import { savesQueryOptions } from "@/features/snapshot/api/saves-query-options";
import { SnapshotPanelsWithErrorBoundary } from "@/features/snapshot/components/snapshot-panels-with-error-boundary";
import { staffKeys } from "@/features/staff/api/staff-keys";

export const Route = createFileRoute("/settings")({
  beforeLoad: ({ location }) => {
    if (location.hash === "managed-club") {
      throw Route.redirect({
        to: "/my-club",
        hash: "managed-club",
        replace: true,
      });
    }
  },
  loader: ({ context: { queryClient } }) =>
    Promise.all([
      queryClient.prefetchQuery(savesQueryOptions),
      queryClient.prefetchQuery(currentSnapshotQueryOptions),
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

function SettingsPage() {
  const queryClient = useQueryClient();
  const invalidateCurrentContext = () => {
    void queryClient.invalidateQueries({ queryKey: searchKeys.all });
    void queryClient.invalidateQueries({ queryKey: playerKeys.all });
    void queryClient.invalidateQueries({ queryKey: moneyballKeys.all });
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
