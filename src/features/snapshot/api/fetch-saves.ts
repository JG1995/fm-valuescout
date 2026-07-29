import { invokeCommand } from "@/lib/tauri-client";
import type { SaveSummary } from "../types/save";

export function fetchSaves() {
  return invokeCommand<SaveSummary[]>("list_saves");
}
