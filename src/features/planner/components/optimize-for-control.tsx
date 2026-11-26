import { Tooltip } from "@/components/ui/tooltip/tooltip";
import type { OptimizeFor } from "../types/development";

const GOALS: readonly { goal: OptimizeFor; label: string }[] = [
  { goal: "strongest", label: "Strongest" },
  { goal: "development", label: "Development" },
];

/** Whether Optimize squads builds the strongest squads or keeps young players in their development slots. */
export function OptimizeForControl({
  optimizeFor,
  maxAge,
  disabled,
  onChange,
}: {
  optimizeFor: OptimizeFor;
  /** The oldest age with a development slot, once it has loaded. */
  maxAge: number | undefined;
  disabled: boolean;
  onChange: (optimizeFor: OptimizeFor) => void;
}) {
  const hints: Record<OptimizeFor, string> = {
    strongest: "Puts every player where the squads are strongest today.",
    development: `Gives ${maxAge === undefined ? "a young player" : `a player aged ${maxAge} or under`} only his development slot. He starts there if he earns it, and otherwise sits behind it.`,
  };
  return (
    <div className="flex items-center gap-2">
      <span className="text-label-md text-on-surface-variant">
        Optimize for
      </span>
      <fieldset className="inline-flex rounded-md border border-outline bg-surface-container-high p-0.5">
        <legend className="sr-only">Optimize for</legend>
        {GOALS.map((option) => (
          <Tooltip key={option.goal} content={hints[option.goal]}>
            <button
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
          </Tooltip>
        ))}
      </fieldset>
    </div>
  );
}
