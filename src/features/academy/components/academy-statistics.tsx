import { formatCount, formatMoney } from "@/utils/format";
import type { AcademyStatistics as AcademyStatisticsData } from "../utils/academy-statistics";

type AcademyStatisticsProps = {
  trackedPlayers: number;
  classes?: number;
  statistics: AcademyStatisticsData;
  status?: AcademyStatisticsStatus;
};

export type AcademyStatisticsStatus = "ready" | "loading" | "error";

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
  const metrics = [
    ...(classes === undefined
      ? []
      : [
          {
            label: "Classes",
            value: formatCount(classes),
            explanation: "Saved Class of YYYY cohorts.",
          },
        ]),
    {
      label: "Tracked players",
      value: formatCount(trackedPlayers),
      explanation: "Players retained in Academy classes for this save.",
    },
    {
      label: "Reported senior players",
      value: formatNullableCount(statistics.reportedSeniorPlayers),
      explanation:
        unavailableExplanation ??
        "Resolved members whose current snapshot reports team_level = senior; this is not a graduation proxy.",
    },
    {
      label: "Graduates",
      value: formatNullableCount(statistics.graduates),
      explanation:
        unavailableExplanation ??
        (statistics.graduates === null
          ? "Graduate data is not available until the memory reader exposes senior league appearances."
          : "Players with at least one reported senior league appearance."),
    },
    {
      label: "Goals",
      value: formatNullableCount(statistics.goals),
      explanation:
        unavailableExplanation ??
        (statistics.goals === null
          ? "Career goals are not available from the current memory reader."
          : "Reported career goals for tracked players."),
    },
    {
      label: "Assists",
      value: formatNullableCount(statistics.assists),
      explanation:
        unavailableExplanation ??
        (statistics.assists === null
          ? "Career assists are not available from the current memory reader."
          : "Reported career assists for tracked players."),
    },
    {
      label: "International caps",
      value: formatNullableCount(statistics.internationalCaps),
      explanation:
        unavailableExplanation ??
        (statistics.internationalCaps === null
          ? "International caps are not available from the current memory reader."
          : "Reported international caps for tracked players."),
    },
    {
      label: "Sale income",
      value:
        statistics.saleFeeGbp === null
          ? "—"
          : formatMoney(statistics.saleFeeGbp),
      explanation:
        unavailableExplanation ??
        (statistics.saleFeeGbp === null
          ? "Sale fees are not available from the current memory reader."
          : "Reported sale fees for tracked players."),
    },
    {
      label: "Released players",
      value: formatNullableCount(statistics.releasedPlayers),
      explanation:
        unavailableExplanation ??
        (statistics.releasedPlayers === null
          ? "Release outcomes are not available from the current memory reader."
          : "Reported released players in this cohort."),
    },
  ];

  return (
    <section aria-label="Academy statistics" aria-busy={status === "loading"}>
      {status !== "ready" ? (
        <p className="mb-3 text-body-sm text-on-surface-variant" role="status">
          {unavailableExplanation}
        </p>
      ) : null}
      <dl className="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
        {metrics.map((metric) => (
          <div
            key={metric.label}
            className="min-w-0 rounded-lg border border-outline-variant bg-surface-container-high p-3"
          >
            <dt className="truncate text-label-sm text-on-surface-variant uppercase">
              {metric.label}
            </dt>
            <dd
              data-testid={`academy-stat-${metric.label.toLowerCase().replaceAll(" ", "-")}`}
              className="mt-1 font-mono text-mono-md text-on-surface tabular-nums"
            >
              {metric.value}
            </dd>
            <p className="mt-1 text-label-sm text-on-surface-variant">
              {metric.explanation}
            </p>
          </div>
        ))}
      </dl>
    </section>
  );
}

function formatNullableCount(value: number | null) {
  return value === null ? "—" : formatCount(value);
}
