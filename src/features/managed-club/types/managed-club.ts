export type ManagedClubStatus = {
  clubName: string | null;
  status: "unconfigured" | "available" | "missing";
  unclassifiedPlayerCount: number;
};
