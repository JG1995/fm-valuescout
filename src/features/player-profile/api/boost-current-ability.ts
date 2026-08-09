import { invokeCommand } from "@/lib/tauri-client";
import type { PlayerBoostResult } from "../types/player-boost";

export function boostCurrentAbility(uid: number) {
  return invokeCommand<PlayerBoostResult>("boost_current_ability", { uid });
}
