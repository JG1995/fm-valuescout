import { useId } from "react";
import { fieldLabelClasses } from "@/components/ui/field/field-styles";
import { SelectField } from "@/components/ui/field/select-field";
import { TextField } from "@/components/ui/field/text-field";
import { cn } from "@/utils/cn";
import { isPercentileOperator } from "@/utils/moneyball-search-metrics";
import type { FilterRule, FilterValue } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import {
  defaultValueForRule,
  type FilterFieldDef,
  isFilterRuleComplete,
  valueShapeForOperator,
} from "../utils/filter-registry";

/** Why a rule cannot apply yet, shown under its value; undefined when complete. */
export function filterValueError(
  rule: FilterRule,
  field: FilterFieldDef,
  view: SearchView,
): string | undefined {
  if (isFilterRuleComplete(rule, view)) {
    return undefined;
  }
  const percentile = isPercentileOperator(rule.op);
  switch (valueShapeForOperator(rule.op)) {
    case "range":
      return percentile
        ? "Enter percentiles from 0 to 100, lowest first"
        : "Enter both numbers, lowest first";
    case "list":
      return "Choose at least one";
    case "single":
      if (percentile) {
        return "Enter a percentile from 0 to 100";
      }
      return field.kind === "string" ? "Enter a value" : "Enter a number";
  }
}

/**
 * Operator and value controls for one rule: a single value, a min–max range,
 * or a set of choices, depending on the operator. Renders two siblings so the
 * caller's layout places them.
 */
export function FilterConditionFields({
  rule,
  field,
  view,
  error,
  onChange,
}: {
  rule: FilterRule;
  field: FilterFieldDef;
  view: SearchView;
  error?: string;
  onChange: (next: FilterRule) => void;
}) {
  const percentile = isPercentileOperator(rule.op);
  const whole = field.kind === "integer" && !percentile;
  const errorId = useId();

  const changeOperator = (op: string) => {
    // A different value shape or scale (raw vs percentile) starts a fresh value.
    const keep =
      valueShapeForOperator(op) === valueShapeForOperator(rule.op) &&
      isPercentileOperator(op) === percentile;
    onChange({
      ...rule,
      op,
      value: keep ? rule.value : defaultValueForRule(rule.field, op, view),
    });
  };
  const setValue = (value: FilterValue) => onChange({ ...rule, value });
  const parseNumber = (text: string) =>
    text === "" ? Number.NaN : whole ? Number.parseInt(text, 10) : Number(text);
  const numberField = (
    label: string,
    value: number,
    onValue: (value: number) => void,
    fieldError: string | undefined,
    invalid: boolean,
  ) => (
    <TextField
      label={label}
      type="number"
      inputMode={whole ? "numeric" : "decimal"}
      min={percentile ? 0 : undefined}
      max={percentile ? 100 : undefined}
      error={fieldError}
      aria-invalid={invalid ? true : undefined}
      value={Number.isFinite(value) ? String(value) : ""}
      onChange={(event) => onValue(parseNumber(event.target.value))}
    />
  );

  const value = rule.value;
  let valueControl: React.ReactNode = null;
  switch (value.type) {
    case "text":
      valueControl = (
        <TextField
          label="Value"
          error={error}
          value={value.value}
          onChange={(event) =>
            setValue({ type: "text", value: event.target.value })
          }
        />
      );
      break;
    case "integer":
    case "number":
      valueControl = numberField(
        "Value",
        value.value,
        (next) => setValue({ type: value.type, value: next }),
        error,
        error !== undefined,
      );
      break;
    case "bool":
      valueControl = (
        <SelectField
          label="Value"
          value={value.value ? "yes" : "no"}
          onChange={(event) =>
            setValue({ type: "bool", value: event.target.value === "yes" })
          }
        >
          <option value="yes">Yes</option>
          <option value="no">No</option>
        </SelectField>
      );
      break;
    case "range": {
      const [min, max] = value.value;
      valueControl = (
        <div className="grid grid-cols-2 gap-2">
          {numberField(
            "Min",
            min,
            (next) => setValue({ type: "range", value: [next, max] }),
            error,
            error !== undefined,
          )}
          {numberField(
            "Max",
            max,
            (next) => setValue({ type: "range", value: [min, next] }),
            undefined,
            error !== undefined,
          )}
        </div>
      );
      break;
    }
    case "list": {
      const chosen = new Set(value.value);
      valueControl = (
        <fieldset className="m-0 min-w-0 space-y-1 border-0 p-0">
          <legend className={fieldLabelClasses}>Value</legend>
          <div className="flex flex-wrap gap-1">
            {field.enumOptions?.map((option) => {
              const checked = chosen.has(option.value);
              return (
                <label
                  key={option.value}
                  className={cn(
                    "inline-flex h-7 cursor-pointer items-center rounded-full border px-2.5 text-label-md transition-colors duration-150 ease-out has-focus-visible:outline-2 has-focus-visible:outline-offset-2 has-focus-visible:outline-primary",
                    checked
                      ? "border-primary bg-primary text-on-primary"
                      : "border-outline text-on-surface-variant hover:bg-surface-container-highest",
                  )}
                >
                  <input
                    type="checkbox"
                    className="sr-only"
                    checked={checked}
                    aria-invalid={error ? true : undefined}
                    aria-describedby={error ? errorId : undefined}
                    onChange={() => {
                      const next = new Set(chosen);
                      if (checked) {
                        next.delete(option.value);
                      } else {
                        next.add(option.value);
                      }
                      setValue({
                        type: "list",
                        value: (field.enumOptions ?? [])
                          .map((candidate) => candidate.value)
                          .filter((candidate) => next.has(candidate)),
                      });
                    }}
                  />
                  {option.label}
                </label>
              );
            })}
          </div>
          {error ? (
            <p className="text-body-sm text-error" id={errorId}>
              {error}
            </p>
          ) : null}
        </fieldset>
      );
      break;
    }
  }

  return (
    <>
      <SelectField
        label="Operator"
        value={rule.op}
        onChange={(event) => changeOperator(event.target.value)}
      >
        {field.operators.map((operator) => (
          <option key={operator.id} value={operator.id}>
            {operator.label}
          </option>
        ))}
      </SelectField>
      <div className="min-w-0">{valueControl}</div>
    </>
  );
}
