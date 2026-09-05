import { create } from "zustand";
import { persist } from "zustand/middleware";
import {
  DEFAULT_MONEYBALL_TABLE_COLUMN_IDS,
  getMoneyballSearchMetric,
} from "@/utils/moneyball-search-metrics";
import {
  DEFAULT_PLAYER_TABLE_COLUMN_IDS,
  getPlayerMetric,
  PLAYER_TABLE_MAX_COLUMN_WIDTH,
  PLAYER_TABLE_MIN_COLUMN_WIDTH,
} from "@/utils/player-metrics";
import { DEFAULT_STAFF_TABLE_COLUMN_IDS } from "@/utils/staff-table-layout";
import {
  isSuggestedTrainingColumnId,
  SUGGESTED_TRAINING_COLUMN_ID,
} from "@/utils/suggested-training";
import { isTacticColumnId, isValidTacticColumnId } from "@/utils/tactic-ids";

const DEFAULT_STAFF_SHORTLIST_COLUMN_IDS = [
  "name",
  "age",
  "nationality",
  "club",
  "ca",
  "pa",
  "preferred_job",
  "club_job",
  "coaching_qualifications",
  ...DEFAULT_STAFF_TABLE_COLUMN_IDS.filter(
    (columnId) =>
      !["name", "age", "nationality", "ca", "pa"].includes(columnId),
  ),
];

export const PLAYER_TABLE_LAYOUT_STORAGE_KEY =
  "fm-valuescout-player-table-layouts";

const PLAYER_TABLE_LAYOUT_VERSION = 7;

/**
 * Squad default: the v6 Squad default with Suggested Training appended far
 * right. Search keeps sharing `DEFAULT_PLAYER_TABLE_COLUMN_IDS` unchanged.
 */
export const DEFAULT_SQUAD_TABLE_COLUMN_IDS = [
  ...withoutDuplicateIdentityColumns(DEFAULT_PLAYER_TABLE_COLUMN_IDS),
  SUGGESTED_TRAINING_COLUMN_ID,
] as const;

export type PlayerTableId =
  | "search"
  | "moneyball-search"
  | "squad"
  | "staff-search"
  | "my-staff"
  | "staff-shortlist";

export type PlayerTableLayout = {
  columnIds: string[];
  widths: Record<string, number>;
};

type PlayerTableLayouts = Record<PlayerTableId, PlayerTableLayout>;

type PlayerTableStore = {
  layouts: PlayerTableLayouts;
  addColumns: (table: PlayerTableId, metricIds: readonly string[]) => void;
  removeColumn: (table: PlayerTableId, metricId: string) => void;
  moveColumn: (
    table: PlayerTableId,
    metricId: string,
    targetIndex: number,
  ) => void;
  setColumnWidth: (
    table: PlayerTableId,
    metricId: string,
    width: number,
  ) => void;
  replaceLayout: (
    table: PlayerTableId,
    nextColumnIds: readonly string[],
  ) => void;
};

type PersistedPlayerTableState = Pick<PlayerTableStore, "layouts">;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function clampWidth(width: number): number {
  return Math.min(
    PLAYER_TABLE_MAX_COLUMN_WIDTH,
    Math.max(PLAYER_TABLE_MIN_COLUMN_WIDTH, Math.round(width)),
  );
}

function isAllowedColumnId(table: PlayerTableId, id: string): boolean {
  if (typeof id !== "string" || id.length === 0) {
    return false;
  }
  if (isSuggestedTrainingColumnId(id)) {
    return table === "squad";
  }
  if (isValidTacticColumnId(id)) {
    return table === "search" || table === "moneyball-search";
  }
  if (isTacticColumnId(id)) {
    return false;
  }
  if (table === "moneyball-search") {
    return (
      getMoneyballSearchMetric(id)?.sortable === true ||
      ["name", "age", "nationality", "club", "division", "value"].includes(id)
    );
  }
  if (table === "search" || table === "squad") {
    return getPlayerMetric(id)?.sortable === true;
  }
  return id.length > 0;
}

function withoutDuplicateIdentityColumns(columnIds: readonly string[]) {
  return columnIds.filter(
    (columnId) => columnId !== "club" && columnId !== "division",
  );
}

