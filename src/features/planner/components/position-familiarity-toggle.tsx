import { useId } from "react";
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
  const hintId = useId();

  return (
    <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
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
          aria-describedby={hintId}
          onChange={(event) => onChange(event.target.checked)}
        />
        Position familiarity
      </label>
      <span id={hintId} className="text-body-sm text-on-surface-variant">
        {checked
          ? "Optimize squads places players only in positions they can play, and marks them down where they are less familiar."
          : "Optimize squads places players in any position, by role scores and preferred foot alone. Goalkeepers and outfield players never swap."}
      </span>
    </div>
  );
}
