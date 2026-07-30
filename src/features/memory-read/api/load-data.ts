import { invokeCommand } from "@/lib/tauri-client";
import type { LoadDataResult } from "../types/load-data";

/**
 * Request a Load Data scan+ingest.
 * @param maxAccepted `null` = unlimited; a positive integer caps accepted players.
 */
export async function loadData(
  maxAccepted: number | null = null,
): Promise<LoadDataResult> {
  return invokeCommand<LoadDataResult>("load_data", { maxAccepted });
}
