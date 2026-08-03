export const TACTIC_LANE_IDS = [
  "goalkeeper",
  "left_back",
  "left_centre_back",
  "right_centre_back",
  "right_back",
  "defensive_midfielder",
  "left_central_midfielder",
  "right_central_midfielder",
  "left_winger",
  "right_winger",
  "centre_forward",
] as const;

export type TacticLane = {
  laneId: string;
  ipWeight: number;
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
