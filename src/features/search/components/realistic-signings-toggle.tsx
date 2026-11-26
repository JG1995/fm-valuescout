import { Tooltip } from "@/components/ui/tooltip/tooltip";
import type { FilterRule } from "../types/filter-rule";
import {
  createRealisticSigningRule,
  REALISTIC_SIGNING_FIELD_ID,
} from "../utils/filter-registry";
import { MAX_FILTER_RULES } from "../utils/search-url-search";

type RealisticSigningsToggleProps = {
  rules: FilterRule[];
  onRulesChange: (rules: FilterRule[]) => void;
  /** The managed club the estimate is made for, or null when there is none to measure. */
  clubName: string | null;
};

function asksForRealistic(rule: FilterRule) {
  return (
    rule.field === REALISTIC_SIGNING_FIELD_ID &&
    rule.value.type === "bool" &&
    (rule.op === "is") === rule.value.value
  );
}

/**
 * Keeps players the managed club could plausibly sign. It adds and removes
 * the realistic-signing filter rule, so the rule also shows as a chip and
 * Clear all removes it.
 */
export function RealisticSigningsToggle({
  rules,
  onRulesChange,
  clubName,
}: RealisticSigningsToggleProps) {
  const checked = rules.some(asksForRealistic);
  const otherRules = rules.filter(
    (rule) => rule.field !== REALISTIC_SIGNING_FIELD_ID,
  );
  const hint =
    clubName === null
      ? "Choose your club in My Club to use this."
      : `An estimate of who would join ${clubName}, from ability and reputation, not Football Manager's own check.`;

  return (
    <Tooltip content={hint}>
      {(tooltipId) => (
        <label className="flex items-center gap-2 text-body-md text-on-surface">
          <input
            type="checkbox"
            checked={checked}
            disabled={
              clubName === null ||
              (!checked && otherRules.length >= MAX_FILTER_RULES)
            }
            aria-describedby={tooltipId}
            onChange={(event) =>
              onRulesChange(
                event.target.checked
                  ? [...otherRules, createRealisticSigningRule()]
                  : otherRules,
              )
            }
          />
          Only realistic signings
        </label>
      )}
    </Tooltip>
  );
}
