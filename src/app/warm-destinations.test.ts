import { describe, expect, it, vi } from "vitest";
import { warmDestinations, warmedDestinations } from "./warm-destinations";

const noPause = () => Promise.resolve();

describe("warmDestinations", () => {
  it("preloads every destination in order, one at a time", async () => {
    let running = 0;
    let mostAtOnce = 0;
    const visited: unknown[] = [];
    const preload = vi.fn(async (destination: unknown) => {
      visited.push(destination);
      running += 1;
      mostAtOnce = Math.max(mostAtOnce, running);
      await Promise.resolve();
      running -= 1;
    });

    await warmDestinations(preload, () => false, noPause);

    expect(visited).toEqual([...warmedDestinations]);
    expect(mostAtOnce).toBe(1);
  });

  it("keeps going when one destination's loader fails", async () => {
    const preload = vi
      .fn()
      .mockRejectedValueOnce(new Error("database is busy"))
      .mockResolvedValue(undefined);

    await warmDestinations(preload, () => false, noPause);

    expect(preload).toHaveBeenCalledTimes(warmedDestinations.length);
  });

  it("stops as soon as the snapshot it was warming for is gone", async () => {
    let cancelled = false;
    const preload = vi.fn(async () => {
      cancelled = true;
    });

    await warmDestinations(preload, () => cancelled, noPause);

    expect(preload).toHaveBeenCalledTimes(1);
  });
});
