import type {
  AcademyCandidate,
  AcademyClass,
  AcademyClassDetail,
  AcademyMember,
} from "@/features/academy/types/academy";

let classes: AcademyClass[] = [];
let nextClassId = 1;
let createError: string | null = null;
let deleteError: string | null = null;
let candidates: AcademyCandidate[] = [];
let membersByClass = new Map<number, AcademyMember[]>();
let assignError: string | null = null;
let removeError: string | null = null;
let outcomeError: string | null = null;
let deferredAssignment: Promise<void> | null = null;
let resolveDeferredAssignment: (() => void) | null = null;
let deferredRemoval: Promise<void> | null = null;
let resolveDeferredRemoval: (() => void) | null = null;
let deferredClassesFetch: {
  promise: Promise<AcademyClass[]>;
  resolve: (value: AcademyClass[]) => void;
} | null = null;

type AcademyClassInput = Omit<AcademyClass, "isAutomatic"> & {
  isAutomatic?: boolean;
};

export function resetAcademyIpcMock() {
  classes = [];
  nextClassId = 1;
  createError = null;
  deleteError = null;
  candidates = [];
  membersByClass = new Map();
  assignError = null;
  removeError = null;
  outcomeError = null;
  deferredAssignment = null;
  resolveDeferredAssignment = null;
  deferredRemoval = null;
  resolveDeferredRemoval = null;
  deferredClassesFetch = null;
}

export function setAcademyClasses(value: AcademyClassInput[]) {
  classes = value
    .map((academyClass) => ({
      ...academyClass,
      isAutomatic: academyClass.isAutomatic ?? false,
    }))
    .sort((left, right) => left.classYear - right.classYear);
  nextClassId =
    Math.max(0, ...classes.map((academyClass) => academyClass.id)) + 1;
}

export function setAcademyCreateError(message: string | null) {
  createError = message;
}

export function setAcademyDeleteError(message: string | null) {
  deleteError = message;
}

export function setAcademyCandidates(value: AcademyCandidate[]) {
  candidates = value.map((candidate) => ({ ...candidate }));
}

export function setAcademyClassMembers(
  classId: number,
  members: AcademyMember[],
) {
  membersByClass.set(
    classId,
    members.map((member) => ({
      ...member,
      outcome: member.outcome ? { ...member.outcome } : null,
    })),
  );
}

export function setAcademyAssignError(message: string | null) {
  assignError = message;
}

export function deferAcademyAssignment() {
  deferredAssignment = new Promise<void>((resolve) => {
    resolveDeferredAssignment = resolve;
  });
  return () => resolveDeferredAssignment?.();
}

export function setAcademyRemoveError(message: string | null) {
  removeError = message;
}

export function setAcademyOutcomeError(message: string | null) {
  outcomeError = message;
}

export function deferAcademyRemoval() {
  deferredRemoval = new Promise<void>((resolve) => {
    resolveDeferredRemoval = resolve;
  });
  return () => resolveDeferredRemoval?.();
}

export function deferAcademyClassesFetch() {
  let resolve: (value: AcademyClass[]) => void = () => undefined;
  const promise = new Promise<AcademyClass[]>((next) => {
    resolve = next;
  });
  const deferred = { promise, resolve };
  deferredClassesFetch = deferred;
  return () => {
    if (deferredClassesFetch !== deferred) {
      return;
    }
    deferredClassesFetch = null;
    resolve(classes.map((academyClass) => ({ ...academyClass })));
  };
}

export function resolveListAcademyClassesIpcMock() {
  if (deferredClassesFetch) {
    return deferredClassesFetch.promise;
  }
  return classes.map((academyClass) => ({ ...academyClass }));
}

