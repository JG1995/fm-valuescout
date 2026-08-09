import { Button } from "@/components/ui/button/button";
import type { PlannerScoreBasis } from "../api/optimize-planner-depth";

type PlannerOptimizerControlsProps = {
  pendingBasis: PlannerScoreBasis | null;
  disabled: boolean;
  onOptimize: (scoreBasis: PlannerScoreBasis) => void;
};

export function PlannerOptimizerControls({
  pendingBasis,
  disabled,
  onOptimize,
}: PlannerOptimizerControlsProps) {
  return (
    <div className="flex flex-wrap gap-2">
      <Button
        disabled={disabled}
        loading={pendingBasis === "current"}
        loadingLabel="Optimizing current…"
        onClick={() => onOptimize("current")}
      >
        Optimize squads
      </Button>
      <Button
        variant="secondary"
        disabled={disabled}
        loading={pendingBasis === "potential"}
        loadingLabel="Optimizing potential…"
        onClick={() => onOptimize("potential")}
      >
        Optimize by potential
      </Button>
    </div>
  );
}
