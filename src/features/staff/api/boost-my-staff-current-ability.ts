import { Channel } from "@tauri-apps/api/core";
import { invokeCommand } from "@/lib/tauri-client";
import type {
  MyStaffBoostProgress,
  MyStaffBoostResult,
} from "../types/my-staff-boost";

export function boostMyStaffCurrentAbility(
  onProgress: (progress: MyStaffBoostProgress) => void,
) {
  const channel = new Channel<MyStaffBoostProgress>();
  channel.onmessage = onProgress;
  return invokeCommand<MyStaffBoostResult>("boost_my_staff_current_ability", {
    onProgress: channel,
  });
}
