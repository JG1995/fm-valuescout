export const healthKeys = {
  all: ["health"] as const,
  status: () => [...healthKeys.all, "status"] as const,
  demoValue: () => [...healthKeys.all, "demo-value"] as const,
};
