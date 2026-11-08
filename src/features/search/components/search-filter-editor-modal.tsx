import { Trash2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { fieldLabelClasses } from "@/components/ui/field/field-styles";
import { SelectField } from "@/components/ui/field/select-field";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import { PlayerMetricPicker } from "@/components/ui/player-metric-picker";
import { isPercentileOperator } from "@/utils/moneyball-search-metrics";
import type {
  FilterCombineMode,
  FilterRule,
  FilterValue,
} from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import {
  createBlankFilterRule,
  defaultOperatorForField,
  defaultValueForField,
  type FilterFieldDef,
  filterFieldsForView,
  getFilterField,
  isFilterRuleComplete,
} from "../utils/filter-registry";
import { searchTableGroups } from "../utils/search-table-groups";
import { capFilterRules, MAX_FILTER_RULES } from "../utils/search-url-search";

type SearchFilterEditorModalProps = {
  open: boolean;
  onClose: () => void;
  rules: FilterRule[];
  combine: FilterCombineMode;
  onApply: (rules: FilterRule[], combine: FilterCombineMode) => void;
  view?: SearchView;
};

/** The applied rules, or one blank rule so an empty editor starts on a field choice. */
function draftFrom(rules: FilterRule[]): FilterRule[] {
  return rules.length > 0
    ? rules.map((rule) => ({ ...rule, value: { ...rule.value } }))
    : [createBlankFilterRule()];
}

function isBlankRule(rule: FilterRule) {
  return rule.field === "";
}

function valueError(
  rule: FilterRule,
  field: FilterFieldDef,
  view: SearchView,
): string | undefined {
  if (isFilterRuleComplete(rule, view)) {
    return undefined;
  }
  if (isPercentileOperator(rule.op)) {
    return "Enter a percentile from 0 to 100";
  }
  return field.kind === "string" ? "Enter a value" : "Enter a number";
}

/** Invisible label line that keeps a control level with the labelled fields beside it. */
function LabelSpacer() {
  return (
    <span aria-hidden className={`${fieldLabelClasses} hidden sm:block`}>
      &nbsp;
    </span>
  );
}

function FilterRuleRow({
  rule,
  onChange,
  onRemove,
  view,
  showError,
  openPicker,
}: {
  rule: FilterRule;
  onChange: (next: FilterRule) => void;
  onRemove: () => void;
  view: SearchView;
  showError: boolean;
  openPicker: boolean;
}) {
  const field = getFilterField(rule.field, view);
  const picker = (
    <PlayerMetricPicker
      label="Field"
      metrics={filterFieldsForView(view)}
      value={rule.field}
      placeholder="Choose a field"
      defaultOpen={openPicker}
      groups={searchTableGroups(view)}
      onChange={(fieldId) => {
        onChange({
          ...rule,
          field: fieldId,
          op: defaultOperatorForField(fieldId, view),
          value: defaultValueForField(fieldId, view),
        });
      }}
    />
  );
  const removeButton = (
    <div className="space-y-1">
      <LabelSpacer />
      <Button
        size="icon"
        variant="ghost"
        icon={Trash2}
        aria-label="Remove filter rule"
        onClick={onRemove}
      />
    </div>
  );
  // Top-aligned so an error under one value does not shift the other fields.
  const rowClasses =
    "grid gap-3 rounded-lg border border-outline-variant bg-surface-container-high p-3 sm:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_minmax(0,1fr)_auto] sm:items-start";

  if (isBlankRule(rule)) {
    return (
      <div className={rowClasses}>
        {picker}
        <div className="space-y-1 sm:col-span-2">
          <LabelSpacer />
          <p className="flex h-8 items-center text-body-sm text-on-surface-variant">
            Choose a field to set its condition.
          </p>
        </div>
        {removeButton}
      </div>
    );
  }
  if (!field) {
    return null;
  }

  const percentile = isPercentileOperator(rule.op);

  const handleOperatorChange = (op: string) => {
    if (isPercentileOperator(op) === percentile) {
      onChange({ ...rule, op });
      return;
    }
    // Raw values and percentiles use different scales, so switching resets the value.
    const value = defaultValueForField(rule.field, view);
    onChange({
      ...rule,
      op,
      value:
        isPercentileOperator(op) &&
        (value.type === "integer" || value.type === "number")
          ? { ...value, value: 90 }
          : value,
    });
  };

  const handleValueChange = (value: FilterValue) => {
    onChange({ ...rule, value });
  };
  const error = showError ? valueError(rule, field, view) : undefined;

  return (
    <div className={rowClasses}>
      {picker}

      <SelectField
        label="Operator"
        value={rule.op}
        onChange={(event) => {
          handleOperatorChange(event.target.value);
        }}
      >
        {field.operators.map((operator) => (
          <option key={operator.id} value={operator.id}>
            {operator.label}
          </option>
        ))}
      </SelectField>

      <div>
        {field.kind === "string" ? (
          <TextField
            label="Value"
            error={error}
            value={rule.value.type === "text" ? rule.value.value : ""}
            onChange={(event) => {
              handleValueChange({ type: "text", value: event.target.value });
            }}
          />
        ) : null}
        {field.kind === "integer" ? (
          <TextField
            label="Value"
            type="number"
            inputMode="numeric"
            min={percentile ? 0 : undefined}
            max={percentile ? 100 : undefined}
            error={error}
            value={
              rule.value.type === "integer" && Number.isFinite(rule.value.value)
                ? String(rule.value.value)
                : ""
            }
            onChange={(event) => {
              handleValueChange({
                type: "integer",
                value: Number.parseInt(event.target.value, 10),
              });
            }}
          />
        ) : null}
        {field.kind === "number" ? (
          <TextField
            label="Value"
            type="number"
            inputMode="decimal"
            min={percentile ? 0 : undefined}
            max={percentile ? 100 : undefined}
            error={error}
            value={
              rule.value.type === "number" && Number.isFinite(rule.value.value)
                ? String(rule.value.value)
                : ""
            }
            onChange={(event) => {
              handleValueChange({
                type: "number",
                value:
                  event.target.value === ""
                    ? Number.NaN
                    : Number(event.target.value),
              });
            }}
          />
        ) : null}
        {field.kind === "boolean" ? (
          <SelectField
            label="Value"
            value={
              rule.value.type === "bool" && rule.value.value ? "yes" : "no"
            }
            onChange={(event) => {
              handleValueChange({
                type: "bool",
                value: event.target.value === "yes",
              });
            }}
          >
            <option value="yes">Yes</option>
            <option value="no">No</option>
          </SelectField>
        ) : null}
        {field.kind === "enum" ? (
          <SelectField
            label="Value"
            value={rule.value.type === "text" ? rule.value.value : ""}
            onChange={(event) => {
              handleValueChange({ type: "text", value: event.target.value });
            }}
          >
            {field.enumOptions?.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </SelectField>
        ) : null}
      </div>

      {removeButton}
    </div>
  );
}

export function SearchFilterEditorModal({
  open,
  onClose,
  rules,
  combine,
  onApply,
  view = "general",
}: SearchFilterEditorModalProps) {
  const [draftRules, setDraftRules] = useState(() => draftFrom(rules));
  const [draftCombine, setDraftCombine] = useState(combine);
  const [openPickerRuleId, setOpenPickerRuleId] = useState<string | null>(null);
  const [showErrors, setShowErrors] = useState(false);
  const [doneAttempts, setDoneAttempts] = useState(0);
  const rulesRef = useRef<HTMLDivElement>(null);
  const atCap = draftRules.length >= MAX_FILTER_RULES;
  const chosenRules = draftRules.filter((rule) => !isBlankRule(rule));
  const draftIsComplete = chosenRules.every((rule) =>
    isFilterRuleComplete(rule, view),
  );
  const hasPostCohortRule =
    view === "moneyball" &&
    draftRules.some(
      (rule) =>
        rule.field.startsWith("moneyball_role.") ||
        isPercentileOperator(rule.op),
    );

  useEffect(() => {
    if (open) {
      setDraftRules(draftFrom(rules));
      setDraftCombine(combine);
      setOpenPickerRuleId(null);
      setShowErrors(false);
    }
  }, [open, rules, combine]);

  useEffect(() => {
    if (doneAttempts > 0) {
      rulesRef.current
        ?.querySelector<HTMLElement>('[aria-invalid="true"]')
        ?.focus();
    }
  }, [doneAttempts]);

  const dismiss = () => {
    setDraftRules(draftFrom(rules));
    setDraftCombine(combine);
    onClose();
  };

  const addRule = () => {
    if (atCap) {
      return;
    }
    // An untouched blank rule is replaced, so repeated adds never stack empty rows.
    const blank = createBlankFilterRule();
    setDraftRules((current) =>
      capFilterRules([...current.filter((rule) => !isBlankRule(rule)), blank]),
    );
    setOpenPickerRuleId(blank.id);
    // A new rule has had no chance to be filled in yet; Done shows errors again.
    setShowErrors(false);
  };

  const apply = () => {
    if (!draftIsComplete) {
      setShowErrors(true);
      setDoneAttempts((count) => count + 1);
      return;
    }
    onApply(chosenRules, draftCombine);
    onClose();
  };

  return (
    <Modal
      open={open}
      onClose={dismiss}
      title="Edit filters"
      variant="informational"
      footer={
        <>
          <Button variant="secondary" onClick={dismiss}>
            Cancel
          </Button>
          <Button onClick={apply}>Done</Button>
        </>
      }
    >
      <div className="space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <p className="text-body-md text-on-surface-variant">
            Changes apply when you select Done.
          </p>
          <fieldset className="inline-flex rounded-md border border-outline bg-surface-container-high p-0.5">
            <legend className="sr-only">Combine filters</legend>
            {(["and", "or"] as const).map((mode) => (
              <button
                key={mode}
                type="button"
                aria-pressed={draftCombine === mode}
                className={
                  draftCombine === mode
                    ? "h-8 rounded-sm bg-primary px-3 text-label-md text-on-primary uppercase"
                    : "h-8 rounded-sm px-3 text-label-md text-on-surface-variant uppercase hover:bg-surface-container-highest"
                }
                onClick={() => setDraftCombine(mode)}
              >
                {mode}
              </button>
            ))}
          </fieldset>
        </div>

        {hasPostCohortRule ? (
          <p
            aria-label="Percentile and Moneyball role filters apply after the comparison cohort is calculated"
            className="text-body-sm text-on-surface-variant"
            role="note"
          >
            Percentile and Moneyball role filters apply after the comparison
            cohort is calculated, so they follow the Filtered cohort or All
            players choice in the toolbar. With AND they narrow that scored
            cohort; with OR they can add scored matches from the full comparison
            cohort alongside other rules.
          </p>
        ) : null}

        <div ref={rulesRef} className="space-y-3">
          {draftRules.length === 0 ? (
            <p className="text-body-md text-on-surface-variant">
              No filter rules yet. Add one below.
            </p>
          ) : (
            draftRules.map((rule) => (
              <FilterRuleRow
                key={rule.id}
                rule={rule}
                onChange={(next) => {
                  setDraftRules((current) =>
                    current.map((item) => (item.id === rule.id ? next : item)),
                  );
                }}
                onRemove={() => {
                  setDraftRules((current) =>
                    current.filter((item) => item.id !== rule.id),
                  );
                }}
                view={view}
                showError={showErrors}
                openPicker={rule.id === openPickerRuleId}
              />
            ))
          )}
        </div>

        <Button variant="secondary" onClick={addRule} disabled={atCap}>
          Add filter
        </Button>
      </div>
    </Modal>
  );
}
