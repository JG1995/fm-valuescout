import type { LucideIcon } from "lucide-react";
import { CircleAlert, CircleCheck, LoaderCircle, X } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { cn } from "@/utils/cn";
import { formatCount, formatLongGameDate } from "@/utils/format";
import type {
  LoadDataProgress,
  LoadDataResult,
  PlayerShortlistReadErrorKind,
  PlayerShortlistsOutcome,
  SaveShortlistsOutcome,
  SeasonStatsErrorKind,
  SeasonStatsOutcome,
} from "../types/load-data";
import { loadDataErrorCopy } from "./load-data-error";

export const loadDataPhaseLabels: Record<LoadDataProgress["phase"], string> = {
  scan: "Scanning…",
  preparing: "Preparing…",
  scoring: "Scoring…",
  saving: "Saving…",
  finalizing: "Finalizing…",
  seasonStats: "Reading stats…",
};

const seasonStatsReadFailureReasons: Record<
  Exclude<SeasonStatsErrorKind, "database">,
  (fileName: string) => string
> = {
  not_found: (fileName) => `${fileName} could not be found`,
  unreadable: (fileName) => `${fileName} could not be opened`,
  not_fm_save: (fileName) => `${fileName} is not a supported FM27 save`,
  unsupported_game: (fileName) => `${fileName} is not a supported FM27 save`,
  unsupported_build: () => "this FM27 build is not supported yet",
  corrupt: () => "the save could not be read safely",
  out_of_bounds: () => "the save could not be read safely",
  changed_during_read: () =>
    "the save changed while it was read. Wait for FM to finish saving, then try again",
};

/**
 * The read of the linked save: one sentence for the stats, then one each for
 * the save's player shortlists and staff shortlist when they were read or
 * failed. `isProblem` is whether the read left the snapshot without new stats
 * or a shortlist unchanged.
 */
export function seasonStatsCopy(stats: SeasonStatsOutcome): {
  text: string;
  isProblem: boolean;
} {
  const steps = [
    statsStepCopy(stats),
    saveShortlistsCopy(stats.shortlists),
    staffShortlistCopy(stats.staffShortlist),
  ].filter((step) => step !== null);
  return {
    text: steps.map((step) => step.text).join(" "),
    isProblem: steps.some((step) => step.isProblem),
  };
}

/** How many shortlists the save gave, or why they were not read; null when there is nothing to say. */
function saveShortlistsCopy(
  shortlists: SaveShortlistsOutcome,
): { text: string; isProblem: boolean } | null {
  switch (shortlists.status) {
    case "read":
      return shortlists.count === 0
        ? null
        : {
            text: `Read ${formatCount(shortlists.count)} ${shortlists.count === 1 ? "shortlist" : "shortlists"} from the save.`,
            isProblem: false,
          };
    case "notRead":
      return null;
    case "failed":
      return {
        text:
          shortlists.kind === "database"
            ? "The save's shortlists could not be stored, so the previous ones are kept."
            : `The save's shortlists were not read: ${seasonStatsReadFailureReasons[shortlists.kind]("the linked save file")}. The previous ones are kept.`,
        isProblem: true,
      };
  }
}

/**
 * How many staff the save's staff shortlists hold and how many of them could
 * not be read, or why none were read; null when there is nothing to say.
 */
function staffShortlistCopy(
  staffShortlist: SaveShortlistsOutcome,
): { text: string; isProblem: boolean } | null {
  switch (staffShortlist.status) {
    case "read": {
      const { count, unread } = staffShortlist;
      if (count === 0) {
        return null;
      }
      return {
        text:
          unread === 0
            ? `Read ${formatCount(count)} shortlisted staff from the save.`
            : `Read ${formatCount(count)} of ${formatCount(count + unread)} shortlisted staff from the save. ${formatCount(unread)} could not be read.`,
        isProblem: false,
      };
    }
    case "notRead":
      return null;
    case "failed":
      return {
        text:
          staffShortlist.kind === "database"
            ? "The save's staff shortlist could not be stored, so the previous one is kept."
            : `The save's staff shortlist was not read: ${seasonStatsReadFailureReasons[staffShortlist.kind]("the linked save file")}. The previous one is kept.`,
        isProblem: true,
      };
  }
}

