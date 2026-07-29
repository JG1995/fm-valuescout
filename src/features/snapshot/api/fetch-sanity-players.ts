import { invokeCommand } from "@/lib/tauri-client";
import type { PlayerSanityRow } from "../types/player-sanity";

export function fetchSanityPlayers(limit = 20) {
  return invokeCommand<PlayerSanityRow[]>("list_sanity_players", { limit });
}
