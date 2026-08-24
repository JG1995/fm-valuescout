import type {
  ClubDnaContext,
  ClubDnaDefinition,
  ClubDnaRemoveResult,
  ClubDnaUpsertResult,
} from "@/features/club-dna/types/club-dna";

type ClubDnaIpcMockMode = "success" | "error" | "busy";

type SetRequest = ClubDnaContext & { attributeIds: string[] };

type Deferred<T> = {
  promise: Promise<T>;
  resolve: (value: T) => void;
  reject: (error: Error) => void;
};

const definitions = new Map<string, ClubDnaDefinition>();
const getDeferred = new Map<string, Deferred<ClubDnaDefinition | null>>();
const setDeferred = new Map<string, Deferred<ClubDnaUpsertResult>>();
const removeDeferred = new Map<string, Deferred<ClubDnaRemoveResult>>();
let getMode: ClubDnaIpcMockMode = "success";
let setMode: ClubDnaIpcMockMode = "success";
let removeMode: ClubDnaIpcMockMode = "success";
let lastSetArgs: SetRequest | undefined;
let lastRemoveArgs: ClubDnaContext | undefined;

function contextKey({ saveId, contextToken }: ClubDnaContext) {
  return `${saveId}:${contextToken}`;
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

function deferredFor<T>(
  map: Map<string, Deferred<T>>,
  context: ClubDnaContext,
) {
  const key = contextKey(context);
  const existing = map.get(key);
  if (existing) {
    return existing;
  }
  const next = deferred<T>();
  map.set(key, next);
  return next;
}

function resolveDeferred<T>(
  map: Map<string, Deferred<T>>,
  context: ClubDnaContext | undefined,
  result: T,
) {
  if (context) {
    const key = contextKey(context);
    map.get(key)?.resolve(result);
    map.delete(key);
    return;
  }
  for (const entry of map.values()) {
    entry.resolve(result);
  }
  map.clear();
}

function rejectDeferred<T>(
  map: Map<string, Deferred<T>>,
  context: ClubDnaContext,
  error: Error,
) {
  const key = contextKey(context);
  map.get(key)?.reject(error);
  map.delete(key);
}

export function resetClubDnaIpcMock() {
  definitions.clear();
  getDeferred.clear();
  setDeferred.clear();
  removeDeferred.clear();
  getMode = "success";
  setMode = "success";
  removeMode = "success";
  lastSetArgs = undefined;
  lastRemoveArgs = undefined;
}

export function setClubDnaIpcMockDefinition(
  context: ClubDnaContext,
  attributeIds: string[],
) {
  definitions.set(contextKey(context), { attributeIds: [...attributeIds] });
}

export function setClubDnaGetIpcMockMode(mode: ClubDnaIpcMockMode) {
  getMode = mode;
}

export function setClubDnaSetIpcMockMode(mode: ClubDnaIpcMockMode) {
  setMode = mode;
}

export function setClubDnaRemoveIpcMockMode(mode: ClubDnaIpcMockMode) {
  removeMode = mode;
}

export function getLastClubDnaSetIpcArgs() {
  return lastSetArgs;
}

export function getLastClubDnaRemoveIpcArgs() {
  return lastRemoveArgs;
}

export function resolveClubDnaGetIpcMock(
  args: unknown,
): Promise<ClubDnaDefinition | null> {
  const context = args as ClubDnaContext;
  if (getMode === "error") {
    return Promise.reject(new Error("Could not load Club DNA"));
  }
  if (getMode === "busy") {
    return deferredFor(getDeferred, context).promise;
  }
  return Promise.resolve(definitions.get(contextKey(context)) ?? null);
}

export function resolveClubDnaSetIpcMock(
  args: unknown,
): Promise<ClubDnaUpsertResult> {
  const request = args as SetRequest;
  lastSetArgs = request;
  if (setMode === "error") {
    return Promise.reject(new Error("Could not save Club DNA"));
  }
  if (setMode === "busy") {
    return deferredFor(setDeferred, request).promise;
  }
  const key = contextKey(request);
  const created = !definitions.has(key);
  const definition = { attributeIds: [...request.attributeIds] };
  definitions.set(key, definition);
  return Promise.resolve({ definition, created });
}

export function resolveClubDnaRemoveIpcMock(
  args: unknown,
): Promise<ClubDnaRemoveResult> {
  const context = args as ClubDnaContext;
  lastRemoveArgs = context;
  if (removeMode === "error") {
    return Promise.reject(new Error("Could not remove Club DNA"));
  }
  if (removeMode === "busy") {
    return deferredFor(removeDeferred, context).promise;
  }
  const removed = definitions.delete(contextKey(context));
  return Promise.resolve({ removed });
}

export function resolveBusyClubDnaGetRequest(
  context?: ClubDnaContext,
  definition: ClubDnaDefinition | null = null,
) {
  resolveDeferred(getDeferred, context, definition);
}

export function rejectBusyClubDnaGetRequest(
  context: ClubDnaContext,
  error = new Error("Could not load Club DNA"),
) {
  rejectDeferred(getDeferred, context, error);
}

export function resolveBusyClubDnaSetRequest(
  context?: ClubDnaContext,
  result: ClubDnaUpsertResult = {
    definition: { attributeIds: ["attr.Acceleration"] },
    created: true,
  },
) {
  resolveDeferred(setDeferred, context, result);
}

export function rejectBusyClubDnaSetRequest(
  context: ClubDnaContext,
  error = new Error("Could not save Club DNA"),
) {
  rejectDeferred(setDeferred, context, error);
}

export function resolveBusyClubDnaRemoveRequest(
  context?: ClubDnaContext,
  result: ClubDnaRemoveResult = { removed: true },
) {
  resolveDeferred(removeDeferred, context, result);
}

export function rejectBusyClubDnaRemoveRequest(
  context: ClubDnaContext,
  error = new Error("Could not remove Club DNA"),
) {
  rejectDeferred(removeDeferred, context, error);
}
