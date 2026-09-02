import { beforeEach, describe, expect, it } from "vitest";
import {
  getMoneyballSearchMetric,
  MONEYBALL_SEARCH_METRICS,
} from "@/utils/moneyball-search-metrics";
import { getPlayerMetric, PLAYER_METRICS } from "@/utils/player-metrics";
import { TACTIC_LANE_IDS } from "@/utils/tactic-ids";
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

  it("migrates version-5 persisted Shortlist layouts preserving existing orders and widths", async () => {
    const customSearch = {
      columnIds: ["name", "age", "ca", "attr.Acceleration"],
      widths: { name: 240, ca: 104, "attr.Acceleration": 216 },
    };
    const customMoneyball = {
      columnIds: ["name", "moneyball.average_rating", "moneyball.goals_per_90"],
      widths: { name: 240, "moneyball.average_rating": 128 },
    };
    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({
        state: {
          layouts: {
            search: customSearch,
            "moneyball-search": customMoneyball,
            squad: {
              columnIds: [...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS],
              widths: {},
            },
          },
        },
        version: 5,
      }),
    );

    await usePlayerTableStore.persist.rehydrate();

    const layouts = usePlayerTableStore.getState().layouts;
    expect(layouts.search).toEqual(customSearch);
    expect(layouts["moneyball-search"]).toEqual(customMoneyball);
    expect(layouts.shortlist).toEqual({
      columnIds: [...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS],
      widths: {},
    });
  });

  it("preserves version-5 identity-only Club and Division layouts without fallback to Name", async () => {
    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({
        state: {
          layouts: {
            search: { columnIds: ["club"], widths: { club: 192 } },
            "moneyball-search": {
              columnIds: ["division"],
              widths: { division: 168 },
            },
          },
        },
        version: 5,
      }),
    );

    await usePlayerTableStore.persist.rehydrate();

    const layouts = usePlayerTableStore.getState().layouts;
    expect(layouts.search).toEqual({
      columnIds: ["club"],
      widths: { club: 192 },
    });
    expect(layouts["moneyball-search"]).toEqual({
      columnIds: ["division"],
      widths: { division: 168 },
    });
    expect(layouts.shortlist).toEqual({
      columnIds: [...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS],
      widths: {},
    });
  });

  it("does not remove the last visible column", () => {
    usePlayerTableStore.setState({
      layouts: {
        search: { columnIds: ["name"], widths: {} },
        "moneyball-search": {
          columnIds: ["moneyball.average_rating"],
          widths: {},
        },
        shortlist: { columnIds: ["name"], widths: {} },
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

  describe("tactic column persistence (Commit 3)", () => {
    const currentGroup = TACTIC_LANE_IDS.map(
      (laneId) => `tactic_current.${laneId}`,
    );
    const potentialGroup = TACTIC_LANE_IDS.map(
      (laneId) => `tactic_potential.${laneId}`,
    );

    it("hydrates a valid synthetic group round-trip and clamps tactic widths", async () => {
      localStorage.setItem(
        PLAYER_TABLE_LAYOUT_STORAGE_KEY,
        JSON.stringify({
          state: {
            layouts: {
              search: {
                columnIds: ["name", ...currentGroup, "ca"],
                widths: {
                  name: 240,
                  "tactic_current.goalkeeper": 10_000,
                  "tactic_current.left_back": 72,
                  ca: 104,
                },
              },
              "moneyball-search": {
                columnIds: ["name", ...potentialGroup],
                widths: { "tactic_potential.goalkeeper": 10_000 },
              },
              shortlist: {
                columnIds: [...currentGroup, ...potentialGroup],
                widths: {},
              },
            },
          },
          version: 6,
        }),
      );
      await usePlayerTableStore.persist.rehydrate();
      const layouts = usePlayerTableStore.getState().layouts;
      expect(layouts.search.columnIds).toEqual(["name", ...currentGroup, "ca"]);
      expect(layouts.search.widths["tactic_current.goalkeeper"]).toBe(360);
      expect(layouts.search.widths["tactic_current.left_back"]).toBe(72);
      expect(layouts["moneyball-search"].columnIds).toEqual([
        "name",
        ...potentialGroup,
      ]);
      expect(
        layouts["moneyball-search"].widths["tactic_potential.goalkeeper"],
      ).toBe(360);
      expect(layouts.shortlist.columnIds).toEqual([
        ...currentGroup,
        ...potentialGroup,
      ]);
    });

    it("drops invalid lane suffixes and widths for unknown tactic IDs", async () => {
      localStorage.setItem(
        PLAYER_TABLE_LAYOUT_STORAGE_KEY,
        JSON.stringify({
          state: {
            layouts: {
              search: {
                columnIds: [
                  "name",
                  "tactic_current.goalkeeper",
                  "tactic_current.not_a_lane",
                  "tactic_potential.not_a_lane",
                  "tactic_current.",
                  "ca",
                ],
                widths: {
                  "tactic_current.goalkeeper": 120,
                  "tactic_current.not_a_lane": 120,
                  "tactic_potential.not_a_lane": 120,
                },
              },
            },
          },
          version: 6,
        }),
      );
      await usePlayerTableStore.persist.rehydrate();
      const search = usePlayerTableStore.getState().layouts.search;
      expect(search.columnIds).toEqual([
        "name",
        "tactic_current.goalkeeper",
        "ca",
      ]);
      expect(search.widths).toEqual({
        "tactic_current.goalkeeper": 120,
      });
      expect(search.widths["tactic_current.not_a_lane"]).toBeUndefined();
      expect(search.widths["tactic_potential.not_a_lane"]).toBeUndefined();
    });

    it("rejects valid synthetic IDs for squad and staff tables", async () => {
      localStorage.setItem(
        PLAYER_TABLE_LAYOUT_STORAGE_KEY,
        JSON.stringify({
          state: {
            layouts: {
              squad: {
                columnIds: ["name", ...currentGroup, "ca"],
                widths: { "tactic_current.goalkeeper": 150 },
              },
              "staff-search": {
                columnIds: ["name", ...potentialGroup],
                widths: { "tactic_potential.goalkeeper": 150 },
              },
              "my-staff": {
                columnIds: [...currentGroup],
                widths: {},
              },
            },
          },
          version: 6,
        }),
      );
      await usePlayerTableStore.persist.rehydrate();
      expect(
        usePlayerTableStore.getState().layouts.squad.columnIds,
      ).not.toEqual(expect.arrayContaining(currentGroup));
      expect(usePlayerTableStore.getState().layouts.squad.columnIds).toEqual([
        "name",
        "ca",
      ]);
      expect(
        usePlayerTableStore.getState().layouts.squad.widths[
          "tactic_current.goalkeeper"
        ],
      ).toBeUndefined();
      expect(
        usePlayerTableStore.getState().layouts["staff-search"].columnIds,
      ).not.toEqual(expect.arrayContaining(potentialGroup));
      expect(
        usePlayerTableStore.getState().layouts["my-staff"].columnIds,
      ).not.toEqual(expect.arrayContaining(currentGroup));
      expect(
        usePlayerTableStore.getState().layouts["my-staff"].columnIds.length,
      ).toBeGreaterThan(0);
      expect(
        usePlayerTableStore.getState().layouts["my-staff"].widths[
          "tactic_current.goalkeeper"
        ],
      ).toBeUndefined();
    });

    it("leaves staff tables unchanged when adding valid tactic IDs via public action", () => {
      for (const table of [
        "staff-search",
        "my-staff",
        "staff-shortlist",
      ] as const) {
        const before = usePlayerTableStore.getState().layouts[table];
        usePlayerTableStore
          .getState()
          .addColumns(table, [
            "tactic_current.goalkeeper",
            "tactic_potential.goalkeeper",
          ]);
        expect(usePlayerTableStore.getState().layouts[table]).toEqual(before);
      }
    });

    it("atomically replaces layout via replaceLayout, deduplicates and preserves order", () => {
      expect(typeof usePlayerTableStore.getState().replaceLayout).toBe(
        "function",
      );
      const next = [
        "ca",
        "tactic_current.goalkeeper",
        "ca",
        "tactic_current.goalkeeper",
        "tactic_potential.goalkeeper",
        "tactic_current.not_a_lane",
        "name",
      ];
      usePlayerTableStore.getState().replaceLayout("search", next);
      const layout = usePlayerTableStore.getState().layouts.search;
      expect(layout.columnIds).toEqual([
        "ca",
        "tactic_current.goalkeeper",
        "tactic_potential.goalkeeper",
        "name",
      ]);
    });

    it("prunes stale widths in the same atomic replaceLayout write", () => {
      usePlayerTableStore.setState({
        layouts: {
          ...defaultPlayerTableLayouts(),
          search: {
            columnIds: ["name", "ca", "tactic_current.goalkeeper"],
            widths: {
              name: 240,
              ca: 104,
              "tactic_current.goalkeeper": 200,
            },
          },
        },
      });
      usePlayerTableStore
        .getState()
        .replaceLayout("search", ["name", "tactic_current.left_back"]);
      const layout = usePlayerTableStore.getState().layouts.search;
      expect(layout.columnIds).toEqual(["name", "tactic_current.left_back"]);
      expect(layout.widths["tactic_current.goalkeeper"]).toBeUndefined();
      expect(layout.widths["tactic_current.left_back"]).toBeUndefined();
      expect(layout.widths.name).toBe(240);
      expect(Object.keys(layout.widths)).toEqual(["name"]);
    });

    it("emits exactly one store notification with complete next layout and pruned widths", () => {
      usePlayerTableStore.setState({
        layouts: {
          ...defaultPlayerTableLayouts(),
          search: {
            columnIds: ["name", "ca", "tactic_current.goalkeeper"],
            widths: { name: 240, ca: 104, "tactic_current.goalkeeper": 200 },
          },
        },
      });
      const snapshots: Array<{
        columnIds: string[];
        widths: Record<string, number>;
      }> = [];
      const unsub = usePlayerTableStore.subscribe((state) => {
        snapshots.push({
          columnIds: [...state.layouts.search.columnIds],
          widths: { ...state.layouts.search.widths },
        });
      });
      usePlayerTableStore
        .getState()
        .replaceLayout("search", ["name", "tactic_current.left_back"]);
      unsub();
      expect(snapshots).toHaveLength(1);
      expect(snapshots[0].columnIds).toEqual([
        "name",
        "tactic_current.left_back",
      ]);
      expect(snapshots[0].widths).toEqual({ name: 240 });
    });

    it("clamps retained widths via replaceLayout", () => {
      usePlayerTableStore.setState({
        layouts: {
          ...defaultPlayerTableLayouts(),
          search: {
            columnIds: ["name", "tactic_current.goalkeeper"],
            widths: { name: 10_000, "tactic_current.goalkeeper": 10_000 },
          },
        },
      });
      usePlayerTableStore
        .getState()
        .replaceLayout("search", ["name", "tactic_current.goalkeeper"]);
      expect(usePlayerTableStore.getState().layouts.search.widths.name).toBe(
        360,
      );
      expect(
        usePlayerTableStore.getState().layouts.search.widths[
          "tactic_current.goalkeeper"
        ],
      ).toBe(360);
    });

    it("falls back to default layout when tactic-only layout toggled off", () => {
      usePlayerTableStore.setState({
        layouts: {
          ...defaultPlayerTableLayouts(),
          search: { columnIds: [...currentGroup], widths: {} },
        },
      });
      usePlayerTableStore.getState().replaceLayout("search", []);
      expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual(
        defaultPlayerTableLayouts().search.columnIds,
      );
      expect(usePlayerTableStore.getState().layouts.search.widths).toEqual({});
    });

    it("falls back to defaults when replaceLayout filters to empty via invalid IDs", () => {
      usePlayerTableStore.setState({
        layouts: {
          ...defaultPlayerTableLayouts(),
          shortlist: { columnIds: ["name", "ca"], widths: { name: 240 } },
        },
      });
      usePlayerTableStore
        .getState()
        .replaceLayout("shortlist", [
          "tactic_current.not_a_lane",
          "tactic_potential.bogus",
        ]);
      expect(
        usePlayerTableStore.getState().layouts.shortlist.columnIds,
      ).toEqual(defaultPlayerTableLayouts().shortlist.columnIds);
    });
  });
});
