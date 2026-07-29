export const snapshotKeys = {
  all: ["snapshot"] as const,
  saves: () => [...snapshotKeys.all, "saves"] as const,
  current: () => [...snapshotKeys.all, "current"] as const,
  sanityPlayers: () => [...snapshotKeys.all, "sanity-players"] as const,
};
