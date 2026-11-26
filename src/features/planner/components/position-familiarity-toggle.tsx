import { Tooltip } from "@/components/ui/tooltip/tooltip";
import { cn } from "@/utils/cn";

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
        <label
          className={cn(
            "flex items-center gap-2 text-body-md",
            disabled ? "text-on-surface-variant" : "text-on-surface",
          )}
        >
          <input
            type="checkbox"
            checked={checked}
            disabled={disabled}
            aria-describedby={tooltipId}
            onChange={(event) => onChange(event.target.checked)}
          />
          Position familiarity
        </label>
      )}
    </Tooltip>
  );
}
