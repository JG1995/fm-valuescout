import { useEffect, useId, useState } from "react";
import type { TacticLane, TacticOptions } from "../types/tactic";
import { TACTIC_PHASES, type TacticView } from "../utils/tactic-editor";
import type { PitchOrientation } from "../utils/tactic-pitch-geometry";
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
// lane. Owns no geometry, scoring, persistence, or mutation logic. This
// wrapper owns the single orientation source: one colocated
// matchMedia("(min-width: 2100px)") read with change subscription and
// cleanup. No dependency, no shared hook; the modal and the shared
// single-phase component never subscribe and stay portrait.
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
  const [landscape, setLandscape] = useState(
    () =>
      typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(min-width: 2100px)").matches,
  );

  useEffect(() => {
    if (
      typeof window === "undefined" ||
      typeof window.matchMedia !== "function"
    ) {
      return;
    }
    const query = window.matchMedia("(min-width: 2100px)");
    // Catch a crossing between the initial read and subscription.
    setLandscape(query.matches);
    const onChange = (event: MediaQueryListEvent) => {
      setLandscape(event.matches);
    };
    query.addEventListener("change", onChange);
    return () => {
      query.removeEventListener("change", onChange);
    };
  }, []);

  const orientation: PitchOrientation = landscape ? "landscape" : "portrait";
  // Both needs 640px for three adjacent collided pairs: 15% anchor gaps
  // must fit two 44px targets plus their 4px split. The workspace scroller
  // owns any overflow; keep the canvas vertical instead of shrinking targets.
  // Single-phase and role-reference sizing remain independent.
  const workspaceCanvasClassName =
    orientation === "portrait"
      ? view === "both"
        ? "mx-auto h-[680px] w-full min-w-[640px] max-w-[640px]"
        : "mx-auto h-[560px] w-full max-w-[520px]"
      : undefined;

  if (view !== "both") {
    return (
      <PlannerTacticPitch
        phase={view}
        lanes={lanes}
        options={options}
        orientation={orientation}
        selectedLaneId={selectedLaneId}
        highlightedLaneId={highlightedLaneId}
        onHighlight={onHighlight}
        onSelectLane={onSelectLane}
        canvasClassName={workspaceCanvasClassName}
      />
    );
  }

  const label = `${TACTIC_PHASES.ip.label} and ${TACTIC_PHASES.oop.label}`;
  const markers = pitchMarkersForView(view, lanes, orientation);

  return (
    <section className="space-y-2" aria-labelledby={headingId}>
      <div className="flex items-center justify-between gap-3">
        <h3
          id={headingId}
          className="text-body-md font-semibold text-on-surface"
        >
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
        orientation={orientation}
        selectedLaneId={selectedLaneId}
        highlightedLaneId={highlightedLaneId}
        linkedHintId={linkedHintId}
        onHighlight={onHighlight}
        onSelectLane={onSelectLane}
        canvasClassName={workspaceCanvasClassName}
      />
    </section>
  );
}
