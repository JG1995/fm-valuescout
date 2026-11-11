export type PlannerContext = {
  saveId: number;
  contextToken: string;
};

export const plannerKeys = {
  all: ["planner"] as const,
  tactic: (context: PlannerContext) =>
    [...plannerKeys.all, "tactic", context] as const,
  tacticOptions: (context: PlannerContext) =>
    [...plannerKeys.all, "tactic-options", context] as const,
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
  // Unassigned players go stale whenever the slot candidates do, so they share
  // that prefix and every invalidation of it.
  unassignedPlayers: (activeSaveId: number) =>
    [...plannerKeys.slotCandidates(), "unassigned", activeSaveId] as const,
  roleReferences: () => [...plannerKeys.all, "role-reference"] as const,
  roleReference: (activeSaveId: number, phase: string, scoreBasis: string) =>
    [...plannerKeys.roleReferences(), activeSaveId, phase, scoreBasis] as const,
};
