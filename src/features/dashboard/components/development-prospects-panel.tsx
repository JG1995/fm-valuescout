import { useQuery } from "@tanstack/react-query";
import { CircleAlert, TrendingUp } from "lucide-react";
import { useState } from "react";
import { Button } from "@/components/ui/button/button";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { cn } from "@/utils/cn";
import { dashboardProspectsQueryOptions } from "../api/dashboard-query-options";
import type {
  DashboardContext,
  DashboardGraphics,
  DashboardProspect,
} from "../types/dashboard";
import { slotLabel } from "../utils/slot-label";
import { CardGrid } from "./card-grid";
import {
  cardPanelSummaryClass,
  dashboardArea,
  dashboardPanelClass,
  dashboardPanelContentClass,
} from "./dashboard-grid";
import { Gain } from "./dashboard-table";
import { expandablePanelClass, PanelExpandButton } from "./panel-expand-button";
import {
  CardSlotLine,
  cardDetailClass,
  cardGridMinHeightClass,
  PLAYER_CARD_MIN,
  PlayerCard,
} from "./player-card";
import { YouthXiModal } from "./youth-xi-modal";

// The oldest player Rust counts as young.
const MAX_AGE = 21;
// Two rows of five: the most prospects one read returns.
const MAX_PROSPECT_COLUMNS = 5;

function ProspectCard({
  prospect,
  graphics,
}: {
  prospect: DashboardProspect;
  graphics: DashboardGraphics;
}) {
  const label = slotLabel(prospect);
  return (
    <PlayerCard
      uid={prospect.uid}
      name={prospect.name}
      portrait={graphics.portrait(prospect.uid, prospect.name)}
      details={<p className={cardDetailClass}>{prospect.age} years</p>}
      metricLabel="Potential"
      metric={
        <ScoreBadge
          variant="hero"
          className="size-auto!"
          score={prospect.potentialScore}
          roleName={`${prospect.name} potential at ${label.compact}`}
        />
      }
    >
      <CardSlotLine
        placement={label.compact}
        context={label.context}
        title={label.full}
      />
      <p className="flex items-center justify-between gap-2">
        <span className="flex items-center gap-1.5 text-body-sm text-on-surface-variant">
          Now
          <ScoreBadge
            variant="card"
            score={prospect.currentScore}
            roleName={`${prospect.name} now at ${label.compact}`}
          />
        </span>
        <span className="flex items-center gap-1.5 text-body-sm text-on-surface-variant">
          Room to grow
          <Gain points={prospect.potentialScore - prospect.currentScore} />
        </span>
      </p>
    </PlayerCard>
  );
}

/**
 * The club's young players with the highest potential in the tactic as cards,
 * and the XI its young players form behind them.
 */
export function DevelopmentProspectsPanel({
  context,
  graphics,
}: {
  context: DashboardContext;
  graphics: DashboardGraphics;
}) {
  const [youthXiOpen, setYouthXiOpen] = useState(false);
  const query = useQuery(dashboardProspectsQueryOptions(context));
  const prospects = query.data ?? null;
  // The XI opens once any slot has a young player.
  const expandable =
    prospects?.youthXi.some((slot) => slot.player !== null) ?? false;
  return (
    <>
      <Panel
        title="Development prospects"
        flush
        actions={
          expandable ? (
            <PanelExpandButton onClick={() => setYouthXiOpen(true)}>
              Youth XI
            </PanelExpandButton>
          ) : null
        }
        className={cn(
          dashboardArea.prospects,
          dashboardPanelClass,
          expandablePanelClass,
        )}
        contentClassName={dashboardPanelContentClass}
      >
        {query.isError ? (
          <EmptyState
            icon={CircleAlert}
            title="Could not load development prospects"
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
            Loading development prospects…
          </p>
        ) : prospects === null || prospects.players.length === 0 ? (
          <div className="flex flex-1 items-center justify-center overflow-y-auto">
            <EmptyState
              icon={TrendingUp}
              title="No prospects with room to grow"
            >
              No player aged {MAX_AGE} or younger is projected to improve in a
              slot of your tactic.
            </EmptyState>
          </div>
        ) : (
          <>
            <p className={cardPanelSummaryClass}>
              Aged {MAX_AGE} or younger with room to grow · highest potential
              first
            </p>
            <div className="flex flex-1 border-t border-outline-variant">
              <CardGrid
                label="Development prospects, highest potential first"
                items={prospects.players}
                min={PLAYER_CARD_MIN}
                maxColumns={MAX_PROSPECT_COLUMNS}
                className={cardGridMinHeightClass}
                itemKey={(prospect) => prospect.uid}
              >
                {(prospect) => (
                  <ProspectCard prospect={prospect} graphics={graphics} />
                )}
              </CardGrid>
            </div>
          </>
        )}
      </Panel>
      {prospects ? (
        <YouthXiModal
          open={youthXiOpen}
          onClose={() => setYouthXiOpen(false)}
          slots={prospects.youthXi}
          maxAge={MAX_AGE}
          graphics={graphics}
        />
      ) : null}
    </>
  );
}
