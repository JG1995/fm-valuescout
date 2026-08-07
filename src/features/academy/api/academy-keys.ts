export const academyKeys = {
  all: ["academy"] as const,
  classes: () => [...academyKeys.all, "classes"] as const,
};
