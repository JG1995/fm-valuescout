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
  /** Percentile scores for requested statistics, for a player with a qualified sample. */
  moneyballPercentiles?: Record<string, number | null>;
  /**
   * The in-game date of the kept snapshot whose statistics stand in for a
   * season under 450 minutes. Absent for this season's statistics.
   */
  moneyballStatsDate?: string;
};

export type SearchPlayersPageState =
  | "ready"
  | "no_current_snapshot"
  | "no_shortlist";

export type SearchPlayersPage = {
  /** `no_shortlist` means the requested shortlist is not one of the save's
   * shortlists. A shortlist whose players are not in the snapshot pages as
   * an empty `ready`. */
  state: SearchPlayersPageState;
  players: PlayerSummary[];
  total: number;
};
