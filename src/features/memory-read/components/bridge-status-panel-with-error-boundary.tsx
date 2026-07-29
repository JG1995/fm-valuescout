import { useQueryClient } from "@tanstack/react-query";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import { bridgeInstallKeys } from "../api/bridge-install-keys";
import { bridgeStatusKeys } from "../api/bridge-status-keys";
import { BridgePluginInstallError } from "./bridge-plugin-install-error";
import { BridgePluginInstallSection } from "./bridge-plugin-install-section";
import { BridgeStatusError } from "./bridge-status-error";
import { BridgeStatusPanel } from "./bridge-status-panel";

export function BridgeStatusPanelWithErrorBoundary({
  activeSaveId,
  onLoadDataSettled,
}: {
  activeSaveId?: number;
  onLoadDataSettled?: () => void;
}) {
  const queryClient = useQueryClient();

  return (
    <div className="space-y-4">
      <ErrorBoundary
        fallback={({ error, reset }) => (
          <BridgePluginInstallError
            error={error}
            onRetry={() => {
              queryClient.resetQueries({
                queryKey: bridgeInstallKeys.all,
              });
              reset();
            }}
          />
        )}
      >
        <BridgePluginInstallSection />
      </ErrorBoundary>
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
        <BridgeStatusPanel
          activeSaveId={activeSaveId}
          onLoadDataSettled={onLoadDataSettled}
        />
      </ErrorBoundary>
    </div>
  );
}
