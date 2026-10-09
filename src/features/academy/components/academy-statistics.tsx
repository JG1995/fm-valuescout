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
          ? "Graduate data is unavailable until career appearances have been imported for every tracked player."
          : "Players with at least one reported all-time career appearance."),
      icon: GraduationCap,
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
    },
    {
      id: "goals",
      label: "Goals",
      value: formatNullableCount(statistics.goals),
      explanation:
        unavailableExplanation ??
        (statistics.goals === null
          ? "Career goals have not been imported for every tracked player."
          : "Reported career goals for tracked players."),
      icon: Goal,
    },
    {
      id: "assists",
      label: "Assists",
      value: formatNullableCount(statistics.assists),
      explanation:
        unavailableExplanation ??
        (statistics.assists === null
          ? "Career assists have not been imported for every tracked player."
          : "Reported career assists for tracked players."),
      icon: Handshake,
    },
    {
      id: "international-caps",
      label: "International caps",
      value: formatNullableCount(statistics.internationalCaps),
      explanation:
        unavailableExplanation ??
        (statistics.internationalCaps === null
          ? "International caps have not been imported for every tracked player."
          : "Reported international caps for tracked players."),
      icon: Globe2,
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
          },
        ]),
    {
      id: "tracked-players",
      label: "Tracked players",
      value: formatCount(trackedPlayers),
      explanation: "Players retained in Academy classes for this save.",
      icon: UsersRound,
    },
    {
      id: "reported-senior-players",
      label: "Reported senior players",
      value: formatNullableCount(statistics.reportedSeniorPlayers),
      explanation:
        unavailableExplanation ??
        "Resolved members whose current snapshot reports team_level = senior; this is not a graduation proxy.",
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
        <dl className="grid grid-cols-2 gap-4 sm:grid-cols-3 xl:grid-cols-6">
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
        <span>{metric.label}</span>
      </dt>
      <dd
        data-testid={`academy-stat-${metric.id}`}
        className="mt-1 font-mono text-mono-lg text-on-surface tabular-nums"
      >
        {metric.value}
        <p className="mt-1 font-sans text-body-sm font-normal text-on-surface-variant normal-case">
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
          className="size-4 shrink-0 text-on-surface-variant"
          strokeWidth={1.5}
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
        <p className="mt-1 font-sans text-body-sm font-normal text-on-surface-variant normal-case">
          {metric.explanation}
        </p>
      </dd>
    </div>
  );
}

function formatNullableCount(value: number | null) {
  return value === null ? "—" : formatCount(value);
}
