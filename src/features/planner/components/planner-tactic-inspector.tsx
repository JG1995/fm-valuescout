import { X } from "lucide-react";
import { type KeyboardEvent, type ReactNode, useId } from "react";
import { SelectField } from "@/components/ui/field/select-field";
import {
  type MetricPickerMetric,
  PlayerMetricPicker,
} from "@/components/ui/player-metric-picker";
import { Tooltip } from "@/components/ui/tooltip/tooltip";
import { cn } from "@/utils/cn";
import {
  MONEYBALL_METRIC_CATEGORIES,
  MONEYBALL_METRICS,
  moneyballMetric,
} from "@/utils/moneyball-metrics";
import {
  positionFamilyLabel,
  positionFamilyOf,
} from "@/utils/position-families";
import { orderedTacticPositions } from "@/utils/position-order";
import type {
  RecruitmentFilter,
  TacticLane,
  TacticOptions,
} from "../types/tactic";
import {
  basePosition,
  linkedPositionDescription,
  phasePosition,
  phasePositionLabel,
  phaseRoleId,
  rolesForPhase,
  TACTIC_PHASES,
  type TacticPhase,
} from "../utils/tactic-editor";

type PlannerTacticInspectorProps = {
  selectedLane: TacticLane;
  lanes: TacticLane[];
  options: TacticOptions;
  phases: TacticPhase[];
  disabled: boolean;
  onWeightChange: (ipWeight: number) => void;
  onPreferredFootChange: (preferredFoot: TacticLane["preferredFoot"]) => void;
  onFootPreferenceChange: (
    footPreference: TacticLane["footPreference"],
  ) => void;
  onPositionChange: (phase: TacticPhase, position: string) => void;
  onRoleChange: (phase: TacticPhase, roleId: string) => void;
  onCustomMetricsChange: (customMetricKeys: string[]) => void;
  onRecruitmentFiltersChange: (filters: RecruitmentFilter[]) => void;
  renderRecruitmentFilters: RenderRecruitmentFilters;
  tab: SlotTab;
  onTabChange: (tab: SlotTab) => void;
};

const MAX_SLOT_METRICS = 5;

const SLOT_METRIC_OPTIONS: readonly MetricPickerMetric[] =
  MONEYBALL_METRICS.map((metric) => ({
    id: metric.id,
    label: metric.label,
    category: MONEYBALL_METRIC_CATEGORIES.find(
      (category) => category.id === metric.categoryId,
    )?.title,
  }));

function nextWeight(value: number, key: string): number | null {
  if (key === "ArrowRight" || key === "ArrowUp") {
    return Math.min(100, value + 1);
  }
  if (key === "ArrowLeft" || key === "ArrowDown") {
    return Math.max(0, value - 1);
  }
  if (key === "Home") {
    return 0;
  }
  if (key === "End") {
    return 100;
  }
  return null;
}

function PhaseControls({
  phase,
  lane,
  lanes,
  options,
  onPositionChange,
  onRoleChange,
}: {
  phase: TacticPhase;
  lane: TacticLane;
  lanes: TacticLane[];
  options: TacticOptions;
  onPositionChange: (position: string) => void;
  onRoleChange: (roleId: string) => void;
}) {
  const position = phasePosition(lane, phase);
  const positionLabel = phasePositionLabel(lane, phase, lanes);
  const roleId = phaseRoleId(lane, phase);
  const roles = rolesForPhase(options, phase, position);
  const selectedRoleIsCompatible = roles.some((role) => role.roleId === roleId);
  const { label, shortLabel } = TACTIC_PHASES[phase];
  const placements = orderedTacticPositions(
    options.placements.includes(position)
      ? options.placements
      : [position, ...options.placements],
  );

  return (
    <fieldset
      aria-label={`${label} settings`}
      className="grid grid-cols-2 gap-3"
    >
      <SelectField
        label={`${shortLabel} ${positionLabel} position`}
        value={position}
        onChange={(event) => onPositionChange(event.target.value)}
      >
        {placements.map((placement) => (
          <option key={placement} value={placement}>
            {placement}
          </option>
        ))}
      </SelectField>
      <SelectField
        label={`${shortLabel} ${positionLabel} role`}
        value={selectedRoleIsCompatible ? roleId : ""}
        disabled={roles.length === 0}
        onChange={(event) => onRoleChange(event.target.value)}
      >
        <option value="">Choose a compatible role</option>
        {roles.map((role) => (
          <option key={role.roleId} value={role.roleId}>
            {role.displayName}
          </option>
        ))}
      </SelectField>
      {roles.length === 0 ? (
        <p className="col-span-2 text-body-sm text-warning">
          No {shortLabel} roles support this position.
        </p>
      ) : null}
    </fieldset>
  );
}

