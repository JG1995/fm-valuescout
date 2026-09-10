import { invokeCommand } from "@/lib/tauri-client";
import type { GraphicsStatus } from "../types/graphics";

export function fetchGraphicsStatus() {
  return invokeCommand<GraphicsStatus>("get_graphics_status");
}
