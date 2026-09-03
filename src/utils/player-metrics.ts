import {
  HIDDEN_ATTRIBUTE_KEYS,
  labelFromPascal,
  PERSONALITY_ATTRIBUTE_KEYS,
  VISIBLE_ATTRIBUTE_KEYS,
} from "@/utils/player-attributes";
import { POSITION_ORDER } from "@/utils/position-order";
import { ROLE_CATALOG } from "@/utils/role-catalog";

export type PlayerMetricKind = "string" | "integer" | "boolean" | "enum";

export type PlayerMetricAlignment = "left" | "right";

export type PlayerMetricCategory =
  | "identity"
  | "club-contract"
  | "ability-reputation"
  | "visible-attributes"
  | "hidden-attributes"
  | "personality"
  | "position-suitability"
  | "current-role-scores"
  | "potential-role-scores";

export type PlayerMetricRoleFamily =
  | "Goalkeepers"
  | "Central defense"
  | "Full-back and wing-back"
  | "Defensive midfield"
  | "Central midfield"
  | "Wide midfield and wings"
  | "Attacking midfield"
  | "Forwards";

export type PlayerMetricOperator = {
  id: string;
  label: string;
};

export type PlayerMetric = {
  id: string;
  label: string;
  category: PlayerMetricCategory;
  kind: PlayerMetricKind;
  align: PlayerMetricAlignment;
  defaultWidth: number;
  sortable: boolean;
  operators: readonly PlayerMetricOperator[];
  enumOptions?: ReadonlyArray<{ value: string; label: string }>;
  roleFamily?: PlayerMetricRoleFamily;
};

export const DEFAULT_PLAYER_TABLE_COLUMN_IDS = [
  "name",
  "age",
  "nationality",
  "club",
  "division",
  "ca",
  "pa",
  "value",
] as const;

export const PLAYER_TABLE_MIN_COLUMN_WIDTH = 72;
export const PLAYER_TABLE_MAX_COLUMN_WIDTH = 360;

type PlayerMetricDefinition = Omit<
  PlayerMetric,
  "align" | "defaultWidth" | "sortable"
> &
  Partial<Pick<PlayerMetric, "align" | "defaultWidth" | "sortable">>;

function defaultWidthForMetric(id: string, kind: PlayerMetricKind): number {
  if (id.startsWith("role.") || id.startsWith("potential_role.")) {
    return 112;
  }
  if (
    id.startsWith("attr.") ||
    id.startsWith("hidden.") ||
    id.startsWith("personality.") ||
    id.startsWith("pos.")
  ) {
    return 88;
  }
  if (kind === "integer") {
    return 96;
  }
  if (kind === "boolean") {
    return 112;
  }
  if (kind === "enum") {
    return 128;
  }
  return 176;
}

function playerMetric(definition: PlayerMetricDefinition): PlayerMetric {
  return {
    ...definition,
    align:
      definition.align ?? (definition.kind === "integer" ? "right" : "left"),
    defaultWidth:
      definition.defaultWidth ??
      defaultWidthForMetric(definition.id, definition.kind),
    sortable: definition.sortable ?? true,
  };
}

const STRING_OPERATORS: readonly PlayerMetricOperator[] = [
  { id: "contains", label: "contains" },
  { id: "not_contains", label: "does not contain" },
  { id: "is", label: "is" },
  { id: "is_not", label: "is not" },
];

const INTEGER_OPERATORS: readonly PlayerMetricOperator[] = [
  { id: "gt", label: "greater than" },
  { id: "lt", label: "less than" },
  { id: "eq", label: "equals" },
  { id: "neq", label: "does not equal" },
];

const BOOLEAN_OPERATORS: readonly PlayerMetricOperator[] = [
  { id: "is", label: "is" },
  { id: "is_not", label: "is not" },
];

const ENUM_OPERATORS: readonly PlayerMetricOperator[] = [
  { id: "is", label: "is" },
  { id: "is_not", label: "is not" },
];

type RoleId = (typeof ROLE_CATALOG)[number]["id"];

