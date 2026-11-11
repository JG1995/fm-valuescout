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

/**
 * What a read of the linked FM save did with its player shortlists, or with
 * its staff shortlist. Only `read` replaced what was stored; `count` is how
 * many player shortlists hold players, or how many staff are shortlisted.
 */
export type SaveShortlistsOutcome =
  | { status: "read"; count: number }
  | { status: "notRead" }
  | { status: "failed"; kind: SeasonStatsErrorKind };

/** The season stats and shortlists read from the active save's linked FM save file. */
export type SeasonStatsOutcome = {
  elapsedMs: number;
  shortlists: SaveShortlistsOutcome;
  staffShortlist: SaveShortlistsOutcome;
} & (
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

/** A shortlist reader kind, or `database` when the shortlist could not be stored. */
export type PlayerShortlistReadErrorKind =
  | "unreadable"
  | "too_large"
  | "not_fm_shortlist"
  | "not_player_shortlist"
  | "unsupported_format"
  | "database";

/**
 * The re-read of the active save's linked FM shortlist files. A re-read
 * discarded because the save or shortlist changed counts as neither, so a
 * save without linked files reports `{ read: 0, failed: [] }`.
 */
export type PlayerShortlistsOutcome = {
  /** How many shortlists had their name and players replaced. */
  read: number;
  /** Each shortlist that kept its previous players. */
  failed: { kind: PlayerShortlistReadErrorKind; fileName: string | null }[];
};

export type LoadDataResult = {
  requestId: string;
  playersFound: number | null;
  storedSnapshot: LoadDataSnapshotSummary;
  effectiveSnapshot: LoadDataSnapshotSummary;
  timings: LoadDataTimings;
  stats: SeasonStatsOutcome;
  shortlists: PlayerShortlistsOutcome;
};
