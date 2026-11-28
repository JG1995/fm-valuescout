import { invokeCommand } from "@/lib/tauri-client";
import type { AcademyMemberOutcomeInput } from "../types/academy";

export function setAcademyMemberOutcome(
  classId: number,
  playerUid: number,
  outcome: AcademyMemberOutcomeInput | null,
) {
  return invokeCommand<void>("set_academy_member_outcome", {
    classId,
    playerUid,
    outcome,
  });
}
