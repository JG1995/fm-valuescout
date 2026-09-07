import { describe, expect, it } from "vitest";
import {
  resolveTableGroupRuns,
  type TableGroupInput,
} from "@/components/player-table/table-groups";
import {
  MONEYBALL_TABLE_GROUPS,
  SEARCH_TABLE_GROUPS,
} from "@/features/search/components/search-results-panel";
import { SQUAD_TABLE_GROUPS } from "@/features/squad/components/squad-overview-panel";
import {
  STAFF_SHORTLIST_TABLE_GROUPS,
  STAFF_TABLE_GROUPS,
} from "@/features/staff/components/staff-search-results-panel";

type LayoutCase = {
  layout: string;
  input: TableGroupInput;
  columns: Array<{ id: string }>;
  expected: Array<{ label: string; span: number }>;
};

// My Staff renders StaffSearchTable with scope "my-staff", which selects the
// shared STAFF_TABLE_GROUPS (shortlist is false outside shortlistOnly mode).
const STAFF_SEARCH_COLUMNS = [
  { id: "age" },
  { id: "ca" },
  { id: "attr.Attacking" },
  { id: "role.assistant_manager" },
  { id: "wage" },
];
const STAFF_SEARCH_RUNS = [
  { label: "Profile", span: 1 },
  { label: "Ability", span: 1 },
  { label: "Attributes", span: 1 },
  { label: "Role Fit", span: 1 },
  { label: "Contract", span: 1 },
];

const CASES: LayoutCase[] = [
  {
    layout: "Search",
    input: SEARCH_TABLE_GROUPS,
    columns: [
      { id: "age" },
      { id: "nationality" },
      { id: "height" },
      { id: "ca" },
      { id: "value" },
      { id: "tactic_current.goalkeeper" },
    ],
    expected: [
      { label: "Profile", span: 3 },
      { label: "Ability", span: 1 },
      { label: "Market", span: 1 },
      { label: "Tactic Fit", span: 1 },
    ],
  },
  {
    layout: "Moneyball",
    input: MONEYBALL_TABLE_GROUPS,
    columns: [
      { id: "age" },
      { id: "height" },
      { id: "value" },
      { id: "moneyball.starts" },
      { id: "moneyball.average_rating" },
      { id: "moneyball_role.amc_attacking_midfielder_ip" },
    ],
    expected: [
      { label: "Profile", span: 2 },
      { label: "Market", span: 1 },
      { label: "Playing Time", span: 1 },
      { label: "Performance", span: 1 },
      { label: "Role Fit", span: 1 },
    ],
  },
  {
    layout: "Squad",
    input: SQUAD_TABLE_GROUPS,
    columns: [
      { id: "age" },
      { id: "height" },
      { id: "ca" },
      { id: "value" },
      { id: "suggested_training" },
      { id: "role.striker_attack" },
    ],
    expected: [
      { label: "Profile", span: 2 },
      { label: "Ability", span: 1 },
      { label: "Market", span: 1 },
      { label: "Development", span: 1 },
      { label: "Role Fit", span: 1 },
    ],
  },
  {
    layout: "Staff Search",
    input: STAFF_TABLE_GROUPS,
    columns: STAFF_SEARCH_COLUMNS,
    expected: STAFF_SEARCH_RUNS,
  },
  {
    layout: "My Staff",
    input: STAFF_TABLE_GROUPS,
    columns: STAFF_SEARCH_COLUMNS,
    expected: STAFF_SEARCH_RUNS,
  },
  {
    layout: "Staff Shortlist",
    input: STAFF_SHORTLIST_TABLE_GROUPS,
    columns: [...STAFF_SEARCH_COLUMNS, { id: "coaching_qualifications" }],
    expected: [...STAFF_SEARCH_RUNS, { label: "Recruitment", span: 1 }],
  },
];

describe("panel-owned grouped headers", () => {
  it.each(CASES)(
    "$layout maps representative visible leaves to $expected",
    ({ input, columns, expected }) => {
      const runs = resolveTableGroupRuns(columns, input);
      expect(
        runs.map((run) => ({ label: run.group.label, span: run.span })),
      ).toEqual(expected);
    },
  );
});
