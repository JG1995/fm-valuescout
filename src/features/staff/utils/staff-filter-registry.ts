import type {
  StaffFilterRule,
  StaffFilterValue,
} from "../types/staff-filter-rule";
import { createStaffFilterRuleId } from "../types/staff-filter-rule";
import {
  defaultOperatorForStaffMetric,
  defaultValueForStaffMetric,
  getStaffMetric,
  STAFF_METRICS,
} from "./staff-metrics";

export const STAFF_MAX_FILTER_RULES = 32;
export const STAFF_FILTER_FIELDS = STAFF_METRICS;

export function getStaffFilterField(fieldId: string) {
  return getStaffMetric(fieldId);
}

export function defaultValueForStaffField(fieldId: string): StaffFilterValue {
  return defaultValueForStaffMetric(fieldId);
}

export function defaultOperatorForStaffField(fieldId: string): string {
  return defaultOperatorForStaffMetric(fieldId);
}

export function createDefaultStaffFilterRule(fieldId = "ca"): StaffFilterRule {
  return {
    id: createStaffFilterRuleId(),
    field: fieldId,
    op: defaultOperatorForStaffField(fieldId),
    value: defaultValueForStaffField(fieldId),
  };
}

export function isStaffFilterRuleComplete(rule: StaffFilterRule): boolean {
  const field = getStaffFilterField(rule.field);
  if (!field?.operators.some((operator) => operator.id === rule.op)) {
    return false;
  }
  if (field.kind === "string") {
    return rule.value.type === "text" && rule.value.value.trim().length > 0;
  }
  return rule.value.type === "integer" && Number.isFinite(rule.value.value);
}

export function completeStaffFilterRules(
  rules: StaffFilterRule[],
): StaffFilterRule[] {
  return rules.filter(isStaffFilterRuleComplete);
}

function isStaffFilterValue(value: unknown): value is StaffFilterValue {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    return false;
  }
  const record = value as Record<string, unknown>;
  return (
    (record.type === "text" && typeof record.value === "string") ||
    (record.type === "integer" &&
      typeof record.value === "number" &&
      Number.isFinite(record.value))
  );
}

function parseStaffFilterValue(
  fieldId: string,
  value: unknown,
): StaffFilterValue | undefined {
  const field = getStaffFilterField(fieldId);
  if (!field) {
    return undefined;
  }
  if (isStaffFilterValue(value)) {
    return field.kind === "string" && value.type === "text"
      ? value
      : field.kind === "integer" && value.type === "integer"
        ? value
        : undefined;
  }
  if (field.kind === "string" && typeof value === "string") {
    return { type: "text", value };
  }
  if (
    field.kind === "integer" &&
    typeof value === "number" &&
    Number.isFinite(value)
  ) {
    return { type: "integer", value };
  }
  return undefined;
}

function parseOneStaffFilterRule(value: unknown): StaffFilterRule | undefined {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    return undefined;
  }
  const record = value as Record<string, unknown>;
  const field = typeof record.field === "string" ? record.field : undefined;
  const op = typeof record.op === "string" ? record.op : undefined;
  if (
    !field ||
    !op ||
    !getStaffFilterField(field)?.operators.some((item) => item.id === op)
  ) {
    return undefined;
  }
  const parsedValue = parseStaffFilterValue(field, record.value);
  if (!parsedValue) {
    return undefined;
  }
  return {
    id:
      typeof record.id === "string" && record.id.length > 0
        ? record.id
        : createStaffFilterRuleId(),
    field,
    op,
    value: parsedValue,
  };
}

export function parseStaffFilters(value: unknown): StaffFilterRule[] {
  if (!Array.isArray(value)) {
    return [];
  }
  const parsed: StaffFilterRule[] = [];
  for (const entry of value) {
    if (parsed.length >= STAFF_MAX_FILTER_RULES) {
      break;
    }
    const rule = parseOneStaffFilterRule(entry);
    if (rule) {
      parsed.push(rule);
    }
  }
  return parsed;
}

export function capStaffFilterRules(rules: StaffFilterRule[]) {
  return rules.slice(0, STAFF_MAX_FILTER_RULES);
}
