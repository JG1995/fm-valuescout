export type AcademyClass = {
  id: number;
  classYear: number;
  memberCount: number;
};

export const ACADEMY_VIEWS = ["overview", "graduates", "class"] as const;
export type AcademyView = (typeof ACADEMY_VIEWS)[number];
