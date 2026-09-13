import { beforeEach, describe, expect, it } from "vitest";
import {
  shortlistFilterKey,
  useShortlistFilterStore,
} from "./use-shortlist-filter-store";

describe("useShortlistFilterStore", () => {
  beforeEach(() => {
    useShortlistFilterStore.getState().reset();
  });

  const choice = (
    surface: "player" | "staff",
    context: { saveId: number; contextToken: string },
  ) =>
    useShortlistFilterStore.getState().choices[
      shortlistFilterKey(surface, context)
    ];

  it("treats an unset choice as uninitialized", () => {
    const context = { saveId: 1, contextToken: "one" };
    expect(choice("player", context)).toBeUndefined();
  });

  it("initializes from presence only once and never overwrites a manual choice", () => {
    const context = { saveId: 1, contextToken: "one" };
    useShortlistFilterStore.getState().initialize("player", context, true);
    expect(choice("player", context)).toBe(true);

    // A later presence refresh must not overwrite the initialized choice.
    useShortlistFilterStore.getState().initialize("player", context, false);
    expect(choice("player", context)).toBe(true);

    // A manual toggle always wins, and later refreshes stay no-ops.
    useShortlistFilterStore.getState().set("player", context, false);
    expect(choice("player", context)).toBe(false);
    useShortlistFilterStore.getState().initialize("player", context, true);
    expect(choice("player", context)).toBe(false);
  });

  it("scopes choices by save context and surface so state cannot leak", () => {
    const saveOne = { saveId: 1, contextToken: "one" };
    const saveTwo = { saveId: 2, contextToken: "two" };
    useShortlistFilterStore.getState().set("player", saveOne, true);
    useShortlistFilterStore.getState().set("staff", saveOne, false);
    useShortlistFilterStore.getState().initialize("player", saveTwo, false);

    expect(choice("player", saveOne)).toBe(true);
    expect(choice("staff", saveOne)).toBe(false);
    expect(choice("player", saveTwo)).toBe(false);
    expect(choice("staff", saveTwo)).toBeUndefined();
  });

  it("reset clears every scoped choice", () => {
    useShortlistFilterStore
      .getState()
      .set("player", { saveId: 1, contextToken: "one" }, true);
    useShortlistFilterStore
      .getState()
      .set("staff", { saveId: 2, contextToken: "two" }, false);
    useShortlistFilterStore.getState().reset();
    expect(useShortlistFilterStore.getState().choices).toEqual({});
  });
});
