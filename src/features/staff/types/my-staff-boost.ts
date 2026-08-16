export type MyStaffBoostResult = {
  updated: number;
  skipped: number;
  failed: number;
  recoveryRequired: boolean;
  recoveryMessage: string | null;
};

export type MyStaffBoostProgress = {
  processed: number;
  total: number;
  updated: number;
  skipped: number;
  failed: number;
};
