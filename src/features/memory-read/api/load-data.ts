import { invokeCommand } from "@/lib/tauri-client";
import type { LoadDataResult } from "../types/load-data";

export async function loadData(): Promise<LoadDataResult> {
  return invokeCommand<LoadDataResult>("load_data");
}
