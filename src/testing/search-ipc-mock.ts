import type { FilterRuleIpc } from "@/features/search/types/filter-rule";
import type { PlayerSuggestHit } from "@/features/search/types/player-suggest-hit";
import type {
  PlayerSummary,
  SearchPlayersPage,
} from "@/features/search/types/player-summary";
import type {
  SearchSortDir,
  SearchSortField,
} from "@/features/search/types/search-sort";
import {
  DEFAULT_SEARCH_SORT_DIR,
  DEFAULT_SEARCH_SORT_FIELD,
  isSearchSortDir,
  isSearchSortField,
} from "@/features/search/types/search-sort";
import {
  resolveGetCurrentSnapshotIpcMock,
  resolveListSanityPlayersIpcMock,
} from "@/testing/snapshot-ipc-mock";

let overridePlayers: PlayerSummary[] | null = null;
let lastSearchPlayersArgs: Record<string, unknown> | null = null;
let suggestOverride: PlayerSuggestHit[] | null = null;
let lastSuggestPlayersArgs: Record<string, unknown> | null = null;

export function setSearchPlayersOverride(players: PlayerSummary[] | null) {
  overridePlayers = players;
}

export function setSuggestPlayersOverride(hits: PlayerSuggestHit[] | null) {
  suggestOverride = hits;
}

export function resetSearchPlayersOverride() {
  overridePlayers = null;
  lastSearchPlayersArgs = null;
  suggestOverride = null;
  lastSuggestPlayersArgs = null;
}

export function getLastSearchPlayersArgs(): Record<string, unknown> | null {
  return lastSearchPlayersArgs;
}

export function getLastSuggestPlayersArgs(): Record<string, unknown> | null {
  return lastSuggestPlayersArgs;
}

function parsePaging(args: unknown): {
  offset: number;
  limit: number;
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
  filters: FilterRuleIpc[];
  filterCombine: "and" | "or";
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
  const sortBy = isSearchSortField(record.sortBy)
    ? record.sortBy
    : DEFAULT_SEARCH_SORT_FIELD;
  const sortDir = isSearchSortDir(record.sortDir)
    ? record.sortDir
    : DEFAULT_SEARCH_SORT_DIR;
  const filterCombine =
    record.filterCombine === "or" ? ("or" as const) : ("and" as const);
  const filters = Array.isArray(record.filters)
    ? record.filters.filter(isFilterRuleIpc)
    : [];
  return { offset, limit, sortBy, sortDir, filters, filterCombine };
}

function isFilterRuleIpc(value: unknown): value is FilterRuleIpc {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const record = value as Record<string, unknown>;
  const valueField = record.value;
  return (
    typeof record.field === "string" &&
    typeof record.op === "string" &&
    (typeof valueField === "string" ||
      typeof valueField === "number" ||
      typeof valueField === "boolean")
  );
}

function compareNullableString(
  a: string | null | undefined,
  b: string | null | undefined,
): number {
  const left = a ?? "";
  const right = b ?? "";
  return left.localeCompare(right, "en", { sensitivity: "base" });
}

function comparePlayers(
  a: PlayerSummary,
  b: PlayerSummary,
  sortBy: SearchSortField,
  sortDir: SearchSortDir,
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
    default: {
      const left = a.dynamicValues?.[sortBy];
      const right = b.dynamicValues?.[sortBy];
      if (typeof left === "number" && typeof right === "number") {
        cmp = left - right;
      } else {
        cmp = compareNullableString(
          left === undefined || left === null ? null : String(left),
          right === undefined || right === null ? null : String(right),
        );
      }
      break;
    }
  }
  if (cmp === 0) {
    return a.uid - b.uid;
  }
  return sortDir === "asc" ? cmp : -cmp;
}

function fieldValue(
  player: PlayerSummary,
  field: string,
): string | number | null {
  switch (field) {
    case "name":
      return player.name;
    case "club":
      return player.club;
    case "division":
      return player.division;
    case "age":
      return player.age;
    case "ca":
      return player.ca;
    case "pa":
      return player.pa;
    case "value":
      return player.marketValueGbp;
    default: {
      const dynamic = player.dynamicValues?.[field];
      if (dynamic === undefined || dynamic === null) {
        return null;
      }
      return dynamic;
    }
  }
}

