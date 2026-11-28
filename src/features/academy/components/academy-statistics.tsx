import {
  BadgeCheck,
  BadgeEuro,
  CircleMinus,
  Goal,
  GraduationCap,
  Handshake,
  Library,
  type LucideIcon,
  UsersRound,
} from "lucide-react";
import { Tooltip } from "@/components/ui/tooltip/tooltip";
import { formatCount, formatMoney } from "@/utils/format";
import type { AcademyStatistics as AcademyStatisticsData } from "../utils/academy-statistics";

type AcademyStatisticsProps = {
  trackedPlayers: number;
  classes?: number;
  statistics: AcademyStatisticsData;
  status?: AcademyStatisticsStatus;
};

export type AcademyStatisticsStatus = "ready" | "loading" | "error";

type Metric = {
  id: string;
  label: string;
  value: string;
  /** A definition or the reason the value is missing, shown on the label. */
  tooltip?: string;
  icon: LucideIcon;
};

export function AcademyStatistics({
  trackedPlayers,
  classes,
  statistics,
  status = "ready",
}: AcademyStatisticsProps) {
  const unavailableExplanation =
    status === "loading"
      ? "Loading Academy details…"
      : status === "error"
        ? "Could not load the Academy details."
        : null;
  // A class with members shows no career figure until the linked FM save has been read.
  const needsSeasonStats = (value: number | null) =>
    status === "ready" && value === null && trackedPlayers > 0
      ? "Needs season stats from your linked FM save."
      : undefined;
  const outcomeMetrics: Metric[] = [
    {
      id: "graduates",
      label: "Graduates",
      value: formatNullableCount(statistics.graduates),
      tooltip:
        needsSeasonStats(statistics.graduates) ??
        "At least one competitive appearance.",
      icon: GraduationCap,
    },
    {
      id: "sale-income",
      label: "Academy income",
      value:
        statistics.saleFeeEur === null
          ? "—"
          : formatMoney(statistics.saleFeeEur),
      tooltip:
        status === "ready" && statistics.saleFeeEur === null
          ? "Unavailable while details load."
          : "Sale fees you recorded.",
      icon: BadgeEuro,
    },
    {
      id: "released-players",
      label: "Released players",
      value: formatNullableCount(statistics.releasedPlayers),
      tooltip:
        status === "ready" && statistics.releasedPlayers === null
          ? "Unavailable while details load."
          : undefined,
      icon: CircleMinus,
    },
    {
      id: "goals",
      label: "Goals",
      value: formatNullableCount(statistics.goals),
      tooltip: needsSeasonStats(statistics.goals),
      icon: Goal,
    },
    {
      id: "assists",
      label: "Assists",
      value: formatNullableCount(statistics.assists),
      tooltip: needsSeasonStats(statistics.assists),
      icon: Handshake,
    },
  ];
  const contextMetrics: Metric[] = [
    ...(classes === undefined
      ? []
      : [
          {
            id: "classes",
            label: "Classes",
            value: formatCount(classes),
            icon: Library,
          },
        ]),
    {
      id: "tracked-players",
      label: "Tracked players",
      value: formatCount(trackedPlayers),
      icon: UsersRound,
    },
    {
      id: "reported-senior-players",
      label: "Reported senior players",
      value: formatNullableCount(statistics.reportedSeniorPlayers),
      tooltip: "In the senior squad in the current snapshot.",
      icon: BadgeCheck,
    },
  ];

  return (
    <div className="space-y-4">
      {status !== "ready" ? (
        <p className="text-body-sm text-on-surface-variant" role="status">
          {unavailableExplanation}
        </p>
      ) : null}
      <section aria-label="Academy outcomes" aria-busy={status === "loading"}>
        <dl className="grid grid-cols-2 gap-4 sm:grid-cols-3 xl:grid-cols-5">
          {outcomeMetrics.map((metric) => (
            <AcademyOutcomeMetric key={metric.id} metric={metric} />
          ))}
        </dl>
      </section>
      <section
        aria-label="Academy context"
        className="border-t border-outline-variant pt-4"
      >
        <dl className="grid grid-cols-2 gap-4 sm:grid-cols-3">
          {contextMetrics.map((metric) => (
            <AcademyContextMetric key={metric.id} metric={metric} />
          ))}
        </dl>
      </section>
    </div>
  );
}

function AcademyOutcomeMetric({ metric }: { metric: Metric }) {
  const Icon = metric.icon;

  return (
    <div className="min-w-0">
      <dt className="flex items-center gap-2 text-label-md text-on-surface-variant uppercase">
        <Icon aria-hidden className="size-4 shrink-0" strokeWidth={1.5} />
        <MetricLabel metric={metric} />
      </dt>
      <dd
        data-testid={`academy-stat-${metric.id}`}
        className="mt-1 font-mono text-mono-lg text-on-surface tabular-nums"
      >
        {metric.value}
      </dd>
    </div>
  );
}

function AcademyContextMetric({ metric }: { metric: Metric }) {
  const Icon = metric.icon;

  return (
    <div className="min-w-0">
      <dt className="flex min-w-0 items-center gap-2">
        <Icon
          aria-hidden
          className="size-4 shrink-0 text-on-surface-variant"
          strokeWidth={1.5}
        />
        <MetricLabel
          metric={metric}
          className="min-w-0 text-label-sm text-on-surface-variant uppercase"
        />
      </dt>
      <dd
        data-testid={`academy-stat-${metric.id}`}
        className="mt-1 min-w-0 pl-6 font-mono text-mono-md text-on-surface tabular-nums"
      >
        {metric.value}
      </dd>
    </div>
  );
}

function MetricLabel({
  metric,
  className,
}: {
  metric: Metric;
  className?: string;
}) {
  const label = <span className={className}>{metric.label}</span>;
  return metric.tooltip ? (
    <Tooltip content={metric.tooltip} focusable>
      {label}
    </Tooltip>
  ) : (
    label
  );
}

function formatNullableCount(value: number | null) {
  return value === null ? "—" : formatCount(value);
}
