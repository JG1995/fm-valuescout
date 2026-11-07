import { invokeCommand } from "@/lib/tauri-client";
import type { CsvImportSummary } from "../types/csv-import-summary";

export function importCsv(path: string) {
  return invokeCommand<CsvImportSummary>("import_csv", { path });
}
