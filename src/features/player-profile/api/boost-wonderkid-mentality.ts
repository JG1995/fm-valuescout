import { invokeCommand } from "@/lib/tauri-client";
import type { PlayerBoostResult } from "../types/player-boost";

export function boostWonderkidMentality(uid: number) {
  return invokeCommand<PlayerBoostResult>("boost_wonderkid_mentality", { uid });
}
