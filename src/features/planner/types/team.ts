export const PLANNER_TEAMS = ["senior", "reserves", "youth"] as const;
export type PlannerTeam = (typeof PLANNER_TEAMS)[number];

export const PLANNER_TEAM_NAMES: Record<PlannerTeam, string> = {
  senior: "Senior",
  reserves: "Reserves",
  youth: "Youth",
};

/** The age limit a team starts with. The senior team never has one. */
export const PLANNER_TEAM_DEFAULT_MAX_AGE: Record<PlannerTeam, number | null> =
  {
    senior: null,
    reserves: 23,
    youth: 18,
  };

export const PLANNER_TEAM_MIN_MAX_AGE = 14;
export const PLANNER_TEAM_MAX_MAX_AGE = 45;
