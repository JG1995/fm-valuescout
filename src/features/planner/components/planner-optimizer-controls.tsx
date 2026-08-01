import { Button } from "@/components/ui/button/button";

type PlannerOptimizerControlsProps = {
  pending: boolean;
  onOptimize: () => void;
};

export function PlannerOptimizerControls({
  pending,
  onOptimize,
}: PlannerOptimizerControlsProps) {
  return (
    <Button
      disabled={pending}
      loading={pending}
      loadingLabel="Optimizing…"
      onClick={onOptimize}
    >
      Optimize squads
    </Button>
  );
}
