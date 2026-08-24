import { invokeCommand } from "@/lib/tauri-client";
import type { ClubDnaContext, ClubDnaUpsertResult } from "../types/club-dna";

export function setClubDna(
  { saveId, contextToken }: ClubDnaContext,
  attributeIds: string[],
) {
  return invokeCommand<ClubDnaUpsertResult>("set_club_dna", {
    saveId,
    contextToken,
    attributeIds,
  });
}
