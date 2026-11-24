import {
  createFilterRuleId,
  type FilterCombineMode,
  type FilterRule,
} from "../types/filter-rule";
import {
  createRealisticSigningRule,
  REALISTIC_SIGNING_FIELD_ID,
} from "./filter-registry";
import { MAX_FILTER_RULES } from "./search-url-search";

/** The oldest age an applied saved filter or tactic slot searches by default. */
export const RECRUITING_MAX_AGE = 28;

/**
 * An applied saved filter or tactic slot with the recruiting defaults: only
 * realistic signings, aged 28 or under. A default the filter already has a
 * rule on stays out, so a filter for older players still finds them. A filter
 * that matches any of its rules gains none, because each would widen it.
 */
export function withRecruitingDefaults({
  rules,
  combine,
  realisticAvailable,
}: {
  rules: FilterRule[];
  combine: FilterCombineMode;
  /** Whether there is a managed club to estimate realistic signings for. */
  realisticAvailable: boolean;
}): { rules: FilterRule[]; combine: FilterCombineMode } {
  if (combine === "or" && rules.length > 1) {
    return { rules, combine };
  }
  const has = (fieldId: string) => rules.some(({ field }) => field === fieldId);
  const defaults: FilterRule[] = [];
  if (realisticAvailable && !has(REALISTIC_SIGNING_FIELD_ID)) {
    defaults.push(createRealisticSigningRule());
  }
  if (!has("age")) {
    defaults.push({
      id: createFilterRuleId(),
      field: "age",
      op: "lte",
      value: { type: "integer", value: RECRUITING_MAX_AGE },
    });
  }
  return {
    rules: [...rules, ...defaults].slice(0, MAX_FILTER_RULES),
    combine: "and",
  };
}
