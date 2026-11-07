export type SaveSummary = {
  id: number;
  contextToken: string;
  name: string;
  isActive: boolean;
  createdAtUtc: string;
  updatedAtUtc: string;
  /** The linked FM save's file name only; the app never receives its directory. */
  linkedSaveFileName: string | null;
};
