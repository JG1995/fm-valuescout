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

/**
 * One of a slot's recruitment filter rules, in Search's filter IPC shape. The
 * route describes it, since only Search knows its fields.
 */
export type DashboardRecruitmentFilter = {
  field: string;
  op: string;
  value: string | number | boolean | number[] | string[];
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
  /** Rules a candidate must pass to be recommended for the slot. */
  recruitmentFilters: DashboardRecruitmentFilter[];
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
  /**
   * The in-game date of the kept snapshot's stats behind a Moneyball score,
   * borrowed because the player has under `minimumMinutes` this season.
   */
  borrowedStatsDate: string | null;
  laneId: string;
  /** The score in the slot after `agePenalty`. */
  score: number;
  /** Points above the slot's incumbent, or the whole score for an empty slot. */
  improvement: number;
  /** Points the score lost for the player's age; 0 at 28 or under. */
  agePenalty: number;
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
  /**
   * Candidates who would improve a considered slot but are held back from
   * every slot they would improve: by its recruitment filters, or by too few
   * minutes.
   */
  filteredOut: number;
  /** Of `filteredOut`, those under `minimumMinutes` minutes. */
  underMinimumMinutes: number;
  /** The competitive minutes a candidate needs while season stats apply. */
  minimumMinutes: number;
  /** Whether the snapshot has season stats, or a kept snapshot has some to lend. */
  seasonStatsAvailable: boolean;
  /**
   * Whether season stats applied: candidates needed `minimumMinutes` and the
   * slots' season-stat filters ran. Always in Moneyball mode with stats.
   */
  seasonStatsApplied: boolean;
};

export type DashboardSigningsRequest = {
  mode: DashboardScoreMode;
  /** One of the save's shortlists, or null for the club's realistic signings. */
  shortlistId: number | null;
  /** Restricts the ranked recommendations to one tactic slot. */
  laneId: string | null;
  /** Whether General mode applies season stats; Moneyball mode always does. */
  seasonStats: boolean;
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
  /** The club has loaned the player out. */
  loanedOut: boolean;
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

/** A developing player in their development slot. */
export type DashboardProspect = DashboardYoungPlayer & DashboardSlotPlacement;

export type DashboardYouthXiSlot = DashboardSlotPlacement & {
  /** Null when no young player can take the slot. */
  player: DashboardYoungPlayer | null;
};

export type DashboardProspects = {
  /** Developing players with room to grow, highest potential first. */
  players: DashboardProspect[];
  /**
   * Every tactic slot with the young player it takes by potential score, each
   * developing player kept to their development slot.
   */
  youthXi: DashboardYouthXiSlot[];
  /** The oldest age at which the save counts a player as developing. */
  maxAge: number;
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
