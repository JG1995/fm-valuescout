import type { SaveDeleteResult } from "@/features/snapshot/api/delete-save";
import type { SaveSummary } from "@/features/snapshot/types/save";
import type {
  SnapshotDeleteResult,
  SnapshotMetadata,
  SnapshotSummary,
} from "@/features/snapshot/types/snapshot";

export type { SnapshotMetadata } from "@/features/snapshot/types/snapshot";

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
  storedSnapshot: SnapshotSummary;
  effectiveSnapshot: SnapshotSummary;
  timings: LoadDataTimings;
};

export type LoadDataIpcMockMode =
  | "success"
  | "truncatedSuccess"
  | "scanFailed"
  | "ingestFailed"
  | "busy";

export type SnapshotManagementIpcMockMode = "success" | "failure" | "busy";
export type ActiveSaveIpcMockMode = "success" | "busy";

const DEFAULT_SAVE: SaveSummary = {
  id: 1,
  contextToken: "save-token-1",
  name: "Default save",
  isActive: true,
  createdAtUtc: "2026-07-28T12:00:00.000Z",
  updatedAtUtc: "2026-07-28T12:00:00.000Z",
};

const SAMPLE_PLAYER_COUNT = 3;

let saves: SaveSummary[] = [{ ...DEFAULT_SAVE }];
const snapshotsBySaveId = new Map<number, { snapshot: SnapshotSummary }>();
let snapshotHistory: SnapshotMetadata[] = [];
let loadDataMode: LoadDataIpcMockMode = "success";
let snapshotDeleteMode: SnapshotManagementIpcMockMode = "success";
let snapshotRenameMode: SnapshotManagementIpcMockMode = "success";
let activeSaveMode: ActiveSaveIpcMockMode = "success";
let lastLoadDataArgs: unknown;
let lastSnapshotManagementArgs: unknown;
let busyDeferred: {
  promise: Promise<LoadDataResult>;
  resolve: (value: LoadDataResult) => void;
} | null = null;
let busySnapshotDeleteDeferred: {
  promise: Promise<SnapshotDeleteResult>;
  resolve: (value: SnapshotDeleteResult) => void;
} | null = null;
let busyActiveSaveDeferred: {
  promise: Promise<SaveSummary>;
  resolve: () => void;
} | null = null;
let nextSaveId = 2;
let nextSnapshotId = 1;
let onLoadDataCall: (() => void) | undefined;
let onSetActiveSaveCall: (() => void) | undefined;
let onDeleteSnapshotCall: (() => void) | undefined;
let onDeleteSaveCall: (() => void) | undefined;

function buildSnapshot(overrides?: Partial<SnapshotSummary>): SnapshotSummary {
  const activeSave = saves.find((save) => save.isActive) ?? saves[0];
  return {
    id: nextSnapshotId,
    contextToken: `snapshot-token-${nextSnapshotId}`,
    saveId: activeSave.id,
    schemaVersion: 6,
    generatedAtUtc: "2026-07-28T15:00:00.000Z",
    gameVersion: "26.0.0",
    supportedGameVersion: "26.0.0",
    bridgeVersion: "0.1.0",
    protocolVersion: 1,
    gameDate: "2026-07-01",
    gameDateSource: "inGame",
    scanTruncated: false,
    maxAccepted: null,
    playerCount: SAMPLE_PLAYER_COUNT,
    loadedAtUtc: "2026-07-28T15:05:00.000Z",
    ...overrides,
  };
}

function buildLoadDataResult(
  overrides?: Partial<LoadDataResult>,
): LoadDataResult {
  const storedSnapshot = buildSnapshot(overrides?.storedSnapshot);
  const effectiveSnapshot = overrides?.effectiveSnapshot
    ? buildSnapshot(overrides.effectiveSnapshot)
    : storedSnapshot;
  return {
    requestId: "req-mock",
    playersFound: storedSnapshot.playerCount,
    scanTruncated: storedSnapshot.scanTruncated,
    maxAccepted: storedSnapshot.maxAccepted,
    storedSnapshot,
    effectiveSnapshot,
    timings: { scanMs: 1200, ingestMs: 400, totalMs: 1600 },
    ...overrides,
  };
}

function activeSave() {
  return saves.find((save) => save.isActive) ?? saves[0];
}

