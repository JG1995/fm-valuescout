export type SnapshotSummary = {
  id: number;
  contextToken: string;
  saveId: number;
  schemaVersion: number;
  generatedAtUtc: string;
  gameVersion: string;
  supportedGameVersion: string;
  bridgeVersion: string;
  protocolVersion: number;
  gameDate: string | null;
  gameDateSource: string;
  scanTruncated: boolean;
  maxAccepted: number | null;
  playerCount: number;
  loadedAtUtc: string;
  /** In-game date of the save whose season stats the Moneyball cohort holds. */
  moneyballStatsGameDate: string | null;
};

export type SnapshotMetadata = {
  id: number;
  contextToken: string;
  saveId: number;
  customName: string | null;
  gameDate: string | null;
  gameDateSource: string;
  playerCount: number;
  loadedAtUtc: string;
  isCurrent: boolean;
  /** Load Data replaces every snapshot of the save that is not kept. */
  isKept: boolean;
};

export type SnapshotDeleteResult = {
  deletedSnapshotId: number;
  saveId: number;
  currentSnapshotId: number | null;
};
