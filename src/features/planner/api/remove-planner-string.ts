import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";

export function removePlannerString(
  stringId: number,
  confirmPopulated: boolean,
) {
  return invokeCommand<PlannerDepth>("remove_planner_string", {
    stringId,
    confirmPopulated,
  });
}
