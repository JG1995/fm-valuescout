import { invokeCommand, TauriCommandError } from "@/lib/tauri-client";
import type { HealthStatus } from "../types/health-status";
import { isHealthSimulateErrorEnabled } from "./health-simulate-error";

export async function fetchHealthStatus(): Promise<HealthStatus> {
  if (isHealthSimulateErrorEnabled()) {
    throw new TauriCommandError("Simulated health check failure");
  }

  return invokeCommand<HealthStatus>("get_status");
}
