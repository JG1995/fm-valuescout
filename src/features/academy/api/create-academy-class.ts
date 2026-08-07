import { invokeCommand } from "@/lib/tauri-client";
import type { AcademyClass } from "../types/academy";

export function createAcademyClass(classYear: number) {
  return invokeCommand<AcademyClass>("create_academy_class", { classYear });
}
