import { invokeCommand } from "@/lib/tauri-client";
import type { MoneyballProfile } from "../types/moneyball-profile";

export function fetchPlayerMoneyball(uid: number) {
  return invokeCommand<MoneyballProfile | null>("get_player_moneyball", {
    uid,
  });
}
