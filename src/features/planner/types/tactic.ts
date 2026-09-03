export { TACTIC_LANE_IDS } from "@/utils/tactic-ids";

export type TacticLane = {
  laneId: string;
  ipWeight: number;
  importanceRank: number | null;
  preferredFoot: "any" | "left" | "right" | "both";
  footPreference: "preferred" | "strict";
  ipPosition: string;
  ipRoleId: string;
  oopPosition: string;
  oopRoleId: string;
};

export type PlannerTactic = {
  lanes: TacticLane[];
};

export type TacticRoleOption = {
  roleId: string;
  displayName: string;
  phase: "in_possession" | "out_of_possession";
  positionTags: string[];
};

export type TacticOptions = {
  placements: string[];
  roles: TacticRoleOption[];
};
