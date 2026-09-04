/** Matches the Rust `PlayerShortlistImportSummary` IPC shape (camelCase). */
export type PlayerShortlistImportSummary = {
  totalPlayers: number;
  storedPlayers: number;
  skippedPlayers: number;
};
