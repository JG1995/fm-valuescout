import { invokeCommand, TauriCommandError } from "@/lib/tauri-client";
import type { SeasonStatsOutcome } from "../types/load-data";

/** A Refresh stats result: the stats outcome, or no read because Load Data was running. */
export type RefreshSeasonStatsResult =
  | SeasonStatsOutcome
  | { status: "loadDataRunning" };

/** Re-reads the linked FM save file of the captured save into its current snapshot. */
export async function refreshSeasonStats(
  saveId: number,
  contextToken: string,
): Promise<RefreshSeasonStatsResult> {
  try {
    return await invokeCommand<SeasonStatsOutcome>("refresh_season_stats", {
      saveId,
      contextToken,
    });
  } catch (error) {
    if (
      error instanceof TauriCommandError &&
      error.kind === "loadDataRunning"
    ) {
      return { status: "loadDataRunning" };
    }
    throw error;
  }
}
