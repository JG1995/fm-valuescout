export const bridgeStatusKeys = {
  all: ["memory-read", "bridge-status"] as const,
  status: () => [...bridgeStatusKeys.all, "status"] as const,
};
