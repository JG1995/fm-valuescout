import type { FilterRule } from "../types/filter-rule";
import { getFilterField } from "./filter-registry";

function operatorSymbol(op: string): string {
  switch (op) {
    case "gt":
      return ">";
    case "lt":
      return "<";
    case "eq":
      return "=";
    case "neq":
      return "≠";
    default:
      return op.replaceAll("_", " ");
  }
}

function formatValue(rule: FilterRule): string {
  const field = getFilterField(rule.field);
  if (!field) {
    return "";
  }

  if (field.kind === "boolean" && rule.value.type === "bool") {
    return rule.value.value ? "Yes" : "No";
  }

  if (field.kind === "enum" && rule.value.type === "text") {
    const option = field.enumOptions?.find(
      (candidate) => candidate.value === rule.value.value,
    );
    return option?.label ?? rule.value.value;
  }

  if (rule.value.type === "integer") {
    return String(rule.value.value);
  }

  if (rule.value.type === "text") {
    return rule.value.value;
  }

  return "";
}

/** Short label for compact filter tags. */
export function formatFilterTagLabel(rule: FilterRule): string {
  const field = getFilterField(rule.field);
  if (!field) {
    return "Unknown filter";
  }

  const value = formatValue(rule);
  const op = rule.op;

  if (field.kind === "string") {
    const opLabel = operatorSymbol(op);
    return `${field.label} ${opLabel} ${value}`;
  }

  if (field.kind === "integer") {
    return `${field.label} ${operatorSymbol(op)} ${value}`;
  }

  if (field.kind === "boolean" || field.kind === "enum") {
    const opLabel = op === "is_not" ? "is not" : "is";
    return `${field.label} ${opLabel} ${value}`;
  }

  return field.label;
}
