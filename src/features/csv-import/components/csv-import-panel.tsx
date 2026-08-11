import { open } from "@tauri-apps/plugin-dialog";
import { CircleAlert, CircleCheck, FileUp, TriangleAlert } from "lucide-react";
import { useLayoutEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import { formatCount } from "@/utils/format";
import { importCsv } from "../api/import-csv";
import type { CsvImportSummary } from "../types/csv-import-summary";
import {
  type CsvImportState,
  importStateForContext,
} from "../utils/import-state";

function formatName(format: CsvImportSummary["format"]) {
  return format === "youthTracker" ? "Youth Tracker" : "Moneyball";
}

const invalidCsvErrorPrefix = "CSV file is invalid: ";

function errorCopy(error: Error) {
  if (
    error.message ===
    "The current save or snapshot changed while the CSV was imported"
  ) {
    return {
      title: "Snapshot changed",
      body: "The active save or snapshot changed while the CSV was imported. Select the CSV again.",
    };
  }

  if (error.message === "Load data before importing a CSV export") {
    return {
      title: "No snapshot loaded",
      body: "Load Data before importing a CSV export.",
    };
  }

  if (error.message === "CSV format is not supported") {
    return {
      title: "CSV format not supported",
      body: "Choose a Youth Tracker or Moneyball CSV export.",
    };
  }

  if (error.message.startsWith(invalidCsvErrorPrefix)) {
    return {
      title: "CSV is invalid",
      body: error.message.slice(invalidCsvErrorPrefix.length),
    };
  }

  return {
    title: "Could not import CSV",
    body: "Choose a supported UTF-8 CSV export and try again.",
  };
}

function ImportOutcome({ state }: { state: CsvImportState }) {
  if (state.status === "idle") {
    return (
      <p className="text-body-md text-on-surface-variant">
        Choose a Youth Tracker or Moneyball export to import. Matching player
        rows replace earlier CSV enrichment; player IDs outside the current
        snapshot are skipped.
      </p>
    );
  }

  if (state.status === "pending") {
    return (
      <div aria-live="polite" role="status" className="flex items-center gap-2">
        <StatusChip icon={FileUp} tone="info">
          Importing CSV
        </StatusChip>
        <span className="text-body-md text-on-surface-variant">
          Importing the selected CSV for the current snapshot.
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

  const { summary } = state;
  const hasSkippedPlayers = summary.skippedPlayers > 0;

  return (
    <div aria-live="polite" role="status" className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <StatusChip icon={CircleCheck} tone="success">
          {`${formatName(summary.format)} imported`}
        </StatusChip>
        <span className="text-body-md text-on-surface">
          {formatCount(summary.storedPlayers)} of{" "}
          {formatCount(summary.totalPlayers)} player IDs were stored.
        </span>
      </div>
      {hasSkippedPlayers ? (
        <p className="flex items-start gap-2 rounded-md border border-warning/40 bg-warning-container px-3 py-2 text-body-sm text-on-warning-container">
          <TriangleAlert
            aria-hidden="true"
            size={16}
            strokeWidth={1.5}
            className="mt-0.5 shrink-0"
          />
          {summary.skippedPlayers === 1
            ? `${formatCount(summary.skippedPlayers)} player ID was skipped because it does not match the current snapshot.`
            : `${formatCount(summary.skippedPlayers)} player IDs were skipped because they do not match the current snapshot.`}
        </p>
      ) : (
        <p className="flex items-center gap-2 text-body-sm text-success">
          <CircleCheck aria-hidden="true" size={16} strokeWidth={1.5} />
          Every exported player ID was stored.
        </p>
      )}
    </div>
  );
}

type CsvImportPanelProps = {
  activeSaveId: number | undefined;
  snapshotId: number | undefined;
  onYouthImported: () => void;
};

export function CsvImportPanel({
  activeSaveId,
  snapshotId,
  onYouthImported,
}: CsvImportPanelProps) {
  const contextKey = `${activeSaveId ?? "none"}:${snapshotId ?? "none"}`;
  const currentContext = useRef(contextKey);
  const contextGeneration = useRef(0);
  const [state, setState] = useState<CsvImportState>({ status: "idle" });
  const visibleState = importStateForContext(state, contextKey);

  useLayoutEffect(() => {
    currentContext.current = contextKey;
    contextGeneration.current += 1;
    setState({ status: "idle" });
  }, [contextKey]);

  const chooseCsv = async () => {
    const selectionContext = contextKey;
    const selectionGeneration = contextGeneration.current;
    try {
      const path = await open({
        multiple: false,
        directory: false,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (
        currentContext.current !== selectionContext ||
        contextGeneration.current !== selectionGeneration ||
        !path
      ) {
        return;
      }

      setState({ status: "pending", contextKey: selectionContext });
      const summary = await importCsv(path);
      if (
        currentContext.current === selectionContext &&
        contextGeneration.current === selectionGeneration
      ) {
        setState({ status: "success", contextKey: selectionContext, summary });
        if (summary.format === "youthTracker") {
          onYouthImported();
        }
      }
    } catch (error) {
      if (
        currentContext.current === selectionContext &&
        contextGeneration.current === selectionGeneration
      ) {
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
      title="CSV enrichment"
      actions={
        <Button
          variant="secondary"
          icon={FileUp}
          loading={visibleState.status === "pending"}
          loadingLabel="Importing CSV…"
          disabled={snapshotId === undefined}
          onClick={() => {
            void chooseCsv();
          }}
        >
          Import CSV
        </Button>
      }
    >
      {snapshotId !== undefined ? (
        <ImportOutcome state={visibleState} />
      ) : (
        <EmptyState icon={FileUp} title="No snapshot loaded">
          Load Data before importing a CSV export.
        </EmptyState>
      )}
    </Panel>
  );
}
