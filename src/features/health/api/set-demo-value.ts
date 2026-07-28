import { invokeCommand } from "@/lib/tauri-client";
import type { DemoValue } from "../types/demo-value";

export async function setDemoValue(value: string): Promise<DemoValue> {
  return invokeCommand<DemoValue>("set_demo_value", { value });
}
