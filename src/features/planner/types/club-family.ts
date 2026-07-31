export const PLANNER_TEAMS = ["senior", "reserves", "youth"] as const;
export type PlannerTeam = (typeof PLANNER_TEAMS)[number];

export const PLANNER_TEAM_LEVELS = ["senior", "reserve", "youth"] as const;
export type PlannerTeamLevel = (typeof PLANNER_TEAM_LEVELS)[number];

export type ClubSource = {
  id: number;
  team: PlannerTeam;
  clubName: string;
  teamLevel: PlannerTeamLevel | null;
  isPrimary: boolean;
};

export type ClubFamily = {
  primaryClub: string | null;
  sources: ClubSource[];
};

export type ClubSourceInput = {
  team: Exclude<PlannerTeam, "senior">;
  clubName: string;
  teamLevel: PlannerTeamLevel | null;
};
