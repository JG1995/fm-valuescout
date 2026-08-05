import { useId } from "react";
import type { TacticLane, TacticOptions } from "../types/tactic";
import {
  type PhasePositionColumn,
  phaseDescription,
  phasePosition,
  phasePositionLabel,
  phasePositionLayout,
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
    id: "striker",
    cells: [
      { id: "striker-left", position: null },
      { id: "striker-center", position: "ST" },
      { id: "striker-right", position: null },
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
    id: "midfield",
    cells: [
      { id: "midfield-left", position: "ML" },
      { id: "midfield-center", position: "MC" },
      { id: "midfield-right", position: "MR" },
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
    id: "defence",
    cells: [
      { id: "defence-left", position: "DL" },
      { id: "defence-center", position: "DC" },
      { id: "defence-right", position: "DR" },
    ],
  },
  {
    id: "goalkeeper",
    cells: [
      { id: "goalkeeper-left", position: null },
      { id: "goalkeeper-center", position: "GK" },
      { id: "goalkeeper-right", position: null },
    ],
  },
];

const CENTRAL_POSITIONS = new Set(["GK", "DC", "DM", "MC", "AMC", "ST"]);

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
  const positionLayout = phasePositionLayout(phase, lanes);

  return (
    <fieldset className="space-y-2 rounded-lg border border-outline-variant bg-surface-container-lowest p-3">
      <legend className="sr-only">{TACTIC_PHASES[phase].label} pitch</legend>
      {PITCH_ROWS.map((row) => (
        <div className="grid min-h-16 grid-cols-5 gap-2" key={row.id}>
          {row.cells.map((cell) => {
            const { position } = cell;
            const positionLanes = position
              ? lanes.filter((lane) => phasePosition(lane, phase) === position)
              : [];
            const usesSlotGrid =
              positionLanes.length > 1 ||
              (position !== null && CENTRAL_POSITIONS.has(position));
            return (
              <div
                className={`${position && CENTRAL_POSITIONS.has(position) ? "col-span-3" : "col-span-1"} flex min-h-16 min-w-0 flex-col items-center justify-center gap-1 rounded-md border border-outline-variant bg-surface-container-high p-1`}
                key={cell.id}
              >
                {positionLanes.length > 0 ? (
                  !usesSlotGrid ? (
                    <LaneButton
                      phase={phase}
                      lane={positionLanes[0]}
                      lanes={lanes}
                      options={options}
                      highlightedLaneId={highlightedLaneId}
                      linkedHintId={linkedHintId}
                      selected={positionLanes[0].laneId === selectedLaneId}
                      onHighlight={onHighlight}
                      onSelect={() => onSelectLane(positionLanes[0].laneId)}
                    />
                  ) : (
                    Array.from(
                      { length: Math.ceil(positionLanes.length / 3) },
                      (_, rowIndex) => {
                        const rowLanes = positionLanes.slice(
                          rowIndex * 3,
                          rowIndex * 3 + 3,
                        );
                        return (
                          <div
                            className="grid w-full min-w-0 grid-cols-3 gap-1"
                            key={rowLanes.map((lane) => lane.laneId).join("-")}
                          >
                            {rowLanes.map((lane) => {
                              const placement = positionLayout.get(lane.laneId);
                              return (
                                <div
                                  className={`${columnClass(placement?.column ?? "centre")} min-w-0`}
                                  key={lane.laneId}
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
                              );
                            })}
                          </div>
                        );
                      },
                    )
                  )
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

function columnClass(column: PhasePositionColumn): string {
  if (column === "left") {
    return "col-start-1";
  }
  if (column === "right") {
    return "col-start-3";
  }
  return "col-start-2";
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
