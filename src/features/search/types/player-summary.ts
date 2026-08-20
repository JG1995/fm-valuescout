export type DynamicCellValue = number | string;

export type PlayerSummary = {
  uid: number;
  name: string;
  age: number | null;
  birthYear: number;
  birthDayOfYear: number;
  nationalities: string[];
  club: string | null;
  division: string | null;
  ca: number;
  pa: number;
  marketValueGbp: number | null;
  /** Values for requested non-basic and derived role fields (field id → nullable cell). */
  dynamicValues?: Record<string, DynamicCellValue | null>;
  /** Percentile scores for requested Moneyball performance metrics only. */
  moneyballPercentiles?: Record<string, number | null>;
};

export type SearchPlayersPage = {
  players: PlayerSummary[];
  total: number;
};
