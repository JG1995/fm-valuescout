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
  /** The score the player is expected to reach, given his age and personality. */
  expectedCombinedScore: number | null;
  potentialCombinedScore: number | null;
};

export type PlannerString = {
  id: number;
  stringOrder: number;
  displayName: string;
  assignments: PlannerAssignment[];
};

export type PlannerDepthTeam = {
  team: PlannerTeam;
  displayName: string;
  /** The oldest age the team's squads take, or null for no limit. */
  maxAge: number | null;
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
