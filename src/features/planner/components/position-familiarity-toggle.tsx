import { Switch } from "@/components/ui/switch/switch";
import { Tooltip } from "@/components/ui/tooltip/tooltip";

/** Whether Optimize squads places players only in positions they can play. */
export function PositionFamiliarityToggle({
  checked,
  disabled,
  onChange,
}: {
  checked: boolean;
  disabled: boolean;
  onChange: (checked: boolean) => void;
}) {
  return (
    <Tooltip
      content={
        checked
          ? "Optimize squads places players only in positions they can play, and marks them down where they are less familiar."
          : "Optimize squads places players in any position, by role scores and preferred foot alone. Goalkeepers and outfield players never swap."
      }
    >
      {(tooltipId) => (
        <Switch
          label="Position familiarity"
          checked={checked}
          disabled={disabled}
          aria-describedby={tooltipId}
          onChange={onChange}
        />
      )}
    </Tooltip>
  );
}
