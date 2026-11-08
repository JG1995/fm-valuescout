import { ChevronDown } from "lucide-react";
import { useEffect, useId, useMemo, useRef, useState } from "react";
import type { TableGroupInput } from "@/components/player-table/table-groups";
import {
  fieldClasses,
  fieldLabelClasses,
} from "@/components/ui/field/field-styles";
import { useAnchoredPopover } from "@/components/ui/use-anchored-popover";

export type MetricPickerMetric = {
  id: string;
  label: string;
  category?: string;
  roleFamily?: string;
};

export type MetricPickerGroup = {
  label: string;
  metrics: MetricPickerMetric[];
};

export type MetricPickerProps = {
  label: string;
  metrics: readonly MetricPickerMetric[];
  value: string;
  onChange: (metricId: string) => void;
  disabled?: boolean;
  /** Trigger text while no metric is selected. */
  placeholder?: string;
  /** Open the picker when it mounts, focusing its search field. */
  defaultOpen?: boolean;
  /** Orders sections like the table's header groups. */
  groups?: TableGroupInput;
  /** `inline` renders the choice as a link-like word inside a sentence, with no visible label. */
  variant?: "field" | "inline";
  /** Name of the search box; defaults to "Search {label}s". */
  searchLabel?: string;
};

const CATEGORY_ORDER = [
  "identity",
  "ability-reputation",
  "staff-attributes",
  "visible-attributes",
  "hidden-attributes",
  "personality",
  "club-contract",
  "position-suitability",
  "current-role-scores",
  "potential-role-scores",
] as const;

const CATEGORY_LABELS: Record<(typeof CATEGORY_ORDER)[number], string> = {
  identity: "Identity",
  "club-contract": "Club and contract",
  "ability-reputation": "Ability and reputation",
  "staff-attributes": "Staff attributes",
  "visible-attributes": "Visible attributes",
  "hidden-attributes": "Hidden attributes",
  personality: "Personality",
  "position-suitability": "Position suitability",
  "current-role-scores": "Current role scores",
  "potential-role-scores": "Potential role scores",
};

const ROLE_FAMILY_ORDER = [
  "Goalkeepers",
  "Central defense",
  "Full-back and wing-back",
  "Defensive midfield",
  "Central midfield",
  "Wide midfield and wings",
  "Attacking midfield",
  "Forwards",
] as const;

const POPUP_MAX_HEIGHT = 320;
const POPUP_MIN_WIDTH = 288;

/**
 * Picker sections: categories, with role scores split by family. Given the
 * table's groups, sections follow the header's group order so the picker,
 * Columns, and the header read the same way.
 */
export function groupsForMetrics<TMetric extends MetricPickerMetric>(
  metrics: readonly TMetric[],
  search: string,
  tableGroups?: TableGroupInput,
): { label: string; metrics: TMetric[] }[] {
  const groups = categoryGroups(metrics, search);
  if (!tableGroups) {
    return groups;
  }
  const order = tableGroups.groups.map((group) => group.id);
  const rank = (group: { metrics: TMetric[] }) => {
    const first = group.metrics[0];
    const index = first
      ? order.indexOf(tableGroups.groupForColumn(first.id) ?? "")
      : -1;
    return index === -1 ? order.length : index;
  };
  return [...groups].sort((left, right) => rank(left) - rank(right));
}

