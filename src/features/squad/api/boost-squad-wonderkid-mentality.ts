import { invokeCommand } from "@/lib/tauri-client";
import type { SquadPlayerBoostResult } from "../types/squad-player-boost";

export function boostSquadWonderkidMentality() {
  return invokeCommand<SquadPlayerBoostResult>(
    "boost_squad_wonderkid_mentality",
  );
}
