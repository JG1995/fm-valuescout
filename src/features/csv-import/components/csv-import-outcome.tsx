import { CircleAlert, CircleCheck, FileUp, TriangleAlert } from "lucide-react";
import { StatusChip } from "@/components/ui/status-chip/status-chip";
import { formatCount } from "@/utils/format";
import type { CsvImportFormat } from "../types/csv-import-summary";
import type { CsvImportState } from "../utils/import-state";

type CsvImportOutcomeProps = {
  state: CsvImportState;
  expectedFormat?: CsvImportFormat;
  youthLabel?: "Youth Academy" | "Youth Tracker";
};

const invalidCsvErrorPrefix = "CSV file is invalid: ";

function formatName(
  format: CsvImportFormat,
  youthLabel: "Youth Academy" | "Youth Tracker",
) {
  return format === "youthTracker" ? youthLabel : "Moneyball";
}

function errorCopy(error: Error, expectedFormat?: CsvImportFormat) {
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

  if (error.message === "CSV does not match the selected upload format") {
    return {
      title: "CSV format does not match",
      body:
        expectedFormat === "moneyball"
          ? "Choose a Moneyball CSV export."
          : "Choose a Youth Academy CSV export.",
    };
  }

  if (error.message === "Drop one CSV file at a time") {
    return {
      title: "Choose one CSV file",
      body: "Drop one CSV file at a time.",
    };
  }

  if (error.message === "CSV format is not supported") {
    return {
      title: "CSV format not supported",
      body:
        expectedFormat === "moneyball"
          ? "Choose a Moneyball CSV export."
          : expectedFormat === "youthTracker"
            ? "Choose a Youth Academy CSV export."
            : "Choose a Youth Tracker or Moneyball CSV export.",
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

export function CsvImportOutcome({
  state,
  expectedFormat,
  youthLabel = "Youth Tracker",
}: CsvImportOutcomeProps) {
  if (state.status === "idle") {
    const prompt = expectedFormat
      ? `Choose a ${formatName(expectedFormat, youthLabel)} CSV export to import.`
      : "Choose a Youth Tracker or Moneyball export to import.";

    return (
      <p className="text-body-md text-on-surface-variant">
        {prompt} Matching player rows replace earlier CSV enrichment; player IDs
        outside the current snapshot are skipped.
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
    const copy = errorCopy(state.error, expectedFormat);

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
          {`${formatName(summary.format, youthLabel)} imported`}
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
