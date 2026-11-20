export type SquadPlayer = {
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
  suggestedTraining: string | null;
  dynamicValues?: Record<string, number | string | null>;
  /** The club has loaned the player out; `club` and `division` are the borrower's. */
  loanedOut: boolean;
};

export type SquadPlayersPage = {
  players: SquadPlayer[];
  total: number;
};
