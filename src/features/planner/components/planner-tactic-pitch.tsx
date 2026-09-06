import { useId } from "react";
import type { TacticLane, TacticOptions } from "../types/tactic";
import {
  canonicalPlacement,
  linkedPositionDescription,
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
  type PitchOrientation,
  portraitCoordinateForPlacement,
  projectPosition,
} from "../utils/tactic-pitch-geometry";

type PlannerTacticPitchProps = {
  phase: TacticPhase;
  lanes: TacticLane[];
  options: TacticOptions;
  orientation?: PitchOrientation;
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
  // Categorical phase accents (chart steel for IP, chart magenta for OOP),
  // never success/error semantics. Borders are opaque so the edge clears
  // 3:1 against both adjacent surfaces; text stays on-surface so small
  // type keeps its contrast. The dual badge always renders, so phase
  // identity stays visible under the gold selected treatment.
  const phaseBorder = phase === "ip" ? "border-chart-2" : "border-chart-3";
  const phaseBadge = phase === "ip" ? "border-chart-2" : "border-chart-3";

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
            ? `${phaseBorder} bg-surface-container-high text-on-surface ring-2 ring-primary/60`
            : `${phaseBorder} bg-surface-container text-on-surface hover:bg-surface-container-high`
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
          className={`mx-auto mb-0.5 block w-fit rounded-full border bg-surface-container-highest px-1 font-mono text-mono-sm text-on-surface ${phaseBadge}`}
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

function toMarker(
  lane: TacticLane,
  phase: TacticPhase,
  orientation: PitchOrientation,
): PitchMarker | null {
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
    coordinate: projectPosition(coordinate, orientation),
    collides: false,
  };
}

