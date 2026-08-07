import { invokeCommand } from "@/lib/tauri-client";
import type { AcademyClassDetail } from "../types/academy";

export function fetchAcademyClass(classId: number) {
  return invokeCommand<AcademyClassDetail>("get_academy_class", { classId });
}
