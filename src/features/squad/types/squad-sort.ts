import { getPlayerMetric } from "@/utils/player-metrics";

export const SQUAD_SORT_FIELDS = [
  "name",
  "age",
  "nationality",
  "club",
  "division",
  "ca",
  "pa",
  "value",
] as const;

export type SquadSortField = string;
export type SquadSortDir = "asc" | "desc";

export const DEFAULT_SQUAD_SORT_FIELD: SquadSortField = "ca";
export const DEFAULT_SQUAD_SORT_DIR: SquadSortDir = "desc";

export function isSquadSortField(value: unknown): value is SquadSortField {
  return typeof value === "string" && getPlayerMetric(value) !== undefined;
}

export function isSquadSortDir(value: unknown): value is SquadSortDir {
  return value === "asc" || value === "desc";
}

export function defaultDirForSquadSortField(
  field: SquadSortField,
): SquadSortDir {
  switch (field) {
    case "name":
    case "nationality":
    case "club":
    case "division":
    case "parent_club":
    case "preferred_foot":
    case "team_level":
    case "position":
      return "asc";
    default:
      return "desc";
  }
}
