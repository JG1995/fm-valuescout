import { invokeCommand } from "@/lib/tauri-client";

export function setPlayerHiddenInformationRevealed(revealed: boolean) {
  return invokeCommand<boolean>("set_player_hidden_information_revealed", {
    revealed,
  });
}
