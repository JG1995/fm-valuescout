export type MoneyballStatistics = Record<string, number | null>;
export type MoneyballPercentiles = Record<string, number | null>;

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
    };