function categoryGroups<TMetric extends MetricPickerMetric>(
  metrics: readonly TMetric[],
  search: string,
): { label: string; metrics: TMetric[] }[] {
  const term = search.trim().toLowerCase();
  const matches = metrics.filter((metric) => {
    if (!term) {
      return true;
    }
    return `${metric.label} ${metric.id} ${metric.roleFamily ?? ""}`
      .toLowerCase()
      .includes(term);
  });

  const groups: { label: string; metrics: TMetric[] }[] = [];
  for (const category of CATEGORY_ORDER) {
    const categoryMetrics = matches.filter(
      (metric) => metric.category === category,
    );
    if (categoryMetrics.length === 0) {
      continue;
    }

    if (
      category !== "current-role-scores" &&
      category !== "potential-role-scores"
    ) {
      groups.push({
        label: CATEGORY_LABELS[category],
        metrics: categoryMetrics,
      });
      continue;
    }

    for (const family of ROLE_FAMILY_ORDER) {
      const familyMetrics = categoryMetrics.filter(
        (metric) => metric.roleFamily === family,
      );
      if (familyMetrics.length > 0) {
        groups.push({
          label: `${CATEGORY_LABELS[category]} · ${family}`,
          metrics: familyMetrics,
        });
      }
    }

    const knownFamilies = new Set<string>(ROLE_FAMILY_ORDER);
    const ungroupedMetrics = categoryMetrics.filter(
      (metric) => !metric.roleFamily || !knownFamilies.has(metric.roleFamily),
    );
    if (ungroupedMetrics.length > 0) {
      groups.push({
        label: CATEGORY_LABELS[category],
        metrics: ungroupedMetrics,
      });
    }
  }
  const knownCategories = new Set(CATEGORY_ORDER);
  const extraCategories = Array.from(
    new Set(
      matches
        .map((metric) => metric.category)
        .filter(
          (category): category is string =>
            category !== undefined && !knownCategories.has(category as never),
        ),
    ),
  );
  for (const category of extraCategories) {
    const categoryMetrics = matches.filter(
      (metric) => metric.category === category,
    );
    groups.push({ label: category, metrics: categoryMetrics });
  }

  const uncategorized = matches.filter(
    (metric) => metric.category === undefined,
  );
  if (uncategorized.length > 0) {
    groups.push({ label: "Metrics", metrics: uncategorized });
  }
  return groups;
}

