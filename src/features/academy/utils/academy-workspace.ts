import { ACADEMY_VIEWS, type AcademyView } from "../types/academy";

export function parseAcademyView(value: unknown): AcademyView {
  return typeof value === "string" &&
    (ACADEMY_VIEWS as readonly string[]).includes(value)
    ? (value as AcademyView)
    : "overview";
}

export function parseAcademyClassId(value: unknown): number | null {
  const classId = typeof value === "number" ? value : Number(value);
  return Number.isInteger(classId) && classId > 0 ? classId : null;
}

export function snapshotYear(gameDate: string | null): number | null {
  if (!gameDate) {
    return null;
  }
  const year = Number(gameDate.slice(0, 4));
  return Number.isInteger(year) && year > 0 ? year : null;
}
