export type SquadPlayerBoostResult = {
  updated: number;
  skipped: number;
  failed: number;
  recoveryRequired: boolean;
  recoveryMessage: string | null;
};
