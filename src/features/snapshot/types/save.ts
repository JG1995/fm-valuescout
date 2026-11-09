export type SaveSummary = {
  id: number;
  contextToken: string;
  name: string;
  isActive: boolean;
  createdAtUtc: string;
  updatedAtUtc: string;
  /** The linked FM save's file name only; the app never receives its directory. */
  linkedSaveFileName: string | null;
  /** The linked FM shortlist's list name, or null when no shortlist file is linked. */
  playerShortlistName: string | null;
  /** The linked FM shortlist's file name only, never its directory. */
  playerShortlistFileName: string | null;
};
