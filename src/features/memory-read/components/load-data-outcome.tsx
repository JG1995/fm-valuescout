import type { LucideIcon } from "lucide-react";
import {
  CircleAlert,
  CircleCheck,
  LoaderCircle,
  TriangleAlert,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { cn } from "@/utils/cn";
import { formatCount, formatMissable } from "@/utils/format";
import type { LoadDataProgress, LoadDataResult } from "../types/load-data";
import { loadDataErrorCopy } from "./load-data-error";

export const loadDataPhaseLabels: Record<LoadDataProgress["phase"], string> = {
  scan: "Scanning…",
  preparing: "Preparing…",
  scoring: "Scoring…",
  saving: "Saving…",
  finalizing: "Finalizing…",
};

function formatDurationMs(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  }
  return `${(ms / 1000).toFixed(1)}s`;
}

function formatLoadTimings(timings: LoadDataResult["timings"]): string {
  if (timings.totalMs === 0) {
    return "";
  }
  return ` Scan ${formatDurationMs(timings.scanMs)}, preparation ${formatDurationMs(timings.prepareMs)}, scoring ${formatDurationMs(timings.scoringMs)}, save ${formatDurationMs(timings.saveMs)}, finalization ${formatDurationMs(timings.finalizeMs)}, total ${formatDurationMs(timings.totalMs)}.`;
}

type LoadDataOutcomeProps = {
  error: Error | null;
  /** Omitted when the load targeted a save the user has since switched away from. */
  result?: LoadDataResult;
  progress?: LoadDataProgress | null;
  onDismiss: () => void;
};

type Banner = {
  icon: LucideIcon;
  tone: string;
  /** Rendered as a lead-in before the body, for failures that need naming. */
  title?: string;
  body: string;
};

const toneClasses = {
  success: "border-success/40 bg-success-container text-on-success-container",
  warning: "border-warning/40 bg-warning-container text-on-warning-container",
  error: "border-error/40 bg-error-container text-on-error-container",
  pending: "border-info/40 bg-info-container text-on-info-container",
};

function resolveBanner({ error, result }: LoadDataOutcomeProps): Banner | null {
  if (error) {
    const copy = loadDataErrorCopy(error);
    return {
      icon: CircleAlert,
      tone: toneClasses.error,
      title: copy.title,
      body: copy.body,
    };
  }

  if (!result) {
    return null;
  }

  const storedSnapshot = result.storedSnapshot;
  const latestSnapshot = result.effectiveSnapshot;
  const loaded = `Loaded ${formatCount(storedSnapshot.playerCount)} players into the database.${formatLoadTimings(result.timings)}`;
  const storedBecameLatest =
    storedSnapshot.contextToken === latestSnapshot.contextToken;
  const latestMessage = storedBecameLatest
    ? " This snapshot is now the latest."
    : ` Stored this snapshot in history; the latest remains ${formatSnapshotDate(latestSnapshot.gameDate)}.`;
  if (storedSnapshot.scanTruncated !== true) {
    return {
      icon: CircleCheck,
      tone: toneClasses.success,
      body: `${loaded}${latestMessage}`,
    };
  }

  const cap = formatMissable(
    storedSnapshot.maxAccepted === null
      ? null
      : formatCount(storedSnapshot.maxAccepted),
  );
  return {
    icon: TriangleAlert,
    tone: toneClasses.warning,
    body: `${loaded} Partial ingest — the scan was capped at ${cap} players.${latestMessage}`,
  };
}

function formatSnapshotDate(gameDate: string | null): string {
  return gameDate ?? "an undated snapshot";
}

export function LoadDataOutcome(props: LoadDataOutcomeProps) {
  const banner = resolveBanner(props);
  const progress = props.progress ?? null;
  const hasPending = progress !== null;
  const hasBanner = banner !== null;

  const outerClassName = hasPending
    ? cn(
        "flex items-center gap-2 border-t px-4 py-2 text-body-sm",
        toneClasses.pending,
      )
    : hasBanner
      ? cn(
          "flex items-center gap-2 border-t px-4 py-2 text-body-sm",
          banner.tone,
        )
      : undefined;

  const progressNode = hasPending ? (
    progress.completed == null || progress.total == null ? (
      <progress
        aria-label={loadDataPhaseLabels[progress.phase]}
        className="h-2 w-32 shrink-0 accent-primary"
      />
    ) : progress.total === 0 ? null : (
      <progress
        aria-label={`${loadDataPhaseLabels[progress.phase]} ${progress.completed} of ${progress.total}`}
        className="h-2 w-32 shrink-0 accent-primary"
        max={progress.total}
        value={progress.completed}
      />
    )
  ) : null;

  // Keep the live region mounted while idle. Progress is its sibling so the
  // visible phase and identical progress name are not announced together.
  return (
    <div className={outerClassName}>
      <div
        aria-live="polite"
        className={
          hasPending || hasBanner
            ? "flex min-w-0 flex-1 items-center gap-2"
            : undefined
        }
      >
        {hasPending ? (
          <>
            <LoaderCircle
              aria-hidden="true"
              size={16}
              strokeWidth={1.5}
              className="shrink-0"
            />
            <p className="min-w-0 flex-1">
              {loadDataPhaseLabels[progress.phase]}
              {progress.completed != null && progress.total != null
                ? ` ${progress.completed} of ${progress.total}`
                : ""}
            </p>
          </>
        ) : hasBanner ? (
          <>
            <banner.icon
              aria-hidden="true"
              size={16}
              strokeWidth={1.5}
              className="shrink-0"
            />
            <p className="min-w-0 flex-1">
              {banner.title ? (
                <span className="text-label-lg">{banner.title}. </span>
              ) : null}
              {banner.body}
            </p>
          </>
        ) : null}
      </div>
      {hasPending ? progressNode : null}
      {hasBanner ? (
        <Button
          size="icon"
          icon={X}
          variant="ghost"
          aria-label="Dismiss Load Data outcome"
          className="-my-1 shrink-0"
          onClick={props.onDismiss}
        />
      ) : null}
    </div>
  );
}
