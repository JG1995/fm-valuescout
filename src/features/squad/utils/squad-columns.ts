import { PLAYER_METRICS, type PlayerMetric } from "@/utils/player-metrics";
import { SUGGESTED_TRAINING_COLUMN_ID } from "@/utils/suggested-training";

/**
 * Squad-only Suggested Training metric. It stays out of the shared
 * `PLAYER_METRICS` catalog so Search, Moneyball, and Staff never see it.
 * The column is configurable, removable, resizable, and re-addable, but
 * never sortable: there is no sort path for suggestion text anywhere.
 */
export const SUGGESTED_TRAINING_METRIC: PlayerMetric = {
  id: SUGGESTED_TRAINING_COLUMN_ID,
  label: "Suggested Training",
  category: "ability-reputation",
  kind: "string",
  align: "left",
  defaultWidth: 176,
  sortable: false,
  operators: [
    { id: "contains", label: "contains" },
    { id: "not_contains", label: "does not contain" },
    { id: "is", label: "is" },
    { id: "is_not", label: "is not" },
  ],
};

/** Header metric list for the Squad table: shared catalog plus the Squad-only entry. */
export const SQUAD_HEADER_METRICS: readonly PlayerMetric[] = [
  ...PLAYER_METRICS,
  SUGGESTED_TRAINING_METRIC,
];

const SQUAD_METRIC_BY_ID = new Map(
  SQUAD_HEADER_METRICS.map((metric) => [metric.id, metric]),
);

export function getSquadTableMetric(
  metricId: string,
): PlayerMetric | undefined {
  return SQUAD_METRIC_BY_ID.get(metricId);
}
