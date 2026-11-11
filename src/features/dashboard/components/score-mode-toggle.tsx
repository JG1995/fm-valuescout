import type { DashboardScoreMode } from "../types/dashboard";

const MODES: readonly { mode: DashboardScoreMode; label: string }[] = [
  { mode: "general", label: "General" },
  { mode: "moneyball", label: "Moneyball" },
];

/** Switches every score on the dashboard between attribute and Moneyball scoring. */
export function ScoreModeToggle({
  mode,
  onChange,
}: {
  mode: DashboardScoreMode;
  onChange: (mode: DashboardScoreMode) => void;
}) {
  return (
    <fieldset className="inline-flex rounded-md border border-outline bg-surface-container-high p-0.5">
      <legend className="sr-only">Score mode</legend>
      {MODES.map((option) => (
        <button
          key={option.mode}
          type="button"
          aria-pressed={mode === option.mode}
          className={
            mode === option.mode
              ? "h-8 rounded-sm bg-primary px-3 text-label-md text-on-primary"
              : "h-8 rounded-sm px-3 text-label-md text-on-surface-variant hover:text-on-surface"
          }
          onClick={() => onChange(option.mode)}
        >
          {option.label}
        </button>
      ))}
    </fieldset>
  );
}
