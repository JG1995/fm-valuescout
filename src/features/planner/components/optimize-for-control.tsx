import type { OptimizeFor } from "../types/development";

const GOALS: readonly { goal: OptimizeFor; label: string }[] = [
  { goal: "strongest", label: "Strongest" },
  { goal: "development", label: "Development" },
];

/** Whether Optimize squads builds the strongest squads or keeps young players in their development slots. */
export function OptimizeForControl({
  optimizeFor,
  disabled,
  onChange,
}: {
  optimizeFor: OptimizeFor;
  disabled: boolean;
  onChange: (optimizeFor: OptimizeFor) => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <span className="text-label-md text-on-surface-variant">
        Optimize for
      </span>
      <fieldset className="inline-flex rounded-md border border-outline bg-surface-container-high p-0.5">
        <legend className="sr-only">Optimize for</legend>
        {GOALS.map((option) => (
          <button
            key={option.goal}
            type="button"
            aria-pressed={optimizeFor === option.goal}
            disabled={disabled}
            className={
              optimizeFor === option.goal
                ? "h-8 rounded-sm bg-primary px-3 text-label-md text-on-primary"
                : "h-8 rounded-sm px-3 text-label-md text-on-surface-variant hover:text-on-surface disabled:hover:text-on-surface-variant"
            }
            onClick={() => {
              if (optimizeFor !== option.goal) {
                onChange(option.goal);
              }
            }}
          >
            {option.label}
          </button>
        ))}
      </fieldset>
    </div>
  );
}