export function pitchMarkersForView(
  view: TacticView,
  lanes: TacticLane[],
  orientation: PitchOrientation = "portrait",
): PitchMarker[] {
  const phases: TacticPhase[] = view === "both" ? ["ip", "oop"] : [view];
  const markers: PitchMarker[] = [];
  for (const lane of lanes) {
    for (const phase of phases) {
      const marker = toMarker(lane, phase, orientation);
      if (marker) {
        markers.push(marker);
      }
    }
  }
  // Unique placements have distinct coordinates, so this order is total and
  // matches the current visual reading order (top-to-bottom,
  // left-to-right) in the active orientation: coordinates are already
  // projected, so portrait attack-up and landscape attack-right both sort
  // by their displayed position.
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

function pitchMarkers(
  phase: TacticPhase,
  lanes: TacticLane[],
  orientation: PitchOrientation,
): PitchMarker[] {
  return pitchMarkersForView(phase, lanes, orientation);
}

type TacticConnector = {
  lane: TacticLane;
  from: PitchCoordinate;
  to: PitchCoordinate;
};

// A collided pair splits around its shared anchor: the IP button ends at
// anchor - 2px and the OOP button starts at anchor + 2px, so a connector
// ending exactly on the anchor lands in the gap between the buttons.
// Chromium resolves mixed percent+px calc() against SVG user units rather
// than viewport pixels (headless probe 2026-09-06: attribute calc(35% - 2px)
// rendered at 165px instead of 173px; style geometry props had no effect),
// so shift the colliding end one viewBox unit into its own button instead:
// one unit clears the 2px gap wherever the Both pitch is at least 200px
// wide (the supported desktop widths) while staying inside the >= 44px
// marker. Only the colliding end shifts; centered markers already attach
// at the button middle. The shift applies on the x axis in projected space
// (after projectPosition), so it follows the displayed horizontal
// IP-left/OOP-right split in both portrait and landscape orientations.

function tacticConnectors(markers: PitchMarker[]): TacticConnector[] {
  const byLane = new Map<
    string,
    { lane: TacticLane; ip?: PitchMarker; oop?: PitchMarker }
  >();
  for (const marker of markers) {
    const entry = byLane.get(marker.lane.laneId) ?? { lane: marker.lane };
    entry[marker.phase] = marker;
    byLane.set(marker.lane.laneId, entry);
  }
  const connectors: TacticConnector[] = [];
  for (const { lane, ip, oop } of byLane.values()) {
    // Both-mode only: single-phase marker sets never hold both phases.
    // Placement is already canonicalized, so legacy-equivalent ST/STC
    // pairs share an identity and render no connector.
    if (ip && oop && ip.placement !== oop.placement) {
      connectors.push({
        lane,
        from: {
          x: ip.coordinate.x - (ip.collides ? 0.01 : 0),
          y: ip.coordinate.y,
        },
        to: {
          x: oop.coordinate.x + (oop.collides ? 0.01 : 0),
          y: oop.coordinate.y,
        },
      });
    }
  }
  return connectors;
}

function toPercent(coordinate: number): number {
  return Math.round(coordinate * 100);
}

export function TacticPitchCanvas({
  legend,
  markers,
  lanes,
  options,
  dual = false,
  orientation = "portrait",
  selectedLaneId,
  highlightedLaneId,
  linkedHintId,
  onHighlight,
  onSelectLane,
  canvasClassName = "h-[420px]",
}: {
  legend: string;
  markers: PitchMarker[];
  lanes: TacticLane[];
  options: TacticOptions;
  dual?: boolean;
  orientation?: PitchOrientation;
  selectedLaneId: string;
  highlightedLaneId: string | null;
  linkedHintId: string;
  onHighlight: (laneId: string | null) => void;
  onSelectLane: (laneId: string) => void;
  canvasClassName?: string;
}) {
  const attackDescriptionId = useId();
  // Landscape is a clockwise 90-degree rotation of the portrait canvas, so
  // portrait attack-up becomes landscape attack-right. Marker labels stay
  // upright HTML; only coordinates and line markings project.
  const attack =
    orientation === "landscape"
      ? { arrow: "→", text: "Attack toward the right" }
      : { arrow: "↑", text: "Attack toward the top" };
  // Both-only transitions: the shared single-phase canvas never holds both
  // phases for a lane, so single-phase modes and the modal render nothing.
  const connectors = dual ? tacticConnectors(markers) : [];
  const selectedLane = lanes.find((lane) => lane.laneId === selectedLaneId);
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
        <span aria-hidden="true">{attack.arrow}</span> {attack.text}
      </p>
      <div
        className={`relative w-full overflow-hidden rounded-md border border-outline-variant bg-surface-container-high ${canvasClassName}`}
      >
        <svg
          aria-hidden="true"
          className="pointer-events-none absolute inset-0 h-full w-full text-outline-variant"
          fill="none"
          preserveAspectRatio="none"
          stroke="currentColor"
          strokeWidth={0.5}
          viewBox="0 0 100 100"
        >
          {orientation === "landscape" ? (
            // Clockwise-projected markings: the halfway line runs vertically
            // and the boxes guard the left (own) and right (attack) goals.
            <>
              <rect height="96" width="96" x="2" y="2" />
              <line x1="50" x2="50" y1="2" y2="98" />
              <circle cx="50" cy="50" r="8" />
              <rect height="30" width="12" x="2" y="35" />
              <rect height="30" width="12" x="86" y="35" />
            </>
          ) : (
            <>
              <rect height="96" width="96" x="2" y="2" />
              <line x1="2" x2="98" y1="50" y2="50" />
              <circle cx="50" cy="50" r="8" />
              <rect height="12" width="30" x="35" y="2" />
              <rect height="12" width="30" x="35" y="86" />
            </>
          )}
        </svg>
        {connectors.length > 0 ? (
          <svg
            aria-hidden="true"
            className="pointer-events-none absolute inset-0 h-full w-full text-primary"
            fill="none"
            preserveAspectRatio="none"
            stroke="currentColor"
            strokeWidth={0.75}
            viewBox="0 0 100 100"
          >
            {connectors.map((connector) => (
              <line
                data-tactic-connector={connector.lane.laneId}
                key={connector.lane.laneId}
                strokeLinecap="round"
                x1={toPercent(connector.from.x)}
                x2={toPercent(connector.to.x)}
                y1={toPercent(connector.from.y)}
                y2={toPercent(connector.to.y)}
              />
            ))}
          </svg>
        ) : null}
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
      {dual ? (
        <p
          data-selected-slot-transition={selectedLane?.laneId ?? "none"}
          className="whitespace-pre-line pt-1 text-center text-label-md text-on-surface"
        >
          {selectedLane
            ? linkedPositionDescription(selectedLane, lanes, options).replace(
                " / ",
                "\n",
              )
            : "Select a position"}
        </p>
      ) : null}
      {dual ? (
        <div className="sr-only">
          {lanes.map((lane) => (
            <p data-slot-transition={lane.laneId} key={lane.laneId}>
              {linkedPositionDescription(lane, lanes, options)}
            </p>
          ))}
        </div>
      ) : null}
    </fieldset>
  );
}

export function PlannerTacticPitch({
  phase,
  lanes,
  options,
  orientation = "portrait",
  selectionHint = "Focus or select this position to highlight its linked counterpart in the other phase.",
  selectedLaneId,
  highlightedLaneId,
  onHighlight,
  onSelectLane,
  canvasClassName,
}: PlannerTacticPitchProps & { canvasClassName?: string }) {
  const { label } = TACTIC_PHASES[phase];
  const selectedLane = lanes.find((lane) => lane.laneId === selectedLaneId);
  const headingId = useId();
  const linkedHintId = useId();
  const markers = pitchMarkers(phase, lanes, orientation);

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
        orientation={orientation}
        selectedLaneId={selectedLaneId}
        highlightedLaneId={highlightedLaneId}
        linkedHintId={linkedHintId}
        onHighlight={onHighlight}
        onSelectLane={onSelectLane}
        canvasClassName={canvasClassName}
      />
    </section>
  );
}