const ROLE_FAMILY_BY_ID = {
  goalkeeper_ip: "Goalkeepers",
  ball_playing_goalkeeper_ip: "Goalkeepers",
  no_nonsense_goalkeeper_ip: "Goalkeepers",
  line_holding_keeper_oop: "Goalkeepers",
  sweeper_keeper_oop: "Goalkeepers",
  centre_back_ip: "Central defense",
  ball_playing_centre_back_ip: "Central defense",
  no_nonsense_centre_back_ip: "Central defense",
  wide_centre_back_ip: "Central defense",
  advanced_centre_back_ip: "Central defense",
  overlapping_centre_back_ip: "Central defense",
  covering_centre_back_oop: "Central defense",
  stopping_centre_back_oop: "Central defense",
  covering_wide_centre_back_oop: "Central defense",
  stopping_wide_centre_back_oop: "Central defense",
  full_back_ip: "Full-back and wing-back",
  inside_full_back_ip: "Full-back and wing-back",
  holding_full_back_oop: "Full-back and wing-back",
  pressing_full_back_oop: "Full-back and wing-back",
  inside_wing_back_ip: "Full-back and wing-back",
  playmaking_wing_back_ip: "Full-back and wing-back",
  wing_back_ip: "Full-back and wing-back",
  advanced_wing_back_ip: "Full-back and wing-back",
  holding_wing_back_oop: "Full-back and wing-back",
  pressing_wing_back_oop: "Full-back and wing-back",
  defensive_midfielder_ip: "Defensive midfield",
  box_to_box_midfielder_ip: "Defensive midfield",
  box_to_box_playmaker_ip: "Defensive midfield",
  deep_lying_playmaker_ip: "Defensive midfield",
  half_back_ip: "Defensive midfield",
  dropping_defensive_midfielder_oop: "Defensive midfield",
  pressing_defensive_midfielder_oop: "Defensive midfield",
  screening_defensive_midfielder_oop: "Defensive midfield",
  wide_covering_defensive_midfielder_oop: "Defensive midfield",
  central_midfielder_ip: "Central midfield",
  advanced_playmaker_ip: "Central midfield",
  midfield_playmaker_ip: "Central midfield",
  wide_central_midfielder_ip: "Central midfield",
  pressing_central_midfielder_oop: "Central midfield",
  screening_central_midfielder_oop: "Central midfield",
  wide_covering_central_midfielder_oop: "Central midfield",
  wide_midfielder_ip: "Wide midfield and wings",
  tracking_wide_midfielder_oop: "Wide midfield and wings",
  wide_outlet_wide_midfielder_oop: "Wide midfield and wings",
  inside_winger_ip: "Wide midfield and wings",
  playmaking_winger_ip: "Wide midfield and wings",
  winger_ip: "Wide midfield and wings",
  attacking_midfielder_ip: "Attacking midfield",
  channel_midfielder_ip: "Attacking midfield",
  free_role_ip: "Attacking midfield",
  second_striker_ip: "Attacking midfield",
  central_outlet_attacking_midfielder_oop: "Attacking midfield",
  splitting_outlet_attacking_midfielder_oop: "Attacking midfield",
  tracking_attacking_midfielder_oop: "Attacking midfield",
  wide_forward_ip: "Forwards",
  inside_forward_ip: "Forwards",
  inside_outlet_winger_oop: "Forwards",
  tracking_winger_oop: "Forwards",
  wide_outlet_winger_oop: "Forwards",
  centre_forward_ip: "Forwards",
  channel_forward_ip: "Forwards",
  deep_lying_forward_ip: "Forwards",
  false_nine_ip: "Forwards",
  poacher_ip: "Forwards",
  target_forward_ip: "Forwards",
  central_outlet_centre_forward_oop: "Forwards",
  splitting_outlet_centre_forward_oop: "Forwards",
  tracking_centre_forward_oop: "Forwards",
  goalkeeper_oop: "Goalkeepers",
  centre_back_oop: "Central defense",
  wide_centre_back_oop: "Central defense",
  full_back_oop: "Full-back and wing-back",
  wing_back_oop: "Full-back and wing-back",
  defensive_midfielder_oop: "Defensive midfield",
  central_midfielder_oop: "Central midfield",
  wide_midfielder_oop: "Wide midfield and wings",
  attacking_midfielder_oop: "Attacking midfield",
  winger_oop: "Wide midfield and wings",
  centre_forward_oop: "Forwards",
} as const satisfies Record<RoleId, PlayerMetricRoleFamily>;

const ATTRIBUTE_METRICS: PlayerMetric[] = VISIBLE_ATTRIBUTE_KEYS.map((key) =>
  playerMetric({
    id: `attr.${key}`,
    label: labelFromPascal(key),
    category: "visible-attributes",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  }),
);

const HIDDEN_METRICS: PlayerMetric[] = HIDDEN_ATTRIBUTE_KEYS.map((key) =>
  playerMetric({
    id: `hidden.${key}`,
    label: `Hidden · ${labelFromPascal(key)}`,
    category: "hidden-attributes",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  }),
);

const PERSONALITY_METRICS: PlayerMetric[] = PERSONALITY_ATTRIBUTE_KEYS.map(
  (key) =>
    playerMetric({
      id: `personality.${key}`,
      label: `Personality · ${labelFromPascal(key)}`,
      category: "personality",
      kind: "integer",
      operators: INTEGER_OPERATORS,
    }),
);

const POSITION_SUITABILITY_METRICS: PlayerMetric[] = POSITION_ORDER.map((key) =>
  playerMetric({
    id: `pos.${key}`,
    label: `Position · ${key} suitability`,
    category: "position-suitability",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  }),
);

const CURRENT_ROLE_SCORE_METRICS: PlayerMetric[] = ROLE_CATALOG.map((role) =>
  playerMetric({
    id: `role.${role.id}`,
    label: `Role · ${role.label}`,
    category: "current-role-scores",
    kind: "integer",
    operators: INTEGER_OPERATORS,
    roleFamily: ROLE_FAMILY_BY_ID[role.id],
  }),
);

