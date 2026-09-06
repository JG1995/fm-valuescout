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
  type TacticView,
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
  dual = false,
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
  dual?: boolean;
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
      } ${dual && phase === "oop" ? "border-dashed" : ""}`}
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
      {dual ? (
        <span
          aria-hidden="true"
          className="mx-auto mb-0.5 block w-fit rounded-full bg-surface-container-highest px-1 font-mono text-mono-sm text-on-surface-variant"
        >
          {shortLabel}
        </span>
      ) : null}
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
  key: string;
  lane: TacticLane;
  phase: TacticPhase;
  placement: string;
  coordinate: PitchCoordinate;
  collides: boolean;
};

function toMarker(lane: TacticLane, phase: TacticPhase): PitchMarker | null {
  const placement = canonicalPlacement(phasePosition(lane, phase));
  const coordinate = portraitCoordinateForPlacement(placement);
  if (!coordinate) {
    return null;
  }
  return {
    key: `${lane.laneId}:${phase}`,
    lane,
    phase,
    placement,
    coordinate: projectPosition(coordinate, "portrait"),
    collides: false,
  };
}

export function pitchMarkersForView(
  view: TacticView,
  lanes: TacticLane[],
): PitchMarker[] {
  const phases: TacticPhase[] = view === "both" ? ["ip", "oop"] : [view];
  const markers: PitchMarker[] = [];
  for (const lane of lanes) {
    for (const phase of phases) {
      const marker = toMarker(lane, phase);
      if (marker) {
        markers.push(marker);
      }
    }
  }
  // Unique placements have distinct coordinates, so this order is total and
  // matches the visual attack-to-goalkeeper, left-to-right reading order.
  // A shared coordinate always pairs one IP and one OOP marker; the phase
  // tie-break keeps IP before OOP in the DOM so tab order matches the
  // IP-left/OOP-right split regardless of lane iteration order.
  const ordered = markers.sort((left, right) =>
    comparePitchOrder(left.coordinate, right.coordinate) === 0
      ? left.phase === right.phase
        ? 0
        : left.phase === "ip"
          ? -1
          : 1
      : comparePitchOrder(left.coordinate, right.coordinate),
  );
  // A lane whose IP and OOP placements share a coordinate would stack two
  // full buttons on one point; split that pair around the normalized point
  // so both phase markers stay visible and clickable. Placements are unique
  // per phase, so a shared coordinate always pairs one IP and one OOP marker.
  const counts = new Map<string, number>();
  for (const marker of ordered) {
    const at = `${marker.coordinate.x}/${marker.coordinate.y}`;
    counts.set(at, (counts.get(at) ?? 0) + 1);
  }
  return ordered.map((marker) => ({
    ...marker,
    collides:
      (counts.get(`${marker.coordinate.x}/${marker.coordinate.y}`) ?? 0) > 1,
  }));
}

function pitchMarkers(phase: TacticPhase, lanes: TacticLane[]): PitchMarker[] {
  return pitchMarkersForView(phase, lanes);
}

export function TacticPitchCanvas({
  legend,
  markers,
  lanes,
  options,
  dual = false,
  selectedLaneId,
  highlightedLaneId,
  linkedHintId,
  onHighlight,
  onSelectLane,
}: {
  legend: string;
  markers: PitchMarker[];
  lanes: TacticLane[];
  options: TacticOptions;
  dual?: boolean;
  selectedLaneId: string;
  highlightedLaneId: string | null;
  linkedHintId: string;
  onHighlight: (laneId: string | null) => void;
  onSelectLane: (laneId: string) => void;
}) {
  const attackDescriptionId = useId();
  return (
    <fieldset
      aria-describedby={attackDescriptionId}
      className="rounded-lg border border-outline-variant bg-surface-container-lowest p-3"
    >
      <legend className="sr-only">{legend}</legend>
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
        {markers.map((marker) => (
          <div
            className={`absolute min-w-11 -translate-y-1/2 ${dual ? "w-[6%]" : "w-[12%]"} ${
              marker.collides
                ? marker.phase === "ip"
                  ? "-translate-x-[calc(100%+2px)]"
                  : "translate-x-[2px]"
                : "-translate-x-1/2"
            }`}
            data-pitch-marker={marker.lane.laneId}
            data-phase={marker.phase}
            data-placement={marker.placement}
            key={marker.key}
            style={{
              left: `${Math.round(marker.coordinate.x * 100)}%`,
              top: `${Math.round(marker.coordinate.y * 100)}%`,
            }}
          >
            <LaneButton
              phase={marker.phase}
              lane={marker.lane}
              lanes={lanes}
              options={options}
              highlightedLaneId={highlightedLaneId}
              linkedHintId={linkedHintId}
              selected={marker.lane.laneId === selectedLaneId}
              dual={dual}
              onHighlight={onHighlight}
              onSelect={() => onSelectLane(marker.lane.laneId)}
            />
          </div>
        ))}
      </div>
    </fieldset>
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
      <TacticPitchCanvas
        legend={`${label} pitch`}
        markers={markers}
        lanes={lanes}
        options={options}
        selectedLaneId={selectedLaneId}
        highlightedLaneId={highlightedLaneId}
        linkedHintId={linkedHintId}
        onHighlight={onHighlight}
        onSelectLane={onSelectLane}
      />
    </section>
  );
}
