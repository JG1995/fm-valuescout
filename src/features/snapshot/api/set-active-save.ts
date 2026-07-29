import { invokeCommand } from "@/lib/tauri-client";
import type { SaveSummary } from "../types/save";

export function setActiveSave(saveId: number) {
  return invokeCommand<SaveSummary>("set_active_save", { saveId });
}
