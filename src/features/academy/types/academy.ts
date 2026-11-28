export type AcademyClass = {
  id: number;
  classYear: number;
  memberCount: number;
};

export type AcademyCandidate = {
  playerUid: number;
  name: string;
  age: number | null;
  positions: Record<string, number | null>;
  currentClub: string;
};

export type AcademyMemberState = "resolved" | "departed" | "unresolved";

export type AcademyMemberOutcomeStatus = "sold" | "released";

export type AcademyMemberOutcome = {
  status: AcademyMemberOutcomeStatus;
  buyingClub: string | null;
  /** Known for a recorded sale, never for one the snapshot shows. */
  saleFeeEur: number | null;
  /** The user recorded it; otherwise the current snapshot shows it. */
  recorded: boolean;
};

/** An outcome to record. */
export type AcademyMemberOutcomeInput = Omit<AcademyMemberOutcome, "recorded">;

export type AcademyMember = {
  playerUid: number;
  lastKnownName: string;
  currentName: string | null;
  state: AcademyMemberState;
  age: number | null;
  nationalities: string[];
  positions: Record<string, number | null>;
  currentClub: string | null;
  parentClub: string | null;
  teamLevel: string | null;
  pa: number | null;
  determination: number | null;
  heightCm: number | null;
  preferredFoot: string | null;
  reportedCareerAppearances: number | null;
  goals: number | null;
  assists: number | null;
  outcome: AcademyMemberOutcome | null;
  isGraduate: boolean | null;
  /** The club has loaned the member out; `currentClub` is the borrower. */
  loanedOut: boolean;
};

export type AcademyClassDetail = AcademyClass & {
  members: AcademyMember[];
};

export const ACADEMY_VIEWS = ["overview", "graduates", "class"] as const;
export type AcademyView = (typeof ACADEMY_VIEWS)[number];