function SlotMetricControls({
  metricKeys,
  onChange,
}: {
  metricKeys: string[];
  onChange: (metricKeys: string[]) => void;
}) {
  const available = SLOT_METRIC_OPTIONS.filter(
    (metric) => !metricKeys.includes(metric.id),
  );

  return (
    <>
      {metricKeys.length === 0 ? (
        <p className="text-body-sm text-on-surface-variant">
          No metrics chosen.
        </p>
      ) : (
        <ol className="space-y-1">
          {metricKeys.map((key) => {
            const { label } = moneyballMetric(key);
            return (
              <li
                key={key}
                className="flex items-center justify-between gap-2 text-body-sm text-on-surface"
              >
                <span className="min-w-0 truncate">{label}</span>
                <button
                  type="button"
                  aria-label={`Remove ${label}`}
                  className="shrink-0 rounded-full p-0.5 text-on-surface-variant hover:bg-surface-container-highest hover:text-on-surface focus-visible:outline-2 focus-visible:outline-primary disabled:pointer-events-none disabled:opacity-50"
                  onClick={() =>
                    onChange(
                      metricKeys.filter((candidate) => candidate !== key),
                    )
                  }
                >
                  <X aria-hidden="true" size={14} />
                </button>
              </li>
            );
          })}
        </ol>
      )}
      {metricKeys.length < MAX_SLOT_METRICS ? (
        <PlayerMetricPicker
          label="Add Moneyball metric"
          searchLabel="Search Moneyball metrics"
          metrics={available}
          value=""
          placeholder="Choose a metric"
          onChange={(key) => onChange([...metricKeys, key])}
        />
      ) : null}
    </>
  );
}

export type SlotTab = "roles" | "scoring" | "recruitment";

const SLOT_TABS: readonly SlotTab[] = ["roles", "scoring", "recruitment"];

const SLOT_TAB_LABELS: Record<SlotTab, string> = {
  roles: "Roles",
  scoring: "Scoring",
  recruitment: "Recruitment",
};

/** Renders the route-provided editor for one slot's recruitment filters. */
export type RenderRecruitmentFilters = (props: {
  filters: RecruitmentFilter[];
  onChange: (filters: RecruitmentFilter[]) => void;
}) => ReactNode;

const MAX_RECRUITMENT_FILTERS = 10;

