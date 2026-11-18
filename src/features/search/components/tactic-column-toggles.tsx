import { Button } from "@/components/ui/button/button";
import type { TacticColumnGroup } from "@/utils/tactic-ids";

export type TacticColumnToggle = {
  group: TacticColumnGroup;
  label: string;
  active: boolean;
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
      {groups.map(({ group, label, active }) => (
        <Button
          key={group}
          variant={active ? "primary" : "secondary"}
          aria-pressed={active}
          disabled={disabled}
          onClick={() => onToggleGroup(group)}
        >
          Add Tactic ({label})
        </Button>
      ))}
    </>
  );
}
