export type CsvImportFormat = "youthTracker";

export type CsvImportSummary = {
  format: CsvImportFormat;
  totalPlayers: number;
  storedPlayers: number;
  skippedPlayers: number;
};
