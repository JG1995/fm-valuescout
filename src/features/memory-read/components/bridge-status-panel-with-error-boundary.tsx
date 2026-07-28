import { useQueryClient } from "@tanstack/react-query";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import { bridgeStatusKeys } from "../api/bridge-status-keys";
import { BridgeStatusError } from "./bridge-status-error";
import { BridgeStatusPanel } from "./bridge-status-panel";

export function BridgeStatusPanelWithErrorBoundary() {
  const queryClient = useQueryClient();

  return (
    <ErrorBoundary
      fallback={({ error, reset }) => (
        <BridgeStatusError
          error={error}
          onRetry={() => {
            queryClient.resetQueries({
              queryKey: bridgeStatusKeys.all,
            });
            reset();
          }}
        />
      )}
    >
      <BridgeStatusPanel />
    </ErrorBoundary>
  );
}
