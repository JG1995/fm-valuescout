import { beforeEach, describe, expect, it } from "vitest";
import { useSavedFiltersStore } from "./use-saved-filters-store";

const STORAGE_KEY = "fm-valuescout-saved-filters";

const pace = [{ id: "pace", field: "attr.Pace", op: "gte", value: 14 }];
const xg = [
  { id: "xg", field: "moneyball.xg_per_90", op: "pct_gte", value: 80 },
];

describe("useSavedFiltersStore", () => {
  beforeEach(() => {
    localStorage.clear();
    useSavedFiltersStore.setState({ savedFilters: [] });
  });

  it("saves a named set and replaces one saved under the same name", () => {
    const { save } = useSavedFiltersStore.getState();
    save({
      name: " Fast wingers ",
      filters: pace,
      combine: "and",
      comparison: "own",
    });
    const [first] = useSavedFiltersStore.getState().savedFilters;
    save({
      name: "fast WINGERS",
      filters: xg,
      combine: "or",
      comparison: "winger+wide_midfielder",
    });

    expect(useSavedFiltersStore.getState().savedFilters).toEqual([
      {
        id: first?.id,
        name: "fast WINGERS",
        filters: xg,
        combine: "or",
        comparison: "winger+wide_midfielder",
      },
    ]);
  });

  it("ignores a blank name and lets a rename replace the set it collides with", () => {
    const { save } = useSavedFiltersStore.getState();
    save({ name: "   ", filters: pace, combine: "and", comparison: "own" });
    expect(useSavedFiltersStore.getState().savedFilters).toEqual([]);

    save({ name: "Pace", filters: pace, combine: "and", comparison: "own" });
    save({ name: "xG", filters: xg, combine: "and", comparison: "own" });
    const xgId = useSavedFiltersStore
      .getState()
      .savedFilters.find((saved) => saved.name === "xG")?.id as string;
    useSavedFiltersStore.getState().rename(xgId, " pace ");

    expect(
      useSavedFiltersStore
        .getState()
        .savedFilters.map(({ id, name, filters }) => ({ id, name, filters })),
    ).toEqual([{ id: xgId, name: "pace", filters: xg }]);

    useSavedFiltersStore.getState().remove(xgId);
    expect(useSavedFiltersStore.getState().savedFilters).toEqual([]);
  });

  it("keeps only well-formed stored sets and normalizes their settings", async () => {
    localStorage.setItem(
      STORAGE_KEY,
      JSON.stringify({
        state: {
          savedFilters: [
            {
              id: "kept",
              name: " Kept ",
              filters: pace,
              combine: "xor",
              comparison: "winger+winger",
            },
            { id: "no-name", name: " ", filters: pace },
            { id: "no-filters", name: "Broken" },
            "not an object",
          ],
        },
        version: 1,
      }),
    );

    await useSavedFiltersStore.persist.rehydrate();

    expect(useSavedFiltersStore.getState().savedFilters).toEqual([
      {
        id: "kept",
        name: "Kept",
        filters: pace,
        combine: "and",
        comparison: "own",
      },
    ]);
  });
});
