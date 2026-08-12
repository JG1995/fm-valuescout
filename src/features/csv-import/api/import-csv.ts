import { invokeCommand } from "@/lib/tauri-client";
import type {
  CsvImportFormat,
  CsvImportSummary,
} from "../types/csv-import-summary";

export function importCsv(path: string, expectedFormat?: CsvImportFormat) {
  return invokeCommand<CsvImportSummary>("import_csv", {
    path,
    ...(expectedFormat ? { expectedFormat } : {}),
  });
}
