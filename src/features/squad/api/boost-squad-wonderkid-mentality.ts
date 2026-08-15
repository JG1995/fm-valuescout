import { Channel } from "@tauri-apps/api/core";
import { invokeCommand } from "@/lib/tauri-client";
import type {
  SquadPlayerBoostProgress,
  SquadPlayerBoostResult,
} from "../types/squad-player-boost";

export function boostSquadWonderkidMentality(
  onProgress: (progress: SquadPlayerBoostProgress) => void,
) {
  const channel = new Channel<SquadPlayerBoostProgress>();
  channel.onmessage = onProgress;
  return invokeCommand<SquadPlayerBoostResult>(
    "boost_squad_wonderkid_mentality",
    { onProgress: channel },
  );
}
