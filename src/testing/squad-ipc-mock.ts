import type {
  SquadPlayer,
  SquadPlayersPage,
} from "@/features/squad/types/squad-player";
import type {
  SquadSortDir,
  SquadSortField,
} from "@/features/squad/types/squad-sort";
import {
  DEFAULT_SQUAD_SORT_DIR,
  DEFAULT_SQUAD_SORT_FIELD,
  isSquadSortDir,
  isSquadSortField,
} from "@/features/squad/types/squad-sort";

let overridePlayers: SquadPlayer[] | null = null;
let lastSquadPlayersArgs: Record<string, unknown> | null = null;

export function setSquadPlayersOverride(players: SquadPlayer[] | null) {
  overridePlayers = players;
}

export function resetSquadPlayersOverride() {
  overridePlayers = null;
  lastSquadPlayersArgs = null;
}

export function getLastSquadPlayersArgs(): Record<string, unknown> | null {
  return lastSquadPlayersArgs;
}

function parsePaging(args: unknown): {
  offset: number;
  limit: number;
  sortBy: SquadSortField;
  sortDir: SquadSortDir;
} {
  const record =
    typeof args === "object" && args !== null
      ? (args as Record<string, unknown>)
      : {};
  const offset =
    typeof record.offset === "number" ? Math.max(0, record.offset) : 0;
  const limit =
    typeof record.limit === "number"
      ? Math.min(200, Math.max(1, record.limit))
      : 50;
  const sortBy = isSquadSortField(record.sortBy)
    ? record.sortBy
    : DEFAULT_SQUAD_SORT_FIELD;
  const sortDir = isSquadSortDir(record.sortDir)
    ? record.sortDir
    : DEFAULT_SQUAD_SORT_DIR;
  return { offset, limit, sortBy, sortDir };
}

function compareNullableString(
  a: string | null | undefined,
  b: string | null | undefined,
): number {
  return (a ?? "").localeCompare(b ?? "", "en", { sensitivity: "base" });
}

function comparePlayers(
  a: SquadPlayer,
  b: SquadPlayer,
  sortBy: SquadSortField,
  sortDir: SquadSortDir,
): number {
  let cmp = 0;
  switch (sortBy) {
    case "name":
      cmp = compareNullableString(a.name, b.name);
      break;
    case "age":
      cmp = (a.age ?? -1) - (b.age ?? -1);
      break;
    case "nationality":
      cmp = compareNullableString(
        a.nationalities.join(", "),
        b.nationalities.join(", "),
      );
      break;
    case "club":
      cmp = compareNullableString(a.club, b.club);
      break;
    case "division":
      cmp = compareNullableString(a.division, b.division);
      break;
    case "ca":
      cmp = a.ca - b.ca;
      break;
    case "pa":
      cmp = a.pa - b.pa;
      break;
    case "value":
      cmp = (a.marketValueGbp ?? -1) - (b.marketValueGbp ?? -1);
      break;
  }
  if (cmp === 0) {
    return a.uid - b.uid;
  }
  return sortDir === "asc" ? cmp : -cmp;
}

export function resolveSquadPlayersIpcMock(args: unknown): SquadPlayersPage {
  lastSquadPlayersArgs =
    typeof args === "object" && args !== null
      ? (args as Record<string, unknown>)
      : {};
  const { offset, limit, sortBy, sortDir } = parsePaging(args);
  const players = [...(overridePlayers ?? [])].sort((a, b) =>
    comparePlayers(a, b, sortBy, sortDir),
  );
  return {
    players: players.slice(offset, offset + limit),
    total: players.length,
  };
}
