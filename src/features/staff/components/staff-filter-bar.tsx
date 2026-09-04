import { SlidersHorizontal, Trash2 } from "lucide-react";
import type { ReactNode } from "react";
import { useCallback, useEffect, useState } from "react";
import { Button } from "@/components/ui/button/button";
import { SelectField } from "@/components/ui/field/select-field";
import { TextField } from "@/components/ui/field/text-field";
import { Modal } from "@/components/ui/modal/modal";
import { Panel } from "@/components/ui/panel/panel";
import { MetricPicker } from "@/components/ui/player-metric-picker";
import type {
  StaffFilterRule,
  StaffFilterValue,
} from "../types/staff-filter-rule";
import {
  capStaffFilterRules,
  completeStaffFilterRules,
  createDefaultStaffFilterRule,
  defaultOperatorForStaffField,
  defaultValueForStaffField,
  getStaffFilterField,
  isStaffFilterRuleComplete,
  STAFF_FILTER_FIELDS,
  STAFF_MAX_FILTER_RULES,
} from "../utils/staff-filter-registry";

type StaffFilterBarProps = {
  rules: StaffFilterRule[];
  combine: "and" | "or";
  onRulesChange: (rules: StaffFilterRule[]) => void;
  onApply: (rules: StaffFilterRule[], combine: "and" | "or") => void;
  headerActions?: ReactNode;
  shortlistOnly?: boolean;
  preferredJob?: string;
  preferredJobOptions?: string[];
  unemployedOnly?: boolean;
  onPreferredJobChange?: (preferredJob: string) => void;
  onUnemployedOnlyChange?: (unemployedOnly: boolean) => void;
};

function formatValue(rule: StaffFilterRule): string {
  return String(rule.value.value);
}

function StaffFilterStrip({
  rules,
  combine,
  onRulesChange,
  onEdit,
  headerActions,
  shortlistOnly,
  preferredJob,
  preferredJobOptions,
  unemployedOnly,
  onPreferredJobChange,
  onUnemployedOnlyChange,
}: Pick<
  StaffFilterBarProps,
  | "rules"
  | "combine"
  | "onRulesChange"
  | "headerActions"
  | "shortlistOnly"
  | "preferredJob"
  | "preferredJobOptions"
  | "unemployedOnly"
  | "onPreferredJobChange"
  | "onUnemployedOnlyChange"
> & {
  onEdit: () => void;
}) {
  const appliedRules = completeStaffFilterRules(rules);
  return (
    <Panel
      title="Filters"
      actions={
        <div className="flex min-w-0 flex-1 flex-wrap items-center justify-end gap-2">
          {appliedRules.length > 0 ? (
            <Button variant="ghost" onClick={() => onRulesChange([])}>
              Clear all
            </Button>
          ) : null}
          <Button variant="secondary" icon={SlidersHorizontal} onClick={onEdit}>
            Edit filters
          </Button>
          {headerActions}
        </div>
      }
    >
      {shortlistOnly ? (
        <div className="flex flex-wrap items-center gap-4 pt-2">
          <label className="flex items-center gap-2 text-body-md text-on-surface">
            Preferred Job
            <select
              className="rounded-md border border-outline bg-surface px-2 py-1 text-on-surface"
              value={preferredJob ?? ""}
              onChange={(event) => onPreferredJobChange?.(event.target.value)}
            >
              <option value="">All jobs</option>
              {(preferredJobOptions ?? []).map((job) => (
                <option key={job} value={job}>
                  {job}
                </option>
              ))}
            </select>
          </label>
          <label className="flex items-center gap-2 text-body-md text-on-surface">
            <input
              type="checkbox"
              checked={unemployedOnly === true}
              onChange={(event) =>
                onUnemployedOnlyChange?.(event.target.checked)
              }
            />
            Only unemployed
          </label>
        </div>
      ) : null}
      {appliedRules.length === 0 ? (
        <p className="text-body-md text-on-surface-variant">
          No filters applied. Use Edit filters to narrow the staff list.
          {combine === "or" ? " Rules combine with OR." : null}
        </p>
      ) : (
        <div className="space-y-2">
          <p className="text-label-md text-on-surface-variant uppercase">
            Combined with {combine}
          </p>
          <div className="flex flex-wrap items-center gap-2">
            {appliedRules.map((rule) => {
              const field = getStaffFilterField(rule.field);
              const operator = field?.operators.find(
                (candidate) => candidate.id === rule.op,
              );
              return (
                <span
                  key={rule.id}
                  className="inline-flex items-center gap-1 rounded-full border border-outline-variant bg-surface-container-high px-2 py-1 text-label-md text-on-surface"
                >
                  {field?.label ?? rule.field} {operator?.label ?? rule.op}{" "}
                  {formatValue(rule)}
                  <button
                    type="button"
                    aria-label={`Remove ${field?.label ?? rule.field} filter`}
                    className="rounded-full p-0.5 text-on-surface-variant hover:bg-surface-container-highest hover:text-on-surface focus-visible:outline-2 focus-visible:outline-primary"
                    onClick={() =>
                      onRulesChange(rules.filter((item) => item.id !== rule.id))
                    }
                  >
                    <span aria-hidden="true">×</span>
                  </button>
                </span>
              );
            })}
          </div>
        </div>
      )}
    </Panel>
  );
}

