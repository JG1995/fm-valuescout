import { DatabaseZap } from "lucide-react";
import type { KeyboardEvent } from "react";
import { useId, useState } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import { formatLongGameDate } from "@/utils/format";
import {
  MONEYBALL_METRIC_CATEGORIES,
  MONEYBALL_METRICS,
} from "@/utils/moneyball-metrics";
import { positionFamilyLabel } from "@/utils/position-families";
import type {
  MoneyballFamilyComparison,
  MoneyballProfile,
} from "../types/moneyball-profile";
import { MoneyballMetricValue } from "./moneyball-metric-value";

function formattedNumber(value: number | null) {
  return value === null ? "—" : value.toLocaleString("en-GB");
}

function MoneyballTabs({
  activeId,
  onChange,
  idPrefix,
}: {
  activeId: string;
  onChange: (id: string) => void;
  idPrefix: string;
}) {
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = MONEYBALL_METRIC_CATEGORIES.findIndex(
      (category) => category.id === activeId,
    );
    if (index < 0) return;

    let nextIndex = index;
    if (event.key === "ArrowRight" || event.key === "ArrowDown") {
      nextIndex = (index + 1) % MONEYBALL_METRIC_CATEGORIES.length;
    } else if (event.key === "ArrowLeft" || event.key === "ArrowUp") {
      nextIndex =
        (index - 1 + MONEYBALL_METRIC_CATEGORIES.length) %
        MONEYBALL_METRIC_CATEGORIES.length;
    } else if (event.key === "Home") {
      nextIndex = 0;
    } else if (event.key === "End") {
      nextIndex = MONEYBALL_METRIC_CATEGORIES.length - 1;
    } else {
      return;
    }

    event.preventDefault();
    const next = MONEYBALL_METRIC_CATEGORIES[nextIndex].id;
    onChange(next);
    document.getElementById(`${idPrefix}-tab-${next}`)?.focus();
  };

  return (
    <div
      role="tablist"
      aria-label="Moneyball metric categories"
      className="flex flex-wrap rounded-md bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {MONEYBALL_METRIC_CATEGORIES.map((category) => {
        const selected = category.id === activeId;
        return (
          <button
            key={category.id}
            id={`${idPrefix}-tab-${category.id}`}
            type="button"
            role="tab"
            aria-selected={selected}
            aria-controls={`${idPrefix}-panel-${category.id}`}
            tabIndex={selected ? 0 : -1}
            className={
              selected
                ? "h-8 cursor-pointer rounded-sm bg-surface-container-lowest px-3 text-label-md text-on-surface focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary"
                : "h-8 cursor-pointer rounded-sm px-3 text-label-md text-on-surface-variant hover:text-on-surface focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary"
            }
            onClick={() => onChange(category.id)}
          >
            {category.title}
          </button>
        );
      })}
    </div>
  );
}

/** The in-game date of the save the Moneyball stats came from, noting a different snapshot date. */
export function MoneyballStatsAsOf({
  moneyballStatsGameDate,
  snapshotGameDate,
}: {
  moneyballStatsGameDate: string | null;
  snapshotGameDate: string | null;
}) {
  if (moneyballStatsGameDate === null) return null;
  const snapshotDateDiffers =
    snapshotGameDate !== null && snapshotGameDate !== moneyballStatsGameDate;
  return (
    <p className="shrink-0 text-body-sm text-on-surface-variant">
      Stats as of {formatLongGameDate(moneyballStatsGameDate)}
      {snapshotDateDiffers
        ? ` · Snapshot dated ${formatLongGameDate(snapshotGameDate)}`
        : null}
    </p>
  );
}

/**
 * Whom the player's percentiles rank him against, with a switch between the
 * families he can be compared in, and notes on short samples and empty groups.
 */
function ComparisonSummary({
  comparisons,
  selected,
  onSelect,
  minutes,
  minimumMinutes,
}: {
  comparisons: readonly MoneyballFamilyComparison[];
  selected: MoneyballFamilyComparison | undefined;
  onSelect: (family: string) => void;
  minutes: number | null;
  minimumMinutes: number;
}) {
  if (selected === undefined) {
    return (
      <p
        role="status"
        className="shrink-0 text-body-sm text-on-surface-variant"
      >
        Percentile scores unavailable: this player is rated at no position.
      </p>
    );
  }
  const label = positionFamilyLabel(selected.family);
  const peers = `${selected.peers.toLocaleString("en-GB")} ${
    selected.peers === 1 ? "peer" : "peers"
  }`;
  return (
    <div className="flex shrink-0 flex-col gap-1 text-body-sm text-on-surface-variant">
      <p
        className="flex flex-wrap items-center gap-x-2"
        title={`Peers have ${minimumMinutes}+ minutes this season and a rating of 16 or higher at the family's positions.`}
      >
        {comparisons.length > 1 ? (
          <label className="flex items-center gap-2">
            Percentiles vs
            <select
              className="h-7 rounded-md border border-outline bg-surface-container-high px-1.5 text-body-sm text-on-surface"
              value={selected.family}
              onChange={(event) => onSelect(event.target.value)}
            >
              {comparisons.map((comparison) => (
                <option key={comparison.family} value={comparison.family}>
                  {positionFamilyLabel(comparison.family)}
                </option>
              ))}
            </select>
          </label>
        ) : (
          <span>Percentiles vs {label.toLowerCase()}</span>
        )}
        <span>· {peers}</span>
      </p>
      {selected.peers === 0 ? (
        <p role="status">
          Not enough data: no {label.toLowerCase()} have {minimumMinutes}{" "}
          minutes yet.
        </p>
      ) : null}
      {(minutes ?? 0) < minimumMinutes ? (
        <p>
          Short sample: under {minimumMinutes} minutes, so this player is ranked
          against the peers without counting among them.
        </p>
      ) : null}
    </div>
  );
}

