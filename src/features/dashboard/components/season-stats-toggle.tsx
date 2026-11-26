import { Tooltip } from "@/components/ui/tooltip/tooltip";
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
  const minutes =
    minimumMinutes === undefined
      ? "enough competitive minutes"
      : `${formatCount(minimumMinutes)}+ competitive minutes`;
  const hint =
    available === false
      ? "No season stats in this snapshot. Link your FM save in Settings, then use Load Data."
      : mode === "moneyball"
        ? `Moneyball scores always use season stats, so candidates need ${minutes} this season.`
        : `Candidates need ${minutes} this season, and slots' season-stat filters apply.`;
  const locked = available === false || mode === "moneyball";

  return (
    <Tooltip content={hint}>
      {(tooltipId) => (
        <label
          className={cn(
            "flex items-center gap-2 text-body-md",
            locked ? "text-on-surface-variant" : "text-on-surface",
          )}
        >
          <input
            type="checkbox"
            checked={available !== false && (mode === "moneyball" || checked)}
            disabled={locked}
            aria-describedby={tooltipId}
            onChange={(event) => onChange(event.target.checked)}
          />
          Season stats
        </label>
      )}
    </Tooltip>
  );
}
