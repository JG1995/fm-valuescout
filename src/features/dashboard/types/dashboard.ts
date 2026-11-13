import type { ReactNode } from "react";

export type DashboardScoreMode = "general" | "moneyball";

/** What a slot's score is made of in the requested mode. */
export type DashboardScoreBasis =
  | "attributes"
  | "moneyball_custom"
  | "moneyball_roles"
  | "unavailable";

export type DashboardLineupPlayer = {
  uid: number;
  name: string;
  score: number;
};

export type DashboardIncumbent = DashboardLineupPlayer & {
  /** The Planner's first-string player, or the best remaining club player. */
  source: "planner" | "best_fit";
};

/** A tactic slot's placement and roles. */
export type DashboardSlotPlacement = {
  laneId: string;
  ipPosition: string;
  ipRoleId: string;
  oopPosition: string;
  oopRoleId: string;
};

export type DashboardLineupSlot = DashboardSlotPlacement & {
  scoreBasis: DashboardScoreBasis;
  incumbent: DashboardIncumbent | null;
  /** The best club player for the slot outside the starting XI. */
  cover: DashboardLineupPlayer | null;
  /** The Planner's pick for the slot when the mode gives them no score there. */
  unscoredPlannerPlayer: string | null;
  /**
   * The compared player who would improve the slot most. Slots answer
   * independently, so one player can be the best signing for several.
   */
  bestSigning: DashboardRecommendation | null;
};

export type DashboardOtherLane = {
  laneId: string;
  improvement: number;
};

export type DashboardRecommendation = {
  uid: number;
  name: string;
  age: number | null;
  club: string | null;
  clubUid: number | null;
  marketValueGbp: number | null;
  /** Season minutes behind a Moneyball score; null in General mode. */
  minutes: number | null;
  laneId: string;
  score: number;
  /** Points above the slot's incumbent, or the whole score for an empty slot. */
  improvement: number;
  otherLanes: DashboardOtherLane[];
};

export type DashboardSignings = {
  tacticIsDefault: boolean;
  /** False only in Moneyball mode when the snapshot has no season stats. */
  scoresAvailable: boolean;
  slots: DashboardLineupSlot[];
  /**
   * How many players are compared: the requested shortlist's players in the
   * snapshot outside the managed club. Realistic signings are not counted, so
   * without a shortlist this is null when the club has any and 0 when it has
   * none. In Moneyball mode a realistic signing also needs season stats.
   */
  candidates: number | null;
  totalRecommendations: number;
  recommendations: DashboardRecommendation[];
};

export type DashboardSigningsRequest = {
  mode: DashboardScoreMode;
  /** One of the save's shortlists, or null for the club's realistic signings. */
  shortlistId: number | null;
  /** Restricts the ranked recommendations to one tactic slot. */
  laneId: string | null;
};

export type DashboardExpiringContract = {
  uid: number;
  name: string;
  age: number | null;
  ca: number;
  marketValueGbp: number | null;
  weeklyWageGbp: number | null;
  /** The contract's last day as `YYYY-MM-DD`. */
  expiryDate: string;
  daysRemaining: number | null;
};

export type DashboardExpiringContracts = {
  /** Months after the in-game date the list covers; null without that date. */
  windowMonths: number | null;
  total: number;
  players: DashboardExpiringContract[];
};

type DashboardYoungPlayer = {
  uid: number;
  name: string;
  age: number;
  currentScore: number;
  potentialScore: number;
};

/** A young player in the tactic slot where their potential score is highest. */
export type DashboardProspect = DashboardYoungPlayer & DashboardSlotPlacement;

export type DashboardYouthXiSlot = DashboardSlotPlacement & {
  /** Null when no young player can play the slot. */
  player: DashboardYoungPlayer | null;
};

export type DashboardProspects = {
  /** Young players with room to grow, highest potential first. */
  players: DashboardProspect[];
  /** Every tactic slot with the young player it takes by potential score. */
  youthXi: DashboardYouthXiSlot[];
};

/**
 * The save, snapshot, and managed club a dashboard read describes. It scopes
 * the cache only: Rust reads the active save itself.
 */
export type DashboardContext = {
  saveId: number;
  contextToken: string;
  snapshotId: number;
  clubName: string;
};

/**
 * Player portraits and club crests. The route supplies them from the graphics
 * feature, because features do not import each other.
 */
export type DashboardGraphics = {
  /** An image that fills its slot, or the slot's fallback when there is none. */
  portrait: (uid: number, name: string) => ReactNode;
  /** A 12px club logo, or nothing when there is none. */
  crest: (clubUid: number | null) => ReactNode;
};
