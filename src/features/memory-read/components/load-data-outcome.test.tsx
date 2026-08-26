import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { LoadDataResult } from "../types/load-data";
import { LoadDataOutcome } from "./load-data-outcome";

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
    timings: { scanMs: 1200, ingestMs: 400, totalMs: 1600 },
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
});
