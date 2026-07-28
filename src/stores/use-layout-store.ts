import { create } from "zustand";

type LayoutState = {
  sidebarOpen: boolean;
  toggleSidebar: () => void;
};

export const useLayoutStore = create<LayoutState>((set) => ({
  sidebarOpen: false,
  toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
}));