export function resolveCreateAcademyClassIpcMock(args: unknown) {
  if (createError) {
    throw createError;
  }

  const classYear =
    typeof args === "object" &&
    args !== null &&
    "classYear" in args &&
    typeof args.classYear === "number"
      ? args.classYear
      : NaN;
  if (!Number.isInteger(classYear) || classYear <= 0) {
    throw "Class year must be a positive integer";
  }
  if (classes.some((academyClass) => academyClass.classYear === classYear)) {
    throw `Class of ${classYear} already exists`;
  }

  const created: AcademyClass = {
    id: nextClassId,
    classYear,
    isAutomatic: false,
    memberCount: 0,
  };
  nextClassId += 1;
  classes = [...classes, created].sort(
    (left, right) => left.classYear - right.classYear,
  );
  return { ...created };
}

export function resolveDeleteAcademyClassIpcMock(args: unknown) {
  if (deleteError) {
    throw deleteError;
  }

  const classId =
    typeof args === "object" &&
    args !== null &&
    "classId" in args &&
    typeof args.classId === "number"
      ? args.classId
      : NaN;
  const confirmed =
    typeof args === "object" &&
    args !== null &&
    "confirmed" in args &&
    args.confirmed === true;
  if (!confirmed) {
    throw "Deleting an academy class requires confirmation";
  }
  if (
    classes.find((academyClass) => academyClass.id === classId)?.isAutomatic
  ) {
    throw "Automatically managed academy classes cannot be deleted";
  }

  const remaining = classes.filter(
    (academyClass) => academyClass.id !== classId,
  );
  if (remaining.length === classes.length) {
    throw `Academy class ${classId} not found`;
  }
  classes = remaining;
  membersByClass.delete(classId);
}

export function resolveGetAcademyClassIpcMock(
  args: unknown,
): AcademyClassDetail {
  const classId = readNumberArg(args, "classId");
  const academyClass = classes.find((candidate) => candidate.id === classId);
  if (!academyClass) {
    throw `Academy class ${classId} not found`;
  }
  return {
    ...academyClass,
    members: (membersByClass.get(classId) ?? []).map((member) => ({
      ...member,
      outcome: member.outcome ? { ...member.outcome } : null,
    })),
  };
}

export function resolveListAcademyCandidatesIpcMock(args: unknown) {
  const search = readStringArg(args, "search").trim().toLowerCase();
  const assigned = new Set(
    Array.from(membersByClass.values()).flatMap((members) =>
      members.map((member) => member.playerUid),
    ),
  );
  return candidates
    .filter(
      (candidate) =>
        !assigned.has(candidate.playerUid) &&
        candidate.name.toLowerCase().includes(search),
    )
    .map((candidate) => ({ ...candidate }));
}

export function resolveAssignAcademyMemberIpcMock(args: unknown) {
  if (assignError) {
    throw assignError;
  }
  const classId = readNumberArg(args, "classId");
  const playerUid = readNumberArg(args, "playerUid");
  if (!classes.some((academyClass) => academyClass.id === classId)) {
    throw `Academy class ${classId} not found`;
  }
  if (
    Array.from(membersByClass.values())
      .flat()
      .some((member) => member.playerUid === playerUid)
  ) {
    throw `Player ${playerUid} is already assigned to an academy class`;
  }
  const candidate = candidates.find(
    (academyCandidate) => academyCandidate.playerUid === playerUid,
  );
  if (!candidate) {
    throw `Player ${playerUid} is not an eligible academy candidate`;
  }
  const assignMember = () => {
    const members = membersByClass.get(classId) ?? [];
    membersByClass.set(classId, [
      ...members,
      academyMemberFromCandidate(candidate),
    ]);
    updateClassMemberCount(classId, members.length + 1);
  };
  const deferred = deferredAssignment;
  if (deferred) {
    return deferred.then(() => {
      if (deferredAssignment === deferred) {
        deferredAssignment = null;
        resolveDeferredAssignment = null;
      }
      assignMember();
    });
  }
  assignMember();
}

