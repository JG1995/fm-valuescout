import { Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button/button";
import { SelectField } from "@/components/ui/field/select-field";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import type {
  FilterCombineMode,
  FilterRule,
  FilterValue,
} from "../types/filter-rule";
import {
  createDefaultFilterRule,
  defaultOperatorForField,
  defaultValueForField,
  FILTER_FIELDS,
  getFilterField,
} from "../utils/filter-registry";

type SearchFilterEditorModalProps = {
  open: boolean;
  onClose: () => void;
  rules: FilterRule[];
  combine: FilterCombineMode;
  onRulesChange: (rules: FilterRule[]) => void;
  onCombineChange: (combine: FilterCombineMode) => void;
};

function FilterRuleRow({
  rule,
  onChange,
  onRemove,
}: {
  rule: FilterRule;
  onChange: (next: FilterRule) => void;
  onRemove: () => void;
}) {
  const field = getFilterField(rule.field);
  if (!field) {
    return null;
  }

  const handleFieldChange = (fieldId: string) => {
    onChange({
      ...rule,
      field: fieldId,
      op: defaultOperatorForField(fieldId),
      value: defaultValueForField(fieldId),
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
      <SelectField
        label="Field"
        value={rule.field}
        onChange={(event) => {
          handleFieldChange(event.target.value);
        }}
      >
        {FILTER_FIELDS.map((candidate) => (
          <option key={candidate.id} value={candidate.id}>
            {candidate.label}
          </option>
        ))}
      </SelectField>

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
  onRulesChange,
  onCombineChange,
}: SearchFilterEditorModalProps) {
  const addRule = () => {
    onRulesChange([...rules, createDefaultFilterRule()]);
  };

  return (
    <Modal
      open={open}
      onClose={onClose}
      title="Edit filters"
      footer={
        <Button variant="secondary" onClick={onClose}>
          Done
        </Button>
      }
    >
      <div className="space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className="text-body-md text-on-surface-variant">
            Changes apply immediately — no Apply button.
          </p>
          <fieldset className="inline-flex rounded-full border border-outline bg-surface-container-high p-0.5">
            <legend className="sr-only">Combine filters</legend>
            {(["and", "or"] as const).map((mode) => (
              <button
                key={mode}
                type="button"
                aria-pressed={combine === mode}
                className={
                  combine === mode
                    ? "rounded-full bg-primary px-3 py-1 text-label-md text-on-primary uppercase"
                    : "rounded-full px-3 py-1 text-label-md text-on-surface-variant uppercase hover:bg-surface-container-highest"
                }
                onClick={() => {
                  onCombineChange(mode);
                }}
              >
                {mode}
              </button>
            ))}
          </fieldset>
        </div>

        <div className="space-y-3">
          {rules.length === 0 ? (
            <p className="text-body-md text-on-surface-variant">
              No filter rules yet. Add one below.
            </p>
          ) : (
            rules.map((rule) => (
              <FilterRuleRow
                key={rule.id}
                rule={rule}
                onChange={(next) => {
                  onRulesChange(
                    rules.map((item) => (item.id === rule.id ? next : item)),
                  );
                }}
                onRemove={() => {
                  onRulesChange(rules.filter((item) => item.id !== rule.id));
                }}
              />
            ))
          )}
        </div>

        <Button variant="secondary" onClick={addRule}>
          Add filter
        </Button>
      </div>
    </Modal>
  );
}