function SlotTabs({
  tab,
  counts,
  idPrefix,
  onTabChange,
}: {
  tab: SlotTab;
  counts: Partial<Record<SlotTab, number>>;
  idPrefix: string;
  onTabChange: (tab: SlotTab) => void;
}) {
  const focusTab = (next: SlotTab) =>
    document.getElementById(`${idPrefix}-tab-${next}`)?.focus();
  const onKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    const index = SLOT_TABS.indexOf(tab);
    const next =
      event.key === "ArrowRight" || event.key === "ArrowDown"
        ? SLOT_TABS[(index + 1) % SLOT_TABS.length]
        : event.key === "ArrowLeft" || event.key === "ArrowUp"
          ? SLOT_TABS[(index - 1 + SLOT_TABS.length) % SLOT_TABS.length]
          : event.key === "Home"
            ? SLOT_TABS[0]
            : event.key === "End"
              ? SLOT_TABS[SLOT_TABS.length - 1]
              : null;
    if (!next) {
      return;
    }
    event.preventDefault();
    onTabChange(next);
    focusTab(next);
  };

  return (
    <div
      role="tablist"
      aria-label="Slot settings"
      className="grid shrink-0 grid-cols-3 rounded-md bg-surface-container-high p-0.5"
      onKeyDown={onKeyDown}
    >
      {SLOT_TABS.map((id) => {
        const selected = id === tab;
        const count = counts[id] ?? 0;
        return (
          <button
            key={id}
            type="button"
            role="tab"
            id={`${idPrefix}-tab-${id}`}
            aria-selected={selected}
            aria-controls={`${idPrefix}-panel-${id}`}
            tabIndex={selected ? 0 : -1}
            className={cn(
              "inline-flex h-8 cursor-pointer items-center justify-center gap-1.5 rounded-sm px-2 text-label-md transition-colors duration-150 ease-out",
              "focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary",
              selected
                ? "bg-surface-container-lowest text-on-surface"
                : "text-on-surface-variant hover:text-on-surface",
            )}
            onClick={() => onTabChange(id)}
          >
            {SLOT_TAB_LABELS[id]}
            {count > 0 ? (
              <span className="rounded-sm bg-surface-container-highest px-1 font-mono text-mono-sm tabular-nums text-on-surface">
                {count}
              </span>
            ) : null}
          </button>
        );
      })}
    </div>
  );
}

function SectionHeading({
  children,
  count,
  hint,
}: {
  children: ReactNode;
  count?: string;
  /** How the section works, in the heading's tooltip. */
  hint?: ReactNode;
}) {
  const heading = (
    <h3 className="text-label-md text-on-surface-variant">{children}</h3>
  );
  return (
    <div className="flex items-baseline justify-between gap-2">
      {hint ? (
        <Tooltip focusable content={hint}>
          {heading}
        </Tooltip>
      ) : (
        heading
      )}
      {count ? (
        <p className="font-mono text-mono-sm text-on-surface-variant tabular-nums">
          {count}
        </p>
      ) : null}
    </div>
  );
}

/**
 * The peers the slot's metrics and filters rank a player among: the position
 * families of its two positions, such as "wingers and wide midfielders".
 */
function slotPeers(lane: TacticLane): string {
  const families = [
    ...new Set(
      [basePosition(lane.ipPosition), basePosition(lane.oopPosition)].map(
        (position) => positionFamilyOf(position),
      ),
    ),
  ].filter((family) => family !== null);
  return families
    .map((family) => positionFamilyLabel(family).toLowerCase())
    .join(" and ");
}

