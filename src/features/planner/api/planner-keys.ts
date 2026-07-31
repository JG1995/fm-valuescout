export const plannerKeys = {
  all: ["planner"] as const,
  clubFamily: () => [...plannerKeys.all, "club-family"] as const,
  clubs: () => [...plannerKeys.all, "clubs"] as const,
  tactic: () => [...plannerKeys.all, "tactic"] as const,
  tacticOptions: () => [...plannerKeys.all, "tactic-options"] as const,
};
