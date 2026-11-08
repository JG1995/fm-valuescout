import { type FormEvent, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { PlayerMetricPicker } from "@/components/ui/player-metric-picker";
import { useAnchoredPopover } from "@/components/ui/use-anchored-popover";
import { isPercentileOperator } from "@/utils/moneyball-search-metrics";
import type { FilterRule } from "../types/filter-rule";
import type { ComparisonPool, SearchView } from "../types/search-view";
import {
  defaultOperatorForField,
  defaultValueForRule,
  filterFieldsForView,
  getFilterField,
  isFilterRuleComplete,
} from "../utils/filter-registry";
import { formatFilterTagLabel } from "../utils/format-filter-label";
import { searchTableGroups } from "../utils/search-table-groups";
import {
  FilterConditionFields,
  filterValueError,
} from "./filter-condition-fields";

/** The raw operator's percentile counterpart, and back. */
const TO_PERCENTILE: Record<string, string> = {
  gte: "pct_gte",
  lte: "pct_lte",
  between: "pct_between",
  eq: "pct_gte",
};
const TO_VALUE: Record<string, string> = {
  pct_gte: "gte",
  pct_lte: "lte",
  pct_between: "between",
};

/**
 * Edits one filter rule in a popover anchored to its chip or to Add filter.
 * The draft stays local; Add or Apply (or Enter) hands back one complete rule,
 * and Cancel, Escape, or a click outside discards it.
 */
export function FilterRulePopover({
  anchor,
  rule,
  isNew,
  view,
  comparisonPool,
  onApply,
  onRemove,
  onClose,
}: {
  anchor: HTMLElement | null;
  rule: FilterRule;
  isNew: boolean;
  view: SearchView;
  comparisonPool?: ComparisonPool;
  onApply: (rule: FilterRule) => void;
  onRemove?: () => void;
  onClose: () => void;
}) {
  const [draft, setDraft] = useState(rule);
  const [showError, setShowError] = useState(false);
  const [focusRequest, setFocusRequest] = useState<"value" | "invalid" | null>(
    rule.field === "" ? null : "value",
  );
  const formRef = useRef<HTMLFormElement | null>(null);
  const { popoverRef, popover } = useAnchoredPopover<HTMLElement>(true, {
    anchor,
    maxHeight: 560,
    minWidth: 360,
  });
  // The popover is hidden until the hook's effect shows it, so the contents
  // (and the field picker that opens on mount) render one pass later.
  const [shown, setShown] = useState(false);
  useEffect(() => setShown(true), []);
  const field = getFilterField(draft.field, view);
  const scaleSwitch =
    view === "moneyball" && field !== undefined && "metric" in field;
  const percentile = isPercentileOperator(draft.op);
  const scored =
    view === "moneyball" &&
    (percentile || (field !== undefined && "role" in field && field.role));

  useEffect(() => {
    if (!shown || focusRequest === null) {
      return;
    }
    const selector =
      focusRequest === "invalid"
        ? '[aria-invalid="true"]'
        : "[data-filter-value] input, [data-filter-value] select";
    formRef.current?.querySelector<HTMLElement>(selector)?.focus();
    setFocusRequest(null);
  }, [focusRequest, shown]);

  useEffect(() => {
    const handlePointerDown = (event: PointerEvent) => {
      const path = event.composedPath();
      const form = formRef.current;
      if (form && !path.includes(form) && !(anchor && path.includes(anchor))) {
        onClose();
      }
    };
    document.addEventListener("pointerdown", handlePointerDown, true);
    return () =>
      document.removeEventListener("pointerdown", handlePointerDown, true);
  }, [anchor, onClose]);

  const chooseField = (fieldId: string) => {
    const op = defaultOperatorForField(fieldId, view);
    setDraft({
      ...draft,
      field: fieldId,
      op,
      value: defaultValueForRule(fieldId, op, view),
    });
    setShowError(false);
    setFocusRequest("value");
  };
  const switchScale = (toPercentile: boolean) => {
    if (toPercentile === percentile) {
      return;
    }
    const op = (toPercentile ? TO_PERCENTILE : TO_VALUE)[draft.op] ?? draft.op;
    setDraft({
      ...draft,
      op,
      value: defaultValueForRule(draft.field, op, view),
    });
    setShowError(false);
  };
  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (!field || !isFilterRuleComplete(draft, view)) {
      setShowError(true);
      setFocusRequest("invalid");
      return;
    }
    onApply(draft);
  };

  const title = isNew
    ? "Add filter"
    : `Edit filter ${formatFilterTagLabel(rule, view)}`;

  return (
    <form
      ref={(element) => {
        formRef.current = element;
        popoverRef.current = element as HTMLDivElement | null;
      }}
      popover={popover}
      aria-label={title}
      className="absolute z-50 m-0 w-[360px] space-y-3 overflow-y-auto rounded-lg border border-outline-variant bg-surface-container-highest p-3 text-left shadow-overlay"
      onSubmit={submit}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          onClose();
        }
      }}
    >
      {shown ? (
        <>
          <PlayerMetricPicker
            label="Field"
            metrics={filterFieldsForView(view)}
            groups={searchTableGroups(view)}
            value={draft.field}
            placeholder="Choose a field"
            defaultOpen={draft.field === ""}
            onChange={chooseField}
          />
          {field ? (
            <>
              {scaleSwitch ? (
                <fieldset className="m-0 flex items-center gap-2 border-0 p-0">
                  <legend className="sr-only">Compare by</legend>
                  <span
                    aria-hidden
                    className="text-label-md text-on-surface-variant"
                  >
                    Compare by
                  </span>
                  <span className="inline-flex rounded-md border border-outline bg-surface-container-high p-0.5">
                    {[
                      { id: "value", label: "Value", active: !percentile },
                      {
                        id: "percentile",
                        label: "Percentile",
                        active: percentile,
                      },
                    ].map((option) => (
                      <button
                        key={option.id}
                        type="button"
                        aria-pressed={option.active}
                        className={
                          option.active
                            ? "h-7 rounded-sm bg-primary px-3 text-label-md text-on-primary"
                            : "h-7 rounded-sm px-3 text-label-md text-on-surface-variant hover:text-on-surface"
                        }
                        onClick={() => switchScale(option.id === "percentile")}
                      >
                        {option.label}
                      </button>
                    ))}
                  </span>
                </fieldset>
              ) : null}
              <div className="space-y-3">
                <FilterConditionFields
                  rule={draft}
                  field={field}
                  view={view}
                  error={
                    showError ? filterValueError(draft, field, view) : undefined
                  }
                  operators={
                    scaleSwitch
                      ? field.operators.filter(
                          (operator) =>
                            isPercentileOperator(operator.id) === percentile,
                        )
                      : field.operators
                  }
                  onChange={(next) => setDraft(next)}
                />
              </div>
              {scored ? (
                <p className="text-body-sm text-on-surface-variant">
                  {comparisonPool === "fullCsv"
                    ? "Scored against all players with stats."
                    : "Scored against the players matching your other filters."}{" "}
                  Change this with Percentiles vs in the toolbar.
                </p>
              ) : null}
            </>
          ) : showError ? (
            <p className="text-body-sm text-error" role="alert">
              Choose a field first.
            </p>
          ) : null}
        </>
      ) : null}
      <div className="flex items-center justify-between gap-2 pt-1">
        {onRemove ? (
          <Button variant="ghost" onClick={onRemove}>
            Remove
          </Button>
        ) : (
          <span />
        )}
        <div className="flex gap-2">
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button type="submit">{isNew ? "Add" : "Apply"}</Button>
        </div>
      </div>
    </form>
  );
}
