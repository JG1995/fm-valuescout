type BatchBoostResult = {
  updated: number;
  skipped: number;
  failed: number;
  recoveryRequired: boolean;
  recoveryMessage: string | null;
};

export type ManagedClubBoostPhase =
  | "wonderkids"
  | "playerCurrentAbility"
  | "staffCurrentAbility";

export type ManagedClubBoostProgress = {
  phase: ManagedClubBoostPhase;
  processed: number;
  total: number;
  updated: number;
  skipped: number;
  failed: number;
};

export type ManagedClubBoostResult = {
  wonderkids: BatchBoostResult;
  playerCurrentAbility: BatchBoostResult | null;
  staffCurrentAbility: BatchBoostResult | null;
};
