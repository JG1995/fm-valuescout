export type StaffRoleScore = {
  roleId: string;
  displayName: string;
  score: number | null;
};

export type StaffDetail = {
  uid: number;
  name: string | null;
  age: number | null;
  birthYear: number | null;
  birthDayOfYear: number | null;
  nationalities: string[];
  nationUid: number | null;
  gender: string;
  club: string | null;
  division: string | null;
  ca: number;
  pa: number;
  jobId: number | null;
  weeklyWageGbp: number | null;
  contractExpiryYear: number | null;
  contractExpiryDayOfYear: number | null;
  attributes: Record<string, number | null>;
  hiddenInformationRevealed: boolean;
  roleScores: StaffRoleScore[];
};
