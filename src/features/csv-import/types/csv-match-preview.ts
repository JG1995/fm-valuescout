export type CsvPreviewFormat = "youthTracker" | "moneyball";

export type CsvMatchPreview = {
  format: CsvPreviewFormat;
  totalPlayers: number;
  matchedPlayers: number;
  unmatchedPlayers: number;
};
