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

const PLAYER_TABLE_LAYOUT_VERSION = 8;

export const IDENTITY_COLUMN_MIN_WIDTH = 240;
export const IDENTITY_COLUMN_DEFAULT_WIDTH = 280;
export const IDENTITY_COLUMN_MAX_WIDTH = 360;

const IDENTITY_COLUMN_IDS = new Set(["name", "club", "division"]);

export function isIdentityColumnId(metricId: string): boolean {
  return IDENTITY_COLUMN_IDS.has(metricId);
}

export function withoutIdentityColumnIds(
  columnIds: readonly string[],
): string[] {
  return columnIds.filter((columnId) => !isIdentityColumnId(columnId));
}

function clampIdentityWidth(width: number | undefined): number {
  if (!Number.isFinite(width)) {
    return IDENTITY_COLUMN_DEFAULT_WIDTH;
  }
  return Math.min(
    IDENTITY_COLUMN_MAX_WIDTH,
    Math.max(IDENTITY_COLUMN_MIN_WIDTH, width as number),
  );
}

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
  identityWidth: number;
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
  setIdentityWidth: (table: PlayerTableId, width: number) => void;
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

/** Pre-v8 allowlist: permits identity IDs so step (2) reproduces v7 exactly. */
function isAllowedV7ColumnId(table: PlayerTableId, id: string): boolean {
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

function isAllowedColumnId(table: PlayerTableId, id: string): boolean {
  if (typeof id !== "string" || id.length === 0 || isIdentityColumnId(id)) {
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
    return [
      "age",
      "nationality",
      "height",
      "moneyball.minutes",
      "moneyball.average_rating",
      "moneyball.goals_per_90",
      "moneyball.assists_per_90",
      "moneyball.xg_per_90",
      "moneyball.xa_per_90",
    ];
  }
  if (table === "squad") {
    return [
      "age",
      "nationality",
      "height",
      "ca",
      "pa",
      "value",
      SUGGESTED_TRAINING_COLUMN_ID,
    ];
  }
  if (table === "search") {
    return ["age", "nationality", "height", "ca", "pa", "value"];
  }
  if (table === "staff-shortlist") {
    return withoutIdentityColumnIds(DEFAULT_STAFF_SHORTLIST_COLUMN_IDS);
  }
  return withoutIdentityColumnIds(DEFAULT_STAFF_TABLE_COLUMN_IDS);
}

/**
 * Immutable historic v8 migration outputs (without Height). Pre-v8
 * default-like and malformed fallbacks must keep returning these exact
 * arrays; fresh defaults and explicit resets use `defaultColumnIds`.
 */
function historicalV8DefaultColumnIds(table: PlayerTableId): string[] {
  if (table === "moneyball-search") {
    return [
      "age",
      "nationality",
      "moneyball.minutes",
      "moneyball.average_rating",
      "moneyball.goals_per_90",
      "moneyball.assists_per_90",
      "moneyball.xg_per_90",
      "moneyball.xa_per_90",
    ];
  }
  if (table === "squad") {
    return [
      "age",
      "nationality",
      "ca",
      "pa",
      "value",
      SUGGESTED_TRAINING_COLUMN_ID,
    ];
  }
  if (table === "search") {
    return ["age", "nationality", "ca", "pa", "value"];
  }
  return defaultColumnIds(table);
}

/** Resulting v7 defaults per table, used only for default-like detection. */
function v7DefaultColumnIds(table: PlayerTableId): string[] {
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
  return {
    columnIds: defaultColumnIds(table),
    widths: {},
    identityWidth: IDENTITY_COLUMN_DEFAULT_WIDTH,
  };
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

function sanitizeIdentityWidth(record: Record<string, unknown>): number {
  const width = record.identityWidth;
  return clampIdentityWidth(typeof width === "number" ? width : undefined);
}

function sanitizeLayout(
  value: unknown,
  table: PlayerTableId,
): PlayerTableLayout {
  const record = isRecord(value) ? value : {};
  const rawIds = record.columnIds;
  const identityWidth = sanitizeIdentityWidth(record);
  if (!Array.isArray(rawIds) || rawIds.length === 0) {
    if (
      Array.isArray(rawIds) &&
      rawIds.length === 0 &&
      typeof record.identityWidth === "number" &&
      Number.isFinite(record.identityWidth)
    ) {
      return { columnIds: [], widths: {}, identityWidth };
    }
    return { ...defaultLayout(table), identityWidth };
  }
  const columnIds = rawIds.filter(
    (metricId, index, all): metricId is string => {
      if (typeof metricId !== "string" || all.indexOf(metricId) !== index) {
        return false;
      }
      return isAllowedColumnId(table, metricId);
    },
  );
  const rawWidths = isRecord(record.widths) ? record.widths : {};
  const widths = Object.fromEntries(
    columnIds.flatMap((metricId) => {
      const width = rawWidths[metricId];
      return typeof width === "number" && Number.isFinite(width)
        ? [[metricId, clampWidth(width)]]
        : [];
    }),
  );

  return { columnIds, widths, identityWidth };
}

function sanitizePersistedState(value: unknown): PersistedPlayerTableState {
  const record = isRecord(value) ? value : {};
  const layouts = isRecord(record.layouts) ? record.layouts : {};
  return {
    layouts: {
      search: sanitizeLayout(layouts.search, "search"),
      "moneyball-search": sanitizeLayout(
        layouts["moneyball-search"],
        "moneyball-search",
      ),
      squad: sanitizeLayout(layouts.squad, "squad"),
      "staff-search": sanitizeLayout(layouts["staff-search"], "staff-search"),
      "my-staff": sanitizeLayout(layouts["my-staff"], "my-staff"),
      "staff-shortlist": sanitizeLayout(
        layouts["staff-shortlist"],
        "staff-shortlist",
      ),
    },
  };
}

type V7TableLayout = {
  columnIds: string[];
  widths: Record<string, number>;
};

function sanitizeV7Layout(
  value: unknown,
  table: PlayerTableId,
  identityOnlyFallback: boolean,
): V7TableLayout {
  const record = isRecord(value) ? value : {};
  const columnIds = Array.isArray(record.columnIds)
    ? record.columnIds.filter((metricId, index, all): metricId is string => {
        if (typeof metricId !== "string" || all.indexOf(metricId) !== index) {
          return false;
        }
        return isAllowedV7ColumnId(table, metricId);
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
      : [...v7DefaultColumnIds(table)];
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

function removeDuplicateIdentityColumns(layout: V7TableLayout): V7TableLayout {
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

function migrateTableToV8(
  value: unknown,
  table: PlayerTableId,
  version: number,
): PlayerTableLayout {
  const record = isRecord(value) ? value : {};
  // (1) Capture the original finite `widths.name` first as the
  // `identityWidth` candidate. It survives even when later normalization
  // strips the name width.
  const rawWidths = isRecord(record.widths) ? record.widths : {};
  const nameWidth = rawWidths.name;
  const identityWidth = clampIdentityWidth(
    typeof nameWidth === "number" ? nameWidth : undefined,
  );
  // (7) Missing/non-array/empty raw `columnIds` is malformed and falls back
  // to the exact historic v8 defaults with `identityWidth` 280.
  const rawIds = record.columnIds;
  if (!Array.isArray(rawIds) || rawIds.length === 0) {
    return {
      columnIds: historicalV8DefaultColumnIds(table),
      widths: {},
      identityWidth: IDENTITY_COLUMN_DEFAULT_WIDTH,
    };
  }
  // (2) Run the pre-v8 upgrades unchanged to produce a v7-equivalent layout.
  let v7 = sanitizeV7Layout(value, table, version < 5);
  if (version < 5) {
    if (
      table === "search" ||
      table === "moneyball-search" ||
      table === "squad"
    ) {
      v7 = removeDuplicateIdentityColumns(v7);
    }
  }
  if (version < 7) {
    // Rollout: a persisted Squad layout still exactly equal to the v6
    // default (default column IDs with default empty widths) gains Suggested
    // Training far right; customized layouts keep their order and content.
    const v6DefaultSquadColumnIds = withoutDuplicateIdentityColumns(
      DEFAULT_PLAYER_TABLE_COLUMN_IDS,
    );
    if (table === "squad") {
      const isV6DefaultLike =
        v7.columnIds.length === v6DefaultSquadColumnIds.length &&
        v7.columnIds.every(
          (id, index) => id === v6DefaultSquadColumnIds[index],
        ) &&
        Object.keys(v7.widths).length === 0;
      if (isV6DefaultLike) {
        v7 = {
          ...v7,
          columnIds: [...v7.columnIds, SUGGESTED_TRAINING_COLUMN_ID],
        };
      }
    }
  }
  // (5) Default-like means the v7-equivalent `columnIds` exactly equal that
  // table's resulting v7 defaults in order with exactly empty widths. Any
  // reorder, resize, add, or remove is custom.
  const v7Defaults = v7DefaultColumnIds(table);
  const isDefaultLike =
    v7.columnIds.length === v7Defaults.length &&
    v7.columnIds.every((id, index) => id === v7Defaults[index]) &&
    Object.keys(v7.widths).length === 0;
  if (isDefaultLike) {
    return {
      columnIds: historicalV8DefaultColumnIds(table),
      widths: {},
      identityWidth,
    };
  }
  // (3, 4, 6) Strip identity IDs/widths, sanitize the remainder through the
  // allowlists with dedupe and width clamping, and keep the custom remainder
  // verbatim — including an empty remainder, which is valid identity-only.
  const stripped = withoutIdentityColumnIds(v7.columnIds);
  const columnIds = stripped.filter(
    (metricId, index, all) =>
      all.indexOf(metricId) === index && isAllowedColumnId(table, metricId),
  );
  const widths = Object.fromEntries(
    columnIds.flatMap((metricId) => {
      const width = v7.widths[metricId];
      return typeof width === "number" && Number.isFinite(width)
        ? [[metricId, clampWidth(width)]]
        : [];
    }),
  );
  return { columnIds, widths, identityWidth };
}

function migratePersistedState(
  persistedState: unknown,
  version: number,
): PersistedPlayerTableState {
  if (version >= PLAYER_TABLE_LAYOUT_VERSION) {
    return sanitizePersistedState(persistedState);
  }
  const record = isRecord(persistedState) ? persistedState : {};
  const layouts = isRecord(record.layouts) ? record.layouts : {};
  return {
    layouts: {
      search: migrateTableToV8(layouts.search, "search", version),
      "moneyball-search": migrateTableToV8(
        layouts["moneyball-search"],
        "moneyball-search",
        version,
      ),
      squad: migrateTableToV8(layouts.squad, "squad", version),
      "staff-search": migrateTableToV8(
        layouts["staff-search"],
        "staff-search",
        version,
      ),
      "my-staff": migrateTableToV8(layouts["my-staff"], "my-staff", version),
      "staff-shortlist": migrateTableToV8(
        layouts["staff-shortlist"],
        "staff-shortlist",
        version,
      ),
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
          if (!layout.columnIds.includes(metricId)) {
            return state;
          }
          const { [metricId]: _removedWidth, ...widths } = layout.widths;
          return {
            layouts: {
              ...state.layouts,
              [table]: {
                columnIds: layout.columnIds.filter((id) => id !== metricId),
                widths,
                identityWidth: layout.identityWidth,
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
            isIdentityColumnId(metricId) ||
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
          if (
            isIdentityColumnId(metricId) ||
            !layout.columnIds.includes(metricId) ||
            !Number.isFinite(width)
          ) {
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
      setIdentityWidth: (table, width) => {
        set((state) => ({
          layouts: {
            ...state.layouts,
            [table]: {
              ...state.layouts[table],
              identityWidth: clampIdentityWidth(width),
            },
          },
        }));
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
                [table]: {
                  columnIds: finalIds,
                  widths: {},
                  identityWidth: layout.identityWidth,
                },
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
              [table]: {
                columnIds: finalIds,
                widths,
                identityWidth: layout.identityWidth,
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
