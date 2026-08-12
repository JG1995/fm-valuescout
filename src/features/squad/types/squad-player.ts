export type SquadPlayer = {
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

export type SquadPlayersPage = {
  players: SquadPlayer[];
  total: number;
};
