import type { FilterRule } from "../types/filter-rule";
import type { FilterView } from "../types/search-view";
import { getFilterField, isRawPercentageRule } from "./filter-registry";

function operatorSymbol(op: string): string {
  switch (op) {
    case "gt":
      return ">";
    case "gte":
      return "≥";
    case "lt":
      return "<";
    case "lte":
      return "≤";
    case "eq":
      return "=";
    case "neq":
      return "≠";
    case "pct_gte":
      return "percentile ≥";
    case "pct_lte":
      return "percentile ≤";
    case "is_not":
      return "is not";
    default:
      return op.replaceAll("_", " ");
  }
}

/** Short label for compact filter tags, e.g. `Age 18–23` or `Position: DR, DL`. */
export function formatFilterTagLabel(
  rule: FilterRule,
  view: FilterView = "search",
): string {
  const field = getFilterField(rule.field, view);
  if (!field) {
    return "Unknown filter";
  }

  const label = field.label;
  const unit = isRawPercentageRule(rule, view) ? "%" : "";
  const value = rule.value;

  switch (value.type) {
    case "range": {
      const scale = rule.op === "pct_between" ? " percentile" : "";
      return `${label}${scale} ${value.value[0]}–${value.value[1]}${unit}`;
    }
    case "list": {
      const choices = value.value
        .map(
          (choice) =>
            field.enumOptions?.find((option) => option.value === choice)
              ?.label ?? choice,
        )
        .join(", ");
      return `${label}: ${rule.op === "not_in" ? "not " : ""}${choices}`;
    }
    case "bool":
      return `${label} ${operatorSymbol(rule.op)} ${value.value ? "Yes" : "No"}`;
    case "text": {
      const option = field.enumOptions?.find(
        (candidate) => candidate.value === value.value,
      );
      return `${label} ${operatorSymbol(rule.op)} ${option?.label ?? value.value}`;
    }
    default:
      return `${label} ${operatorSymbol(rule.op)} ${value.value}${unit}`;
  }
}
