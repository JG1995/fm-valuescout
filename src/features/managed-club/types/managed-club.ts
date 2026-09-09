export type ManagedClubOption = {
  clubName: string;
  clubUid: number | null;
};

export type ManagedClubStatus = {
  clubName: string | null;
  clubUid: number | null;
  status: "unconfigured" | "available" | "missing";
  unclassifiedPlayerCount: number;
};
