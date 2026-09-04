import { invokeCommand } from "@/lib/tauri-client";
import type { PlayerShortlistImportSummary } from "../types/player-shortlist-import-summary";

export function importPlayerShortlistCsv(
  path: string,
): Promise<PlayerShortlistImportSummary> {
  return invokeCommand<PlayerShortlistImportSummary>(
    "import_player_shortlist_csv",
    { path },
  );
}
