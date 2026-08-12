export type SquadCurrentAbilityBoostResult = {
  updated: number;
  skipped: number;
  failed: number;
  recoveryRequired: boolean;
  recoveryMessage: string | null;
};
