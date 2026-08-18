export const plannerKeys = {
  all: ["planner"] as const,
  tactic: () => [...plannerKeys.all, "tactic"] as const,
  tacticOptions: () => [...plannerKeys.all, "tactic-options"] as const,
  depth: () => [...plannerKeys.all, "depth"] as const,
  slotCandidates: () => [...plannerKeys.all, "slot-candidates"] as const,
  slotCandidate: (
    activeSaveId: number,
    team: string,
    laneId: string,
    search: string,
  ) =>
    [
      ...plannerKeys.slotCandidates(),
      activeSaveId,
      team,
      laneId,
      search,
    ] as const,
};
