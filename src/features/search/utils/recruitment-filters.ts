import { isPercentileOperator } from "@/utils/moneyball-search-metrics";
import type { FilterRule, FilterRuleIpc } from "../types/filter-rule";
import { filterRuleToSearchIpc, getFilterField } from "./filter-registry";
import { formatFilterTagLabel } from "./format-filter-label";
import { parseSearchFilters } from "./search-url-search";

/** Mirrors Rust `MAX_SLOT_FILTER_RULES` in `features/search/filter.rs`. */
export const MAX_RECRUITMENT_FILTERS = 10;

/** Undoes the ×100 rounding error of a fraction shown as a percentage. */
function toPercentage(value: number): number {
  return Math.round(value * 100 * 1e6) / 1e6;
}

/**
 * A stored recruitment filter as an editable rule on the displayed scale, or
 * undefined when its field is not offered. Stored rules use Search's IPC
 * form, where raw values of fraction-scale statistics are 0–1.
 */
export function recruitmentFilterToRule(
  filter: FilterRuleIpc,
  id: string,
): FilterRule | undefined {
  const [rule] = parseSearchFilters([{ ...filter, id }], "slot");
  if (!rule) {
    return undefined;
  }
  const field = getFilterField(rule.field, "slot");
  const fraction =
    field !== undefined &&
    "metric" in field &&
    field.metric?.percentageScale === "fraction" &&
    !isPercentileOperator(rule.op);
  if (!fraction) {
    return rule;
  }
  switch (rule.value.type) {
    case "number":
    case "integer":
      return {
        ...rule,
        value: { ...rule.value, value: toPercentage(rule.value.value) },
      };
    case "range":
      return {
        ...rule,
        value: {
          type: "range",
          value: [
            toPercentage(rule.value.value[0]),
            toPercentage(rule.value.value[1]),
          ],
        },
      };
    default:
      return rule;
  }
}

export function recruitmentFilterFromRule(rule: FilterRule): FilterRuleIpc {
  return filterRuleToSearchIpc(rule, "slot");
}

/** A short label for a stored recruitment filter, e.g. `Height ≥ 183`. */
export function formatRecruitmentFilter(filter: FilterRuleIpc): string {
  const rule = recruitmentFilterToRule(filter, "label");
  return rule ? formatFilterTagLabel(rule, "slot") : "Unknown filter";
}
