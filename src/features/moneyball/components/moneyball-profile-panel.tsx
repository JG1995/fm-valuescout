import { DatabaseZap } from "lucide-react";
import type { KeyboardEvent } from "react";
import { useId, useState } from "react";
import { EmptyState } from "@/components/ui/empty-state/empty-state";
import { Panel } from "@/components/ui/panel/panel";
import {
  MONEYBALL_METRIC_CATEGORIES,
  MONEYBALL_METRICS,
} from "@/utils/moneyball-metrics";
import { orderedPositions } from "@/utils/position-order";
import type { MoneyballProfile } from "../types/moneyball-profile";
import { MoneyballMetricValue } from "./moneyball-metric-value";

function formattedNumber(value: number | null) {
  return value === null ? "—" : value.toLocaleString("en-GB");
}

function contextPrice(profile: Extract<MoneyballProfile, { state: "ready" }>) {
  if (profile.askingPriceKind === "not_for_sale") return "Not for sale";
  if (profile.askingPriceLowerEur === null) return "—";
  const lower = `€${profile.askingPriceLowerEur.toLocaleString("en-GB")}`;
  if (
    profile.askingPriceKind !== "range" ||
    profile.askingPriceUpperEur === null
  ) {
    return lower;
  }
  return `${lower}–€${profile.askingPriceUpperEur.toLocaleString("en-GB")}`;
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
      className="inline-flex max-w-full rounded-full bg-surface-container-high p-0.5"
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
                ? "cursor-pointer rounded-full bg-primary px-3 py-1.5 text-label-md text-on-primary"
                : "cursor-pointer rounded-full px-3 py-1.5 text-label-md text-on-surface-variant hover:text-on-surface"
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

export function MoneyballProfilePanel({
  profile,
}: {
  profile: MoneyballProfile;
}) {
  const idPrefix = `moneyball-${useId().replaceAll(":", "")}`;
  const [categoryId, setCategoryId] = useState(
    MONEYBALL_METRIC_CATEGORIES[0].id,
  );

  if (profile.state === "noData") {
    return (
      <Panel title="Moneyball">
        <EmptyState
          icon={DatabaseZap}
          title="No Moneyball data for this player"
        >
          This player was not included in the current Moneyball import. Import a
          Moneyball CSV from Player Search or My Club Squad to analyse the
          current snapshot.
        </EmptyState>
      </Panel>
    );
  }

  if (profile.state === "needsReimport") {
    return (
      <Panel title="Moneyball">
        <EmptyState icon={DatabaseZap} title="Re-import Moneyball data">
          This import was saved before percentile scores were available.
          Re-import the Moneyball CSV from Player Search or My Club Squad to
          analyse it.
        </EmptyState>
      </Panel>
    );
  }

  const category = MONEYBALL_METRIC_CATEGORIES.find(
    (candidate) => candidate.id === categoryId,
  );
  if (!category) return null;

  return (
    <Panel
      title="Moneyball"
      className="flex min-h-0 flex-col [&>div:last-child]:min-h-0 [&>div:last-child]:flex-1"
    >
      <div className="flex h-full min-h-0 flex-col gap-4">
        <dl className="grid grid-cols-2 gap-x-5 gap-y-2 border-b border-outline-variant pb-4 text-body-md sm:grid-cols-4">
          <div>
            <dt className="text-on-surface-variant">Asking price</dt>
            <dd className="font-mono tabular-nums">{contextPrice(profile)}</dd>
          </div>
          <div>
            <dt className="text-on-surface-variant">Starts</dt>
            <dd className="font-mono tabular-nums">
              {formattedNumber(profile.starts)}
            </dd>
          </div>
          <div>
            <dt className="text-on-surface-variant">Sub appearances</dt>
            <dd className="font-mono tabular-nums">
              {formattedNumber(profile.substituteAppearances)}
            </dd>
          </div>
          <div>
            <dt className="text-on-surface-variant">Minutes</dt>
            <dd className="font-mono tabular-nums">
              {formattedNumber(profile.minutes)}
            </dd>
          </div>
        </dl>
        {profile.comparisonBasis.kind === "available" ? (
          <p className="text-body-sm text-on-surface-variant">
            Natural positions:{" "}
            {orderedPositions(profile.comparisonBasis.naturalPositions).join(
              ", ",
            )}{" "}
            · {profile.comparisonBasis.comparisonPlayerCount} comparison{" "}
            {profile.comparisonBasis.comparisonPlayerCount === 1
              ? "player"
              : "players"}
          </p>
        ) : (
          <p role="status" className="text-body-sm text-on-surface-variant">
            Percentile scores unavailable: this player has no natural position.
          </p>
        )}
        <div className="overflow-x-auto pb-0.5">
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
              <dl className="grid min-w-0 grid-cols-1 gap-x-5 lg:grid-cols-2">
                {candidate.metricIds.map((metricId) => {
                  const metric = MONEYBALL_METRICS_BY_ID.get(metricId);
                  if (!metric) return null;
                  return (
                    <MoneyballMetricValue
                      key={metric.id}
                      metric={metric}
                      value={profile.statistics[metric.id] ?? null}
                      score={
                        profile.comparisonBasis.kind === "available"
                          ? (profile.percentiles?.[metric.id] ?? null)
                          : null
                      }
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
