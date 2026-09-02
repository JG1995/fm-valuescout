import { Channel } from "@tauri-apps/api/core";
import { invokeCommand } from "@/lib/tauri-client";
import type { LoadDataProgress, LoadDataResult } from "../types/load-data";

/**
 * Request a Load Data scan+ingest with command-scoped best-effort progress.
 * @param maxAccepted `null` = unlimited; a positive integer caps accepted players.
 * @param onProgress callback receiving ordered phase events bound to the captured save context.
 */
export async function loadData(
  maxAccepted: number | null = null,
  onProgress: (progress: LoadDataProgress) => void = () => undefined,
): Promise<LoadDataResult> {
  const channel = new Channel<LoadDataProgress>();
  channel.onmessage = onProgress;
  return invokeCommand<LoadDataResult>("load_data", {
    maxAccepted,
    onProgress: channel,
  });
}
