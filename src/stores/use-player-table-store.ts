import { create } from "zustand";
import { persist } from "zustand/middleware";
import {
  DEFAULT_PLAYER_TABLE_COLUMN_IDS,
  getPlayerMetric,
  PLAYER_TABLE_MAX_COLUMN_WIDTH,
  PLAYER_TABLE_MIN_COLUMN_WIDTH,
} from "@/utils/player-metrics";

export const PLAYER_TABLE_LAYOUT_STORAGE_KEY =
  "fm-valuescout-player-table-layouts";

export type PlayerTableId = "search" | "squad" | "staff-search" | "my-staff";

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

function defaultLayout(table: PlayerTableId): PlayerTableLayout {
  return {
    columnIds:
      table === "search" || table === "squad"
        ? [...DEFAULT_PLAYER_TABLE_COLUMN_IDS]
        : [],
    widths: {},
  };
}

export function defaultPlayerTableLayouts(): PlayerTableLayouts {
  return {
    search: defaultLayout("search"),
    squad: defaultLayout("squad"),
    "staff-search": defaultLayout("staff-search"),
    "my-staff": defaultLayout("my-staff"),
  };
}

function sanitizeLayout(
  value: unknown,
  table: PlayerTableId,
): PlayerTableLayout {
  const record = isRecord(value) ? value : {};
  const columnIds = Array.isArray(record.columnIds)
    ? record.columnIds.filter((metricId, index, all): metricId is string => {
        if (typeof metricId !== "string" || all.indexOf(metricId) !== index) {
          return false;
        }
        return table === "search" || table === "squad"
          ? getPlayerMetric(metricId)?.sortable === true
          : metricId.length > 0;
      })
    : [];
  const visibleColumnIds =
    columnIds.length > 0
      ? columnIds
      : table === "search" || table === "squad"
        ? [...DEFAULT_PLAYER_TABLE_COLUMN_IDS]
        : [];
  const rawWidths = isRecord(record.widths) ? record.widths : {};
  const widths = Object.fromEntries(
    visibleColumnIds.flatMap((metricId) => {
      const width = rawWidths[metricId];
      return typeof width === "number" && Number.isFinite(width)
        ? [[metricId, clampWidth(width)]]
        : [];
    }),
  );

  return { columnIds: visibleColumnIds, widths };
}

function sanitizePersistedState(value: unknown): PersistedPlayerTableState {
  const record = isRecord(value) ? value : {};
  const layouts = isRecord(record.layouts) ? record.layouts : {};
  return {
    layouts: {
      search: sanitizeLayout(layouts.search, "search"),
      squad: sanitizeLayout(layouts.squad, "squad"),
      "staff-search": sanitizeLayout(layouts["staff-search"], "staff-search"),
      "my-staff": sanitizeLayout(layouts["my-staff"], "my-staff"),
    },
  };
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
              (table === "search" || table === "squad"
                ? getPlayerMetric(metricId)?.sortable === true
                : metricId.length > 0) &&
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
    }),
    {
      name: PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      version: 2,
      partialize: (state) => ({ layouts: state.layouts }),
      migrate: (persistedState) => sanitizePersistedState(persistedState),
      merge: (persistedState, currentState) => ({
        ...currentState,
        ...sanitizePersistedState(persistedState),
      }),
    },
  ),
);
