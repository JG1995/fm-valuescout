import {
  BadgeCheck,
  BadgeEuro,
  CircleMinus,
  Globe2,
  Goal,
  GraduationCap,
  Handshake,
  Library,
  type LucideIcon,
  UsersRound,
} from "lucide-react";
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
  explanation: string;
  icon: LucideIcon;
  iconClassName: string;
};

export function AcademyStatistics({
  trackedPlayers,
  classes,
  statistics,
  status = "ready",
}: AcademyStatisticsProps) {
  const unavailableExplanation =
    status === "loading"
      ? "Waiting for the current Academy details to load."
      : status === "error"
        ? "Unavailable because the current Academy details could not be loaded."
        : null;
  const outcomeMetrics: Metric[] = [
    {
      id: "graduates",
      label: "Graduates",
      value: formatNullableCount(statistics.graduates),
      explanation:
        unavailableExplanation ??
        (statistics.graduates === null
          ? "Graduate data is not available until the memory reader exposes senior league appearances."
          : "Players with at least one reported senior league appearance."),
      icon: GraduationCap,
      iconClassName: "text-primary",
    },
    {
      id: "sale-income",
      label: "Academy income",
      value:
        statistics.saleFeeEur === null
          ? "—"
          : formatMoney(statistics.saleFeeEur),
      explanation:
        unavailableExplanation ??
        (statistics.saleFeeEur === null
          ? "Manual Academy outcomes are unavailable while details load."
          : "Manual sale fees recorded for tracked players."),
      icon: BadgeEuro,
      iconClassName: "text-success",
    },
    {
      id: "released-players",
      label: "Released players",
      value: formatNullableCount(statistics.releasedPlayers),
      explanation:
        unavailableExplanation ??
        (statistics.releasedPlayers === null
          ? "Manual Academy outcomes are unavailable while details load."
          : "Players manually marked as released in this cohort."),
      icon: CircleMinus,
      iconClassName: "text-on-surface-variant",
    },
    {
      id: "goals",
      label: "Goals",
      value: formatNullableCount(statistics.goals),
      explanation:
        unavailableExplanation ??
        (statistics.goals === null
          ? "Career goals are not available from the current memory reader."
          : "Reported career goals for tracked players."),
      icon: Goal,
      iconClassName: "text-primary",
    },
    {
      id: "assists",
      label: "Assists",
      value: formatNullableCount(statistics.assists),
      explanation:
        unavailableExplanation ??
        (statistics.assists === null
          ? "Career assists are not available from the current memory reader."
          : "Reported career assists for tracked players."),
      icon: Handshake,
      iconClassName: "text-primary",
    },
    {
      id: "international-caps",
      label: "International caps",
      value: formatNullableCount(statistics.internationalCaps),
      explanation:
        unavailableExplanation ??
        (statistics.internationalCaps === null
          ? "International caps are not available from the current memory reader."
          : "Reported international caps for tracked players."),
      icon: Globe2,
      iconClassName: "text-primary",
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
            explanation: "Saved Class of YYYY cohorts.",
            icon: Library,
            iconClassName: "text-on-surface-variant",
          },
        ]),
    {
      id: "tracked-players",
      label: "Tracked players",
      value: formatCount(trackedPlayers),
      explanation: "Players retained in Academy classes for this save.",
      icon: UsersRound,
      iconClassName: "text-on-surface-variant",
    },
    {
      id: "reported-senior-players",
      label: "Reported senior players",
      value: formatNullableCount(statistics.reportedSeniorPlayers),
      explanation:
        unavailableExplanation ??
        "Resolved members whose current snapshot reports team_level = senior; this is not a graduation proxy.",
      icon: BadgeCheck,
      iconClassName: "text-on-surface-variant",
    },
  ];

  return (
    <div className="space-y-3">
      {status !== "ready" ? (
        <p className="text-body-sm text-on-surface-variant" role="status">
          {unavailableExplanation}
        </p>
      ) : null}
      <section
        aria-label="Academy outcomes"
        aria-busy={status === "loading"}
        className="space-y-3"
      >
        <dl className="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-6">
          {outcomeMetrics.map((metric) => (
            <AcademyMetricCard key={metric.id} metric={metric} />
          ))}
        </dl>
      </section>
      <section
        aria-label="Academy context"
        className="border-t border-outline-variant pt-3"
      >
        <dl className="grid grid-cols-2 gap-x-6 gap-y-3 sm:grid-cols-3">
          {contextMetrics.map((metric) => (
            <AcademyContextMetric key={metric.id} metric={metric} />
          ))}
        </dl>
      </section>
    </div>
  );
}

function AcademyMetricCard({ metric }: { metric: Metric }) {
  const Icon = metric.icon;

  return (
    <div className="min-w-0 rounded-lg border border-outline-variant bg-surface-container-high p-4">
      <dt className="flex items-start justify-between gap-3 text-label-md text-on-surface-variant uppercase">
        <span className="pt-1">{metric.label}</span>
        <span className="grid size-8 shrink-0 place-items-center rounded-md bg-surface-container-highest">
          <Icon aria-hidden className={`size-5 ${metric.iconClassName}`} />
        </span>
      </dt>
      <dd
        data-testid={`academy-stat-${metric.id}`}
        className="mt-3 font-mono text-mono-lg text-on-surface tabular-nums"
      >
        {metric.value}
        <p className="mt-2 font-sans text-body-sm font-normal text-on-surface-variant normal-case">
          {metric.explanation}
        </p>
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
          className={`size-4 shrink-0 ${metric.iconClassName}`}
        />
        <span className="min-w-0 text-label-sm text-on-surface-variant uppercase">
          {metric.label}
        </span>
      </dt>
      <dd
        data-testid={`academy-stat-${metric.id}`}
        className="mt-1 min-w-0 pl-6 font-mono text-mono-md text-on-surface tabular-nums"
      >
        {metric.value}
        <p className="mt-0.5 font-sans text-label-sm font-normal text-on-surface-variant normal-case">
          {metric.explanation}
        </p>
      </dd>
    </div>
  );
}

function formatNullableCount(value: number | null) {
  return value === null ? "—" : formatCount(value);
}
