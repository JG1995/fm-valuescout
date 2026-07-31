import { invokeCommand } from "@/lib/tauri-client";
import type { PlayerDetail } from "../types/player-detail";

export function fetchGetPlayer(uid: number) {
  return invokeCommand<PlayerDetail | null>("get_player", { uid });
}
