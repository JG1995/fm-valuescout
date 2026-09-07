import { beforeEach, describe, expect, it } from "vitest";
import {
  getMoneyballSearchMetric,
  MONEYBALL_SEARCH_METRICS,
} from "@/utils/moneyball-search-metrics";
import { getPlayerMetric, PLAYER_METRICS } from "@/utils/player-metrics";
import { DEFAULT_STAFF_TABLE_COLUMN_IDS } from "@/utils/staff-table-layout";
import { TACTIC_LANE_IDS } from "@/utils/tactic-ids";
import {
  defaultPlayerTableLayouts,
  PLAYER_TABLE_LAYOUT_STORAGE_KEY,
  usePlayerTableStore,
  withoutIdentityColumnIds,
} from "./use-player-table-store";

const DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS = [
  "age",
  "nationality",
  "height",
  "ca",
  "pa",
  "value",
];

describe("usePlayerTableStore", () => {
  beforeEach(() => {
    localStorage.clear();
    usePlayerTableStore.setState({ layouts: defaultPlayerTableLayouts() });
  });

  it("starts player tables with Height immediately after Nationality", () => {
    const layouts = defaultPlayerTableLayouts();
    expect(layouts.search.columnIds).toEqual([
      "age",
      "nationality",
      "height",
      "ca",
      "pa",
      "value",
    ]);
    expect(layouts["moneyball-search"].columnIds).toEqual([
      "age",
      "nationality",
      "height",
      "moneyball.minutes",
      "moneyball.average_rating",
      "moneyball.goals_per_90",
      "moneyball.assists_per_90",
      "moneyball.xg_per_90",
      "moneyball.xa_per_90",
    ]);
    expect(layouts.squad.columnIds).toEqual([
      "age",
      "nationality",
      "height",
      "ca",
      "pa",
      "value",
      "suggested_training",
    ]);
  });

  it("keeps persisted v8 layouts without Height exactly as stored", async () => {
    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({
        state: {
          layouts: {
            search: {
              columnIds: ["age", "nationality", "ca", "pa", "value"],
              widths: { ca: 104 },
              identityWidth: 320,
            },
            "moneyball-search": {
              columnIds: ["age", "nationality", "moneyball.minutes"],
              widths: {},
              identityWidth: 280,
            },
            squad: {
              columnIds: [
                "age",
                "nationality",
                "ca",
                "pa",
                "value",
                "suggested_training",
              ],
              widths: {},
              identityWidth: 280,
            },
          },
        },
        version: 8,
      }),
    );

    await usePlayerTableStore.persist.rehydrate();

    const layouts = usePlayerTableStore.getState().layouts;
    expect(layouts.search).toEqual({
      columnIds: ["age", "nationality", "ca", "pa", "value"],
      widths: { ca: 104 },
      identityWidth: 320,
    });
    expect(layouts["moneyball-search"]).toEqual({
      columnIds: ["age", "nationality", "moneyball.minutes"],
      widths: {},
      identityWidth: 280,
    });
    expect(layouts.squad).toEqual({
      columnIds: [
        "age",
        "nationality",
        "ca",
        "pa",
        "value",
        "suggested_training",
      ],
      widths: {},
      identityWidth: 280,
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
        columnIds: ["age", "nationality", "ca", "pa", "value"],
        widths: { ca: 104 },
        identityWidth: 240,
      },
      "moneyball-search": {
        columnIds: ["age", "nationality", "moneyball.minutes"],
        widths: { "moneyball.minutes": 128 },
        identityWidth: 240,
      },
      squad: {
        columnIds: ["age", "nationality", "ca", "pa", "value"],
        widths: { ca: 104 },
        identityWidth: 240,
      },
      "staff-search": {
        columnIds: ["role.scout"],
        widths: { "role.scout": 184 },
        identityWidth: 280,
      },
    });
  });

  it("migrates v4 identity-only player layouts to identity-only without resetting defaults", async () => {
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
    expect(layouts.search).toEqual({
      columnIds: [],
      widths: {},
      identityWidth: 240,
    });
    expect(layouts["moneyball-search"]).toEqual({
      columnIds: [],
      widths: {},
      identityWidth: 280,
    });
    expect(layouts.squad).toEqual({
      columnIds: [],
      widths: {},
      identityWidth: 280,
    });
  });

  it.each([5, 6])(
    "strips explicitly re-added Club and Division in v%s layouts while keeping valid columns",
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
        columnIds: ["ca"],
        widths: { ca: 104 },
        identityWidth: 240,
      });
      expect(layouts["moneyball-search"]).toEqual({
        columnIds: ["moneyball.minutes"],
        widths: { "moneyball.minutes": 128 },
        identityWidth: 240,
      });
      expect(layouts.squad).toEqual({
        columnIds: ["ca"],
        widths: { ca: 104 },
        identityWidth: 240,
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
        columnIds: ["value", "attr.Acceleration"],
        widths: { value: 112, "attr.Acceleration": 216 },
        identityWidth: 240,
      },
      "moneyball-search": {
        columnIds: ["moneyball.minutes", "moneyball.goals_per_90"],
        widths: {
          "moneyball.minutes": 128,
          "moneyball.goals_per_90": 112,
        },
        identityWidth: 240,
      },
      squad: {
        columnIds: ["ca", "attr.Agility"],
        widths: { ca: 104, "attr.Agility": 216 },
        identityWidth: 240,
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
      columnIds: ["ca"],
      widths: { ca: 72 },
      identityWidth: 360,
    });
    expect(usePlayerTableStore.getState().layouts.squad).toEqual({
      columnIds: [
        "age",
        "nationality",
        "ca",
        "pa",
        "value",
        "suggested_training",
      ],
      widths: {},
      identityWidth: 280,
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
        "height",
        "ca",
        "pa",
        "value",
        "attr.Acceleration",
      ],
      widths: { "attr.Acceleration": 360 },
    });
    expect(usePlayerTableStore.getState().layouts.squad).toEqual({
      columnIds: [
        ...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS,
        "suggested_training",
      ],
      widths: {},
      identityWidth: 280,
    });
  });

  it("adds independent staff layout slots without changing player layouts", () => {
    usePlayerTableStore.setState({
      layouts: {
        ...defaultPlayerTableLayouts(),
        "staff-search": { columnIds: [], widths: {}, identityWidth: 280 },
        "my-staff": { columnIds: [], widths: {}, identityWidth: 280 },
        "staff-shortlist": { columnIds: [], widths: {}, identityWidth: 280 },
      },
    });
    const store = usePlayerTableStore.getState();
    store.addColumns("staff-search", ["role.scout", "attr.Adaptability"]);
    store.setColumnWidth("staff-search", "role.scout", 184);
    store.moveColumn("staff-search", "role.scout", 1);

    expect(usePlayerTableStore.getState().layouts["staff-search"]).toEqual({
      columnIds: ["attr.Adaptability", "role.scout"],
      widths: { "role.scout": 184 },
      identityWidth: 280,
    });
    expect(usePlayerTableStore.getState().layouts["my-staff"]).toEqual({
      columnIds: [],
      widths: {},
      identityWidth: 280,
    });
    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      ...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS,
    ]);
  });

  it("starts Shortlist with its recruitment context before score columns", () => {
    expect(
      defaultPlayerTableLayouts()["staff-shortlist"].columnIds.slice(0, 7),
    ).toEqual([
      "age",
      "nationality",
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
        .layouts["staff-shortlist"].columnIds.slice(0, 7),
    ).toEqual([
      "age",
      "nationality",
      "ca",
      "pa",
      "preferred_job",
      "club_job",
      "coaching_qualifications",
    ]);
  });

  it("moves a visible column to either edge without changing its width or the other table", () => {
    const store = usePlayerTableStore.getState();
    store.addColumns("search", ["reputation"]);
    store.setColumnWidth("search", "reputation", 248);

    store.moveColumn("search", "reputation", 0);
    expect(usePlayerTableStore.getState().layouts.search).toEqual({
      columnIds: [
        "reputation",
        "age",
        "nationality",
        "height",
        "ca",
        "pa",
        "value",
      ],
      widths: { reputation: 248 },
      identityWidth: 280,
    });

    store.moveColumn("search", "reputation", 6);
    expect(usePlayerTableStore.getState().layouts.search).toEqual({
      columnIds: [
        "age",
        "nationality",
        "height",
        "ca",
        "pa",
        "value",
        "reputation",
      ],
      widths: { reputation: 248 },
      identityWidth: 280,
    });
    expect(usePlayerTableStore.getState().layouts.squad).toEqual({
      columnIds: [
        ...DEFAULT_VISIBLE_PLAYER_TABLE_COLUMN_IDS,
        "suggested_training",
      ],
      widths: {},
      identityWidth: 280,
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

  it("migrates version-5 persisted layouts preserving existing orders and widths", async () => {
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
            shortlist: {
              columnIds: ["name", "ca"],
              widths: { name: 240 },
            },
          },
        },
        version: 5,
      }),
    );

    await usePlayerTableStore.persist.rehydrate();

    const layouts = usePlayerTableStore.getState().layouts;
    expect(layouts.search).toEqual({
      columnIds: ["age", "ca", "attr.Acceleration"],
      widths: { ca: 104, "attr.Acceleration": 216 },
      identityWidth: 240,
    });
    expect(layouts["moneyball-search"]).toEqual({
      columnIds: ["moneyball.average_rating", "moneyball.goals_per_90"],
      widths: { "moneyball.average_rating": 128 },
      identityWidth: 240,
    });
    expect(layouts).not.toHaveProperty("shortlist");
  });

  it("migrates version-5 identity-only Club and Division layouts to identity-only", async () => {
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
      columnIds: [],
      widths: {},
      identityWidth: 280,
    });
    expect(layouts["moneyball-search"]).toEqual({
      columnIds: [],
      widths: {},
      identityWidth: 280,
    });
  });

  it("allows removeColumn down to zero analysis columns for identity-only tables", () => {
    usePlayerTableStore.setState({
      layouts: {
        ...defaultPlayerTableLayouts(),
        search: { columnIds: ["ca"], widths: {}, identityWidth: 280 },
      },
    });

    usePlayerTableStore.getState().removeColumn("search", "ca");

    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([]);
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
      expect(layouts.search.columnIds).toEqual([...currentGroup, "ca"]);
      expect(layouts.search.widths["tactic_current.goalkeeper"]).toBe(360);
      expect(layouts.search.widths["tactic_current.left_back"]).toBe(72);
      expect(layouts.search.identityWidth).toBe(240);
      expect(layouts["moneyball-search"].columnIds).toEqual([
        ...potentialGroup,
      ]);
      expect(
        layouts["moneyball-search"].widths["tactic_potential.goalkeeper"],
      ).toBe(360);
      expect(layouts).not.toHaveProperty("shortlist");
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
      expect(search.columnIds).toEqual(["tactic_current.goalkeeper", "ca"]);
      expect(search.widths).toEqual({
        "tactic_current.goalkeeper": 120,
      });
      expect(search.identityWidth).toBe(280);
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
        usePlayerTableStore.getState().layouts["staff-search"].columnIds,
      ).toEqual([]);
      expect(
        usePlayerTableStore.getState().layouts["my-staff"].columnIds,
      ).not.toEqual(expect.arrayContaining(currentGroup));
      expect(
        usePlayerTableStore.getState().layouts["my-staff"].columnIds,
      ).toEqual(withoutIdentityColumnIds([...DEFAULT_STAFF_TABLE_COLUMN_IDS]));
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
      ]);
    });

    it("prunes stale widths in the same atomic replaceLayout write", () => {
      usePlayerTableStore.setState({
        layouts: {
          ...defaultPlayerTableLayouts(),
          search: {
            columnIds: ["ca", "tactic_current.goalkeeper"],
            widths: {
              ca: 104,
              "tactic_current.goalkeeper": 200,
            },
            identityWidth: 280,
          },
        },
      });
      usePlayerTableStore
        .getState()
        .replaceLayout("search", ["ca", "tactic_current.left_back"]);
      const layout = usePlayerTableStore.getState().layouts.search;
      expect(layout.columnIds).toEqual(["ca", "tactic_current.left_back"]);
      expect(layout.widths["tactic_current.goalkeeper"]).toBeUndefined();
      expect(layout.widths["tactic_current.left_back"]).toBeUndefined();
      expect(layout.widths.ca).toBe(104);
      expect(Object.keys(layout.widths)).toEqual(["ca"]);
    });

    it("emits exactly one store notification with complete next layout and pruned widths", () => {
      usePlayerTableStore.setState({
        layouts: {
          ...defaultPlayerTableLayouts(),
          search: {
            columnIds: ["ca", "tactic_current.goalkeeper"],
            widths: { ca: 104, "tactic_current.goalkeeper": 200 },
            identityWidth: 280,
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
        .replaceLayout("search", ["ca", "tactic_current.left_back"]);
      unsub();
      expect(snapshots).toHaveLength(1);
      expect(snapshots[0].columnIds).toEqual([
        "ca",
        "tactic_current.left_back",
      ]);
      expect(snapshots[0].widths).toEqual({ ca: 104 });
    });

    it("clamps retained widths via replaceLayout", () => {
      usePlayerTableStore.setState({
        layouts: {
          ...defaultPlayerTableLayouts(),
          search: {
            columnIds: ["ca", "tactic_current.goalkeeper"],
            widths: { ca: 10_000, "tactic_current.goalkeeper": 10_000 },
            identityWidth: 280,
          },
        },
      });
      usePlayerTableStore
        .getState()
        .replaceLayout("search", ["ca", "tactic_current.goalkeeper"]);
      expect(usePlayerTableStore.getState().layouts.search.widths.ca).toBe(360);
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
          search: {
            columnIds: [...currentGroup],
            widths: {},
            identityWidth: 280,
          },
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
          search: {
            columnIds: ["ca"],
            widths: { ca: 240 },
            identityWidth: 280,
          },
        },
      });
      usePlayerTableStore
        .getState()
        .replaceLayout("search", [
          "tactic_current.not_a_lane",
          "tactic_potential.bogus",
        ]);
      expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual(
        defaultPlayerTableLayouts().search.columnIds,
      );
    });
  });

  describe("suggested training column (Commit 5)", () => {
    it("allows Suggested Training only in the Squad table", () => {
      const store = usePlayerTableStore.getState();
      store.addColumns("squad", ["suggested_training"]);
      expect(usePlayerTableStore.getState().layouts.squad.columnIds).toContain(
        "suggested_training",
      );

      for (const table of [
        "search",
        "moneyball-search",
        "staff-search",
        "my-staff",
        "staff-shortlist",
      ] as const) {
        const before = usePlayerTableStore.getState().layouts[table];
        usePlayerTableStore
          .getState()
          .addColumns(table, ["suggested_training"]);
        expect(usePlayerTableStore.getState().layouts[table]).toEqual(before);
      }
    });

    it("appends Suggested Training to a v6 default-like Squad layout", async () => {
      localStorage.setItem(
        PLAYER_TABLE_LAYOUT_STORAGE_KEY,
        JSON.stringify({
          state: {
            layouts: {
              search: {
                columnIds: ["name", "age", "nationality", "ca", "pa", "value"],
                widths: {},
              },
              squad: {
                columnIds: ["name", "age", "nationality", "ca", "pa", "value"],
                widths: {},
              },
            },
          },
          version: 6,
        }),
      );

      await usePlayerTableStore.persist.rehydrate();

      const layouts = usePlayerTableStore.getState().layouts;
      expect(layouts.squad.columnIds).toEqual([
        "age",
        "nationality",
        "ca",
        "pa",
        "value",
        "suggested_training",
      ]);
      expect(layouts.squad.identityWidth).toBe(280);
      expect(layouts.search.columnIds).toEqual([
        "age",
        "nationality",
        "ca",
        "pa",
        "value",
      ]);
      expect(layouts.search.columnIds).not.toContain("suggested_training");
    });

    it("preserves a customized v6 Squad layout without appending", async () => {
      localStorage.setItem(
        PLAYER_TABLE_LAYOUT_STORAGE_KEY,
        JSON.stringify({
          state: {
            layouts: {
              squad: {
                columnIds: ["name", "ca", "attr.Acceleration"],
                widths: { name: 240 },
              },
            },
          },
          version: 6,
        }),
      );

      await usePlayerTableStore.persist.rehydrate();

      expect(usePlayerTableStore.getState().layouts.squad).toEqual({
        columnIds: ["ca", "attr.Acceleration"],
        widths: {},
        identityWidth: 240,
      });
    });

    it("preserves a v6 default-ID Squad layout with custom widths without appending", async () => {
      localStorage.setItem(
        PLAYER_TABLE_LAYOUT_STORAGE_KEY,
        JSON.stringify({
          state: {
            layouts: {
              squad: {
                columnIds: ["name", "age", "nationality", "ca", "pa", "value"],
                widths: { name: 240 },
              },
            },
          },
          version: 6,
        }),
      );

      await usePlayerTableStore.persist.rehydrate();

      expect(usePlayerTableStore.getState().layouts.squad).toEqual({
        columnIds: ["age", "nationality", "ca", "pa", "value"],
        widths: {},
        identityWidth: 240,
      });
    });

    it("strips Suggested Training from every non-Squad layout", async () => {
      localStorage.setItem(
        PLAYER_TABLE_LAYOUT_STORAGE_KEY,
        JSON.stringify({
          state: {
            layouts: {
              search: {
                columnIds: ["name", "suggested_training", "ca"],
                widths: {},
              },
              squad: {
                columnIds: ["name", "suggested_training", "ca"],
                widths: {},
              },
            },
          },
          version: 7,
        }),
      );

      await usePlayerTableStore.persist.rehydrate();

      const layouts = usePlayerTableStore.getState().layouts;
      expect(layouts.search.columnIds).toEqual(["ca"]);
      expect(layouts.search.identityWidth).toBe(280);
      expect(layouts.squad.columnIds).toEqual(["suggested_training", "ca"]);
      expect(layouts.squad.identityWidth).toBe(280);
    });
  });
});

