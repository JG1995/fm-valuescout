import { invokeCommand } from "@/lib/tauri-client";
import type { PlayerShortlistChoice } from "../types/player-shortlist-choice";

/**
 * Adds the FM shortlist file at `path` to the save, or re-reads it when the
 * save already links that path, and stores its players.
 */
export function choosePlayerShortlist(
  saveId: number,
  contextToken: string,
  path: string,
): Promise<PlayerShortlistChoice> {
  return invokeCommand<PlayerShortlistChoice>("choose_player_shortlist", {
    saveId,
    contextToken,
    path,
  });
}
