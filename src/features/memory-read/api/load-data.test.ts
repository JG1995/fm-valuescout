import { beforeEach, describe, expect, it, vi } from "vitest";
import * as tauriClient from "@/lib/tauri-client";
import { loadData } from "./load-data";

vi.mock("@/lib/tauri-client", () => ({
  invokeCommand: vi.fn(),
}));

describe("loadData API", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("constructs a typed Channel<LoadDataProgress> and forwards ordered progress callback with exact onProgress argument", async () => {
    const invokeMock = vi.mocked(tauriClient.invokeCommand);
    invokeMock.mockImplementation(async (_cmd, args) => {
      // Capture that onProgress is a Channel with onmessage set
      const channel = (args as Record<string, unknown>).onProgress as {
        onmessage?: unknown;
      };
      expect(channel).toBeDefined();
      expect(typeof channel).toBe("object");
      return {} as never;
    });

    const onProgress = vi.fn();

    await loadData(123, 42, "tok-abc", onProgress);

    expect(invokeMock).toHaveBeenCalledWith(
      "load_data",
      expect.objectContaining({
        saveId: 42,
        contextToken: "tok-abc",
        maxAccepted: 123,
        onProgress: expect.objectContaining({
          onmessage: onProgress,
        }),
      }),
    );

    // Verify ordered delivery via the captured channel
    const capturedArgs = invokeMock.mock.calls[0][1] as Record<string, unknown>;
    const capturedChannel = capturedArgs.onProgress as {
      onmessage: (p: unknown) => void;
    };

    const events = [
      { saveId: 1, contextToken: "tok-1", phase: "scan" },
      {
        saveId: 1,
        contextToken: "tok-1",
        phase: "preparing",
        completed: 10,
        total: 10,
      },
      {
        saveId: 1,
        contextToken: "tok-1",
        phase: "scoring",
        completed: 0,
        total: 10,
      },
      {
        saveId: 1,
        contextToken: "tok-1",
        phase: "saving",
        completed: 10,
        total: 10,
      },
      {
        saveId: 1,
        contextToken: "tok-1",
        phase: "finalizing",
        completed: 1,
        total: 1,
      },
    ];

    for (const event of events) {
      capturedChannel.onmessage(event);
    }

    expect(onProgress).toHaveBeenCalledTimes(events.length);
    expect(onProgress).toHaveBeenNthCalledWith(1, events[0]);
    expect(onProgress).toHaveBeenNthCalledWith(5, events[4]);
  });

  it("passes null maxAccepted with channel when cap is off", async () => {
    const invokeMock = vi.mocked(tauriClient.invokeCommand);
    invokeMock.mockResolvedValue({} as never);

    await loadData(null, 1, "save-token-1", vi.fn());

    expect(invokeMock).toHaveBeenCalledWith(
      "load_data",
      expect.objectContaining({
        saveId: 1,
        contextToken: "save-token-1",
        maxAccepted: null,
        onProgress: expect.any(Object),
      }),
    );
  });
});
