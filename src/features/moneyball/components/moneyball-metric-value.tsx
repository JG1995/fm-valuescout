import { AttributeRow } from "@/components/ui/attribute-row/attribute-row";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMoneyballMetric } from "../utils/format-moneyball-metric";
import type { MoneyballMetric } from "../utils/moneyball-metrics";

type MoneyballMetricValueProps = {
  metric: MoneyballMetric;
  value: number | null | undefined;
  score: number | null | undefined;
};

export function MoneyballMetricValue({
  metric,
  value,
  score,
}: MoneyballMetricValueProps) {
  return (
    <AttributeRow label={metric.label}>
      <span className="inline-flex items-center gap-2">
        <span
          className={
            value === null || value === undefined
              ? "text-on-surface-variant"
              : undefined
          }
        >
          {formatMoneyballMetric(metric, value)}
        </span>
        {value !== null &&
        value !== undefined &&
        score !== null &&
        score !== undefined ? (
          <ScoreBadge score={score} roleName={metric.label} variant="card" />
        ) : null}
      </span>
    </AttributeRow>
  );
}