function defaultColumnIds(table: PlayerTableId): string[] {
  if (table === "moneyball-search") {
    return withoutDuplicateIdentityColumns(DEFAULT_MONEYBALL_TABLE_COLUMN_IDS);
  }
  if (table === "squad") {
    return [...DEFAULT_SQUAD_TABLE_COLUMN_IDS];
  }
  if (table === "search") {
    return withoutDuplicateIdentityColumns(DEFAULT_PLAYER_TABLE_COLUMN_IDS);
  }
  if (table === "staff-shortlist") {
    return [...DEFAULT_STAFF_SHORTLIST_COLUMN_IDS];
  }
  return [...DEFAULT_STAFF_TABLE_COLUMN_IDS];
}

function defaultLayout(table: PlayerTableId): PlayerTableLayout {
  return { columnIds: defaultColumnIds(table), widths: {} };
}

export function defaultPlayerTableLayouts(): PlayerTableLayouts {
  return {
    search: defaultLayout("search"),
    "moneyball-search": defaultLayout("moneyball-search"),
    squad: defaultLayout("squad"),
    "staff-search": defaultLayout("staff-search"),
    "my-staff": defaultLayout("my-staff"),
    "staff-shortlist": defaultLayout("staff-shortlist"),
  };
}

function sanitizeLayout(
  value: unknown,
  table: PlayerTableId,
  identityOnlyFallback = false,
): PlayerTableLayout {
  const record = isRecord(value) ? value : {};
  const columnIds = Array.isArray(record.columnIds)
    ? record.columnIds.filter((metricId, index, all): metricId is string => {
        if (typeof metricId !== "string" || all.indexOf(metricId) !== index) {
          return false;
        }
        return isAllowedColumnId(table, metricId);
      })
    : [];
  const useNameFallback =
    identityOnlyFallback &&
    (table === "search" || table === "moneyball-search" || table === "squad") &&
    columnIds.length > 0 &&
    withoutDuplicateIdentityColumns(columnIds).length === 0;
  const visibleColumnIds = useNameFallback
    ? ["name"]
    : columnIds.length > 0
      ? columnIds
      : [...defaultLayout(table).columnIds];
  const rawWidths = isRecord(record.widths) ? record.widths : {};
  const widths = useNameFallback
    ? {}
    : Object.fromEntries(
        visibleColumnIds.flatMap((metricId) => {
          const width = rawWidths[metricId];
          return typeof width === "number" && Number.isFinite(width)
            ? [[metricId, clampWidth(width)]]
            : [];
        }),
      );

  return { columnIds: visibleColumnIds, widths };
}

function sanitizePersistedState(
  value: unknown,
  identityOnlyFallback = false,
): PersistedPlayerTableState {
  const record = isRecord(value) ? value : {};
  const layouts = isRecord(record.layouts) ? record.layouts : {};
  return {
    layouts: {
      search: sanitizeLayout(layouts.search, "search", identityOnlyFallback),
      "moneyball-search": sanitizeLayout(
        layouts["moneyball-search"],
        "moneyball-search",
        identityOnlyFallback,
      ),
      squad: sanitizeLayout(layouts.squad, "squad", identityOnlyFallback),
      "staff-search": sanitizeLayout(layouts["staff-search"], "staff-search"),
      "my-staff": sanitizeLayout(layouts["my-staff"], "my-staff"),
      "staff-shortlist": sanitizeLayout(
        layouts["staff-shortlist"],
        "staff-shortlist",
      ),
    },
  };
}

function removeDuplicateIdentityColumns(
  layout: PlayerTableLayout,
): PlayerTableLayout {
  const columnIds = withoutDuplicateIdentityColumns(layout.columnIds);
  return {
    columnIds,
    widths: Object.fromEntries(
      Object.entries(layout.widths).filter(([metricId]) =>
        columnIds.includes(metricId),
      ),
    ),
  };
}

function migratePersistedState(
  persistedState: unknown,
  version: number,
): PersistedPlayerTableState {
  const state = sanitizePersistedState(persistedState, version < 5);
  if (version >= PLAYER_TABLE_LAYOUT_VERSION) {
    return state;
  }
  let layouts = state.layouts;
  if (version < 5) {
    layouts = {
      ...layouts,
      search: removeDuplicateIdentityColumns(layouts.search),
      "moneyball-search": removeDuplicateIdentityColumns(
        layouts["moneyball-search"],
      ),
      squad: removeDuplicateIdentityColumns(layouts.squad),
    };
  }
  if (version < 7) {
    // Rollout: a persisted Squad layout still exactly equal to the v6
    // default (default column IDs with default empty widths) gains Suggested
    // Training far right; customized layouts keep their order and content.
    // Other tables sanitize the ID away through `isAllowedColumnId`.
    const v6DefaultSquadColumnIds = withoutDuplicateIdentityColumns(
      DEFAULT_PLAYER_TABLE_COLUMN_IDS,
    );
    const squad = layouts.squad;
    const isV6DefaultLike =
      squad.columnIds.length === v6DefaultSquadColumnIds.length &&
      squad.columnIds.every(
        (id, index) => id === v6DefaultSquadColumnIds[index],
      ) &&
      Object.keys(squad.widths).length === 0;
    if (isV6DefaultLike) {
      layouts = {
        ...layouts,
        squad: {
          ...squad,
          columnIds: [...squad.columnIds, SUGGESTED_TRAINING_COLUMN_ID],
        },
      };
    }
  }
  return { layouts };
}

