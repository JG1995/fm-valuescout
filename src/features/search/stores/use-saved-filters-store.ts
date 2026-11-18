import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { FilterCombineMode } from "../types/filter-rule";
import { createFilterRuleId } from "../types/filter-rule";
import {
  type MoneyballComparison,
  parseMoneyballComparison,
} from "../types/search-view";
import {
  type FilterRuleUrl,
  MAX_FILTER_RULES,
  parseSearchCombine,
} from "../utils/search-url-search";

/** A named set of Search filters: rules, how they combine, and Percentiles vs. */
export type SavedFilter = {
  id: string;
  name: string;
  filters: FilterRuleUrl[];
  combine: FilterCombineMode;
  comparison: MoneyballComparison;
};

type SavedFiltersState = {
  savedFilters: SavedFilter[];
  /** Saves under `name`, replacing a saved filter of the same name. */
  save: (filter: Omit<SavedFilter, "id">) => void;
  rename: (id: string, name: string) => void;
  remove: (id: string) => void;
};

/** Names compare without case or surrounding spaces. */
function sameName(left: string, right: string): boolean {
  return left.trim().toLowerCase() === right.trim().toLowerCase();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/**
 * A stored entry, or undefined when it is malformed. Rules are kept as
 * stored and parsed when applied, so a field the app no longer offers drops
 * out then.
 */
function sanitizeSavedFilter(value: unknown): SavedFilter | undefined {
  if (!isRecord(value)) {
    return undefined;
  }
  const { id, name, filters } = value;
  if (
    typeof id !== "string" ||
    typeof name !== "string" ||
    name.trim() === "" ||
    !Array.isArray(filters)
  ) {
    return undefined;
  }
  return {
    id,
    name: name.trim(),
    filters: filters.slice(0, MAX_FILTER_RULES) as FilterRuleUrl[],
    combine: parseSearchCombine(value.combine),
    comparison: parseMoneyballComparison(value.comparison),
  };
}

/**
 * Player Search's saved filters. They name fields, not players, so they are
 * the app's rather than a save's, and live in this browser profile.
 */
export const useSavedFiltersStore = create<SavedFiltersState>()(
  persist(
    (set) => ({
      savedFilters: [],
      save: (filter) => {
        const name = filter.name.trim();
        if (name === "") {
          return;
        }
        set((state) => {
          const existing = state.savedFilters.find((saved) =>
            sameName(saved.name, name),
          );
          const next: SavedFilter = {
            ...filter,
            name,
            id: existing?.id ?? createFilterRuleId(),
          };
          return {
            savedFilters: existing
              ? state.savedFilters.map((saved) =>
                  saved.id === existing.id ? next : saved,
                )
              : [...state.savedFilters, next],
          };
        });
      },
      rename: (id, name) => {
        const trimmed = name.trim();
        if (trimmed === "") {
          return;
        }
        set((state) => ({
          savedFilters: state.savedFilters
            // Renaming onto another saved filter's name replaces that one.
            .filter(
              (saved) => saved.id === id || !sameName(saved.name, trimmed),
            )
            .map((saved) =>
              saved.id === id ? { ...saved, name: trimmed } : saved,
            ),
        }));
      },
      remove: (id) =>
        set((state) => ({
          savedFilters: state.savedFilters.filter((saved) => saved.id !== id),
        })),
    }),
    {
      name: "fm-valuescout-saved-filters",
      version: 1,
      partialize: (state) => ({ savedFilters: state.savedFilters }),
      merge: (persisted, current) => {
        const stored = isRecord(persisted) ? persisted.savedFilters : undefined;
        return {
          ...current,
          savedFilters: Array.isArray(stored)
            ? stored.flatMap((entry) => sanitizeSavedFilter(entry) ?? [])
            : [],
        };
      },
    },
  ),
);
