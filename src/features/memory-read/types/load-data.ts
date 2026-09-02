export type LoadDataPhase =
  | "scan"
  | "preparing"
  | "scoring"
  | "saving"
  | "finalizing";

export type LoadDataProgress = {
  saveId: number;
  contextToken: string;
  phase: LoadDataPhase;
} & (
  | { completed: number; total: number }
  | { completed?: never; total?: never }
);

export type LoadDataSnapshotSummary = {
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
};

export type LoadDataTimings = {
  scanMs: number;
  prepareMs: number;
  scoringMs: number;
  saveMs: number;
  finalizeMs: number;
  totalMs: number;
  ingestMs: number;
};

export type LoadDataResult = {
  requestId: string;
  playersFound: number | null;
  scanTruncated: boolean | null;
  maxAccepted: number | null;
  storedSnapshot: LoadDataSnapshotSummary;
  effectiveSnapshot: LoadDataSnapshotSummary;
  timings: LoadDataTimings;
};