function historySnapshotState(snapshot: SnapshotMetadata) {
  return {
    snapshot: buildSnapshot({
      id: snapshot.id,
      contextToken: snapshot.contextToken,
      saveId: snapshot.saveId,
      gameDate: snapshot.gameDate,
      gameDateSource: snapshot.gameDateSource,
      playerCount: snapshot.playerCount,
      loadedAtUtc: snapshot.loadedAtUtc,
    }),
  };
}

function activeSaveSnapshot() {
  const storedSnapshot = snapshotsBySaveId.get(activeSave().id) ?? null;
  const currentHistorySnapshot = snapshotHistory.find(
    (snapshot) => snapshot.saveId === activeSave().id && snapshot.isCurrent,
  );
  if (
    currentHistorySnapshot &&
    storedSnapshot?.snapshot.id === currentHistorySnapshot.id
  ) {
    return storedSnapshot;
  }
  return currentHistorySnapshot
    ? historySnapshotState(currentHistorySnapshot)
    : storedSnapshot;
}

function snapshotOrder(left: SnapshotMetadata, right: SnapshotMetadata) {
  if (left.gameDate === null && right.gameDate !== null) {
    return 1;
  }
  if (left.gameDate !== null && right.gameDate === null) {
    return -1;
  }
  if (left.gameDate !== right.gameDate) {
    return (right.gameDate ?? "").localeCompare(left.gameDate ?? "");
  }
  if (left.loadedAtUtc !== right.loadedAtUtc) {
    return right.loadedAtUtc.localeCompare(left.loadedAtUtc);
  }
  return right.id - left.id;
}

function copySnapshotMetadata(snapshot: SnapshotMetadata): SnapshotMetadata {
  return { ...snapshot };
}

function snapshotsForSave(saveId: number) {
  return snapshotHistory
    .filter((snapshot) => snapshot.saveId === saveId)
    .sort(snapshotOrder);
}

function promoteCurrentSnapshot(saveId: number) {
  const nextCurrent = snapshotsForSave(saveId)[0] ?? null;
  snapshotHistory = snapshotHistory.map((snapshot) =>
    snapshot.saveId === saveId
      ? { ...snapshot, isCurrent: snapshot.id === nextCurrent?.id }
      : snapshot,
  );
  return nextCurrent?.id ?? null;
}

function parseSnapshotMutationArgs(args: unknown) {
  const snapshotId =
    typeof args === "object" && args !== null && "snapshotId" in args
      ? Number(args.snapshotId)
      : NaN;
  const contextToken =
    typeof args === "object" && args !== null && "contextToken" in args
      ? args.contextToken
      : null;
  return {
    snapshotId,
    contextToken: typeof contextToken === "string" ? contextToken : "",
  };
}

function parseSaveDeleteArgs(args: unknown) {
  const saveId =
    typeof args === "object" && args !== null && "saveId" in args
      ? Number(args.saveId)
      : NaN;
  const contextToken =
    typeof args === "object" && args !== null && "contextToken" in args
      ? args.contextToken
      : null;
  return {
    saveId,
    contextToken: typeof contextToken === "string" ? contextToken : "",
  };
}

export function resetSnapshotIpcMock() {
  saves = [{ ...DEFAULT_SAVE }];
  snapshotsBySaveId.clear();
  snapshotHistory = [];
  loadDataMode = "success";
  snapshotDeleteMode = "success";
  snapshotRenameMode = "success";
  activeSaveMode = "success";
  lastLoadDataArgs = undefined;
  lastSnapshotManagementArgs = undefined;
  busyDeferred = null;
  busySnapshotDeleteDeferred = null;
  busyActiveSaveDeferred = null;
  nextSaveId = 2;
  nextSnapshotId = 1;
  onLoadDataCall = undefined;
  onSetActiveSaveCall = undefined;
  onDeleteSnapshotCall = undefined;
  onDeleteSaveCall = undefined;
}

export function getLastLoadDataIpcArgs() {
  return lastLoadDataArgs;
}

export function getLastSnapshotManagementIpcArgs() {
  return lastSnapshotManagementArgs;
}

export function observeSnapshotIpcCall(
  command: "loadData" | "setActiveSave" | "deleteSnapshot" | "deleteSave",
  observer: (() => void) | undefined,
) {
  if (command === "loadData") onLoadDataCall = observer;
  if (command === "setActiveSave") onSetActiveSaveCall = observer;
  if (command === "deleteSnapshot") onDeleteSnapshotCall = observer;
  if (command === "deleteSave") onDeleteSaveCall = observer;
}

