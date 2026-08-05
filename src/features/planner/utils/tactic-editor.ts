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

export type PhasePositionColumn = "left" | "centre" | "right";

export type PhasePositionPlacement = {
  column: PhasePositionColumn;
  row: number;
  rowSize: 1 | 2 | 3;
};

const CENTRAL_POSITION_LABELS: Record<string, string> = {
  DC: "DC",
  DM: "DM",
  MC: "MC",
  AMC: "AMC",
  ST: "STC",
};

const CENTRAL_COLUMNS: Record<1 | 2 | 3, PhasePositionColumn[]> = {
  1: ["centre"],
  2: ["right", "left"],
  3: ["right", "centre", "left"],
};

function positionPlacement(
  position: string,
  index: number,
  count: number,
): PhasePositionPlacement {
  if (!(position in CENTRAL_POSITION_LABELS)) {
    return { column: "centre", row: index, rowSize: 1 };
  }

  const row = Math.floor(index / 3);
  const rowSize = Math.min(3, count - row * 3) as 1 | 2 | 3;
  const column = CENTRAL_COLUMNS[rowSize][index % 3] ?? "centre";

  return { column, row, rowSize };
}

export function phasePositionLayout(
  phase: TacticPhase,
  lanes: TacticLane[],
): Map<string, PhasePositionPlacement> {
  const grouped = new Map<string, TacticLane[]>();
  for (const lane of lanes) {
    const position = phasePosition(lane, phase);
    const positionLanes = grouped.get(position) ?? [];
    positionLanes.push(lane);
    grouped.set(position, positionLanes);
  }

  const layout = new Map<string, PhasePositionPlacement>();
  for (const [position, positionLanes] of grouped) {
    positionLanes.forEach((lane, index) => {
      layout.set(
        lane.laneId,
        positionPlacement(position, index, positionLanes.length),
      );
    });
  }
  return layout;
}

export function phasePositionLabel(
  lane: TacticLane,
  phase: TacticPhase,
  lanes: TacticLane[],
): string {
  const position = phasePosition(lane, phase);
  if (!position) {
    return "Position";
  }

  const label = CENTRAL_POSITION_LABELS[position] ?? position;
  const placement = phasePositionLayout(phase, lanes).get(lane.laneId);
  if (!placement) {
    return label;
  }

  const positionedLabel =
    position in CENTRAL_POSITION_LABELS && placement.column !== "centre"
      ? `${label}${placement.column === "left" ? "L" : "R"}`
      : label;
  return placement.row === 0
    ? positionedLabel
    : `${positionedLabel} (row ${placement.row + 1})`;
}

export function phaseDescription(
  lane: TacticLane,
  phase: TacticPhase,
  lanes: TacticLane[],
  options: TacticOptions,
): string {
  return `${phasePositionLabel(lane, phase, lanes)} · ${roleLabel(lane, phase, options)}`;
}

export function linkedPositionDescription(
  lane: TacticLane,
  lanes: TacticLane[],
  options: TacticOptions,
): string {
  return `IP: ${phaseDescription(lane, "ip", lanes, options)} / OOP: ${phaseDescription(lane, "oop", lanes, options)}`;
}

export function linkedPositionDescriptionForId(
  laneId: string,
  lanes: TacticLane[],
  options: TacticOptions,
): string {
  const lane = lanes.find((candidate) => candidate.laneId === laneId);
  return lane
    ? linkedPositionDescription(lane, lanes, options)
    : "Unknown tactical position";
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
    return `The tactic must contain ${TACTIC_LANE_IDS.length} linked positions.`;
  }

  const importanceRanks = new Set<number>();
  for (const lane of tactic.lanes) {
    if (
      !Number.isFinite(lane.ipWeight) ||
      lane.ipWeight < 0 ||
      lane.ipWeight > 1
    ) {
      return `${phaseDescription(lane, "ip", tactic.lanes, options)} IP score weight must be between 0% and 100%.`;
    }
    if (
      lane.importanceRank !== null &&
      (!Number.isInteger(lane.importanceRank) ||
        lane.importanceRank < 1 ||
        lane.importanceRank > TACTIC_LANE_IDS.length)
    ) {
      return `${linkedPositionDescription(lane, tactic.lanes, options)} importance rank must be between 1 and ${TACTIC_LANE_IDS.length}.`;
    }
    if (
      lane.importanceRank !== null &&
      importanceRanks.has(lane.importanceRank)
    ) {
      return `${linkedPositionDescription(lane, tactic.lanes, options)} cannot use importance rank ${lane.importanceRank}; it is already used.`;
    }
    if (lane.importanceRank !== null) {
      importanceRanks.add(lane.importanceRank);
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
        const position = phasePositionLabel(lane, phase, tactic.lanes);
        return `Choose a compatible ${TACTIC_PHASES[phase].shortLabel} role for ${position}.`;
      }
    }
  }

  return null;
}
