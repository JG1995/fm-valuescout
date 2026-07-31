import { invokeCommand } from "@/lib/tauri-client";
import type { ClubFamily, ClubSourceInput } from "../types/club-family";

export function savePlannerClubFamily(
  primaryClub: string,
  sources: ClubSourceInput[],
) {
  return invokeCommand<ClubFamily>("save_planner_club_family", {
    primaryClub,
    sources,
  });
}
