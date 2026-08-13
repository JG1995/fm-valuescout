import { beforeEach, describe, expect, it } from "vitest";
import { DEFAULT_PLAYER_TABLE_COLUMN_IDS } from "@/utils/player-metrics";
import {
  PLAYER_TABLE_LAYOUT_STORAGE_KEY,
  usePlayerTableStore,
} from "./use-player-table-store";

describe("usePlayerTableStore", () => {
  beforeEach(() => {
    localStorage.clear();
    usePlayerTableStore.setState({
      layouts: {
        search: {
          columnIds: [...DEFAULT_PLAYER_TABLE_COLUMN_IDS],
          widths: {},
        },
        squad: {
          columnIds: [...DEFAULT_PLAYER_TABLE_COLUMN_IDS],
          widths: {},
        },
      },
    });
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

  it("does not remove the last visible column", () => {
    usePlayerTableStore.setState({
      layouts: {
        search: { columnIds: ["name"], widths: {} },
        squad: {
          columnIds: [...DEFAULT_PLAYER_TABLE_COLUMN_IDS],
          widths: {},
        },
      },
    });

    usePlayerTableStore.getState().removeColumn("search", "name");

    expect(usePlayerTableStore.getState().layouts.search.columnIds).toEqual([
      "name",
    ]);
  });
});
