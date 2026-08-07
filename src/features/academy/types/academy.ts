export type AcademyClass = {
  id: number;
  classYear: number;
  isAutomatic: boolean;
  memberCount: number;
};

export type AcademyCandidate = {
  playerUid: number;
  name: string;
  age: number | null;
  positions: Record<string, number>;
  currentClub: string;
};

export type AcademyMemberState = "resolved" | "departed" | "unresolved";

export type AcademyMemberOutcomeStatus = "sold" | "released";

export type AcademyMemberOutcome = {
  status: AcademyMemberOutcomeStatus;
  buyingClub: string | null;
  saleFeeEur: number | null;
};

export type AcademyMember = {
  playerUid: number;
  lastKnownName: string;
  currentName: string | null;
  state: AcademyMemberState;
  age: number | null;
  nationalities: string[];
  positions: Record<string, number>;
  currentClub: string | null;
  parentClub: string | null;
  teamLevel: string | null;
  pa: number | null;
  determination: number | null;
  heightCm: number | null;
  preferredFoot: string | null;
  seniorLeagueAppearances: number | null;
  goals: number | null;
  assists: number | null;
  internationalCaps: number | null;
  outcome: AcademyMemberOutcome | null;
  isGraduate: boolean | null;
};

export type AcademyClassDetail = AcademyClass & {
  members: AcademyMember[];
};

export const ACADEMY_VIEWS = ["overview", "graduates", "class"] as const;
export type AcademyView = (typeof ACADEMY_VIEWS)[number];
