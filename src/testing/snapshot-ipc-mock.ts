type SaveSummary = {
  id: number;
  name: string;
  isActive: boolean;
  createdAtUtc: string;
  updatedAtUtc: string;
};

type SnapshotSummary = {
  id: number;
  saveId: number;
  schemaVersion: number;
  generatedAtUtc: string;
  gameVersion: string;
  supportedGameVersion: string;
  bridgeVersion: string;
  protocolVersion: number;
  gameDate: string | null;
  gameDateSource: string;
  scanTruncated: boolean;
  maxAccepted: number | null;
  playerCount: number;
  loadedAtUtc: string;
};

type PlayerSanityRow = {
  name: string;
  ca: number;
  club: string | null;
};

type LoadDataTimings = {
  scanMs: number;
  ingestMs: number;
  totalMs: number;
};

type LoadDataResult = {
  requestId: string;
  playersFound: number | null;
  scanTruncated: boolean | null;
  maxAccepted: number | null;
  snapshot: SnapshotSummary;
  timings: LoadDataTimings;
};

export type LoadDataIpcMockMode =
  | "success"
  | "truncatedSuccess"
  | "scanFailed"
  | "ingestFailed"
  | "busy";

const DEFAULT_SAVE: SaveSummary = {
  id: 1,
  name: "Default save",
  isActive: true,
  createdAtUtc: "2026-07-28T12:00:00.000Z",
  updatedAtUtc: "2026-07-28T12:00:00.000Z",
};

const SAMPLE_PLAYERS: PlayerSanityRow[] = [
  { name: "Alex Morgan", ca: 165, club: "Metro FC" },
  { name: "Jordan Lee", ca: 142, club: "Riverside United" },
  { name: "Sam Rivera", ca: 178, club: null },
];

let saves: SaveSummary[] = [{ ...DEFAULT_SAVE }];
const snapshotsBySaveId = new Map<
  number,
  { snapshot: SnapshotSummary; players: PlayerSanityRow[] }
>();
let loadDataMode: LoadDataIpcMockMode = "success";
let lastLoadDataArgs: unknown;
let busyDeferred: {
  promise: Promise<LoadDataResult>;
  resolve: (value: LoadDataResult) => void;
} | null = null;
let nextSaveId = 2;

function buildSnapshot(overrides?: Partial<SnapshotSummary>): SnapshotSummary {
  const activeSave = saves.find((save) => save.isActive) ?? saves[0];
  return {
    id: 1,
    saveId: activeSave.id,
    schemaVersion: 5,
    generatedAtUtc: "2026-07-28T15:00:00.000Z",
    gameVersion: "26.0.0",
    supportedGameVersion: "26.0.0",
    bridgeVersion: "0.1.0",
    protocolVersion: 1,
    gameDate: "2026-07-01",
    gameDateSource: "inGame",
    scanTruncated: false,
    maxAccepted: null,
    playerCount: SAMPLE_PLAYERS.length,
    loadedAtUtc: "2026-07-28T15:05:00.000Z",
    ...overrides,
  };
}

function buildLoadDataResult(
  overrides?: Partial<LoadDataResult>,
): LoadDataResult {
  const snapshot = buildSnapshot(overrides?.snapshot);
  return {
    requestId: "req-mock",
    playersFound: snapshot.playerCount,
    scanTruncated: snapshot.scanTruncated,
    maxAccepted: snapshot.maxAccepted,
    snapshot,
    timings: { scanMs: 1200, ingestMs: 400, totalMs: 1600 },
    ...overrides,
  };
}

function activeSave() {
  return saves.find((save) => save.isActive) ?? saves[0];
}

function activeSaveSnapshot() {
  return snapshotsBySaveId.get(activeSave().id) ?? null;
}

export function resetSnapshotIpcMock() {
  saves = [{ ...DEFAULT_SAVE }];
  snapshotsBySaveId.clear();
  loadDataMode = "success";
  lastLoadDataArgs = undefined;
  busyDeferred = null;
  nextSaveId = 2;
}

export function getLastLoadDataIpcArgs() {
  return lastLoadDataArgs;
}

export function setLoadDataIpcMockMode(mode: LoadDataIpcMockMode) {
  loadDataMode = mode;
  if (mode !== "busy") {
    busyDeferred = null;
  }
}

