import { useId } from "react";
import type { TacticLane, TacticOptions } from "../types/tactic";
import {
  phaseDescription,
  phasePosition,
  phasePositionLabel,
  roleLabel,
  TACTIC_PHASES,
  type TacticPhase,
} from "../utils/tactic-editor";

type PlannerTacticPitchProps = {
  phase: TacticPhase;
  lanes: TacticLane[];
  options: TacticOptions;
  selectedLaneId: string;
  highlightedLaneId: string | null;
  onHighlight: (laneId: string | null) => void;
  onSelectLane: (laneId: string) => void;
};

const PITCH_ROWS = [
  {
    id: "goalkeeper",
    cells: [
      { id: "goalkeeper-left", position: null },
      { id: "goalkeeper-center", position: "GK" },
      { id: "goalkeeper-right", position: null },
    ],
  },
  {
    id: "defence",
    cells: [
      { id: "defence-left", position: "DL" },
      { id: "defence-center", position: "DC" },
      { id: "defence-right", position: "DR" },
    ],
  },
  {
    id: "wide-defence",
    cells: [
      { id: "wide-defence-left", position: "WBL" },
      { id: "wide-defence-center", position: "DM" },
      { id: "wide-defence-right", position: "WBR" },
    ],
  },
  {
    id: "midfield",
    cells: [
      { id: "midfield-left", position: "ML" },
      { id: "midfield-center", position: "MC" },
      { id: "midfield-right", position: "MR" },
    ],
  },
  {
    id: "attack-midfield",
    cells: [
      { id: "attack-midfield-left", position: "AML" },
      { id: "attack-midfield-center", position: "AMC" },
      { id: "attack-midfield-right", position: "AMR" },
    ],
  },
  {
    id: "striker",
    cells: [
      { id: "striker-left", position: null },
      { id: "striker-center", position: "ST" },
      { id: "striker-right", position: null },
    ],
  },
];

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

function PitchBoard({
  phase,
  lanes,
  options,
  selectedLaneId,
  highlightedLaneId,
  linkedHintId,
  onHighlight,
  onSelectLane,
}: Pick<
  PlannerTacticPitchProps,
  | "phase"
  | "lanes"
  | "options"
  | "selectedLaneId"
  | "highlightedLaneId"
  | "onHighlight"
  | "onSelectLane"
> & { linkedHintId: string }) {
  return (
    <fieldset className="space-y-2 rounded-lg border border-outline-variant bg-surface-container-lowest p-3">
      <legend className="sr-only">{TACTIC_PHASES[phase].label} pitch</legend>
      {PITCH_ROWS.map((row) => (
        <div className="grid min-h-16 grid-cols-3 gap-2" key={row.id}>
          {row.cells.map((cell) => {
            const { position } = cell;
            const positionLanes = position
              ? lanes.filter((lane) => phasePosition(lane, phase) === position)
              : [];
            return (
              <div
                className="flex min-h-16 flex-col items-center justify-center gap-1 rounded-md border border-outline-variant bg-surface-container-high p-1"
                key={cell.id}
              >
                {positionLanes.length > 0 ? (
                  positionLanes.map((lane) => (
                    <LaneButton
                      key={lane.laneId}
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
                  ))
                ) : position ? (
                  <span className="text-label-sm text-on-surface-variant">
                    {position}
                  </span>
                ) : null}
              </div>
            );
          })}
        </div>
      ))}
    </fieldset>
  );
}

export function PlannerTacticPitch({
  phase,
  lanes,
  options,
  selectedLaneId,
  highlightedLaneId,
  onHighlight,
  onSelectLane,
}: PlannerTacticPitchProps) {
  const { label, shortLabel } = TACTIC_PHASES[phase];
  const selectedLane = lanes.find((lane) => lane.laneId === selectedLaneId);
  const headingId = useId();
  const linkedHintId = useId();

  return (
    <section className="space-y-3" aria-labelledby={headingId}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <h3 id={headingId} className="text-headline-sm text-on-surface">
            {label}
          </h3>
          <p className="text-body-sm text-on-surface-variant">
            Select a position to edit its {shortLabel} placement and role in the
            position inspector.
          </p>
        </div>
        <span className="shrink-0 rounded-full bg-surface-container-high px-2 py-1 font-mono text-mono-sm text-on-surface-variant">
          {selectedLane
            ? phaseDescription(selectedLane, phase, lanes, options)
            : "Select a position"}
        </span>
      </div>
      <p id={linkedHintId} className="sr-only">
        Focus or select this position to highlight its linked counterpart in the
        other phase.
      </p>
      <PitchBoard
        phase={phase}
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
