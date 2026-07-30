import type { LucideIcon } from "lucide-react";
import { CircleAlert, CircleCheck, TriangleAlert } from "lucide-react";
import { cn } from "@/utils/cn";
import { formatCount, formatMissable } from "@/utils/format";
import type { LoadDataResult } from "../types/load-data";
import { loadDataErrorCopy } from "./load-data-error";

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
  return ` Scan ${formatDurationMs(timings.scanMs)}, ingest ${formatDurationMs(timings.ingestMs)}, total ${formatDurationMs(timings.totalMs)}.`;
}

type LoadDataOutcomeProps = {
  error: Error | null;
  /** Omitted when the load targeted a save the user has since switched away from. */
  result?: LoadDataResult;
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

  const loaded = `Loaded ${formatCount(result.snapshot.playerCount)} players into the database.${formatLoadTimings(result.timings)}`;
  if (result.snapshot.scanTruncated !== true) {
    return { icon: CircleCheck, tone: toneClasses.success, body: loaded };
  }

  const cap = formatMissable(
    result.snapshot.maxAccepted === null
      ? null
      : formatCount(result.snapshot.maxAccepted),
  );
  return {
    icon: TriangleAlert,
    tone: toneClasses.warning,
    body: `${loaded} Partial ingest — the scan was capped at ${cap} players.`,
  };
}

export function LoadDataOutcome(props: LoadDataOutcomeProps) {
  const banner = resolveBanner(props);

  // The region is always in the DOM, empty when idle: a live region created at
  // the same moment as its text is usually missed by screen readers.
  return (
    <div aria-live="polite">
      {banner ? (
        <div
          className={cn(
            "flex items-start gap-2 border-t px-4 py-2 text-body-sm",
            banner.tone,
          )}
        >
          <banner.icon
            aria-hidden="true"
            size={16}
            strokeWidth={1.5}
            className="mt-0.5 shrink-0"
          />
          <p>
            {banner.title ? (
              <span className="text-label-lg">{banner.title}. </span>
            ) : null}
            {banner.body}
          </p>
        </div>
      ) : null}
    </div>
  );
}
