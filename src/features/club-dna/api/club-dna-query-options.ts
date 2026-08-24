import { queryOptions } from "@tanstack/react-query";
import { invokeCommand } from "@/lib/tauri-client";
import type { ClubDnaContext, ClubDnaDefinition } from "../types/club-dna";
import { clubDnaKeys } from "./club-dna-keys";

export function getClubDna({ saveId, contextToken }: ClubDnaContext) {
  return invokeCommand<ClubDnaDefinition | null>("get_club_dna", {
    saveId,
    contextToken,
  });
}

export function clubDnaQueryOptions(context: ClubDnaContext) {
  return queryOptions({
    queryKey: clubDnaKeys.definition(context),
    queryFn: () => getClubDna(context),
  });
}
