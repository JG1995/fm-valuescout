import { Button } from "@/components/ui/button/button";

type HealthStatusErrorProps = {
  error: Error;
  onRetry: () => void;
};

export function HealthStatusError({ error, onRetry }: HealthStatusErrorProps) {
  return (
    <div className="space-y-3">
      <p className="text-on-background/80">
        Could not load health data.{" "}
        <span className="text-on-background">{error.message}</span>
      </p>
      <Button type="button" variant="secondary" onClick={onRetry}>
        Retry
      </Button>
    </div>
  );
}