function statsStepCopy(stats: SeasonStatsOutcome): {
  text: string;
  isProblem: boolean;
} {
  switch (stats.status) {
    case "attached":
      return {
        text: `Attached season stats for ${formatCount(stats.matchedPlayers)} players from the save dated ${formatLongGameDate(stats.gameDate)}.`,
        isProblem: false,
      };
    case "notLinked":
      return {
        text: "No FM save file is linked, so no season stats were read. Link one in Settings.",
        isProblem: false,
      };
    case "noMatches":
      return {
        text: `No players in the save dated ${formatLongGameDate(stats.gameDate)} matched the current snapshot, so season stats were not changed.`,
        isProblem: true,
      };
    case "staleContext":
      return {
        text: "The save or snapshot changed while season stats were read, so they were not attached.",
        isProblem: true,
      };
    case "noSnapshot":
      return {
        text: "There is no current snapshot to attach season stats to.",
        isProblem: true,
      };
    case "failed":
      return {
        text:
          stats.kind === "database"
            ? "Season stats could not be stored, so the previous stats are unchanged."
            : `Season stats were not read: ${seasonStatsReadFailureReasons[stats.kind](stats.fileName ?? "the linked save file")}.`,
        isProblem: true,
      };
  }
}

const shortlistReadFailureReasons: Record<
  Exclude<PlayerShortlistReadErrorKind, "database">,
  (fileName: string) => string
> = {
  unreadable: (fileName) => `${fileName} could not be found or opened`,
  too_large: (fileName) => `${fileName} is too large to be an FM shortlist`,
  not_fm_shortlist: (fileName) => `${fileName} is not a readable FM shortlist`,
  not_player_shortlist: (fileName) => `${fileName} is not a player shortlist`,
  unsupported_format: (fileName) =>
    `${fileName} uses a shortlist format that is not supported`,
};

function shortlistFailureCopy({
  kind,
  fileName,
}: PlayerShortlistsOutcome["failed"][number]): string {
  if (fileName === null) {
    return "The shortlist files could not be updated, so their previous players are kept.";
  }
  const reason =
    kind === "database"
      ? `${fileName} could not be stored`
      : shortlistReadFailureReasons[kind](fileName);
  return `${reason}, so its previous players are kept.`;
}

/**
 * How many linked shortlist files were re-read plus each failed file, or null
 * when the save links none.
 */
function shortlistsCopy({
  read,
  failed,
}: PlayerShortlistsOutcome): { text: string; isProblem: boolean } | null {
  if (read === 0 && failed.length === 0) {
    return null;
  }
  return {
    text: [
      `Read ${formatCount(read)} ${read === 1 ? "shortlist file" : "shortlist files"}.`,
      ...failed.map(shortlistFailureCopy),
    ].join(" "),
    isProblem: failed.length > 0,
  };
}

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
  return ` Scan ${formatDurationMs(timings.scanMs)}, preparation ${formatDurationMs(timings.prepareMs)}, scoring ${formatDurationMs(timings.scoringMs)}, save ${formatDurationMs(timings.saveMs)}, finalization ${formatDurationMs(timings.finalizeMs)}, season stats ${formatDurationMs(timings.statsMs)}, total ${formatDurationMs(timings.totalMs)}.`;
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
  const stats = seasonStatsCopy(result.stats);
  const shortlist = shortlistsCopy(result.shortlists);
  const isProblem = stats.isProblem || shortlist?.isProblem === true;
  return {
    icon: isProblem ? CircleAlert : CircleCheck,
    tone: isProblem ? toneClasses.warning : toneClasses.success,
    body: `${loaded}${latestMessage} ${stats.text}${shortlist ? ` ${shortlist.text}` : ""}`,
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
