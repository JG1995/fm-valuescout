import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { CircleAlert, ListPlus, Target, UserRoundCheck } from "lucide-react";
import { type ReactNode, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { fieldClasses } from "@/components/ui/field/field-styles";
import { Panel } from "@/components/ui/panel/panel";
import { cn } from "@/utils/cn";
import { formatCount } from "@/utils/format";
import { dashboardSigningsQueryOptions } from "../api/dashboard-query-options";
import type {
  DashboardContext,
  DashboardScoreMode,
  DashboardSignings,
} from "../types/dashboard";
import { slotLabel } from "../utils/slot-label";
import { RecommendedSigningsTable } from "./recommended-signings-table";
import { ScoreModeToggle } from "./score-mode-toggle";
import { StartingXiList } from "./starting-xi-list";

export type DashboardShortlistOption = { id: number; label: string };

// A plain path, so Search's validateSearch supplies its own defaults.
const playerSearchPath: string = "/search";

const linkButtonClass =
  "inline-flex h-8 shrink-0 items-center rounded-md border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high active:bg-surface-container-highest";

function plural(count: number, noun: string) {
  return `${formatCount(count)} ${noun}${count === 1 ? "" : "s"}`;
}

function SigningsSummary({
  signings,
  selectedSlot,
  refreshing,
  onClearSlot,
}: {
  signings: DashboardSignings;
  selectedSlot: string | null;
  refreshing: boolean;
  onClearSlot: () => void;
}) {
  const shown = signings.recommendations.length;
  const total = signings.totalRecommendations;
  return (
    <div className="flex min-h-8 flex-wrap items-center justify-between gap-2 px-4 pb-2">
      <p className="text-body-sm text-on-surface-variant" role="status">
        {selectedSlot === null
          ? `${formatCount(total)} of ${plural(signings.shortlistedPlayers, "shortlisted player")} would improve your XI`
          : `${plural(total, "shortlisted player")} would improve ${selectedSlot}`}
        {total > shown ? ` · top ${shown} shown` : null}
        {refreshing ? " · Updating…" : null}
      </p>
      {selectedSlot === null ? null : (
        <Button variant="ghost" onClick={onClearSlot}>
          Show all slots
        </Button>
      )}
    </div>
  );
}

function SigningsEmptyState({
  signings,
  hasShortlists,
  selectedSlot,
  onClearSlot,
  onUseGeneral,
}: {
  signings: DashboardSignings;
  hasShortlists: boolean;
  selectedSlot: string | null;
  onClearSlot: () => void;
  onUseGeneral: () => void;
}): ReactNode {
  if (!signings.scoresAvailable) {
    return (
      <EmptyState
        icon={Target}
        title="No season stats in this snapshot"
        action={
          <div className="flex flex-wrap justify-center gap-2 pt-2">
            <Link to="/settings" className={linkButtonClass}>
              Open Settings
            </Link>
            <Button variant="secondary" onClick={onUseGeneral}>
              Use General scores
            </Button>
          </div>
        }
      >
        Moneyball scores come from the FM save linked in Settings. Link it, then
        use Load Data to read its season stats.
      </EmptyState>
    );
  }
  if (!hasShortlists) {
    return (
      <EmptyState
        icon={ListPlus}
        title="No shortlist loaded"
        action={
          <Link to={playerSearchPath} className={cn(linkButtonClass, "mt-2")}>
            Open Player Search
          </Link>
        }
      >
        Recommendations compare your squad with shortlisted players. Add an FM
        shortlist in Player Search, or link your FM save in Settings to read its
        shortlists.
      </EmptyState>
    );
  }
  if (signings.shortlistedPlayers === 0) {
    return (
      <EmptyState icon={ListPlus} title="No shortlisted players to compare">
        None of this shortlist's players are in the current snapshot outside
        your club.
      </EmptyState>
    );
  }
  if (selectedSlot !== null) {
    return (
      <EmptyState
        icon={UserRoundCheck}
        title={`No shortlisted player improves ${selectedSlot}`}
        action={
          <Button variant="secondary" className="mt-2" onClick={onClearSlot}>
            Show all slots
          </Button>
        }
      >
        Your current player scores at least as high there as every shortlisted
        player.
      </EmptyState>
    );
  }
  return (
    <EmptyState
      icon={UserRoundCheck}
      title="No shortlisted player improves your XI"
    >
      Your current players score at least as high in every slot as the
      shortlisted players who can play there.
    </EmptyState>
  );
}

/**
 * The club's starting XI beside the shortlisted players who would improve it.
 * Both read one result, so the score mode always changes them together.
 */
export function SigningsWorkspace({
  context,
  mode,
  onModeChange,
  shortlists,
}: {
  context: DashboardContext;
  mode: DashboardScoreMode;
  onModeChange: (mode: DashboardScoreMode) => void;
  shortlists: readonly DashboardShortlistOption[];
}) {
  const [chosenShortlistId, setChosenShortlistId] = useState<number | null>(
    null,
  );
  const [laneId, setLaneId] = useState<string | null>(null);
  // A shortlist removed elsewhere falls back to all of them.
  const shortlistId = shortlists.some(({ id }) => id === chosenShortlistId)
    ? chosenShortlistId
    : null;
  const query = useQuery(
    dashboardSigningsQueryOptions(context, { mode, shortlistId, laneId }),
  );
  const signings = query.data ?? null;
  const selectedSlot = signings?.slots.find((slot) => slot.laneId === laneId);
  const selectedSlotName = selectedSlot
    ? slotLabel(selectedSlot).compact
    : null;
  const hasPlannerPicks =
    signings?.slots.some((slot) => slot.incumbent?.source === "planner") ??
    false;

  const controls = (
    <>
      {shortlists.length > 0 ? (
        <label className="flex items-center gap-2 text-body-md text-on-surface">
          Shortlist
          <select
            className={cn(fieldClasses, "max-w-56")}
            value={shortlistId ?? ""}
            onChange={(event) =>
              setChosenShortlistId(
                event.target.value === "" ? null : Number(event.target.value),
              )
            }
          >
            <option value="">All shortlists</option>
            {shortlists.map(({ id, label }) => (
              <option key={id} value={id}>
                {label}
              </option>
            ))}
          </select>
        </label>
      ) : null}
      <ScoreModeToggle mode={mode} onChange={onModeChange} />
    </>
  );

  return (
    <div className="grid grid-cols-[minmax(0,1fr)_24rem] gap-4">
      <Panel
        title="Recommended signings"
        flush
        actions={controls}
        className="flex flex-col"
        contentClassName="flex min-h-72 flex-1 flex-col"
      >
        {query.isError ? (
          <EmptyState
            icon={CircleAlert}
            title="Could not load recommended signings"
            action={
              <Button
                variant="secondary"
                className="mt-2"
                onClick={() => void query.refetch()}
              >
                Retry
              </Button>
            }
          >
            {query.error.message}
          </EmptyState>
        ) : query.isPending ? (
          <p className="p-4 text-body-md text-on-surface-variant" role="status">
            Loading recommended signings…
          </p>
        ) : signings === null ? (
          <EmptyState icon={CircleAlert} title="No squad to compare">
            The active save has no current snapshot for your managed club.
          </EmptyState>
        ) : signings.recommendations.length === 0 ? (
          <div className="flex flex-1 items-center justify-center">
            <SigningsEmptyState
              signings={signings}
              hasShortlists={shortlists.length > 0}
              selectedSlot={selectedSlotName}
              onClearSlot={() => setLaneId(null)}
              onUseGeneral={() => onModeChange("general")}
            />
          </div>
        ) : (
          <>
            <SigningsSummary
              signings={signings}
              selectedSlot={selectedSlotName}
              refreshing={query.isPlaceholderData}
              onClearSlot={() => setLaneId(null)}
            />
            {/* The XI beside it sets the row's height; the list scrolls inside it. */}
            <div className="relative flex-1 border-t border-outline-variant">
              <div
                className={cn(
                  "absolute inset-0 overflow-auto",
                  query.isPlaceholderData && "opacity-60",
                )}
              >
                <RecommendedSigningsTable
                  signings={signings}
                  mode={mode}
                  showOtherSlots={laneId === null}
                />
              </div>
            </div>
          </>
        )}
      </Panel>
      <Panel
        title="Starting XI"
        flush
        actions={
          <Link
            to="/my-club"
            search={{ view: "planner" }}
            className={linkButtonClass}
          >
            Open Planner
          </Link>
        }
      >
        {query.isError ? (
          <p className="p-4 text-body-md text-on-surface-variant">
            The starting XI could not be loaded.
          </p>
        ) : query.isPending ? (
          <p className="p-4 text-body-md text-on-surface-variant" role="status">
            Loading starting XI…
          </p>
        ) : signings === null || !signings.scoresAvailable ? (
          <p className="p-4 text-body-md text-on-surface-variant">
            {signings === null
              ? "No current snapshot for your managed club."
              : "Moneyball scores need season stats from your linked FM save."}
          </p>
        ) : (
          <>
            <StartingXiList
              slots={signings.slots}
              selectedLaneId={laneId}
              onSelectLane={setLaneId}
            />
            <div className="space-y-1 border-t border-outline-variant px-4 py-2 text-body-sm text-on-surface-variant">
              <p>
                {hasPlannerPicks
                  ? "A pin marks the Planner's first string; other slots take the best remaining fit."
                  : "The Planner has no first string, so every slot takes the best fit."}{" "}
                Select a slot to see its signings.
              </p>
              {signings.tacticIsDefault ? (
                <p>
                  These are the default tactic's slots.{" "}
                  <Link
                    to="/my-club"
                    search={{ view: "tactic" }}
                    className="text-on-surface underline underline-offset-2"
                  >
                    Set up your tactic
                  </Link>
                </p>
              ) : null}
            </div>
          </>
        )}
      </Panel>
    </div>
  );
}
