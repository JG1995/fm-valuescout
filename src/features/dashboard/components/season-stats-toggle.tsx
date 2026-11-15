import { useId } from "react";
import { cn } from "@/utils/cn";
import { formatCount } from "@/utils/format";
import type { DashboardScoreMode } from "../types/dashboard";

/**
 * Whether General mode's recommendations use season stats: candidates need a
 * minimum of competitive minutes, and slots' season-stat filters apply.
 * Moneyball scores always use them, and a snapshot without them cannot.
 */
export function SeasonStatsToggle({
  mode,
  checked,
  available,
  minimumMinutes,
  onChange,
}: {
  mode: DashboardScoreMode;
  checked: boolean;
  /** Whether the snapshot has season stats; unknown while the first read loads. */
  available: boolean | undefined;
  minimumMinutes: number | undefined;
  onChange: (checked: boolean) => void;
}) {
  const hintId = useId();
  const minutes =
    minimumMinutes === undefined
      ? "enough competitive minutes"
      : `${formatCount(minimumMinutes)}+ competitive minutes`;
  const hint =
    available === false
      ? "This snapshot has no season stats. Link your FM save in Settings, then use Load Data."
      : mode === "moneyball"
        ? `Moneyball scores always use season stats, so candidates need ${minutes} this season.`
        : `Candidates need ${minutes} this season, and slots' season-stat filters apply. Leagues further into their season reach that sooner.`;
  const locked = available === false || mode === "moneyball";

  return (
    <>
      <label
        className={cn(
          "flex items-center gap-2 text-body-md",
          locked ? "text-on-surface-variant" : "text-on-surface",
        )}
        title={hint}
      >
        <input
          type="checkbox"
          checked={available !== false && (mode === "moneyball" || checked)}
          disabled={locked}
          aria-describedby={hintId}
          onChange={(event) => onChange(event.target.checked)}
        />
        Season stats
      </label>
      {/* Outside the label, so the hint describes the checkbox without joining its name. */}
      <span id={hintId} className="sr-only">
        {hint}
      </span>
    </>
  );
}
