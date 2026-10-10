import { useId } from "react";
import type { TacticLane, TacticOptions } from "../types/tactic";
import {
  linkedPositionDescription,
  orderedTacticLanes,
} from "../utils/tactic-editor";

type PlannerTacticLaneListProps = {
  lanes: TacticLane[];
  options: TacticOptions;
  selectedLaneId: string;
  onSelectLane: (laneId: string) => void;
};

// Selection-only Tactical XI panel. Rows are ordinary buttons sharing the
// editor's selectedLaneId: activating a row or a pitch marker writes the
// same id, so the pitch, inspector, and highlight state follow unchanged.
// Owns no state, scoring, or analytics.
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
      <h3 id={headingId} className="text-body-md font-semibold text-on-surface">
        Tactical XI
      </h3>
      <ul className="min-h-0 flex-1 space-y-1 overflow-y-auto p-1">
        {orderedTacticLanes(lanes).map((lane) => {
          const selected = lane.laneId === selectedLaneId;
          return (
            <li key={lane.laneId}>
              <button
                type="button"
                aria-pressed={selected}
                onClick={() => onSelectLane(lane.laneId)}
                className={`block w-full scroll-m-1 whitespace-pre-line cursor-pointer rounded-md border border-transparent px-2 py-1.5 text-left text-body-sm transition-[background-color,border-color,box-shadow] duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
                  selected
                    ? "bg-surface-container-high font-semibold text-on-surface"
                    : "text-on-surface hover:bg-surface-container-high"
                }`}
              >
                {linkedPositionDescription(lane, lanes, options).replace(
                  " / ",
                  "\n",
                )}
              </button>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
