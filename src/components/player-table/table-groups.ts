/**
 * Shared grouped-header contract for the analysis-first tables.
 *
 * Each consumer panel supplies a view-owned {@link TableGroupInput}: the
 * groups in display priority plus a total `groupForColumn` mapper (prefix or
 * catalog-category rules allowed). Shared code derives contiguous runs over
 * the visible leaf columns in order — a new run starts whenever the group
 * changes, so an interrupted group repeats — and assigns every unmapped leaf
 * to the automatic `Other` fallback. Views never define `Other` themselves.
 * A group with no visible leaves renders nothing.
 *
 * Commit 5 consumes this same input for the grouped Columns control, so
 * header and Columns control cannot diverge.
 */
export type TableGroupDef = {
  id: string;
  label: string;
};

export type TableGroupInput = {
  groups: readonly TableGroupDef[];
  groupForColumn: (columnId: string) => string | null | undefined;
};

export type TableGroupRun = {
  group: TableGroupDef;
  startIndex: number;
  span: number;
};

export const FALLBACK_TABLE_GROUP: TableGroupDef = {
  id: "other",
  label: "Other",
};

export function resolveTableGroupRuns(
  columns: readonly { id: string }[],
  input: TableGroupInput | undefined,
): TableGroupRun[] {
  if (!input) {
    return [];
  }
  const byId = new Map(
    [...input.groups, FALLBACK_TABLE_GROUP].map((group) => [group.id, group]),
  );
  const runs: TableGroupRun[] = [];
  columns.forEach((column, index) => {
    const group =
      byId.get(input.groupForColumn(column.id) ?? FALLBACK_TABLE_GROUP.id) ??
      FALLBACK_TABLE_GROUP;
    const last = runs[runs.length - 1];
    if (last && last.group.id === group.id) {
      last.span += 1;
    } else {
      runs.push({ group, startIndex: index, span: 1 });
    }
  });
  return runs;
}
