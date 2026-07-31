export const plannerKeys = {
  all: ["planner"] as const,
  clubFamily: () => [...plannerKeys.all, "club-family"] as const,
  clubs: () => [...plannerKeys.all, "clubs"] as const,
};
