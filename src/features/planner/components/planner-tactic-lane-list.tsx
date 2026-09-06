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
      className="w-full shrink-0 space-y-2 rounded-lg border border-outline-variant bg-surface-container-high p-3 2xl:w-64"
      aria-labelledby={headingId}
    >
      <h3 id={headingId} className="text-headline-sm text-on-surface">
        Tactical XI
      </h3>
      <ul className="space-y-1">
        {orderedTacticLanes(lanes).map((lane) => {
          const selected = lane.laneId === selectedLaneId;
          return (
            <li key={lane.laneId}>
              <button
                type="button"
                aria-pressed={selected}
                onClick={() => onSelectLane(lane.laneId)}
                className={`block w-full cursor-pointer rounded-md border px-2 py-1.5 text-left text-body-sm transition-[background-color,border-color,box-shadow] duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
                  selected
                    ? "border-primary bg-primary-container text-primary ring-2 ring-primary/60"
                    : "border-outline-variant bg-surface-container text-on-surface hover:bg-surface-container-high"
                }`}
              >
                {linkedPositionDescription(lane, lanes, options)}
              </button>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
