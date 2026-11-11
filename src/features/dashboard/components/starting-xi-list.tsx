import { Pin, TrendingDown, TriangleAlert } from "lucide-react";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import { cn } from "@/utils/cn";
import type {
  DashboardGraphics,
  DashboardLineupSlot,
} from "../types/dashboard";
import { orderedSlots, slotContext, slotLabel } from "../utils/slot-label";
import { Score } from "./dashboard-table";
import { Portrait } from "./portrait";

const secondaryLineClass =
  "block truncate text-[11px] leading-4 text-on-surface-variant";

function coverLine(slot: DashboardLineupSlot) {
  if (slot.unscoredPlannerPlayer !== null) {
    return `Planner pick ${slot.unscoredPlannerPlayer} has no score here`;
  }
  return slot.cover
    ? `Cover: ${slot.cover.name} ${slot.cover.score}`
    : "No cover";
}

/** The lowest incumbent score, when the XI's scores differ at all. */
function weakestScore(slots: readonly DashboardLineupSlot[]) {
  const scores = slots.flatMap((slot) =>
    slot.incumbent ? [slot.incumbent.score] : [],
  );
  const lowest = Math.min(...scores);
  return scores.some((score) => score > lowest) ? lowest : null;
}

/**
 * The club's XI by tactic slot. The rows share the list's height, and their
 * portraits grow with them. Each row selects its slot, which narrows the
 * recommended signings to it; selecting it again shows every slot.
 */
export function StartingXiList({
  slots,
  selectedLaneId,
  onSelectLane,
  graphics,
}: {
  slots: readonly DashboardLineupSlot[];
  selectedLaneId: string | null;
  onSelectLane: (laneId: string | null) => void;
  graphics: DashboardGraphics;
}) {
  const weakest = weakestScore(slots);
  return (
    <ul
      aria-label="Starting XI by tactic slot"
      className="flex flex-1 flex-col"
    >
      {orderedSlots(slots).map((slot) => {
        const label = slotLabel(slot);
        const selected = slot.laneId === selectedLaneId;
        return (
          <li
            key={slot.laneId}
            className="flex min-h-table-row-height-two-line flex-1 basis-0 [container-type:size]"
          >
            <button
              type="button"
              aria-pressed={selected}
              title={label.full}
              onClick={() => onSelectLane(selected ? null : slot.laneId)}
              className={cn(
                "grid w-full grid-cols-[6.5rem_auto_minmax(0,1fr)_auto] items-center gap-2 border-t border-outline-variant px-4 text-left transition-colors duration-150 ease-out hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary",
                selected && "bg-surface-container-high",
              )}
            >
              <span className="min-w-0">
                <span
                  className={cn(
                    "block truncate text-body-sm text-on-surface",
                    selected && "font-semibold",
                  )}
                >
                  {label.compact}
                </span>
                <span className={secondaryLineClass}>{slotContext(slot)}</span>
              </span>
              <Portrait className="size-[clamp(1.75rem,64cqh,3.5rem)] rounded-sm">
                {slot.incumbent
                  ? graphics.portrait(slot.incumbent.uid, slot.incumbent.name)
                  : null}
              </Portrait>
              <span className="min-w-0">
                <span className="flex min-w-0 items-center gap-1">
                  {slot.incumbent?.source === "planner" ? (
                    <>
                      <Pin
                        aria-hidden="true"
                        size={12}
                        strokeWidth={1.5}
                        className="shrink-0 text-on-surface-variant"
                      />
                      <span className="sr-only">Planner pick:</span>
                    </>
                  ) : null}
                  <span
                    className={cn(
                      "truncate text-body-sm",
                      slot.incumbent
                        ? "text-on-surface"
                        : "text-on-surface-variant",
                      selected && "font-semibold",
                    )}
                  >
                    {slot.incumbent?.name ?? "No player"}
                  </span>
                </span>
                <span className={secondaryLineClass}>{coverLine(slot)}</span>
              </span>
              <span className="flex shrink-0 items-center gap-2">
                {slot.incumbent === null ? (
                  <StatusChip tone="warning" icon={TriangleAlert}>
                    Gap
                  </StatusChip>
                ) : slot.incumbent.score === weakest ? (
                  <StatusChip tone="neutral" icon={TrendingDown}>
                    Weakest
                  </StatusChip>
                ) : null}
                <span className="w-7 text-right">
                  <Score
                    score={slot.incumbent?.score ?? null}
                    name={`${slot.incumbent?.name ?? "No player"} at ${label.compact}`}
                  />
                </span>
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
