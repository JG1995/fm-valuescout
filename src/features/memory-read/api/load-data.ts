import { invokeCommand } from "@/lib/tauri-client";
import type { LoadDataResult } from "../types/load-data";

function parseLoadDataError(error: unknown): string {
  if (typeof error === "object" && error !== null && "message" in error) {
    const message = (error as { message?: string }).message;
    if (message) {
      return message;
    }
  }

  return String(error);
}

export async function loadData(): Promise<LoadDataResult> {
  try {
    return await invokeCommand<LoadDataResult>("load_data");
  } catch (error) {
    throw new Error(parseLoadDataError(error));
  }
}
