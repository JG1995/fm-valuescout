import { useId } from "react";
import { SelectField } from "@/components/ui/field/select-field";
import {
  TACTIC_LANE_IDS,
  type TacticLane,
  type TacticOptions,
} from "../types/tactic";
import {
  linkedPositionDescription,
  phasePosition,
  phasePositionLabel,
  phaseRoleId,
  rolesForPhase,
  TACTIC_PHASES,
  type TacticPhase,
} from "../utils/tactic-editor";

type PlannerTacticInspectorProps = {
  selectedLane: TacticLane;
  lanes: TacticLane[];
  options: TacticOptions;
  phases: TacticPhase[];
  onWeightChange: (ipWeight: number) => void;
  onRankChange: (importanceRank: number | null) => void;
  onPreferredFootChange: (preferredFoot: TacticLane["preferredFoot"]) => void;
  onFootPreferenceChange: (
    footPreference: TacticLane["footPreference"],
  ) => void;
  onPositionChange: (phase: TacticPhase, position: string) => void;
  onRoleChange: (phase: TacticPhase, roleId: string) => void;
};

function nextWeight(value: number, key: string): number | null {
  if (key === "ArrowRight" || key === "ArrowUp") {
    return Math.min(100, value + 1);
  }
  if (key === "ArrowLeft" || key === "ArrowDown") {
    return Math.max(0, value - 1);
  }
  if (key === "Home") {
    return 0;
  }
  if (key === "End") {
    return 100;
  }
  return null;
}

function PhaseControls({
  phase,
  lane,
  lanes,
  options,
  onPositionChange,
  onRoleChange,
}: {
  phase: TacticPhase;
  lane: TacticLane;
  lanes: TacticLane[];
  options: TacticOptions;
  onPositionChange: (position: string) => void;
  onRoleChange: (roleId: string) => void;
}) {
  const position = phasePosition(lane, phase);
  const positionLabel = phasePositionLabel(lane, phase, lanes);
  const roleId = phaseRoleId(lane, phase);
  const roles = rolesForPhase(options, phase, position);
  const selectedRoleIsCompatible = roles.some((role) => role.roleId === roleId);
  const { shortLabel } = TACTIC_PHASES[phase];
  const placements = options.placements.includes(position)
    ? options.placements
    : [position, ...options.placements];

  return (
    <>
      <SelectField
        label={`${shortLabel} ${positionLabel} position`}
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
        label={`${shortLabel} ${positionLabel} role`}
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
    </>
  );
}

export function PlannerTacticInspector({
  selectedLane,
  lanes,
  options,
  phases,
  onWeightChange,
  onRankChange,
  onPreferredFootChange,
  onFootPreferenceChange,
  onPositionChange,
  onRoleChange,
}: PlannerTacticInspectorProps) {
  const weightId = useId();
  const headingId = useId();
  const weight = Math.round(selectedLane.ipWeight * 100);

  return (
    <section
      className="space-y-2 rounded-lg border border-outline-variant bg-surface-container-high p-3"
      aria-labelledby={headingId}
    >
      <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
        <h3 id={headingId} className="text-label-lg text-on-surface">
          Selected position settings
        </h3>
        <p className="text-body-sm text-on-surface-variant">
          {linkedPositionDescription(selectedLane, lanes, options)}
        </p>
      </div>

      <div className="grid gap-3 [grid-template-columns:repeat(auto-fit,minmax(9rem,1fr))]">
        <div className="space-y-1">
          <label
            className="block text-label-md text-on-surface-variant"
            htmlFor={weightId}
          >
            IP/OOP score weight
          </label>
          <input
            id={weightId}
            type="range"
            min="0"
            max="100"
            step="1"
            value={weight}
            aria-label="IP/OOP score weight"
            aria-valuetext={`IP ${weight}%, OOP ${100 - weight}%`}
            className="h-2 w-full cursor-pointer accent-primary focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
            onKeyDown={(event) => {
              const next = nextWeight(weight, event.key);
              if (next === null) {
                return;
              }
              event.preventDefault();
              onWeightChange(next / 100);
            }}
            onChange={(event) =>
              onWeightChange(Number(event.target.value) / 100)
            }
          />
          <p className="font-mono text-mono-sm text-on-surface tabular-nums">
            IP {weight}% / OOP {100 - weight}%
          </p>
        </div>

        <SelectField
          label="Importance rank"
          value={selectedLane.importanceRank?.toString() ?? ""}
          onChange={(event) =>
            onRankChange(
              event.target.value === "" ? null : Number(event.target.value),
            )
          }
        >
          <option value="">No rank</option>
          {TACTIC_LANE_IDS.map((laneId, index) => (
            <option key={laneId} value={index + 1}>
              {index + 1}
            </option>
          ))}
        </SelectField>
        <SelectField
          label="Preferred foot"
          value={selectedLane.preferredFoot}
          onChange={(event) =>
            onPreferredFootChange(
              event.target.value as TacticLane["preferredFoot"],
            )
          }
        >
          <option value="any">Either</option>
          <option value="left">Left</option>
          <option value="right">Right</option>
          <option value="both">Both</option>
        </SelectField>
        <SelectField
          label="Foot preference"
          value={selectedLane.footPreference}
          disabled={selectedLane.preferredFoot === "any"}
          onChange={(event) =>
            onFootPreferenceChange(
              event.target.value as TacticLane["footPreference"],
            )
          }
        >
          <option value="preferred">Preferred</option>
          <option value="strict">Strict</option>
        </SelectField>
        {phases.map((phase) => (
          <PhaseControls
            key={phase}
            phase={phase}
            lane={selectedLane}
            lanes={lanes}
            options={options}
            onPositionChange={(position) => onPositionChange(phase, position)}
            onRoleChange={(roleId) => onRoleChange(phase, roleId)}
          />
        ))}
      </div>
    </section>
  );
}
