export type StaffDynamicValue = number | null;

export type StaffSummary = {
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
  dynamicValues?: Record<string, StaffDynamicValue>;
};

export type StaffPageState = "ready" | "no_current_snapshot" | "no_club_family";

export type StaffPage = {
  state: StaffPageState;
  staff: StaffSummary[];
  total: number;
};
