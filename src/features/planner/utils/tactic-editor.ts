import type {
  PlannerTactic,
  TacticLane,
  TacticOptions,
  TacticRoleOption,
} from "../types/tactic";
import { TACTIC_LANE_IDS } from "../types/tactic";

export type TacticPhase = "ip" | "oop";
export type TacticView = TacticPhase | "both";

export const TACTIC_VIEWS: TacticView[] = ["ip", "oop", "both"];

export const TACTIC_PHASES: Record<
  TacticPhase,
  { label: string; shortLabel: string; rolePhase: TacticRoleOption["phase"] }
> = {
  ip: {
    label: "In-Possession",
    shortLabel: "IP",
    rolePhase: "in_possession",
  },
  oop: {
    label: "Out-of-Possession",
    shortLabel: "OOP",
    rolePhase: "out_of_possession",
  },
};

const LANE_LABELS: Record<string, string> = {
  goalkeeper: "Goalkeeper",
  left_back: "Left back",
  left_centre_back: "Left centre-back",
  right_centre_back: "Right centre-back",
  right_back: "Right back",
  defensive_midfielder: "Defensive midfielder",
  left_central_midfielder: "Left central midfielder",
  right_central_midfielder: "Right central midfielder",
  left_winger: "Left winger",
  right_winger: "Right winger",
  centre_forward: "Centre forward",
};

export function laneLabel(laneId: string): string {
  return LANE_LABELS[laneId] ?? laneId.replace(/_/g, " ");
}

export function phasePosition(lane: TacticLane, phase: TacticPhase): string {
  return phase === "ip" ? lane.ipPosition : lane.oopPosition;
}

export function phaseRoleId(lane: TacticLane, phase: TacticPhase): string {
  return phase === "ip" ? lane.ipRoleId : lane.oopRoleId;
}

export function updatePhaseLane(
  lane: TacticLane,
  phase: TacticPhase,
  position: string,
  roleId: string,
): TacticLane {
  if (phase === "ip") {
    return { ...lane, ipPosition: position, ipRoleId: roleId };
  }
  return { ...lane, oopPosition: position, oopRoleId: roleId };
}

export function rolesForPhase(
  options: TacticOptions,
  phase: TacticPhase,
  position: string,
): TacticRoleOption[] {
  return options.roles.filter(
    (role) =>
      role.phase === TACTIC_PHASES[phase].rolePhase &&
      role.positionTags.includes(position),
  );
}

export function roleLabel(
  lane: TacticLane,
  phase: TacticPhase,
  options: TacticOptions,
): string {
  return (
    options.roles.find((role) => role.roleId === phaseRoleId(lane, phase))
      ?.displayName ?? "Choose a role"
  );
}

export function cloneTactic(tactic: PlannerTactic): PlannerTactic {
  return {
    lanes: tactic.lanes.map((lane) => ({ ...lane })),
  };
}

export function tacticEquals(
  left: PlannerTactic,
  right: PlannerTactic,
): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

export function validateTacticDraft(
  tactic: PlannerTactic,
  options: TacticOptions,
): string | null {
  if (tactic.lanes.length !== TACTIC_LANE_IDS.length) {
    return `The tactic must contain ${TACTIC_LANE_IDS.length} linked lanes.`;
  }

  for (const [index, lane] of tactic.lanes.entries()) {
    if (
      !Number.isFinite(lane.ipWeight) ||
      lane.ipWeight < 0 ||
      lane.ipWeight > 1
    ) {
      return `Lane ${index + 1} IP score weight must be between 0% and 100%.`;
    }
    for (const phase of ["ip", "oop"] as const) {
      const position = phasePosition(lane, phase);
      const role = options.roles.find(
        (candidate) => candidate.roleId === phaseRoleId(lane, phase),
      );
      if (
        !position ||
        !role ||
        role.phase !== TACTIC_PHASES[phase].rolePhase ||
        !role.positionTags.includes(position)
      ) {
        return `Choose a compatible ${TACTIC_PHASES[phase].shortLabel} role for lane ${index + 1}.`;
      }
    }
  }

  return null;
}