export function resolveBusyLoadDataRequest(result?: LoadDataResult) {
  busyDeferred?.resolve(result ?? buildLoadDataResult());
  busyDeferred = null;
}

export function resolveListSavesIpcMock() {
  if (saves.length === 0) {
    saves = [{ ...DEFAULT_SAVE }];
  }
  return saves.map((save) => ({ ...save }));
}

export function resolveGetCurrentSnapshotIpcMock() {
  const state = activeSaveSnapshot();
  return state ? { ...state.snapshot } : null;
}

export function resolveListSanityPlayersIpcMock() {
  const state = activeSaveSnapshot();
  return state ? state.players.map((player) => ({ ...player })) : [];
}

export function resolveCreateSaveIpcMock(args: unknown) {
  const name =
    typeof args === "object" &&
    args !== null &&
    "name" in args &&
    typeof args.name === "string"
      ? args.name.trim()
      : "";

  if (!name) {
    throw "Save name must not be empty";
  }

  const created: SaveSummary = {
    id: nextSaveId,
    name,
    isActive: false,
    createdAtUtc: "2026-07-28T16:00:00.000Z",
    updatedAtUtc: "2026-07-28T16:00:00.000Z",
  };
  nextSaveId += 1;
  saves = [...saves, created];
  return created;
}

export function resolveRenameSaveIpcMock(args: unknown) {
  const saveId =
    typeof args === "object" && args !== null && "saveId" in args
      ? Number(args.saveId)
      : NaN;
  const name =
    typeof args === "object" &&
    args !== null &&
    "name" in args &&
    typeof args.name === "string"
      ? args.name.trim()
      : "";

  if (!name) {
    throw "Save name must not be empty";
  }

  const index = saves.findIndex((save) => save.id === saveId);
  if (index < 0) {
    throw `Save ${saveId} not found`;
  }

  const updated = {
    ...saves[index],
    name,
    updatedAtUtc: "2026-07-28T16:05:00.000Z",
  };
  saves = saves.map((save, currentIndex) =>
    currentIndex === index ? updated : save,
  );
  return updated;
}

export function resolveSetActiveSaveIpcMock(args: unknown) {
  const saveId =
    typeof args === "object" && args !== null && "saveId" in args
      ? Number(args.saveId)
      : NaN;

  const target = saves.find((save) => save.id === saveId);
  if (!target) {
    throw `Save ${saveId} not found`;
  }

  saves = saves.map((save) => ({
    ...save,
    isActive: save.id === saveId,
    updatedAtUtc:
      save.id === saveId ? "2026-07-28T16:10:00.000Z" : save.updatedAtUtc,
  }));

  return saves.find((save) => save.id === saveId) ?? target;
}

export function resolveLoadDataIpcMock(
  args?: unknown,
): Promise<LoadDataResult> {
  lastLoadDataArgs = args;

  if (loadDataMode === "busy") {
    if (!busyDeferred) {
      let resolve!: (value: LoadDataResult) => void;
      const promise = new Promise<LoadDataResult>((res) => {
        resolve = res;
      });
      busyDeferred = { promise, resolve };
    }
    return busyDeferred.promise;
  }

  if (loadDataMode === "scanFailed") {
    return Promise.reject({
      phase: "scan",
      kind: "failed",
      message: "scan produced zero player candidates",
    });
  }

  if (loadDataMode === "ingestFailed") {
    return Promise.reject({
      phase: "ingest",
      message: "dump validation failed",
    });
  }

  const truncated = loadDataMode === "truncatedSuccess";
  const result = buildLoadDataResult({
    playersFound: truncated ? 500 : SAMPLE_PLAYERS.length,
    scanTruncated: truncated,
    maxAccepted: truncated ? 500 : null,
    snapshot: buildSnapshot({
      scanTruncated: truncated,
      maxAccepted: truncated ? 500 : null,
      playerCount: truncated ? 500 : SAMPLE_PLAYERS.length,
    }),
  });

  const players = truncated
    ? [{ name: "Capped Player", ca: 150, club: "Cap City" }]
    : [...SAMPLE_PLAYERS];

  snapshotsBySaveId.set(activeSave().id, {
    snapshot: result.snapshot,
    players,
  });

  return Promise.resolve(result);
}
