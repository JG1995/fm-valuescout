import { Plus } from "lucide-react";
import {
  type ReactNode,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import { TableToolbar } from "@/components/player-table/table-toolbar";
import { Button } from "@/components/ui/button/button";
import type { FilterCombineMode, FilterRule } from "../types/filter-rule";
import type { ComparisonPool, SearchView } from "../types/search-view";
import {
  completeFilterRules,
  createBlankFilterRule,
  defaultOperatorForField,
  defaultValueForRule,
} from "../utils/filter-registry";
import { capFilterRules, MAX_FILTER_RULES } from "../utils/search-url-search";
import { FilterRulePopover } from "./filter-rule-popover";
import { SearchFilterStrip } from "./search-filter-strip";

/** Ask the bar to start a new rule for a field, e.g. from a column menu. */
export type FilterFieldRequest = { field: string; key: number };

type SearchFilterBarProps = {
  rules: FilterRule[];
  combine: FilterCombineMode;
  onRulesChange: (rules: FilterRule[]) => void;
  onApply: (rules: FilterRule[], combine: FilterCombineMode) => void;
  view?: SearchView;
  comparisonPool?: ComparisonPool;
  fieldRequest?: FilterFieldRequest | null;
  summary?: ReactNode;
  columnsControl?: ReactNode;
  datasetToggles?: ReactNode;
};

type Editing =
  | { kind: "new"; rule: FilterRule; anchor: HTMLElement | null }
  | { kind: "edit"; rule: FilterRule; anchor: HTMLElement | null };

/**
 * Search dataset controls composed into the shared table toolbar: summary,
 * Add filter, chips that open their rule editor, Clear all, Columns, and
 * caller-owned dataset toggles. Each rule is edited in a popover and applies
 * on its own; typing stays query-silent until Add or Apply. The caller adds a
 * filtered field's column in `onApply`. Page actions stay in the page header.
 */
export function SearchFilterBar({
  rules,
  combine,
  onRulesChange,
  onApply,
  view = "general",
  comparisonPool,
  fieldRequest,
  summary,
  columnsControl,
  datasetToggles,
}: SearchFilterBarProps) {
  const [editing, setEditing] = useState<Editing | null>(null);
  const addRef = useRef<HTMLSpanElement>(null);
  const appliedRules = completeFilterRules(rules, view);
  const atCap = rules.length >= MAX_FILTER_RULES;

  const startNewRule = useCallback(
    (field?: string) => {
      const rule = field
        ? {
            ...createBlankFilterRule(),
            field,
            op: defaultOperatorForField(field, view),
            value: defaultValueForRule(
              field,
              defaultOperatorForField(field, view),
              view,
            ),
          }
        : createBlankFilterRule();
      setEditing({ kind: "new", rule, anchor: addRef.current });
    },
    [view],
  );

  const handledRequest = useRef<number | null>(null);
  useEffect(() => {
    if (!fieldRequest || fieldRequest.key === handledRequest.current) {
      return;
    }
    handledRequest.current = fieldRequest.key;
    if (!atCap) {
      startNewRule(fieldRequest.field);
    }
  }, [atCap, fieldRequest, startNewRule]);

  const close = useCallback(() => {
    const anchor = editing?.anchor;
    setEditing(null);
    const target =
      anchor?.isConnected && anchor !== addRef.current
        ? anchor.querySelector("button")
        : addRef.current?.querySelector("button");
    target?.focus();
  }, [editing]);

  const apply = (rule: FilterRule) => {
    onApply(
      editing?.kind === "edit"
        ? rules.map((candidate) =>
            candidate.id === rule.id ? rule : candidate,
          )
        : capFilterRules([...rules, rule]),
      combine,
    );
    close();
  };

  return (
    <>
      <TableToolbar
        toolbarLabel="Player results toolbar"
        summary={summary}
        filterSummary={
          <span ref={addRef} className="inline-flex">
            <Button
              variant="secondary"
              icon={Plus}
              disabled={atCap}
              aria-expanded={editing?.kind === "new"}
              onClick={() =>
                editing?.kind === "new" ? close() : startNewRule()
              }
            >
              Add filter
            </Button>
          </span>
        }
        filterChips={
          appliedRules.length > 0 ? (
            <SearchFilterStrip
              rules={rules}
              combine={combine}
              onRulesChange={onRulesChange}
              onCombineChange={(next) => onApply(rules, next)}
              onEditRule={(ruleId, anchor) => {
                const rule = rules.find((candidate) => candidate.id === ruleId);
                if (rule) {
                  setEditing({ kind: "edit", rule, anchor });
                }
              }}
              view={view}
            />
          ) : undefined
        }
        onClearAll={
          appliedRules.length > 0 ? () => onRulesChange([]) : undefined
        }
        columnsControl={columnsControl}
        datasetToggles={datasetToggles}
      />
      {editing ? (
        <FilterRulePopover
          key={editing.rule.id}
          anchor={editing.anchor}
          rule={editing.rule}
          isNew={editing.kind === "new"}
          view={view}
          comparisonPool={comparisonPool}
          onApply={apply}
          onRemove={
            editing.kind === "edit"
              ? () => {
                  onRulesChange(
                    rules.filter(
                      (candidate) => candidate.id !== editing.rule.id,
                    ),
                  );
                  setEditing(null);
                  addRef.current?.querySelector("button")?.focus();
                }
              : undefined
          }
          onClose={close}
        />
      ) : null}
    </>
  );
}
