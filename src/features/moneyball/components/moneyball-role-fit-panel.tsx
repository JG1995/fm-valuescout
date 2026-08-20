import { ArrowDown, ArrowUp } from "lucide-react";
import { useState } from "react";
import { ProfilePositionPicker } from "@/components/profile-position-picker/profile-position-picker";
import { Panel } from "@/components/ui/panel/panel";
import { ScoreBadge } from "@/components/ui/score-badge/score-badge";
import { formatMissable } from "@/utils/format";
import {
  defaultProfilePosition,
  type PositionFamiliarityMap,
  rolesForScorePosition,
} from "@/utils/profile-position-roles";
import { rolePhaseLabel } from "@/utils/role-phase";
import type {
  MoneyballRoleContribution,
  MoneyballRoleScore,
} from "../types/moneyball-profile";

function formatWeight(weight: number) {
  return `${(weight * 100).toLocaleString("en-GB", {
    maximumFractionDigits: 1,
  })}%`;
}

function formatContribution(value: number | null) {
  return value === null
    ? "Contribution unavailable"
    : `Contribution ${value.toLocaleString("en-GB", {
        maximumFractionDigits: 1,
      })}`;
}

function MetricContribution({
  contribution,
}: {
  contribution: MoneyballRoleContribution;
}) {
  return (
    <div className="grid gap-1 border-t border-outline-variant/70 py-2 sm:grid-cols-[minmax(0,1fr)_auto] sm:gap-x-4">
      <dt className="min-w-0 text-body-sm text-on-surface">
        <span className="block truncate" title={contribution.metricKey}>
          {contribution.sourceLabel}
        </span>
        <span className="block text-[11px] text-on-surface-variant">
          {contribution.metricKey} ·{" "}
          {contribution.direction === "lower"
            ? "Lower is better"
            : "Higher is better"}
        </span>
      </dt>
      <dd className="grid grid-cols-2 gap-x-3 gap-y-1 text-right text-[11px] text-on-surface-variant sm:grid-cols-3">
        <span>Weight {formatWeight(contribution.weight)}</span>
        <span>
          {contribution.percentile === null
            ? "Percentile unavailable"
            : `Percentile ${contribution.percentile}`}
        </span>
        <span>{formatContribution(contribution.weightedContribution)}</span>
      </dd>
    </div>
  );
}

function RoleExplanation({
  role,
  catalogVersion,
}: {
  role: MoneyballRoleScore;
  catalogVersion: number;
}) {
  return (
    <details className="mt-1 rounded-md bg-surface-container-low px-2">
      <summary className="cursor-pointer py-2 text-body-sm text-on-surface focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary">
        <span className="font-medium">{role.displayName}</span>
        <span className="ml-2 text-[11px] text-on-surface-variant">
          {rolePhaseLabel(role.phase)}
        </span>
      </summary>
      <div className="pb-2">
        <p className="pb-1 text-[11px] text-on-surface-variant">
          Catalog v{catalogVersion} · full imported cohort.
        </p>
        {role.score === null ? (
          <p className="pb-1 text-body-sm text-on-surface-variant">
            Score unavailable: one or more metrics are missing.
          </p>
        ) : (
          <p className="pb-1 text-body-sm text-on-surface-variant">
            Weighted percentile score from the full imported Moneyball cohort.
          </p>
        )}
        <dl>
          {role.contributions.map((contribution) => (
            <MetricContribution
              key={contribution.metricKey}
              contribution={contribution}
            />
          ))}
        </dl>
      </div>
    </details>
  );
}

function MoneyballScore({ role }: { role: MoneyballRoleScore }) {
  if (role.score === null) {
    return (
      <span
        role="img"
        aria-label={`${role.displayName} Moneyball score: unavailable`}
        className="font-mono text-mono-sm text-on-surface-variant tabular-nums"
      >
        {formatMissable(null)}
      </span>
    );
  }

  return (
    <ScoreBadge
      score={role.score}
      roleName={`${role.displayName} Moneyball score`}
      variant="card"
    />
  );
}

