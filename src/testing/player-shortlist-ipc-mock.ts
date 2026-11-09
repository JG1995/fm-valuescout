import type { PlayerShortlistChoice } from "@/features/search/types/player-shortlist-choice";
import { setSavePlayerShortlistIpcMock } from "@/testing/snapshot-ipc-mock";

const DEFAULT_CHOICE: PlayerShortlistChoice = {
  name: "Targets",
  fileName: "Targets.fmf",
  totalPlayers: 9,
  matchedPlayers: 7,
};

let choice = DEFAULT_CHOICE;
let chooseError: unknown;
let lastChooseArgs: unknown;
let lastRemoveArgs: unknown;

export function resetPlayerShortlistIpcMock() {
  choice = DEFAULT_CHOICE;
  chooseError = undefined;
  lastChooseArgs = undefined;
  lastRemoveArgs = undefined;
}

export function setPlayerShortlistChoiceIpcMockResult(
  nextChoice: PlayerShortlistChoice,
) {
  choice = nextChoice;
  chooseError = undefined;
}

/** Rejects the next choices with `error`, such as `{ kind, fileName }` as Rust sends it. */
export function setPlayerShortlistChoiceIpcMockError(error: unknown) {
  chooseError = error;
}

export function getLastChoosePlayerShortlistIpcArgs() {
  return lastChooseArgs;
}

export function getLastRemovePlayerShortlistIpcArgs() {
  return lastRemoveArgs;
}

function saveIdOf(args: unknown) {
  return typeof args === "object" && args !== null && "saveId" in args
    ? Number(args.saveId)
    : Number.NaN;
}

/** A successful choice links the shortlist to the save that `list_saves` reports. */
export function resolveChoosePlayerShortlistIpcMock(args: unknown) {
  lastChooseArgs = args;
  if (chooseError !== undefined) {
    return Promise.reject(chooseError);
  }
  setSavePlayerShortlistIpcMock(saveIdOf(args), choice);
  return Promise.resolve(choice);
}

export function resolveRemovePlayerShortlistIpcMock(args: unknown) {
  lastRemoveArgs = args;
  setSavePlayerShortlistIpcMock(saveIdOf(args), null);
  return Promise.resolve(null);
}
