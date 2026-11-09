import { invokeCommand } from "@/lib/tauri-client";

/** Unlinks the save's shortlist file and clears its stored players. */
export function removePlayerShortlist(
  saveId: number,
  contextToken: string,
): Promise<void> {
  return invokeCommand<void>("remove_player_shortlist", {
    saveId,
    contextToken,
  });
}
