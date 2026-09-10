export const graphicsKeys = {
  all: ["graphics"] as const,
  status: () => [...graphicsKeys.all, "status"] as const,
  results: () => [...graphicsKeys.all, "result"] as const,
  result: (generation: number, kind: string, uid: number) =>
    [...graphicsKeys.results(), { generation, kind, uid }] as const,
};
