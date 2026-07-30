export type LoadDataSnapshotSummary = {
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

export type LoadDataTimings = {
  scanMs: number;
  ingestMs: number;
  totalMs: number;
};

export type LoadDataResult = {
  requestId: string;
  playersFound: number | null;
  scanTruncated: boolean | null;
  maxAccepted: number | null;
  snapshot: LoadDataSnapshotSummary;
  timings: LoadDataTimings;
};
