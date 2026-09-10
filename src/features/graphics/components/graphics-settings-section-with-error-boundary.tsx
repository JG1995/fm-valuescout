import { useQueryClient } from "@tanstack/react-query";
import { CircleAlert } from "lucide-react";
import { ErrorBoundary } from "@/components/error-boundary/error-boundary";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { graphicsKeys } from "../api/graphics-keys";
import { GraphicsSettingsSection } from "./graphics-settings-section";

function GraphicsSectionError({
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
        title="Could not load graphics data"
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

export function GraphicsSettingsSectionWithErrorBoundary() {
  const queryClient = useQueryClient();

  return (
    <ErrorBoundary
      fallback={({ error, reset }) => (
        <GraphicsSectionError
          error={error}
          onRetry={() => {
            queryClient.resetQueries({ queryKey: graphicsKeys.status() });
            reset();
          }}
        />
      )}
    >
      <GraphicsSettingsSection />
    </ErrorBoundary>
  );
}
