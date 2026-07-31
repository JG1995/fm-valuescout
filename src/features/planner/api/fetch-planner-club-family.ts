import { invokeCommand } from "@/lib/tauri-client";
import type { ClubFamily } from "../types/club-family";

export function fetchPlannerClubFamily() {
  return invokeCommand<ClubFamily>("get_planner_club_family");
}
