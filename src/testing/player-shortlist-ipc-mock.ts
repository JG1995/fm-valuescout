import type { PlayerShortlistChoice } from "@/features/search/types/player-shortlist-choice";
import {
  resolveListSavesIpcMock,
  setSavePlayerShortlistsIpcMock,
} from "@/testing/snapshot-ipc-mock";

const CHOICE_COUNTS = { totalPlayers: 9, matchedPlayers: 7 };

let chooseError: unknown;
let removeError: unknown;
let lastChooseArgs: unknown;
let lastRemoveArgs: unknown;

export function resetPlayerShortlistIpcMock() {
  chooseError = undefined;
  removeError = undefined;
  lastChooseArgs = undefined;
  lastRemoveArgs = undefined;
}

/** Rejects the next adds with `error`, such as `{ kind, fileName }` as Rust sends it. */
export function setPlayerShortlistChoiceIpcMockError(error: unknown) {
  chooseError = error;
}

/** Rejects the next removals with `error`. */
export function setPlayerShortlistRemoveIpcMockError(error: unknown) {
  removeError = error;
}

export function getLastChoosePlayerShortlistIpcArgs() {
  return lastChooseArgs;
}

export function getLastRemovePlayerShortlistIpcArgs() {
  return lastRemoveArgs;
}

function numberArg(args: unknown, name: string) {
  return typeof args === "object" && args !== null && name in args
    ? Number((args as Record<string, unknown>)[name])
    : Number.NaN;
}

function shortlistsOf(saveId: number) {
  return (
    resolveListSavesIpcMock().find((save) => save.id === saveId)
      ?.playerShortlists ?? []
  );
}

/**
 * A successful add lists the file's shortlist on the save that `list_saves`
 * reports, named after the file, or updates the save's shortlist for that file.
 */
export function resolveChoosePlayerShortlistIpcMock(args: unknown) {
  lastChooseArgs = args;
  if (chooseError !== undefined) {
    return Promise.reject(chooseError);
  }
  const saveId = numberArg(args, "saveId");
  const path =
    typeof args === "object" && args !== null && "path" in args
      ? String(args.path)
      : "";
  const fileName = path.split(/[\\/]/).pop() ?? path;
  const name = fileName.replace(/\.fmf$/i, "");
  const shortlists = shortlistsOf(saveId);
  const linked = shortlists.find(
    (shortlist) => shortlist.fileName === fileName,
  );
  const id =
    linked?.id ??
    Math.max(0, ...shortlists.map((shortlist) => shortlist.id)) + 1;
  setSavePlayerShortlistsIpcMock(saveId, [
    ...shortlists.filter((shortlist) => shortlist.id !== id),
    { id, name, fileName },
  ]);
  const choice: PlayerShortlistChoice = {
    id,
    name,
    fileName,
    ...CHOICE_COUNTS,
  };
  return Promise.resolve(choice);
}

export function resolveRemovePlayerShortlistIpcMock(args: unknown) {
  lastRemoveArgs = args;
  if (removeError !== undefined) {
    return Promise.reject(removeError);
  }
  const saveId = numberArg(args, "saveId");
  const shortlistId = numberArg(args, "shortlistId");
  setSavePlayerShortlistsIpcMock(
    saveId,
    shortlistsOf(saveId).filter((shortlist) => shortlist.id !== shortlistId),
  );
  return Promise.resolve(null);
}
