import { create } from "zustand";
import { persist } from "zustand/middleware";

type LayoutState = {
  /** The nav rail is always visible; expanded swaps icon-only for icon-plus-label. */
  railExpanded: boolean;
  toggleRail: () => void;
};

export const useLayoutStore = create<LayoutState>()(
  persist(
    (set) => ({
      railExpanded: false,
      toggleRail: () => set((state) => ({ railExpanded: !state.railExpanded })),
    }),
    { name: "fm-valuescout-layout" },
  ),
);
