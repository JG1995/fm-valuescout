export type MoneyballStatistics = Record<string, number | null>;
export type MoneyballPercentiles = Record<string, number | null>;

export type MoneyballRoleContribution = {
  metricKey: string;
  sourceLabel: string;
  weight: number;
  direction: "higher" | "lower";
  percentile: number | null;
  weightedContribution: number | null;
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

export type MoneyballProfile =
  | { state: "noData" }
  | { state: "needsReimport" }
  | {
      state: "ready";
      askingPriceKind: "single" | "range" | "not_for_sale" | null;
      askingPriceLowerEur: number | null;
      askingPriceUpperEur: number | null;
      starts: number | null;
      substituteAppearances: number | null;
      minutes: number | null;
      statistics: MoneyballStatistics;
      percentiles: MoneyballPercentiles;
      roleCatalogVersion: number;
      roleScores: MoneyballRoleScore[];
    };