function copyRules(rules: StaffFilterRule[]): StaffFilterRule[] {
  return rules.map((rule) => ({ ...rule, value: { ...rule.value } }));
}

function FilterRuleRow({
  rule,
  onChange,
  onRemove,
}: {
  rule: StaffFilterRule;
  onChange: (rule: StaffFilterRule) => void;
  onRemove: () => void;
}) {
  const field = getStaffFilterField(rule.field);
  if (!field) {
    return null;
  }
  const updateValue = (value: StaffFilterValue) => onChange({ ...rule, value });
  return (
    <div className="grid gap-3 rounded-lg border border-outline-variant bg-surface-container-high p-3 sm:grid-cols-[minmax(0,1.2fr)_minmax(0,1fr)_minmax(0,1fr)_auto] sm:items-end">
      <MetricPicker
        label="Field"
        metrics={STAFF_FILTER_FIELDS}
        value={rule.field}
        onChange={(fieldId) =>
          onChange({
            ...rule,
            field: fieldId,
            op: defaultOperatorForStaffField(fieldId),
            value: defaultValueForStaffField(fieldId),
          })
        }
      />
      <SelectField
        label="Operator"
        value={rule.op}
        onChange={(event) => onChange({ ...rule, op: event.target.value })}
      >
        {field.operators.map((operator) => (
          <option key={operator.id} value={operator.id}>
            {operator.label}
          </option>
        ))}
      </SelectField>
      <div>
        {field.kind === "string" ? (
          <TextField
            label="Value"
            value={rule.value.type === "text" ? rule.value.value : ""}
            onChange={(event) =>
              updateValue({ type: "text", value: event.target.value })
            }
          />
        ) : (
          <TextField
            label="Value"
            type="number"
            inputMode="numeric"
            value={
              rule.value.type === "integer" ? String(rule.value.value) : "0"
            }
            onChange={(event) => {
              const value = Number.parseInt(event.target.value, 10);
              updateValue({
                type: "integer",
                value: Number.isNaN(value) ? 0 : value,
              });
            }}
          />
        )}
      </div>
      <Button
        size="icon"
        variant="ghost"
        icon={Trash2}
        aria-label="Remove filter rule"
        onClick={onRemove}
      />
    </div>
  );
}

