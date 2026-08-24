import type { ClubDnaContext } from "../types/club-dna";

export const clubDnaKeys = {
  all: ["club-dna"] as const,
  definition: ({ saveId, contextToken }: ClubDnaContext) =>
    [...clubDnaKeys.all, "definition", { saveId, contextToken }] as const,
};
