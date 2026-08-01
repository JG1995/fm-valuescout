import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";

export function movePlannerPlayer(
  stringId: number,
  laneId: string,
  playerUid: number,
) {
  return invokeCommand<PlannerDepth>("move_planner_player", {
    stringId,
    laneId,
    playerUid,
  });
}
