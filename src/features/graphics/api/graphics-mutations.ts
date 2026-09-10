import { invokeCommand } from "@/lib/tauri-client";
import type { GraphicsStatus } from "../types/graphics";

export const chooseGraphicsRoot = () =>
  invokeCommand<GraphicsStatus>("choose_graphics_root");
export const clearGraphicsRoot = () =>
  invokeCommand<GraphicsStatus>("clear_graphics_root");
export const rescanGraphics = () =>
  invokeCommand<GraphicsStatus>("rescan_graphics");
