import { invokeCommand } from "@/lib/tauri-client";
import type { ClubDnaContext, ClubDnaRemoveResult } from "../types/club-dna";

export function removeClubDna({ saveId, contextToken }: ClubDnaContext) {
  return invokeCommand<ClubDnaRemoveResult>("remove_club_dna", {
    saveId,
    contextToken,
  });
}
