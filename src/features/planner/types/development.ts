/** What Optimize squads builds. */
export type OptimizeFor = "strongest" | "development";

/** What Optimize builds for the save, and where its young players are growing. */
export type PlannerDevelopment = {
  optimizeFor: OptimizeFor;
  /** The oldest age at which a player still counts as developing. */
  maxAge: number;
  players: {
    playerUid: number;
    /** His development slot and the slots that train the same. */
    laneIds: string[];
  }[];
};