export function MoneyballRoleFitPanel({
  positions,
  roleScores,
  catalogVersion,
}: {
  positions: PositionFamiliarityMap;
  roleScores: readonly MoneyballRoleScore[];
  catalogVersion: number;
}) {
  const [selectedPosition, setSelectedPosition] = useState(() =>
    defaultProfilePosition(positions, roleScores),
  );
  const [direction, setDirection] = useState<"ascending" | "descending">(
    "descending",
  );
  const roles = rolesForScorePosition(roleScores, selectedPosition, direction);
  const familiarity = positions[selectedPosition];
  const toggleSort = () => {
    setDirection((current) =>
      current === "descending" ? "ascending" : "descending",
    );
  };
  const SortIcon = direction === "ascending" ? ArrowUp : ArrowDown;

  return (
    <Panel
      title="Moneyball role fit"
      actions={
        <span className="text-label-sm text-on-surface-variant">
          Moneyball score
        </span>
      }
      className="flex min-h-0 flex-col"
      contentClassName="flex min-h-0 flex-1 flex-col"
    >
      <section
        aria-label={`Moneyball role fit for ${selectedPosition}`}
        className="grid h-full min-h-0 gap-4 sm:grid-cols-[minmax(180px,0.8fr)_minmax(240px,1.2fr)]"
      >
        <ProfilePositionPicker
          positions={positions}
          selectedPosition={selectedPosition}
          onSelectPosition={setSelectedPosition}
        />
        <div className="flex min-h-0 min-w-0 flex-col">
          <div className="min-h-0 flex-1 overflow-y-auto pr-1">
            <table className="w-full table-fixed border-collapse">
              <caption className="sr-only">
                Moneyball role scores for {selectedPosition}
              </caption>
              <colgroup>
                <col />
                <col className="w-[110px]" />
              </colgroup>
              <thead className="sticky top-0 z-10 bg-surface-container">
                <tr className="border-b border-outline-variant">
                  <th scope="col" className="pb-2 text-left font-normal">
                    <h3 className="text-headline-sm text-on-surface">
                      {selectedPosition}
                    </h3>
                    <p className="text-body-sm text-on-surface-variant">
                      {roles.length} {roles.length === 1 ? "role" : "roles"}
                      {typeof familiarity === "number" && familiarity > 0
                        ? ` · familiarity ${familiarity}`
                        : ""}
                    </p>
                    <span className="sr-only">Role</span>
                  </th>
                  <th
                    scope="col"
                    aria-sort={direction}
                    className="w-[110px] pb-1 text-center align-bottom"
                  >
                    <button
                      type="button"
                      className="inline-flex min-h-8 w-full items-center justify-center gap-1 rounded-md px-1 text-label-sm text-primary transition-colors duration-150 ease-out"
                      onClick={toggleSort}
                    >
                      <span>Moneyball score</span>
                      <SortIcon
                        size={12}
                        strokeWidth={1.5}
                        aria-hidden="true"
                      />
                    </button>
                  </th>
                </tr>
              </thead>
              <tbody>
                {roles.length === 0 ? (
                  <tr>
                    <td
                      colSpan={2}
                      className="h-24 text-center text-body-sm text-on-surface-variant"
                    >
                      No Moneyball roles use this position.
                    </td>
                  </tr>
                ) : null}
                {roles.map((role) => (
                  <tr
                    key={role.roleId}
                    className="border-b border-outline-variant/70 align-top"
                  >
                    <td className="min-w-0 pr-2">
                      <RoleExplanation
                        role={role}
                        catalogVersion={catalogVersion}
                      />
                    </td>
                    <td className="pt-2 text-center">
                      <MoneyballScore role={role} />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>
    </Panel>
  );
}
