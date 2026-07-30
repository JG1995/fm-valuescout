import type { FilterRule, FilterValue } from "../types/filter-rule";
import { createFilterRuleId } from "../types/filter-rule";

import { ROLE_CATALOG } from "./role-catalog";

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

/** Visible attribute keys — dump PascalCase (bridge AttributeEntries). */
const ATTR_KEYS = [
  "Crossing",
  "Dribbling",
  "Finishing",
  "Heading",
  "LongShots",
  "Marking",
  "OffTheBall",
  "Passing",
  "PenaltyTaking",
  "Tackling",
  "Vision",
  "Handling",
  "AerialReach",
  "CommandOfArea",
  "Communication",
  "Kicking",
  "Throwing",
  "Anticipation",
  "Decisions",
  "OneOnOnes",
  "Positioning",
  "Reflexes",
  "FirstTouch",
  "Technique",
  "Flair",
  "Corners",
  "Teamwork",
  "WorkRate",
  "LongThrows",
  "Eccentricity",
  "RushingOut",
  "Punching",
  "Acceleration",
  "FreeKicks",
  "Strength",
  "Stamina",
  "Pace",
  "JumpingReach",
  "Leadership",
  "Balance",
  "Bravery",
  "Aggression",
  "Agility",
  "NaturalFitness",
  "Determination",
  "Composure",
  "Concentration",
] as const;

const HIDDEN_ATTR_KEYS = [
  "Dirtiness",
  "Consistency",
  "ImportantMatches",
  "InjuryProneness",
  "Versatility",
] as const;

const PERSONALITY_KEYS = [
  "Adaptability",
  "Ambition",
  "Loyalty",
  "Pressure",
  "Professionalism",
  "Sportsmanship",
  "Temperament",
  "Controversy",
] as const;

const POSITION_KEYS = [
  "GK",
  "SW",
  "DL",
  "DC",
  "DR",
  "DM",
  "ML",
  "MC",
  "MR",
  "AML",
  "AMC",
  "AMR",
  "ST",
  "WBL",
  "WBR",
] as const;

function labelFromPascal(key: string): string {
  return key.replaceAll(/([a-z])([A-Z])/g, "$1 $2");
}

const ATTRIBUTE_FIELDS: FilterFieldDef[] = ATTR_KEYS.map((key) => ({
  id: `attr.${key}`,
  label: labelFromPascal(key),
  kind: "integer" as const,
  operators: INTEGER_OPERATORS,
}));

const HIDDEN_FIELDS: FilterFieldDef[] = HIDDEN_ATTR_KEYS.map((key) => ({
  id: `hidden.${key}`,
  label: `Hidden · ${labelFromPascal(key)}`,
  kind: "integer" as const,
  operators: INTEGER_OPERATORS,
}));

const PERSONALITY_FIELDS: FilterFieldDef[] = PERSONALITY_KEYS.map((key) => ({
  id: `personality.${key}`,
  label: `Personality · ${labelFromPascal(key)}`,
  kind: "integer" as const,
  operators: INTEGER_OPERATORS,
}));

const POSITION_SUITABILITY_FIELDS: FilterFieldDef[] = POSITION_KEYS.map(
  (key) => ({
    id: `pos.${key}`,
    label: `Position · ${key} suitability`,
    kind: "integer" as const,
    operators: INTEGER_OPERATORS,
  }),
);

const ROLE_SCORE_FIELDS: FilterFieldDef[] = ROLE_CATALOG.map((role) => ({
  id: `role.${role.id}`,
  label: `Role · ${role.label}`,
  kind: "integer" as const,
  operators: INTEGER_OPERATORS,
}));

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
  {
    id: "nationality",
    label: "Nationality",
    kind: "string",
    operators: STRING_OPERATORS,
  },
  {
    id: "position",
    label: "Position",
    kind: "enum",
    operators: ENUM_OPERATORS,
    enumOptions: POSITION_KEYS.map((key) => ({ value: key, label: key })),
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
  ...ATTRIBUTE_FIELDS,
  ...HIDDEN_FIELDS,
  ...PERSONALITY_FIELDS,
  ...POSITION_SUITABILITY_FIELDS,
  ...ROLE_SCORE_FIELDS,
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
