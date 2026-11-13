import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatCount, formatMoney } from "@/utils/format";
import type {
  DashboardGraphics,
  DashboardLineupSlot,
  DashboardRecommendation,
  DashboardScoreMode,
} from "../types/dashboard";
import { slotContext, slotLabel } from "../utils/slot-label";
import { Gain } from "./dashboard-table";
import { CardSlotLine, cardDetailClass, PlayerCard } from "./player-card";

/** A recommended signing's club and age, for the line under their name. */
export function signingClubLine(recommendation: DashboardRecommendation) {
  return [
    recommendation.club ?? "No club",
    recommendation.age === null ? null : `${recommendation.age} years`,
  ]
    .filter((part) => part !== null)
    .join(" · ");
}

/** A recommended signing's value and, behind a Moneyball score, season minutes. */
export function signingValueLine(
  recommendation: DashboardRecommendation,
  mode: DashboardScoreMode,
) {
  return [
    recommendation.marketValueGbp === null
      ? "Value unknown"
      : formatMoney(recommendation.marketValueGbp),
    mode === "moneyball" && recommendation.minutes !== null
      ? `${formatCount(recommendation.minutes)} min`
      : null,
  ]
    .filter((part) => part !== null)
    .join(" · ");
}

/** One recommended signing: who, for which slot, and the points gained there. */
export function SigningCard({
  recommendation,
  slots,
  mode,
  graphics,
}: {
  recommendation: DashboardRecommendation;
  slots: ReadonlyMap<string, DashboardLineupSlot>;
  mode: DashboardScoreMode;
  graphics: DashboardGraphics;
}) {
  const slot = slots.get(recommendation.laneId);
  if (!slot) return null;
  const label = slotLabel(slot);
  const alsoImproves = recommendation.otherLanes
    .map((other) => {
      const otherSlot = slots.get(other.laneId);
      return otherSlot
        ? `${slotLabel(otherSlot).compact} +${other.improvement}`
        : null;
    })
    .filter((text) => text !== null)
    .join(" · ");
  return (
    <PlayerCard
      uid={recommendation.uid}
      name={recommendation.name}
      view={mode}
      portrait={graphics.portrait(recommendation.uid, recommendation.name)}
      details={
        <>
          <p className={cardDetailClass}>
            <span className="flex size-3 shrink-0 items-center justify-center rounded-[2px] bg-surface-container-high">
              {graphics.crest(recommendation.clubUid)}
            </span>
            <span className="truncate">{signingClubLine(recommendation)}</span>
          </p>
          <p className={cardDetailClass}>
            <span className="truncate">
              {signingValueLine(recommendation, mode)}
            </span>
          </p>
        </>
      }
      metricLabel="Gain"
      metric={<Gain points={recommendation.improvement} size="lg" />}
    >
      <CardSlotLine
        placement={label.compact}
        context={slotContext(slot)}
        title={label.full}
        aside={alsoImproves ? `Also ${alsoImproves}` : undefined}
      />
      <p className="flex items-center justify-between gap-2">
        <span className="truncate text-body-sm text-on-surface-variant">
          {slot.incumbent ? (
            <>
              Replaces{" "}
              <span className="text-on-surface">{slot.incumbent.name}</span>
            </>
          ) : (
            "Fills an empty slot"
          )}
        </span>
        <span className="flex shrink-0 items-center gap-1.5">
          {slot.incumbent ? (
            <>
              <ScoreBadge
                variant="card"
                score={slot.incumbent.score}
                roleName={`${slot.incumbent.name} at ${label.compact}`}
              />
              <span
                aria-hidden="true"
                className="text-label-sm text-on-surface-variant"
              >
                →
              </span>
            </>
          ) : null}
          <ScoreBadge
            variant="card"
            score={recommendation.score}
            roleName={`${recommendation.name} at ${label.compact}`}
          />
        </span>
      </p>
    </PlayerCard>
  );
}
