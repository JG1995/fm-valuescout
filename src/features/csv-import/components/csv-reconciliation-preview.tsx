import { open } from "@tauri-apps/plugin-dialog";
import {
  CircleAlert,
  CircleCheck,
  FileSearch,
  TriangleAlert,
} from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import { formatCount } from "@/utils/format";
import { previewCsvMatches } from "../api/preview-csv-matches";
import type { CsvMatchPreview } from "../types/csv-match-preview";
import {
  type CsvPreviewState,
  previewStateForContext,
} from "../utils/preview-state";

function formatName(format: CsvMatchPreview["format"]) {
  return format === "youthTracker" ? "Youth Tracker" : "Moneyball";
}

function errorCopy(error: Error) {
  if (error.message === "The current save changed while the CSV was read") {
    return {
      title: "Snapshot changed",
      body: "The active save or snapshot changed while the CSV was checked. Select the CSV again.",
    };
  }

  if (error.message === "Load data before previewing a CSV export") {
    return {
      title: "No snapshot loaded",
      body: "Load Data before previewing a CSV export.",
    };
  }

  if (error.message === "CSV format is not supported") {
    return {
      title: "CSV format not supported",
      body: "Choose a Youth Tracker or Moneyball CSV export.",
    };
  }

  return {
    title: "Could not preview CSV",
    body: "Choose a supported UTF-8 CSV export and try again.",
  };
}

function PreviewOutcome({ state }: { state: CsvPreviewState }) {
  if (state.status === "idle") {
    return (
      <p className="text-body-md text-on-surface-variant">
        Choose one Youth Tracker or Moneyball export to compare its player IDs
        with the current snapshot. The CSV is not imported or saved.
      </p>
    );
  }

  if (state.status === "pending") {
    return (
      <div aria-live="polite" role="status" className="flex items-center gap-2">
        <StatusChip icon={FileSearch} tone="info">
          Checking CSV
        </StatusChip>
        <span className="text-body-md text-on-surface-variant">
          Checking the selected CSV against the current snapshot.
        </span>
      </div>
    );
  }

  if (state.status === "error") {
    const copy = errorCopy(state.error);

    return (
      <div
        role="alert"
        className="flex items-start gap-2 rounded-md border border-error/40 bg-error-container px-3 py-2 text-body-sm text-on-error-container"
      >
        <CircleAlert
          aria-hidden="true"
          size={16}
          strokeWidth={1.5}
          className="mt-0.5 shrink-0"
        />
        <p>
          <span className="text-label-lg">{copy.title}. </span>
          {copy.body}
        </p>
      </div>
    );
  }

  const { preview } = state;
  const hasUnmatchedPlayers = preview.unmatchedPlayers > 0;

  return (
    <div aria-live="polite" role="status" className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <StatusChip icon={FileSearch} tone="info">
          {`${formatName(preview.format)} detected`}
        </StatusChip>
        <span className="text-body-md text-on-surface">
          {formatCount(preview.matchedPlayers)} of{" "}
          {formatCount(preview.totalPlayers)} player IDs match the current
          snapshot.
        </span>
      </div>
      {hasUnmatchedPlayers ? (
        <p className="flex items-start gap-2 rounded-md border border-warning/40 bg-warning-container px-3 py-2 text-body-sm text-on-warning-container">
          <TriangleAlert
            aria-hidden="true"
            size={16}
            strokeWidth={1.5}
            className="mt-0.5 shrink-0"
          />
          {formatCount(preview.unmatchedPlayers)} exported player ID
          {preview.unmatchedPlayers === 1 ? " does" : "s do"} not match the
          current snapshot.
        </p>
      ) : (
        <p className="flex items-center gap-2 text-body-sm text-success">
          <CircleCheck aria-hidden="true" size={16} strokeWidth={1.5} />
          Every exported player ID matches the current snapshot.
        </p>
      )}
    </div>
  );
}

type CsvReconciliationPreviewProps = {
  activeSaveId: number | undefined;
  snapshotId: number | undefined;
};

export function CsvReconciliationPreview({
  activeSaveId,
  snapshotId,
}: CsvReconciliationPreviewProps) {
  const contextKey = `${activeSaveId ?? "none"}:${snapshotId ?? "none"}`;
  const currentContext = useRef(contextKey);
  const [state, setState] = useState<CsvPreviewState>({ status: "idle" });
  const visibleState = previewStateForContext(state, contextKey);

  useLayoutEffect(() => {
    currentContext.current = contextKey;
    setState({ status: "idle" });
  }, [contextKey]);

  const chooseCsv = async () => {
    const selectionContext = contextKey;
    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (currentContext.current !== selectionContext || !path) {
        return;
      }

      setState({ status: "pending", contextKey: selectionContext });
      const preview = await previewCsvMatches(path);
      if (currentContext.current === selectionContext) {
        setState({ status: "success", contextKey: selectionContext, preview });
      }
    } catch (error) {
      if (currentContext.current === selectionContext) {
        setState({
          status: "error",
          contextKey: selectionContext,
          error: error instanceof Error ? error : new Error(String(error)),
        });
      }
    }
  };

  return (
    <Panel
      title="CSV reconciliation"
      actions={
        <Button
          variant="secondary"
          icon={FileSearch}
          loading={visibleState.status === "pending"}
          loadingLabel="Checking CSV…"
          disabled={snapshotId === undefined}
          onClick={() => {
            void chooseCsv();
          }}
        >
          Choose CSV
        </Button>
      }
    >
      {snapshotId !== undefined ? (
        <PreviewOutcome state={visibleState} />
      ) : (
        <EmptyState icon={FileSearch} title="No snapshot loaded">
          Load Data before previewing a CSV export.
        </EmptyState>
      )}
    </Panel>
  );
}
