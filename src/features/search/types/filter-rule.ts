export type FilterCombineMode = "and" | "or";

export type FilterValue =
  | { type: "text"; value: string }
  | { type: "integer"; value: number }
  | { type: "bool"; value: boolean };

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
  value: string | number | boolean;
};

export function filterValueToIpc(
  value: FilterValue,
): string | number | boolean {
  switch (value.type) {
    case "text":
      return value.value;
    case "integer":
      return value.value;
    case "bool":
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
