import type {
  PlayerSummary,
  SearchPlayersPage,
} from "@/features/search/types/player-summary";
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

function parsePaging(args: unknown): { offset: number; limit: number } {
  const offset =
    typeof args === "object" &&
    args !== null &&
    "offset" in args &&
    typeof args.offset === "number"
      ? Math.max(0, args.offset)
      : 0;
  const limit =
    typeof args === "object" &&
    args !== null &&
    "limit" in args &&
    typeof args.limit === "number"
      ? Math.min(200, Math.max(1, args.limit))
      : 50;
  return { offset, limit };
}

function fromSanityRows(): PlayerSummary[] {
  return resolveListSanityPlayersIpcMock()
    .map((row, index) => ({
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
    }))
    .sort((a, b) => b.ca - a.ca || a.uid - b.uid);
}

/** Builds a paged search response from the active snapshot mock state. */
export function resolveSearchPlayersIpcMock(args: unknown): SearchPlayersPage {
  const snapshot = resolveGetCurrentSnapshotIpcMock();
  if (!snapshot) {
    return { players: [], total: 0 };
  }

  const { offset, limit } = parsePaging(args);
  const players = overridePlayers
    ? [...overridePlayers].sort((a, b) => b.ca - a.ca || a.uid - b.uid)
    : fromSanityRows();

  return {
    players: players.slice(offset, offset + limit),
    total: players.length,
  };
}
