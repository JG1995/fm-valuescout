import { invokeCommand } from "@/lib/tauri-client";

export function deleteAcademyClass(classId: number) {
  return invokeCommand<void>("delete_academy_class", {
    classId,
    confirmed: true,
  });
}
