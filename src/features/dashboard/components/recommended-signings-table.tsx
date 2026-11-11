import { formatCount, formatMoney } from "@/utils/format";
import type {
  DashboardRecommendation,
  DashboardScoreMode,
  DashboardSignings,
} from "../types/dashboard";
import { slotContext, slotLabel } from "../utils/slot-label";
import {
  DashboardTable,
  Gain,
  numericCellClass,
  PlayerCell,
  rowClass,
  Score,
  SlotCell,
} from "./dashboard-table";

function playerDetail(
  recommendation: DashboardRecommendation,
  mode: DashboardScoreMode,
) {
  return [
    recommendation.club ?? "No club",
    recommendation.age === null ? null : `${recommendation.age} years`,
    mode === "moneyball" && recommendation.minutes !== null
      ? `${formatCount(recommendation.minutes)} min`
      : null,
  ]
    .filter((part) => part !== null)
    .join(" · ");
}

/** Shortlisted players ranked by the points they add to a slot of the XI. */
export function RecommendedSigningsTable({
  signings,
  mode,
  showOtherSlots,
}: {
  signings: DashboardSignings;
  mode: DashboardScoreMode;
  /** Whether rows list the further slots a player improves. */
  showOtherSlots: boolean;
}) {
  const slots = new Map(signings.slots.map((slot) => [slot.laneId, slot]));
  return (
    <DashboardTable
      caption="Shortlisted players ranked by improvement over your current player in a tactic slot"
      headings={[
        { label: "Player" },
        { label: "Slot", width: "w-40" },
        { label: "Replaces", width: "w-48" },
        { label: "Score", numeric: true, width: "w-14" },
        { label: "Gain", numeric: true, width: "w-14" },
        { label: "Value", numeric: true, width: "w-20" },
        ...(showOtherSlots ? [{ label: "Also improves", width: "w-32" }] : []),
      ]}
    >
      {signings.recommendations.map((recommendation) => {
        const slot = slots.get(recommendation.laneId);
        if (!slot) return null;
        const label = slotLabel(slot);
        return (
          <tr key={recommendation.uid} className={rowClass}>
            <PlayerCell
              uid={recommendation.uid}
              name={recommendation.name}
              detail={playerDetail(recommendation, mode)}
              view={mode}
            />
            <SlotCell label={label} context={slotContext(slot)} />
            <td className="min-w-0 px-2">
              {slot.incumbent ? (
                <span className="flex items-center justify-between gap-2">
                  <span
                    className="truncate text-body-sm text-on-surface"
                    title={slot.incumbent.name}
                  >
                    {slot.incumbent.name}
                  </span>
                  <Score
                    score={slot.incumbent.score}
                    name={`${slot.incumbent.name} at ${label.compact}`}
                  />
                </span>
              ) : (
                <span className="text-body-sm text-on-surface-variant">
                  Empty slot
                </span>
              )}
            </td>
            <td className="px-2 text-right">
              <Score
                score={recommendation.score}
                name={`${recommendation.name} at ${label.compact}`}
              />
            </td>
            <td className="px-2 text-right">
              <Gain points={recommendation.improvement} />
            </td>
            <td className={numericCellClass}>
              {recommendation.marketValueGbp === null ? (
                <span className="text-on-surface-variant">—</span>
              ) : (
                formatMoney(recommendation.marketValueGbp)
              )}
            </td>
            {showOtherSlots ? (
              <td className="truncate px-2 text-body-sm text-on-surface-variant">
                {recommendation.otherLanes
                  .map((other) => {
                    const otherSlot = slots.get(other.laneId);
                    return otherSlot
                      ? `${slotLabel(otherSlot).compact} +${other.improvement}`
                      : null;
                  })
                  .filter((text) => text !== null)
                  .join(" · ") || "—"}
              </td>
            ) : null}
          </tr>
        );
      })}
    </DashboardTable>
  );
}
