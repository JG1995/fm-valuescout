import { Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { SelectField } from "@/components/ui/field/select-field";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import { PlayerMetricPicker } from "@/components/ui/player-metric-picker";
import type {
  FilterCombineMode,
  FilterRule,
  FilterValue,
} from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import {
  createDefaultFilterRule,
  defaultOperatorForField,
  defaultValueForField,
  filterFieldsForView,
  getFilterField,
  isFilterRuleComplete,
} from "../utils/filter-registry";
import { capFilterRules, MAX_FILTER_RULES } from "../utils/search-url-search";

type SearchFilterEditorModalProps = {
  open: boolean;
  onClose: () => void;
  rules: FilterRule[];
  combine: FilterCombineMode;
  onApply: (rules: FilterRule[], combine: FilterCombineMode) => void;
  view?: SearchView;
};

function copyRules(rules: FilterRule[]): FilterRule[] {
  return rules.map((rule) => ({ ...rule, value: { ...rule.value } }));
}

function FilterRuleRow({
  rule,
  onChange,
  onRemove,
  view,
}: {
  rule: FilterRule;
  onChange: (next: FilterRule) => void;
  onRemove: () => void;
  view: SearchView;
}) {
  const field = getFilterField(rule.field, view);
  if (!field) {
    return null;
  }

  const handleFieldChange = (fieldId: string) => {
    onChange({
      ...rule,
      field: fieldId,
      op: defaultOperatorForField(fieldId, view),
      value: defaultValueForField(fieldId, view),
    });
  };

  const handleOperatorChange = (op: string) => {
    onChange({ ...rule, op });
  };

  const handleValueChange = (value: FilterValue) => {
    onChange({ ...rule, value });
  };

  return (
    <div className="grid gap-3 rounded-lg border border-outline-variant bg-surface-container-high p-3 sm:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_minmax(0,1fr)_auto] sm:items-end">
      <PlayerMetricPicker
        label="Field"
        metrics={filterFieldsForView(view)}
        value={rule.field}
        onChange={handleFieldChange}
      />

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
            value={
              rule.value.type === "integer" ? String(rule.value.value) : "0"
            }
            onChange={(event) => {
              const parsed = Number.parseInt(event.target.value, 10);
              handleValueChange({
                type: "integer",
                value: Number.isNaN(parsed) ? 0 : parsed,
              });
            }}
          />
        ) : null}
        {field.kind === "number" ? (
          <TextField
            label="Value"
            type="number"
            inputMode="decimal"
            value={
              rule.value.type === "number" ? String(rule.value.value) : "0"
            }
            onChange={(event) => {
              const parsed = Number(event.target.value);
              handleValueChange({
                type: "number",
                value: Number.isNaN(parsed) ? 0 : parsed,
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

      <Button
        size="icon"
        variant="ghost"
        icon={Trash2}
        aria-label="Remove filter rule"
        onClick={onRemove}
      />
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
  const [draftRules, setDraftRules] = useState(() => copyRules(rules));
  const [draftCombine, setDraftCombine] = useState(combine);
  const atCap = draftRules.length >= MAX_FILTER_RULES;
  const draftIsComplete = draftRules.every((rule) =>
    isFilterRuleComplete(rule, view),
  );
  const hasMoneyballRoleRule =
    view === "moneyball" &&
    draftRules.some((rule) => rule.field.startsWith("moneyball_role."));

  useEffect(() => {
    if (open) {
      setDraftRules(copyRules(rules));
      setDraftCombine(combine);
    }
  }, [open, rules, combine]);

  const dismiss = () => {
    setDraftRules(copyRules(rules));
    setDraftCombine(combine);
    onClose();
  };

  const addRule = () => {
    if (atCap) {
      return;
    }
    setDraftRules((current) =>
      capFilterRules([
        ...current,
        createDefaultFilterRule(
          view === "moneyball" ? "moneyball.average_rating" : "ca",
          view,
        ),
      ]),
    );
  };

  const apply = () => {
    if (!draftIsComplete) {
      return;
    }
    onApply(draftRules, draftCombine);
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
          <Button disabled={!draftIsComplete} onClick={apply}>
            Done
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className="text-body-md text-on-surface-variant">
            Changes apply when you select Done.
          </p>
          <fieldset className="inline-flex rounded-full border border-outline bg-surface-container-high p-0.5">
            <legend className="sr-only">Combine filters</legend>
            {(["and", "or"] as const).map((mode) => (
              <button
                key={mode}
                type="button"
                aria-pressed={draftCombine === mode}
                className={
                  draftCombine === mode
                    ? "rounded-full bg-primary px-3 py-1 text-label-md text-on-primary uppercase"
                    : "rounded-full px-3 py-1 text-label-md text-on-surface-variant uppercase hover:bg-surface-container-highest"
                }
                onClick={() => setDraftCombine(mode)}
              >
                {mode}
              </button>
            ))}
          </fieldset>
        </div>

        {hasMoneyballRoleRule ? (
          <p
            aria-label="Moneyball role filters apply after the comparison cohort is calculated"
            className="text-body-sm text-on-surface-variant"
            role="note"
          >
            Moneyball role filters apply after the comparison cohort is
            calculated. With AND they narrow that scored cohort; with OR they
            can add scored matches from the full comparison cohort alongside
            other rules.
          </p>
        ) : null}

        {!draftIsComplete ? (
          <p className="text-body-sm text-error" role="alert">
            Complete every filter rule before applying filters.
          </p>
        ) : null}

        <div className="space-y-3">
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
