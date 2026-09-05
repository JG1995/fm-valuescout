import type { SaveDeleteResult } from "@/features/snapshot/api/delete-save";
import type { SnapshotGameDateUpdateResult } from "@/features/snapshot/api/update-snapshot-date";
import type { SaveSummary } from "@/features/snapshot/types/save";
import type {
  SnapshotDeleteResult,
  SnapshotMetadata,
  SnapshotSummary,
} from "@/features/snapshot/types/snapshot";

export type { SnapshotMetadata } from "@/features/snapshot/types/snapshot";

type LoadDataTimings = {
  scanMs: number;
  prepareMs: number;
  scoringMs: number;
  saveMs: number;
  finalizeMs: number;
  totalMs: number;
  ingestMs: number;
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
  | "historicalSuccess"
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
let snapshotDateEditMode: SnapshotManagementIpcMockMode = "success";
let activeSaveMode: ActiveSaveIpcMockMode = "success";
let lastLoadDataArgs: unknown;
let lastLoadDataProgressChannel: {
  onmessage?: (progress: unknown) => void;
} | null = null;
let lastSnapshotManagementArgs: unknown;
let busyDeferred: {
  promise: Promise<LoadDataResult>;
  resolve: (value: LoadDataResult) => void;
  reject: (error: unknown) => void;
  capturedSave: SaveSummary;
  capturedNextSnapshotId: number;
} | null = null;
let busySnapshotDeleteDeferred: {
  promise: Promise<SnapshotDeleteResult>;
  resolve: (value: SnapshotDeleteResult) => void;
} | null = null;
let busySnapshotDateEditDeferred: {
  promise: Promise<SnapshotGameDateUpdateResult>;
  resolve: (value: SnapshotGameDateUpdateResult) => void;
  reject: (error: unknown) => void;
  args: unknown;
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
let onGetCurrentSnapshotCall: (() => void) | undefined;
let onUpdateSnapshotDateCall: (() => void) | undefined;
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
  const defaultTimings: LoadDataTimings = {
    scanMs: 1200,
    prepareMs: 300,
    scoringMs: 400,
    saveMs: 200,
    finalizeMs: 200,
    totalMs: 2100,
    ingestMs: 400,
  };
  const { timings: overrideTimings, ...restOverrides } = overrides ?? {};
  return {
    requestId: "req-mock",
    playersFound: storedSnapshot.playerCount,
    scanTruncated: storedSnapshot.scanTruncated,
    maxAccepted: storedSnapshot.maxAccepted,
    storedSnapshot,
    effectiveSnapshot,
    timings: { ...defaultTimings, ...(overrideTimings ?? {}) },
    ...restOverrides,
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

function isCanonicalGameDate(value: string) {
  if (value.length !== 10 || value[4] !== "-" || value[7] !== "-") {
    return false;
  }
  for (let index = 0; index < value.length; index += 1) {
    if (index === 4 || index === 7) {
      continue;
    }
    const code = value.charCodeAt(index);
    if (code < 48 || code > 57) {
      return false;
    }
  }
  const year = Number(value.slice(0, 4));
  const month = Number(value.slice(5, 7));
  const day = Number(value.slice(8, 10));
  if (year < 1 || month < 1 || month > 12 || day < 1) {
    return false;
  }
  const leapYear = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  const daysInMonth =
    month === 2
      ? leapYear
        ? 29
        : 28
      : month === 4 || month === 6 || month === 9 || month === 11
        ? 30
        : 31;
  return day <= daysInMonth;
}

function applySnapshotDateEdit(args: unknown): SnapshotGameDateUpdateResult {
  const { snapshotId, contextToken } = parseSnapshotMutationArgs(args);
  const gameDate =
    typeof args === "object" && args !== null && "gameDate" in args
      ? args.gameDate
      : null;
  if (typeof gameDate !== "string" || !isCanonicalGameDate(gameDate)) {
    throw new Error("Game date must be a valid date in YYYY-MM-DD format");
  }
  const target = snapshotHistory.find(
    (snapshot) =>
      snapshot.id === snapshotId && snapshot.contextToken === contextToken,
  );
  if (!target) {
    throw new Error("Snapshot changed or no longer exists");
  }
  const previousCurrentSnapshotId =
    snapshotsForSave(target.saveId).find((snapshot) => snapshot.isCurrent)
      ?.id ?? null;
  snapshotHistory = snapshotHistory.map((snapshot) =>
    snapshot.id === target.id ? { ...snapshot, gameDate } : snapshot,
  );
  const currentSnapshotId = promoteCurrentSnapshot(target.saveId);
  const snapshot = snapshotHistory.find((entry) => entry.id === target.id);
  if (!snapshot) {
    throw new Error("Snapshot changed or no longer exists");
  }
  return {
    snapshot: copySnapshotMetadata(snapshot),
    previousCurrentSnapshotId,
    currentSnapshotId,
  };
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
  snapshotDateEditMode = "success";
  activeSaveMode = "success";
  lastLoadDataArgs = undefined;
  lastLoadDataProgressChannel = null;
  lastSnapshotManagementArgs = undefined;
  busyDeferred = null;
  busySnapshotDeleteDeferred = null;
  busySnapshotDateEditDeferred = null;
  busyActiveSaveDeferred = null;
  nextSaveId = 2;
  nextSnapshotId = 1;
  onLoadDataCall = undefined;
  onSetActiveSaveCall = undefined;
  onDeleteSnapshotCall = undefined;
  onGetCurrentSnapshotCall = undefined;
  onUpdateSnapshotDateCall = undefined;
  onDeleteSaveCall = undefined;
}

export function getLastLoadDataIpcArgs() {
  return lastLoadDataArgs;
}

export function getLastLoadDataProgressChannel(): {
  onmessage?: (progress: unknown) => void;
} | null {
  return lastLoadDataProgressChannel;
}

export function emitLoadDataProgress(progress: unknown) {
  lastLoadDataProgressChannel?.onmessage?.(progress);
}

export function getLastSnapshotManagementIpcArgs() {
  return lastSnapshotManagementArgs;
}

export function observeSnapshotIpcCall(
  command:
    | "loadData"
    | "setActiveSave"
    | "deleteSnapshot"
    | "getCurrentSnapshot"
    | "updateSnapshotDate"
    | "deleteSave",
  observer: (() => void) | undefined,
) {
  if (command === "loadData") onLoadDataCall = observer;
  if (command === "setActiveSave") onSetActiveSaveCall = observer;
  if (command === "deleteSnapshot") onDeleteSnapshotCall = observer;
  if (command === "getCurrentSnapshot") onGetCurrentSnapshotCall = observer;
  if (command === "updateSnapshotDate") onUpdateSnapshotDateCall = observer;
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

export function setSnapshotDateEditIpcMockMode(
  mode: SnapshotManagementIpcMockMode,
) {
  snapshotDateEditMode = mode;
  if (mode !== "busy") {
    busySnapshotDateEditDeferred = null;
  }
}

export function resolveUpdateSnapshotDateIpcMock(
  args: unknown,
): Promise<SnapshotGameDateUpdateResult> {
  onUpdateSnapshotDateCall?.();
  lastSnapshotManagementArgs = args;
  if (snapshotDateEditMode === "failure") {
    return Promise.reject(new Error("Snapshot date update failed"));
  }
  if (snapshotDateEditMode === "busy") {
    if (!busySnapshotDateEditDeferred) {
      let resolve!: (value: SnapshotGameDateUpdateResult) => void;
      let reject!: (error: unknown) => void;
      const promise = new Promise<SnapshotGameDateUpdateResult>((res, rej) => {
        resolve = res;
        reject = rej;
      });
      busySnapshotDateEditDeferred = { promise, resolve, reject, args };
    }
    return busySnapshotDateEditDeferred.promise;
  }
  try {
    return Promise.resolve(applySnapshotDateEdit(args));
  } catch (error) {
    return Promise.reject(error);
  }
}

export function resolveBusySnapshotDateEditRequest() {
  const deferred = busySnapshotDateEditDeferred;
  busySnapshotDateEditDeferred = null;
  if (!deferred) {
    return;
  }
  try {
    deferred.resolve(applySnapshotDateEdit(deferred.args));
  } catch (error) {
    deferred.reject(error);
  }
}

export function resolveBusyLoadDataRequest(result?: LoadDataResult) {
  if (!busyDeferred) return;
  const deferred = busyDeferred;
  busyDeferred = null;
  if (result) {
    deferred.resolve(result);
    return;
  }
  // Use captured identity at invocation, not current active save
  const previousNextSnapshotId = nextSnapshotId;
  nextSnapshotId = deferred.capturedNextSnapshotId;
  const savesSnapshot = saves;
  const activeId = deferred.capturedSave.id;
  // Temporarily restore captured save as active for snapshot building
  const originalActive = saves.find((s) => s.isActive);
  saves = saves.map((s) => ({ ...s, isActive: s.id === activeId }));
  const built = buildLoadDataResult();
  saves = savesSnapshot;
  nextSnapshotId = previousNextSnapshotId;
  // Apply side effects based on built result using captured state
  const resultToApply = built;
  // Replicate non-busy side effects but using captured save context
  // For busy we don't auto-apply history branching; caller supplied result handles it when needed
  // Here we apply the same logic as non-busy but scoped to captured save
  snapshotsBySaveId.set(activeId, {
    snapshot: resultToApply.effectiveSnapshot,
  });
  snapshotHistory = [
    ...snapshotHistory
      .filter((snapshot) => snapshot.id !== resultToApply.effectiveSnapshot.id)
      .map((snapshot) =>
        snapshot.saveId === resultToApply.effectiveSnapshot.saveId
          ? { ...snapshot, isCurrent: false }
          : snapshot,
      ),
    {
      id: resultToApply.effectiveSnapshot.id,
      contextToken: resultToApply.effectiveSnapshot.contextToken,
      saveId: resultToApply.effectiveSnapshot.saveId,
      customName: null,
      gameDate: resultToApply.effectiveSnapshot.gameDate,
      gameDateSource: resultToApply.effectiveSnapshot.gameDateSource,
      playerCount: resultToApply.effectiveSnapshot.playerCount,
      loadedAtUtc: resultToApply.effectiveSnapshot.loadedAtUtc,
      isCurrent: true,
    },
  ];
  nextSnapshotId = Math.max(
    nextSnapshotId,
    resultToApply.effectiveSnapshot.id + 1,
  );
  // Restore original active for resolution value but keep captured save id in snapshot
  void originalActive;
  deferred.resolve(resultToApply);
}

export function rejectBusyLoadDataRequest(error?: unknown) {
  if (!busyDeferred) return;
  const deferred = busyDeferred;
  busyDeferred = null;
  deferred.reject(
    error ?? { phase: "ingest", message: "dump validation failed" },
  );
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
  onGetCurrentSnapshotCall?.();
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
    throw new Error("Save name must not be empty");
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
    throw new Error("Save name must not be empty");
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
    throw new Error("Snapshot rename failed");
  }

  const { snapshotId, contextToken } = parseSnapshotMutationArgs(args);
  const customName =
    typeof args === "object" && args !== null && "customName" in args
      ? args.customName
      : null;
  if (customName !== null && typeof customName !== "string") {
    throw new Error("Snapshot name is invalid");
  }

  const index = snapshotHistory.findIndex(
    (snapshot) =>
      snapshot.id === snapshotId && snapshot.contextToken === contextToken,
  );
  if (index < 0) {
    throw new Error("Snapshot changed or no longer exists");
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
    throw new Error("Save changed or no longer exists");
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
  const channel =
    typeof args === "object" &&
    args !== null &&
    "onProgress" in args &&
    typeof (args as Record<string, unknown>).onProgress === "object" &&
    (args as Record<string, unknown>).onProgress !== null
      ? ((args as Record<string, unknown>).onProgress as {
          onmessage?: (progress: unknown) => void;
        })
      : null;
  if (channel) {
    lastLoadDataProgressChannel = channel;
  }

  if (loadDataMode === "busy") {
    if (!busyDeferred) {
      let resolve!: (value: LoadDataResult) => void;
      let reject!: (error: unknown) => void;
      const promise = new Promise<LoadDataResult>((res, rej) => {
        resolve = res;
        reject = rej;
      });
      const parsedSaveId =
        typeof args === "object" &&
        args !== null &&
        "saveId" in args &&
        typeof (args as Record<string, unknown>).saveId === "number"
          ? Number((args as Record<string, unknown>).saveId)
          : typeof args === "object" &&
              args !== null &&
              "save_id" in args &&
              typeof (args as Record<string, unknown>).save_id === "number"
            ? Number((args as Record<string, unknown>).save_id)
            : NaN;
      const parsedToken =
        typeof args === "object" &&
        args !== null &&
        "contextToken" in args &&
        typeof (args as Record<string, unknown>).contextToken === "string"
          ? String((args as Record<string, unknown>).contextToken)
          : typeof args === "object" &&
              args !== null &&
              "context_token" in args &&
              typeof (args as Record<string, unknown>).context_token ===
                "string"
            ? String((args as Record<string, unknown>).context_token)
            : "";
      if (Number.isNaN(parsedSaveId) || !parsedToken) {
        throw new Error(
          "busy Load Data mock requires saveId and contextToken identity",
        );
      }
      const existing = saves.find(
        (s) => s.id === parsedSaveId && s.contextToken === parsedToken,
      );
      const capturedSave: SaveSummary = existing
        ? { ...existing }
        : {
            id: parsedSaveId,
            contextToken: parsedToken,
            name: "Captured save",
            isActive: true,
            createdAtUtc: "2026-07-28T12:00:00.000Z",
            updatedAtUtc: "2026-07-28T12:00:00.000Z",
          };
      busyDeferred = {
        promise,
        resolve,
        reject,
        capturedSave,
        capturedNextSnapshotId: nextSnapshotId,
      };
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

  const isHistorical = loadDataMode === "historicalSuccess";
  const truncated = loadDataMode === "truncatedSuccess";
  let result: LoadDataResult;
  if (isHistorical) {
    const existingCurrent = snapshotHistory.find(
      (snapshot) => snapshot.isCurrent,
    );
    const storedSnapshot = buildSnapshot({
      scanTruncated: false,
      maxAccepted: null,
      playerCount: SAMPLE_PLAYER_COUNT,
      gameDate: "2026-07-01",
      loadedAtUtc: "2026-07-28T15:00:00.000Z",
    });
    const effectiveSnapshot = existingCurrent
      ? buildSnapshot({
          id: existingCurrent.id,
          contextToken: existingCurrent.contextToken,
          saveId: existingCurrent.saveId,
          gameDate: existingCurrent.gameDate,
          gameDateSource: existingCurrent.gameDateSource,
          playerCount: existingCurrent.playerCount,
          loadedAtUtc: existingCurrent.loadedAtUtc,
        })
      : buildSnapshot({
          id: nextSnapshotId + 1,
          contextToken: `snapshot-token-${nextSnapshotId + 1}`,
          gameDate: "2027-08-16",
          loadedAtUtc: "2027-08-16T12:00:00.000Z",
        });
    result = buildLoadDataResult({
      playersFound: storedSnapshot.playerCount,
      scanTruncated: false,
      maxAccepted: null,
      storedSnapshot,
      effectiveSnapshot,
    });
  } else {
    result = buildLoadDataResult({
      playersFound: truncated ? 500 : SAMPLE_PLAYER_COUNT,
      scanTruncated: truncated,
      maxAccepted: truncated ? 500 : null,
      storedSnapshot: buildSnapshot({
        scanTruncated: truncated,
        maxAccepted: truncated ? 500 : null,
        playerCount: truncated ? 500 : SAMPLE_PLAYER_COUNT,
      }),
    });
  }

  if (isHistorical) {
    snapshotHistory = [
      ...snapshotHistory.filter(
        (snapshot) => snapshot.id !== result.storedSnapshot.id,
      ),
      {
        id: result.storedSnapshot.id,
        contextToken: result.storedSnapshot.contextToken,
        saveId: result.storedSnapshot.saveId,
        customName: null,
        gameDate: result.storedSnapshot.gameDate,
        gameDateSource: result.storedSnapshot.gameDateSource,
        playerCount: result.storedSnapshot.playerCount,
        loadedAtUtc: result.storedSnapshot.loadedAtUtc,
        isCurrent: false,
      },
    ];
    const effectiveAlreadyCurrent = snapshotHistory.some(
      (snapshot) =>
        snapshot.id === result.effectiveSnapshot.id && snapshot.isCurrent,
    );
    if (!effectiveAlreadyCurrent) {
      snapshotsBySaveId.set(result.effectiveSnapshot.saveId, {
        snapshot: result.effectiveSnapshot,
      });
      snapshotHistory = [
        ...snapshotHistory.filter(
          (snapshot) => snapshot.id !== result.effectiveSnapshot.id,
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
    }
    nextSnapshotId = Math.max(
      nextSnapshotId,
      result.storedSnapshot.id + 1,
      result.effectiveSnapshot.id + 1,
    );
  } else {
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
  }

  return Promise.resolve(result);
}