function StaffFilterEditorModal({
  open,
  onClose,
  rules,
  combine,
  onApply,
}: Omit<StaffFilterBarProps, "onRulesChange"> & {
  open: boolean;
  onClose: () => void;
}) {
  const [draftRules, setDraftRules] = useState(() => copyRules(rules));
  const [draftCombine, setDraftCombine] = useState(combine);
  const atCap = draftRules.length >= STAFF_MAX_FILTER_RULES;
  const complete = draftRules.every(isStaffFilterRuleComplete);

  useEffect(() => {
    if (open) {
      setDraftRules(copyRules(rules));
      setDraftCombine(combine);
    }
  }, [combine, open, rules]);

  const dismiss = () => {
    setDraftRules(copyRules(rules));
    setDraftCombine(combine);
    onClose();
  };

  return (
    <Modal
      open={open}
      onClose={dismiss}
      title="Edit staff filters"
      variant="informational"
      footer={
        <>
          <Button variant="secondary" onClick={dismiss}>
            Cancel
          </Button>
          <Button
            disabled={!complete}
            onClick={() => {
              if (complete) {
                onApply(draftRules, draftCombine);
                onClose();
              }
            }}
          >
            Done
          </Button>
        </>
      }
    >
      <div className="space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <p className="text-body-md text-on-surface-variant">
            Changes apply when you select Done.
          </p>
          <fieldset className="inline-flex rounded-full border border-outline bg-surface-container-high p-0.5">
            <legend className="sr-only">Combine filters</legend>
            {(["and", "or"] as const).map((mode) => (
              <button
                key={mode}
                type="button"
                aria-pressed={draftCombine === mode}
                className={
                  draftCombine === mode
                    ? "rounded-full bg-primary px-3 py-1 text-label-md text-on-primary uppercase"
                    : "rounded-full px-3 py-1 text-label-md text-on-surface-variant uppercase hover:bg-surface-container-highest"
                }
                onClick={() => setDraftCombine(mode)}
              >
                {mode}
              </button>
            ))}
          </fieldset>
        </div>
        {!complete ? (
          <p className="text-body-sm text-error" role="alert">
            Complete every filter rule before applying filters.
          </p>
        ) : null}
        <div className="space-y-3">
          {draftRules.length === 0 ? (
            <p className="text-body-md text-on-surface-variant">
              No filter rules yet. Add one below.
            </p>
          ) : (
            draftRules.map((rule) => (
              <FilterRuleRow
                key={rule.id}
                rule={rule}
                onChange={(next) =>
                  setDraftRules((current) =>
                    current.map((item) => (item.id === rule.id ? next : item)),
                  )
                }
                onRemove={() =>
                  setDraftRules((current) =>
                    current.filter((item) => item.id !== rule.id),
                  )
                }
              />
            ))
          )}
        </div>
        <Button
          variant="secondary"
          disabled={atCap}
          onClick={() =>
            setDraftRules((current) =>
              capStaffFilterRules([...current, createDefaultStaffFilterRule()]),
            )
          }
        >
          Add filter
        </Button>
      </div>
    </Modal>
  );
}

export function StaffFilterBar({
  rules,
  combine,
  onRulesChange,
  onApply,
  headerActions,
  shortlistOnly,
  preferredJob,
  preferredJobOptions,
  unemployedOnly,
  onPreferredJobChange,
  onUnemployedOnlyChange,
}: StaffFilterBarProps) {
  const [open, setOpen] = useState(false);
  const close = useCallback(() => setOpen(false), []);
  return (
    <>
      <StaffFilterStrip
        rules={rules}
        combine={combine}
        onRulesChange={onRulesChange}
        onEdit={() => setOpen(true)}
        headerActions={headerActions}
        shortlistOnly={shortlistOnly}
        preferredJob={preferredJob}
        preferredJobOptions={preferredJobOptions}
        unemployedOnly={unemployedOnly}
        onPreferredJobChange={onPreferredJobChange}
        onUnemployedOnlyChange={onUnemployedOnlyChange}
      />
      <StaffFilterEditorModal
        open={open}
        onClose={close}
        rules={rules}
        combine={combine}
        onApply={onApply}
      />
    </>
  );
}
