import type { SquadSortDir, SquadSortField } from "../types/squad-sort";
import {
  DEFAULT_SQUAD_SORT_DIR,
  DEFAULT_SQUAD_SORT_FIELD,
} from "../types/squad-sort";

export const squadKeys = {
  // Planner owns the club-family query contract, so its existing invalidation
  // boundary also refreshes this Squad presentation read.
  all: ["planner", "squad"] as const,
  players: (
    offset: number,
    limit: number,
    sortBy: SquadSortField = DEFAULT_SQUAD_SORT_FIELD,
    sortDir: SquadSortDir = DEFAULT_SQUAD_SORT_DIR,
  ) =>
    [...squadKeys.all, "players", { offset, limit, sortBy, sortDir }] as const,
};