const POTENTIAL_ROLE_SCORE_METRICS: PlayerMetric[] = ROLE_CATALOG.map((role) =>
  playerMetric({
    id: `potential_role.${role.id}`,
    label: `Potential role · ${role.label}`,
    category: "potential-role-scores",
    kind: "integer",
    operators: INTEGER_OPERATORS,
    roleFamily: ROLE_FAMILY_BY_ID[role.id],
  }),
);

const PLAYER_METRIC_DEFINITIONS: readonly PlayerMetricDefinition[] = [
  {
    id: "name",
    label: "Name",
    category: "identity",
    kind: "string",
    defaultWidth: 224,
    operators: STRING_OPERATORS,
  },
  {
    id: "nationality",
    label: "Nationality",
    category: "identity",
    kind: "string",
    defaultWidth: 160,
    operators: STRING_OPERATORS,
  },
  {
    id: "age",
    label: "Age",
    category: "identity",
    kind: "integer",
    align: "left",
    defaultWidth: 144,
    operators: INTEGER_OPERATORS,
  },
  {
    id: "birth_year",
    label: "Birth year",
    category: "identity",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "preferred_foot",
    label: "Preferred foot",
    category: "identity",
    kind: "enum",
    operators: ENUM_OPERATORS,
    enumOptions: [
      { value: "left", label: "Left" },
      { value: "right", label: "Right" },
      { value: "either", label: "Either" },
    ],
  },
  {
    id: "club",
    label: "Club",
    category: "club-contract",
    kind: "string",
    defaultWidth: 192,
    operators: STRING_OPERATORS,
  },
  {
    id: "division",
    label: "Division",
    category: "club-contract",
    kind: "string",
    defaultWidth: 168,
    operators: STRING_OPERATORS,
  },
  {
    id: "parent_club",
    label: "Parent club",
    category: "club-contract",
    kind: "string",
    operators: STRING_OPERATORS,
  },
  {
    id: "height",
    label: "Height",
    category: "club-contract",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "wage",
    label: "Wage",
    category: "club-contract",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "value",
    label: "Value",
    category: "club-contract",
    kind: "integer",
    defaultWidth: 112,
    operators: INTEGER_OPERATORS,
  },
  {
    id: "contract_year",
    label: "Contract year",
    category: "club-contract",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "transfer_listed",
    label: "Transfer listed",
    category: "club-contract",
    kind: "boolean",
    operators: BOOLEAN_OPERATORS,
  },
  {
    id: "loan_listed",
    label: "Loan listed",
    category: "club-contract",
    kind: "boolean",
    operators: BOOLEAN_OPERATORS,
  },
  {
    id: "not_for_sale",
    label: "Not for sale",
    category: "club-contract",
    kind: "boolean",
    operators: BOOLEAN_OPERATORS,
  },
  {
    id: "set_for_release",
    label: "Set for release",
    category: "club-contract",
    kind: "boolean",
    operators: BOOLEAN_OPERATORS,
  },
  {
    id: "on_loan",
    label: "On loan",
    category: "club-contract",
    kind: "boolean",
    operators: BOOLEAN_OPERATORS,
  },
  {
    id: "team_level",
    label: "Team level",
    category: "club-contract",
    kind: "enum",
    operators: ENUM_OPERATORS,
    enumOptions: [
      { value: "senior", label: "Senior" },
      { value: "reserve", label: "Reserve" },
      { value: "youth", label: "Youth" },
    ],
  },
  {
    id: "ca",
    label: "CA",
    category: "ability-reputation",
    kind: "integer",
    defaultWidth: 72,
    operators: INTEGER_OPERATORS,
  },
  {
    id: "club_dna",
    label: "Club DNA",
    category: "ability-reputation",
    kind: "integer",
    defaultWidth: 88,
    operators: INTEGER_OPERATORS,
  },
  {
    id: "pa",
    label: "PA",
    category: "ability-reputation",
    kind: "integer",
    defaultWidth: 72,
    operators: INTEGER_OPERATORS,
  },
  {
    id: "reputation",
    label: "Reputation",
    category: "ability-reputation",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "world_reputation",
    label: "World reputation",
    category: "ability-reputation",
    kind: "integer",
    operators: INTEGER_OPERATORS,
  },
  {
    id: "position",
    label: "Position",
    category: "position-suitability",
    kind: "enum",
    defaultWidth: 144,
    operators: ENUM_OPERATORS,
    enumOptions: POSITION_ORDER.map((key) => ({ value: key, label: key })),
  },
];

export const PLAYER_METRICS: readonly PlayerMetric[] = [
  ...PLAYER_METRIC_DEFINITIONS.map(playerMetric),
  ...ATTRIBUTE_METRICS,
  ...HIDDEN_METRICS,
  ...PERSONALITY_METRICS,
  ...POSITION_SUITABILITY_METRICS,
  ...CURRENT_ROLE_SCORE_METRICS,
  ...POTENTIAL_ROLE_SCORE_METRICS,
];

const METRIC_BY_ID = new Map(
  PLAYER_METRICS.map((metric) => [metric.id, metric]),
);

export function getPlayerMetric(metricId: string): PlayerMetric | undefined {
  return METRIC_BY_ID.get(metricId);
}
