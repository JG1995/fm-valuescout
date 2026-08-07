import { invokeCommand } from "@/lib/tauri-client";
import type { AcademyClass } from "../types/academy";

export function fetchAcademyClasses() {
  return invokeCommand<AcademyClass[]>("list_academy_classes");
}
