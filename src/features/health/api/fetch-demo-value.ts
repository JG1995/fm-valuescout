import { invokeCommand } from "@/lib/tauri-client";
import type { DemoValue } from "../types/demo-value";

export async function fetchDemoValue(): Promise<DemoValue> {
  return invokeCommand<DemoValue>("get_demo_value");
}
