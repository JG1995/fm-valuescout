export const PLANNER_TEAMS = ["senior", "reserves", "youth"] as const;
export type PlannerTeam = (typeof PLANNER_TEAMS)[number];

export const PLANNER_TEAM_NAMES: Record<PlannerTeam, string> = {
  senior: "Senior",
  reserves: "Reserves",
  youth: "Youth",
};
