import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { CircleAlert, ListPlus, Target, UserRoundCheck } from "lucide-react";
import { type ReactNode, useMemo, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { fieldClasses } from "@/components/ui/field/field-styles";
import { Panel } from "@/components/ui/panel/panel";
import { cn } from "@/utils/cn";
import { formatCount } from "@/utils/format";
import { dashboardSigningsQueryOptions } from "../api/dashboard-query-options";
import type {
  DashboardContext,
  DashboardGraphics,
  DashboardScoreMode,
  DashboardSignings,
} from "../types/dashboard";
import { slotLabel } from "../utils/slot-label";
import { CardGrid } from "./card-grid";
import {
  cardPanelSummaryClass,
  dashboardArea,
  dashboardPanelClass,
  dashboardPanelContentClass,
} from "./dashboard-grid";
import {
  aboveExpandClass,
  expandablePanelClass,
  PanelExpandButton,
} from "./panel-expand-button";
import { cardGridMinHeightClass, PLAYER_CARD_MIN } from "./player-card";
import { ScoreModeToggle } from "./score-mode-toggle";
import { SigningCard } from "./signing-card";
import { SlotSigningsModal } from "./slot-signings-modal";
import { StartingXiList } from "./starting-xi-list";

export type DashboardShortlistOption = { id: number; label: string };

// A plain path, so Search's validateSearch supplies its own defaults.
const playerSearchPath: string = "/search";

const linkButtonClass =
  "inline-flex h-8 shrink-0 items-center rounded-md border border-outline px-4 text-label-lg text-on-surface transition-colors duration-150 ease-out hover:bg-surface-container-high active:bg-surface-container-highest";

// Two rows of six: the most recommendations one read returns.
const MAX_SIGNING_COLUMNS = 6;

function plural(count: number, noun: string) {
  return `${formatCount(count)} ${noun}${count === 1 ? "" : "s"}`;
}

/** How many shortlisted players would improve the XI or the selected slot. */
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
  const total = signings.totalRecommendations;
  return (
    <>
      {selectedSlot === null
        ? `${formatCount(total)} of ${plural(signings.shortlistedPlayers, "shortlisted player")} would improve your XI`
        : `${plural(total, "shortlisted player")} would improve ${selectedSlot}`}
      {total > 0 ? " · best first" : null}
      {selectedSlot === null ? null : (
        <>
          {" · "}
          <button
            type="button"
            className={cn(
              aboveExpandClass,
              "cursor-pointer text-on-surface underline underline-offset-2",
            )}
            onClick={onClearSlot}
          >
            Show all slots
          </button>
        </>
      )}
      {refreshing ? " · Updating…" : null}
    </>
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
 * The best recommended signings as cards, the club's starting XI they are
 * measured against, and the per-slot view behind the cards. All three read one
 * result, so the score mode always changes them together.
 */
export function SigningsWorkspace({
  context,
  mode,
  onModeChange,
  shortlists,
  graphics,
}: {
  context: DashboardContext;
  mode: DashboardScoreMode;
  onModeChange: (mode: DashboardScoreMode) => void;
  shortlists: readonly DashboardShortlistOption[];
  graphics: DashboardGraphics;
}) {
  const [chosenShortlistId, setChosenShortlistId] = useState<number | null>(
    null,
  );
  const [laneId, setLaneId] = useState<string | null>(null);
  const [slotsOpen, setSlotsOpen] = useState(false);
  // A shortlist removed elsewhere falls back to all of them.
  const shortlist = shortlists.find(({ id }) => id === chosenShortlistId);
  const shortlistId = shortlist?.id ?? null;
  const query = useQuery(
    dashboardSigningsQueryOptions(context, { mode, shortlistId, laneId }),
  );
  const signings = query.data ?? null;
  const slots = useMemo(
    () => new Map(signings?.slots.map((slot) => [slot.laneId, slot])),
    [signings],
  );
  const selectedSlot = laneId === null ? undefined : slots.get(laneId);
  const selectedSlotName = selectedSlot
    ? slotLabel(selectedSlot).compact
    : null;
  const hasPlannerPicks =
    signings?.slots.some((slot) => slot.incumbent?.source === "planner") ??
    false;
  // The per-slot view opens once any slot has a signing to show.
  const expandable =
    signings?.slots.some((slot) => slot.bestSigning !== null) ?? false;

  const controls = (
    <>
      <div
        className={cn(aboveExpandClass, "flex flex-wrap items-center gap-2")}
      >
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
      </div>
      {expandable ? (
        <PanelExpandButton onClick={() => setSlotsOpen(true)}>
          Best per slot
        </PanelExpandButton>
      ) : null}
    </>
  );

  return (
    <>
      <Panel
        title="Recommended signings"
        flush
        actions={controls}
        className={cn(
          dashboardArea.signings,
          dashboardPanelClass,
          expandablePanelClass,
        )}
        contentClassName={dashboardPanelContentClass}
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
          <div
            className={cn(
              aboveExpandClass,
              "flex flex-1 items-center justify-center overflow-y-auto",
            )}
          >
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
            <p className={cardPanelSummaryClass} role="status">
              <SigningsSummary
                signings={signings}
                selectedSlot={selectedSlotName}
                refreshing={query.isPlaceholderData}
                onClearSlot={() => setLaneId(null)}
              />
            </p>
            <div
              className={cn(
                "flex flex-1 border-t border-outline-variant",
                query.isPlaceholderData && "opacity-60",
              )}
            >
              <CardGrid
                label="Recommended signings, best first"
                items={signings.recommendations}
                min={PLAYER_CARD_MIN}
                maxColumns={MAX_SIGNING_COLUMNS}
                className={cardGridMinHeightClass}
                itemKey={(recommendation) => recommendation.uid}
              >
                {(recommendation) => (
                  <SigningCard
                    recommendation={recommendation}
                    slots={slots}
                    mode={mode}
                    graphics={graphics}
                  />
                )}
              </CardGrid>
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
        className={cn(dashboardArea.lineup, dashboardPanelClass)}
        contentClassName={dashboardPanelContentClass}
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
              graphics={graphics}
            />
            <div className="shrink-0 space-y-1 border-t border-outline-variant px-4 py-2 text-body-sm text-on-surface-variant">
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
      {signings ? (
        <SlotSigningsModal
          open={slotsOpen}
          onClose={() => setSlotsOpen(false)}
          signings={signings}
          mode={mode}
          onModeChange={onModeChange}
          shortlistLabel={shortlist?.label ?? "All shortlists"}
          refreshing={query.isPlaceholderData}
          graphics={graphics}
        />
      ) : null}
    </>
  );
}