export function setLoadDataIpcMockMode(mode: LoadDataIpcMockMode) {
  loadDataMode = mode;
  if (mode !== "busy") {
    busyDeferred = null;
  }
}

export function setActiveSaveIpcMockMode(mode: ActiveSaveIpcMockMode) {
  activeSaveMode = mode;
  if (mode !== "busy") {
    busyActiveSaveDeferred = null;
  }
}

export function resolvePendingSetActiveSaveIpcMock() {
  busyActiveSaveDeferred?.resolve();
  busyActiveSaveDeferred = null;
}

export function setSnapshotHistoryIpcMock(snapshots: SnapshotMetadata[]) {
  snapshotHistory = snapshots.map(copySnapshotMetadata);
  nextSnapshotId = Math.max(
    nextSnapshotId,
    ...snapshotHistory.map((snapshot) => snapshot.id + 1),
  );
}

export function setSnapshotDeleteIpcMockMode(
  mode: SnapshotManagementIpcMockMode,
) {
  snapshotDeleteMode = mode;
  if (mode !== "busy") {
    busySnapshotDeleteDeferred = null;
  }
}

export function setSnapshotRenameIpcMockMode(
  mode: SnapshotManagementIpcMockMode,
) {
  snapshotRenameMode = mode;
}

export function resolveBusyLoadDataRequest(result?: LoadDataResult) {
  busyDeferred?.resolve(result ?? buildLoadDataResult());
  busyDeferred = null;
}

export function resolveBusySnapshotDeleteRequest(
  result?: SnapshotDeleteResult,
) {
  busySnapshotDeleteDeferred?.resolve(
    result ?? {
      deletedSnapshotId: 0,
      saveId: 0,
      currentSnapshotId: null,
    },
  );
  busySnapshotDeleteDeferred = null;
}

export function resolveListSavesIpcMock() {
  if (saves.length === 0) {
    saves = [{ ...DEFAULT_SAVE }];
  }
  return saves.map((save) => ({ ...save }));
}

export function resolveListSnapshotsIpcMock(args: unknown) {
  const saveId =
    typeof args === "object" && args !== null && "saveId" in args
      ? Number(args.saveId)
      : activeSave().id;
  return snapshotsForSave(saveId).map(copySnapshotMetadata);
}

export function resolveGetCurrentSnapshotIpcMock() {
  const state = activeSaveSnapshot();
  return state ? { ...state.snapshot } : null;
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
    contextToken: `save-token-${nextSaveId}`,
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
  onSetActiveSaveCall?.();
  const saveId =
    typeof args === "object" && args !== null && "saveId" in args
      ? Number(args.saveId)
      : NaN;

  const target = saves.find((save) => save.id === saveId);
  if (!target) {
    throw `Save ${saveId} not found`;
  }

  const setActiveSave = () => {
    saves = saves.map((save) => ({
      ...save,
      isActive: save.id === saveId,
      updatedAtUtc:
        save.id === saveId ? "2026-07-28T16:10:00.000Z" : save.updatedAtUtc,
    }));
    return saves.find((save) => save.id === saveId) ?? target;
  };

  if (activeSaveMode === "busy") {
    if (!busyActiveSaveDeferred) {
      let resolve!: () => void;
      const promise = new Promise<SaveSummary>((res) => {
        resolve = () => res(setActiveSave());
      });
      busyActiveSaveDeferred = { promise, resolve };
    }
    return busyActiveSaveDeferred.promise;
  }

  return setActiveSave();
}

export function resolveRenameSnapshotIpcMock(args: unknown) {
  lastSnapshotManagementArgs = args;
  if (snapshotRenameMode === "failure") {
    throw "Snapshot rename failed";
  }

  const { snapshotId, contextToken } = parseSnapshotMutationArgs(args);
  const customName =
    typeof args === "object" && args !== null && "customName" in args
      ? args.customName
      : null;
  if (customName !== null && typeof customName !== "string") {
    throw "Snapshot name is invalid";
  }

  const index = snapshotHistory.findIndex(
    (snapshot) =>
      snapshot.id === snapshotId && snapshot.contextToken === contextToken,
  );
  if (index < 0) {
    throw "Snapshot changed or no longer exists";
  }

  const updated = {
    ...snapshotHistory[index],
    customName: customName?.trim() || null,
  };
  snapshotHistory = snapshotHistory.map((snapshot, currentIndex) =>
    currentIndex === index ? updated : snapshot,
  );
  return copySnapshotMetadata(updated);
}

