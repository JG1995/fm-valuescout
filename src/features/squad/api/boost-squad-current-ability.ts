import { invokeCommand } from "@/lib/tauri-client";
import type { SquadPlayerBoostResult } from "../types/squad-player-boost";

export function boostSquadCurrentAbility() {
  return invokeCommand<SquadPlayerBoostResult>("boost_squad_current_ability");
}
