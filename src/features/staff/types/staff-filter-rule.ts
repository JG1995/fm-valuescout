export type StaffFilterValue =
  | { type: "text"; value: string }
  | { type: "integer"; value: number };

export type StaffFilterRule = {
  id: string;
  field: string;
  op: string;
  value: StaffFilterValue;
};

export type StaffFilterRuleIpc = {
  field: string;
  op: string;
  value: string | number;
};

export function staffFilterValueToIpc(
  value: StaffFilterValue,
): string | number {
  return value.value;
}

export function staffFilterRuleToIpc(
  rule: StaffFilterRule,
): StaffFilterRuleIpc {
  return {
    field: rule.field,
    op: rule.op,
    value: staffFilterValueToIpc(rule.value),
  };
}

export function createStaffFilterRuleId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `staff-filter-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;
}
