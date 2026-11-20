import type { PlannerTeam } from "./team";

/** The level of a player's FM team. */
export type FmTeamLevel = "senior" | "reserve" | "youth";

export const FM_TEAM_LEVEL_NAMES: Record<FmTeamLevel, string> = {
  senior: "First team",
  reserve: "Reserves",
  youth: "Youth",
};

/**
 * A Planner-placed player whose FM team is at another level than his squad, or
 * whose FM team the snapshot does not know.
 */
export type PlannerFmMove = {
  playerUid: number;
  name: string;
  age: number | null;
  /** Null when unknown; the bridge then reads his team from FM. */
  fromLevel: FmTeamLevel | null;
  toTeam: PlannerTeam;
  toTeamName: string;
  /** Older than the squad's age limit. FM does not check age, so he moves anyway. */
  overAge: boolean;
};

export type PlannerFmMoves = {
  moves: PlannerFmMove[];
  /** Why the moves cannot be applied now; null when they can. */
  unavailableReason: string | null;
};

export type PlannerFmMoveResult = {
  moved: number[];
  /** Players FM already had in that team. */
  unchanged: number[];
  failed: { playerUid: number; name: string; reason: string }[];
  /** The error that stopped the moves early; FM may have made some first. */
  error: string | null;
};
