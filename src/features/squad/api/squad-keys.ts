import type { SquadSortDir, SquadSortField } from "../types/squad-sort";
import {
  DEFAULT_SQUAD_SORT_DIR,
  DEFAULT_SQUAD_SORT_FIELD,
} from "../types/squad-sort";

export const squadKeys = {
  // Planner owns Squad membership queries, so its invalidation boundary also
  // refreshes this presentation read after managed-club changes.
  all: ["planner", "squad"] as const,
  players: (
    offset: number,
    limit: number,
    sortBy: SquadSortField = DEFAULT_SQUAD_SORT_FIELD,
    sortDir: SquadSortDir = DEFAULT_SQUAD_SORT_DIR,
    requestedFields: string[] = [],
  ) =>
    [
      ...squadKeys.all,
      "players",
      { offset, limit, sortBy, sortDir, requestedFields },
    ] as const,
};
