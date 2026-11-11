import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerDepth } from "../types/depth";

/** Pins the player in a cell so Optimize keeps him, or hands the cell back to it. */
export function setPlannerAssignmentPinned(
  stringId: number,
  laneId: string,
  pinned: boolean,
) {
  return invokeCommand<PlannerDepth>("set_planner_assignment_pinned", {
    stringId,
    laneId,
    pinned,
  });
}
