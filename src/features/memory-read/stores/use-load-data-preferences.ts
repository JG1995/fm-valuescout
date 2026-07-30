import { create } from "zustand";
import { persist } from "zustand/middleware";

/** Diagnostic default when enabling the Load Data player cap. */
export const DEFAULT_PLAYER_CAP = 500;

type LoadDataPreferencesState = {
  /** When false, Load Data requests an unlimited scan (`maxAccepted: null`). */
  playerCapEnabled: boolean;
  /** Positive accepted-player limit used when `playerCapEnabled` is true. */
  playerCap: number;
  setPlayerCapEnabled: (enabled: boolean) => void;
  setPlayerCap: (cap: number) => void;
};

export const useLoadDataPreferences = create<LoadDataPreferencesState>()(
  persist(
    (set) => ({
      playerCapEnabled: false,
      playerCap: DEFAULT_PLAYER_CAP,
      setPlayerCapEnabled: (enabled) =>
        set((state) => ({
          playerCapEnabled: enabled,
          playerCap:
            enabled && state.playerCap < 1
              ? DEFAULT_PLAYER_CAP
              : state.playerCap,
        })),
      setPlayerCap: (cap) => set({ playerCap: cap }),
    }),
    { name: "fm-valuescout-load-data" },
  ),
);
