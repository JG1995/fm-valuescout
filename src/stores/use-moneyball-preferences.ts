import { create } from "zustand";
import { persist } from "zustand/middleware";

export type DefaultAnalysisView = "general" | "moneyball";

export const DEFAULT_ANALYSIS_VIEW: DefaultAnalysisView = "general";

type MoneyballPreferencesState = {
  defaultAnalysisView: DefaultAnalysisView;
  setDefaultAnalysisView: (view: DefaultAnalysisView) => void;
};

function persistedView(value: unknown): DefaultAnalysisView {
  return value === "moneyball" ? "moneyball" : DEFAULT_ANALYSIS_VIEW;
}

export const useMoneyballPreferences = create<MoneyballPreferencesState>()(
  persist(
    (set) => ({
      defaultAnalysisView: DEFAULT_ANALYSIS_VIEW,
      setDefaultAnalysisView: (view) =>
        set({ defaultAnalysisView: persistedView(view) }),
    }),
    {
      name: "fm-valuescout-moneyball-preferences",
      version: 1,
      merge: (persisted, current) => {
        const state = persisted as
          | { defaultAnalysisView?: unknown }
          | undefined;
        return {
          ...current,
          defaultAnalysisView: persistedView(state?.defaultAnalysisView),
        };
      },
    },
  ),
);
