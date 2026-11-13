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

/** One player shortlist of a save, read from its linked FM save. */
export type PlayerShortlistSummary = {
  id: number;
  /** The list's name in FM, unique in the save; "(Default)" for FM's unnamed list. */
  name: string;
};
