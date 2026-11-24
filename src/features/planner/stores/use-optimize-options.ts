import { create } from "zustand";

type OptimizeOptionsState = {
  /** Whether Optimize holds a player's position familiarity against him. */
  positionFamiliarity: boolean;
  setPositionFamiliarity: (positionFamiliarity: boolean) => void;
};

/** How Optimize squads places players. It lasts until the app restarts. */
export const useOptimizeOptions = create<OptimizeOptionsState>((set) => ({
  positionFamiliarity: true,
  setPositionFamiliarity: (positionFamiliarity) => set({ positionFamiliarity }),
}));
