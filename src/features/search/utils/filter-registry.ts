import type { FilterRule, FilterValue } from "../types/filter-rule";
import { createFilterRuleId } from "../types/filter-rule";

export type FilterFieldKind = "string" | "integer" | "boolean" | "enum";

export type FilterOperator = {
  id: string;
  label: string;
};

export type FilterFieldDef = {
  id: string;
  label: string;
  kind: FilterFieldKind;
  operators: FilterOperator[];
  enumOptions?: ReadonlyArray<{ value: string; label: string }>;
};

const STRING_OPERATORS: FilterOperator[] = [
  { id: "contains", label: "contains" },
  { id: "not_contains", label: "does not contain" },
  { id: "is", label: "is" },
  { id: "is_not", label: "is not" },
];

const INTEGER_OPERATORS: FilterOperator[] = [
  { id: "gt", label: "greater than" },
  { id: "lt", label: "less than" },
  { id: "eq", label: "equals" },
  { id: "neq", label: "does not equal" },
];

const BOOLEAN_OPERATORS: FilterOperator[] = [
  { id: "is", label: "is" },
  { id: "is_not", label: "is not" },
];

const ENUM_OPERATORS: FilterOperator[] = [
  { id: "is", label: "is" },
  { id: "is_not", label: "is not" },
];

export const FILTER_FIELDS: FilterFieldDef[] = [
  { id: "name", label: "Name", kind: "string", operators: STRING_OPERATORS },
  { id: "club", label: "Club", kind: "string", operators: STRING_OPERATORS },
  {
    id: "division",
    label: "Division",
    kind: "string",
    operators: STRING_OPERATORS,
  },
  {
    id: "parent_club",
    label: "Parent club",
    kind: "string",
    operators: STRING_OPERATORS,
  },
  { id: "age", label: "Age", kind: "integer", operators: INTEGER_OPERATORS },
  { id: "ca", label: "CA", kind: "integer", operators: INTEGER_OPERATORS },
  { id: "pa", label: "PA", kind: "integer", operators: INTEGER_OPERATORS },
  {
    id: "height",
    label: "Height",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  { id: "wage", label: "Wage", kind: "integer", operators: INTEGER_OPERATORS },
  {
    id: "value",
    label: "Value",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "reputation",
    label: "Reputation",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "world_reputation",
    label: "World reputation",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "birth_year",
    label: "Birth year",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "contract_year",
    label: "Contract year",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "transfer_listed",
    label: "Transfer listed",
    kind: "boolean",
    operators: BOOLEAN_OPERATORS,
  },
  {
    id: "loan_listed",
    label: "Loan listed",
    kind: "boolean",
    operators: BOOLEAN_OPERATORS,
  },
  {
    id: "not_for_sale",
    label: "Not for sale",
    kind: "boolean",
    operators: BOOLEAN_OPERATORS,
  },
  {
    id: "set_for_release",
    label: "Set for release",
    kind: "boolean",
    operators: BOOLEAN_OPERATORS,
  },
  {
    id: "on_loan",
    label: "On loan",
    kind: "boolean",
    operators: BOOLEAN_OPERATORS,
  },
  {
    id: "preferred_foot",
    label: "Preferred foot",
    kind: "enum",
    operators: ENUM_OPERATORS,
    enumOptions: [
      { value: "left", label: "Left" },
      { value: "right", label: "Right" },
      { value: "either", label: "Either" },
    ],
  },
  {
    id: "team_level",
    label: "Team level",
    kind: "enum",
    operators: ENUM_OPERATORS,
    enumOptions: [
      { value: "senior", label: "Senior" },
      { value: "reserve", label: "Reserve" },
      { value: "youth", label: "Youth" },
    ],
  },
];

const FIELD_BY_ID = new Map(FILTER_FIELDS.map((field) => [field.id, field]));

export function getFilterField(fieldId: string): FilterFieldDef | undefined {
  return FIELD_BY_ID.get(fieldId);
}

export function defaultValueForField(fieldId: string): FilterValue {
  const field = getFilterField(fieldId);
  if (!field) {
    return { type: "text", value: "" };
  }
  switch (field.kind) {
    case "string":
      return { type: "text", value: "" };
    case "integer":
      return { type: "integer", value: 0 };
    case "boolean":
      return { type: "bool", value: true };
    case "enum":
      return { type: "text", value: field.enumOptions?.[0]?.value ?? "" };
  }
}

export function defaultOperatorForField(fieldId: string): string {
  const field = getFilterField(fieldId);
  return field?.operators[0]?.id ?? "contains";
}

export function createDefaultFilterRule(fieldId = "ca"): FilterRule {
  return {
    id: createFilterRuleId(),
    field: fieldId,
    op: defaultOperatorForField(fieldId),
    value: defaultValueForField(fieldId),
  };
}

export function isFilterRuleComplete(rule: FilterRule): boolean {
  const field = getFilterField(rule.field);
  if (!field) {
    return false;
  }
  if (!field.operators.some((operator) => operator.id === rule.op)) {
    return false;
  }

  switch (field.kind) {
    case "string":
      return rule.value.type === "text" && rule.value.value.trim().length > 0;
    case "integer":
      return rule.value.type === "integer" && Number.isFinite(rule.value.value);
    case "boolean":
      return rule.value.type === "bool";
    case "enum":
      return (
        rule.value.type === "text" &&
        (field.enumOptions?.some(
          (option) => option.value === rule.value.value,
        ) ??
          false)
      );
  }
}

export function completeFilterRules(rules: FilterRule[]): FilterRule[] {
  return rules.filter(isFilterRuleComplete);
}
