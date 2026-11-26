export type MoneyballStatistics = Record<string, number | null>;
export type MoneyballPercentiles = Record<string, number | null>;

export type MoneyballRoleContribution = {
  metricKey: string;
  sourceLabel: string;
  weight: number;
  direction: "higher" | "lower";
  percentile: number | null;
  weightedContribution: number | null;
  availableFromSource: boolean;
};

export type MoneyballRoleScore = {
  roleId: string;
  displayName: string;
  phase: string;
  positionFamily: string;
  positionTags: string[];
  score: number | null;
  contributions: MoneyballRoleContribution[];
};

/**
 * The player's percentiles among one position family's peers: the players
 * with `minimumMinutes` this season rated 15 or higher at its positions.
 */
export type MoneyballFamilyComparison = {
  /** A role catalog position family ID. */
  family: string;
  peers: number;
  percentiles: MoneyballPercentiles;
};

/**
 * What moved a player's role scores for the league he plays in. A strength is
 * the mean current ability of the best players at the league's clubs.
 */
export type MoneyballLeagueBreakdown = {
  /** The player's league, or null when the snapshot records none for him. */
  division: string | null;
  strength: number;
  /** Too few of the league's clubs are measured to take its strength as measured. */
  estimated: boolean;
  referenceDivision: string;
  referenceStrength: number;
  /** Score points added to every role score: negative in a weaker league. */
  shift: number;
};

/** The active save's choice to move Moneyball scores for league strength. */
export type MoneyballLeagueAdjustment = {
  enabled: boolean;
  /** The league scores are moved to, or null until Load Data measures leagues. */
  referenceDivision: string | null;
};

export type MoneyballBorrowedStats = {
  /** The in-game date of the kept snapshot's stats. */
  statsDate: string;
  /** The player's minutes this season. */
  currentMinutes: number;
};

export type MoneyballProfile =
  | { state: "noData" }
  | { state: "needsReimport" }
  | {
      state: "ready";
      starts: number | null;
      substituteAppearances: number | null;
      minutes: number | null;
      statistics: MoneyballStatistics;
      /**
       * Each family the player can be compared in, his best position's first.
       * Empty when he is rated at no position.
       */
      comparisons: MoneyballFamilyComparison[];
      /** The fewest minutes that make a player one of a family's peers. */
      minimumMinutes: number;
      roleCatalogVersion: number | null;
      /** Each role ranked among its own family's peers. */
      roleScores: MoneyballRoleScore[] | null;
      /** Null when role scores are not moved for league strength. */
      leagueAdjustment: MoneyballLeagueBreakdown | null;
      /**
       * Set when the player has under `minimumMinutes` this season, so the
       * stats, percentiles, and role scores are those of the latest kept
       * snapshot in which he has enough, ranked among that snapshot's peers.
       */
      borrowedStats: MoneyballBorrowedStats | null;
    };
