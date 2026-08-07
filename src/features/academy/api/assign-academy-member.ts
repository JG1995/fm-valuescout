import { invokeCommand } from "@/lib/tauri-client";

export function assignAcademyMember(classId: number, playerUid: number) {
  return invokeCommand<void>("assign_academy_member", { classId, playerUid });
}
