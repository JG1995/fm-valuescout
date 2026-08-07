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
let deferredAssignment: Promise<void> | null = null;
let resolveDeferredAssignment: (() => void) | null = null;
let deferredRemoval: Promise<void> | null = null;
let resolveDeferredRemoval: (() => void) | null = null;

export function resetAcademyIpcMock() {
  classes = [];
  nextClassId = 1;
  createError = null;
  deleteError = null;
  candidates = [];
  membersByClass = new Map();
  assignError = null;
  removeError = null;
  deferredAssignment = null;
  resolveDeferredAssignment = null;
  deferredRemoval = null;
  resolveDeferredRemoval = null;
}

export function setAcademyClasses(value: AcademyClass[]) {
  classes = value.map((academyClass) => ({ ...academyClass }));
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
    members.map((member) => ({ ...member })),
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

export function deferAcademyRemoval() {
  deferredRemoval = new Promise<void>((resolve) => {
    resolveDeferredRemoval = resolve;
  });
  return () => resolveDeferredRemoval?.();
}

export function resolveListAcademyClassesIpcMock() {
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
    memberCount: 0,
  };
  nextClassId += 1;
  classes = [...classes, created].sort(
    (left, right) => right.classYear - left.classYear,
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
    seniorLeagueAppearances: null,
    goals: null,
    assists: null,
    internationalCaps: null,
    saleFeeGbp: null,
    isReleased: null,
    isGraduate: null,
  };
}
