import { beforeEach, describe, expect, it } from "vitest";
import {
  DEFAULT_ANALYSIS_VIEW,
  useMoneyballPreferences,
} from "./use-moneyball-preferences";

describe("useMoneyballPreferences", () => {
  beforeEach(() => {
    localStorage.clear();
    useMoneyballPreferences.setState({
      defaultAnalysisView: DEFAULT_ANALYSIS_VIEW,
    });
  });

  it("defaults when no preference is stored", async () => {
    await useMoneyballPreferences.persist.rehydrate();

    expect(useMoneyballPreferences.getState().defaultAnalysisView).toBe(
      "general",
    );
  });

  it.each(["general", "moneyball"] as const)(
    "hydrates a valid %s preference",
    async (defaultAnalysisView) => {
      localStorage.setItem(
        "fm-valuescout-moneyball-preferences",
        JSON.stringify({ state: { defaultAnalysisView }, version: 1 }),
      );

      await useMoneyballPreferences.persist.rehydrate();

      expect(useMoneyballPreferences.getState().defaultAnalysisView).toBe(
        defaultAnalysisView,
      );
    },
  );

  it.each([
    ["malformed JSON", "{"],
    [
      "unknown value",
      JSON.stringify({ state: { defaultAnalysisView: "unknown" }, version: 1 }),
    ],
    [
      "prior version",
      JSON.stringify({
        state: { defaultAnalysisView: "moneyball" },
        version: 0,
      }),
    ],
  ])("defaults %s to General", async (_label, storedValue) => {
    localStorage.setItem("fm-valuescout-moneyball-preferences", storedValue);

    await useMoneyballPreferences.persist.rehydrate();

    expect(useMoneyballPreferences.getState().defaultAnalysisView).toBe(
      "general",
    );
  });

  it("persists the shared Moneyball default", () => {
    useMoneyballPreferences.getState().setDefaultAnalysisView("moneyball");

    expect(useMoneyballPreferences.getState().defaultAnalysisView).toBe(
      "moneyball",
    );
    expect(
      JSON.parse(
        localStorage.getItem("fm-valuescout-moneyball-preferences") ?? "{}",
      ),
    ).toMatchObject({
      state: { defaultAnalysisView: "moneyball" },
      version: 1,
    });
  });
});
