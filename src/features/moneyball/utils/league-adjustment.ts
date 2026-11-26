import type { MoneyballLeagueBreakdown } from "../types/moneyball-profile";

/** Score points with their direction: +12, −24, or 0. */
export function signedPoints(points: number) {
  if (points > 0) return `+${points}`;
  return points < 0 ? `−${Math.abs(points)}` : "0";
}

/** One line saying how a player's league moved his role scores. */
export function leagueAdjustmentSummary(adjustment: MoneyballLeagueBreakdown) {
  if (adjustment.division === adjustment.referenceDivision) {
    return `Reference league: ${adjustment.division}`;
  }
  const league =
    adjustment.division === null
      ? "no league recorded"
      : `${adjustment.division} ${Math.round(adjustment.strength)}`;
  const moved =
    adjustment.shift === 0
      ? "role scores not moved"
      : `role scores ${signedPoints(adjustment.shift)}`;
  return `League strength: ${moved} (${league}${
    adjustment.estimated ? ", estimated" : ""
  } against ${adjustment.referenceDivision} ${Math.round(
    adjustment.referenceStrength,
  )})`;
}
