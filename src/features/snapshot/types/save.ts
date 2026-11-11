export type SaveSummary = {
  id: number;
  contextToken: string;
  name: string;
  isActive: boolean;
  createdAtUtc: string;
  updatedAtUtc: string;
  /** The linked FM save's file name only; the app never receives its directory. */
  linkedSaveFileName: string | null;
  /** The save's player shortlists, by name case-insensitively, then by file name. */
  playerShortlists: PlayerShortlistSummary[];
};

/** One player shortlist of a save: a linked FM file, or a list read from the linked FM save. */
export type PlayerShortlistSummary = {
  id: number;
  /** The list name from the last successful read; a file's may be empty. */
  name: string;
  /**
   * The linked file's name only, as the app never receives its directory, or
   * null for a list read from the linked FM save.
   */
  fileName: string | null;
};
