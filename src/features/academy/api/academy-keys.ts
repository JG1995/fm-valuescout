export const academyKeys = {
  all: ["academy"] as const,
  classes: () => [...academyKeys.all, "classes"] as const,
  academyClass: (classId: number) =>
    [...academyKeys.all, "class", classId] as const,
  candidates: () => [...academyKeys.all, "candidates"] as const,
  candidate: (search: string) => [...academyKeys.candidates(), search] as const,
};
