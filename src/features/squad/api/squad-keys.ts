import type { SquadSortDir, SquadSortField } from "../types/squad-sort";
import {
  DEFAULT_SQUAD_SORT_DIR,
  DEFAULT_SQUAD_SORT_FIELD,
} from "../types/squad-sort";

export type SquadPlayerPageContext = {
  activeSave: { id: number; contextToken: string } | null;
  currentSnapshot: { id: number; saveId: number } | null;
  managedClub: { clubName: string | null; status: string } | null;
};

export const squadKeys = {
  // Planner owns Squad membership queries, so its invalidation boundary also
  // refreshes this presentation read after managed-club changes.
  all: ["planner", "squad"] as const,
  playerPages: () => [...squadKeys.all, "players"] as const,
  players: (
    offset: number,
    limit: number,
    sortBy: SquadSortField = DEFAULT_SQUAD_SORT_FIELD,
    sortDir: SquadSortDir = DEFAULT_SQUAD_SORT_DIR,
    requestedFields: string[] = [],
    context: SquadPlayerPageContext = {
      activeSave: null,
      currentSnapshot: null,
      managedClub: null,
    },
  ) =>
    [
      ...squadKeys.playerPages(),
      { offset, limit, sortBy, sortDir, requestedFields, context },
    ] as const,
};
