import { create } from "zustand";

/** The URL search state Player Search last showed, restored by the top navigation. */
export type SearchViewMemory = {
  sort: string;
  dir: string;
  filters: unknown[];
  combine: string;
  comparison?: string;
};

type SearchViewMemoryState = {
  memory: SearchViewMemory | null;
  remember: (memory: SearchViewMemory) => void;
};

/** Session memory of Player Search's filters, sort, and comparison. */
export const useSearchViewMemory = create<SearchViewMemoryState>((set) => ({
  memory: null,
  remember: (memory) => set({ memory }),
}));
