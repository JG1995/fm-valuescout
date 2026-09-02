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

const PLAYER_TABLE_LAYOUT_VERSION = 6;

export type PlayerTableId =
  | "search"
  | "moneyball-search"
  | "shortlist"
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

function withoutDuplicateIdentityColumns(columnIds: readonly string[]) {
  return columnIds.filter(
    (columnId) => columnId !== "club" && columnId !== "division",
  );
}

function defaultLayout(table: PlayerTableId): PlayerTableLayout {
  return {
    columnIds:
      table === "moneyball-search"
        ? withoutDuplicateIdentityColumns(DEFAULT_MONEYBALL_TABLE_COLUMN_IDS)
        : table === "search" || table === "squad" || table === "shortlist"
          ? withoutDuplicateIdentityColumns(DEFAULT_PLAYER_TABLE_COLUMN_IDS)
          : table === "staff-shortlist"
            ? [...DEFAULT_STAFF_SHORTLIST_COLUMN_IDS]
            : [...DEFAULT_STAFF_TABLE_COLUMN_IDS],
    widths: {},
  };
}

export function defaultPlayerTableLayouts(): PlayerTableLayouts {
  return {
    search: defaultLayout("search"),
    "moneyball-search": defaultLayout("moneyball-search"),
    shortlist: defaultLayout("shortlist"),
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
        return table === "moneyball-search"
          ? getMoneyballSearchMetric(metricId)?.sortable === true ||
              [
                "name",
                "age",
                "nationality",
                "club",
                "division",
                "value",
              ].includes(metricId)
          : table === "search" || table === "squad" || table === "shortlist"
            ? getPlayerMetric(metricId)?.sortable === true
            : metricId.length > 0;
      })
    : [];
  const useNameFallback =
    identityOnlyFallback &&
    (table === "search" ||
      table === "moneyball-search" ||
      table === "squad" ||
      table === "shortlist") &&
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
      shortlist: sanitizeLayout(
        layouts.shortlist,
        "shortlist",
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
  if (version < 5) {
    return {
      layouts: {
        ...state.layouts,
        search: removeDuplicateIdentityColumns(state.layouts.search),
        "moneyball-search": removeDuplicateIdentityColumns(
          state.layouts["moneyball-search"],
        ),
        squad: removeDuplicateIdentityColumns(state.layouts.squad),
      },
    };
  }
  return state;
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
              (table === "moneyball-search"
                ? getMoneyballSearchMetric(metricId)?.sortable === true ||
                  [
                    "name",
                    "age",
                    "nationality",
                    "club",
                    "division",
                    "value",
                  ].includes(metricId)
                : table === "search" ||
                    table === "squad" ||
                    table === "shortlist"
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
