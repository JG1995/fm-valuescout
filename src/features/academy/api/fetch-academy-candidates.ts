import { invokeCommand } from "@/lib/tauri-client";
import type { AcademyCandidate } from "../types/academy";

export function fetchAcademyCandidates(search: string) {
  return invokeCommand<AcademyCandidate[]>("list_academy_candidates", {
    search,
  });
}
