import { create } from "zustand";

type SearchView = "general" | "moneyball";

/** The URL search state a Search view last showed, restored by the top navigation. */
export type SearchViewMemory = {
  sort: string;
  dir: string;
  filters: unknown[];
  combine: string;
  comparison?: string;
};

type SearchViewMemoryState = {
  views: Partial<Record<SearchView, SearchViewMemory>>;
  remember: (view: SearchView, memory: SearchViewMemory) => void;
};

/**
 * Session memory of each Search view's filters, sort, and Moneyball comparison, so
 * switching between Search and Moneyball returns to where each was left.
 * General and Moneyball fields differ, so nothing carries across views.
 */
export const useSearchViewMemory = create<SearchViewMemoryState>((set) => ({
  views: {},
  remember: (view, memory) =>
    set((state) => ({ views: { ...state.views, [view]: memory } })),
}));
