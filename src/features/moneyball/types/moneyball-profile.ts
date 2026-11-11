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

export type MoneyballComparisonBasis =
  | {
      kind: "available";
      naturalPositions: string[];
      comparisonPlayerCount: number;
    }
  | { kind: "unavailableNoNaturalPosition" };

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

export type MoneyballProfile =
  | { state: "noData" }
  | { state: "needsReimport" }
  | {
      state: "ready";
      starts: number | null;
      substituteAppearances: number | null;
      minutes: number | null;
      statistics: MoneyballStatistics;
      percentiles: MoneyballPercentiles | null;
      roleCatalogVersion: number | null;
      roleScores: MoneyballRoleScore[] | null;
      comparisonBasis: MoneyballComparisonBasis;
      /** Null when role scores are not moved for league strength. */
      leagueAdjustment: MoneyballLeagueBreakdown | null;
    };
