export const graphicsKeys = {
  all: ["graphics"] as const,
  status: () => [...graphicsKeys.all, "status"] as const,
};