describe("v8 identity migration (Commit 4)", () => {
  const V7_STAFF_SHORTLIST_DEFAULT: string[] = [
    "name",
    "age",
    "nationality",
    "club",
    "ca",
    "pa",
    "preferred_job",
    "club_job",
    "coaching_qualifications",
    ...DEFAULT_STAFF_TABLE_COLUMN_IDS.filter(
      (columnId) =>
        !["name", "age", "nationality", "ca", "pa"].includes(columnId),
    ),
  ];
  const V7_DEFAULTS: Record<string, string[]> = {
    search: ["name", "age", "nationality", "ca", "pa", "value"],
    "moneyball-search": [
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
    squad: [
      "name",
      "age",
      "nationality",
      "ca",
      "pa",
      "value",
      "suggested_training",
    ],
    "staff-search": [...DEFAULT_STAFF_TABLE_COLUMN_IDS],
    "my-staff": [...DEFAULT_STAFF_TABLE_COLUMN_IDS],
    "staff-shortlist": [...V7_STAFF_SHORTLIST_DEFAULT],
  };

  const V8_DEFAULTS: Record<string, string[]> = {
    search: ["age", "nationality", "height", "ca", "pa", "value"],
    "moneyball-search": [
      "age",
      "nationality",
      "height",
      "moneyball.minutes",
      "moneyball.average_rating",
      "moneyball.goals_per_90",
      "moneyball.assists_per_90",
      "moneyball.xg_per_90",
      "moneyball.xa_per_90",
    ],
    squad: [
      "age",
      "nationality",
      "height",
      "ca",
      "pa",
      "value",
      "suggested_training",
    ],
    "staff-search": withoutIdentityColumnIds([
      ...DEFAULT_STAFF_TABLE_COLUMN_IDS,
    ]),
    "my-staff": withoutIdentityColumnIds([...DEFAULT_STAFF_TABLE_COLUMN_IDS]),
    "staff-shortlist": withoutIdentityColumnIds(V7_STAFF_SHORTLIST_DEFAULT),
  };

  // Immutable historic v8 migration outputs (without Height). Pre-v8
  // default-like and malformed fallbacks must keep returning these exact
  // arrays; fresh defaults use V8_DEFAULTS above.
  const V8_HISTORIC_DEFAULTS: Record<string, string[]> = {
    search: ["age", "nationality", "ca", "pa", "value"],
    "moneyball-search": [
      "age",
      "nationality",
      "moneyball.minutes",
      "moneyball.average_rating",
      "moneyball.goals_per_90",
      "moneyball.assists_per_90",
      "moneyball.xg_per_90",
      "moneyball.xa_per_90",
    ],
    squad: ["age", "nationality", "ca", "pa", "value", "suggested_training"],
  };

  async function rehydrateVersion(
    layouts: Record<string, unknown>,
    version: number,
  ) {
    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({ state: { layouts }, version }),
    );
    await usePlayerTableStore.persist.rehydrate();
    return usePlayerTableStore.getState().layouts;
  }

  it("ships exact v8 analysis defaults with a separate 280 identity width", () => {
    const layouts = defaultPlayerTableLayouts();
    for (const [table, ids] of Object.entries(V8_DEFAULTS)) {
      expect(layouts[table as keyof typeof layouts].columnIds).toEqual(ids);
      expect(layouts[table as keyof typeof layouts].widths).toEqual({});
      expect(layouts[table as keyof typeof layouts].identityWidth).toBe(280);
    }
    const shortlist = layouts["staff-shortlist"];
    expect(shortlist.columnIds).toEqual(V8_DEFAULTS["staff-shortlist"]);
    expect(shortlist.columnIds).toContain("preferred_job");
    expect(shortlist.identityWidth).toBe(280);
  });

  it.each(Object.keys(V7_DEFAULTS))(
    "rolls a v7 default-like %s layout to the exact historic v8 defaults",
    async (table) => {
      const layouts = await rehydrateVersion(
        { [table]: { columnIds: V7_DEFAULTS[table], widths: {} } },
        7,
      );
      expect(layouts[table as keyof typeof layouts].columnIds).toEqual(
        V8_HISTORIC_DEFAULTS[table] ?? V8_DEFAULTS[table],
      );
      expect(layouts[table as keyof typeof layouts].widths).toEqual({});
    },
  );

  it("keeps a customized v7 layout with clamped widths and a captured identity width", async () => {
    const layouts = await rehydrateVersion(
      {
        search: {
          columnIds: ["ca", "name", "age", "attr.Acceleration"],
          widths: { ca: 104, name: 300, "attr.Acceleration": 10_000 },
        },
      },
      7,
    );
    expect(layouts.search).toEqual({
      columnIds: ["ca", "age", "attr.Acceleration"],
      widths: { ca: 104, "attr.Acceleration": 360 },
      identityWidth: 300,
    });
  });

  it.each([
    { nameWidth: 100, expected: 240 },
    { nameWidth: 300, expected: 300 },
    { nameWidth: 500, expected: 360 },
  ])(
    "clamps a captured widths.name of $nameWidth to identityWidth $expected",
    async ({ nameWidth, expected }) => {
      const layouts = await rehydrateVersion(
        {
          search: {
            columnIds: ["name", "ca"],
            widths: { name: nameWidth },
          },
        },
        7,
      );
      expect(layouts.search.identityWidth).toBe(expected);
      expect(layouts.search.widths.name).toBeUndefined();
    },
  );

  it.each([{ widths: {} }, { widths: { name: Number.NaN } }])(
    "falls back to identityWidth 280 when widths.name is absent or non-finite",
    async ({ widths }) => {
      const layouts = await rehydrateVersion(
        { search: { columnIds: ["name", "ca"], widths } },
        7,
      );
      expect(layouts.search.identityWidth).toBe(280);
    },
  );

  it("keeps identity-only layouts valid instead of rolling them to defaults", async () => {
    const layouts = await rehydrateVersion(
      {
        search: { columnIds: ["name", "club", "division"], widths: {} },
        squad: { columnIds: ["ca"], widths: {} },
      },
      7,
    );
    expect(layouts.search.columnIds).toEqual([]);
    expect(layouts.search.widths).toEqual({});
    expect(layouts.search.identityWidth).toBe(280);
    expect(layouts.squad.columnIds).toEqual(["ca"]);
  });

  it("falls back to historic v8 defaults with identityWidth 280 for malformed layouts", async () => {
    const layouts = await rehydrateVersion(
      {
        search: { widths: { ca: 200 } },
        "moneyball-search": { widths: { "moneyball.minutes": 200 } },
        squad: { columnIds: [], widths: {} },
      },
      7,
    );
    expect(layouts.search.columnIds).toEqual(V8_HISTORIC_DEFAULTS.search);
    expect(layouts.search.widths).toEqual({});
    expect(layouts.search.identityWidth).toBe(280);
    expect(layouts["moneyball-search"]).toEqual({
      columnIds: V8_HISTORIC_DEFAULTS["moneyball-search"],
      widths: {},
      identityWidth: 280,
    });
    expect(layouts.squad.columnIds).toEqual(V8_HISTORIC_DEFAULTS.squad);
  });

  it("drops tactic IDs outside Search/Moneyball and Suggested Training outside Squad", async () => {
    const layouts = await rehydrateVersion(
      {
        search: {
          columnIds: ["ca", "suggested_training", "name"],
          widths: {},
        },
        squad: {
          columnIds: ["ca", "tactic_current.goalkeeper", "name"],
          widths: {},
        },
      },
      7,
    );
    expect(layouts.search.columnIds).toEqual(["ca"]);
    expect(layouts.squad.columnIds).toEqual(["ca"]);
  });

  it("rejects identity IDs across addColumns, replaceLayout, moveColumn, and setColumnWidth", () => {
    const store = usePlayerTableStore.getState();
    const before = store.layouts.search;
    store.addColumns("search", ["name", "club", "division"]);
    expect(usePlayerTableStore.getState().layouts.search).toEqual(before);
    store.replaceLayout("search", ["name", "club", "division"]);
    expect(
      usePlayerTableStore.getState().layouts.search.columnIds,
    ).not.toContain("name");
    store.moveColumn("search", "name", 0);
    expect(usePlayerTableStore.getState().layouts.search).toEqual(
      usePlayerTableStore.getState().layouts.search,
    );
    store.setColumnWidth("search", "name", 300);
    expect(
      usePlayerTableStore.getState().layouts.search.widths.name,
    ).toBeUndefined();
  });

  it("clamps setIdentityWidth to the 240-360 identity bounds", () => {
    const store = usePlayerTableStore.getState();
    store.setIdentityWidth("search", 100);
    expect(usePlayerTableStore.getState().layouts.search.identityWidth).toBe(
      240,
    );
    store.setIdentityWidth("search", 500);
    expect(usePlayerTableStore.getState().layouts.search.identityWidth).toBe(
      360,
    );
    store.setIdentityWidth("search", 300);
    expect(usePlayerTableStore.getState().layouts.search.identityWidth).toBe(
      300,
    );
  });

  it("preserves identity width across nonempty replaceLayout and rehydration", async () => {
    usePlayerTableStore.setState({
      layouts: {
        ...defaultPlayerTableLayouts(),
        search: {
          columnIds: ["ca"],
          widths: { ca: 104 },
          identityWidth: 280,
        },
      },
    });
    usePlayerTableStore.getState().setIdentityWidth("search", 320);
    usePlayerTableStore
      .getState()
      .replaceLayout("search", ["ca", "tactic_current.goalkeeper"]);

    const layout = usePlayerTableStore.getState().layouts.search;
    expect(layout.columnIds).toEqual(["ca", "tactic_current.goalkeeper"]);
    expect(layout.widths.ca).toBe(104);
    expect(layout.identityWidth).toBe(320);

    localStorage.setItem(
      PLAYER_TABLE_LAYOUT_STORAGE_KEY,
      JSON.stringify({
        state: { layouts: usePlayerTableStore.getState().layouts },
        version: 8,
      }),
    );
    await usePlayerTableStore.persist.rehydrate();

    expect(usePlayerTableStore.getState().layouts.search.identityWidth).toBe(
      320,
    );
  });
});
