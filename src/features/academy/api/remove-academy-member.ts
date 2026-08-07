import { invokeCommand } from "@/lib/tauri-client";

export function removeAcademyMember(classId: number, playerUid: number) {
  return invokeCommand<void>("remove_academy_member", { classId, playerUid });
}
