import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import { completeFilterRules } from "../utils/filter-registry";
import { FilterTag } from "./filter-tag";

type SearchFilterStripProps = {
  rules: FilterRule[];
  combine: FilterCombineMode;
  onRulesChange: (rules: FilterRule[]) => void;
  view?: SearchView;
};

/**
 * Search dataset chips renderer. This supplies the `filterChips` slot of
 * the shared table toolbar: removable per-rule chips (or the empty-state
 * text when no filters apply). Clear-all, Edit, summary, Columns, and
 * dataset toggles live in the toolbar contract; page actions are never
 * accepted here.
 */
export function SearchFilterStrip({
  rules,
  combine,
  onRulesChange,
  view = "general",
}: SearchFilterStripProps) {
  const appliedRules = completeFilterRules(rules, view);

  const removeRule = (ruleId: string) => {
    onRulesChange(rules.filter((rule) => rule.id !== ruleId));
  };

  if (appliedRules.length === 0) {
    return (
      <p className="text-body-md text-on-surface-variant">
        No filters applied. Use Edit filters to narrow the player list.
        {combine === "or" ? " Rules combine with OR." : null}
      </p>
    );
  }

  return (
    <div className="space-y-2">
      <p className="text-label-md text-on-surface-variant uppercase">
        Combined with {combine}
      </p>
      <div className="flex flex-wrap items-center gap-2">
        {appliedRules.map((rule) => (
          <FilterTag
            key={rule.id}
            rule={rule}
            view={view}
            onRemove={() => {
              removeRule(rule.id);
            }}
          />
        ))}
      </div>
    </div>
  );
}
