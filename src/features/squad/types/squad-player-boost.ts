export type SquadPlayerBoostResult = {
  updated: number;
  skipped: number;
  failed: number;
  recoveryRequired: boolean;
  recoveryMessage: string | null;
};

export type SquadPlayerBoostProgress = {
  processed: number;
  total: number;
  updated: number;
  skipped: number;
  failed: number;
};