export function MoneyballProfilePanel({
  profile,
  moneyballStatsGameDate,
  snapshotGameDate,
}: {
  profile: MoneyballProfile;
  moneyballStatsGameDate: string | null;
  snapshotGameDate: string | null;
}) {
  const idPrefix = `moneyball-${useId().replaceAll(":", "")}`;
  const [categoryId, setCategoryId] = useState(
    MONEYBALL_METRIC_CATEGORIES[0].id,
  );
  const [family, setFamily] = useState<string | null>(null);

  if (profile.state === "noData") {
    return (
      <Panel title="Moneyball">
        <EmptyState
          icon={DatabaseZap}
          title="No Moneyball data for this player"
        >
          This player has no Moneyball stats in the current snapshot. Moneyball
          stats come from the FM save file linked in Settings. Use Refresh stats
          in Moneyball Search to read the latest save.
        </EmptyState>
      </Panel>
    );
  }

  if (profile.state === "needsReimport") {
    return (
      <Panel title="Moneyball">
        <EmptyState icon={DatabaseZap} title="Refresh Moneyball stats">
          These stats were saved before percentile scores were available. Use
          Refresh stats in Moneyball Search to read the linked FM save file
          again.
        </EmptyState>
      </Panel>
    );
  }

  const category = MONEYBALL_METRIC_CATEGORIES.find(
    (candidate) => candidate.id === categoryId,
  );
  if (!category) return null;
  const comparison =
    profile.comparisons.find((candidate) => candidate.family === family) ??
    profile.comparisons[0];

  return (
    <Panel
      title="Moneyball"
      className="flex min-h-0 w-full flex-col [&>div:last-child]:min-h-0 [&>div:last-child]:flex-1"
    >
      <div className="flex h-full min-h-0 flex-col gap-2">
        <dl className="grid shrink-0 grid-cols-[repeat(auto-fit,minmax(144px,1fr))] gap-x-4 gap-y-2 border-b border-outline-variant pb-2 text-body-md">
          <div>
            <dt className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
              Starts
            </dt>
            <dd className="font-mono text-mono-md text-on-surface tabular-nums">
              {formattedNumber(profile.starts)}
            </dd>
          </div>
          <div>
            <dt className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
              Sub appearances
            </dt>
            <dd className="font-mono text-mono-md text-on-surface tabular-nums">
              {formattedNumber(profile.substituteAppearances)}
            </dd>
          </div>
          <div>
            <dt className="text-label-sm text-on-surface-variant uppercase tracking-[0.08em]">
              Minutes
            </dt>
            <dd className="font-mono text-mono-md text-on-surface tabular-nums">
              {formattedNumber(profile.minutes)}
            </dd>
          </div>
        </dl>
        <ComparisonSummary
          comparisons={profile.comparisons}
          selected={comparison}
          onSelect={setFamily}
          minutes={profile.minutes}
          minimumMinutes={profile.minimumMinutes}
        />
        {profile.borrowedStats ? (
          <p className="shrink-0 text-body-sm text-on-surface-variant">
            Stats as of {formatLongGameDate(profile.borrowedStats.statsDate)},
            from a kept snapshot:{" "}
            {formattedNumber(profile.borrowedStats.currentMinutes)} minutes this
            season is under {profile.minimumMinutes}.
          </p>
        ) : (
          <MoneyballStatsAsOf
            moneyballStatsGameDate={moneyballStatsGameDate}
            snapshotGameDate={snapshotGameDate}
          />
        )}
        <div className="shrink-0 p-0.5">
          <MoneyballTabs
            activeId={category.id}
            onChange={setCategoryId}
            idPrefix={idPrefix}
          />
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto pr-1">
          {MONEYBALL_METRIC_CATEGORIES.map((candidate) => (
            <div
              key={candidate.id}
              id={`${idPrefix}-panel-${candidate.id}`}
              role="tabpanel"
              aria-labelledby={`${idPrefix}-tab-${candidate.id}`}
              hidden={candidate.id !== category.id}
            >
              <dl className="grid min-w-0 grid-cols-[repeat(auto-fit,minmax(240px,1fr))] gap-x-4 [&_dt]:whitespace-normal">
                {candidate.metricIds.map((metricId) => {
                  const metric = MONEYBALL_METRICS_BY_ID.get(metricId);
                  if (!metric) return null;
                  return (
                    <MoneyballMetricValue
                      key={metric.id}
                      metric={metric}
                      value={profile.statistics[metric.id] ?? null}
                      score={comparison?.percentiles[metric.id] ?? null}
                    />
                  );
                })}
              </dl>
            </div>
          ))}
        </div>
      </div>
    </Panel>
  );
}

const MONEYBALL_METRICS_BY_ID = new Map(
  MONEYBALL_METRICS.map((metric) => [metric.id, metric] as const),
);
