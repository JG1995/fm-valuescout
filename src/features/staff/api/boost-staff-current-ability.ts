import { invokeCommand } from "@/lib/tauri-client";
import type { StaffBoostResult } from "../types/staff-boost";

export function boostStaffCurrentAbility(uid: number) {
  return invokeCommand<StaffBoostResult>("boost_staff_current_ability", {
    uid,
  });
}
