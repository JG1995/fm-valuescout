import { ChartColumn, ListFilter } from "lucide-react";
import { useId } from "react";
import type { TacticLane, TacticOptions } from "../types/tactic";
import {
  canonicalPlacement,
  linkedPositionDescription,
  orderedTacticLanes,
  roleLabel,
} from "../utils/tactic-editor";

type PlannerTacticLaneListProps = {
  lanes: TacticLane[];
  options: TacticOptions;
  selectedLaneId: string;
  onSelectLane: (laneId: string) => void;
};

function plural(count: number, noun: string) {
  return `${count} ${noun}${count === 1 ? "" : "s"}`;
}

/** A slot's custom-metric or recruitment-filter count, shown only when set. */
function SlotCount({
  count,
  icon: Icon,
  label,
}: {
  count: number;
  icon: typeof ListFilter;
  label: string;
}) {
  if (count === 0) {
    return null;
  }
  return (
    <span
      className="inline-flex items-center gap-0.5 font-mono text-mono-sm text-on-surface-variant tabular-nums"
      title={label}
    >
      <Icon aria-hidden="true" size={12} strokeWidth={2} />
      {count}
    </span>
  );
}

// The Tactical XI: one readable row per slot with both phases' positions
// and roles, plus the slot's custom-metric and recruitment-filter counts.
// Rows are buttons sharing the editor's selectedLaneId with the pitch.
export function PlannerTacticLaneList({
  lanes,
  options,
  selectedLaneId,
  onSelectLane,
}: PlannerTacticLaneListProps) {
  const headingId = useId();

  return (
    <section
      className="flex min-h-0 min-w-0 flex-col gap-2 border-r border-outline-variant pr-4"
      aria-labelledby={headingId}
    >
      <h2 id={headingId} className="text-body-md font-semibold text-on-surface">
        Tactical XI
      </h2>
      <ul className="min-h-0 flex-1 space-y-0.5 overflow-y-auto p-1">
        {orderedTacticLanes(lanes).map((lane) => {
          const selected = lane.laneId === selectedLaneId;
          const ipPlacement = canonicalPlacement(lane.ipPosition);
          const oopPlacement = canonicalPlacement(lane.oopPosition);
          const metrics = plural(lane.customMetricKeys.length, "custom metric");
          const filters = plural(
            lane.recruitmentFilters.length,
            "recruitment filter",
          );
          const counts = [
            lane.customMetricKeys.length > 0 ? metrics : null,
            lane.recruitmentFilters.length > 0 ? filters : null,
          ].filter((count) => count !== null);
          return (
            <li key={lane.laneId}>
              <button
                type="button"
                aria-pressed={selected}
                aria-label={[
                  linkedPositionDescription(lane, lanes, options),
                  ...counts,
                ].join(", ")}
                title={linkedPositionDescription(lane, lanes, options)}
                onClick={() => onSelectLane(lane.laneId)}
                className={`grid w-full scroll-m-1 cursor-pointer grid-cols-[3rem_minmax(0,1fr)_auto] items-start gap-x-2 rounded-md border border-transparent px-2 py-1.5 text-left transition-[background-color,border-color,box-shadow] duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
                  selected
                    ? "bg-surface-container-high"
                    : "hover:bg-surface-container-high"
                }`}
              >
                <span className="flex flex-col font-mono text-mono-sm text-on-surface">
                  <span className={selected ? "font-semibold" : undefined}>
                    {ipPlacement}
                  </span>
                  {oopPlacement === ipPlacement ? null : (
                    <span className="text-on-surface-variant">
                      {oopPlacement}
                    </span>
                  )}
                </span>
                <span className="min-w-0 text-body-sm">
                  <span className="flex items-center gap-1.5 text-on-surface">
                    <span
                      aria-hidden="true"
                      className="inline-block size-1.5 shrink-0 rounded-full bg-chart-2"
                    />
                    <span
                      className={`truncate ${selected ? "font-semibold" : ""}`}
                    >
                      {roleLabel(lane, "ip", options)}
                    </span>
                  </span>
                  <span className="flex items-center gap-1.5 text-on-surface-variant">
                    <span
                      aria-hidden="true"
                      className="inline-block size-1.5 shrink-0 rounded-full bg-chart-3"
                    />
                    <span className="truncate">
                      {roleLabel(lane, "oop", options)}
                    </span>
                  </span>
                </span>
                <span className="flex flex-col items-end gap-0.5 pt-0.5">
                  <SlotCount
                    count={lane.customMetricKeys.length}
                    icon={ChartColumn}
                    label={metrics}
                  />
                  <SlotCount
                    count={lane.recruitmentFilters.length}
                    icon={ListFilter}
                    label={filters}
                  />
                </span>
              </button>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