export function MetricPicker({
  label,
  metrics,
  value,
  onChange,
  disabled = false,
  placeholder = "Choose a metric",
  defaultOpen = false,
  groups: tableGroups,
  variant = "field",
  searchLabel: searchLabelOverride,
}: MetricPickerProps) {
  const searchInputId = useId();
  const listboxId = useId();
  const optionIdPrefix = useId();
  const rootRef = useRef<HTMLDivElement>(null);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const activeOptionRef = useRef<HTMLButtonElement>(null);
  const [open, setOpen] = useState(defaultOpen);
  const [search, setSearch] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);
  const selected = metrics.find((metric) => metric.id === value);
  const groups = useMemo(
    () => groupsForMetrics(metrics, search, tableGroups),
    [metrics, search, tableGroups],
  );
  const options = groups.flatMap((group) => group.metrics);
  const activeMetric = options[activeIndex];
  const searchLabel = searchLabelOverride ?? `Search ${label.toLowerCase()}s`;
  const {
    anchorRef: triggerRef,
    popoverRef: popupRef,
    popover,
  } = useAnchoredPopover<HTMLButtonElement>(open, {
    maxHeight: POPUP_MAX_HEIGHT,
    minWidth: POPUP_MIN_WIDTH,
  });

  useEffect(() => {
    if (!open) {
      return;
    }
    setSearch("");
    setActiveIndex(0);
    searchInputRef.current?.focus();
  }, [open]);

  useEffect(() => {
    if (!open) {
      return;
    }
    const handlePointerDown = (event: PointerEvent) => {
      const root = rootRef.current;
      if (root && !event.composedPath().includes(root)) {
        setOpen(false);
      }
    };
    document.addEventListener("pointerdown", handlePointerDown, true);
    return () =>
      document.removeEventListener("pointerdown", handlePointerDown, true);
  }, [open]);

  useEffect(() => {
    if (!activeMetric) {
      return;
    }
    activeOptionRef.current?.scrollIntoView?.({ block: "nearest" });
  }, [activeMetric]);

  const selectMetric = (metric: MetricPickerMetric) => {
    onChange(metric.id);
    closePicker();
  };

  const closePicker = () => {
    setOpen(false);
    triggerRef.current?.focus();
  };

  return (
    <div
      ref={rootRef}
      className={
        variant === "inline" ? "relative inline-block" : "relative space-y-1"
      }
    >
      {variant === "inline" ? null : (
        <span className={fieldLabelClasses}>{label}</span>
      )}
      <button
        ref={triggerRef}
        type="button"
        aria-controls={open ? listboxId : undefined}
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label={`${label}: ${selected?.label ?? placeholder}`}
        title={selected?.label}
        className={
          variant === "inline"
            ? "inline-flex items-center gap-0.5 rounded-sm text-on-surface underline decoration-dotted underline-offset-4 hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary"
            : `${fieldClasses} flex w-full items-center justify-between gap-2 text-left`
        }
        disabled={disabled}
        onClick={() => setOpen((current) => !current)}
      >
        <span className="min-w-0 truncate">
          {selected?.label ?? placeholder}
        </span>
        <ChevronDown
          aria-hidden="true"
          className="shrink-0"
          size={variant === "inline" ? 14 : 16}
        />
      </button>

      {open ? (
        <div
          ref={popupRef}
          popover={popover}
          className="absolute z-50 m-0 flex flex-col overflow-hidden rounded-md border border-outline-variant bg-surface-container-highest p-0 text-left shadow-overlay"
        >
          <div className="border-b border-outline-variant p-2">
            <label className="sr-only" htmlFor={searchInputId}>
              {searchLabel}
            </label>
            <input
              ref={searchInputRef}
              aria-activedescendant={
                activeMetric
                  ? `${optionIdPrefix}-${activeMetric.id}`
                  : undefined
              }
              aria-controls={listboxId}
              aria-expanded
              aria-haspopup="listbox"
              className={`${fieldClasses} w-full`}
              id={searchInputId}
              placeholder={`${searchLabel}…`}
              role="combobox"
              type="search"
              value={search}
              onChange={(event) => {
                setSearch(event.target.value);
                setActiveIndex(0);
              }}
              onKeyDown={(event) => {
                if (event.key === "Escape") {
                  event.preventDefault();
                  event.stopPropagation();
                  closePicker();
                  return;
                }
                if (options.length === 0) {
                  return;
                }
                if (event.key === "ArrowDown") {
                  event.preventDefault();
                  setActiveIndex((index) => (index + 1) % options.length);
                } else if (event.key === "ArrowUp") {
                  event.preventDefault();
                  setActiveIndex(
                    (index) => (index - 1 + options.length) % options.length,
                  );
                } else if (event.key === "Enter" && activeMetric) {
                  event.preventDefault();
                  selectMetric(activeMetric);
                }
              }}
            />
          </div>
          <div
            aria-label={`${label} options`}
            className="min-h-0 overflow-y-auto p-1"
            id={listboxId}
            role="listbox"
          >
            {groups.length === 0 ? (
              <p className="px-2 py-3 text-body-sm text-on-surface-variant">
                No metrics match this search.
              </p>
            ) : (
              groups.map((group) => (
                <fieldset
                  key={group.label}
                  className="m-0 min-w-0 border-0 p-0"
                >
                  <legend className="px-2 pt-2 pb-1 text-label-md text-on-surface-variant uppercase">
                    {group.label}
                  </legend>
                  {group.metrics.map((metric) => {
                    const index = options.findIndex(
                      (option) => option.id === metric.id,
                    );
                    const active = index === activeIndex;
                    return (
                      <button
                        key={metric.id}
                        ref={active ? activeOptionRef : undefined}
                        type="button"
                        aria-selected={metric.id === value}
                        className={`flex w-full rounded-sm px-2 py-1.5 text-left text-body-sm text-on-surface hover:bg-surface-container-high focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-primary ${
                          active ? "bg-surface-container-high" : ""
                        }`}
                        id={`${optionIdPrefix}-${metric.id}`}
                        role="option"
                        onMouseDown={(event) => event.preventDefault()}
                        onMouseEnter={() => setActiveIndex(index)}
                        onClick={() => selectMetric(metric)}
                      >
                        {metric.label}
                      </button>
                    );
                  })}
                </fieldset>
              ))
            )}
          </div>
        </div>
      ) : null}
    </div>
  );
}

/** Compatibility export for the existing player Search and filter callers. */
export const PlayerMetricPicker = MetricPicker;
