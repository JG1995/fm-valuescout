export type SnapshotSummary = {
  id: number;
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
};

export type SnapshotDeleteResult = {
  deletedSnapshotId: number;
  saveId: number;
  currentSnapshotId: number | null;
};
