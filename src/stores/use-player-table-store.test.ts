import { beforeEach, describe, expect, it } from "vitest";
import {
  getMoneyballSearchMetric,
  MONEYBALL_SEARCH_METRICS,
} from "@/utils/moneyball-search-metrics";
import { getPlayerMetric, PLAYER_METRICS } from "@/utils/player-metrics";
import {
  defaultPlayerTableLayouts,
  PLAYER_TABLE_LAYOUT_STORAGE_KEY,
  usePlayerTableStore,
} from "./use-player-table-store";

const DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS = [
  "name",
  "age",
  "nationality",
  "ca",
  "pa",
  "value",
];

describe("usePlayerTableStore", () => {
  beforeEach(() => {
    localStorage.clear();
    usePlayerTableStore.setState({ layouts: defaultPlayerTableLayouts() });
  });

  it("starts player layouts without duplicate Club and Division columns", () => {
    expect(defaultPlayerTableLayouts()).toMatchObject({
      search: { columnIds: DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS },
      "moneyball-search": {
        columnIds: [
          "name",
          "age",
          "nationality",
          "moneyball.minutes",
          "moneyball.average_rating",
          "moneyball.goals_per_90",
          "moneyball.assists_per_90",
          "moneyball.xg_per_90",
          "moneyball.xa_per_90",
        ],
      },
      squad: { columnIds: DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS },
    });
  });

  it("migrates v4 default-like player layouts without changing unrelated preferences", async () => {
    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({
        state: {
          layouts: {
            search: {
              columnIds: [
                "name",
                "age",
                "nationality",
                "club",
                "division",
                "ca",
                "pa",
                "value",
              ],
              widths: { name: 240, club: 192, division: 168, ca: 104 },
            },
            "moneyball-search": {
              columnIds: [
                "name",
                "age",
                "nationality",
                "club",
                "division",
                "moneyball.minutes",
              ],
              widths: {
                name: 240,
                club: 192,
                division: 168,
                "moneyball.minutes": 128,
              },
            },
            squad: {
              columnIds: [
                "name",
                "age",
                "nationality",
                "club",
                "division",
                "ca",
                "pa",
                "value",
              ],
              widths: { name: 240, club: 192, division: 168, ca: 104 },
            },
            "staff-search": {
              columnIds: ["name", "club", "role.scout"],
              widths: { club: 220, "role.scout": 184 },
            },
          },
        },
        version: 4,
      }),
    );

    await usePlayerTableStore.persist.rehydrate();

    expect(usePlayerTableStore.getState().layouts).toMatchObject({
      search: {
        columnIds: DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS,
        widths: { name: 240, ca: 104 },
      },
      "moneyball-search": {
        columnIds: ["name", "age", "nationality", "moneyball.minutes"],
        widths: { name: 240, "moneyball.minutes": 128 },
      },
      squad: {
        columnIds: DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS,
        widths: { name: 240, ca: 104 },
      },
      "staff-search": {
        columnIds: ["name", "club", "role.scout"],
        widths: { club: 220, "role.scout": 184 },
      },
    });
  });

  it("migrates v4 identity-only player layouts to Name without resetting defaults", async () => {
    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({
        state: {
          layouts: {
            search: {
              columnIds: ["club"],
              widths: { club: 192, ca: 104, name: 240 },
            },
            "moneyball-search": {
              columnIds: ["division"],
              widths: { division: 168, "moneyball.minutes": 128 },
            },
            squad: {
              columnIds: ["club", "division"],
              widths: { club: 192, division: 168, ca: 104 },
            },
          },
        },
        version: 4,
      }),
    );

    await usePlayerTableStore.persist.rehydrate();

    const layouts = usePlayerTableStore.getState().layouts;
    expect(layouts.search).toEqual({ columnIds: ["name"], widths: {} });
    expect(layouts["moneyball-search"]).toEqual({
      columnIds: ["name"],
      widths: {},
    });
    expect(layouts.squad).toEqual({ columnIds: ["name"], widths: {} });
  });

  it.each([5, 6])(
    "preserves explicitly re-added Club and Division in v%s layouts",
    async (version) => {
      localStorage.setItem(
        PLAYER_TABLE_LAYOUT_STORAGE_KEY,
        JSON.stringify({
          state: {
            layouts: {
              search: {
                columnIds: ["club", "name", "division", "ca"],
                widths: { club: 192, name: 240, division: 168, ca: 104 },
              },
              "moneyball-search": {
                columnIds: ["division", "moneyball.minutes", "club", "name"],
                widths: {
                  division: 168,
                  "moneyball.minutes": 128,
                  club: 192,
                  name: 240,
                },
              },
              squad: {
                columnIds: ["ca", "club", "division", "name"],
                widths: { ca: 104, club: 192, division: 168, name: 240 },
              },
            },
          },
          version,
        }),
      );

      await usePlayerTableStore.persist.rehydrate();

      const layouts = usePlayerTableStore.getState().layouts;
      expect(layouts.search).toEqual({
        columnIds: ["club", "name", "division", "ca"],
        widths: { club: 192, name: 240, division: 168, ca: 104 },
      });
      expect(layouts["moneyball-search"]).toEqual({
        columnIds: ["division", "moneyball.minutes", "club", "name"],
        widths: {
          division: 168,
          "moneyball.minutes": 128,
          club: 192,
          name: 240,
        },
      });
      expect(layouts.squad).toEqual({
        columnIds: ["ca", "club", "division", "name"],
        widths: { ca: 104, club: 192, division: 168, name: 240 },
      });
    },
  );

  it("migrates v4 custom player layouts while retaining Club and Division picker metrics", async () => {
    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({
        state: {
          layouts: {
            search: {
              columnIds: [
                "value",
                "club",
                "attr.Acceleration",
                "division",
                "name",
              ],
              widths: {
                value: 112,
                club: 192,
                "attr.Acceleration": 216,
                division: 168,
                name: 240,
              },
            },
            "moneyball-search": {
              columnIds: [
                "moneyball.minutes",
                "club",
                "moneyball.goals_per_90",
                "division",
                "name",
              ],
              widths: {
                "moneyball.minutes": 128,
                club: 192,
                "moneyball.goals_per_90": 112,
                division: 168,
                name: 240,
              },
            },
            squad: {
              columnIds: ["ca", "club", "name", "division", "attr.Agility"],
              widths: {
                ca: 104,
                club: 192,
                name: 240,
                division: 168,
                "attr.Agility": 216,
              },
            },
          },
        },
        version: 4,
      }),
    );

    await usePlayerTableStore.persist.rehydrate();

    expect(usePlayerTableStore.getState().layouts).toMatchObject({
      search: {
        columnIds: ["value", "attr.Acceleration", "name"],
        widths: { value: 112, "attr.Acceleration": 216, name: 240 },
      },
      "moneyball-search": {
        columnIds: ["moneyball.minutes", "moneyball.goals_per_90", "name"],
        widths: {
          "moneyball.minutes": 128,
          "moneyball.goals_per_90": 112,
          name: 240,
        },
      },
      squad: {
        columnIds: ["ca", "name", "attr.Agility"],
        widths: { ca: 104, name: 240, "attr.Agility": 216 },
      },
    });
    expect(PLAYER_METRICS.map((metric) => metric.id)).toEqual(
      expect.arrayContaining(["club", "division"]),
    );
    expect(MONEYBALL_SEARCH_METRICS.map((metric) => metric.id)).toEqual(
      expect.arrayContaining(["club", "division"]),
    );
    expect(getPlayerMetric("club")?.sortable).toBe(true);
    expect(getMoneyballSearchMetric("division")?.sortable).toBe(true);
  });

  it("hydrates safe independent layouts from malformed saved preferences", async () => {
    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({
        state: {
          layouts: {
            search: {
              columnIds: ["unknown.metric", "ca", "ca", "name"],
              widths: { ca: -10, name: 10_000, "unknown.metric": 180 },
            },
            squad: { columnIds: [], widths: { ca: 200 } },
          },
        },
        version: 1,
      }),
    );

    await usePlayerTableStore.persist.rehydrate();

    expect(usePlayerTableStore.getState().layouts.search).toEqual({
      columnIds: ["ca", "name"],
      widths: { ca: 72, name: 360 },
    });
    expect(usePlayerTableStore.getState().layouts.squad).toEqual({
      columnIds: [...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS],
      widths: { ca: 200 },
    });
  });

  it("retains Club DNA in current Search and Squad layouts without defaulting it", async () => {
    const store = usePlayerTableStore.getState();
    store.addColumns("search", ["club_dna"]);
    store.addColumns("squad", ["club_dna"]);

    expect(defaultPlayerTableLayouts().search.columnIds).not.toContain(
      "club_dna",
    );
    expect(defaultPlayerTableLayouts().squad.columnIds).not.toContain(
      "club_dna",
    );

    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({
        state: { layouts: usePlayerTableStore.getState().layouts },
        version: 5,
      }),
    );
    await usePlayerTableStore.persist.rehydrate();

    expect(usePlayerTableStore.getState().layouts.search.columnIds).toContain(
      "club_dna",
    );
    expect(usePlayerTableStore.getState().layouts.squad.columnIds).toContain(
      "club_dna",
    );
  });

  it("keeps table layouts independent while appending columns once and clamping widths", () => {
    const store = usePlayerTableStore.getState();

    store.addColumns("search", ["attr.Acceleration", "attr.Acceleration"]);
    store.setColumnWidth("search", "attr.Acceleration", 10_000);
    store.removeColumn("search", "name");

    expect(usePlayerTableStore.getState().layouts.search).toMatchObject({
      columnIds: [
        "age",
        "nationality",
        "ca",
        "pa",
        "value",
        "attr.Acceleration",
      ],
      widths: { "attr.Acceleration": 360 },
    });
    expect(usePlayerTableStore.getState().layouts.squad).toEqual({
      columnIds: [...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS],
      widths: {},
    });
  });

  it("adds independent staff layout slots without changing player layouts", () => {
    usePlayerTableStore.setState({
      layouts: {
        ...defaultPlayerTableLayouts(),
        "staff-search": { columnIds: [], widths: {} },
        "my-staff": { columnIds: [], widths: {} },
        "staff-shortlist": { columnIds: [], widths: {} },
      },
    });
    const store = usePlayerTableStore.getState();
    store.addColumns("staff-search", ["role.scout", "attr.Adaptability"]);
    store.setColumnWidth("staff-search", "role.scout", 184);
    store.moveColumn("staff-search", "role.scout", 1);

    expect(usePlayerTableStore.getState().layouts["staff-search"]).toEqual({
      columnIds: ["attr.Adaptability", "role.scout"],
      widths: { "role.scout": 184 },
    });
    expect(usePlayerTableStore.getState().layouts["my-staff"]).toEqual({
      columnIds: [],
      widths: {},
    });
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      ...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS,
    ]);
  });

  it("starts Shortlist with its recruitment context before score columns", () => {
    expect(
      defaultPlayerTableLayouts()["staff-shortlist"].columnIds.slice(0, 9),
    ).toEqual([
      "name",
      "age",
      "nationality",
      "club",
      "ca",
      "pa",
      "preferred_job",
      "club_job",
      "coaching_qualifications",
    ]);
  });

  it("adds the current Shortlist defaults when hydrating v2 preferences", async () => {
    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({
        state: {
          layouts: {
            search: { columnIds: ["name"], widths: {} },
            squad: { columnIds: ["name"], widths: {} },
            "staff-search": { columnIds: ["name"], widths: {} },
            "my-staff": { columnIds: ["name"], widths: {} },
          },
        },
        version: 2,
      }),
    );

    await usePlayerTableStore.persist.rehydrate();

    expect(
      usePlayerTableStore
        .getState()
        .layouts["staff-shortlist"].columnIds.slice(0, 9),
    ).toEqual([
      "name",
      "age",
      "nationality",
      "club",
      "ca",
      "pa",
      "preferred_job",
      "club_job",
      "coaching_qualifications",
    ]);
  });

  it("moves a visible column to either edge without changing its width or the other table", () => {
    const store = usePlayerTableStore.getState();
    store.addColumns("search", ["club"]);
    store.setColumnWidth("search", "club", 248);

    store.moveColumn("search", "club", 0);
    expect(usePlayerTableStore.getState().layouts.search).toEqual({
      columnIds: ["club", "name", "age", "nationality", "ca", "pa", "value"],
      widths: { club: 248 },
    });

    store.moveColumn("search", "club", 6);
    expect(usePlayerTableStore.getState().layouts.search).toEqual({
      columnIds: ["name", "age", "nationality", "ca", "pa", "value", "club"],
      widths: { club: 248 },
    });
    expect(usePlayerTableStore.getState().layouts.squad).toEqual({
      columnIds: [...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS],
      widths: {},
    });
  });

  it("ignores unknown, out-of-range, and no-op column moves", () => {
    const before = usePlayerTableStore.getState().layouts.search;
    const store = usePlayerTableStore.getState();

    store.moveColumn("search", "unknown.metric", 0);
    expect(usePlayerTableStore.getState().layouts.search).toEqual(before);
    store.moveColumn("search", "ca", -1);
    expect(usePlayerTableStore.getState().layouts.search).toEqual(before);
    store.moveColumn("search", "ca", 99);
    expect(usePlayerTableStore.getState().layouts.search).toEqual(before);
    store.moveColumn("search", "ca", Number.NaN);
    expect(usePlayerTableStore.getState().layouts.search).toEqual(before);
    store.moveColumn("search", "ca", 2.5);
    expect(usePlayerTableStore.getState().layouts.search).toEqual(before);
    store.moveColumn("search", "ca", 3);

    expect(usePlayerTableStore.getState().layouts.search).toEqual(before);
  });

  it("does not remove the last visible column", () => {
    usePlayerTableStore.setState({
      layouts: {
        search: { columnIds: ["name"], widths: {} },
        "moneyball-search": {
          columnIds: ["moneyball.average_rating"],
          widths: {},
        },
        squad: {
          columnIds: [...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS],
          widths: {},
        },
        "staff-search": { columnIds: [], widths: {} },
        "my-staff": { columnIds: [], widths: {} },
        "staff-shortlist": { columnIds: [], widths: {} },
      },
    });

    usePlayerTableStore.getState().removeColumn("search", "name");

    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      "name",
    ]);
  });
});
