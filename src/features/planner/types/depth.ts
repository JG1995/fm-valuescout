import type { PlannerTactic } from "./tactic";
import type { PlannerTeam } from "./team";

export type PlannerAssignmentState = "resolved" | "outside_pool" | "unresolved";

export type PlannerAssignment = {
  id: number;
  laneId: string;
  playerUid: number;
  lastKnownName: string;
  currentName: string | null;
  state: PlannerAssignmentState;
  combinedScore: number | null;
  potentialCombinedScore: number | null;
};

export type PlannerString = {
  id: number;
  stringOrder: number;
  assignments: PlannerAssignment[];
};

export type PlannerDepthTeam = {
  team: PlannerTeam;
  displayName: string;
  strings: PlannerString[];
};

export type PlannerDepth = {
  tactic: PlannerTactic;
  teams: PlannerDepthTeam[];
};

export type PlannerAssignmentLocation = {
  team: PlannerTeam;
  stringId: number;
  stringOrder: number;
  laneId: string;
};

export type PlannerSlotCandidate = {
  playerUid: number;
  name: string;
  currentClub: string;
  ipScore: number | null;
  oopScore: number | null;
  combinedScore: number | null;
  assignmentLocation: PlannerAssignmentLocation | null;
};
