import { useQueryClient } from "@tanstack/react-query";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import { healthKeys } from "../api/health-keys";
import { setHealthSimulateError } from "../api/health-simulate-error";
import { HealthStatusError } from "./health-status-error";
import { HealthStatusPanel } from "./health-status-panel";

export function HealthStatusPanelWithErrorBoundary() {
  const queryClient = useQueryClient();

  return (
    <ErrorBoundary
      fallback={({ error, reset }) => (
        <HealthStatusError
          error={error}
          onRetry={() => {
            setHealthSimulateError(false);
            queryClient.resetQueries({
              queryKey: healthKeys.all,
            });
            reset();
          }}
        />
      )}
    >
      <HealthStatusPanel />
    </ErrorBoundary>
  );
}
