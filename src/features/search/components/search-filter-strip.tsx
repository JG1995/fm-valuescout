import { SlidersHorizontal } from "lucide-react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button/button";
import { Panel } from "@/components/ui/panel/panel";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { SearchView } from "../types/search-view";
import { completeFilterRules } from "../utils/filter-registry";
import { FilterTag } from "./filter-tag";

type SearchFilterStripProps = {
  rules: FilterRule[];
  combine: FilterCombineMode;
  onRulesChange: (rules: FilterRule[]) => void;
  onEdit: () => void;
  actions?: ReactNode;
  afterEditActions?: ReactNode;
  view?: SearchView;
};

export function SearchFilterStrip({
  rules,
  combine,
  onRulesChange,
  onEdit,
  actions,
  afterEditActions,
  view = "general",
}: SearchFilterStripProps) {
  const appliedRules = completeFilterRules(rules, view);

  const clearAll = () => {
    onRulesChange([]);
  };

  const removeRule = (ruleId: string) => {
    onRulesChange(rules.filter((rule) => rule.id !== ruleId));
  };

  return (
    <Panel
      title="Filters"
      actions={
        <div className="flex flex-wrap items-center gap-2">
          {actions}
          {appliedRules.length > 0 ? (
            <Button variant="ghost" onClick={clearAll}>
              Clear all
            </Button>
          ) : null}
          <Button variant="secondary" icon={SlidersHorizontal} onClick={onEdit}>
            Edit filters
          </Button>
          {afterEditActions}
        </div>
      }
    >
      {appliedRules.length === 0 ? (
        <p className="text-body-md text-on-surface-variant">
          No filters applied. Use Edit filters to narrow the player list.
          {combine === "or" ? " Rules combine with OR." : null}
        </p>
      ) : (
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
      )}
    </Panel>
  );
}
