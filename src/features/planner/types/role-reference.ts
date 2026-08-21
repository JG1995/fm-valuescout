export type PlannerRoleReferencePhase = "in_possession" | "out_of_possession";

export type PlannerRoleReferenceScoreBasis = "current" | "potential";

export type PlannerRoleReferencePlayer = {
  playerUid: number;
  name: string;
  currentScore: number | null;
  potentialScore: number | null;
};

export type PlannerRoleReferenceLane = {
  laneId: string;
  players: PlannerRoleReferencePlayer[];
};

export type PlannerRoleReference = {
  lanes: PlannerRoleReferenceLane[];
  noEligible: PlannerRoleReferencePlayer[];
};
