import { Channel } from "@tauri-apps/api/core";
import { invokeCommand } from "@/lib/tauri-client";
import type { LoadDataProgress, LoadDataResult } from "../types/load-data";

/**
 * Request a Load Data scan+ingest bound to the invocation-time save context.
 * @param maxAccepted `null` = unlimited; a positive integer caps accepted players.
 * @param saveId exact save id captured at invocation time.
 * @param contextToken exact immutable context token captured at invocation time.
 * @param onProgress callback receiving ordered phase events bound to the captured save context.
 */
export async function loadData(
  maxAccepted: number | null,
  saveId: number,
  contextToken: string,
  onProgress: (progress: LoadDataProgress) => void = () => undefined,
): Promise<LoadDataResult> {
  const channel = new Channel<LoadDataProgress>();
  channel.onmessage = onProgress;
  return invokeCommand<LoadDataResult>("load_data", {
    saveId,
    contextToken,
    maxAccepted,
    onProgress: channel,
  });
}
