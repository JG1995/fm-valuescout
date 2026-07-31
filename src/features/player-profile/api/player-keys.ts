export const playerKeys = {
  all: ["player"] as const,
  detail: (uid: number) => [...playerKeys.all, uid] as const,
};
