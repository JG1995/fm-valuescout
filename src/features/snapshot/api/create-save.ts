import { invokeCommand } from "@/lib/tauri-client";
import type { SaveSummary } from "../types/save";

export function createSave(name: string, saveFilePath: string | null = null) {
  return invokeCommand<SaveSummary>("create_save", { name, saveFilePath });
}
