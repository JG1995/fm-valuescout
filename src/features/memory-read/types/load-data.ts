export type LoadDataPhase =
  | "scan"
  | "preparing"
  | "scoring"
  | "saving"
  | "finalizing"
  | "seasonStats";

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
  statsMs: number;
  totalMs: number;
  ingestMs: number;
};

/** A save-file error kind, or `database` when the cohort could not be stored. */
export type SeasonStatsErrorKind =
  | "not_found"
  | "unreadable"
  | "not_fm_save"
  | "unsupported_game"
  | "unsupported_build"
  | "corrupt"
  | "out_of_bounds"
  | "changed_during_read"
  | "database";

/** The season stats read from the active save's linked FM save file. */
export type SeasonStatsOutcome = { elapsedMs: number } & (
  | {
      status: "attached";
      matchedPlayers: number;
      unmatchedPlayers: number;
      gameDate: string;
    }
  | { status: "noMatches"; unmatchedPlayers: number; gameDate: string }
  | { status: "notLinked" }
  | { status: "noSnapshot" }
  | { status: "staleContext" }
  | { status: "failed"; kind: SeasonStatsErrorKind; fileName: string | null }
);

export type LoadDataResult = {
  requestId: string;
  playersFound: number | null;
  storedSnapshot: LoadDataSnapshotSummary;
  effectiveSnapshot: LoadDataSnapshotSummary;
  timings: LoadDataTimings;
  stats: SeasonStatsOutcome;
};