export function resolveRemoveAcademyMemberIpcMock(args: unknown) {
  if (removeError) {
    throw removeError;
  }
  const classId = readNumberArg(args, "classId");
  const playerUid = readNumberArg(args, "playerUid");
  const removeMember = () => {
    const members = membersByClass.get(classId) ?? [];
    const remaining = members.filter(
      (member) => member.playerUid !== playerUid,
    );
    if (remaining.length === members.length) {
      throw `Player ${playerUid} is not assigned to academy class ${classId}`;
    }
    membersByClass.set(classId, remaining);
    updateClassMemberCount(classId, remaining.length);
  };
  const deferred = deferredRemoval;
  if (deferred) {
    return deferred.then(() => {
      if (deferredRemoval === deferred) {
        deferredRemoval = null;
        resolveDeferredRemoval = null;
      }
      removeMember();
    });
  }
  removeMember();
}

export function resolveSetAcademyMemberOutcomeIpcMock(args: unknown) {
  if (outcomeError) {
    throw outcomeError;
  }
  const classId = readNumberArg(args, "classId");
  const playerUid = readNumberArg(args, "playerUid");
  const outcome = readOutcomeArg(args);
  const members = membersByClass.get(classId) ?? [];
  const member = members.find((candidate) => candidate.playerUid === playerUid);
  if (!member) {
    throw `Player ${playerUid} is not assigned to academy class ${classId}`;
  }
  if (outcome?.status === "sold") {
    if (!outcome.buyingClub?.trim()) {
      throw "Sale outcomes require a buying club";
    }
    if (
      outcome.saleFeeEur === null ||
      !Number.isInteger(outcome.saleFeeEur) ||
      outcome.saleFeeEur < 0
    ) {
      throw "Sale outcomes require a non-negative whole-euro fee";
    }
  }
  if (
    outcome?.status === "released" &&
    (outcome.buyingClub !== null || outcome.saleFeeEur !== null)
  ) {
    throw "Released outcomes cannot include sale details";
  }

  membersByClass.set(
    classId,
    members.map((candidate) =>
      candidate.playerUid === playerUid
        ? { ...candidate, outcome: outcome ? { ...outcome } : null }
        : candidate,
    ),
  );
}

function readNumberArg(args: unknown, name: string) {
  if (typeof args !== "object" || args === null) {
    return NaN;
  }
  const record = args as Record<string, unknown>;
  return typeof record[name] === "number" ? record[name] : NaN;
}

function readStringArg(args: unknown, name: string) {
  if (typeof args !== "object" || args === null) {
    return "";
  }
  const record = args as Record<string, unknown>;
  return typeof record[name] === "string" ? record[name] : "";
}

function readOutcomeArg(args: unknown): AcademyMember["outcome"] {
  if (typeof args !== "object" || args === null || !("outcome" in args)) {
    return null;
  }
  const outcome = (args as Record<string, unknown>).outcome;
  if (outcome === null) {
    return null;
  }
  if (typeof outcome !== "object" || outcome === null) {
    throw "Academy outcome is invalid";
  }
  const record = outcome as Record<string, unknown>;
  if (record.status !== "sold" && record.status !== "released") {
    throw "Academy outcome is invalid";
  }
  return {
    status: record.status,
    buyingClub:
      typeof record.buyingClub === "string" ? record.buyingClub : null,
    saleFeeEur:
      typeof record.saleFeeEur === "number" ? record.saleFeeEur : null,
  };
}

function updateClassMemberCount(classId: number, memberCount: number) {
  classes = classes.map((academyClass) =>
    academyClass.id === classId
      ? { ...academyClass, memberCount }
      : academyClass,
  );
}

function academyMemberFromCandidate(
  candidate: AcademyCandidate,
): AcademyMember {
  return {
    playerUid: candidate.playerUid,
    lastKnownName: candidate.name,
    currentName: candidate.name,
    state: "resolved",
    age: candidate.age,
    nationalities: [],
    positions: candidate.positions,
    currentClub: candidate.currentClub,
    parentClub: null,
    teamLevel: null,
    pa: null,
    determination: null,
    heightCm: null,
    preferredFoot: null,
    reportedCareerAppearances: null,
    goals: null,
    assists: null,
    internationalCaps: null,
    outcome: null,
    isGraduate: null,
  };
}
