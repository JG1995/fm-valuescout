import { Plus } from "lucide-react";
import { useCallback, useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button/button";
import type { FilterRule, FilterRuleIpc } from "../types/filter-rule";
import { createBlankFilterRule } from "../utils/filter-registry";
import {
  MAX_RECRUITMENT_FILTERS,
  recruitmentFilterFromRule,
  recruitmentFilterToRule,
} from "../utils/recruitment-filters";
import { FilterRulePopover } from "./filter-rule-popover";
import { FilterTag } from "./filter-tag";

type Editing = {
  rule: FilterRule;
  /** The edited rule's index, or null for a new rule. */
  index: number | null;
  anchor: HTMLElement | null;
};

/**
 * Edits one tactic slot's recruitment filters with Search's filter chips and
 * rule editor. Rules arrive and leave in Search's IPC form; a disabled
 * ancestor fieldset disables every control.
 */
export function RecruitmentFilterEditor({
  filters,
  onChange,
}: {
  filters: readonly FilterRuleIpc[];
  onChange: (filters: FilterRuleIpc[]) => void;
}) {
  const [editing, setEditing] = useState<Editing | null>(null);
  const addRef = useRef<HTMLSpanElement>(null);
  const rules = useMemo(
    () =>
      filters.map((filter, index) =>
        recruitmentFilterToRule(filter, `rule-${index}`),
      ),
    [filters],
  );
  const atCap = filters.length >= MAX_RECRUITMENT_FILTERS;

  const close = useCallback(() => {
    const anchor = editing?.anchor;
    setEditing(null);
    const target =
      anchor?.isConnected && anchor !== addRef.current
        ? anchor.querySelector("button")
        : addRef.current?.querySelector("button");
    target?.focus();
  }, [editing]);

  const replace = (index: number | null, rule: FilterRule | null) => {
    const next = [...filters];
    if (index === null) {
      if (rule) {
        next.push(recruitmentFilterFromRule(rule));
      }
    } else if (rule) {
      next[index] = recruitmentFilterFromRule(rule);
    } else {
      next.splice(index, 1);
    }
    onChange(next);
  };

  return (
    <div className="space-y-2">
      {filters.length > 0 ? (
        <ul className="flex flex-wrap gap-1.5" aria-label="Recruitment filters">
          {rules.map((rule, index) => (
            <li
              // Rules have no identity of their own; their order is stable.
              // biome-ignore lint/suspicious/noArrayIndexKey: see above
              key={index}
              className="max-w-full"
            >
              {rule ? (
                <FilterTag
                  rule={rule}
                  view="slot"
                  onEdit={(anchor) => setEditing({ rule, index, anchor })}
                  onRemove={() => replace(index, null)}
                />
              ) : (
                <span className="text-body-sm text-on-surface-variant">
                  Unknown filter
                </span>
              )}
            </li>
          ))}
        </ul>
      ) : null}
      <span ref={addRef} className="inline-flex">
        <Button
          variant="secondary"
          icon={Plus}
          disabled={atCap}
          aria-expanded={editing?.index === null}
          onClick={() =>
            editing?.index === null
              ? close()
              : setEditing({
                  rule: createBlankFilterRule(),
                  index: null,
                  anchor: addRef.current,
                })
          }
        >
          Add filter
        </Button>
      </span>
      {editing ? (
        <FilterRulePopover
          key={editing.rule.id}
          anchor={editing.anchor}
          rule={editing.rule}
          isNew={editing.index === null}
          view="slot"
          onApply={(rule) => {
            replace(editing.index, rule);
            close();
          }}
          onRemove={
            editing.index === null
              ? undefined
              : () => {
                  replace(editing.index, null);
                  setEditing(null);
                  addRef.current?.querySelector("button")?.focus();
                }
          }
          onClose={close}
        />
      ) : null}
    </div>
  );
}
