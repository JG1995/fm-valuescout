import { useId } from "react";
import { SelectField } from "@/components/ui/field/select-field";
import type { TacticLane, TacticOptions } from "../types/tactic";
import {
  laneLabel,
  phasePosition,
  phaseRoleId,
  roleLabel,
  rolesForPhase,
  TACTIC_PHASES,
  type TacticPhase,
} from "../utils/tactic-editor";

type PlannerTacticPitchProps = {
  phase: TacticPhase;
  lanes: TacticLane[];
  options: TacticOptions;
  selectedLaneId: string;
  onSelectLane: (laneId: string) => void;
  onPositionChange: (laneId: string, position: string) => void;
  onRoleChange: (laneId: string, roleId: string) => void;
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
  laneNumber,
  options,
  selected,
  onSelect,
}: {
  phase: TacticPhase;
  lane: TacticLane;
  laneNumber: number;
  options: TacticOptions;
  selected: boolean;
  onSelect: () => void;
}) {
  const position = phasePosition(lane, phase);
  const role = roleLabel(lane, phase, options);
  const { shortLabel } = TACTIC_PHASES[phase];

  return (
    <button
      type="button"
      aria-label={`${shortLabel} lane ${laneNumber}: ${position}, ${role}`}
      aria-pressed={selected}
      className={`min-h-11 w-full rounded-md border px-1 py-1 text-center transition-colors duration-150 ease-out focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary ${
        selected
          ? "border-primary bg-primary-container text-primary"
          : "border-outline-variant bg-surface-container text-on-surface hover:bg-surface-container-high"
      }`}
      onClick={onSelect}
    >
      <span className="block font-mono text-mono-sm tabular-nums">
        {laneNumber}
      </span>
      <span
        className="block truncate text-[11px]"
        title={`${position} · ${role}`}
      >
        {position}
      </span>
    </button>
  );
}

function PitchBoard({
  phase,
  lanes,
  options,
  selectedLaneId,
  onSelectLane,
}: Pick<
  PlannerTacticPitchProps,
  "phase" | "lanes" | "options" | "selectedLaneId" | "onSelectLane"
>) {
  return (
    <fieldset className="space-y-2 rounded-lg border border-outline-variant bg-surface-container-lowest p-3">
      <legend className="sr-only">{TACTIC_PHASES[phase].label} pitch</legend>
      {PITCH_ROWS.map((row) => (
        <div className="grid min-h-16 grid-cols-3 gap-2" key={row.id}>
          {row.cells.map((cell) => {
            const { position } = cell;
            const positionLanes = position
              ? lanes
                  .map((lane, index) => ({ lane, laneNumber: index + 1 }))
                  .filter(({ lane }) => phasePosition(lane, phase) === position)
              : [];
            return (
              <div
                className="flex min-h-16 flex-col items-center justify-center gap-1 rounded-md border border-outline-variant bg-surface-container-high p-1"
                key={cell.id}
              >
                {positionLanes.length > 0 ? (
                  positionLanes.map(({ lane, laneNumber }) => (
                    <LaneButton
                      key={lane.laneId}
                      phase={phase}
                      lane={lane}
                      laneNumber={laneNumber}
                      options={options}
                      selected={lane.laneId === selectedLaneId}
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

function TacticLaneControls({
  phase,
  lane,
  laneNumber,
  options,
  onPositionChange,
  onRoleChange,
}: {
  phase: TacticPhase;
  lane: TacticLane;
  laneNumber: number;
  options: TacticOptions;
  onPositionChange: (position: string) => void;
  onRoleChange: (roleId: string) => void;
}) {
  const position = phasePosition(lane, phase);
  const roleId = phaseRoleId(lane, phase);
  const roles = rolesForPhase(options, phase, position);
  const selectedRoleIsCompatible = roles.some((role) => role.roleId === roleId);
  const { label, shortLabel } = TACTIC_PHASES[phase];
  const placements = options.placements.includes(position)
    ? options.placements
    : [position, ...options.placements];
  const headingId = useId();

  return (
    <section
      className="space-y-3 rounded-lg border border-outline-variant bg-surface-container-high p-3"
      aria-labelledby={headingId}
    >
      <div>
        <h4 id={headingId} className="text-label-lg text-on-surface">
          Lane {laneNumber} · {laneLabel(lane.laneId)}
        </h4>
        <p className="text-body-sm text-on-surface-variant">{label} role fit</p>
      </div>
      <SelectField
        label={`${shortLabel} lane ${laneNumber} position`}
        value={position}
        onChange={(event) => onPositionChange(event.target.value)}
      >
        {placements.map((placement) => (
          <option key={placement} value={placement}>
            {placement}
          </option>
        ))}
      </SelectField>
      <SelectField
        label={`${shortLabel} lane ${laneNumber} role`}
        value={selectedRoleIsCompatible ? roleId : ""}
        disabled={roles.length === 0}
        onChange={(event) => onRoleChange(event.target.value)}
      >
        <option value="">Choose a compatible role</option>
        {roles.map((role) => (
          <option key={role.roleId} value={role.roleId}>
            {role.displayName}
          </option>
        ))}
      </SelectField>
      {roles.length === 0 ? (
        <p className="text-body-sm text-warning">
          No {shortLabel} roles support this position.
        </p>
      ) : null}
    </section>
  );
}

export function PlannerTacticPitch({
  phase,
  lanes,
  options,
  selectedLaneId,
  onSelectLane,
  onPositionChange,
  onRoleChange,
}: PlannerTacticPitchProps) {
  const { label, shortLabel } = TACTIC_PHASES[phase];
  const selectedLane = lanes.find((lane) => lane.laneId === selectedLaneId);
  const selectedLaneNumber = selectedLane
    ? lanes.findIndex((lane) => lane.laneId === selectedLaneId) + 1
    : 0;
  const headingId = useId();

  return (
    <section className="space-y-3" aria-labelledby={headingId}>
      <div className="flex items-start justify-between gap-3">
        <div>
          <h3 id={headingId} className="text-headline-sm text-on-surface">
            {label}
          </h3>
          <p className="text-body-sm text-on-surface-variant">
            Select a numbered lane to edit its {shortLabel} placement and role.
          </p>
        </div>
        <span className="shrink-0 rounded-full bg-surface-container-high px-2 py-1 font-mono text-mono-sm text-on-surface-variant">
          {selectedLane ? `Lane ${selectedLaneNumber}` : "Select a lane"}
        </span>
      </div>
      <div className="grid items-start gap-3 xl:grid-cols-[minmax(0,1fr)_minmax(13rem,15rem)]">
        <PitchBoard
          phase={phase}
          lanes={lanes}
          options={options}
          selectedLaneId={selectedLaneId}
          onSelectLane={onSelectLane}
        />
        {selectedLane ? (
          <TacticLaneControls
            phase={phase}
            lane={selectedLane}
            laneNumber={selectedLaneNumber}
            options={options}
            onPositionChange={(position) =>
              onPositionChange(selectedLane.laneId, position)
            }
            onRoleChange={(roleId) => onRoleChange(selectedLane.laneId, roleId)}
          />
        ) : (
          <div className="rounded-lg border border-dashed border-outline-variant p-4 text-body-sm text-on-surface-variant">
            Select a lane on the pitch to edit its phase placement and role.
          </div>
        )}
      </div>
    </section>
  );
}
