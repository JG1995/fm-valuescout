export type SaveSummary = {
  id: number;
  contextToken: string;
  name: string;
  isActive: boolean;
  createdAtUtc: string;
  updatedAtUtc: string;
  /** The linked FM save's file name only; the app never receives its directory. */
  linkedSaveFileName: string | null;
  /** The linked FM shortlists, by name case-insensitively, then by file name. */
  playerShortlists: PlayerShortlistSummary[];
};

/** One linked FM shortlist of a save. */
export type PlayerShortlistSummary = {
  id: number;
  /** The list name from the file's last successful read; it may be empty. */
  name: string;
  /** The file name only; the app never receives its directory. */
  fileName: string;
};
