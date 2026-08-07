import { invokeCommand } from "@/lib/tauri-client";
import type { AcademyMemberOutcome } from "../types/academy";

export function setAcademyMemberOutcome(
  classId: number,
  playerUid: number,
  outcome: AcademyMemberOutcome | null,
) {
  return invokeCommand<void>("set_academy_member_outcome", {
    classId,
    playerUid,
    outcome,
  });
}
