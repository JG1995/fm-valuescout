export const bridgeInstallKeys = {
  all: ["memory-read", "bridge-install"] as const,
  status: () => [...bridgeInstallKeys.all, "status"] as const,
};
