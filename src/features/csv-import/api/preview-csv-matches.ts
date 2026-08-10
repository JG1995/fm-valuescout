import { invokeCommand } from "@/lib/tauri-client";
import type { CsvMatchPreview } from "../types/csv-match-preview";

export function previewCsvMatches(path: string) {
  return invokeCommand<CsvMatchPreview>("preview_csv_matches", { path });
}
