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
};

export type SearchPlayersPage = {
  players: PlayerSummary[];
  total: number;
};
