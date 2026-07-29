import { invokeCommand } from "@/lib/tauri-client";
import type { SaveSummary } from "../types/save";

export function renameSave(saveId: number, name: string) {
  return invokeCommand<SaveSummary>("rename_save", { saveId, name });
}
