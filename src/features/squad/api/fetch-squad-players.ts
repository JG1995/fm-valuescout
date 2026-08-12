import { invokeCommand } from "@/lib/tauri-client";
import type { SquadPlayersPage } from "../types/squad-player";
import type { SquadSortDir, SquadSortField } from "../types/squad-sort";
import {
  DEFAULT_SQUAD_SORT_DIR,
  DEFAULT_SQUAD_SORT_FIELD,
} from "../types/squad-sort";

/** Matches Rust `DEFAULT_SQUAD_PAGE_LIMIT` and stays below table virtualization. */
export const SQUAD_PAGE_SIZE = 50;

export function fetchSquadPlayers(
  offset = 0,
  limit = SQUAD_PAGE_SIZE,
  sortBy: SquadSortField = DEFAULT_SQUAD_SORT_FIELD,
  sortDir: SquadSortDir = DEFAULT_SQUAD_SORT_DIR,
) {
  return invokeCommand<SquadPlayersPage>("list_squad_players", {
    offset,
    limit,
    sortBy,
    sortDir,
  });
}
