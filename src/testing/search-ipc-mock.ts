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

export function setSearchPlayersOverride(players: PlayerSummary[] | null) {
  overridePlayers = players;
}

export function resetSearchPlayersOverride() {
  overridePlayers = null;
}

function parsePaging(args: unknown): {
  offset: number;
  limit: number;
  sortBy: SearchSortField;
  sortDir: SearchSortDir;
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
  return { offset, limit, sortBy, sortDir };
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
  }
  if (cmp === 0) {
    return a.uid - b.uid;
  }
  return sortDir === "asc" ? cmp : -cmp;
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

/** Builds a paged search response from the active snapshot mock state. */
export function resolveSearchPlayersIpcMock(args: unknown): SearchPlayersPage {
  const snapshot = resolveGetCurrentSnapshotIpcMock();
  if (!snapshot) {
    return { players: [], total: 0 };
  }

  const { offset, limit, sortBy, sortDir } = parsePaging(args);
  const players = [...(overridePlayers ?? fromSanityRows())].sort((a, b) =>
    comparePlayers(a, b, sortBy, sortDir),
  );

  return {
    players: players.slice(offset, offset + limit),
    total: players.length,
  };
}
