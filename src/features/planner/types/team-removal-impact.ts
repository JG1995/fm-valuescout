import type { PlannerTeam } from "./team";

export type PlannerStaffingTargetRemovalImpact = {
  jobId: string;
  jobLabel: string;
  slotCount: number;
};

export type PlannerStringRemovalImpact = {
  stringId: number;
  displayName: string;
  assignmentCount: number;
};

export type PlannerTeamRemovalImpact = {
  team: PlannerTeam;
  displayName: string;
  assignmentCount: number;
  staffingTargets: PlannerStaffingTargetRemovalImpact[];
  strings: PlannerStringRemovalImpact[];
};
