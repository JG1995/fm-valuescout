import { useId } from "react";
import type { TacticLane, TacticOptions } from "../types/tactic";
import { TACTIC_PHASES, type TacticView } from "../utils/tactic-editor";
import {
  PlannerTacticPitch,
  pitchMarkersForView,
  TacticPitchCanvas,
} from "./planner-tactic-pitch";

type PlannerPhaseAwareTacticPitchProps = {
  view: TacticView;
  lanes: TacticLane[];
  options: TacticOptions;
  selectedLaneId: string;
  highlightedLaneId: string | null;
  onHighlight: (laneId: string | null) => void;
  onSelectLane: (laneId: string) => void;
};

// Workspace-only wrapper: one canvas per view. Single-phase views reuse the
// shared portrait component (the same path the role-reference modal keeps);
// Both composes the shared canvas with two phase-distinguished markers per
// lane. Owns no geometry, scoring, persistence, or mutation logic.
export function PlannerPhaseAwareTacticPitch({
  view,
  lanes,
  options,
  selectedLaneId,
  highlightedLaneId,
  onHighlight,
  onSelectLane,
}: PlannerPhaseAwareTacticPitchProps) {
  const headingId = useId();
  const linkedHintId = useId();

  if (view !== "both") {
    return (
      <PlannerTacticPitch
        phase={view}
        lanes={lanes}
        options={options}
        selectedLaneId={selectedLaneId}
        highlightedLaneId={highlightedLaneId}
        onHighlight={onHighlight}
        onSelectLane={onSelectLane}
      />
    );
  }

  const label = `${TACTIC_PHASES.ip.label} and ${TACTIC_PHASES.oop.label}`;
  const markers = pitchMarkersForView(view, lanes);

  return (
    <section className="space-y-2" aria-labelledby={headingId}>
      <div className="flex items-center justify-between gap-3">
        <h3 id={headingId} className="text-headline-sm text-on-surface">
          {label}
        </h3>
      </div>
      <p id={linkedHintId} className="sr-only">
        Focus or select this position to highlight its linked counterpart in the
        other phase.
      </p>
      <TacticPitchCanvas
        legend={`${label} pitch`}
        markers={markers}
        lanes={lanes}
        options={options}
        dual
        selectedLaneId={selectedLaneId}
        highlightedLaneId={highlightedLaneId}
        linkedHintId={linkedHintId}
        onHighlight={onHighlight}
        onSelectLane={onSelectLane}
      />
    </section>
  );
}
