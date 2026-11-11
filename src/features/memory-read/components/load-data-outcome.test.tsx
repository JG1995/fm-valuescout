import { render, screen, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type {
  LoadDataProgress,
  LoadDataResult,
  PlayerShortlistsOutcome,
  SeasonStatsOutcome,
} from "../types/load-data";
import { LoadDataOutcome, loadDataPhaseLabels } from "./load-data-outcome";

function resultWithHistoricalStoredSnapshot(): LoadDataResult {
  return {
    requestId: "R1",
    playersFound: 1,
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
      statsMs: 100,
      totalMs: 2200,
      ingestMs: 400,
    },
    stats: {
      status: "notLinked",
      shortlists: { status: "notRead" },
      elapsedMs: 100,
    },
    shortlists: { read: 0, failed: [] },
  };
}

function successResult(): LoadDataResult {
  return {
    requestId: "req-1",
    playersFound: 3,
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
      statsMs: 100,
      totalMs: 2200,
      ingestMs: 400,
    },
    stats: {
      status: "notLinked",
      shortlists: { status: "notRead" },
      elapsedMs: 100,
    },
    shortlists: { read: 0, failed: [] },
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

    // scan 1.2s, preparation 300ms, scoring 400ms, save 200ms, finalization 200ms, season stats 100ms, total 2.2s
    expect(screen.getByText(/Scan 1\.2s/i)).toBeInTheDocument();
    expect(screen.getByText(/preparation 300ms/i)).toBeInTheDocument();
    expect(screen.getByText(/scoring 400ms/i)).toBeInTheDocument();
    expect(screen.getByText(/save 200ms/i)).toBeInTheDocument();
    expect(screen.getByText(/finalization 200ms/i)).toBeInTheDocument();
    expect(screen.getByText(/season stats 100ms/i)).toBeInTheDocument();
    expect(screen.getByText(/total 2\.2s/i)).toBeInTheDocument();
    // ingestMs is aggregate, must not appear as primary bucket
    expect(screen.queryByText(/ingest 400ms/i)).toBeNull();
    expect(screen.queryByText(/%/)).toBeNull();
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
        statsMs: 0,
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
    expect(loadDataPhaseLabels.seasonStats).toBe("Reading stats…");
  });

  it.each<[string, SeasonStatsOutcome, string]>([
    [
      "attached stats with the save's date",
      {
        status: "attached",
        matchedPlayers: 35903,
        unmatchedPlayers: 12,
        gameDate: "2027-01-01",
        shortlists: { status: "notRead" },
        elapsedMs: 1300,
      },
      "Attached season stats for 35,903 players from the save dated 1st January 2027.",
    ],
    [
      "link guidance",
      { status: "notLinked", shortlists: { status: "notRead" }, elapsedMs: 0 },
      "No FM save file is linked, so no season stats were read. Link one in Settings.",
    ],
    [
      "a read that matched no player",
      {
        status: "noMatches",
        unmatchedPlayers: 4,
        gameDate: "2026-07-01",
        shortlists: { status: "notRead" },
        elapsedMs: 900,
      },
      "No players in the save dated 1st July 2026 matched the current snapshot, so season stats were not changed.",
    ],
    [
      "a discarded stale read",
      {
        status: "staleContext",
        shortlists: { status: "notRead" },
        elapsedMs: 10,
      },
      "The save or snapshot changed while season stats were read, so they were not attached.",
    ],
    [
      "a missing file by name",
      {
        status: "failed",
        kind: "not_found",
        fileName: "Career.fm",
        shortlists: { status: "notRead" },
        elapsedMs: 1,
      },
      "Season stats were not read: Career.fm could not be found.",
    ],
    [
      "a file that is not an FM27 save",
      {
        status: "failed",
        kind: "not_fm_save",
        fileName: "Notes.fm",
        shortlists: { status: "notRead" },
        elapsedMs: 1,
      },
      "Season stats were not read: Notes.fm is not a supported FM27 save.",
    ],
    [
      "an unsupported build",
      {
        status: "failed",
        kind: "unsupported_build",
        fileName: "Career.fm",
        shortlists: { status: "notRead" },
        elapsedMs: 1,
      },
      "Season stats were not read: this FM27 build is not supported yet.",
    ],
    [
      "a damaged save",
      {
        status: "failed",
        kind: "corrupt",
        fileName: "Career.fm",
        shortlists: { status: "notRead" },
        elapsedMs: 1,
      },
      "Season stats were not read: the save could not be read safely.",
    ],
    [
      "a save FM was still writing",
      {
        status: "failed",
        kind: "changed_during_read",
        fileName: "Career.fm",
        shortlists: { status: "notRead" },
        elapsedMs: 1,
      },
      "Season stats were not read: the save changed while it was read. Wait for FM to finish saving, then try again.",
    ],
    [
      "a failed write",
      {
        status: "failed",
        kind: "database",
        fileName: null,
        shortlists: { status: "notRead" },
        elapsedMs: 1,
      },
      "Season stats could not be stored, so the previous stats are unchanged.",
    ],
  ])("reports %s after the snapshot outcome", (_case, stats, expected) => {
    render(
      <LoadDataOutcome
        error={null}
        result={{ ...successResult(), stats }}
        onDismiss={() => undefined}
      />,
    );

    const outcome = screen.getByText(/Loaded 3 players into the database/i);
    expect(outcome).toHaveTextContent(
      `This snapshot is now the latest. ${expected}`,
    );
  });

  const attached = {
    status: "attached",
    matchedPlayers: 2,
    unmatchedPlayers: 0,
    gameDate: "2027-01-01",
    elapsedMs: 1,
  } as const;
  const attachedSentence =
    "Attached season stats for 2 players from the save dated 1st January 2027.";

  it.each<[string, SeasonStatsOutcome["shortlists"], string, boolean]>([
    [
      "several shortlists read from the save",
      { status: "read", count: 5 },
      `${attachedSentence} Read 5 shortlists from the save.`,
      false,
    ],
    [
      "one shortlist read from the save",
      { status: "read", count: 1 },
      `${attachedSentence} Read 1 shortlist from the save.`,
      false,
    ],
    [
      "shortlists an FM build stores differently",
      { status: "failed", kind: "unsupported_build" },
      `${attachedSentence} The save's shortlists were not read: this FM27 build is not supported yet. The previous ones are kept.`,
      true,
    ],
    [
      "shortlists that could not be stored",
      { status: "failed", kind: "database" },
      `${attachedSentence} The save's shortlists could not be stored, so the previous ones are kept.`,
      true,
    ],
  ])(
    "reports %s after the season stats",
    (_case, shortlists, expected, isProblem) => {
      render(
        <LoadDataOutcome
          error={null}
          result={{
            ...successResult(),
            stats: { ...attached, shortlists },
            shortlists: { read: 0, failed: [] },
          }}
          onDismiss={() => undefined}
        />,
      );

      const outcome = screen.getByText(/Loaded 3 players into the database/i);
      expect(outcome).toHaveTextContent(expected);
      expect(outcome.parentElement?.parentElement).toHaveClass(
        isProblem ? "bg-warning-container" : "bg-success-container",
      );
    },
  );

  it("says nothing about shortlists when the save's hold no players", () => {
    render(
      <LoadDataOutcome
        error={null}
        result={{
          ...successResult(),
          stats: { ...attached, shortlists: { status: "read", count: 0 } },
          shortlists: { read: 0, failed: [] },
        }}
        onDismiss={() => undefined}
      />,
    );

    const outcome = screen.getByText(/Loaded 3 players into the database/i);
    expect(outcome).toHaveTextContent(attachedSentence);
    expect(outcome).not.toHaveTextContent("shortlist");
  });

  const statsSentence =
    "No FM save file is linked, so no season stats were read. Link one in Settings.";

  it.each<[string, PlayerShortlistsOutcome, string, boolean]>([
    [
      "one re-read shortlist",
      { read: 1, failed: [] },
      "Read 1 shortlist file.",
      false,
    ],
    [
      "several re-read shortlists",
      { read: 3, failed: [] },
      "Read 3 shortlist files.",
      false,
    ],
    [
      "each failed file by name with its reason",
      {
        read: 1,
        failed: [
          { kind: "unreadable", fileName: "Targets.fmf" },
          { kind: "not_player_shortlist", fileName: "View.fmf" },
          { kind: "database", fileName: "Loans.fmf" },
        ],
      },
      "Read 1 shortlist file. Targets.fmf could not be found or opened, so its previous players are kept. View.fmf is not a player shortlist, so its previous players are kept. Loans.fmf could not be stored, so its previous players are kept.",
      true,
    ],
    [
      "shortlists that could not be updated at all",
      { read: 0, failed: [{ kind: "database", fileName: null }] },
      "Read 0 shortlist files. The shortlist files could not be updated, so their previous players are kept.",
      true,
    ],
  ])(
    "reports %s after the season stats",
    (_case, shortlists, expected, isProblem) => {
      render(
        <LoadDataOutcome
          error={null}
          result={{ ...successResult(), shortlists }}
          onDismiss={() => undefined}
        />,
      );

      const outcome = screen.getByText(/Loaded 3 players into the database/i);
      expect(outcome).toHaveTextContent(`${statsSentence} ${expected}`);
      expect(outcome.parentElement?.parentElement).toHaveClass(
        isProblem ? "bg-warning-container" : "bg-success-container",
      );
    },
  );

  it("reports nothing about shortlists when the save has none", () => {
    render(
      <LoadDataOutcome
        error={null}
        result={successResult()}
        onDismiss={() => undefined}
      />,
    );

    expect(
      screen.getByText(/Loaded 3 players into the database/i).textContent,
    ).toMatch(new RegExp(`${statsSentence}$`));
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
