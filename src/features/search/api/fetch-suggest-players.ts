import { invokeCommand } from "@/lib/tauri-client";
import type { PlayerSuggestHit } from "../types/player-suggest-hit";

/** Matches Rust `DEFAULT_SUGGEST_LIMIT`. */
export const SUGGEST_DEFAULT_LIMIT = 10;

export function fetchSuggestPlayers(
  query: string,
  limit = SUGGEST_DEFAULT_LIMIT,
) {
  return invokeCommand<PlayerSuggestHit[]>("suggest_players", {
    query,
    limit,
  });
}
