import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { LoadDataProgress, LoadDataResult } from "../types/load-data";
import { LoadDataOutcome, loadDataPhaseLabels } from "./load-data-outcome";

function resultWithHistoricalStoredSnapshot(
  scanTruncated = false,
): LoadDataResult {
  return {
    requestId: "R1",
    playersFound: 1,
    scanTruncated,
    maxAccepted: scanTruncated ? 1 : null,
    storedSnapshot: {
      id: 1,
      contextToken: "snapshot-token-1",
      saveId: 1,
      schemaVersion: 6,
      generatedAtUtc: "2026-08-14T12:00:00.000Z",
      gameVersion: "26.0.0",
      supportedGameVersion: "26.0.0",
      bridgeVersion: "0.1.0",
      protocolVersion: 1,
      gameDate: "2026-08-14",
      gameDateSource: "memory",
      scanTruncated,
      maxAccepted: scanTruncated ? 1 : null,
      playerCount: 1,
      loadedAtUtc: "2026-08-14T12:00:00.000Z",
    },
    effectiveSnapshot: {
      id: 2,
      contextToken: "snapshot-token-2",
      saveId: 1,
      schemaVersion: 6,
      generatedAtUtc: "2027-08-16T12:00:00.000Z",
      gameVersion: "26.0.0",
      supportedGameVersion: "26.0.0",
      bridgeVersion: "0.1.0",
      protocolVersion: 1,
      gameDate: "2027-08-16",
      gameDateSource: "memory",
      scanTruncated: false,
      maxAccepted: null,
      playerCount: 1,
      loadedAtUtc: "2027-08-16T12:00:00.000Z",
    },
    timings: {
      scanMs: 1200,
      prepareMs: 300,
      scoringMs: 400,
      saveMs: 200,
      finalizeMs: 200,
      totalMs: 2100,
      ingestMs: 400,
    },
  };
}

function successResult(): LoadDataResult {
  return {
    requestId: "req-1",
    playersFound: 3,
    scanTruncated: false,
    maxAccepted: null,
    storedSnapshot: {
      id: 1,
      contextToken: "snapshot-token-1",
      saveId: 1,
      schemaVersion: 6,
      generatedAtUtc: "2026-08-14T12:00:00.000Z",
      gameVersion: "26.0.0",
      supportedGameVersion: "26.0.0",
      bridgeVersion: "0.1.0",
      protocolVersion: 1,
      gameDate: "2026-08-14",
      gameDateSource: "memory",
      scanTruncated: false,
      maxAccepted: null,
      playerCount: 3,
      loadedAtUtc: "2026-08-14T12:05:00.000Z",
    },
    effectiveSnapshot: {
      id: 1,
      contextToken: "snapshot-token-1",
      saveId: 1,
      schemaVersion: 6,
      generatedAtUtc: "2026-08-14T12:00:00.000Z",
      gameVersion: "26.0.0",
      supportedGameVersion: "26.0.0",
      bridgeVersion: "0.1.0",
      protocolVersion: 1,
      gameDate: "2026-08-14",
      gameDateSource: "memory",
      scanTruncated: false,
      maxAccepted: null,
      playerCount: 3,
      loadedAtUtc: "2026-08-14T12:05:00.000Z",
    },
    timings: {
      scanMs: 1200,
      prepareMs: 300,
      scoringMs: 400,
      saveMs: 200,
      finalizeMs: 200,
      totalMs: 2100,
      ingestMs: 400,
    },
  };
}

