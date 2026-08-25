import { useQueryClient } from "@tanstack/react-query";
import { CircleAlert } from "lucide-react";
import { Suspense } from "react";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import type { ClearPlayerResultContext } from "@/components/player-table/player-result-context";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { snapshotKeys } from "../api/snapshot-keys";
import { SaveSwitcher } from "./save-switcher";
import { SnapshotHistoryPanel } from "./snapshot-history-panel";
import { SnapshotOverviewPanel } from "./snapshot-overview-panel";

function SnapshotSectionError({
  error,
  onRetry,
}: {
  error: Error;
  onRetry: () => void;
}) {
  return (
    <Panel>
      <EmptyState
        icon={CircleAlert}
        title="Could not load snapshot data"
        action={
          <Button variant="secondary" onClick={onRetry}>
            Retry
          </Button>
        }
      >
        {error.message}
      </EmptyState>
    </Panel>
  );
}

type SnapshotPanelsWithErrorBoundaryProps = {
  onCurrentContextChanged?: () => void;
  onBeforeContextChange: ClearPlayerResultContext;
};

export function SnapshotPanelsWithErrorBoundary({
  onCurrentContextChanged,
  onBeforeContextChange,
}: SnapshotPanelsWithErrorBoundaryProps) {
  const queryClient = useQueryClient();

  return (
    <div className="space-y-gutter">
      <ErrorBoundary
        fallback={({ error, reset }) => (
          <SnapshotSectionError
            error={error}
            onRetry={() => {
              queryClient.resetQueries({ queryKey: snapshotKeys.saves() });
              reset();
            }}
          />
        )}
      >
        <SaveSwitcher
          onBeforeContextChange={onBeforeContextChange}
          onCurrentContextChanged={onCurrentContextChanged}
        />
      </ErrorBoundary>
      <ErrorBoundary
        fallback={({ error, reset }) => (
          <SnapshotSectionError
            error={error}
            onRetry={() => {
              queryClient.resetQueries({ queryKey: snapshotKeys.all });
              reset();
            }}
          />
        )}
      >
        <Suspense
          fallback={
            <Panel title="Snapshot history">
              <p className="text-body-md text-on-surface-variant">
                Loading snapshot history…
              </p>
            </Panel>
          }
        >
          <SnapshotHistoryPanel
            onBeforeContextChange={onBeforeContextChange}
            onCurrentContextChanged={onCurrentContextChanged}
          />
        </Suspense>
      </ErrorBoundary>
      <ErrorBoundary
        fallback={({ error, reset }) => (
          <SnapshotSectionError
            error={error}
            onRetry={() => {
              queryClient.resetQueries({ queryKey: snapshotKeys.all });
              reset();
            }}
          />
        )}
      >
        <SnapshotOverviewPanel />
      </ErrorBoundary>
    </div>
  );
}
