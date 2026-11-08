export type FilterCombineMode = "and" | "or";

export type FilterValue =
  | { type: "text"; value: string }
  | { type: "integer"; value: number }
  | { type: "number"; value: number }
  | { type: "bool"; value: boolean }
  /** `[min, max]` for `between` and `pct_between`. */
  | { type: "range"; value: [number, number] }
  /** The choices for `in` and `not_in`. */
  | { type: "list"; value: string[] };

export type FilterRule = {
  id: string;
  field: string;
  op: string;
  value: FilterValue;
};

/** IPC payload shape — mirrors Rust `FilterRuleInput`. */
export type FilterRuleIpc = {
  field: string;
  op: string;
  value: string | number | boolean | number[] | string[];
};

export function filterValueToIpc(value: FilterValue): FilterRuleIpc["value"] {
  switch (value.type) {
    case "range":
      return [...value.value];
    case "list":
      return [...value.value];
    default:
      return value.value;
  }
}

export function filterRuleToIpc(rule: FilterRule): FilterRuleIpc {
  return {
    field: rule.field,
    op: rule.op,
    value: filterValueToIpc(rule.value),
  };
}

export function createFilterRuleId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `filter-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}
