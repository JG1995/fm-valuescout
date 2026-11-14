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
  age: number | null;
  /** Placed or pinned by the user: Optimize keeps the player here. */
  pinned: boolean;
  /** Older than the team's age limit. */
  overAge: boolean;
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

/** A managed-club player whom no squad holds. */
export type PlannerUnassignedPlayer = {
  playerUid: number;
  name: string;
  age: number | null;
  /** The tactic lane he fits best now; null when he suits none. */
  laneId: string | null;
  currentScore: number | null;
  /** The score he is expected to reach in that lane. */
  expectedScore: number | null;
};

/** A player aged 18 to 23 outside the senior squad, ranked for a loan. */
export type PlannerLoanCandidate = {
  playerUid: number;
  name: string;
  age: number;
  /** The team and string that hold him; null when no squad does. */
  team: PlannerTeam | null;
  stringName: string | null;
  /** The tactic lane he is judged in. */
  laneId: string;
  currentScore: number;
  /** The score he is expected to reach in that lane. */
  expectedScore: number;
  /** How much a loan would help him, from 0 to 100. */
  benefit: number;
  /** Whether the benefit is high enough to mark him on the board. */
  recommended: boolean;
};

export type PlannerAssignmentLocation = {
  team: PlannerTeam;
  stringId: number;
  stringName: string;
  laneId: string;
};

/** How a player's positions suit a slot. Optimize never places an unsuited player. */
export type PlannerPositionFit = "suited" | "unfamiliar" | "unsuited";
/** How a player's foot suits a slot. Optimize never places an excluded player. */
export type PlannerFootFit = "matches" | "mismatch" | "excluded";

export type PlannerSlotCandidate = {
  playerUid: number;
  name: string;
  currentClub: string;
  age: number | null;
  /** Older than the age limit of the team being picked for. */
  overAge: boolean;
  ipScore: number | null;
  oopScore: number | null;
  combinedScore: number | null;
  /** The score the player is expected to reach, given his age and personality. */
  expectedCombinedScore: number | null;
  positionFit: PlannerPositionFit;
  footFit: PlannerFootFit;
  assignmentLocation: PlannerAssignmentLocation | null;
};
