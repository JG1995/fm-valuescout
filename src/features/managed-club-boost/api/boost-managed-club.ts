import { Channel } from "@tauri-apps/api/core";
import { invokeCommand } from "@/lib/tauri-client";
import type {
  ManagedClubBoostProgress,
  ManagedClubBoostResult,
} from "../types/managed-club-boost";

export function boostManagedClub(
  onProgress: (progress: ManagedClubBoostProgress) => void,
) {
  const channel = new Channel<ManagedClubBoostProgress>();
  channel.onmessage = onProgress;
  return invokeCommand<ManagedClubBoostResult>("boost_managed_club", {
    onProgress: channel,
  });
}
