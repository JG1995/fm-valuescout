import {
  getMoneyballSearchMetric,
  MONEYBALL_SEARCH_METRICS,
  type MoneyballSearchMetric,
} from "./moneyball-search-metrics";
import {
  getPlayerMetric,
  PLAYER_METRICS,
  type PlayerMetric,
} from "./player-metrics";

/** A Player Search field: a player field or a season field. */
export type SearchMetric = PlayerMetric | MoneyballSearchMetric;

/** Every Player Search field: player fields, then season fields. */
export const SEARCH_METRICS: readonly SearchMetric[] = [
  ...PLAYER_METRICS,
  ...MONEYBALL_SEARCH_METRICS,
];

export function getSearchMetric(id: string): SearchMetric | undefined {
  return getPlayerMetric(id) ?? getMoneyballSearchMetric(id);
}

/** Whether a field or tactic column reads season stats: playing time, a statistic, or a Moneyball Score. */
export function isSeasonFieldId(id: string): boolean {
  return (
    id.startsWith("moneyball.") ||
    id.startsWith("moneyball_role.") ||
    id.startsWith("tactic_moneyball.")
  );
}
