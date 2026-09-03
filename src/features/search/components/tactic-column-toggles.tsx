import { Button } from "@/components/ui/button/button";
import type { TacticColumnGroup } from "@/utils/tactic-ids";

type TacticColumnTogglesProps = {
  currentActive: boolean;
  potentialActive: boolean;
  disabled: boolean;
  onToggleGroup: (group: TacticColumnGroup) => void;
};

export function TacticColumnToggles({
  currentActive,
  potentialActive,
  disabled,
  onToggleGroup,
}: TacticColumnTogglesProps) {
  return (
    <>
      <Button
        variant={currentActive ? "primary" : "secondary"}
        aria-pressed={currentActive}
        disabled={disabled}
        onClick={() => onToggleGroup("current")}
      >
        Add Tactic (Current)
      </Button>
      <Button
        variant={potentialActive ? "primary" : "secondary"}
        aria-pressed={potentialActive}
        disabled={disabled}
        onClick={() => onToggleGroup("potential")}
      >
        Add Tactic (Potential)
      </Button>
    </>
  );
}