function matchFilterRule(player: PlayerSummary, rule: FilterRuleIpc): boolean {
  const raw = fieldValue(player, rule.field);
  const { op } = rule;

  if (typeof rule.value === "boolean") {
    return false;
  }

  if (
    rule.field === "name" ||
    rule.field === "club" ||
    rule.field === "division"
  ) {
    const text = raw === null ? "" : String(raw);
    const needle = String(rule.value);
    const haystack = text.toLowerCase();
    const target = needle.toLowerCase();
    switch (op) {
      case "contains":
        return haystack.includes(target);
      case "not_contains":
        return raw !== null && !haystack.includes(target);
      case "is":
        return raw !== null && haystack === target;
      case "is_not":
        return raw !== null && haystack !== target;
      default:
        return false;
    }
  }

  if (typeof rule.value === "string") {
    return false;
  }

  const number = typeof raw === "number" ? raw : null;
  if (number === null) {
    return false;
  }

  switch (op) {
    case "gt":
      return number > rule.value;
    case "lt":
      return number < rule.value;
    case "eq":
      return number === rule.value;
    case "neq":
      return number !== rule.value;
    default:
      return false;
  }
}

function applyFilters(
  players: PlayerSummary[],
  filters: FilterRuleIpc[],
  combine: "and" | "or",
): PlayerSummary[] {
  if (filters.length === 0) {
    return players;
  }

  return players.filter((player) => {
    const matches = filters.map((rule) => matchFilterRule(player, rule));
    return combine === "and" ? matches.every(Boolean) : matches.some(Boolean);
  });
}

function fromSanityRows(): PlayerSummary[] {
  return resolveListSanityPlayersIpcMock().map((row, index) => ({
    uid: index + 1,
    name: row.name,
    age: 25,
    birthYear: 2001,
    birthDayOfYear: 80,
    nationalities: ["ENG"],
    club: row.club,
    division: row.club ? "Premier Division" : null,
    ca: row.ca,
    pa: row.ca + 10,
    marketValueGbp: row.ca * 100_000,
  }));
}

/** Ranked name suggestions for the top-bar global search. */
export function resolveSuggestPlayersIpcMock(
  args: unknown,
): PlayerSuggestHit[] {
  lastSuggestPlayersArgs =
    typeof args === "object" && args !== null
      ? (args as Record<string, unknown>)
      : {};

  const record = lastSuggestPlayersArgs;
  const query = typeof record.query === "string" ? record.query.trim() : "";
  if (query.length === 0) {
    return [];
  }

  if (suggestOverride) {
    return suggestOverride;
  }

  const snapshot = resolveGetCurrentSnapshotIpcMock();
  if (!snapshot) {
    return [];
  }

  const limit =
    typeof record.limit === "number"
      ? Math.min(20, Math.max(1, record.limit))
      : 10;
  const needle = query.toLowerCase();
  const players = overridePlayers ?? fromSanityRows();

  const ranked = players
    .filter((player) => player.name.toLowerCase().includes(needle))
    .map((player) => {
      const name = player.name.toLowerCase();
      const tier = name === needle ? 0 : name.startsWith(needle) ? 1 : 2;
      return { player, tier };
    })
    .sort((a, b) => {
      if (a.tier !== b.tier) {
        return a.tier - b.tier;
      }
      if (a.player.ca !== b.player.ca) {
        return b.player.ca - a.player.ca;
      }
      return a.player.uid - b.player.uid;
    })
    .slice(0, limit);

  return ranked.map(({ player }) => ({
    uid: player.uid,
    name: player.name,
    ca: player.ca,
  }));
}

/** Builds a paged search response from the active snapshot mock state. */
export function resolveSearchPlayersIpcMock(args: unknown): SearchPlayersPage {
  lastSearchPlayersArgs =
    typeof args === "object" && args !== null
      ? (args as Record<string, unknown>)
      : {};

  const snapshot = resolveGetCurrentSnapshotIpcMock();
  if (!snapshot) {
    return { players: [], total: 0 };
  }

  const { offset, limit, sortBy, sortDir, filters, filterCombine } =
    parsePaging(args);
  const players = applyFilters(
    [...(overridePlayers ?? fromSanityRows())],
    filters,
    filterCombine,
  ).sort((a, b) => comparePlayers(a, b, sortBy, sortDir));

  return {
    players: players.slice(offset, offset + limit),
    total: players.length,
  };
}
