import { create } from "zustand";
import type { ShortlistSaveContext } from "./use-shortlist-filter-store";

export function playerShortlistSelectionKey(
  context: ShortlistSaveContext,
): string {
  return `${context.saveId}:${context.contextToken}`;
}

type PlayerShortlistSelectionState = {
  /**
   * The shortlist selected in Player Search General and Moneyball, keyed by
   * save context. An absent key means no shortlist. Nothing here persists
   * across app restarts.
   */
  selections: Record<string, number>;
  select: (context: ShortlistSaveContext, shortlistId: number) => void;
  /** Clears the context's selection only while it is still `shortlistId`. */
  clear: (context: ShortlistSaveContext, shortlistId?: number) => void;
  reset: () => void;
};

export const usePlayerShortlistSelectionStore =
  create<PlayerShortlistSelectionState>()((set) => ({
    selections: {},
    select: (context, shortlistId) =>
      set((state) => ({
        selections: {
          ...state.selections,
          [playerShortlistSelectionKey(context)]: shortlistId,
        },
      })),
    clear: (context, shortlistId) =>
      set((state) => {
        const key = playerShortlistSelectionKey(context);
        if (
          !(key in state.selections) ||
          (shortlistId !== undefined && state.selections[key] !== shortlistId)
        ) {
          return state;
        }
        const { [key]: _cleared, ...selections } = state.selections;
        return { selections };
      }),
    reset: () => set({ selections: {} }),
  }));