describe("LoadDataOutcome", () => {
  it("distinguishes an earlier stored snapshot from the latest data", () => {
    render(
      <LoadDataOutcome
        error={null}
        result={resultWithHistoricalStoredSnapshot()}
        onDismiss={() => undefined}
      />,
    );

    expect(
      screen.getByText(
        /Stored this snapshot in history; the latest remains 2027-08-16\./i,
      ),
    ).toBeInTheDocument();
    expect(screen.queryByText(/This snapshot is now the latest/i)).toBeNull();
  });

  it("treats an equal numeric ID with a replacement token as historical", () => {
    const result = resultWithHistoricalStoredSnapshot();
    result.effectiveSnapshot = {
      ...result.effectiveSnapshot,
      id: result.storedSnapshot.id,
      contextToken: "snapshot-token-1-replacement",
    };
    render(
      <LoadDataOutcome
        error={null}
        result={result}
        onDismiss={() => undefined}
      />,
    );

    expect(
      screen.getByText(/Stored this snapshot in history/i),
    ).toBeInTheDocument();
    expect(screen.queryByText(/This snapshot is now the latest/i)).toBeNull();
  });

  it("keeps the history message on a truncated earlier load", () => {
    render(
      <LoadDataOutcome
        error={null}
        result={resultWithHistoricalStoredSnapshot(true)}
        onDismiss={() => undefined}
      />,
    );

    expect(screen.getByText(/Partial ingest/i)).toBeInTheDocument();
    expect(
      screen.getByText(
        /Stored this snapshot in history; the latest remains 2027-08-16\./i,
      ),
    ).toBeInTheDocument();
  });

  it("exposes a stable polite live region while idle", () => {
    const { container } = render(
      <LoadDataOutcome error={null} onDismiss={() => undefined} />,
    );

    const liveRegion = container.querySelector('[aria-live="polite"]');
    expect(liveRegion).toBeInTheDocument();
    expect(liveRegion?.textContent?.trim()).toBe("");
    expect(screen.queryByRole("progressbar")).toBeNull();
    expect(screen.queryByText(/Loaded/i)).toBeNull();
  });

  it("renders scan as visible text plus indeterminate native progress", () => {
    const progress: LoadDataProgress = {
      saveId: 1,
      contextToken: "save-token-1",
      phase: "scan",
    };
    render(
      <LoadDataOutcome
        error={null}
        progress={progress}
        onDismiss={() => undefined}
      />,
    );

    expect(screen.getByText("Scanning…")).toBeInTheDocument();
    const bar = screen.getByRole("progressbar", { name: "Scanning…" });
    expect(bar).toBeInTheDocument();
    expect(bar).not.toHaveAttribute("value");
    expect(bar).not.toHaveAttribute("max");
    // No invented percent
    expect(screen.queryByText(/%/)).toBeNull();
  });

  it.each([
    ["preparing", "Preparing…"],
    ["scoring", "Scoring…"],
    ["saving", "Saving…"],
    ["finalizing", "Finalizing…"],
  ] as const)(
    "renders %s with determinate progress and accessible name",
    (phase, label) => {
      const progress = {
        saveId: 1,
        contextToken: "save-token-1",
        phase,
        completed: 5,
        total: 10,
      } as LoadDataProgress;
      const { container } = render(
        <LoadDataOutcome
          error={null}
          progress={progress}
          onDismiss={() => undefined}
        />,
      );

      const liveRegion = container.querySelector(
        '[aria-live="polite"]',
      ) as HTMLElement;
      expect(
        within(liveRegion).getByText(`${label} 5 of 10`),
      ).toBeInTheDocument();
      const bar = screen.getByRole("progressbar", {
        name: `${label} 5 of 10`,
      });
      expect(bar).toHaveAttribute("value", "5");
      expect(bar).toHaveAttribute("max", "10");
      expect(bar.closest("[aria-live]")).toBeNull();
      expect(screen.queryByText(/%/)).toBeNull();
    },
  );

  it("renders determinate 0/total start and total/total completion", () => {
    const start: LoadDataProgress = {
      saveId: 1,
      contextToken: "save-token-1",
      phase: "scoring",
      completed: 0,
      total: 10,
    };
    const { rerender } = render(
      <LoadDataOutcome
        error={null}
        progress={start}
        onDismiss={() => undefined}
      />,
    );
    expect(
      screen.getByRole("progressbar", { name: "Scoring… 0 of 10" }),
    ).toHaveAttribute("value", "0");

    const complete: LoadDataProgress = {
      saveId: 1,
      contextToken: "save-token-1",
      phase: "scoring",
      completed: 10,
      total: 10,
    };
    rerender(
      <LoadDataOutcome
        error={null}
        progress={complete}
        onDismiss={() => undefined}
      />,
    );
    expect(
      screen.getByRole("progressbar", { name: "Scoring… 10 of 10" }),
    ).toHaveAttribute("value", "10");
  });

  it("handles total 0 without percent invention", () => {
    const progress = {
      saveId: 1,
      contextToken: "save-token-1",
      phase: "preparing",
      completed: 0,
      total: 0,
    } as LoadDataProgress;
    const { container } = render(
      <LoadDataOutcome
        error={null}
        progress={progress}
        onDismiss={() => undefined}
      />,
    );

    const liveRegion = container.querySelector(
      '[aria-live="polite"]',
    ) as HTMLElement;
    expect(
      within(liveRegion).getByText("Preparing… 0 of 0"),
    ).toBeInTheDocument();
    // total 0 omits progress element to avoid invalid max, but count copy remains
    expect(screen.queryByRole("progressbar")).toBeNull();
    expect(screen.queryByText(/%/)).toBeNull();
  });

  it("replaces rather than stacks ordered phase updates", () => {
    const scan: LoadDataProgress = {
      saveId: 1,
      contextToken: "save-token-1",
      phase: "scan",
    };
    const { rerender } = render(
      <LoadDataOutcome
        error={null}
        progress={scan}
        onDismiss={() => undefined}
      />,
    );
    expect(screen.getByText("Scanning…")).toBeInTheDocument();

    const preparing: LoadDataProgress = {
      saveId: 1,
      contextToken: "save-token-1",
      phase: "preparing",
      completed: 10,
      total: 10,
    };
    rerender(
      <LoadDataOutcome
        error={null}
        progress={preparing}
        onDismiss={() => undefined}
      />,
    );
    expect(screen.queryByText("Scanning…")).toBeNull();
    expect(screen.getByText("Preparing… 10 of 10")).toBeInTheDocument();
    expect(
      screen.getByRole("progressbar", { name: "Preparing… 10 of 10" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("progressbar")).toBeInTheDocument();
    // only one progressbar exists
    expect(screen.getAllByRole("progressbar")).toHaveLength(1);
  });

  it("replaces pending with success and preserves dismiss", async () => {
    const progress: LoadDataProgress = {
      saveId: 1,
      contextToken: "save-token-1",
      phase: "scan",
    };
    const onDismiss = () => undefined;
    const { rerender } = render(
      <LoadDataOutcome
        error={null}
        progress={progress}
        onDismiss={onDismiss}
      />,
    );
    expect(screen.getByText("Scanning…")).toBeInTheDocument();

    rerender(
      <LoadDataOutcome
        error={null}
        result={successResult()}
        onDismiss={onDismiss}
      />,
    );
    expect(screen.queryByText("Scanning…")).toBeNull();
    expect(screen.queryByRole("progressbar")).toBeNull();
    expect(
      screen.getByText(/Loaded 3 players into the database/i),
    ).toBeInTheDocument();

    // dismiss button still visible for success
    expect(
      screen.getByRole("button", { name: "Dismiss Load Data outcome" }),
    ).toBeInTheDocument();
  });

  it("replaces pending with error and preserves dismiss", () => {
    const progress: LoadDataProgress = {
      saveId: 1,
      contextToken: "save-token-1",
      phase: "saving",
      completed: 0,
      total: 2,
    };
    const { rerender } = render(
      <LoadDataOutcome
        error={null}
        progress={progress}
        onDismiss={() => undefined}
      />,
    );
    expect(screen.getByText("Saving… 0 of 2")).toBeInTheDocument();

    const error = new Error("dump validation failed");
    rerender(<LoadDataOutcome error={error} onDismiss={() => undefined} />);
    expect(screen.queryByText("Saving… 0 of 2")).toBeNull();
    expect(screen.queryByRole("progressbar")).toBeNull();
    expect(screen.getByText(/dump validation failed/i)).toBeInTheDocument();
  });

  it("reports every disjoint timing bucket with human readable units", () => {
    render(
      <LoadDataOutcome
        error={null}
        result={successResult()}
        onDismiss={() => undefined}
      />,
    );

    // scan 1.2s, preparation 300ms, scoring 400ms, save 200ms, finalization 200ms, total 2.1s
    expect(screen.getByText(/Scan 1\.2s/i)).toBeInTheDocument();
    expect(screen.getByText(/preparation 300ms/i)).toBeInTheDocument();
    expect(screen.getByText(/scoring 400ms/i)).toBeInTheDocument();
    expect(screen.getByText(/save 200ms/i)).toBeInTheDocument();
    expect(screen.getByText(/finalization 200ms/i)).toBeInTheDocument();
    expect(screen.getByText(/total 2\.1s/i)).toBeInTheDocument();
    // ingestMs is aggregate, must not appear as primary bucket
    expect(screen.queryByText(/ingest 400ms/i)).toBeNull();
    expect(screen.queryByText(/%/)).toBeNull();
  });

  it("preserves truncated copy alongside detailed timings", () => {
    render(
      <LoadDataOutcome
        error={null}
        result={resultWithHistoricalStoredSnapshot(true)}
        onDismiss={() => undefined}
      />,
    );

    expect(screen.getByText(/Partial ingest/i)).toBeInTheDocument();
    expect(screen.getByText(/Scan 1\.2s/i)).toBeInTheDocument();
  });

  it("preserves historical copy alongside detailed timings", () => {
    render(
      <LoadDataOutcome
        error={null}
        result={resultWithHistoricalStoredSnapshot()}
        onDismiss={() => undefined}
      />,
    );

    expect(
      screen.getByText(/Stored this snapshot in history/i),
    ).toBeInTheDocument();
    expect(screen.getByText(/Scan 1\.2s/i)).toBeInTheDocument();
  });

  it("keeps zero-total timing empty gracefully", () => {
    const zero: LoadDataResult = {
      ...successResult(),
      timings: {
        scanMs: 0,
        prepareMs: 0,
        scoringMs: 0,
        saveMs: 0,
        finalizeMs: 0,
        totalMs: 0,
        ingestMs: 0,
      },
    };
    render(
      <LoadDataOutcome
        error={null}
        result={zero}
        onDismiss={() => undefined}
      />,
    );

    expect(
      screen.getByText(/Loaded 3 players into the database\./i),
    ).toBeInTheDocument();
    expect(screen.queryByText(/Scan/)).toBeNull();
    expect(screen.queryByText(/total/)).toBeNull();
  });

  it("exposes phase labels map for button parity", () => {
    expect(loadDataPhaseLabels.scan).toBe("Scanning…");
    expect(loadDataPhaseLabels.preparing).toBe("Preparing…");
    expect(loadDataPhaseLabels.scoring).toBe("Scoring…");
    expect(loadDataPhaseLabels.saving).toBe("Saving…");
    expect(loadDataPhaseLabels.finalizing).toBe("Finalizing…");
  });

  it("keeps progressbar outside the polite live region while phase text stays inside", () => {
    const progress: LoadDataProgress = {
      saveId: 1,
      contextToken: "save-token-1",
      phase: "scoring",
      completed: 3,
      total: 10,
    };
    const { container } = render(
      <LoadDataOutcome
        error={null}
        progress={progress}
        onDismiss={() => undefined}
      />,
    );

    const liveRegion = container.querySelector(
      '[aria-live="polite"]',
    ) as HTMLElement;
    expect(liveRegion).toBeInTheDocument();
    // phase/count is inside the stable live region
    expect(
      within(liveRegion).getByText("Scoring… 3 of 10"),
    ).toBeInTheDocument();
    const bar = screen.getByRole("progressbar", { name: "Scoring… 3 of 10" });
    // accessible name remains queryable but no aria-live ancestor to double-announce
    expect(bar.closest("[aria-live]")).toBeNull();
    // indeterminate variant likewise
    const { container: c2 } = render(
      <LoadDataOutcome
        error={null}
        progress={{ saveId: 1, contextToken: "save-token-1", phase: "scan" }}
        onDismiss={() => undefined}
      />,
    );
    const live2 = c2.querySelector('[aria-live="polite"]') as HTMLElement;
    expect(within(live2).getByText("Scanning…")).toBeInTheDocument();
    const scanBarByLabel = within(c2).getByRole("progressbar", {
      name: "Scanning…",
    });
    expect(scanBarByLabel.closest("[aria-live]")).toBeNull();
  });

  it("does not invent overall percent anywhere", () => {
    const progress: LoadDataProgress = {
      saveId: 1,
      contextToken: "save-token-1",
      phase: "preparing",
      completed: 5,
      total: 10,
    };
    const { rerender } = render(
      <LoadDataOutcome
        error={null}
        progress={progress}
        onDismiss={() => undefined}
      />,
    );
    expect(screen.queryByText(/%/)).toBeNull();
    rerender(
      <LoadDataOutcome
        error={null}
        result={successResult()}
        onDismiss={() => undefined}
      />,
    );
    expect(screen.queryByText(/%/)).toBeNull();
    // no width percent style either
    const bar = document.querySelector("progress");
    expect(bar).toBeNull();
  });
});
