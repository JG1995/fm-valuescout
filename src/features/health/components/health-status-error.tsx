import { CircleAlert } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";

type HealthStatusErrorProps = {
  error: Error;
  onRetry: () => void;
};

export function HealthStatusError({ error, onRetry }: HealthStatusErrorProps) {
  return (
    <Panel>
      <EmptyState
        icon={CircleAlert}
        title="Could not load health data"
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
