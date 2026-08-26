import type { PlannerTeam } from "./team";

export type PlannerStaffingTargetRemovalImpact = {
  jobId: string;
  jobLabel: string;
  slotCount: number;
};

export type PlannerTeamRemovalImpact = {
  team: PlannerTeam;
  displayName: string;
  assignmentCount: number;
  staffingTargets: PlannerStaffingTargetRemovalImpact[];
};
