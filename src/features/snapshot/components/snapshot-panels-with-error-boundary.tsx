import { useQueryClient } from "@tanstack/react-query";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import { snapshotKeys } from "../api/snapshot-keys";
import { SaveSwitcher } from "./save-switcher";
import { SnapshotOverviewPanel } from "./snapshot-overview-panel";

function SnapshotSectionError({
  error,
  onRetry,
}: {
  error: Error;
  onRetry: () => void;
}) {
  return (
    <div className="space-y-2 rounded-md border border-on-background/20 p-4">
      <p className="text-on-background/80">
        Could not load snapshot data.{" "}
        <span className="text-on-background">{error.message}</span>
      </p>
      <button
        type="button"
        className="rounded-md border border-on-background/20 px-3 py-2 text-on-background"
        onClick={onRetry}
      >
        Retry
      </button>
    </div>
  );
}

export function SnapshotPanelsWithErrorBoundary() {
  const queryClient = useQueryClient();

  return (
    <div className="space-y-4">
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
        <SaveSwitcher />
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
