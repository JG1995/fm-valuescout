import { invokeCommand } from "@/lib/tauri-client";

export function fetchPlannerClubs() {
  return invokeCommand<string[]>("list_planner_clubs");
}
