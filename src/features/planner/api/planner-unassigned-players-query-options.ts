import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type { PlannerUnassignedPlayer } from "../types/depth";
import { plannerKeys } from "./planner-keys";

/** The managed club's players in no squad, best current fit first. */
export function plannerUnassignedPlayersQueryOptions(activeSaveId: number) {
  return queryOptions({
    queryKey: plannerKeys.unassignedPlayers(activeSaveId),
    queryFn: () =>
      invokeCommand<PlannerUnassignedPlayer[]>(
        "get_planner_unassigned_players",
      ),
  });
}
