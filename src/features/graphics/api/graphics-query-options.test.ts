import { describe, expect, it } from "vitest";
import { DEFAULT_GRAPHICS_STATUS } from "./graphics-ipc-mock";
import { graphicsStatusQueryOptions } from "./graphics-query-options";

describe("graphicsStatusQueryOptions", () => {
  it("polls while the background index rebuild is active", () => {
    const refetchInterval = graphicsStatusQueryOptions.refetchInterval;

    expect(refetchInterval).toBeTypeOf("function");
    if (typeof refetchInterval !== "function") return;

    expect(
      refetchInterval({
        state: {
          data: { ...DEFAULT_GRAPHICS_STATUS, rebuilding: true },
        },
      } as never),
    ).toBeGreaterThan(0);
    expect(
      refetchInterval({
        state: {
          data: { ...DEFAULT_GRAPHICS_STATUS, rebuilding: false },
        },
      } as never),
    ).toBe(false);
  });
});
