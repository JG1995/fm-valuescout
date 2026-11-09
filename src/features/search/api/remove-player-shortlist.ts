import { invokeCommand } from "@/lib/tauri-client";

/** Deletes one of the save's shortlists with its stored players. */
export function removePlayerShortlist(
  saveId: number,
  contextToken: string,
  shortlistId: number,
): Promise<void> {
  return invokeCommand<void>("remove_player_shortlist", {
    saveId,
    contextToken,
    shortlistId,
  });
}