export function resolveDeleteSnapshotIpcMock(
  args: unknown,
): Promise<SnapshotDeleteResult> {
  onDeleteSnapshotCall?.();
  lastSnapshotManagementArgs = args;
  if (snapshotDeleteMode === "failure") {
    return Promise.reject("Snapshot deletion failed");
  }

  const { snapshotId, contextToken } = parseSnapshotMutationArgs(args);
  const target = snapshotHistory.find(
    (snapshot) =>
      snapshot.id === snapshotId && snapshot.contextToken === contextToken,
  );
  if (!target) {
    return Promise.reject("Snapshot changed or no longer exists");
  }

  const deleteSnapshot = () => {
    snapshotHistory = snapshotHistory.filter(
      (snapshot) => snapshot.id !== target.id,
    );
    const currentSnapshotId = target.isCurrent
      ? promoteCurrentSnapshot(target.saveId)
      : (snapshotsForSave(target.saveId).find((snapshot) => snapshot.isCurrent)
          ?.id ?? null);
    return {
      deletedSnapshotId: target.id,
      saveId: target.saveId,
      currentSnapshotId,
    };
  };

  if (snapshotDeleteMode === "busy") {
    if (!busySnapshotDeleteDeferred) {
      let resolve!: (value: SnapshotDeleteResult) => void;
      const promise = new Promise<SnapshotDeleteResult>((res) => {
        resolve = res;
      });
      busySnapshotDeleteDeferred = { promise, resolve };
    }
    return busySnapshotDeleteDeferred.promise;
  }

  return Promise.resolve(deleteSnapshot());
}

export function resolveDeleteSaveIpcMock(args: unknown): SaveDeleteResult {
  onDeleteSaveCall?.();
  lastSnapshotManagementArgs = args;
  const { saveId, contextToken } = parseSaveDeleteArgs(args);
  const target = saves.find(
    (save) => save.id === saveId && save.contextToken === contextToken,
  );
  if (!target) {
    throw "Save changed or no longer exists";
  }

  saves = saves.filter((save) => save.id !== target.id);
  snapshotHistory = snapshotHistory.filter(
    (snapshot) => snapshot.saveId !== target.id,
  );
  snapshotsBySaveId.delete(target.id);

  if (target.isActive) {
    const fallback = saves[0];
    if (fallback) {
      saves = saves.map((save) => ({
        ...save,
        isActive: save.id === fallback.id,
      }));
    } else {
      const replacement: SaveSummary = {
        id: target.id,
        contextToken: `save-token-${target.id}-replacement`,
        name: "Default save",
        isActive: true,
        createdAtUtc: "2026-07-28T16:20:00.000Z",
        updatedAtUtc: "2026-07-28T16:20:00.000Z",
      };
      saves = [replacement];
    }
  }

  return {
    deletedSaveId: target.id,
    deletedWasActive: target.isActive,
    activeSave: { ...activeSave() },
  };
}

export function resolveLoadDataIpcMock(
  args?: unknown,
): Promise<LoadDataResult> {
  onLoadDataCall?.();
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
    playersFound: truncated ? 500 : SAMPLE_PLAYER_COUNT,
    scanTruncated: truncated,
    maxAccepted: truncated ? 500 : null,
    storedSnapshot: buildSnapshot({
      scanTruncated: truncated,
      maxAccepted: truncated ? 500 : null,
      playerCount: truncated ? 500 : SAMPLE_PLAYER_COUNT,
    }),
  });

  snapshotsBySaveId.set(activeSave().id, {
    snapshot: result.effectiveSnapshot,
  });
  snapshotHistory = [
    ...snapshotHistory
      .filter((snapshot) => snapshot.id !== result.effectiveSnapshot.id)
      .map((snapshot) =>
        snapshot.saveId === result.effectiveSnapshot.saveId
          ? { ...snapshot, isCurrent: false }
          : snapshot,
      ),
    {
      id: result.effectiveSnapshot.id,
      contextToken: result.effectiveSnapshot.contextToken,
      saveId: result.effectiveSnapshot.saveId,
      customName: null,
      gameDate: result.effectiveSnapshot.gameDate,
      gameDateSource: result.effectiveSnapshot.gameDateSource,
      playerCount: result.effectiveSnapshot.playerCount,
      loadedAtUtc: result.effectiveSnapshot.loadedAtUtc,
      isCurrent: true,
    },
  ];
  nextSnapshotId += 1;

  return Promise.resolve(result);
}