export function PlannerTacticInspector({
  selectedLane,
  lanes,
  options,
  phases,
  disabled,
  tab,
  onTabChange,
  onWeightChange,
  onPreferredFootChange,
  onFootPreferenceChange,
  onPositionChange,
  onRoleChange,
  onCustomMetricsChange,
  onRecruitmentFiltersChange,
  renderRecruitmentFilters,
}: PlannerTacticInspectorProps) {
  const weightId = useId();
  const headingId = useId();
  const tabIdPrefix = useId();
  const weight = Math.round(selectedLane.ipWeight * 100);
  const panelProps = (id: SlotTab) => ({
    id: `${tabIdPrefix}-panel-${id}`,
    role: "tabpanel" as const,
    "aria-labelledby": `${tabIdPrefix}-tab-${id}`,
    hidden: id !== tab,
    className: "space-y-3 py-3",
  });

  return (
    <section
      className="flex h-full min-h-0 flex-col gap-3 border-l border-outline-variant pl-4"
      aria-labelledby={headingId}
    >
      <div className="flex shrink-0 flex-col gap-0.5">
        <h2
          id={headingId}
          className="text-body-md font-semibold text-on-surface"
        >
          Selected Slot
        </h2>
        <p className="whitespace-pre-line text-body-sm text-on-surface-variant">
          {linkedPositionDescription(selectedLane, lanes, options).replace(
            " / ",
            "\n",
          )}
        </p>
      </div>

      <SlotTabs
        tab={tab}
        counts={{
          scoring: selectedLane.customMetricKeys.length,
          recruitment: selectedLane.recruitmentFilters.length,
        }}
        idPrefix={tabIdPrefix}
        onTabChange={onTabChange}
      />

      <fieldset
        disabled={disabled}
        className="min-h-0 flex-1 overflow-y-auto pr-1"
      >
        <div {...panelProps("roles")}>
          <div className="space-y-1">
            <SectionHeading>Phase Influence</SectionHeading>
            <label
              className="block text-label-md text-on-surface-variant"
              htmlFor={weightId}
            >
              IP/OOP score weight
            </label>
            <input
              id={weightId}
              type="range"
              min="0"
              max="100"
              step="1"
              value={weight}
              aria-label="IP/OOP score weight"
              aria-valuetext={`IP ${weight}%, OOP ${100 - weight}%`}
              className="h-2 w-full cursor-pointer accent-primary focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
              onKeyDown={(event) => {
                const next = nextWeight(weight, event.key);
                if (next === null) {
                  return;
                }
                event.preventDefault();
                onWeightChange(next / 100);
              }}
              onChange={(event) =>
                onWeightChange(Number(event.target.value) / 100)
              }
            />
            <p className="font-mono text-mono-sm text-on-surface tabular-nums">
              IP {weight}% / OOP {100 - weight}%
            </p>
          </div>

          <div className="space-y-2 border-t border-outline-variant pt-3">
            <SectionHeading>Foot Rule</SectionHeading>
            <div className="grid grid-cols-2 gap-3">
              <SelectField
                label="Preferred foot"
                value={selectedLane.preferredFoot}
                onChange={(event) =>
                  onPreferredFootChange(
                    event.target.value as TacticLane["preferredFoot"],
                  )
                }
              >
                <option value="any">Either</option>
                <option value="left">Left</option>
                <option value="right">Right</option>
                <option value="both">Both</option>
              </SelectField>
              <SelectField
                label="Foot preference"
                value={selectedLane.footPreference}
                disabled={selectedLane.preferredFoot === "any"}
                onChange={(event) =>
                  onFootPreferenceChange(
                    event.target.value as TacticLane["footPreference"],
                  )
                }
              >
                <option value="preferred">Preferred</option>
                <option value="strict">Strict</option>
              </SelectField>
            </div>
          </div>

          {phases.map((phase) => (
            <div
              key={phase}
              className="space-y-2 border-t border-outline-variant pt-3"
            >
              <h3 className="flex items-center gap-2 text-label-md text-on-surface-variant">
                <span
                  aria-hidden="true"
                  className={`inline-block size-2 rounded-full ${phase === "ip" ? "bg-chart-2" : "bg-chart-3"}`}
                />
                {phase === "ip"
                  ? "In Possession (IP)"
                  : "Out of Possession (OOP)"}
              </h3>
              <PhaseControls
                phase={phase}
                lane={selectedLane}
                lanes={lanes}
                options={options}
                onPositionChange={(position) =>
                  onPositionChange(phase, position)
                }
                onRoleChange={(roleId) => onRoleChange(phase, roleId)}
              />
            </div>
          ))}
        </div>

        <div {...panelProps("scoring")}>
          <SectionHeading
            count={`${selectedLane.customMetricKeys.length}/${MAX_SLOT_METRICS}`}
            hint={`The mean of each metric's percentile among ${slotPeers(selectedLane)} with 450+ minutes rated 16 or higher. Both phases count; the IP/OOP weight does not apply.`}
          >
            Moneyball Metrics
          </SectionHeading>
          <SlotMetricControls
            metricKeys={selectedLane.customMetricKeys}
            onChange={onCustomMetricsChange}
          />
        </div>

        <div {...panelProps("recruitment")}>
          <SectionHeading
            count={`${selectedLane.recruitmentFilters.length}/${MAX_RECRUITMENT_FILTERS}`}
            hint={`Recommended signings must pass every filter; your own players are never filtered. Percentiles rank among ${slotPeers(selectedLane)} with 450+ minutes rated 16 or higher.`}
          >
            Recruitment Filters
          </SectionHeading>
          {renderRecruitmentFilters({
            filters: selectedLane.recruitmentFilters,
            onChange: onRecruitmentFiltersChange,
          })}
        </div>
      </fieldset>
    </section>
  );
}
