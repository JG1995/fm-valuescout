import { useId } from "react";
import type { TacticLane, TacticOptions } from "../types/tactic";
import {
  basePosition,
  type PhasePositionPlacement,
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
  selectionHint?: string;
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

const MIN_PITCH_SLOT_COUNT = 3;
const MAX_PITCH_SLOT_COUNT = 5;
const TACTIC_PHASE_IDS: TacticPhase[] = ["ip", "oop"];
const POSITION_COLUMN_ORDER = { left: 0, centre: 1, right: 2 } as const;

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
  const slotCount = tacticSlotCount(lanes);

  return (
    <fieldset
      className="space-y-2 rounded-lg border border-outline-variant bg-surface-container-lowest p-3"
      data-pitch-slot-count={slotCount}
    >
      <legend className="sr-only">{TACTIC_PHASES[phase].label} pitch</legend>
      {PITCH_ROWS.map((row) => {
        const positionLanes = row.cells.map((cell) =>
          cell.position
            ? lanes.filter(
                (lane) =>
                  basePosition(phasePosition(lane, phase)) === cell.position,
              )
            : [],
        );
        const visualRowCount = Math.max(
          1,
          ...positionLanes.flatMap((cellLanes) =>
            cellLanes.map(
              (lane) => (positionLayout.get(lane.laneId)?.row ?? 0) + 1,
            ),
          ),
        );

        return (
          <div className="space-y-1" key={row.id}>
            {Array.from({ length: visualRowCount }, (_, visualRow) => {
              const rowLanes = positionLanes.map((cellLanes) =>
                cellLanes.filter(
                  (lane) =>
                    (positionLayout.get(lane.laneId)?.row ?? 0) === visualRow,
                ),
              );
              const groupTracks = positionGroupTracks(
                rowLanes,
                slotCount,
                positionLayout,
              );
              const visualRowKey =
                rowLanes
                  .flat()
                  .map((lane) => lane.laneId)
                  .join("-") || "empty";

              return (
                <div
                  className="grid min-h-16 gap-1"
                  data-pitch-band={row.id}
                  key={`${row.id}-${visualRowKey}`}
                  style={{
                    gridTemplateColumns: `repeat(${slotCount * 2}, minmax(0, 1fr))`,
                  }}
                >
                  {row.cells.map((cell, cellIndex) => {
                    const { start, span } = groupTracks[cellIndex];
                    if (span === 0) {
                      return null;
                    }

                    const cellRowLanes = [...rowLanes[cellIndex]].sort(
                      (left, right) => {
                        const leftColumn =
                          positionLayout.get(left.laneId)?.column ?? "centre";
                        const rightColumn =
                          positionLayout.get(right.laneId)?.column ?? "centre";
                        return (
                          POSITION_COLUMN_ORDER[leftColumn] -
                          POSITION_COLUMN_ORDER[rightColumn]
                        );
                      },
                    );
                    return (
                      <div
                        className="grid min-h-16 min-w-0 gap-1 rounded-md border border-outline-variant bg-surface-container-high"
                        data-position-group={cell.position ?? undefined}
                        data-position-slot-count={cellRowLanes.length}
                        key={cell.id}
                        style={{
                          gridColumn: `${start} / span ${span}`,
                          gridRow: 1,
                          gridTemplateColumns: "subgrid",
                        }}
                      >
                        {cellRowLanes.length === 0 ? (
                          <span
                            className="flex items-center justify-center text-label-sm text-on-surface-variant"
                            style={{ gridColumn: "1 / -1", gridRow: 1 }}
                          >
                            {cell.position === "ST" ? "STC" : cell.position}
                          </span>
                        ) : (
                          cellRowLanes.map((lane) => {
                            const placement = positionLayout.get(lane.laneId);
                            const slotStart =
                              cellIndex === 1
                                ? centralSlotStart(placement)
                                : cellIndex === 0
                                  ? 1
                                  : span - 1;
                            const transform = outerSlotTransform(
                              cellIndex,
                              span,
                            );

                            return (
                              <div
                                className="z-10 flex min-w-0 items-center p-1"
                                data-position-slot={lane.laneId}
                                key={lane.laneId}
                                style={{
                                  gridColumn: `${slotStart} / span 2`,
                                  gridRow: 1,
                                  transform,
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
                            );
                          })
                        )}
                      </div>
                    );
                  })}
                </div>
              );
            })}
          </div>
        );
      })}
    </fieldset>
  );
}

function tacticSlotCount(lanes: TacticLane[]): number {
  let densestRow = 0;

  for (const phase of TACTIC_PHASE_IDS) {
    const layout = phasePositionLayout(phase, lanes);
    for (const pitchRow of PITCH_ROWS) {
      const rowPositions = new Set(
        pitchRow.cells.flatMap((cell) =>
          cell.position ? [cell.position] : [],
        ),
      );
      const rowWidths = new Map<number, { centre: number; outer: number }>();

      for (const lane of lanes) {
        const position = basePosition(phasePosition(lane, phase));
        if (!rowPositions.has(position)) {
          continue;
        }
        const placement = layout.get(lane.laneId);
        const visualRow = placement?.row ?? 0;
        const width = rowWidths.get(visualRow) ?? { centre: 0, outer: 0 };
        if (pitchRow.cells[1].position === position) {
          width.centre = Math.max(width.centre, placement?.rowSize ?? 1);
        } else {
          width.outer += 1;
        }
        rowWidths.set(visualRow, width);
      }

      densestRow = Math.max(
        densestRow,
        ...Array.from(
          rowWidths.values(),
          ({ centre, outer }) => centre + outer,
        ),
      );
    }
  }

  return Math.min(
    MAX_PITCH_SLOT_COUNT,
    Math.max(MIN_PITCH_SLOT_COUNT, densestRow),
  );
}

function positionGroupTracks(
  rowLanes: TacticLane[][],
  slotCount: number,
  layout: Map<string, PhasePositionPlacement>,
): { start: number; span: number }[] {
  const centreSlots = Math.max(
    1,
    ...rowLanes[1].map((lane) => layout.get(lane.laneId)?.rowSize ?? 1),
  );
  const outerTracks = slotCount * 2 - centreSlots * 2;
  const leftMinimum = rowLanes[0].length > 0 ? 2 : 0;
  const rightMinimum = rowLanes[2].length > 0 ? 2 : 0;
  const idealLeftTracks = slotCount - centreSlots;
  const leftTracks = Math.min(
    Math.max(idealLeftTracks, leftMinimum),
    outerTracks - rightMinimum,
  );
  const rightTracks = outerTracks - leftTracks;

  return [
    { start: 1, span: leftTracks },
    { start: leftTracks + 1, span: centreSlots * 2 },
    { start: leftTracks + centreSlots * 2 + 1, span: rightTracks },
  ];
}

function centralSlotStart(
  placement: PhasePositionPlacement | undefined,
): number {
  if (!placement || placement.column === "left") {
    return 1;
  }
  if (placement.column === "right") {
    return (placement.rowSize - 1) * 2 + 1;
  }
  return Math.floor(placement.rowSize / 2) * 2 + 1;
}

function outerSlotTransform(
  cellIndex: number,
  groupTrackCount: number,
): string | undefined {
  if (cellIndex === 1 || groupTrackCount <= 2) {
    return undefined;
  }

  const remainingTrackCount = groupTrackCount - 2;
  const distancePercent = remainingTrackCount * 25;
  const gapOffsetRem = remainingTrackCount * 0.0625;

  return cellIndex === 0
    ? `translateX(calc(${distancePercent}% + ${gapOffsetRem}rem))`
    : `translateX(calc(-${distancePercent}% - ${gapOffsetRem}rem))`;
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
