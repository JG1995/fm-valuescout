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
  /** Ordered Moneyball statistic keys for the slot's custom score (0–5). */
  customMetricKeys: string[];
  /**
   * Rules a signing must pass to be recommended for the slot (0–10). They never
   * apply to the club's own players.
   */
  recruitmentFilters: RecruitmentFilter[];
};

/**
 * One recruitment filter rule in Search's filter IPC shape: raw values of
 * fraction-scale statistics are 0–1, percentiles 0–100.
 */
export type RecruitmentFilter = {
  field: string;
  op: string;
  value: string | number | boolean | number[] | string[];
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
