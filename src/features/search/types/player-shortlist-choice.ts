/** Matches the Rust `PlayerShortlistChoice` IPC shape (camelCase). */
export type PlayerShortlistChoice = {
  /** The added or updated shortlist, which the route selects. */
  id: number;
  name: string;
  fileName: string;
  totalPlayers: number;
  matchedPlayers: number;
};

/** The `kind` of a rejected choice or removal; Rust never sends the file's path. */
export type PlayerShortlistErrorKind =
  | "unreadable"
  | "too_large"
  | "not_fm_shortlist"
  | "not_player_shortlist"
  | "unsupported_format"
  | "no_snapshot"
  | "no_matching_players"
  | "stale_context"
  | "database";
