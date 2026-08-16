import { invokeCommand } from "@/lib/tauri-client";

export function setHiddenInformationRevealed(revealed: boolean) {
  return invokeCommand<boolean>("set_hidden_information_revealed", {
    revealed,
  });
}
