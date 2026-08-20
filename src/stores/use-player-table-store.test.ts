import { beforeEach, describe, expect, it } from "vitest";
import { DEFAULT_PLAYER_TABLE_COLUMN_IDS } from "@/utils/player-metrics";
import {
  defaultPlayerTableLayouts,
  PLAYER_TABLE_LAYOUT_STORAGE_KEY,
  usePlayerTableStore,
} from "./use-player-table-store";

describe("usePlayerTableStore", () => {
  beforeEach(() => {
    localStorage.clear();
    usePlayerTableStore.setState({ layouts: defaultPlayerTableLayouts() });
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
      columnIds: [...DEFAULT_PLAYER_TABLE_COLUMN_IDS],
      widths: { ca: 200 },
    });
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
        "club",
        "division",
        "ca",
        "pa",
        "value",
        "attr.Acceleration",
      ],
      widths: { "attr.Acceleration": 360 },
    });
    expect(usePlayerTableStore.getState().layouts.squad).toEqual({
      columnIds: [...DEFAULT_PLAYER_TABLE_COLUMN_IDS],
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
      ...DEFAULT_PLAYER_TABLE_COLUMN_IDS,
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
    store.setColumnWidth("search", "club", 248);

    store.moveColumn("search", "club", 0);
    expect(usePlayerTableStore.getState().layouts.search).toEqual({
      columnIds: [
        "club",
        "name",
        "age",
        "nationality",
        "division",
        "ca",
        "pa",
        "value",
      ],
      widths: { club: 248 },
    });

    store.moveColumn("search", "club", 7);
    expect(usePlayerTableStore.getState().layouts.search).toEqual({
      columnIds: [
        "name",
        "age",
        "nationality",
        "division",
        "ca",
        "pa",
        "value",
        "club",
      ],
      widths: { club: 248 },
    });
    expect(usePlayerTableStore.getState().layouts.squad).toEqual({
      columnIds: [...DEFAULT_PLAYER_TABLE_COLUMN_IDS],
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
    store.moveColumn("search", "ca", 5);

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
          columnIds: [...DEFAULT_PLAYER_TABLE_COLUMN_IDS],
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
