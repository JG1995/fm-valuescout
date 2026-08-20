import type { MoneyballMetric } from "./moneyball-metrics";

export function formatMoneyballMetric(
  metric: MoneyballMetric,
  value: number | null | undefined,
) {
  if (value === null || value === undefined) return "—";

  if (metric.kind === "percentage") {
    const percentage =
      metric.percentageScale === "fraction" ? value * 100 : value;
    return `${percentage.toFixed(metric.precision)}%`;
  }

  const formatted = value.toFixed(metric.precision);
  if (metric.kind !== "distance") return formatted;
  return `${formatted}${metric.id.endsWith("_per_90") ? " km / 90" : " km"}`;
}