export const usePlayerTableStore = create<PlayerTableStore>()(
  persist(
    (set) => ({
      layouts: defaultPlayerTableLayouts(),
      addColumns: (table, metricIds) => {
        set((state) => {
          const layout = state.layouts[table];
          const additions = metricIds.filter(
            (metricId, index) =>
              isAllowedColumnId(table, metricId) &&
              !layout.columnIds.includes(metricId) &&
              metricIds.indexOf(metricId) === index,
          );
          if (additions.length === 0) {
            return state;
          }
          return {
            layouts: {
              ...state.layouts,
              [table]: {
                ...layout,
                columnIds: [...layout.columnIds, ...additions],
              },
            },
          };
        });
      },
      removeColumn: (table, metricId) => {
        set((state) => {
          const layout = state.layouts[table];
          if (
            layout.columnIds.length === 1 ||
            !layout.columnIds.includes(metricId)
          ) {
            return state;
          }
          const { [metricId]: _removedWidth, ...widths } = layout.widths;
          return {
            layouts: {
              ...state.layouts,
              [table]: {
                columnIds: layout.columnIds.filter((id) => id !== metricId),
                widths,
              },
            },
          };
        });
      },
      moveColumn: (table, metricId, targetIndex) => {
        set((state) => {
          const layout = state.layouts[table];
          const currentIndex = layout.columnIds.indexOf(metricId);
          if (
            currentIndex < 0 ||
            !Number.isInteger(targetIndex) ||
            targetIndex < 0 ||
            targetIndex >= layout.columnIds.length ||
            currentIndex === targetIndex
          ) {
            return state;
          }
          const columnIds = [...layout.columnIds];
          columnIds.splice(currentIndex, 1);
          columnIds.splice(targetIndex, 0, metricId);
          return {
            layouts: {
              ...state.layouts,
              [table]: { ...layout, columnIds },
            },
          };
        });
      },
      setColumnWidth: (table, metricId, width) => {
        set((state) => {
          const layout = state.layouts[table];
          if (!layout.columnIds.includes(metricId) || !Number.isFinite(width)) {
            return state;
          }
          return {
            layouts: {
              ...state.layouts,
              [table]: {
                ...layout,
                widths: { ...layout.widths, [metricId]: clampWidth(width) },
              },
            },
          };
        });
      },
      replaceLayout: (table, nextColumnIds) => {
        set((state) => {
          const layout = state.layouts[table];
          const deduped: string[] = [];
          for (let idx = 0; idx < nextColumnIds.length; idx += 1) {
            const id = nextColumnIds[idx];
            if (typeof id !== "string" || deduped.includes(id)) {
              continue;
            }
            if (nextColumnIds.indexOf(id) !== idx) {
              continue;
            }
            if (!isAllowedColumnId(table, id)) {
              continue;
            }
            deduped.push(id);
          }
          let finalIds = deduped;
          if (finalIds.length === 0) {
            finalIds = [...defaultLayout(table).columnIds];
            return {
              layouts: {
                ...state.layouts,
                [table]: { columnIds: finalIds, widths: {} },
              },
            };
          }
          const widths: Record<string, number> = {};
          for (const id of finalIds) {
            const width = layout.widths[id];
            if (typeof width === "number" && Number.isFinite(width)) {
              widths[id] = clampWidth(width);
            }
          }
          return {
            layouts: {
              ...state.layouts,
              [table]: { columnIds: finalIds, widths },
            },
          };
        });
      },
    }),
    {
      name: PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      version: PLAYER_TABLE_LAYOUT_VERSION,
      partialize: (state) => ({ layouts: state.layouts }),
      migrate: migratePersistedState,
      merge: (persistedState, currentState) => ({
        ...currentState,
        ...sanitizePersistedState(persistedState),
      }),
    },
  ),
);
