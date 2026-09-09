import { invokeCommand } from "@/lib/tauri-client";
import type { ManagedClubStatus } from "../types/managed-club";

export function setManagedClub(clubName: string, clubUid: number | null) {
  return invokeCommand<ManagedClubStatus>("set_managed_club", {
    clubName,
    clubUid,
  });
}
