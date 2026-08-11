import { invokeCommand } from "@/lib/tauri-client";
import type { SaveSummary } from "../types/save";

export type SaveDeleteResult = {
  deletedSaveId: number;
  deletedWasActive: boolean;
  activeSave: SaveSummary;
};

export function deleteSave(saveId: number, contextToken: string) {
  return invokeCommand<SaveDeleteResult>("delete_save", {
    saveId,
    contextToken,
  });
}
