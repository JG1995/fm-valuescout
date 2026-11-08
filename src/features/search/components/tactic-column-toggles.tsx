import { Button } from "@/components/ui/button/button";
import type { TacticColumnGroup } from "@/utils/tactic-ids";

export type TacticColumnToggle = {
  group: TacticColumnGroup;
  label: string;
  active: boolean;
  /** False when the group has nothing to add; active columns stay removable. */
  canAdd?: boolean;
};

type TacticColumnTogglesProps = {
  groups: readonly TacticColumnToggle[];
  disabled: boolean;
  onToggleGroup: (group: TacticColumnGroup) => void;
};

export function TacticColumnToggles({
  groups,
  disabled,
  onToggleGroup,
}: TacticColumnTogglesProps) {
  return (
    <>
      {groups.map(({ group, label, active, canAdd = true }) => (
        <Button
          key={group}
          variant={active ? "primary" : "secondary"}
          aria-pressed={active}
          disabled={disabled || (!active && !canAdd)}
          onClick={() => onToggleGroup(group)}
        >
          Add Tactic ({label})
        </Button>
      ))}
    </>
  );
}
