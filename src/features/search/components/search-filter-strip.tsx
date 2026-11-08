import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import { completeFilterRules } from "../utils/filter-registry";
import { FilterTag } from "./filter-tag";

type SearchFilterStripProps = {
  rules: FilterRule[];
  combine: FilterCombineMode;
  onRulesChange: (rules: FilterRule[]) => void;
  onCombineChange?: (combine: FilterCombineMode) => void;
  onEditRule?: (ruleId: string, anchor: HTMLElement) => void;
  view?: SearchView;
};

/**
 * Applied-filter chips for the toolbar's second row. Each chip opens its rule
 * editor or removes the rule; with two or more rules an All / Any toggle sets
 * how they combine, applying straight away.
 */
export function SearchFilterStrip({
  rules,
  combine,
  onRulesChange,
  onCombineChange,
  onEditRule,
  view = "general",
}: SearchFilterStripProps) {
  const appliedRules = completeFilterRules(rules, view);

  const removeRule = (ruleId: string) => {
    onRulesChange(rules.filter((rule) => rule.id !== ruleId));
  };

  if (appliedRules.length === 0) {
    return null;
  }

  return (
    <div className="flex min-w-0 flex-wrap items-center gap-2">
      {appliedRules.length > 1 ? (
        onCombineChange ? (
          <fieldset className="m-0 inline-flex items-center gap-2 border-0 p-0">
            <legend className="sr-only">Match</legend>
            <span
              aria-hidden
              className="text-label-md text-on-surface-variant uppercase"
            >
              Match
            </span>
            <span className="inline-flex rounded-md border border-outline bg-surface-container-high p-0.5">
              {(["and", "or"] as const).map((mode) => (
                <button
                  key={mode}
                  type="button"
                  aria-pressed={combine === mode}
                  className={
                    combine === mode
                      ? "h-6 rounded-sm bg-primary px-2.5 text-label-md text-on-primary"
                      : "h-6 rounded-sm px-2.5 text-label-md text-on-surface-variant hover:text-on-surface"
                  }
                  onClick={() => onCombineChange(mode)}
                >
                  {mode === "and" ? "All" : "Any"}
                </button>
              ))}
            </span>
          </fieldset>
        ) : (
          <p className="text-label-md text-on-surface-variant uppercase">
            Combined with {combine}
          </p>
        )
      ) : null}
      <div className="flex min-w-0 flex-wrap items-center gap-2">
        {appliedRules.map((rule) => (
          <FilterTag
            key={rule.id}
            rule={rule}
            view={view}
            onEdit={
              onEditRule ? (anchor) => onEditRule(rule.id, anchor) : undefined
            }
            onRemove={() => {
              removeRule(rule.id);
            }}
          />
        ))}
      </div>
    </div>
  );
}
