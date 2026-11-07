import { invokeCommand } from "@/lib/tauri-client";
import type { SaveSummary } from "../types/save";

/** Links the FM save file at `path` to the save, or removes the link when `path` is null. */
export function setSaveFileLink(
  saveId: number,
  contextToken: string,
  path: string | null,
) {
  return invokeCommand<SaveSummary>("set_save_file_link", {
    saveId,
    contextToken,
    path,
  });
}
