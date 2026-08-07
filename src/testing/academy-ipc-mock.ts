import type { AcademyClass } from "@/features/academy/types/academy";

let classes: AcademyClass[] = [];
let nextClassId = 1;
let createError: string | null = null;
let deleteError: string | null = null;

export function resetAcademyIpcMock() {
  classes = [];
  nextClassId = 1;
  createError = null;
  deleteError = null;
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
}
