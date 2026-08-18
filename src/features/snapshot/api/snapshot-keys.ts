export const snapshotKeys = {
  all: ["snapshot"] as const,
  saves: () => [...snapshotKeys.all, "saves"] as const,
  current: () => [...snapshotKeys.all, "current"] as const,
  history: (saveId: number) =>
    [...snapshotKeys.all, "history", saveId] as const,
};
