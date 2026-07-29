import { invokeCommand } from "@/lib/tauri-client";
import type { SaveSummary } from "../types/save";

export function createSave(name: string) {
  return invokeCommand<SaveSummary>("create_save", { name });
}
