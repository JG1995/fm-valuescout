import { Channel } from "@tauri-apps/api/core";
import { invokeCommand } from "@/lib/tauri-client";
import type {
  SquadPlayerBoostProgress,
  SquadPlayerBoostResult,
} from "../types/squad-player-boost";

export function boostSquadCurrentAbility(
  onProgress: (progress: SquadPlayerBoostProgress) => void,
) {
  const channel = new Channel<SquadPlayerBoostProgress>();
  channel.onmessage = onProgress;
  return invokeCommand<SquadPlayerBoostResult>("boost_squad_current_ability", {
    onProgress: channel,
  });
}
