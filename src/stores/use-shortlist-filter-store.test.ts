import { beforeEach, describe, expect, it } from "vitest";
import {
  shortlistFilterKey,
  useShortlistFilterStore,
} from "./use-shortlist-filter-store";

describe("useShortlistFilterStore", () => {
  beforeEach(() => {
    useShortlistFilterStore.getState().reset();
  });

  const choice = (context: { saveId: number; contextToken: string }) =>
    useShortlistFilterStore.getState().choices[
      shortlistFilterKey("staff", context)
    ];

  it("treats an unset choice as uninitialized", () => {
    const context = { saveId: 1, contextToken: "one" };
    expect(choice(context)).toBeUndefined();
  });

  it("initializes from presence only once and never overwrites a manual choice", () => {
    const context = { saveId: 1, contextToken: "one" };
    useShortlistFilterStore.getState().initialize("staff", context, true);
    expect(choice(context)).toBe(true);

    // A later presence refresh must not overwrite the initialized choice.
    useShortlistFilterStore.getState().initialize("staff", context, false);
    expect(choice(context)).toBe(true);

    // A manual toggle always wins, and later refreshes stay no-ops.
    useShortlistFilterStore.getState().set("staff", context, false);
    expect(choice(context)).toBe(false);
    useShortlistFilterStore.getState().initialize("staff", context, true);
    expect(choice(context)).toBe(false);
  });

  it("scopes choices by save context so state cannot leak", () => {
    const saveOne = { saveId: 1, contextToken: "one" };
    const saveTwo = { saveId: 2, contextToken: "two" };
    useShortlistFilterStore.getState().set("staff", saveOne, true);
    useShortlistFilterStore.getState().initialize("staff", saveTwo, false);

    expect(choice(saveOne)).toBe(true);
    expect(choice(saveTwo)).toBe(false);
    expect(choice({ saveId: 1, contextToken: "replaced" })).toBeUndefined();
  });

  it("reset clears every scoped choice", () => {
    useShortlistFilterStore
      .getState()
      .set("staff", { saveId: 1, contextToken: "one" }, true);
    useShortlistFilterStore
      .getState()
      .set("staff", { saveId: 2, contextToken: "two" }, false);
    useShortlistFilterStore.getState().reset();
    expect(useShortlistFilterStore.getState().choices).toEqual({});
  });
});
