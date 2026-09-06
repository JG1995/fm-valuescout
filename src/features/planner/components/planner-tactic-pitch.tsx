import { useId } from "react";
import type { TacticLane, TacticOptions } from "../types/tactic";
import {
  canonicalPlacement,
  phaseDescription,
  phasePosition,
  phasePositionLabel,
  roleLabel,
  TACTIC_PHASES,
  type TacticPhase,
} from "../utils/tactic-editor";
import {
  comparePitchOrder,
  type PitchCoordinate,
  portraitCoordinateForPlacement,
  projectPosition,
} from "../utils/tactic-pitch-geometry";

type PlannerTacticPitchProps = {
  phase: TacticPhase;
  lanes: TacticLane[];
  options: TacticOptions;
  selectionHint?: string;
  selectedLaneId: string;
  highlightedLaneId: string | null;
  onHighlight: (laneId: string | null) => void;
  onSelectLane: (laneId: string) => void;
};

function LaneButton({
  phase,
  lane,
  lanes,
  options,
  highlightedLaneId,
  linkedHintId,
  selected,
  onHighlight,
  onSelect,
}: {
  phase: TacticPhase;
  lane: TacticLane;
  lanes: TacticLane[];
  options: TacticOptions;
  highlightedLaneId: string | null;
  linkedHintId: string;
  selected: boolean;
  onHighlight: (laneId: string | null) => void;
  onSelect: () => void;
}) {
  const position = phasePositionLabel(lane, phase, lanes);
  const role = roleLabel(lane, phase, options);
  const description = phaseDescription(lane, phase, lanes, options);
  const { shortLabel } = TACTIC_PHASES[phase];
  const highlighted = highlightedLaneId === lane.laneId;

  return (
    <button
      type="button"
      aria-label={`${shortLabel}: ${description}`}
      aria-describedby={linkedHintId}
      aria-pressed={selected}
      className={`min-h-11 w-full rounded-md border px-1 py-1 text-center transition-[background-color,border-color,box-shadow] duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
        selected
          ? "border-primary bg-primary-container text-primary ring-2 ring-primary/60"
          : highlighted
            ? "border-primary bg-surface-container-high text-on-surface ring-2 ring-primary/60"
            : "border-outline-variant bg-surface-container text-on-surface hover:bg-surface-container-high"
      }`}
      onBlur={() => {
        if (!selected) {
          onHighlight(null);
        }
      }}
      onClick={onSelect}
      onFocus={() => onHighlight(lane.laneId)}
      onMouseEnter={() => onHighlight(lane.laneId)}
      onMouseLeave={() => {
        if (!selected) {
          onHighlight(null);
        }
      }}
    >
      <span className="block truncate text-label-md" title={description}>
        {position}
      </span>
      <span className="block truncate text-[11px]" title={description}>
        {role}
      </span>
    </button>
  );
}

type PitchMarker = {
  lane: TacticLane;
  placement: string;
  coordinate: PitchCoordinate;
};

function pitchMarkers(phase: TacticPhase, lanes: TacticLane[]): PitchMarker[] {
  const markers: PitchMarker[] = [];
  for (const lane of lanes) {
    const placement = canonicalPlacement(phasePosition(lane, phase));
    const coordinate = portraitCoordinateForPlacement(placement);
    if (!coordinate) {
      continue;
    }
    markers.push({
      lane,
      placement,
      coordinate: projectPosition(coordinate, "portrait"),
    });
  }
  // Unique placements have distinct coordinates, so this order is total and
  // matches the visual attack-to-goalkeeper, left-to-right reading order.
  return markers.sort((left, right) =>
    comparePitchOrder(left.coordinate, right.coordinate),
  );
}

export function PlannerTacticPitch({
  phase,
  lanes,
  options,
  selectionHint = "Focus or select this position to highlight its linked counterpart in the other phase.",
  selectedLaneId,
  highlightedLaneId,
  onHighlight,
  onSelectLane,
}: PlannerTacticPitchProps) {
  const { label } = TACTIC_PHASES[phase];
  const selectedLane = lanes.find((lane) => lane.laneId === selectedLaneId);
  const headingId = useId();
  const linkedHintId = useId();
  const attackDescriptionId = useId();
  const markers = pitchMarkers(phase, lanes);

  return (
    <section className="space-y-2" aria-labelledby={headingId}>
      <div className="flex items-center justify-between gap-3">
        <h3 id={headingId} className="text-headline-sm text-on-surface">
          {label}
        </h3>
        <span className="shrink-0 rounded-full bg-surface-container-high px-2 py-1 font-mono text-mono-sm text-on-surface-variant">
          {selectedLane
            ? phaseDescription(selectedLane, phase, lanes, options)
            : "Select a position"}
        </span>
      </div>
      <p id={linkedHintId} className="sr-only">
        {selectionHint}
      </p>
      <fieldset
        aria-describedby={attackDescriptionId}
        className="rounded-lg border border-outline-variant bg-surface-container-lowest p-3"
      >
        <legend className="sr-only">{label} pitch</legend>
        <p
          className="flex items-center gap-1 pb-2 text-label-md text-on-surface-variant"
          id={attackDescriptionId}
        >
          <span aria-hidden="true">↑</span> Attack toward the top
        </p>
        <div className="relative h-[420px] w-full overflow-hidden rounded-md border border-outline-variant bg-surface-container-high">
          <svg
            aria-hidden="true"
            className="pointer-events-none absolute inset-0 h-full w-full text-outline-variant"
            fill="none"
            preserveAspectRatio="none"
            stroke="currentColor"
            strokeWidth={0.5}
            viewBox="0 0 100 100"
          >
            <rect height="96" width="96" x="2" y="2" />
            <line x1="2" x2="98" y1="50" y2="50" />
            <circle cx="50" cy="50" r="8" />
            <rect height="12" width="30" x="35" y="2" />
            <rect height="12" width="30" x="35" y="86" />
          </svg>
          {markers.map(({ lane, placement, coordinate }) => (
            <div
              className="absolute w-[12%] min-w-11 -translate-x-1/2 -translate-y-1/2"
              data-pitch-marker={lane.laneId}
              data-placement={placement}
              key={lane.laneId}
              style={{
                left: `${Math.round(coordinate.x * 100)}%`,
                top: `${Math.round(coordinate.y * 100)}%`,
              }}
            >
              <LaneButton
                phase={phase}
                lane={lane}
                lanes={lanes}
                options={options}
                highlightedLaneId={highlightedLaneId}
                linkedHintId={linkedHintId}
                selected={lane.laneId === selectedLaneId}
                onHighlight={onHighlight}
                onSelect={() => onSelectLane(lane.laneId)}
              />
            </div>
          ))}
        </div>
      </fieldset>
    </section>
  );
}
