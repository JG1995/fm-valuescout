import { create } from "zustand";

/**
 * Search surfaces that can restrict their results to a stored shortlist.
 * Player Search and Staff Search keep independent choices.
 */
export type ShortlistSurface = "player" | "staff";

/** The save identity that scopes a shortlist filter choice. */
export type ShortlistSaveContext = {
  saveId: number;
  contextToken: string;
};

export function shortlistFilterKey(
  surface: ShortlistSurface,
  context: ShortlistSaveContext,
): string {
  return `${surface}:${context.saveId}:${context.contextToken}`;
}

type ShortlistFilterState = {
  /**
   * Session-only choices keyed by surface + save context. An absent key is
   * uninitialized: the route derives the default from actual shortlist
   * presence exactly once, and later query refreshes must not overwrite a
   * manual toggle. Nothing here persists across app restarts.
   */
  choices: Record<string, boolean>;
  /** Sets the presence-derived default only when the choice is unset. */
  initialize: (
    surface: ShortlistSurface,
    context: ShortlistSaveContext,
    present: boolean,
  ) => void;
  /** Records a manual user toggle for the save context. */
  set: (
    surface: ShortlistSurface,
    context: ShortlistSaveContext,
    enabled: boolean,
  ) => void;
  reset: () => void;
};

export const useShortlistFilterStore = create<ShortlistFilterState>()(
  (set) => ({
    choices: {},
    initialize: (surface, context, present) =>
      set((state) => {
        const key = shortlistFilterKey(surface, context);
        return key in state.choices
          ? state
          : { choices: { ...state.choices, [key]: present } };
      }),
    set: (surface, context, enabled) =>
      set((state) => ({
        choices: {
          ...state.choices,
          [shortlistFilterKey(surface, context)]: enabled,
        },
      })),
    reset: () => set({ choices: {} }),
  }),
);
