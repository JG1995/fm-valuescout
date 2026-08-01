import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";

export function assignPlannerPlayer(
  stringId: number,
  laneId: string,
  playerUid: number,
) {
  return invokeCommand<PlannerDepth>("assign_planner_player", {
    stringId,
    laneId,
    playerUid,
  });
}
