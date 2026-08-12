import { invokeCommand } from "@/lib/tauri-client";
import type { SquadCurrentAbilityBoostResult } from "../types/squad-current-ability-boost";

export function boostSquadCurrentAbility() {
  return invokeCommand<SquadCurrentAbilityBoostResult>(
    "boost_squad_current_ability",
  );
}
