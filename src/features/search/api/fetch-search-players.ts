import { invokeCommand } from "@/lib/tauri-client";
import type { SearchPlayersPage } from "../types/player-summary";

/** Matches Rust `DEFAULT_PAGE_LIMIT` — window size for virtualized fetch. */
export const SEARCH_PAGE_SIZE = 50;

export function fetchSearchPlayers(offset = 0, limit = SEARCH_PAGE_SIZE) {
  return invokeCommand<SearchPlayersPage>("search_players", { offset, limit });
}
