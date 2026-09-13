export type DynamicCellValue = number | string;

export type PlayerSummary = {
  uid: number;
  name: string;
  age: number | null;
  birthYear: number;
  birthDayOfYear: number;
  nationalities: string[];
  club: string | null;
  currentClubUid: number | null;
  division: string | null;
  ca: number;
  pa: number;
  marketValueGbp: number | null;
  /** Values for requested non-basic and derived role fields (field id → nullable cell). */
  dynamicValues?: Record<string, DynamicCellValue | null>;
  /** Percentile scores for requested Moneyball performance metrics only. */
  moneyballPercentiles?: Record<string, number | null>;
};

export type SearchPlayersPageState =
  | "ready"
  | "no_current_snapshot"
  | "no_shortlist";

export type SearchPlayersPage = {
  /** Response state mirrors StaffPage: the shortlist probe needs it to tell
   * a save with no stored shortlist apart from a stored shortlist whose
   * entries match no current-snapshot player (both page as empty). */
  state: SearchPlayersPageState;
  players: PlayerSummary[];
  total: number;
};
