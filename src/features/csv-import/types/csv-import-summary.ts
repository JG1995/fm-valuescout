export type CsvImportFormat = "youthTracker" | "moneyball";

export type CsvImportSummary = {
  format: CsvImportFormat;
  totalPlayers: number;
  storedPlayers: number;
  skippedPlayers: number;
};
